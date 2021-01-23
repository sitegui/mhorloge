use crate::tokenize::TokenId;

pub struct ClusterGraph {}

#[derive(Debug, Clone)]
pub struct Constraints {}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Constraint {
    pub coexist: bool,
    pub order: Order,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Order {
    AThenB,
    BThenA,
    None,
}

impl Constraints {
    pub fn get(&self, a: TokenId, b: TokenId) -> Constraint {
        todo!()
    }
}
