use crate::languages::english::English;
use crate::languages::french::French;
use crate::languages::portuguese::Portuguese;
use crate::languages::Language;
use crate::models::time::Time;
use anyhow::{bail, Error};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use structopt::StructOpt;

/// Generate the time phrases for the given languages
#[derive(Debug, StructOpt)]
pub struct GeneratePhrases {
    /// Determine the languages to use. Available languages are: "english", "french" and
    /// "portuguese". Multiple languages can be requested by separating them by comma. By
    /// default, all time phrases will be generated, that is, from 00:00 to 23:59 with 1-minute
    /// precision. To change the precision, append ":" followed by an integer representing the
    /// desired precision after each language name. Each language can determine their own
    /// precision.
    ///
    /// Full example: "english:5,french" will generate for both languages, using a 1-minute
    /// precision for French and 5-minute precision for English.
    languages: String,
    /// The output file. Each line will be like "english 12 45 QUARTER TO ONE". The first word
    /// is the language, followed by the hours, then minutes and finally the time phrase.
    output: PathBuf,
}

pub fn generate_phrases(cmd: GeneratePhrases) -> Result<(), Error> {
    let mut file = BufWriter::new(File::open(cmd.output)?);

    for mut language_tag in cmd.languages.split(',') {
        let precision;
        match language_tag.find(':') {
            None => precision = 1,
            Some(pos) => {
                precision = language_tag[pos + 1..].parse()?;
                language_tag = &language_tag[..pos];
            }
        }

        let language: &dyn Language = match language_tag {
            "english" => &English,
            "french" => &French,
            "portuguese" => &Portuguese,
            _ => bail!("Language was not recognized: {}", language_tag),
        };

        for time in Time::all_times().step_by(precision as usize) {
            writeln!(
                file,
                "{} {} {} {}",
                language_tag,
                time.hours(),
                time.minutes(),
                language.spell(time)
            )?;
        }
    }

    Ok(())
}
