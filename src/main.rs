use crate::languages::portuguese::Portuguese;
use crate::languages::Language;
use crate::models::phrase::PhraseSpec;
use crate::models::texts::Texts;
use crate::models::time::Time;

mod languages;
mod models;
mod optimizer;
mod tokenize;

fn main() {
    env_logger::init();
    log::info!("Starting");

    let (texts, phrases) = phrases(&[Box::new(Portuguese)]);
    let phrases = tokenize::tokenize(&phrases, 5, 1, 17, 1_000);
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
