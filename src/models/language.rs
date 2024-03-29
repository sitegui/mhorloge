use crate::models::time::Time;

use crate::generate_phrases::{english, french, german, portuguese};
use crate::models::text::Text;
use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Represents a possible language, that can spell out any valid time
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Language {
    English,
    French,
    Portuguese,
    German,
}

impl FromStr for Language {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "English" => Ok(Language::English),
            "French" => Ok(Language::French),
            "Portuguese" => Ok(Language::Portuguese),
            "German" => Ok(Language::German),
            _ => Err(anyhow!("Language was not recognized: {}", s)),
        }
    }
}

impl Language {
    pub fn spell(self, time: Time) -> Vec<Text> {
        let phrase = match self {
            Language::English => english::spell(time),
            Language::French => french::spell(time),
            Language::Portuguese => portuguese::spell(time),
            Language::German => german::spell(time),
        };

        phrase
            .split(' ')
            .map(|word| word.parse().expect("Valid Text"))
            .collect()
    }
}
