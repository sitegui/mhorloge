use crate::models::texts::{TextTag, Texts};
use crate::models::token_graph::Token;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct PhraseSpec {
    id: u16,
    words: Vec<TextTag>,
}

#[derive(Debug, Clone)]
pub struct Phrase {
    words: Vec<Arc<Token>>,
}

impl PhraseSpec {
    pub fn new(texts: &mut Texts, id: u16, phrase: &str) -> Self {
        let words = phrase.split(' ').map(|text| texts.encode(text)).collect();
        PhraseSpec { id, words }
    }

    pub fn words(&self) -> &[TextTag] {
        &self.words
    }
}

impl Phrase {
    pub fn new(words: Vec<Arc<Token>>) -> Self {
        Phrase { words }
    }

    pub fn words(&self) -> &[Arc<Token>] {
        &self.words
    }
}
