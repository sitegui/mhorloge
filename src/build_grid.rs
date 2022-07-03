use crate::models::grid::XY;
use crate::models::grid_bag::GridBag;
use crate::models::merge_dag::MergeDag;
use crate::models::token::{Token, TokenId};
use crate::models::token_relations::TokenRelations;
use crate::models::word::WordId;
use itertools::Itertools;
use std::cmp::Reverse;

pub fn build_grid(
    token_graph: &MergeDag<WordId, Token>,
    trim_grid_bag_size: usize,
) -> MergeDag<TokenId, GridBag> {
    let relations = TokenRelations::new(token_graph);

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

    // Regroup tokens into grids
    let mut n = 0;
    for inserting_token in tokens_to_insert {
        let inserted = insert_token(trim_grid_bag_size, &relations, &mut graph, inserting_token);

        if inserted {
            graph.svg(format!("data/grid-bag-{}.svg", n)).unwrap();
            n += 1;
        }
    }

    // Take the best grid from each back, in topological order
    let best_grids = graph
        .group_depths()
        .into_iter()
        .sorted_by_key(|&(_, depth)| depth)
        .map(|(group_id, _)| {
            let bag = &graph[group_id];
            let best_grid = bag.best_grid();

            println!("Tokens: {}", bag.tokens().iter().format(" "));
            println!("{}", best_grid);

            best_grid
        })
        .collect_vec();

    // Put all grids together
    let mut result = best_grids[0].clone();
    for &grid in &best_grids[1..] {
        let (x, y) = result.space();
        result.add_grid(grid, XY::new(*x.end() + 2, *y.start()));
    }

    println!("{}", result);

    todo!()
}

fn insert_token(
    trim_grid_bag_size: usize,
    relations: &TokenRelations,
    graph: &mut MergeDag<TokenId, GridBag>,
    inserting_token: &Token,
) -> bool {
    let inserting_group = graph.group(inserting_token.id);
    if inserting_group.1.tokens().len() > 1 {
        log::debug!(
            "Skip {} because it is already part of a non-trivial grid",
            inserting_token
        );
        return false;
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
                target_group.1.tokens().iter().format(" "),
                target_group.1.grids().len()
            );

            if let Some(merged_bag) =
                target_group
                    .1
                    .with_inserted(relations, inserting_token, trim_grid_bag_size)
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
        true
    } else {
        false
    }
}
