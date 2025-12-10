use std::collections::HashMap;

use bevy::prelude::Entity;
use petgraph::algo::isomorphism::is_isomorphic_subgraph_matching;
use petgraph::graph::{DiGraph, NodeIndex};

use crate::circuit::graph::{CircuitEdge, CircuitGraphQuery, CircuitNode};

// A motif is a collection of parts in a specific configuration (a small subgraph)
// a.k.a. subcircuit, pattern, template. These are matched via subgraph isomorphism.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeKind {
    Branch,
    Pin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    WireSegment,
    Via,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeLabel {
    pub kind: NodeKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EdgeLabel {
    pub kind: EdgeKind,
}

impl From<&CircuitNode> for NodeKind {
    fn from(node: &CircuitNode) -> Self {
        match node {
            CircuitNode::Branch => NodeKind::Branch,
            CircuitNode::Pin => NodeKind::Pin,
        }
    }
}

impl From<&CircuitEdge> for EdgeKind {
    fn from(edge: &CircuitEdge) -> Self {
        match edge {
            CircuitEdge::WireSegment => EdgeKind::WireSegment,
            CircuitEdge::Via => EdgeKind::Via,
        }
    }
}

pub struct Motif {
    pub name: String,
    pub description: String,
    pub graph: DiGraph<NodeLabel, EdgeLabel>,
}

impl Motif {
    pub fn new(name: impl Into<String>, description: impl Into<String>, graph: DiGraph<NodeLabel, EdgeLabel>) -> Self {
        Self { name: name.into(), description: description.into(), graph }
    }
}

/// Convert the ECS `CircuitGraph` into a labeled petgraph `DiGraph` using caller-provided labelers.
/// Returns the graph and an index mapping from `Entity` -> `NodeIndex`.
pub fn to_petgraph<NF, EF>(
    graph: &CircuitGraphQuery,
    mut node_labeler: NF,
    mut edge_labeler: EF,
) -> (DiGraph<NodeLabel, EdgeLabel>, HashMap<Entity, NodeIndex>)
where
    NF: FnMut(Entity) -> NodeLabel,
    EF: FnMut(Entity) -> EdgeLabel,
{
    let mut g: DiGraph<NodeLabel, EdgeLabel> = DiGraph::new();
    let mut entity_to_index: HashMap<Entity, NodeIndex> = HashMap::new();

    // Add nodes
    for (entity, _outgoing, _incoming) in graph.nodes_iter() {
        let idx = g.add_node(node_labeler(entity));
        entity_to_index.insert(entity, idx);
    }

    // Add edges
    for (edge_entity, from, to) in graph.edges_iter() {
        if let (Some(&from_idx), Some(&to_idx)) = (entity_to_index.get(&from.0), entity_to_index.get(&to.0)) {
            g.add_edge(from_idx, to_idx, edge_labeler(edge_entity));
        }
    }

    (g, entity_to_index)
}

/// Convenience: equality-based subgraph existence test.
pub fn motif_exists_in_graph(motif: &Motif, target: &DiGraph<NodeLabel, EdgeLabel>) -> bool {
    is_isomorphic_subgraph_matching(
        &motif.graph,
        target,
        |a: &NodeLabel, b: &NodeLabel| a.kind == b.kind,
        |a: &EdgeLabel, b: &EdgeLabel| a.kind == b.kind,
    )
}


// // 1) Create a small motif
// let mut motif_graph: DiGraph<NodeLabel, EdgeLabel> = DiGraph::new();
// let a = motif_graph.add_node(NodeLabel { kind: NodeKind::Pin });
// let b = motif_graph.add_node(NodeLabel { kind: NodeKind::Pin });
// motif_graph.add_edge(a, b, EdgeLabel { kind: EdgeKind::WireSegment });
// let motif = Motif::new("TwoPins", "Two pins connected", motif_graph);

// // 2) Convert current ECS world graph to petgraph with labels
// let (world_pg, _map) = to_petgraph(&graph,
//     |_entity| NodeLabel { kind: NodeKind::Pin /* or Branch based on your data */ },
//     |_edge_entity| EdgeLabel { kind: EdgeKind::WireSegment }
// );

// // 3) Test for existence
// let exists = motif_exists_in_graph(&motif, &world_pg);