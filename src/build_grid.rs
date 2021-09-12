use crate::models::word::Word;
use crate::models::word_grid::{Orientation, Position, WordGrid, WriteStats};
use crate::tokenize::token_graph::{TokenGraph, TokenSpecId};
use itertools::Itertools;
use petgraph::Direction;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
struct InsertWord<'a> {
    grid: Rc<RefCell<WordGrid>>,
    node_id: TokenSpecId,
    word: &'a Word,
    base: Position,
    orientation: Orientation,
    stats: WriteStats,
}

pub fn build_grid(token_graph: &TokenGraph) {
    let mut remaining = token_graph.clone();
    let mut grids: Vec<Rc<RefCell<WordGrid>>> = vec![];

    loop {
        let free_nodes = remaining
            .graph()
            .externals(Direction::Incoming)
            .collect_vec();
        log::debug!("free_nodes = {:?}", free_nodes);

        if free_nodes.is_empty() {
            break;
        }

        let mut best = None;
        for node_id in free_nodes {
            let word_tag = remaining.graph()[node_id];
            let word = remaining.words().decode(word_tag);

            for grid in &grids {
                best_insert_word(&mut best, node_id, grid, word);
            }
        }

        match best {
            None => {
                // TODO: use some better logic
                let node_id = remaining
                    .graph()
                    .externals(Direction::Incoming)
                    .next()
                    .unwrap();
                let word_tag = remaining.graph()[node_id];
                let word = remaining.words().decode(word_tag);
                log::debug!("No best, will simply create new grid with {}", word);
                let mut grid = WordGrid::new();
                grid.write(
                    Position { row: 0, column: 0 },
                    Orientation::Horizontal,
                    word,
                );
                grids.push(Rc::new(RefCell::new(grid)));

                remaining.remove_token(node_id);
            }
            Some(best) => {
                log::debug!(
                    "Best: {} {:?} {}. Cost = {:?}",
                    best.base,
                    best.orientation,
                    best.word,
                    best.cost()
                );
                best.grid
                    .borrow_mut()
                    .write(best.base, best.orientation, best.word);
                remaining.remove_token(best.node_id);
            }
        }

        log::debug!("Grids are:");
        for grid in &grids {
            log::debug!("\n{}", grid.borrow());
        }
    }
}

fn best_insert_word<'a>(
    best_so_far: &mut Option<InsertWord<'a>>,
    node_id: TokenSpecId,
    grid: &Rc<RefCell<WordGrid>>,
    word: &'a Word,
) {
    for (word_offset, &word_letter) in word.letters().iter().enumerate() {
        for (position, grid_letter) in grid.borrow().letters() {
            if word_letter == grid_letter {
                log::debug!("Found pivot {} for {} at {}", word_letter, word, position);
                best_pivot_word(
                    best_so_far,
                    node_id,
                    grid,
                    word,
                    word_offset as i32,
                    position,
                );
            }
        }
    }
}

fn best_pivot_word<'a>(
    best_so_far: &mut Option<InsertWord<'a>>,
    node_id: TokenSpecId,
    grid: &Rc<RefCell<WordGrid>>,
    word: &'a Word,
    word_offset: i32,
    position: Position,
) {
    for orientation in [
        Orientation::Vertical,
        Orientation::Horizontal,
        Orientation::Diagonal,
    ] {
        let base = position.advance(orientation, -word_offset);
        if let Some(stats) = grid.borrow().write_dry_run(base, orientation, word) {
            let new = InsertWord {
                grid: grid.clone(),
                node_id,
                word,
                base,
                orientation,
                stats,
            };

            match best_so_far {
                Some(prev_best) if new.cost() < prev_best.cost() => {
                    *best_so_far = Some(new);
                }
                None => {
                    *best_so_far = Some(new);
                }
                _ => {}
            };
        }
    }
}

impl<'a> InsertWord<'a> {
    fn cost(&self) -> (i32, i32) {
        (-self.stats.reused_letters, -self.stats.empty_neighbors)
    }
}
