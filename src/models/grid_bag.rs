use crate::models::grid::{Direction, Grid};
use crate::models::merge_dag::Group;
use crate::models::token::{Token, TokenId};

#[derive(Debug, Clone)]
pub struct GridBag {
    tokens: Vec<TokenId>,
    grids: Vec<Grid>,
}

impl Group<&'_ Token> for GridBag {
    fn new(token: &Token) -> Self {
        GridBag {
            tokens: vec![token.id],
            grids: vec![
                Grid::new(token, Direction::Horizontal),
                Grid::new(token, Direction::Vertical),
                Grid::new(token, Direction::Diagonal),
            ],
        }
    }

    fn merge(&mut self, other: Self) {
        self.tokens.extend(other.tokens);
    }
}
