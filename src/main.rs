use crate::models::aspect_ratio::AspectRatio;
use anyhow::Result;
use itertools::Itertools;
use jemallocator::Jemalloc;
use std::collections::BTreeSet;
use std::env::VarError;
use std::fmt::Write;
use std::path::PathBuf;
use std::time::Instant;
use std::{env, fs};
use structopt::StructOpt;

use crate::models::grid::Grid;
use crate::models::merge_dag::MergeDag;
use crate::models::phrase::Phrase;
use crate::models::phrase_book::PhraseBook;
use crate::models::positioned_token::XY;
use crate::models::token::Token;
use crate::models::word::WordId;

mod arrange;
mod build_grid;
mod generate_phrases;
mod models;
mod tokenize;
mod utils;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "mhorloge",
    about = "CLI for problems related to the mhorloge project."
)]
struct Opt {
    /// Determine the languages to use. Available languages are: "English", "French", "Portuguese"
    /// and "German". Multiple languages can be requested by separating them by comma. By
    /// default, all time phrases will be generated, that is, from 00:00 to 12:00 with 1-minute
    /// precision. To change the precision, append ":" followed by an integer representing the
    /// desired precision after each language name. Each language can determine their own
    /// precision.
    ///
    /// Full example: "English:5,French" will generate for both languages, using a 1-minute
    /// precision for French and 5-minute precision for English.
    languages: String,
    /// The output JSON file.
    output: PathBuf,
    /// The visual output of the graph, in SVG format.
    ///
    /// This requires that a binary called `dot` be available. Tested with version 2.43.0.
    /// You can install it with the `graphviz` package.
    #[structopt(long)]
    output_svg: Option<PathBuf>,
    /// TODO
    #[structopt(long, default_value = "10000")]
    trim_grid_bag_size: usize,
    /// TODO
    #[structopt(long)]
    allow_diagonal: bool,
    /// TODO
    #[structopt(long, default_value = "16:9")]
    aspect_ratio: AspectRatio,
}

fn main() -> Result<()> {
    let start = Instant::now();

    if let Err(VarError::NotPresent) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "INFO");
    }
    env_logger::init();
    log::info!("Starting");

    let options = Opt::from_args();
    let phrase_book = generate_phrases::generate_phrases(&options.languages)?;
    log::info!("Generated {} phrases", phrase_book.phrases().len());

    let token_graph = tokenize::tokenize(&phrase_book, options.output_svg.as_deref());
    log::info!(
        "Formed token graph with {} tokens",
        token_graph.groups_len(),
    );

    let final_grid = build_grid::build_grid(
        phrase_book.phrases(),
        &token_graph,
        options.trim_grid_bag_size,
        options.allow_diagonal,
        options.aspect_ratio,
    );

    let mut all_html = r#"<!doctype html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport"
              content="width=device-width, user-scalable=no, initial-scale=1.0, maximum-scale=1.0, minimum-scale=1.0">
        <meta http-equiv="X-UA-Compatible" content="ie=edge">
        <title>Grid</title>
        <style>
        .grid {
            white-space: pre;
            font-family: monospace;
            border: thin black solid;
            display: inline-block;
            color: #ccc;
            font-size: 16px;
            margin: 5px;
            padding: 3px;
        }
        
        .phrase {
            color: black;
        }
        
        .highlight {
            color: blue;
            font-weight: bold;
        }
        </style>
    </head>
    <body>"#.to_string();

    let unique_phrases = phrase_book.phrases().iter().unique_by(|phrase| {
        phrase
            .words
            .iter()
            .map(|&word| &phrase_book[word])
            .join(" ")
    });
    for phrase in unique_phrases {
        all_html += &grid_to_html(&phrase_book, &token_graph, &final_grid, phrase);
    }

    all_html += "</body></html>";

    fs::write("data/grid.html", all_html).unwrap();

    log::info!("Done in {:?}", start.elapsed());

    Ok(())
}

fn grid_to_html(
    phrase_book: &PhraseBook,
    token_graph: &MergeDag<WordId, Token>,
    grid: &Grid,
    phrase: &Phrase,
) -> String {
    let phrase_tokens: BTreeSet<_> = phrase
        .words
        .iter()
        .map(|&word| {
            let token = token_graph.group(word).1;
            token.id
        })
        .collect();

    let positioned_tokens = grid
        .tokens()
        .iter()
        .copied()
        .filter(|positioned_token| phrase_tokens.contains(&positioned_token.token_id()));

    let highlights: BTreeSet<_> = positioned_tokens
        .flat_map(|positioned_token| positioned_token.iter_pos())
        .collect();

    let (dx, dy) = grid.space();
    let mut html = format!(
        "<div class=\"grid\"><p class=\"phrase\">{}</p>",
        phrase
            .words
            .iter()
            .map(|&word| &phrase_book[word])
            .format(" ")
    );

    for y in dy {
        for x in dx.clone() {
            let pos = XY::new(x, y);
            let letter = grid.get(pos).map(|l| l.as_char()).unwrap_or(' ');
            if highlights.contains(&pos) {
                write!(html, "<span class=\"highlight\">{}</span>", letter).unwrap();
            } else {
                html.push(letter);
            }
        }

        html.push('\n');
    }

    html.pop();
    html += "</div>";

    html
}
