use std::convert::TryInto;
use std::path::PathBuf;

use anyhow::Error;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use time::Time;

use crate::generate_phrases::language::Language;
use crate::utils::create_file;

mod english;
mod french;
mod language;
mod portuguese;
pub mod time;

/// Generate the time phrases for the given languages
#[derive(Debug, StructOpt)]
pub struct GeneratePhrases {
    /// Determine the languages to use. Available languages are: "English", "French" and
    /// "Portuguese". Multiple languages can be requested by separating them by comma. By
    /// default, all time phrases will be generated, that is, from 00:00 to 23:59 with 1-minute
    /// precision. To change the precision, append ":" followed by an integer representing the
    /// desired precision after each language name. Each language can determine their own
    /// precision.
    ///
    /// Full example: "English:5,French" will generate for both languages, using a 1-minute
    /// precision for French and 5-minute precision for English.
    languages: String,
    /// The output JSON file.
    output: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratePhrasesOut {
    pub phrases: Vec<GeneratePhrasesOutEl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratePhrasesOutEl {
    pub id: PhraseId,
    pub language: Language,
    pub time: Time,
    pub phrase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
#[serde(transparent)]
pub struct PhraseId(u16);

pub fn generate_phrases(cmd: GeneratePhrases) -> Result<(), Error> {
    let output = create_file(cmd.output)?;

    let mut phrases = vec![];
    for mut language_tag in cmd.languages.split(',') {
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
            phrases.push(GeneratePhrasesOutEl {
                id: next_id,
                language,
                time,
                phrase: language.spell(time),
            });
        }
    }

    serde_json::to_writer_pretty(output, &GeneratePhrasesOut { phrases })?;

    Ok(())
}
