use crate::models::grid_bag::GridBag;
use crate::models::merge_dag::MergeDag;
use crate::models::token::{Token, TokenId};
use crate::models::token_relations::TokenRelations;
use crate::models::word::WordId;
use itertools::Itertools;
use std::cmp::Reverse;
use std::collections::BTreeMap;

pub fn build_grid(token_graph: &MergeDag<WordId, Token>) -> MergeDag<TokenId, GridBag> {
    let relations = TokenRelations::new(token_graph);
    let token_by_id: BTreeMap<_, _> = token_graph
        .groups()
        .map(|(_, token)| (token.id, token))
        .collect();

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

    // Build a next-level graph that will merge tokens into grid bags
    let seed_grid_bags = token_graph
        .groups()
        .map(|(_, token)| (token.id, GridBag::new(token)))
        .collect_vec();
    let edges = token_graph
        .group_edges()
        .map(|(source, target)| (source.id, target.id))
        .collect_vec();
    let mut graph = MergeDag::new(seed_grid_bags, &edges);

    for inserting_token in tokens_to_insert {
        let inserting_group = graph.group(inserting_token.id);
        if inserting_group.1.tokens().len() > 1 {
            log::debug!(
                "Skip {} because it is already part of a non-trivial grid",
                inserting_token
            );
            continue;
        }

        // Find all possible candidates
        let mut candidates = vec![];
        for target_group in graph.groups() {
            if target_group.0 == inserting_group.0 {
                continue;
            }

            if !graph.has_indirect_path(inserting_group.0, target_group.0) {
                log::debug!(
                    "Insert {} into bag with {} ({} grids)",
                    inserting_token,
                    target_group
                        .1
                        .tokens()
                        .iter()
                        .map(|&id| &token_by_id[&id])
                        .format(" "),
                    target_group.1.grids().len()
                );
                if let Some(merged_bag) = target_group.1.with_inserted(&relations, inserting_token)
                {
                    candidates.push((target_group.0, merged_bag));
                }
            }
        }

        // Pick the best candidate
        let best_candidate = candidates.into_iter().max_by_key(|(_, new_bag)| {
            new_bag
                .grids()
                .iter()
                .map(|grid| grid.num_letters())
                .max()
                .unwrap()
        });

        if let Some((target_group, new_bag)) = best_candidate {
            log::info!(
                "Inserted {}, producing bag with {} tokens and {} grids",
                inserting_token,
                new_bag.tokens().len(),
                new_bag.grids().len()
            );
            log::debug!("Produce bag: {}", new_bag);

            graph.merge_groups(inserting_group.0, target_group, |receiving_bag, _| {
                *receiving_bag = new_bag;
            });
        }
    }

    for (_, bag) in graph.groups() {
        println!("{}", bag.grids()[0]);
    }

    todo!()
}
