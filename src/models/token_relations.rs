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

        for (index_a, _) in graph.groups() {
            let token_a = &graph[index_a];
            for index_b in graph.reachable_groups(index_a) {
                if index_a == index_b {
                    continue;
                }

                let token_b = &graph[index_b];
                log::debug!(
                    "{}({}) is before {}({})",
                    token_a,
                    token_a.id.0,
                    token_b,
                    token_b.id.0
                );
                relations[token_a.id.0 as usize][token_b.id.0 as usize] = TokenRelation::IsBefore;
                relations[token_b.id.0 as usize][token_a.id.0 as usize] = TokenRelation::IsAfter;
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
