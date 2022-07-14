use crate::models::grid::Grid;
use crate::models::positioned_token::Direction;
use crate::models::token::Token;
use crate::models::token_relations::TokenRelations;
use itertools::Itertools;
use std::fmt;

#[derive(Debug, Clone)]
pub struct GridBag {
    tokens: Vec<Token>,
    grids: Vec<Grid>,
}

impl GridBag {
    pub fn new(token: &Token) -> Self {
        let grids = if token.text.letters().len() == 1 {
            vec![Grid::new(token, Direction::Horizontal)]
        } else {
            vec![
                Grid::new(token, Direction::Horizontal),
                Grid::new(token, Direction::Vertical),
                Grid::new(token, Direction::Diagonal),
            ]
        };
        GridBag {
            tokens: vec![token.clone()],
            grids,
        }
    }

    pub fn with_inserted(
        &self,
        relations: &TokenRelations,
        token: &Token,
        trim_size: usize,
        max_width: i32,
        max_height: i32,
    ) -> Option<Self> {
        let mut new_grids = self
            .grids
            .iter()
            .flat_map(|grid| grid.enumerate_insertions(relations, token))
            .filter(|grid| {
                let (width, height) = grid.size();
                width <= max_width && height <= max_height
            })
            .collect_vec();

        if new_grids.len() > trim_size {
            // Take the grid with the least amount of letters, because they're usually more
            // interesting
            let weights = new_grids
                .iter()
                .map(|grid| grid.weight())
                .sorted()
                .collect_vec();
            let cutoff = weights[trim_size - 1];

            let initial_size = new_grids.len();
            new_grids.retain(|grid| grid.weight() <= cutoff);
            let final_size = new_grids.len();

            log::debug!(
                "Trimmed grid bag {} -> {} (weight <= {:?})",
                initial_size,
                final_size,
                cutoff
            );
        }

        if new_grids.is_empty() {
            None
        } else {
            let mut new_tokens = self.tokens.clone();
            new_tokens.push(token.clone());
            Some(GridBag {
                tokens: new_tokens,
                grids: new_grids,
            })
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
        // for (i, grid) in self.grids.iter().enumerate() {
        //     writeln!(f, "Grid {}/{}:\n{}", i, self.grids.len(), grid)?;
        // }
        // Ok(())
        write!(f, "{}", self.tokens.iter().format("\n"))
    }
}
