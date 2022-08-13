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
pub struct LyricsPhrasesInput {
    pub video_id: String,
    pub total_duration: i32,
    pub elements: Vec<WordOrSpace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WordOrSpace {
    Word {
        text: Text,
        #[serde(default)]
        times: Vec<i32>,
    },
    Space(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsPhrasesOutput {
    pub phrases: Vec<LyricsPhrase>,
    pub total_duration: i32,
}

/// Represents each phrase in the lyrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsPhrase {
    pub words: Vec<LyricsWord>,
}

/// Represents each word in the lyrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsWord {
    pub text: Text,
    /// The position in time (0=start, 1=end) of this word in the whole song
    pub stop: Option<f64>,
}
