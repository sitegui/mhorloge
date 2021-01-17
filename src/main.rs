use crate::languages::portuguese::Portuguese;
use crate::languages::Language;
use crate::models::phrase::PhraseSpec;
use crate::models::texts::Texts;
use crate::models::time::Time;
use crate::tokenize::Schedule;

mod languages;
mod models;
mod optimizer;
mod tokenize;

fn main() {
    env_logger::init();
    log::info!("Starting");

    let (texts, phrases) = phrases(&[Box::new(Portuguese)]);
    let phrases = tokenize::tokenize(
        &texts,
        &phrases[..3 * 60],
        &[
            Schedule {
                max_actions: 3,
                max_values: 10,
                patience: 5,
            },
            Schedule {
                max_actions: 5,
                max_values: 100,
                patience: 10,
            },
            Schedule {
                max_actions: 10,
                max_values: 1000,
                patience: 20,
            },
            Schedule {
                max_actions: 100,
                max_values: 10000,
                patience: 200,
            },
        ],
        17,
    );
}

fn phrases(languages: &[Box<dyn Language>]) -> (Texts, Vec<PhraseSpec>) {
    let mut texts = Texts::new();
    let mut phrases = vec![];

    for language in languages {
        for time in Time::all_times() {
            phrases.push(PhraseSpec::new(&mut texts, &language.spell(time)));
        }
    }

    (texts, phrases)
}
