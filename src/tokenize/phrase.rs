use crate::generate_phrases::PhraseId;
use crate::tokenize::texts::{TextTag, Texts};

#[derive(Debug, Clone)]
pub struct PhraseSpec {
    id: PhraseId,
    words: Vec<TextTag>,
}

impl PhraseSpec {
    pub fn new(texts: &mut Texts, id: PhraseId, phrase: &str) -> Self {
        let words = phrase.split(' ').map(|text| texts.encode(text)).collect();
        PhraseSpec { id, words }
    }

    pub fn words(&self) -> &[TextTag] {
        &self.words
    }

    pub fn id(&self) -> PhraseId {
        self.id
    }
}
