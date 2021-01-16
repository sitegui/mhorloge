use crate::models::phrase::{Phrase, PhraseSpec};
use crate::models::token_graph::{TokenGraph, TokenId};
use crate::optimizer::population::{Individual, PopulationOptimizer};
use crate::optimizer::selector::Selector;
use itertools::Itertools;
use rand::rngs::SmallRng;
use rand::SeedableRng;

/// This is the first part of the problem: determine which tokens to consider, given a list of
/// phrases.
pub fn tokenize(
    phrases: &[PhraseSpec],
    max_actions: usize,
    target_population: usize,
    rng_seed: u64,
    epochs: usize,
) -> Vec<Phrase> {
    let graph = TokenGraph::new(phrases);
    let initial_len = graph.letters_len();
    let graph = ScoredGraph::new(graph, initial_len);

    let rng = SmallRng::seed_from_u64(rng_seed);
    let mut optimization = PopulationOptimizer::new(
        vec![graph],
        Selector::new(max_actions, target_population, rng),
    );

    for epoch in 0..epochs {
        let best = optimization.best();
        log::info!(
            "Start epoch {} with {} individuals. Best score = {} with {} tokens",
            epoch,
            optimization.population().len(),
            best.score(),
            best.tokens_len,
        );
        optimization.evolve();
    }

    let best = optimization.into_best();
    log::info!(
        "Finished with score = {} with {} tokens",
        best.score(),
        best.tokens_len
    );
    best.graph.into_phrases()
}

#[derive(Debug, Clone)]
struct ScoredGraph<'a> {
    graph: TokenGraph<'a>,
    initial_letters: usize,
    letters_len: usize,
    tokens_len: usize,
}

#[derive(Debug, Clone, Copy)]
struct MergeTokens(TokenId, TokenId);

impl<'a> ScoredGraph<'a> {
    pub fn new(graph: TokenGraph<'a>, initial_letters: usize) -> Self {
        ScoredGraph {
            initial_letters,
            letters_len: graph.letters_len(),
            tokens_len: graph.tokens_len(),
            graph,
        }
    }
}

impl<'a> Individual for ScoredGraph<'a> {
    type Action = MergeTokens;
    type Score = u32;

    fn possible_actions(&self) -> Vec<Self::Action> {
        let mut actions = vec![];

        for tokens in self.graph.tokens_by_text().values() {
            let unmerged_tokens = tokens
                .iter()
                .filter(|&&token| !self.graph.get(token).is_merged());
            for (&a, &b) in unmerged_tokens.tuple_combinations::<(_, _)>() {
                actions.push(MergeTokens(a, b));
            }
        }

        actions
    }

    fn evolve(&self, action: Self::Action) -> Self {
        let mut merged = self.graph.clone();
        merged.merge_tokens(action.0, action.1);
        ScoredGraph::new(merged, self.initial_letters)
    }

    fn score(&self) -> Self::Score {
        (self.initial_letters - self.letters_len) as u32
    }
}
