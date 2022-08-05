use crate::models::grid::Grid;
use crate::models::grid_bag::GridBag;
use crate::models::merge_dag::MergeDag;
use crate::models::token::Token;
use crate::models::token_relations::TokenRelations;
use crate::models::word::WordId;
use crate::{AspectRatio, Phrase};
use itertools::Itertools;
use std::cmp::Reverse;

pub fn build_grid(
    phrases: &[Phrase],
    token_graph: &MergeDag<WordId, Token>,
    trim_grid_bag_size: usize,
    allow_diagonal: bool,
    aspect_ratio: AspectRatio,
) -> Grid {
    let relations = TokenRelations::new(token_graph, phrases);

    // List in which order the tokens will be merged into the grid bags: start from the "outer"
    // tokens, that is, the tokens with the least depth.
    let tokens_to_insert = token_graph
        .group_depths()
        .into_iter()
        .sorted_by_key(|&(token_id, depth)| {
            let token = &token_graph[token_id];
            (depth, Reverse(token.text.letters().len()), token.id)
        })
        .map(|(token_id, _)| &token_graph[token_id])
        .collect_vec();
    log::debug!(
        "Will build grid with tokens: {}",
        tokens_to_insert.iter().format(", ")
    );

    // Regroup tokens into grids
    let mut grid_bag = GridBag::new(aspect_ratio);
    let num_tokens = tokens_to_insert.len();
    for (i, inserting_token) in tokens_to_insert.into_iter().enumerate() {
        log::info!(
            "Insert {} into bag with {} grids ({}/{})",
            inserting_token,
            grid_bag.grids().len(),
            i,
            num_tokens
        );

        grid_bag.insert(&relations, inserting_token, allow_diagonal);
        grid_bag.trim(trim_grid_bag_size);
    }

    grid_bag.best_grid().clone()
}
