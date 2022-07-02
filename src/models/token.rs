use crate::models::text::Text;
use crate::models::word::{Word, WordId};
use std::fmt;

/// Represents a text drawn in the grid, which can be shared between multiple words.
#[derive(Debug, Clone)]
pub struct Token {
    pub id: TokenId,
    pub text: Text,
    pub words: Vec<WordId>,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct TokenId(pub u16);

impl Token {
    pub fn new(word: &Word) -> Self {
        Token {
            id: TokenId(word.id.0),
            text: word.text.clone(),
            words: vec![word.id],
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}
