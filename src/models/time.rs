use serde::{Deserialize, Serialize};
use std::fmt;

/// Represent an instant the day, from 00:00 to 23:59
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Time {
    hours: u8,
    minutes: u8,
}

impl Time {
    pub fn new(hours: u8, minutes: u8) -> Self {
        assert!(hours < 24);
        assert!(minutes < 60);
        Time { hours, minutes }
    }

    pub fn hours(self) -> u8 {
        self.hours
    }

    pub fn minutes(self) -> u8 {
        self.minutes
    }

    pub fn all_times() -> impl Iterator<Item = Time> {
        (0..24).flat_map(|hours| (0..60).map(move |minutes| Time::new(hours, minutes)))
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hours, self.minutes)
    }
}
