use crate::models::language::Language;
use crate::models::phrase::TimePhrase;
use crate::models::time::Time;

pub mod english;
pub mod french;
pub mod german;
pub mod portuguese;

pub fn generate_phrases(language_specs: &[(Language, i32)]) -> Vec<TimePhrase> {
    let mut phrases = vec![];

    for &(language, precision) in language_specs {
        for time in Time::all_times().step_by(precision as usize) {
            phrases.push(TimePhrase {
                language,
                time,
                texts: language.spell(time),
            });
        }
    }

    phrases
}
