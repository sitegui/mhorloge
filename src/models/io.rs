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
