use anyhow::{ensure, Result};
use itertools::Itertools;
use petgraph::algo::DfsSpace;
use petgraph::dot::{Config, Dot};
use petgraph::prelude::{EdgeRef, NodeIndex, StableDiGraph};
use petgraph::stable_graph::EdgeReference;
use petgraph::visit::{Dfs, EdgeFiltered, IntoEdgeReferences, IntoNodeReferences, Walker};
use petgraph::{algo, Direction};
use std::collections::BTreeMap;
use std::fmt::Display;
use std::fs;
use std::io::Write;
use std::ops::Index;
use std::path::Path;
use std::process::{Command, Stdio};

/// Represents a direct acyclic graph, whose nodes can be grouped together.
///
/// In a sense, this represents two DAGs that are related
#[derive(Debug, Clone)]
pub struct MergeDag<NodeId, Group> {
    merged_graph: StableDiGraph<Group, (), u16>,
    group_by_node: BTreeMap<NodeId, NodeIndex<u16>>,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct GroupId(NodeIndex<u16>);

impl<NodeId: Copy + Ord, Group> MergeDag<NodeId, Group> {
    pub fn new(seed_groups: Vec<(NodeId, Group)>, edges: &[(NodeId, NodeId)]) -> Self {
        let mut merged_graph = StableDiGraph::<Group, (), u16>::default();
        let mut group_by_node = BTreeMap::new();

        for (node_id, seed_group) in seed_groups {
            let group_id = merged_graph.add_node(seed_group);
            let inserted = group_by_node.insert(node_id, group_id).is_none();
            assert!(inserted);
        }

        for (a, b) in edges {
            let group_a = group_by_node[a];
            let group_b = group_by_node[b];
            merged_graph.update_edge(group_a, group_b, ());
        }

        MergeDag {
            merged_graph,
            group_by_node,
        }
    }

    /// Merge two groups together (as identified by one of their node ids). The `group_b` is
    /// removed from the graph and the `group_a` is modified in place by the callback `merge`.
    ///
    /// # Panic
    /// This will panic if both nodes are part of the same group
    pub fn merge_groups(
        &mut self,
        group_a: GroupId,
        group_b: GroupId,
        merge: impl FnOnce(&mut Group, Group),
    ) {
        assert_ne!(group_a, group_b);
        let group_a = group_a.0;
        let group_b = group_b.0;

        // Update mapping of groups, from `b` to `a`
        for target_group in self.group_by_node.values_mut() {
            if *target_group == group_b {
                *target_group = group_a;
            }
        }

        // Copy all edges from `b` to `a`: incoming and outgoing
        let graph = &mut self.merged_graph;
        let mut neighbors = graph
            .neighbors_directed(group_b, Direction::Incoming)
            .detach();
        while let Some(neighbor) = neighbors.next_node(graph) {
            if neighbor != group_a {
                graph.update_edge(neighbor, group_a, ());
            }
        }
        let mut neighbors = graph
            .neighbors_directed(group_b, Direction::Outgoing)
            .detach();
        while let Some(neighbor) = neighbors.next_node(graph) {
            if neighbor != group_a {
                graph.update_edge(group_a, neighbor, ());
            }
        }

        // Merge groups
        let group = graph.remove_node(group_b).unwrap();
        merge(&mut graph[group_a], group);
    }

    pub fn nodes_len(&self) -> usize {
        self.group_by_node.len()
    }

    pub fn group(&self, node: NodeId) -> (GroupId, &Group) {
        let group = self.group_by_node[&node];
        (GroupId(group), &self.merged_graph[group])
    }

    pub fn groups(&self) -> impl Iterator<Item = (GroupId, &Group)> {
        self.merged_graph
            .node_references()
            .map(|(id, group)| (GroupId(id), group))
    }

    pub fn groups_len(&self) -> usize {
        self.merged_graph.node_count()
    }

    pub fn group_edges(&self) -> impl Iterator<Item = (&Group, &Group)> {
        self.merged_graph.edge_references().map(move |edge| {
            let source = &self.merged_graph[edge.source()];
            let target = &self.merged_graph[edge.target()];
            (source, target)
        })
    }

    pub fn dot(&self) -> String
    where
        Group: Display,
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
        Group: Display,
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

    pub fn reachable_groups(&self, start: GroupId) -> impl Iterator<Item = GroupId> + '_ {
        Dfs::new(&self.merged_graph, start.0)
            .iter(&self.merged_graph)
            .map(GroupId)
    }

    /// Return if there is any path connecting the two groups
    pub fn has_path(&self, a: GroupId, b: GroupId) -> bool {
        let a = a.0;
        let b = b.0;

        let space = &mut DfsSpace::new(&self.merged_graph);
        algo::has_path_connecting(&self.merged_graph, a, b, Some(space))
            || algo::has_path_connecting(&self.merged_graph, b, a, Some(space))
    }

    /// Return if there is any indirect path connecting the two groups. Unlike
    /// [`MergeDag::has_path`], direct paths (`a` and `b` are direct neighbors) do not count.
    pub fn has_indirect_path(&self, a: GroupId, b: GroupId) -> bool {
        let a = a.0;
        let b = b.0;

        let graph_no_a_b = EdgeFiltered(&self.merged_graph, |edge: EdgeReference<(), u16>| {
            edge.source() != a || edge.target() != b
        });
        let graph_no_b_a = EdgeFiltered(&self.merged_graph, |edge: EdgeReference<(), u16>| {
            edge.source() != b || edge.target() != a
        });

        let space = &mut DfsSpace::new(&self.merged_graph);

        algo::has_path_connecting(&graph_no_a_b, a, b, Some(space))
            || algo::has_path_connecting(&graph_no_b_a, b, a, Some(space))
    }

    /// Return all groups with their depths. `0` means that this group has no "parent".
    pub fn group_depths(&self) -> Vec<(GroupId, usize)> {
        let mut result = vec![];

        // Create a new graph with the same topology
        let mut simple_graph = self.merged_graph.map(|_, _| (), |_, _| ());
        let mut depth = 0;

        // Detect and remove source nodes
        loop {
            let roots = simple_graph.externals(Direction::Incoming).collect_vec();

            for &root in &roots {
                result.push((GroupId(root), depth));
                simple_graph.remove_node(root);
            }

            if roots.is_empty() {
                break;
            }

            depth += 1;
        }

        result
    }
}

impl<NodeId, Group> Index<GroupId> for MergeDag<NodeId, Group> {
    type Output = Group;

    fn index(&self, index: GroupId) -> &Self::Output {
        &self.merged_graph[index.0]
    }
}
