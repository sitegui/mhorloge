use crate::models::grid::Grid;
use crate::models::token::Token;
use crate::models::token_relations::TokenRelations;
use crate::AspectRatio;
use itertools::Itertools;
use rand::prelude::SliceRandom;
use rayon::prelude::*;
use std::{fmt, mem};

#[derive(Debug, Clone)]
pub struct GridBag {
    tokens: Vec<Token>,
    grids: Vec<Grid>,
    /// The target aspect ratio of this grid
    target_aspect: AspectRatio,
}

impl GridBag {
    pub fn new(target_aspect: AspectRatio) -> Self {
        GridBag {
            tokens: vec![],
            grids: vec![Grid::new()],
            target_aspect,
        }
    }

    pub fn insert(&mut self, relations: &TokenRelations, token: &Token, allow_diagonal: bool) {
        self.grids = self
            .grids
            .par_iter()
            .flat_map(|grid| grid.enumerate_insertions(relations, token, allow_diagonal))
            .collect();

        self.tokens.push(token.clone());
    }

    pub fn trim(&mut self, max_size: usize) {
        if self.grids.len() > max_size {
            let initial_size = self.grids.len();

            let mut grids = mem::take(&mut self.grids);
            grids.shuffle(&mut rand::thread_rng());
            grids.par_sort_unstable_by_key(|grid| self.weight_for_grid(grid));
            grids.truncate(max_size);
            self.grids = grids;

            let final_size = self.grids.len();

            log::debug!("Trimmed grid bag {} -> {}", initial_size, final_size);
        }
    }

    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    pub fn grids(&self) -> &[Grid] {
        &self.grids
    }

    pub fn best_grid(&self) -> &Grid {
        self.grids
            .iter()
            .min_by_key(|grid| self.weight_for_grid(grid))
            .unwrap()
    }

    /// A grid with lower weight is deemed more interesting
    fn weight_for_grid(&self, grid: &Grid) -> (i16, i16, i16) {
        let (width, height) = grid.size();
        let area = width * height;

        let (aspect_width, aspect_height) = self.target_aspect.cover(width, height);
        let aspect_area = aspect_width * aspect_height;

        (aspect_area, grid.num_letters(), area)
    }
}

impl fmt::Display for GridBag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.tokens.iter().format("\n"))
    }
}
