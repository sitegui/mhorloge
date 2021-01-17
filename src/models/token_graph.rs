use crate::models::phrase::{Phrase, PhraseSpec};
use crate::models::texts::{TextTag, Texts};
use itertools::Itertools;
use petgraph::algo;
use petgraph::algo::DfsSpace;
use petgraph::dot::{Config, Dot};
use petgraph::prelude::*;
use petgraph::visit::{IntoNodeReferences, Visitable, Walker};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

pub type TokenId = NodeIndex<u16>;

#[derive(Debug, Clone)]
pub struct TokenGraph<'a> {
    /// Each node represents a token. Each edge `A -> B` says that `A` must happen *before* `B`.
    /// The tokens are never removed from this graph, so their indexes are stable.
    graph: DiGraph<TokenSpec, (), u16>,
    /// Store the list of tokens (merged or not) by their text
    tokens_by_text: Arc<BTreeMap<TextTag, Vec<TokenId>>>,
    texts: &'a Texts,
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
    pub fn new(texts: &'a Texts, phrases: &'a [PhraseSpec]) -> Self {
        let mut graph = DiGraph::default();
        let mut tokens_by_text: BTreeMap<_, Vec<_>> = BTreeMap::new();

        for phrase in phrases {
            let mut prev_token = None;
            for &text in phrase.words() {
                let next_token = graph.add_node(TokenSpec::new(text));
                if let Some(prev_token) = prev_token {
                    graph.add_edge(prev_token, next_token, ());
                }
                prev_token = Some(next_token);

                tokens_by_text.entry(text).or_default().push(next_token);
            }
        }

        TokenGraph {
            texts,
            graph,
            phrases,
            tokens_by_text: Arc::new(tokens_by_text),
        }
    }

    /// Return the total number of letters used by concrete (that is, non-merged) tokens
    pub fn letters_len(&self) -> usize {
        (&self.graph)
            .node_references()
            .filter(|(_, node)| node.merged_with.is_none())
            .map(|(_, node)| node.text.len())
            .sum()
    }

    /// Return the total number of concrete (that is, non-merged) tokens
    pub fn tokens_len(&self) -> usize {
        (&self.graph)
            .node_references()
            .filter(|(_, node)| node.merged_with.is_none())
            .count()
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

    pub fn tokens_by_text(&self) -> &BTreeMap<TextTag, Vec<TokenId>> {
        self.tokens_by_text.as_ref()
    }

    pub fn get(&self, id: TokenId) -> TokenSpec {
        self.graph[id]
    }

    /// Merge the two given tokens
    pub fn merge_tokens(&mut self, a: TokenId, b: TokenId) {
        assert!(!self.graph[a].is_merged());
        assert!(!self.graph[b].is_merged());

        // Mark `b` as merged
        self.graph[b].merged_with = Some(a);

        // Copy all edges from `b` to `a`: incoming and outgoing
        let mut neighbors = self
            .graph
            .neighbors_directed(b, Direction::Incoming)
            .detach();
        while let Some(neighbor) = neighbors.next_node(&self.graph) {
            self.graph.update_edge(neighbor, a, ());
        }
        let mut neighbors = self
            .graph
            .neighbors_directed(b, Direction::Outgoing)
            .detach();
        while let Some(neighbor) = neighbors.next_node(&self.graph) {
            self.graph.update_edge(a, neighbor, ());
        }

        // Remove all incoming edges to `b`: it will be "disconnected" from the graph.
        // The outgoing edges can stay: they will no longer be used
        while let Some(edge) = self.graph.first_edge(b, Direction::Incoming) {
            self.graph.remove_edge(edge);
        }
    }

    /// Check if two tokens can be merged without creating a cycle
    pub fn can_merge_tokens(
        &self,
        a: TokenId,
        b: TokenId,
        dfs_space: &mut DfsSpace<TokenId, <DiGraph<TokenSpec, (), u16> as Visitable>::Map>,
    ) -> bool {
        a != b
            && !algo::has_path_connecting(&self.graph, a, b, Some(dfs_space))
            && !algo::has_path_connecting(&self.graph, b, a, Some(dfs_space))
    }

    pub fn dfs_space(&self) -> DfsSpace<TokenId, <DiGraph<TokenSpec, (), u16> as Visitable>::Map> {
        DfsSpace::new(&self.graph)
    }

    pub fn texts(&self) -> &'a Texts {
        self.texts
    }

    pub fn dot(&self) -> String {
        let debug_graph = self.graph.filter_map(
            |id, node| {
                if node.is_merged() {
                    None
                } else {
                    Some(format!("{}({})", self.texts.decode(node.text), id.index()))
                }
            },
            |_, _| Some(""),
        );

        Dot::with_config(&debug_graph, &[Config::EdgeNoLabel]).to_string()
    }
}

impl fmt::Display for TokenGraph<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "TokenGraph {{")?;

        let graph = &self.graph;
        let texts = self.texts;

        let mut merged = vec![];
        for source in graph.externals(Direction::Incoming) {
            if let Some(merged_with) = graph[source].merged_with {
                merged.push((source, merged_with));
            } else {
                let mut bfs = Bfs::new(graph, source);
                let first_id = bfs.next(graph).unwrap();
                write!(
                    f,
                    "\t{}({}): ",
                    texts.decode(graph[first_id].text),
                    first_id.index()
                )?;
                writeln!(
                    f,
                    "{}",
                    bfs.iter(graph).format_with(", ", |node, f| {
                        f(&format_args!(
                            "{}({})",
                            texts.decode(graph[node].text),
                            node.index()
                        ))
                    })
                )?;
            }
        }

        writeln!(
            f,
            "\tMerged: {}",
            merged
                .into_iter()
                .format_with(", ", |(source, merged_with), f| {
                    f(&format_args!(
                        "{}({}->{})",
                        texts.decode(graph[source].text),
                        source.index(),
                        merged_with.index()
                    ))
                })
        )?;

        writeln!(f, "}}")
    }
}

impl TokenSpec {
    pub fn new(text: TextTag) -> Self {
        TokenSpec {
            text,
            merged_with: None,
        }
    }

    pub fn is_merged(self) -> bool {
        self.merged_with.is_some()
    }
}
