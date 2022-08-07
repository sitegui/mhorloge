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
#[serde(transparent)]
pub struct LyricsPhrasesInput(pub Vec<WordOrSpace>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WordOrSpace {
    Word {
        word: String,
        #[serde(default)]
        times: Vec<f64>,
    },
    Space(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsPhrasesOutput {
    pub phrases: Vec<LyricsPhrase>,
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
    pub word_index: u8,
    pub time_ms: i32,
}
