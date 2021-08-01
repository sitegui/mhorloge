use std::env;
use std::env::VarError;

use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;

mod arrange;
mod generate_phrases;
mod models;
mod tokenize;
mod utils;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "mhorloge",
    about = "CLI for problems related to the mhorloge project."
)]
struct Opt {
    /// Determine the languages to use. Available languages are: "English", "French" and
    /// "Portuguese". Multiple languages can be requested by separating them by comma. By
    /// default, all time phrases will be generated, that is, from 00:00 to 23:59 with 1-minute
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
}

fn main() -> Result<()> {
    if let Err(VarError::NotPresent) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "INFO");
    }
    env_logger::init();
    log::info!("Starting");

    let options = Opt::from_args();
    let (texts, phrases) = generate_phrases::generate_phrases(&options.languages)?;
    tokenize::tokenize(&texts, &phrases, options.output_svg.as_deref())?;

    log::info!("Done");
    Ok(())
}
