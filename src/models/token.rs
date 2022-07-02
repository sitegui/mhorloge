use crate::models::text::Text;
use crate::models::word::WordId;
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

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}
