use crate::models::phrase::{Phrase, PhraseSpec};
use crate::models::texts::Texts;
use crate::models::token_graph::{TokenGraph, TokenId};
use crate::optimizer::population::{PopulationOptimizer, Value, WeightedValue};
use itertools::Itertools;
use rand::rngs::SmallRng;
use rand::seq::{IteratorRandom, SliceRandom};
use rand::SeedableRng;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::fs;

#[derive(Debug, Copy, Clone)]
pub struct Schedule {
    pub max_actions: usize,
    pub max_values: usize,
    pub patience: usize,
}

/// This is the first part of the problem: determine which tokens to consider, given a list of
/// phrases.
pub fn tokenize(
    texts: &Texts,
    phrases: &[PhraseSpec],
    schedules: &[Schedule],
    rng_seed: u64,
) -> Vec<Phrase> {
    let mut graph = TokenGraph::new(texts, phrases);

    // Detect texts that can be entirely merged
    #[derive(Debug, Default, Copy, Clone)]
    struct Info {
        max_in_phrase: usize,
        total: usize,
    }
    let mut info_by_text: BTreeMap<_, Info> = BTreeMap::new();
    for phrase in phrases {
        let mut count_by_text: BTreeMap<_, usize> = BTreeMap::new();
        for &text in phrase.words() {
            *count_by_text.entry(text).or_default() += 1;
        }

        for (text, count) in count_by_text {
            let info = info_by_text.entry(text).or_default();
            info.total += count;
            info.max_in_phrase = info.max_in_phrase.max(count);
        }
    }
    let collapsable_texts = info_by_text
        .into_iter()
        .filter_map(|(text, info)| {
            if info.max_in_phrase == 1 {
                Some((text, info))
            } else {
                None
            }
        })
        .sorted_by_key(|(text, info)| Reverse(text.len() * info.total))
        .collect_vec();
    log::debug!(
        "Collapsable = {}",
        collapsable_texts
            .iter()
            .format_with(", ", |&(text, info), f| {
                f(&format_args!("{}x{}", info.total, texts.decode(text)))
            })
    );

    // Collapse those tokens
    let mut dfs_space = graph.dfs_space();
    for (text, _) in collapsable_texts {
        let mut merged = 0;
        let mut tokens = graph.tokens_by_text()[&text].clone().into_iter();
        let mergeable = tokens.len() - 1;
        let first = tokens.next().unwrap();
        for other in tokens {
            if graph.can_merge_tokens(first, other, &mut dfs_space) {
                graph.merge_tokens(first, other);
                merged += 1;
            }
        }
        log::debug!(
            "Merged {} out of {} for {}",
            merged,
            mergeable,
            texts.decode(text)
        );
    }

    log::debug!("Starting point: {}", graph);
    let initial_len = graph.letters_len();
    let graph = WeightedGraph::new(graph, initial_len);

    let rng = SmallRng::seed_from_u64(rng_seed);

    let mut optimization = PopulationOptimizer::new(rng, vec![graph]);

    let mut epoch = 0;
    for &schedule in schedules {
        log::info!("Start schedule {:?}", schedule);

        // Reset weight
        let (rng, values) = optimization.into_parts();
        let initial_letters = values.iter().map(|graph| graph.letters_len).max().unwrap();
        let values = values
            .into_iter()
            .map(|mut graph| {
                graph.graph.shrink();
                WeightedGraph::new(graph.graph, initial_letters)
            })
            .collect();
        optimization = PopulationOptimizer::new(rng, values);

        let mut prev_weight = 0.;
        let mut repeated = 0;
        loop {
            let best = optimization.best();
            if prev_weight >= best.weight() {
                repeated += 1;
                if repeated == schedule.patience {
                    break;
                }
            } else {
                repeated = 0;
            }
            prev_weight = best.weight();

            log::info!(
                "Start epoch {} with {} individuals. Best weight = {} with {} tokens, patience {}/{}",
                epoch,
                optimization.len(),
                best.weight(),
                best.tokens_len,
                repeated,
                schedule.patience
            );
            optimization.evolve(schedule.max_actions, schedule.max_values);
            epoch += 1;
        }
    }

    let best = optimization.into_best();
    log::info!(
        "Finished with weight = {} with {} tokens",
        best.weight(),
        best.tokens_len
    );
    log::debug!("Result: {}", best.graph);

    fs::write("data/graph.dot", best.graph.dot()).unwrap();

    best.graph.into_phrases()
}

#[derive(Debug, Clone)]
struct WeightedGraph<'a> {
    graph: TokenGraph<'a>,
    initial_letters: usize,
    letters_len: usize,
    tokens_len: usize,
    /// weight = 1 + (saved letters) ^ 2
    weight: f64,
}

#[derive(Debug, Clone, Copy)]
struct MergeTokens(TokenId, TokenId);

impl<'a> WeightedGraph<'a> {
    pub fn new(graph: TokenGraph<'a>, initial_letters: usize) -> Self {
        let letters_len = graph.letters_len();
        let saved_letters = 1. + (initial_letters - letters_len) as f64;
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
        let texts = self
            .graph
            .tokens_by_text()
            .iter()
            .filter_map(|(&text, tokens)| {
                let unmerged_tokens = tokens
                    .iter()
                    .filter(|&&token| !self.graph.get(token).is_merged())
                    .count();
                if unmerged_tokens == 1 {
                    None
                } else {
                    Some((text, text.len() * unmerged_tokens))
                }
            })
            .collect_vec();
        let (text, _) = *texts.choose_weighted(rng, |&(_, weight)| weight).unwrap();
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
