use crate::models::grid::{Direction, Grid};
use crate::models::grid_bag::GridBag;
use crate::models::merge_dag::{Group, MergeDag};
use crate::models::token::{Token, TokenId};
use crate::models::token_relations::TokenRelations;
use crate::models::word::Word;
use itertools::Itertools;

pub fn build_grid<'a>(token_graph: &'a MergeDag<&Word, Token>) -> MergeDag<&'a Token, GridBag> {
    let relations = TokenRelations::new(token_graph);

    // Build a next-level graph that will merge tokens into grid bags
    let tokens = token_graph.groups().map(|(_, token)| token).collect_vec();
    let edges = token_graph
        .group_edges()
        .map(|(source, target)| (source.id, target.id))
        .collect_vec();

    let grid = Grid::new(tokens[0], Direction::Horizontal);
    println!("{}", grid);

    let mut total = 0;
    for &other_token in &tokens[1..] {
        println!("Insertions of {}", other_token);
        for each in grid.enumerate_insertions(&relations, other_token) {
            println!("{}", each);
            total += 1;
        }
    }
    println!("Total = {}", total);

    let mut graph = MergeDag::new(tokens, &edges);

    graph
}
