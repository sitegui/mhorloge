use crate::models::language::Language;
use crate::models::time::Time;
use crate::models::word::WordId;

/// Represents a phrase that describes a time in a given language
#[derive(Debug, Clone)]
pub struct Phrase {
    pub id: PhraseId,
    pub language: Language,
    pub time: Time,
    pub words: Vec<WordId>,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhraseId(pub u16);
