mod extract_segments;

use crate::{GridOutput, LyricsPhrasesOutput};
use itertools::Itertools;
use ordered_float::OrderedFloat;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

#[derive(Debug, Clone)]
struct MaybeScheduledWord {
    letters: Vec<(i16, i16)>,
    percentage: Option<f64>,
}

#[derive(Debug, Clone)]
struct ScheduledWord {
    letters: Vec<(i16, i16)>,
    percentage: f64,
    /// Measured in %/word
    speed: f64,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
struct LetterStop {
    percentage: OrderedFloat<f64>,
    /// Measured in %/word
    speed: OrderedFloat<f64>,
}

pub fn compile_css(phrases: &LyricsPhrasesOutput, grid: &GridOutput) -> String {
    let total_duration = phrases.total_duration as f64;

    let mut all_words = vec![];
    for (lyrics_phrase, grid_phrase) in phrases.phrases.iter().zip(&grid.phrases) {
        let mut letters = grid_phrase.words.iter().cloned();
        let mut scheduled_words = lyrics_phrase
            .phrase
            .split(' ')
            .map(|word| MaybeScheduledWord {
                letters: letters.by_ref().take(word.len()).collect(),
                percentage: None,
            })
            .collect_vec();

        for stop in &lyrics_phrase.stops {
            let scheduled = &mut scheduled_words[stop.word_index as usize];
            assert!(
                scheduled.percentage.is_none(),
                "A word can only have a single stop"
            );
            scheduled.percentage = Some(100.0 * stop.time as f64 / total_duration);
        }

        all_words.extend(scheduled_words);
    }

    // Make sure the endpoints are scheduled
    all_words[0].percentage.get_or_insert(0.0);
    let last_i = all_words.len() - 1;
    all_words[last_i].percentage.get_or_insert(100.0);

    // Interpolate to schedule all words
    let scheduled = all_words
        .iter()
        .enumerate()
        .filter_map(|(i, word)| word.percentage.map(|percentage| (i, percentage)))
        .collect_vec();

    log::info!(
        "Extracted info for {} words, {} scheduled words",
        all_words.len(),
        scheduled.len()
    );

    let mut all_scheduled_words = vec![];
    for ((before_i, before_percentage), (after_i, after_percentage)) in
        scheduled.into_iter().tuple_windows::<(_, _)>()
    {
        let dw = after_i - before_i;
        let dp = after_percentage - before_percentage;
        let speed = dp / dw as f64;

        all_scheduled_words.extend(all_words[before_i..after_i].iter().enumerate().map(
            |(i, word)| {
                ScheduledWord {
                    letters: word.letters.clone(),
                    percentage: word
                        .percentage
                        .unwrap_or(before_percentage + speed * i as f64),
                    speed,
                }
            },
        ));
    }

    // Add last word (set last speed base on previous state)
    let last_word = &all_words[all_words.len() - 1];
    all_scheduled_words.push(ScheduledWord {
        letters: last_word.letters.clone(),
        percentage: 100.0,
        speed: all_scheduled_words[all_scheduled_words.len() - 1].speed,
    });

    // Convert timing information to each letter
    let mut timeline_by_letter = BTreeMap::new();
    for word in all_scheduled_words {
        for letter in word.letters {
            timeline_by_letter
                .entry(letter)
                .or_insert_with(BTreeSet::new)
                .insert(LetterStop {
                    percentage: OrderedFloat(word.percentage),
                    speed: OrderedFloat(word.speed),
                });
        }
    }

    // Detect unique timelines
    let mut letters_by_timeline = BTreeMap::new();
    for (letter, timeline) in timeline_by_letter {
        letters_by_timeline
            .entry(timeline)
            .or_insert_with(Vec::new)
            .push(letter);
    }

    log::info!("Extracted {} unique timelines", letters_by_timeline.len());

    let mut source = String::new();

    // Write CSS rules to map letters to @keyframes
    let animation_duration = total_duration / 1e3;
    for (timeline_i, letters) in letters_by_timeline.values().enumerate() {
        writeln!(
            source,
            "{} {{",
            letters
                .iter()
                .format_with(", ", |letter, f| f(&format_args!(
                    ".letter-{}-{}",
                    letter.0, letter.1
                ))),
        )
        .unwrap();
        writeln!(
            source,
            "    animation: {}s lyrics-timeline-{};",
            animation_duration, timeline_i
        )
        .unwrap();
        writeln!(source, "}}").unwrap();
    }

    let config = AnimationConfig {
        ease_in: 100.0 * 100.0 / total_duration,
        before: 100.0 * 100.0 / total_duration,
        after: 1000.0 * 100.0 / total_duration,
        ease_out: 100.0 * 100.0 / total_duration,
        discrete_time_steps: 100.0 * 100.0 / total_duration,
    };

    // Write CSS rules for the @keyframes
    for (timeline_i, timeline) in letters_by_timeline.keys().enumerate() {
        writeln!(source, "@keyframes lyrics-timeline-{} {{", timeline_i).unwrap();
        for frame in compile_keyframes(config, timeline) {
            writeln!(source, "    {:.2}% {{", frame.time_percentage).unwrap();
            writeln!(
                source,
                "        opacity: {:.0}%; /* {:.1}s */",
                frame.effect_percentage,
                frame.time_percentage * total_duration / 100.0 / 1e3
            )
            .unwrap();
            writeln!(source, "    }}").unwrap();
        }
        writeln!(source, "}}").unwrap();
    }

    source
}

#[derive(Debug, Clone, Copy)]
struct Highlight {
    start: f64,
    start_on: f64,
    end_on: f64,
    end: f64,
}

#[derive(Debug, Clone, Copy)]
struct AnimationConfig {
    ease_in: f64,
    before: f64,
    after: f64,
    ease_out: f64,
    discrete_time_steps: f64,
}

#[derive(Debug, Clone, Copy)]
struct Keyframe {
    time_percentage: f64,
    effect_percentage: f64,
}

impl Highlight {
    fn new(config: AnimationConfig, stop: LetterStop) -> Self {
        let ease_in = config.ease_in;
        let before = config.before;
        let after = config.after;
        let ease_out = config.ease_out;

        let percentage = stop.percentage.into_inner();
        Self {
            start: (percentage - before - ease_in).clamp(0.0, 100.0),
            start_on: (percentage - before).clamp(0.0, 100.0),
            end_on: (percentage + after).clamp(0.0, 100.0),
            end: (percentage + after + ease_out).clamp(0.0, 100.0),
        }
    }

    fn get(self, at: f64) -> f64 {
        fn interpolate(x1: f64, x2: f64, y1: f64, y2: f64, p: f64) -> f64 {
            y1 + (p - x1) / (x2 - x1) * (y2 - y1)
        }

        if at <= self.start {
            0.0
        } else if at <= self.start_on {
            interpolate(self.start, self.start_on, 0.0, 100.0, at)
        } else if at <= self.end_on {
            100.0
        } else if at <= self.end {
            interpolate(self.end_on, self.end, 100.0, 0.0, at)
        } else {
            0.0
        }
    }
}

fn compile_keyframes(config: AnimationConfig, timeline: &BTreeSet<LetterStop>) -> Vec<Keyframe> {
    log::debug!("compile_keyframes {:?}", timeline);

    // Convert each stop to a interval of change that highlights the given letter.
    // Sort the highlights by start so that we can detect conflicts next
    let highlights = timeline
        .iter()
        .map(|&stop| Highlight::new(config, stop))
        .sorted_by_key(|highlight| OrderedFloat(highlight.start))
        .collect_vec();

    // Categorize the highlights based on whether they conflict with some other
    let mut conflicting_highlights = vec![];
    let mut non_conflicting_highlights = vec![];
    let mut latest_end = 0.0;
    for highlight in highlights {
        if highlight.start < latest_end {
            conflicting_highlights.push(highlight);
        } else {
            non_conflicting_highlights.push(highlight);
        }

        latest_end = latest_end.max(highlight.end);
    }

    let mut all_frames = compile_non_conflicting_highlights(&non_conflicting_highlights);
    all_frames.extend(compile_conflicting_highlights(
        config,
        &conflicting_highlights,
    ));

    all_frames
}

fn compile_non_conflicting_highlights(highlights: &[Highlight]) -> Vec<Keyframe> {
    let mut frames = vec![];

    for &highlight in highlights {
        frames.push(Keyframe {
            time_percentage: highlight.start,
            effect_percentage: 0.0,
        });
        frames.push(Keyframe {
            time_percentage: highlight.start_on,
            effect_percentage: 100.0,
        });
        frames.push(Keyframe {
            time_percentage: highlight.end_on,
            effect_percentage: 100.0,
        });
        frames.push(Keyframe {
            time_percentage: highlight.end,
            effect_percentage: 0.0,
        });
    }

    frames
}

fn compile_conflicting_highlights(
    config: AnimationConfig,
    highlights: &[Highlight],
) -> Vec<Keyframe> {
    let mut frames = BTreeMap::new();

    for &highlight in highlights {
        // Change to a discrete timeline
        let start = (highlight.start / config.discrete_time_steps).floor() as i32;
        let end = (highlight.end / config.discrete_time_steps).ceil() as i32;

        for i in start..=end {
            let entry = frames.entry(i).or_insert(0.0f64);
            *entry = (*entry).max(highlight.get(i as f64 * config.discrete_time_steps));
        }
    }

    frames
        .into_iter()
        .map(|(i, effect_percentage)| Keyframe {
            time_percentage: i as f64 * config.discrete_time_steps,
            effect_percentage,
        })
        .collect_vec()
}
