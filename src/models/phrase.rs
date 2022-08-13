use crate::models::language::Language;
use crate::models::text::Text;
use crate::models::time::Time;
use crate::models::word::WordId;
use serde::{Deserialize, Serialize};

/// Represents a phrase that describes a time in a given language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePhrase {
    pub language: Language,
    #[serde(flatten)]
    pub time: Time,
    pub texts: Vec<Text>,
}

/// Represents a phrase
#[derive(Debug, Clone)]
pub struct Phrase {
    pub id: PhraseId,
    pub words: Vec<WordId>,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhraseId(pub u16);
