use crate::{GridInput, GridOutput};
use itertools::Itertools;
use std::collections::BTreeSet;

pub fn compile_html(grid_input: &GridInput, grid: &GridOutput) -> String {
    let page = include_str!("template.html");
    let page = page.replacen("${GRID}", &compile_grid(grid), 1);
    page.replacen("${PHRASES}", &compile_phrases(grid_input, grid), 1)
}

pub fn compile_grid(grid: &GridOutput) -> String {
    grid.grid
        .iter()
        .enumerate()
        .format_with("<br>\n", |(j, letters), f| {
            for (i, letter) in letters.iter().enumerate() {
                f(&format_args!(
                    "<span class=\"letter-off\">{}<span class=\"letter-on letter-on-{}-{}\">{}</span></span>",
                    letter, i, j, letter
                ))?;
            }
            Ok(())
        })
        .to_string()
}

fn compile_phrases(grid_input: &GridInput, grid_output: &GridOutput) -> String {
    let mut seen_texts = BTreeSet::new();

    grid_input
        .phrases
        .iter()
        .zip(&grid_output.phrases)
        .format_with("\n", |(phrase_input, phrase_output), f| {
            let phrase_text = phrase_input.texts.iter().format(" ").to_string();

            if seen_texts.insert(phrase_text.clone()) {
                let letters = phrase_output
                    .words
                    .iter()
                    .flat_map(|word| &word.letters)
                    .format_with(" ", |letter, f| {
                        f(&format_args!("letter-on-{}-{}", letter.0, letter.1))
                    });

                f(&format_args!(
                    "<option value=\"{}\">{}</option>",
                    letters, phrase_text
                ))?;
            }

            Ok(())
        })
        .to_string()
}
