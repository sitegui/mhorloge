use crate::models::time::Time;

use crate::generate_phrases::{english, french, portuguese};
use crate::models::word::Word;
use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::str::FromStr;

/// Represents a possible language, that can spell out any valid time
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Language {
    English,
    French,
    Portuguese,
}

impl FromStr for Language {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "English" => Ok(Language::English),
            "French" => Ok(Language::French),
            "Portuguese" => Ok(Language::Portuguese),
            _ => Err(anyhow!("Language was not recognized: {}", s)),
        }
    }
}

impl Language {
    pub fn spell(self, time: Time) -> Vec<Word> {
        let text = match self {
            Language::English => english::spell(time),
            Language::French => french::spell(time),
            Language::Portuguese => portuguese::spell(time),
        };

        text.split(' ')
            .map(|word| Word::try_from(word).unwrap())
            .collect()
    }
}
