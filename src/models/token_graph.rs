use crate::models::phrase::{Phrase, PhraseSpec};
use crate::models::texts::TextTag;
use petgraph::prelude::*;
use petgraph::visit::{IntoNodeReferences, Walker};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub type TokenId = NodeIndex<u16>;

#[derive(Debug, Clone)]
pub struct TokenGraph<'a> {
    /// Each node represents a token. Each edge `A -> B` says that `A` must happen *before* `B`.
    /// The tokens are never removed from this graph, so their indexes are stable.
    graph: DiGraph<TokenSpec, (), u16>,
    phrases: &'a [PhraseSpec],
}

/// A token that is not yet known to be present in the final puzzle, unlike [`Token`].
#[derive(Debug, Clone, Copy)]
pub struct TokenSpec {
    text: TextTag,
    merged_with: Option<TokenId>,
}

/// A concrete token, that will at some point be spatially placed in the puzzle
#[derive(Debug, Clone)]
pub struct Token {
    id: TokenId,
    text: TextTag,
    /// All concrete token ids that must be spatially placed **after** this one
    followed_by: BTreeSet<TokenId>,
}

impl<'a> TokenGraph<'a> {
    pub fn new(phrases: &'a [PhraseSpec]) -> Self {
        let mut graph = DiGraph::default();

        for phrase in phrases {
            let mut prev_token = None;
            for &text in phrase.words() {
                let next_token = graph.add_node(TokenSpec::new(text));

                if let Some(prev_token) = prev_token {
                    graph.add_edge(prev_token, next_token, ());
                }

                prev_token = Some(next_token);
            }
        }

        TokenGraph { graph, phrases }
    }

    pub fn into_phrases(self) -> Vec<Phrase> {
        // IntelliJ need some help with type inference
        let graph = &self.graph;

        // Create the final tokens: each unmerged spec represents a final token
        let mut tokens = BTreeMap::new();
        for (id, spec) in graph.node_references() {
            if spec.merged_with.is_none() {
                // Collect all other ids reachable from this node
                let mut followed_by: BTreeSet<_> = Bfs::new(graph, id).iter(graph).collect();
                followed_by.remove(&id);

                tokens.insert(
                    id,
                    Arc::new(Token {
                        id,
                        text: spec.text,
                        followed_by,
                    }),
                );
            }
        }

        // Map phrases to tokens: we will iterate over the words in the same sequence as when the
        // graph was created in `new`, so we know the token's id.
        let mut next_index = 0;
        self.phrases
            .iter()
            .map(|phrase_spec| {
                let words = phrase_spec
                    .words()
                    .iter()
                    .map(|_| {
                        {
                            // Find the "root" token id
                            let mut token_id = TokenId::new(next_index);
                            while let Some(merged_with) = graph[token_id].merged_with {
                                token_id = merged_with;
                            }

                            next_index += 1;
                            tokens[&token_id].clone()
                        }
                    })
                    .collect();
                Phrase::new(words)
            })
            .collect()
    }
}

impl TokenSpec {
    pub fn new(text: TextTag) -> Self {
        TokenSpec {
            text,
            merged_with: None,
        }
    }
}
