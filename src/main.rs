use std::env::VarError;
use std::path::PathBuf;
use std::time::Instant;
use std::{env, fs};

use anyhow::{ensure, Result};
use jemallocator::Jemalloc;
use structopt::StructOpt;

use crate::models::aspect_ratio::AspectRatio;
use crate::models::grid::Grid;
use crate::models::io::{
    GridInput, GridOutput, GridOutputPhrase, LyricsPhrase, LyricsPhraseStop, LyricsPhrasesInput,
    LyricsPhrasesOutput, TimePhrasesOutput, WordOrSpace,
};
use crate::models::language::Language;
use crate::models::merge_dag::MergeDag;
use crate::models::phrase::Phrase;
use crate::models::phrase_book::PhraseBook;
use crate::models::positioned_token::XY;
use crate::models::token::Token;
use crate::models::word::WordId;

mod build_grid;
mod compile_lyrics_css;
mod generate_phrases;
mod models;
mod tokenize;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "mhorloge",
    about = "CLI for problems related to the mhorloge project."
)]
enum Options {
    /// Generate time phrases and save them into a file
    TimePhrases {
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
        /// The path to a file where to write the output as JSON, represented by `TimePhrasesOutput`.
        phrases_output: PathBuf,
    },
    /// Generate phrases from the format produced by `web/sync-lyrics.html` tool
    LyricsPhrases {
        /// The path to the input JSON file, represented by `LyricsPhrasesInput`.
        lyrics_input: PathBuf,
        /// The path to a file where to write the output as JSON, represented by `LyricsPhrasesOutput`.
        phrases_output: PathBuf,
    },
    /// Generate a grid for a given set of phrases
    Grid {
        /// The path to the input JSON file, represented by `GridInput`.
        phrases_input: PathBuf,
        /// The path to a file where to write the output as JSON, represented by `GridOutput`.
        grid_output: PathBuf,
        /// If present, will also try to position the token diagonally.
        #[structopt(long)]
        allow_diagonal: bool,
        /// The target aspect ratio, expressed by two integers separated by a colon ":".
        #[structopt(long, default_value = "16:9")]
        aspect_ratio: AspectRatio,
        /// Multiple grids are constructed at each step of the algorithm. This controls how many
        /// grids at most can be considered.
        #[structopt(long, default_value = "10000")]
        max_grid_bag_size: usize,
        /// When given, will produce a debug SVG with a visual representation of the "token graph".
        ///
        /// This requires that a binary called `dot` be available. Tested with version 2.43.0.
        /// You can install it with the `graphviz` package.
        #[structopt(long)]
        debug_tokens_svg: Option<PathBuf>,
        /// When merging repeated words from different phrases together - into what's internally
        /// called tokens - they create chains that can be bigger than the original phrase.
        ///
        /// This setting controls controls their maximum size, expressed in number of words above
        /// the longest original phrase.
        #[structopt(long, default_value = "1")]
        chain_growth_head_space: i32,
    },
    /// Generate the CSS to sync each letter of a grid with a song's lyrics
    LyricsCssAnimation {
        /// The path to the lyrics input JSON file, represented by `LyricsPhrasesOutput`.
        phrases_input: PathBuf,
        /// The path to the grid input JSON file, represented by `GridOutput`.
        grid_input: PathBuf,
        /// The path to a file where to write the output as CSS.
        css_output: PathBuf,
    },
}

fn main() -> Result<()> {
    let start = Instant::now();

    if let Err(VarError::NotPresent) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "INFO");
    }
    env_logger::init();
    log::info!("Starting");

    let options: Options = Options::from_args();
    match options {
        Options::TimePhrases {
            languages,
            phrases_output,
        } => {
            time_phrases(languages, phrases_output)?;
        }
        Options::LyricsPhrases {
            lyrics_input,
            phrases_output,
        } => {
            lyrics_phrases(lyrics_input, phrases_output)?;
        }
        Options::Grid {
            phrases_input,
            grid_output,
            allow_diagonal,
            aspect_ratio,
            max_grid_bag_size,
            debug_tokens_svg,
            chain_growth_head_space,
        } => {
            grid(
                phrases_input,
                grid_output,
                allow_diagonal,
                aspect_ratio,
                max_grid_bag_size,
                debug_tokens_svg,
                chain_growth_head_space,
            )?;
        }
        Options::LyricsCssAnimation {
            phrases_input,
            grid_input,
            css_output,
        } => lyrics_css_animation(phrases_input, grid_input, css_output)?,
    }

    log::info!("Done in {:?}", start.elapsed());

    Ok(())
}

fn lyrics_css_animation(
    phrases_input: PathBuf,
    grid_input: PathBuf,
    css_output: PathBuf,
) -> Result<()> {
    let phrases: LyricsPhrasesOutput = serde_json::from_str(&fs::read_to_string(&phrases_input)?)?;
    let grid: GridOutput = serde_json::from_str(&fs::read_to_string(&grid_input)?)?;

    let css_source = compile_lyrics_css::compile_lyrics_css(phrases, grid);

    fs::write(&css_output, css_source)?;

    Ok(())
}

fn time_phrases(languages: String, phrases_output: PathBuf) -> Result<()> {
    let mut language_specs = vec![];

    for mut language_tag in languages.split(',') {
        let precision;
        match language_tag.find(':') {
            None => precision = 1,
            Some(pos) => {
                precision = language_tag[pos + 1..].parse()?;
                language_tag = &language_tag[..pos];
            }
        }

        let language: Language = language_tag.parse()?;
        language_specs.push((language, precision));
    }

    let phrases = generate_phrases::generate_phrases(&language_specs);
    log::info!("Generated {} phrases", phrases.len());

    if let Some(parent) = phrases_output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        &phrases_output,
        serde_json::to_string_pretty(&TimePhrasesOutput { phrases })?,
    )?;

    Ok(())
}

fn lyrics_phrases(lyrics_input: PathBuf, phrases_output: PathBuf) -> Result<()> {
    let lyrics_input: LyricsPhrasesInput =
        serde_json::from_str(&fs::read_to_string(&lyrics_input)?)?;

    let lines = lyrics_input.stops.split(|element| match element {
        WordOrSpace::Word { .. } => false,
        WordOrSpace::Space(text) => text.contains('\n'),
    });

    let mut phrases = vec![];

    for line in lines {
        let mut phrase = LyricsPhrase {
            phrase: String::new(),
            stops: vec![],
        };
        let mut word_index = 0;

        for element in line {
            match element {
                WordOrSpace::Word { word, times } => {
                    phrase.phrase += word;
                    phrase.phrase.push(' ');
                    for &time in times {
                        phrase.stops.push(LyricsPhraseStop { word_index, time });
                    }
                    word_index += 1;
                }
                WordOrSpace::Space(_) => {}
            }
        }

        if !phrase.phrase.is_empty() {
            ensure!(phrase.phrase.pop() == Some(' '));
            phrases.push(phrase);
        }
    }

    fs::write(
        &phrases_output,
        serde_json::to_string_pretty(&LyricsPhrasesOutput {
            phrases,
            total_duration: lyrics_input.total_duration,
        })?,
    )?;

    Ok(())
}

fn grid(
    phrases_input: PathBuf,
    grid_output: PathBuf,
    allow_diagonal: bool,
    aspect_ratio: AspectRatio,
    max_grid_bag_size: usize,
    debug_tokens_svg: Option<PathBuf>,
    chain_growth_head_space: i32,
) -> Result<()> {
    let grid_input: GridInput = serde_json::from_str(&fs::read_to_string(&phrases_input)?)?;

    let mut phrase_book = PhraseBook::default();
    for phrase in grid_input.phrases {
        phrase_book.insert_phrase(&phrase.phrase);
    }
    log::info!("Read {} phrases", phrase_book.phrases().len());

    let token_graph = tokenize::tokenize(&phrase_book, chain_growth_head_space);
    log::info!(
        "Formed token graph with {} tokens",
        token_graph.groups_len(),
    );

    if let Some(debug_tokens_svg) = &debug_tokens_svg {
        token_graph.svg(debug_tokens_svg)?;
    }

    let best_grid = build_grid::build_grid(
        phrase_book.phrases(),
        &token_graph,
        max_grid_bag_size,
        allow_diagonal,
        aspect_ratio,
    );

    let (width, height) = best_grid.size();
    log::info!("Build grid {}x{}", width, height);

    let (aspect_width, aspect_height) = aspect_ratio.cover(width, height);
    let mut final_grid = best_grid.clone();
    final_grid.fill_to_size(aspect_width, aspect_height, &mut rand::thread_rng())?;
    log::info!("Filled grid into {}x{}", aspect_width, aspect_height);

    let final_letters = final_grid
        .to_letters()
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|letter| letter.expect("Grid is totally filled"))
                .collect()
        })
        .collect();

    let final_phrases = phrase_book
        .phrases()
        .iter()
        .map(|phrase| GridOutputPhrase {
            letters: phrase_to_letter_positions(&token_graph, &final_grid, phrase),
        })
        .collect();

    if let Some(parent) = grid_output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        &grid_output,
        serde_json::to_string(&GridOutput {
            minimal_grid: best_grid.to_letters(),
            grid: final_letters,
            phrases: final_phrases,
        })?,
    )?;

    Ok(())
}

fn phrase_to_letter_positions(
    token_graph: &MergeDag<WordId, Token>,
    grid: &Grid,
    phrase: &Phrase,
) -> Vec<(i16, i16)> {
    let top_left = grid.top_left();

    phrase
        .words
        .iter()
        .map(|&word| token_graph.group(word).1)
        .flat_map(|token| {
            grid.positions_for_token(token.id)
                .expect("The token must be present")
        })
        .map(|pos| {
            let abs_pos = pos - top_left;
            (abs_pos.x, abs_pos.y)
        })
        .collect()
}
