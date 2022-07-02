use crate::models::grid::{Direction, Grid};
use crate::models::token::{Token, TokenId};
use crate::models::token_relations::TokenRelations;
use itertools::Itertools;
use std::fmt;

#[derive(Debug, Clone)]
pub struct GridBag {
    tokens: Vec<TokenId>,
    grids: Vec<Grid>,
}

impl GridBag {
    pub fn new(token: &Token) -> Self {
        GridBag {
            tokens: vec![token.id],
            grids: vec![
                Grid::new(token, Direction::Horizontal),
                Grid::new(token, Direction::Vertical),
                Grid::new(token, Direction::Diagonal),
            ],
        }
    }

    pub fn with_inserted(&self, relations: &TokenRelations, token: &Token) -> Option<Self> {
        let new_grids = self
            .grids
            .iter()
            .flat_map(|grid| grid.enumerate_insertions(relations, token))
            .collect_vec();

        if new_grids.is_empty() {
            None
        } else {
            let mut new_tokens = self.tokens.clone();
            new_tokens.push(token.id);
            Some(GridBag {
                tokens: new_tokens,
                grids: new_grids,
            })
        }
    }

    pub fn tokens(&self) -> &[TokenId] {
        &self.tokens
    }

    pub fn grids(&self) -> &[Grid] {
        &self.grids
    }
}

impl fmt::Display for GridBag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, grid) in self.grids.iter().enumerate() {
            writeln!(f, "Grid {}/{}:\n{}", i, self.grids.len(), grid)?;
        }
        Ok(())
    }
}
