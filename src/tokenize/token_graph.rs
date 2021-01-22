use crate::tokenize::phrase::PhraseSpec;
use crate::tokenize::texts::{TextTag, Texts};
use crate::tokenize::{TokenId, TokenizeOut, TokenizeOutEl, TokenizeOutPhraseEl};
use itertools::Itertools;
use petgraph::algo;
use petgraph::algo::DfsSpace;
use petgraph::dot::{Config, Dot};
use petgraph::prelude::*;
use petgraph::visit::{IntoNodeReferences, Visitable, Walker};
use std::cell::{Cell, RefCell, RefMut};
use std::collections::BTreeMap;
use std::fmt;

pub type TokenSpecId = NodeIndex<u16>;
type TokenGraphDfsSpace = DfsSpace<TokenSpecId, <DiGraph<TokenSpec, (), u16> as Visitable>::Map>;

#[derive(Debug, Clone)]
pub struct TokenGraph<'a> {
    /// Each node represents a token. Each edge `A -> B` says that `A` must happen *before* `B`.
    /// The tokens are never removed from this graph, so their indexes are stable.
    graph: DiGraph<TokenSpec, (), u16>,
    texts: &'a Texts,
    phrases: &'a [PhraseSpec],
    /// Number of letters of the last "check point"
    initial_letters: usize,
    weight: Cell<Option<f64>>,
    dfs_space: RefCell<Option<TokenGraphDfsSpace>>,
    tokens_by_text: RefCell<Option<BTreeMap<TextTag, Vec<TokenSpecId>>>>,
}

/// A token that is not yet known to be present in the final puzzle, unlike [`Token`].
#[derive(Debug, Clone, Copy)]
pub struct TokenSpec {
    text: TextTag,
    merged_with: Option<TokenSpecId>,
}

impl<'a> TokenGraph<'a> {
    pub fn new(texts: &'a Texts, phrases: &'a [PhraseSpec]) -> Self {
        let mut graph = DiGraph::default();
        let mut initial_letters = 0;

        for phrase in phrases {
            let mut prev_token = None;
            for &text in phrase.words() {
                let next_token = graph.add_node(TokenSpec::new(text));
                if let Some(prev_token) = prev_token {
                    graph.add_edge(prev_token, next_token, ());
                }
                prev_token = Some(next_token);
                initial_letters += text.len();
            }
        }

        TokenGraph {
            texts,
            graph,
            phrases,
            initial_letters,
            weight: Cell::new(None),
            dfs_space: RefCell::new(None),
            tokens_by_text: RefCell::new(None),
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

    /// Return the weight associated with this graph. The greater the weight, the better this graph
    /// is at having a simpler representation of the phrases.
    ///
    /// It's defined as `1 + (saved letters) ^ 2`.
    pub fn weight(&self) -> f64 {
        // The result is cached
        self.weight.get().unwrap_or_else(|| {
            let saved_letters = self.initial_letters - self.letters_len();
            let weight = 1.0 + (saved_letters as f64).powi(2);
            self.weight.set(Some(weight));
            weight
        })
    }

    pub fn get(&self, id: TokenSpecId) -> TokenSpec {
        self.graph[id]
    }

    /// Merge the two given tokens
    pub fn with_merged_tokens(&self, a: TokenSpecId, b: TokenSpecId) -> TokenGraph<'a> {
        let mut result = TokenGraph {
            graph: self.graph.clone(),
            texts: self.texts,
            phrases: self.phrases,
            initial_letters: self.initial_letters,
            weight: Cell::new(None),
            dfs_space: RefCell::new(None),
            tokens_by_text: RefCell::new(None),
        };
        assert!(!result.graph[a].is_merged());
        assert!(!result.graph[b].is_merged());

        // Mark `b` as merged
        result.graph[b].merged_with = Some(a);

        // Copy all edges from `b` to `a`: incoming and outgoing
        let mut neighbors = result
            .graph
            .neighbors_directed(b, Direction::Incoming)
            .detach();
        while let Some(neighbor) = neighbors.next_node(&result.graph) {
            result.graph.update_edge(neighbor, a, ());
        }
        let mut neighbors = result
            .graph
            .neighbors_directed(b, Direction::Outgoing)
            .detach();
        while let Some(neighbor) = neighbors.next_node(&result.graph) {
            result.graph.update_edge(a, neighbor, ());
        }

        // Remove all incoming edges to `b`: it will be "disconnected" from the graph.
        // The outgoing edges can stay: they will no longer be used
        while let Some(edge) = result.graph.first_edge(b, Direction::Incoming) {
            result.graph.remove_edge(edge);
        }

        result
    }

    /// Check if two tokens can be merged without creating a cycle
    pub fn can_merge_tokens(&self, a: TokenSpecId, b: TokenSpecId) -> bool {
        if a == b {
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
            |_, node| {
                if node.is_merged() {
                    None
                } else {
                    Some(self.texts.decode(node.text))
                }
            },
            |_, _| Some(""),
        );

        Dot::with_config(&debug_graph, &[Config::EdgeNoLabel]).to_string()
    }

    /// Check point the current state, by dropping accumulated garbage and resetting the weight
    pub fn check_point(&mut self, initial_letters: usize) {
        self.graph.retain_edges(|graph, edge| {
            let source = graph.edge_endpoints(edge).unwrap().0;
            !graph[source].is_merged()
        });
        self.initial_letters = initial_letters;
        self.weight.set(None);
        *self.dfs_space.get_mut() = None;
    }

    pub fn phrases(&self) -> &'a [PhraseSpec] {
        self.phrases
    }

    /// Returns a map with the unmerged tokens grouped by text
    pub fn tokens_by_text(&self) -> RefMut<'_, BTreeMap<TextTag, Vec<TokenSpecId>>> {
        RefMut::map(self.tokens_by_text.borrow_mut(), |tokens_by_text| {
            tokens_by_text.get_or_insert_with(|| {
                let mut map: BTreeMap<_, Vec<_>> = BTreeMap::new();
                for (id, node) in (&self.graph).node_references() {
                    if !node.is_merged() {
                        map.entry(node.text).or_default().push(id);
                    }
                }
                map
            })
        })
    }

    pub fn into_output(self) -> TokenizeOut {
        // IntelliJ need some help with type inference
        let graph = &self.graph;

        // Create the final tokens: each unmerged spec represents a final token
        let tokens = graph
            .node_references()
            .filter(|(_, spec)| !spec.is_merged())
            .map(|(id, spec)| {
                // Collect all other ids reachable from this node
                let followed_by: Vec<_> = Bfs::new(graph, id)
                    .iter(graph)
                    .filter_map(|node| if node != id { Some(node.into()) } else { None })
                    .collect();

                TokenizeOutEl {
                    id: id.into(),
                    text: self.texts.decode(spec.text).to_owned(),
                    followed_by,
                }
            })
            .collect();

        // Map phrases to tokens: we will iterate over the words in the same sequence as when the
        // graph was created in `new`, so we know the token's id.
        let mut next_index = 0;
        let phrases = self
            .phrases
            .iter()
            .map(|phrase_spec| {
                let tokens = phrase_spec
                    .words()
                    .iter()
                    .map(|_| {
                        // Find the "root" token id
                        let mut token_id = TokenSpecId::new(next_index);
                        while let Some(merged_with) = graph[token_id].merged_with {
                            token_id = merged_with;
                        }

                        next_index += 1;
                        token_id.into()
                    })
                    .collect();
                TokenizeOutPhraseEl {
                    id: phrase_spec.id(),
                    tokens,
                }
            })
            .collect();

        TokenizeOut { phrases, tokens }
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

impl From<TokenSpecId> for TokenId {
    fn from(id: TokenSpecId) -> TokenId {
        TokenId(id.index() as u16)
    }
}
