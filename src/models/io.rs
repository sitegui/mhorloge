use crate::models::letter::Letter;
use crate::models::phrase::TimePhrase;
use crate::models::text::Text;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePhrasesOutput {
    pub phrases: Vec<TimePhrase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridInput {
    pub phrases: Vec<GridInputPhrase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridInputPhrase {
    pub texts: Vec<Text>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridOutput {
    pub minimal_grid: Vec<Vec<Option<Letter>>>,
    pub grid: Vec<Vec<Letter>>,
    pub phrases: Vec<GridOutputPhrase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridOutputPhrase {
    pub words: Vec<GridOutputWord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridOutputWord {
    pub letters: Vec<(i16, i16)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsPuzzleInput {
    pub video_id: String,
    pub total_duration: i32,
    pub phrases: Vec<LyricsPhrase>,
}

/// Represents each phrase in the lyrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsPhrase {
    pub texts: Vec<Text>,
    pub start: i32,
    pub end: i32,
}
