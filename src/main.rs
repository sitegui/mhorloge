use crate::commands::generate_phrases::{generate_phrases, GeneratePhrases};
use crate::commands::tokenize::{tokenize, Tokenize};
use anyhow::Error;
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
    env_logger::init();
    log::info!("Starting");

    let opt = Opt::from_args();
    println!("{:?}", opt);

    match opt {
        Opt::GeneratePhrases(cmd) => generate_phrases(cmd),
        Opt::Tokenize(cmd) => tokenize(cmd),
    }
}
