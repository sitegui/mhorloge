use crate::generate_phrases::PhraseId;
use crate::models::language::Language;
use crate::models::texts::TextTag;
use crate::models::time::Time;

#[derive(Debug, Clone)]
pub struct Phrase {
    id: PhraseId,
    language: Language,
    time: Time,
    words: Vec<TextTag>,
}

impl Phrase {
    pub fn new(id: PhraseId, language: Language, time: Time, words: Vec<TextTag>) -> Self {
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

    pub fn words(&self) -> &[TextTag] {
        &self.words
    }
}
