use std::convert::TryInto;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::models::language::Language;
use crate::models::phrase::Phrase;
use crate::models::texts::Texts;
use crate::models::time::Time;

pub mod english;
pub mod french;
pub mod portuguese;

#[derive(Debug, Clone, Serialize, Deserialize, Copy, Ord, PartialOrd, Eq, PartialEq)]
#[serde(transparent)]
pub struct PhraseId(pub u16);

pub fn generate_phrases(languages_spec: &str) -> Result<(Texts, Vec<Phrase>)> {
    let mut phrases = vec![];
    let mut texts = Texts::new();

    for mut language_tag in languages_spec.split(',') {
        let precision;
        match language_tag.find(':') {
            None => precision = 1,
            Some(pos) => {
                precision = language_tag[pos + 1..].parse()?;
                language_tag = &language_tag[..pos];
            }
        }

        let language: Language = language_tag.parse()?;

        for time in Time::all_times().step_by(precision as usize) {
            let next_id = PhraseId(phrases.len().try_into()?);
            let words = texts.encode_words(&language.spell(time));
            phrases.push(Phrase::new(next_id, language, time, words));
        }
    }

    Ok((texts, phrases))
}
