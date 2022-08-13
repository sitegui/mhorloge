use crate::compile_lyrics_page::MaybeScheduledWord;
use crate::models::text::Text;
use crate::{grid, GridOutput, LyricsPhrasesOutput};
use anyhow::{ensure, Result};
use itertools::Itertools;

/// Represents a sequence of at least two words. The first and the last ones are "timed", that is,
/// they are attached to a point in time.
#[derive(Debug, Clone)]
pub enum FlowSegment {
    Single(TimedWord),
    Multiple {
        first: TimedWord,
        others: Vec<FlowWord>,
        last: TimedWord,
    },
}

#[derive(Debug, Clone)]
pub enum FlowWord {
    Timed(TimedWord),
    Untimed(UntimedWord),
}

#[derive(Debug, Clone)]
pub struct TimedWord {
    text: Text,
    letters: Vec<(i16, i16)>,
    stop: f64,
}

#[derive(Debug, Clone)]
pub struct UntimedWord {
    text: Text,
    letters: Vec<(i16, i16)>,
}

fn extract_segments(phrases: &LyricsPhrasesOutput, grid: &GridOutput) -> Result<Vec<FlowSegment>> {
    let mut words = extract_flow_words(phrases, grid)?;

    ensure!(words.len() >= 2);

    let mut segments = vec![];
    let mut words = words.into_iter();
    let mut segment =
        FlowSegment::Single(words.next().expect("at least 2 words").ensure_timed(0.0));
    for word in words {
        match word {
            FlowWord::Timed(timed) => match segment {
                FlowSegment::Single(first) => {
                    segment = FlowSegment::Multiple {
                        first,
                        others: vec![],
                        last: timed,
                    }
                }
                FlowSegment::Multiple {
                    first,
                    others,
                    last,
                } => {}
            },
            FlowWord::Untimed(_) => {}
        }
    }

    Ok(segments)
}

fn extract_flow_words(phrases: &LyricsPhrasesOutput, grid: &GridOutput) -> Result<Vec<FlowWord>> {
    let mut words = vec![];

    ensure!(phrases.phrases.len() == grid.phrases.len());
    for (lyrics_phrase, grid_phrase) in phrases.phrases.iter().zip(&grid.phrases) {
        ensure!(lyrics_phrase.words.len() == grid_phrase.words.len());
        for (lyrics_word, grid_word) in lyrics_phrase.words.iter().zip(&grid_phrase.words) {
            let text = lyrics_word.text.clone();
            let letters = grid_word.letters.clone();
            ensure!(text.letters().len() == letters.len());

            let word = match lyrics_word.stop {
                None => FlowWord::Untimed(UntimedWord { text, letters }),
                Some(stop) => FlowWord::Timed(TimedWord {
                    text,
                    letters,
                    stop,
                }),
            };
            words.push(word);
        }
    }

    Ok(words)
}

impl FlowWord {
    fn ensure_timed(self, stop: f64) -> TimedWord {
        match self {
            FlowWord::Timed(timed) => timed,
            FlowWord::Untimed(untimed) => TimedWord {
                text: untimed.text,
                letters: untimed.letters,
                stop,
            },
        }
    }
}
