use crate::models::texts::{TextTag, Texts};
use crate::tokenize::phrase::PhraseSpec;
use crate::tokenize::{PhrasedWordId, WordId};
use anyhow::{ensure, Result};
use itertools::Itertools;
use petgraph::algo;
use petgraph::algo::DfsSpace;
use petgraph::dot::{Config, Dot};
use petgraph::prelude::*;
use petgraph::visit::{IntoNodeReferences, Visitable, Walker};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub type TokenSpecId = NodeIndex<u16>;
type TokenGraphDfsSpace = DfsSpace<TokenSpecId, <DiGraph<TokenSpec, (), u16> as Visitable>::Map>;

#[derive(Debug, Clone)]
pub struct TokenGraph<'a> {
    /// Each node represents a token in one or multiple phrases.
    /// Each edge `A -> B` says that `A` must happen *before* `B`.
    graph: StableDiGraph<TokenSpec, (), u16>,
    texts: &'a Texts,
    phrases: &'a [PhraseSpec],
    /// Map each word location into the graph token that represents it.
    /// Multiple words with the same text can be mapped to the same token.
    word_locations: BTreeMap<PhrasedWordId, TokenSpecId>,
    dfs_space: RefCell<Option<TokenGraphDfsSpace>>,
}

/// A token that is not yet known to be present in the final puzzle, unlike [`Token`].
#[derive(Debug, Clone)]
pub struct TokenSpec {
    text_tag: TextTag,
}

impl<'a> TokenGraph<'a> {
    pub fn new(texts: &'a Texts, phrases: &'a [PhraseSpec]) -> Self {
        let mut graph = StableDiGraph::default();
        let mut word_locations = BTreeMap::new();

        for phrase in phrases {
            let mut prev_token = None;
            for (word_index, &text_tag) in phrase.words().iter().enumerate() {
                let token_spec = TokenSpec { text_tag };
                let next_token = graph.add_node(token_spec);

                if let Some(prev_token) = prev_token {
                    graph.add_edge(prev_token, next_token, ());
                }
                prev_token = Some(next_token);

                word_locations.insert(
                    PhrasedWordId {
                        phrase: phrase.id(),
                        word: WordId(word_index as u16),
                    },
                    next_token,
                );
            }
        }

        TokenGraph {
            texts,
            graph,
            phrases,
            word_locations,
            dfs_space: RefCell::new(None),
        }
    }

    /// Return the total number of letters
    pub fn letters_len(&self) -> usize {
        (&self.graph)
            .node_references()
            .map(|(_, node)| node.text_tag.len())
            .sum()
    }

    /// Return the total number tokens
    pub fn tokens_len(&self) -> usize {
        self.graph.node_count()
    }

    /// Merge the two given tokens, if possible.
    ///
    /// The node `a` will receive all words and connections from `b`, then `b` will be removed.
    pub fn merge_tokens(&mut self, a: TokenSpecId, b: TokenSpecId) -> Result<(), ()> {
        if !self.can_merge_tokens(a, b) {
            return Err(());
        }

        // Update mapping of words, from `b` to `a`
        for targeted_token in self.word_locations.values_mut() {
            if *targeted_token == b {
                *targeted_token = a;
            }
        }

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

        self.graph.remove_node(b);

        Ok(())
    }

    /// Check if two tokens can be merged without creating a cycle
    pub fn can_merge_tokens(&self, a: TokenSpecId, b: TokenSpecId) -> bool {
        if a == b || self.graph[a].text_tag != self.graph[b].text_tag {
            // Simple cases
            false
        } else {
            let mut space = self.dfs_space.borrow_mut();
            let space = space.get_or_insert_with(|| DfsSpace::new(&self.graph));
            !algo::has_path_connecting(&self.graph, a, b, Some(space))
                && !algo::has_path_connecting(&self.graph, b, a, Some(space))
        }
    }

    pub fn texts(&self) -> &'a Texts {
        self.texts
    }

    pub fn dot(&self) -> String {
        let debug_graph = self.graph.filter_map(
            |_, node| Some(self.texts.decode(node.text_tag)),
            |_, _| Some(""),
        );

        Dot::with_config(&debug_graph, &[Config::EdgeNoLabel]).to_string()
    }

    /// Save the graph as a SVG file.
    ///
    /// This requires that a binary called `dot` be available. Tested with version 2.43.0.
    /// You can install it with the `graphviz` package.
    pub fn svg(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut command = Command::new("dot");
        command
            .args(&["-T", "svg", "-Gsplines=ortho", "-o"])
            .arg(path.as_ref());
        if log::log_enabled!(log::Level::Debug) {
            command.arg("-v");
        }
        let mut dot = command.stdin(Stdio::piped()).spawn()?;

        dot.stdin
            .as_ref()
            .unwrap()
            .write_all(self.dot().as_bytes())?;

        ensure!(dot.wait()?.success(), "Failed to generate SVG");

        Ok(())
    }

    pub fn find_token(&self, location: PhrasedWordId) -> TokenSpecId {
        *self.word_locations.get(&location).unwrap()
    }

    // pub fn to_output(&self) -> TokenizeOut {
    //     // IntelliJ need some help with type inference
    //     let graph = &self.graph;
    //
    //     // Create the final tokens: each unmerged spec represents a final token
    //     let tokens = graph
    //         .node_references()
    //         .filter(|(_, spec)| !spec.is_merged())
    //         .map(|(id, spec)| {
    //             // Collect all other ids reachable from this node
    //             let followed_by: Vec<_> = Bfs::new(graph, id)
    //                 .iter(graph)
    //                 .filter_map(|node| if node != id { Some(node.into()) } else { None })
    //                 .collect();
    //
    //             TokenizeOutEl {
    //                 id: id.into(),
    //                 text: self.texts.decode(spec.text).to_owned(),
    //                 followed_by,
    //             }
    //         })
    //         .collect();
    //
    //     // Map phrases to tokens: we will iterate over the words in the same sequence as when the
    //     // graph was created in `new`, so we know the token's id.
    //     let mut next_index = 0;
    //     let phrases = self
    //         .phrases
    //         .iter()
    //         .map(|phrase_spec| {
    //             let tokens = phrase_spec
    //                 .words()
    //                 .iter()
    //                 .map(|_| {
    //                     // Find the "root" token id
    //                     let mut token_id = TokenSpecId::new(next_index);
    //                     while let Some(merged_with) = graph[token_id].merged_with {
    //                         token_id = merged_with;
    //                     }
    //
    //                     next_index += 1;
    //                     token_id.into()
    //                 })
    //                 .collect();
    //             TokenizeOutPhraseEl {
    //                 id: phrase_spec.id(),
    //                 tokens,
    //             }
    //         })
    //         .collect();
    //
    //     TokenizeOut { phrases, tokens }
    // }
}

impl fmt::Display for TokenGraph<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "TokenGraph {{")?;

        let graph = &self.graph;
        let texts = self.texts;

        for source in graph.externals(Direction::Incoming) {
            let mut bfs = Bfs::new(graph, source);
            let first_id = bfs.next(graph).unwrap();
            write!(
                f,
                "\t{}({}): ",
                texts.decode(graph[first_id].text_tag),
                first_id.index()
            )?;
            writeln!(
                f,
                "{}",
                bfs.iter(graph).format_with(", ", |node, f| {
                    f(&format_args!(
                        "{}({})",
                        texts.decode(graph[node].text_tag),
                        node.index()
                    ))
                })
            )?;
        }

        writeln!(f, "}}")
    }
}
