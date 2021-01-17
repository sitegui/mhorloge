use crate::models::phrase::{Phrase, PhraseSpec};
use crate::models::texts::Texts;
use crate::models::token_graph::{TokenGraph, TokenId};
use crate::optimizer::population::{PopulationOptimizer, Value, WeightedValue};
use itertools::Itertools;
use rand::rngs::SmallRng;
use rand::seq::{IteratorRandom, SliceRandom};
use rand::SeedableRng;

/// This is the first part of the problem: determine which tokens to consider, given a list of
/// phrases.
pub fn tokenize(
    texts: &Texts,
    phrases: &[PhraseSpec],
    max_actions: usize,
    max_values: usize,
    rng_seed: u64,
) -> Vec<Phrase> {
    let graph = TokenGraph::new(texts, phrases);
    log::info!("Starting point: {}", graph);
    let initial_len = graph.letters_len();
    let graph = WeightedGraph::new(graph, initial_len);

    let rng = SmallRng::seed_from_u64(rng_seed);

    let mut optimization = PopulationOptimizer::new(rng, vec![graph], max_actions, max_values);

    let mut prev_weight = None;
    let mut repeated = 0;
    let mut epoch = 0;
    loop {
        let best = optimization.best();
        if prev_weight == Some(best.weight()) {
            repeated += 1;
            if repeated == 5 {
                break;
            }
        } else {
            repeated = 0;
        }
        prev_weight = Some(best.weight());

        log::info!(
            "Start epoch {} with {} individuals. Best weight = {} with {} tokens",
            epoch,
            optimization.len(),
            best.weight(),
            best.tokens_len,
        );
        optimization.evolve();
        epoch += 1;
    }

    let best = optimization.into_best();
    log::info!(
        "Finished with weight = {} with {} tokens",
        best.weight(),
        best.tokens_len
    );
    log::info!("Result: {}", best.graph);
    best.graph.into_phrases()
}

#[derive(Debug, Clone)]
struct WeightedGraph<'a> {
    graph: TokenGraph<'a>,
    initial_letters: usize,
    letters_len: usize,
    tokens_len: usize,
    /// weight = (saved letters) ^ 2
    weight: f64,
}

#[derive(Debug, Clone, Copy)]
struct MergeTokens(TokenId, TokenId);

impl<'a> WeightedGraph<'a> {
    pub fn new(graph: TokenGraph<'a>, initial_letters: usize) -> Self {
        let letters_len = graph.letters_len();
        let saved_letters = (initial_letters - letters_len) as f64;
        WeightedGraph {
            initial_letters,
            letters_len,
            tokens_len: graph.tokens_len(),
            graph,
            weight: saved_letters * saved_letters,
        }
    }

    pub fn with_merged_tokens(&self, a: TokenId, b: TokenId) -> Self {
        let mut merged = self.graph.clone();
        merged.merge_tokens(a, b);
        WeightedGraph::new(merged, self.initial_letters)
    }
}

impl<'a> Value for WeightedGraph<'a> {
    fn evolve(&self, max_actions: usize, rng: &mut SmallRng) -> Vec<WeightedValue<Self>> {
        // Choose which text to target
        let texts = self.graph.tokens_by_text().keys().copied().collect_vec();
        let text = *texts.choose_weighted(rng, |text| text.len()).unwrap();
        log::debug!("Selected text {:?}", self.graph.texts().decode(text));

        let unmerged_tokens = self.graph.tokens_by_text()[&text]
            .iter()
            .filter(|&&token| !self.graph.get(token).is_merged());
        let a = *unmerged_tokens.clone().choose(rng).unwrap();

        let mut space = self.graph.dfs_space();
        let mut new_values = Vec::with_capacity(max_actions);
        for &b in unmerged_tokens {
            if self.graph.can_merge_tokens(a, b, &mut space) {
                let merged = WeightedValue::new(self.with_merged_tokens(a, b));
                new_values.push(merged);

                if new_values.len() == max_actions {
                    break;
                }
            }
        }
        new_values
    }

    fn weight(&self) -> f64 {
        self.weight
    }
}
