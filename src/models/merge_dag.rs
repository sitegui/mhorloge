use anyhow::{ensure, Result};
use petgraph::algo::DfsSpace;
use petgraph::dot::{Config, Dot};
use petgraph::prelude::{NodeIndex, StableDiGraph};
use petgraph::stable_graph::NodeReferences;
use petgraph::visit::IntoNodeReferences;
use petgraph::{algo, Direction};
use std::collections::BTreeMap;
use std::fmt::Display;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Represents a direct acyclic graph, whose nodes can be grouped together.
///
/// In a sense, this represents two DAGs that are related
#[derive(Debug, Clone)]
pub struct MergeDag<N: Node, G> {
    merged_graph: StableDiGraph<G, (), u16>,
    groups: BTreeMap<N::Id, NodeIndex<u16>>,
}

pub trait Node {
    type Id: Copy + Ord;

    fn id(&self) -> Self::Id;
}

pub trait Group<N> {
    fn new(n: N) -> Self;

    fn merge(&mut self, other: Self);
}

impl<N: Node, G: Group<N>> MergeDag<N, G> {
    pub fn new(nodes: Vec<N>, edges: &[(N::Id, N::Id)]) -> Self {
        let mut merged_graph = StableDiGraph::<G, (), u16>::default();
        let mut groups = BTreeMap::new();

        for node in nodes {
            let node_index = node.id();
            let group_id = merged_graph.add_node(G::new(node));
            let inserted = groups.insert(node_index, group_id).is_none();
            assert!(inserted);
        }

        for (a, b) in edges {
            let group_a = groups[a];
            let group_b = groups[b];
            merged_graph.update_edge(group_a, group_b, ());
        }

        MergeDag {
            merged_graph,
            groups,
        }
    }

    pub fn merge_groups(&mut self, node_a: N::Id, node_b: N::Id) -> Result<(), ()> {
        let group_a = self.groups[&node_a];
        let group_b = self.groups[&node_b];

        if !self.can_merge_groups(group_a, group_b) {
            return Err(());
        }

        // Update mapping of words, from `b` to `a`
        for targeted_token in self.groups.values_mut() {
            if *targeted_token == group_b {
                *targeted_token = group_a;
            }
        }

        // Copy all edges from `b` to `a`: incoming and outgoing
        let graph = &mut self.merged_graph;
        let mut neighbors = graph
            .neighbors_directed(group_b, Direction::Incoming)
            .detach();
        while let Some(neighbor) = neighbors.next_node(graph) {
            graph.update_edge(neighbor, group_a, ());
        }
        let mut neighbors = graph
            .neighbors_directed(group_b, Direction::Outgoing)
            .detach();
        while let Some(neighbor) = neighbors.next_node(graph) {
            graph.update_edge(group_a, neighbor, ());
        }

        // Merge groups
        let group = graph.remove_node(group_b).unwrap();
        graph[group_a].merge(group);

        Ok(())
    }

    pub fn nodes_len(&self) -> usize {
        self.groups.len()
    }

    pub fn group(&self, node: N::Id) -> (NodeIndex<u16>, &G) {
        let group = self.groups[&node];
        (group, &self.merged_graph[group])
    }

    pub fn groups(&self) -> NodeReferences<G, u16> {
        self.merged_graph.node_references()
    }

    pub fn groups_len(&self) -> usize {
        self.merged_graph.node_count()
    }

    pub fn dot(&self) -> String
    where
        G: Display,
    {
        let debug_graph = self
            .merged_graph
            .filter_map(|_, group| Some(group), |_, _| Some(""));

        Dot::with_config(&debug_graph, &[Config::EdgeNoLabel]).to_string()
    }

    /// Save the graph as a SVG file.
    ///
    /// This requires that a binary called `dot` be available. Tested with version 2.43.0.
    /// You can install it with the `graphviz` package.
    pub fn svg(&self, path: impl AsRef<Path>) -> Result<()>
    where
        G: Display,
    {
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

    /// Check if two tokens can be merged without creating a cycle
    fn can_merge_groups(&self, a: NodeIndex<u16>, b: NodeIndex<u16>) -> bool {
        if a == b {
            // Simple cases
            false
        } else {
            let space = &mut DfsSpace::new(&self.merged_graph);
            !algo::has_path_connecting(&self.merged_graph, a, b, Some(space))
                && !algo::has_path_connecting(&self.merged_graph, b, a, Some(space))
        }
    }
}
