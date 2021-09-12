use crate::models::phrase::Phrase;
use crate::models::words::{WordTag, Words};
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
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub type TokenSpecId = NodeIndex<u16>;
type TokenGraphDfsSpace = DfsSpace<TokenSpecId, <DiGraph<WordTag, (), u16> as Visitable>::Map>;

#[derive(Debug, Clone)]
pub struct TokenGraph<'a> {
    /// Each node represents a token in one or multiple phrases.
    /// Each edge `A -> B` says that `A` must happen *before* `B`.
    graph: StableDiGraph<WordTag, (), u16>,
    words: &'a Words,
    phrases: &'a [Phrase],
    /// Map each word location into the graph token that represents it.
    /// Multiple words with the same text can be mapped to the same token.
    word_locations: BTreeMap<PhrasedWordId, TokenSpecId>,
    dfs_space: RefCell<Option<TokenGraphDfsSpace>>,
}

impl<'a> TokenGraph<'a> {
    pub fn new(words: &'a Words, phrases: &'a [Phrase]) -> Self {
        let mut graph = StableDiGraph::default();
        let mut word_locations = BTreeMap::new();

        for phrase in phrases {
            let mut prev_token = None;
            for (word_index, &word_tag) in phrase.word_tags().iter().enumerate() {
                let next_token = graph.add_node(word_tag);

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
            words,
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
            .map(|(_, word_tag)| word_tag.len())
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
        if a == b || self.graph[a] != self.graph[b] {
            // Simple cases
            false
        } else {
            let mut space = self.dfs_space.borrow_mut();
            let space = space.get_or_insert_with(|| DfsSpace::new(&self.graph));
            !algo::has_path_connecting(&self.graph, a, b, Some(space))
                && !algo::has_path_connecting(&self.graph, b, a, Some(space))
        }
    }

    pub fn words(&self) -> &'a Words {
        self.words
    }

    pub fn dot(&self) -> String {
        let debug_graph = self.graph.filter_map(
            |_, &word_tag| Some(self.words.decode(word_tag)),
            |_, _| Some(""),
        );

        Dot::with_config(&debug_graph, &[Config::EdgeNoLabel]).to_string()
    }

    /// Save the graph as a SVG file.
    ///
    /// This requires that a binary called `dot` be available. Tested with version 2.43.0.
    /// You can install it with the `graphviz` package.
    pub fn svg(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut command = Command::new("dot");
        command
            .args(&["-T", "svg", "-Gsplines=ortho", "-o"])
            .arg(path);
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

    pub fn graph(&self) -> &StableDiGraph<WordTag, (), u16> {
        &self.graph
    }

    pub fn remove_token(&mut self, id: TokenSpecId) {
        self.graph.remove_node(id);
    }
}

impl fmt::Display for TokenGraph<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "TokenGraph {{")?;

        let graph = &self.graph;
        let words = self.words;

        for source in graph.externals(Direction::Incoming) {
            let mut bfs = Bfs::new(graph, source);
            let first_id = bfs.next(graph).unwrap();
            write!(
                f,
                "\t{}({}): ",
                words.decode(graph[first_id]),
                first_id.index()
            )?;
            writeln!(
                f,
                "{}",
                bfs.iter(graph).format_with(", ", |node, f| {
                    f(&format_args!(
                        "{}({})",
                        words.decode(graph[node]),
                        node.index()
                    ))
                })
            )?;
        }

        writeln!(f, "}}")
    }
}
