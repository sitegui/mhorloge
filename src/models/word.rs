use crate::models::merge_dag::Node;
use crate::models::text::Text;
use std::fmt;

/// Represents a word in a specific phrase
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Word {
    pub id: WordId,
    pub text: Text,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct WordId(pub u16);

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl Node for &'_ Word {
    type Id = WordId;

    fn id(&self) -> Self::Id {
        self.id
    }
}
