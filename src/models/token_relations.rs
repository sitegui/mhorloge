use crate::models::merge_dag::MergeDag;
use crate::models::token::{Token, TokenId};
use crate::models::word::WordId;

/// Represents the relative positioning constraint between any pair of tokens
#[derive(Debug, Clone)]
pub struct TokenRelations {
    relations: Vec<Vec<TokenRelation>>,
}

#[derive(Debug, Clone, Copy)]
pub enum TokenRelation {
    IsBefore,
    IsAfter,
    None,
}

impl TokenRelations {
    pub fn new(graph: &MergeDag<WordId, Token>) -> Self {
        let max_token_id = graph.groups().map(|(_, token)| token.id).max().unwrap();
        let length = max_token_id.0 as usize + 1;

        // Create a matrix with all `None`s
        let mut relations = vec![];
        relations.resize_with(length, || vec![TokenRelation::None; length]);

        for (a_index, _) in graph.groups() {
            let a_id = graph[a_index].id;
            for b_index in graph.reachable_groups(a_index) {
                let b_id = graph[b_index].id;
                relations[a_id.0 as usize][b_id.0 as usize] = TokenRelation::IsBefore;
                relations[b_id.0 as usize][a_id.0 as usize] = TokenRelation::IsAfter;
            }
        }

        TokenRelations { relations }
    }

    /// Return the relation between these two tokens. For example, [`TokenRelation::IsBefore`] is
    /// returned if `a` must be positioned **before** `b`
    pub fn get(&self, a: TokenId, b: TokenId) -> TokenRelation {
        self.relations[a.0 as usize][b.0 as usize]
    }
}
