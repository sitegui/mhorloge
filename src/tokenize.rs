// pub mod token_graph;

use crate::models::merge_dag::MergeDag;
use crate::models::phrase_book::PhraseBook;
use crate::models::text::Text;
use crate::models::token::Token;
use crate::models::word::WordId;
use itertools::Itertools;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub struct RepeatedSequence<'a> {
    texts: Vec<&'a Text>,
    instances: Vec<&'a [WordId]>,
}

pub fn tokenize(book: &PhraseBook) -> MergeDag<WordId, Token> {
    let mut seed_tokens = vec![];
    let mut edges = vec![];
    for phrase in book.phrases() {
        for &word_id in &phrase.words {
            let token = Token::new(&book[word_id]);
            seed_tokens.push((word_id, token));
        }

        for (&before, &after) in phrase.words.iter().tuple_windows::<(_, _)>() {
            edges.push((before, after));
        }
    }

    let mut graph = MergeDag::new(seed_tokens, &edges);
    log::info!("Initial token graph has {} words", graph.nodes_len(),);

    let sequences = extract_sequences(book);
    log::info!("Will try to merge {} sequences", sequences.len());
    for sequence in &sequences {
        merge_sequence(&mut graph, sequence);
    }

    graph
}

/// Extract all sequences of one or more words that repeat at least twice in the phrases.
/// The sequences are sorted by descending word length first and then total number of letters in all
/// instances.
fn extract_sequences(book: &PhraseBook) -> Vec<RepeatedSequence> {
    let max_words_per_phrase = book
        .phrases()
        .iter()
        .map(|phrase| phrase.words.len())
        .max()
        .unwrap();

    (1..=max_words_per_phrase)
        .flat_map(|length| extract_sequences_with_length(book, length))
        .sorted_by_key(|sequence| {
            let letters_per_instance: usize =
                sequence.texts.iter().map(|text| text.letters().len()).sum();
            let total_letters = letters_per_instance * sequence.instances.len();
            Reverse((sequence.texts.len(), total_letters))
        })
        .collect_vec()
}

/// Extract sequences of `length` words from all phrases and collect all those that repeat more than
/// once. For each sequence, it will regroup all the word locations that compose each instance of
/// the repeated sequence.
fn extract_sequences_with_length(book: &PhraseBook, length: usize) -> Vec<RepeatedSequence> {
    // Collect all sequences
    let mut sequences: BTreeMap<_, Vec<_>> = BTreeMap::new();
    for phrase in book.phrases() {
        for words in phrase.words.windows(length) {
            let texts = words
                .iter()
                .map(|&word_id| &book[word_id].text)
                .collect_vec();
            sequences.entry(texts).or_default().push(words);
        }
    }

    // Select the sequences of interest
    sequences
        .into_iter()
        .filter_map(|(texts, instances)| {
            if instances.len() == 1 {
                None
            } else {
                Some(RepeatedSequence { texts, instances })
            }
        })
        .collect_vec()
}

fn merge_sequence(graph: &mut MergeDag<WordId, Token>, sequence: &RepeatedSequence) {
    log::debug!("Will merge sequence: {}", sequence.texts.iter().format(" "));

    for i in 0..sequence.texts.len() {
        let locations = sequence.instances.iter().map(|loc| loc[i]).collect_vec();
        merge_locations(graph, &locations);
    }
}

/// Merge the given locations together, as much as possible. Each location will be taken in sequence
/// and will be merged with the previous ones. If that's not possible, it will start a new group of
/// its own. The following locations will try to merge with the first group. When not possible, it
/// will try with the second, and so on until no group accepts it. In this case, a new group will
/// created again.
fn merge_locations(graph: &mut MergeDag<WordId, Token>, words: &[WordId]) {
    let mut group_roots = Vec::new();

    let unique_tokens_before: BTreeSet<_> = words
        .iter()
        .map(|&word_id| graph.group(word_id).0)
        .collect();

    if unique_tokens_before.len() < 2 {
        return;
    }

    for &word in words {
        let mut merged = false;
        let word_group = graph.group(word).0;

        for &root in &group_roots {
            if !graph.has_path(root, word_group) {
                graph.merge_groups(root, word_group, |base_token, new_token| {
                    base_token.words.extend(new_token.words);
                });
                merged = true;
                break;
            }
        }

        if !merged {
            group_roots.push(word_group);
        }
    }

    let unique_tokens_after: BTreeSet<_> = words
        .iter()
        .map(|&word_id| graph.group(word_id).0)
        .collect();

    log::debug!(
        "Merged {} words, from {} tokens into {} tokens",
        words.len(),
        unique_tokens_before.len(),
        unique_tokens_after.len(),
    );
}
