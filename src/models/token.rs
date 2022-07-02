use crate::models::merge_dag::{Group, Node};
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

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl Group<&'_ Word> for Token {
    fn new(word: &Word) -> Self {
        Token {
            id: TokenId(word.id.0),
            text: word.text.clone(),
            words: vec![word.id],
        }
    }

    fn merge(&mut self, other: Self) {
        assert_eq!(self.text, other.text);
        self.words.extend(other.words);
    }
}

impl Node for &'_ Token {
    type Id = TokenId;

    fn id(&self) -> Self::Id {
        self.id
    }
}
