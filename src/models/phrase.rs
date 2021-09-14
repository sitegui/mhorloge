use crate::generate_phrases::PhraseId;
use crate::models::language::Language;
use crate::models::time::Time;
use crate::models::word::Word;

#[derive(Debug, Clone)]
pub struct Phrase {
    id: PhraseId,
    language: Language,
    time: Time,
    words: Vec<Word>,
}

impl Phrase {
    pub fn new(id: PhraseId, language: Language, time: Time, words: Vec<Word>) -> Self {
        Phrase {
            id,
            language,
            time,
            words,
        }
    }

    pub fn id(&self) -> PhraseId {
        self.id
    }

    pub fn words(&self) -> &[Word] {
        &self.words
    }
}
