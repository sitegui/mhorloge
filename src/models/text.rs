use crate::models::letter::Letter;
use anyhow::{ensure, Error, Result};
use itertools::Itertools;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

/// Represents a non-empty sequence of letters
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Text {
    letters: Vec<Letter>,
}

impl Text {
    pub fn letters(&self) -> &[Letter] {
        &self.letters
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.letters.iter().format(""))
    }
}

impl FromStr for Text {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let letters = s
            .chars()
            .map(Letter::try_from)
            .collect::<Result<Vec<_>>>()?;
        ensure!(!letters.is_empty());
        Ok(Text { letters })
    }
}

impl Serialize for Text {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Text {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}
