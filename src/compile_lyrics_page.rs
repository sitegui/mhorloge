mod keyframes;

use std::collections::BTreeMap;
use std::fmt;

use anyhow::{ensure, Result};
use itertools::Itertools;

use crate::build_grid::compile_html::compile_grid;
use crate::compile_lyrics_page::keyframes::{extract_frames, Keyframes};
use crate::{GridOutput, LyricsPuzzleInput};

/// Configure the animation curve timings. Measurements are in `ms`
#[derive(Debug, Clone, Copy)]
pub struct AnimationConfig {
    pub ease_in: i32,
    pub margin_before: i32,
    pub margin_after: i32,
    pub ease_out: i32,
    /// The ratio of the phrase duration that is dedicated to animate the letter as a incoming wave
    pub letters_entering: f64,
    pub discrete_time_step: i32,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
struct Animation {
    start_ease_in: i32,
    end_ease_in: i32,
    start_ease_out: i32,
    end_ease_out: i32,
}

#[derive(Debug, Clone)]
struct LettersAnimation {
    letters: Vec<(i16, i16)>,
    total_duration: i32,
    keyframes: Keyframes,
}

pub fn compile_lyrics_page(
    phrases: &LyricsPuzzleInput,
    grid: &GridOutput,
    config: AnimationConfig,
) -> Result<String> {
    let page = include_str!("compile_lyrics_page/template.html");
    let page = page.replacen("${STYLE}", &compile_css(phrases, grid, config)?, 1);
    let page = page.replacen("${GRID}", &compile_grid(grid), 1);
    let page = page.replacen("${VIDEO_ID}", &phrases.video_id, 1);

    Ok(page)
}

fn compile_css(
    phrases: &LyricsPuzzleInput,
    grid: &GridOutput,
    config: AnimationConfig,
) -> Result<String> {
    // Schedule each letter in time
    let mut timelines_per_letter = BTreeMap::new();
    ensure!(phrases.phrases.len() == grid.phrases.len());
    for (lyrics_phrase, grid_phrase) in phrases.phrases.iter().zip(&grid.phrases) {
        ensure!(lyrics_phrase.texts.len() == grid_phrase.words.len());
        let end_ease_in = lyrics_phrase.start - config.margin_before;
        let start_ease_out = lyrics_phrase.end + config.margin_after;
        let entering_duration =
            ((start_ease_out - end_ease_in) as f64 * config.letters_entering).floor();

        let letters = grid_phrase.words.iter().flat_map(|word| &word.letters);
        let entering_step = entering_duration / (letters.clone().count() - 1) as f64;

        for (i, &letter) in letters.enumerate() {
            let end_ease_in = end_ease_in + (i as f64 * entering_step) as i32;
            timelines_per_letter
                .entry(letter)
                .or_insert_with(Vec::new)
                .push(Animation {
                    start_ease_in: end_ease_in - config.ease_in,
                    end_ease_in,
                    start_ease_out,
                    end_ease_out: start_ease_out + config.ease_out,
                });
        }
    }
    log::info!(
        "Extracted {} timelines",
        timelines_per_letter
            .values()
            .map(|timeline| timeline.len())
            .sum::<usize>()
    );

    // Detect unique timelines
    let mut letters_by_timeline = BTreeMap::new();
    for (letter, mut timeline) in timelines_per_letter {
        timeline.sort();
        letters_by_timeline
            .entry(timeline)
            .or_insert_with(Vec::new)
            .push(letter);
    }
    log::info!("Extracted {} unique timelines", letters_by_timeline.len());

    let letter_animations = letters_by_timeline
        .into_iter()
        .enumerate()
        .map(|(i, (timeline, letters))| LettersAnimation {
            letters,
            total_duration: phrases.total_duration,
            keyframes: extract_frames(
                i as i32,
                phrases.total_duration,
                config.discrete_time_step,
                &timeline,
            ),
        })
        .collect_vec();

    Ok(letter_animations.into_iter().format("\n").to_string())
}

impl Animation {
    fn get(self, at: i32) -> f64 {
        fn interpolate(x1: i32, x2: i32, y1: f64, y2: f64, p: i32) -> f64 {
            y1 + (p - x1) as f64 / (x2 - x1) as f64 * (y2 - y1)
        }

        if at <= self.start_ease_in {
            0.0
        } else if at <= self.end_ease_in {
            interpolate(self.start_ease_in, self.end_ease_in, 0.0, 100.0, at)
        } else if at <= self.start_ease_out {
            100.0
        } else if at <= self.end_ease_out {
            interpolate(self.start_ease_out, self.end_ease_out, 100.0, 0.0, at)
        } else {
            0.0
        }
    }
}

impl fmt::Display for LettersAnimation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "{} {{animation: {}ms lyrics-timeline-{};}}",
            self.letters
                .iter()
                .format_with(", ", |letter, f| f(&format_args!(
                    ".letter-on-{}-{}",
                    letter.0, letter.1
                ))),
            self.total_duration,
            self.keyframes.id
        )?;

        writeln!(f, "{}", self.keyframes)
    }
}
