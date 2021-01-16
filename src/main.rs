use crate::languages::portuguese::Portuguese;
use crate::languages::Language;
use crate::models::phrase::PhraseSpec;
use crate::models::texts::Texts;
use crate::models::time::Time;
use crate::models::token_graph::TokenGraph;

mod languages;
mod models;
mod optimizer;
mod tokenize;

fn main() {
    let (texts, phrases) = phrases(&[Box::new(Portuguese)]);
    let graph = TokenGraph::new(&phrases);
    let phrases = graph.into_phrases();
    eprintln!("phrases = {:?}", phrases);
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
