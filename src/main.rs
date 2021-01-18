use crate::commands::generate_phrases::{generate_phrases, GeneratePhrases};
use crate::commands::tokenize::{tokenize, Tokenize};
use anyhow::Error;
use std::env;
use std::env::VarError;
use structopt::StructOpt;

mod commands;
mod languages;
mod models;
mod optimizer;
mod tokenize;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "mhorloge",
    about = "CLI for problems related to the mhorloge project."
)]
enum Opt {
    GeneratePhrases(GeneratePhrases),
    Tokenize(Tokenize),
}

fn main() -> Result<(), Error> {
    if let Err(VarError::NotPresent) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "INFO");
    }
    env_logger::init();
    log::info!("Starting");

    match Opt::from_args() {
        Opt::GeneratePhrases(cmd) => generate_phrases(cmd)?,
        Opt::Tokenize(cmd) => tokenize(cmd)?,
    };

    log::info!("Done");
    Ok(())
}
