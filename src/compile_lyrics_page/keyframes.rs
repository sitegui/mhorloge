use crate::compile_lyrics_page::Animation;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Keyframes {
    pub id: i32,
    frames: Vec<Keyframe>,
}

#[derive(Debug, Clone, Copy)]
struct Keyframe {
    time_percentage: f64,
    effect_percentage: f64,
}

pub(super) fn extract_frames(
    id: i32,
    total_duration: i32,
    discrete_time_step: i32,
    timeline: &[Animation],
) -> Keyframes {
    log::debug!("extract_frames id={} timeline={:?}", id, timeline);

    // Categorize the animations based on whether they conflict with some other
    let mut latest_end_ease_out = 0;
    let mut is_conflicting = vec![false; timeline.len()];
    for (i, animation) in timeline.iter().enumerate() {
        if animation.start_ease_in < latest_end_ease_out {
            // Set the current and the previous animation to conflict
            is_conflicting[i] = true;
            is_conflicting[i - 1] = true;
        }

        latest_end_ease_out = latest_end_ease_out.max(animation.end_ease_out);
    }

    let mut conflicting_animations = vec![];
    let mut non_conflicting_animations = vec![];
    for (&animation, is_conflicting) in timeline.iter().zip(is_conflicting) {
        if is_conflicting {
            conflicting_animations.push(animation);
        } else {
            non_conflicting_animations.push(animation);
        }
    }

    let mut frames = extract_non_conflicting_frames(total_duration, &non_conflicting_animations);
    frames.extend(extract_conflicting_frames(
        total_duration,
        discrete_time_step,
        &conflicting_animations,
    ));

    Keyframes { id, frames }
}

fn extract_non_conflicting_frames(total_duration: i32, animations: &[Animation]) -> Vec<Keyframe> {
    let mut frames = vec![];

    for &animation in animations {
        frames.push(Keyframe::new(total_duration, animation.start_ease_in, 0.0));
        frames.push(Keyframe::new(total_duration, animation.end_ease_in, 100.0));
        frames.push(Keyframe::new(
            total_duration,
            animation.start_ease_out,
            100.0,
        ));
        frames.push(Keyframe::new(total_duration, animation.end_ease_out, 0.0));
    }

    frames
}

fn extract_conflicting_frames(
    total_duration: i32,
    discrete_time_step: i32,
    animations: &[Animation],
) -> Vec<Keyframe> {
    log::debug!("extract_conflicting_frames {:?}", animations);

    let mut frames = BTreeMap::new();

    for &animation in animations {
        // Change to a discrete timeline
        let start = animation.start_ease_in / discrete_time_step;
        let end = (animation.end_ease_out + discrete_time_step - 1) / discrete_time_step;

        for i in start..=end {
            let entry = frames.entry(i).or_insert(0.0f64);
            *entry = (*entry).max(animation.get(i * discrete_time_step));
        }
    }

    let x = frames
        .into_iter()
        .map(|(i, effect_percentage)| {
            Keyframe::new(total_duration, i * discrete_time_step, effect_percentage)
        })
        .collect_vec();
    log::debug!("{:?}", x);
    x
}

impl Keyframe {
    fn new(total_duration: i32, time: i32, effect_percentage: f64) -> Self {
        Self {
            time_percentage: 100.0 * time as f64 / total_duration as f64,
            effect_percentage,
        }
    }
}

impl fmt::Display for Keyframe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:.2}% {{opacity: {:.0}%;}}",
            self.time_percentage, self.effect_percentage
        )
    }
}

impl fmt::Display for Keyframes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "@keyframes lyrics-timeline-{} {{", self.id)?;
        for frame in &self.frames {
            writeln!(f, "{}", frame)?;
        }
        write!(f, "}}")
    }
}
