use crate::models::letter::Letter;
use itertools::Itertools;
use std::convert::TryFrom;
use std::fmt;

/// Represents a non-empty sequence of letters
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Text {
    letters: Vec<Letter>,
}

impl Text {
    pub fn new(text: &str) -> Self {
        let letters: Vec<_> = text.chars().map(|c| Letter::try_from(c).unwrap()).collect();
        assert!(!letters.is_empty());
        Text { letters }
    }

    pub fn letters(&self) -> &[Letter] {
        &self.letters
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.letters.iter().format(""))
    }
}
