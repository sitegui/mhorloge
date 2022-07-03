use anyhow::Result;

use crate::models::language::Language;
use crate::models::phrase_book::PhraseBook;
use crate::models::time::Time;

pub mod english;
pub mod french;
pub mod german;
pub mod portuguese;

pub fn generate_phrases(languages_spec: &str) -> Result<PhraseBook> {
    let mut phrases = PhraseBook::default();

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

        let gui = Time::new(21, 55);
        println!("{}", language.spell(gui));

        for time in Time::all_times().step_by(precision as usize) {
            phrases.insert_phrase(language, time, &language.spell(time));
        }
    }

    Ok(phrases)
}
