use crate::models::letter::Letter;
use crate::models::phrase::TimePhrase;
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
    pub phrase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridOutput {
    pub minimal_grid: Vec<Vec<Option<Letter>>>,
    pub grid: Vec<Vec<Letter>>,
    pub phrases: Vec<GridOutputPhrase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridOutputPhrase {
    pub letters: Vec<(i16, i16)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsPhrasesInput {
    pub video_id: String,
    pub total_duration: i32,
    pub stops: Vec<WordOrSpace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WordOrSpace {
    Word {
        word: String,
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
    pub phrase: String,
    pub stops: Vec<LyricsPhraseStop>,
}

/// Represents each keyframe in the lyrics syncing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsPhraseStop {
    pub word_index: i32,
    pub time: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsKeyframesOutput {
    pub animations: Vec<LyricsAnimation>,
    pub animation_per_letter: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsAnimation {
    pub name: String,
    pub keyframes: Vec<LyricsKeyframe>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsKeyframe {
    pub percentage: f64,
}
