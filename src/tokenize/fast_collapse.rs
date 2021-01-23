use crate::models::texts::TextTag;
use crate::optimizer::grasp::Grasp;
use crate::tokenize::token_graph::TokenGraph;
use itertools::Itertools;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use std::cmp::Reverse;
use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, Default, Copy, Clone)]
struct TextInfo {
    max_in_phrase: usize,
    total: usize,
}

/// Detect texts that can probably be merged into a single token. That's the case of texts that
/// happen at most once per phrase. `num` candidates will be generated.
pub fn fast_collapse<'a>(
    base: &TokenGraph<'a>,
    rng: &mut SmallRng,
    num_candidates: usize,
    grasp_size: usize,
) -> Vec<TokenGraph<'a>> {
    // Collect information by text
    let mut info_by_text: BTreeMap<_, TextInfo> = BTreeMap::new();
    for phrase in base.phrases() {
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

    // Detect texts of interest and sort by keeping the ones with the maximum total letters first
    let collapsable_texts: VecDeque<_> = info_by_text
        .into_iter()
        .filter_map(|(text, info)| {
            if info.max_in_phrase == 1 {
                Some((text, info))
            } else {
                None
            }
        })
        .sorted_by_key(|(text, info)| Reverse(info.total * text.len()))
        .collect();
    log::debug!(
        "Collapsable = {}",
        collapsable_texts
            .iter()
            .format_with(", ", |&(text, info), f| {
                f(&format_args!(
                    "{}x{}",
                    info.total,
                    base.texts().decode(text)
                ))
            })
    );

    (0..num_candidates)
        .map(|_| candidate(base, rng, Grasp::new(collapsable_texts.clone(), grasp_size)))
        .collect_vec()
}

/// Collapse those tokens
fn candidate<'a>(
    base: &TokenGraph<'a>,
    rng: &mut SmallRng,
    mut collapsable_texts: Grasp<(TextTag, TextInfo)>,
) -> TokenGraph<'a> {
    let mut graph = base.clone();

    while let Some((text, _)) = collapsable_texts.pop(rng) {
        // Find the token with the given text and iterate in a random order
        let mut tokens = graph.tokens_by_text()[&text].clone();
        tokens.shuffle(rng);
        let tokens_len = tokens.len();
        let mut tokens = tokens.into_iter();

        // Merge into "sink" tokens
        let mut sink_tokens = vec![tokens.next().unwrap()];
        'merge_next: for other in tokens {
            for &sink in &sink_tokens {
                if graph.can_merge_tokens(sink, other) {
                    graph = graph.with_merged_tokens(sink, other);
                    continue 'merge_next;
                }
            }

            // This token could not merge with any previous "sink", so it becomes a "sink" itself.
            sink_tokens.push(other);
        }

        log::debug!(
            "Merged {}x{} into {} sinks",
            base.texts().decode(text),
            tokens_len,
            sink_tokens.len(),
        );
    }

    graph
}
