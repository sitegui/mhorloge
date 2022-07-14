use crate::models::merge_dag::MergeDag;
use crate::models::token::{Token, TokenId};
use crate::models::word::WordId;
use crate::Phrase;
use itertools::Itertools;

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
    pub fn new(graph: &MergeDag<WordId, Token>, phrases: &[Phrase]) -> Self {
        let max_token_id = graph.groups().map(|(_, token)| token.id).max().unwrap();
        let length = max_token_id.0 as usize + 1;

        // Create a matrix with all `None`s
        let mut relations = vec![];
        relations.resize_with(length, || vec![TokenRelation::None; length]);

        for phrase in phrases {
            for (&word_before, &word_after) in phrase.words.iter().tuple_windows::<(_, _)>() {
                let token_before = graph.group(word_before).1.id;
                let token_after = graph.group(word_after).1.id;
                relations[token_before.0 as usize][token_after.0 as usize] =
                    TokenRelation::IsBefore;
                relations[token_after.0 as usize][token_before.0 as usize] = TokenRelation::IsAfter;
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
