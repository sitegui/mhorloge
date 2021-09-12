use crate::generate_phrases::PhraseId;
use crate::models::language::Language;
use crate::models::time::Time;
use crate::models::words::WordTag;

#[derive(Debug, Clone)]
pub struct Phrase {
    id: PhraseId,
    language: Language,
    time: Time,
    word_tags: Vec<WordTag>,
}

impl Phrase {
    pub fn new(id: PhraseId, language: Language, time: Time, word_tags: Vec<WordTag>) -> Self {
        Phrase {
            id,
            language,
            time,
            word_tags,
        }
    }

    pub fn id(&self) -> PhraseId {
        self.id
    }

    pub fn word_tags(&self) -> &[WordTag] {
        &self.word_tags
    }
}
