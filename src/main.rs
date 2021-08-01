use std::env;
use std::env::VarError;

use crate::generate_phrases::GeneratePhrases;
use crate::tokenize::Tokenize;
use anyhow::Error;
use structopt::StructOpt;

mod generate_phrases;
mod models;
mod tokenize;
mod utils;

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
        Opt::GeneratePhrases(cmd) => generate_phrases::generate_phrases(cmd)?,
        Opt::Tokenize(cmd) => tokenize::tokenize(cmd)?,
    };

    log::info!("Done");
    Ok(())
}
