mod phrase;
mod token_graph;

use crate::generate_phrases::{GeneratePhrasesOut, PhraseId};
use crate::models::texts::{TextTag, Texts};
use crate::tokenize::phrase::PhraseSpec;
use crate::tokenize::token_graph::TokenGraph;
use crate::utils::read_json;
use anyhow::Result;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::PathBuf;
use structopt::StructOpt;

/// Read the phrases from a file and define which tokens will need to be placed in the final puzzle
/// and the position constraint between them.
#[derive(Debug, StructOpt)]
pub struct Tokenize {
    /// The input file, generated by `generate-phrases`.
    phrases: PathBuf,
    /// The output JSON file.
    output: PathBuf,
    /// The visual output of the graph, in SVG format.
    ///
    /// This requires that a binary called `dot` be available. Tested with version 2.43.0.
    /// You can install it with the `graphviz` package.
    #[structopt(long)]
    output_svg: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizeOut {
    pub phrases: Vec<TokenizeOutPhraseEl>,
    pub tokens: Vec<TokenizeOutEl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizeOutPhraseEl {
    pub id: PhraseId,
    pub tokens: Vec<TokenId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizeOutEl {
    pub id: TokenId,
    pub text: String,
    /// All concrete token ids that must be spatially placed **after** this one
    pub followed_by: Vec<TokenId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, Eq, PartialEq)]
#[serde(transparent)]
pub struct TokenId(pub u16);

/// Represents the index of a word in a given phrase
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct WordId(pub u16);

/// Indexes a word globally (phrase and its word)
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhrasedWordId {
    phrase: PhraseId,
    word: WordId,
}

#[derive(Debug)]
pub struct RepeatedSequence {
    text_tags: Vec<TextTag>,
    instances: Vec<Vec<PhrasedWordId>>,
}

pub fn tokenize(cmd: Tokenize) -> Result<()> {
    let input: GeneratePhrasesOut = read_json(cmd.phrases)?;

    let mut texts = Texts::new();
    let phrases = input
        .phrases
        .into_iter()
        .map(|el| PhraseSpec::new(&mut texts, el.id, &el.phrase))
        .collect_vec();

    let mut graph = TokenGraph::new(&texts, &phrases);
    log::info!(
        "Base solution has {} tokens and {} letters",
        graph.tokens_len(),
        graph.letters_len()
    );

    let sequences = extract_sequences(&phrases);
    for sequence in &sequences {
        merge_sequence(&mut graph, sequence);
    }

    log::info!(
        "Final solution has {} tokens and {} letters",
        graph.tokens_len(),
        graph.letters_len()
    );
    println!("{}", graph);

    if let Some(output_svg) = cmd.output_svg {
        graph.svg(output_svg)?;
    }

    Ok(())
}

/// Extract all sequences of one or more words that repeat at least twice in the phrases.
/// The sequences are sorted by descending word length first and then total number of letters in all
/// instances.
fn extract_sequences(phrases: &[PhraseSpec]) -> Vec<RepeatedSequence> {
    let max_words_per_phrase = phrases
        .iter()
        .map(|phrase| phrase.words().len())
        .max()
        .unwrap();

    (1..=max_words_per_phrase)
        .flat_map(|length| extract_sequences_with_length(&phrases, length))
        .sorted_by_key(|sequence| {
            let letters_per_instance: usize = sequence.text_tags.iter().map(|tag| tag.len()).sum();
            let total_letters = letters_per_instance * sequence.instances.len();
            Reverse((sequence.text_tags.len(), total_letters))
        })
        .collect_vec()
}

/// Extract sequences of `length` words from all phrases and collect all those that repeat more than
/// once. For each sequence, it will regroup all the word locations that compose each instance of
/// the repeated sequence.
fn extract_sequences_with_length(phrases: &[PhraseSpec], length: usize) -> Vec<RepeatedSequence> {
    // Collect all sequences
    let mut sequences: BTreeMap<_, Vec<_>> = BTreeMap::new();
    for phrase in phrases {
        let max_end = phrase.words().len().saturating_sub(length - 1);
        for start_index in 0..max_end {
            let end_index = start_index + length;
            let text_tags = phrase.words()[start_index..end_index].to_vec();
            let locations = (start_index..end_index)
                .map(|index| PhrasedWordId {
                    phrase: phrase.id(),
                    word: WordId(index as u16),
                })
                .collect_vec();
            sequences.entry(text_tags).or_default().push(locations);
        }
    }

    // Select the sequences of interest
    sequences
        .into_iter()
        .filter_map(|(text_tags, instances)| {
            if instances.len() == 1 {
                None
            } else {
                Some(RepeatedSequence {
                    text_tags,
                    instances,
                })
            }
        })
        .collect_vec()
}

fn merge_sequence(graph: &mut TokenGraph, sequence: &RepeatedSequence) {
    log::info!(
        "Will merge sequence: {}",
        sequence
            .text_tags
            .iter()
            .format_with(" ", |&tag, f| { f(&graph.texts().decode(tag)) })
    );

    for i in 0..sequence.text_tags.len() {
        let locations = sequence.instances.iter().map(|loc| loc[i]).collect_vec();
        merge_locations(graph, &locations);
    }
}

/// Merge the given locations together, as much as possible. Each location will be taken in sequence
/// and will be merged with the previous ones. If that's not possible, it will start a new group of
/// its own. The following locations will try to merge with the first group. When not possible, it
/// will try with the second, and so on until no group accepts it. In this case, a new group will
/// created again.
fn merge_locations(graph: &mut TokenGraph, locations: &[PhrasedWordId]) {
    let mut group_roots = Vec::new();
    let mut visited_tokens = BTreeSet::new();
    let mut num_merges = 0;

    for &location in locations {
        let token = graph.find_token(location);
        if !visited_tokens.insert(token) {
            continue;
        }

        let mut merged = false;
        for &root in &group_roots {
            if graph.merge_tokens(root, token).is_ok() {
                merged = true;
                num_merges += 1;
                break;
            }
        }
        if !merged {
            group_roots.push(token);
        }
    }

    log::debug!(
        "Merged {} locations, {} unique tokens: {} merges into {} groups",
        locations.len(),
        visited_tokens.len(),
        num_merges,
        group_roots.len(),
    );
}

impl fmt::Display for PhrasedWordId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.phrase.0, self.word.0)
    }
}
