use crate::models::grid::Grid;
use crate::models::token::Token;
use crate::models::token_relations::TokenRelations;
use itertools::Itertools;
use std::{fmt, mem};

#[derive(Debug, Clone)]
pub struct GridBag {
    tokens: Vec<Token>,
    grids: Vec<Grid>,
}

impl GridBag {
    pub fn new() -> Self {
        GridBag {
            tokens: vec![],
            grids: vec![Grid::default()],
        }
    }

    pub fn insert(&mut self, relations: &TokenRelations, token: &Token, allow_diagonal: bool) {
        self.grids = self
            .grids
            .iter()
            .flat_map(|grid| grid.enumerate_insertions(relations, token, allow_diagonal))
            .collect_vec();

        self.tokens.push(token.clone());
    }

    pub fn trim(&mut self, trim_size: usize) {
        if self.grids.len() > trim_size {
            let initial_size = self.grids.len();

            let mut grids = mem::take(&mut self.grids);
            grids.sort_by_key(|grid| grid.weight());
            grids.truncate(trim_size);
            self.grids = grids;

            let final_size = self.grids.len();

            log::debug!("Trimmed grid bag {} -> {}", initial_size, final_size,);
        }
    }

    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    pub fn grids(&self) -> &[Grid] {
        &self.grids
    }

    pub fn best_grid(&self) -> &Grid {
        self.grids.iter().min_by_key(|grid| grid.weight()).unwrap()
    }
}

impl fmt::Display for GridBag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.tokens.iter().format("\n"))
    }
}
