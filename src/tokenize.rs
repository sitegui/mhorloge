use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{ensure, Error};
use itertools::Itertools;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use crate::generate_phrases::{GeneratePhrasesOut, PhraseId};
use crate::models::texts::Texts;
use crate::optimizer::population::{PopulationOptimizer, Value};
use crate::tokenize::phrase::PhraseSpec;
use crate::tokenize::token_graph::TokenGraph;
use crate::utils::{create_file, read_json};

mod fast_collapse;
mod phrase;
mod token_graph;

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
    #[structopt(long, default_value = "17")]
    seed: u64,
    #[structopt(long, default_value = "10")]
    initial_candidates: usize,
    #[structopt(long, default_value = "5")]
    grasp_size: usize,
    #[structopt(long, default_value = "3")]
    max_actions: usize,
    #[structopt(long, default_value = "3")]
    patience: usize,
    #[structopt(long, default_value = "5")]
    eras: usize,
    #[structopt(long, default_value = "2")]
    era_delta: usize,
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

pub fn tokenize(cmd: Tokenize) -> Result<(), Error> {
    let input: GeneratePhrasesOut = read_json(cmd.phrases)?;
    let output = create_file(cmd.output)?;

    let mut texts = Texts::new();
    let phrases = input
        .phrases
        .into_iter()
        .map(|el| PhraseSpec::new(&mut texts, el.id, &el.phrase))
        .collect_vec();

    // Build initial candidates
    let base = TokenGraph::new(&texts, &phrases);
    log::info!(
        "Base solution has {} tokens and {} letters",
        base.tokens_len(),
        base.letters_len()
    );
    let mut rng = SmallRng::seed_from_u64(cmd.seed);
    let initial_candidates =
        fast_collapse::fast_collapse(&base, &mut rng, cmd.initial_candidates, cmd.grasp_size);

    // Optimize
    let mut optimization = PopulationOptimizer::new(rng, initial_candidates);
    let mut patience = cmd.patience;
    let mut max_actions = cmd.max_actions;
    let mut max_values = cmd.initial_candidates;
    for _ in 0..cmd.eras {
        // Checkpoint all graphs
        let initial_letters = optimization
            .values()
            .iter()
            .map(|graph| graph.letters_len())
            .max()
            .unwrap();
        for graph in optimization.values_mut() {
            graph.check_point(initial_letters);
        }

        let best = optimization.best();
        log::info!(
            "Best solution has {} tokens and {} letters",
            best.tokens_len(),
            best.letters_len()
        );

        optimization.evolve_era(patience, max_actions, max_values);
        patience += cmd.era_delta;
        max_actions *= cmd.era_delta;
        max_values *= cmd.era_delta;
    }

    // Prepare output
    let best = optimization.into_best();
    log::info!(
        "Best solution has {} tokens and {} letters",
        best.tokens_len(),
        best.letters_len()
    );
    serde_json::to_writer_pretty(output, &best.to_output())?;

    if let Some(output_svg) = &cmd.output_svg {
        let mut command = Command::new("dot");
        command
            .args(&["-T", "svg", "-Gsplines=ortho", "-o"])
            .arg(output_svg);
        if log::log_enabled!(log::Level::Debug) {
            command.arg("-v");
        }
        let mut dot = command.stdin(Stdio::piped()).spawn()?;

        dot.stdin
            .as_ref()
            .unwrap()
            .write_all(best.dot().as_bytes())?;

        ensure!(dot.wait()?.success(), "Failed to generate SVG");
    }

    Ok(())
}

impl<'a> Value for TokenGraph<'a> {
    fn evolve(&self, max_actions: usize, rng: &mut SmallRng) -> Vec<Self> {
        // Choose which text to target
        let tokens_by_text = self.tokens_by_text();
        let texts = tokens_by_text
            .iter()
            .filter_map(|(&text, tokens)| {
                if tokens.len() == 1 {
                    None
                } else {
                    Some((text, text.len() * tokens.len()))
                }
            })
            .collect_vec();

        // Generate the desired number of new graphs
        let mut new_values = Vec::with_capacity(max_actions);
        for _ in 0..max_actions {
            let (text, _) = *texts.choose_weighted(rng, |&(_, weight)| weight).unwrap();
            log::debug!("Selected text {:?}", self.texts().decode(text));

            let unmerged_tokens = &tokens_by_text[&text];
            let a = *unmerged_tokens.choose(rng).unwrap();

            for &b in unmerged_tokens {
                if self.can_merge_tokens(a, b) {
                    new_values.push(self.with_merged_tokens(a, b));
                    if new_values.len() == max_actions {
                        return new_values;
                    }
                }
            }
        }
        new_values
    }

    fn weight(&self) -> f64 {
        self.weight()
    }
}
