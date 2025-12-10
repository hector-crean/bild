use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// [ Node A ]                                   [ Edge e1 ]                                   [ Node B ]
// - OutgoingEdges: [e1, e2, ...]               - EdgeFrom(A)                                 - IncomingEdges: [e1, ...]
// - IncomingEdges: [ ... ]                     - EdgeTo(B)                                   - OutgoingEdges: [ ... ]
// - (your domain comps, e.g., Pin/Part/Net)    - (optional payloads)                         - (your domain comps)

/// What kind of schematic "edge" this entity represents.
/// Attach to `CircuitEdge` entities; geometry (e.g., Polyline) lives alongside this.
#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize, Default)]
#[reflect(Component, Serialize, Deserialize)]
pub enum CircuitEdge {
    #[default]
    WireSegment,
    Via,
}

// Core junctions
#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize, Default)]
#[reflect(Component, Serialize, Deserialize)]
#[require(Transform, GlobalTransform, IncomingEdges, OutgoingEdges)]
pub enum CircuitNode {
    #[default]
    Branch, // tee/merge dot
    Pin,
}

#[derive(Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub enum NetScope {
    Local,        // within the current sheet
    Hierarchical, // across hierarchy via sheet ports
}

#[derive(Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub enum PortDirection {
    In,
    Out,
    InOut,
}

// Separate annotation components (attach to CircuitNode entities)
#[derive(Component, Debug, Clone)]
pub struct NetLabel {
    pub name: String,
    pub scope: NetScope,
}

#[derive(Component, Debug, Clone)]
pub struct GlobalLabel {
    pub name: String,
}

#[derive(Component, Debug, Clone)]
pub struct Port {
    pub name: String,
    pub direction: PortDirection,
}

#[derive(Component, Debug, Clone)]
pub struct TestPoint {
    pub label: Option<String>,
}

#[derive(Component, Debug, Clone)]
pub struct NoConnect;

// ============================================================================
// GENERIC GRAPH RELATIONSHIP COMPONENTS
// ============================================================================

/// Edge component: indicates the source node (`from`).
#[derive(Component, Debug, Clone, Copy)]
#[relationship(relationship_target = OutgoingEdges)]
#[require(CircuitEdge)]
pub struct EdgeFrom(#[relationship] pub Entity);

/// Edge component: indicates the target node (`to`).
#[derive(Component, Debug, Clone, Copy)]
#[relationship(relationship_target = IncomingEdges)]
#[require(CircuitEdge)]
pub struct EdgeTo(#[relationship] pub Entity);

/// Optional: display color hint for an edge (used by gizmos/UI)
#[derive(Component, Debug, Clone, Copy)]
pub struct EdgeColor(pub Color);

/// Optional: weight/cost associated with an edge (used for algorithms)
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct EdgeWeight(pub f32);

/// Transform of the start node (from EdgeFrom). Automatically propagated from the source node.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct EdgeStartTransform(pub Transform);

/// Transform of the end node (from EdgeTo). Automatically propagated from the target node.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct EdgeEndTransform(pub Transform);

// ============================================================================
// OPINIONATED MARKERS FOR CIRCUIT-STYLE GRAPHS
// ============================================================================

/// Reverse index: all edges that start at a node (outgoing).
#[derive(Component, Debug, Default)]
#[relationship_target(relationship = EdgeFrom)]
pub struct OutgoingEdges(Vec<Entity>);

impl OutgoingEdges {
    pub fn iter(&self) -> impl ExactSizeIterator<Item = Entity> + '_ {
        self.0.iter().copied()
    }
}

/// Reverse index: all edges that end at a node (incoming).
#[derive(Component, Debug, Default)]
#[relationship_target(relationship = EdgeTo)]
pub struct IncomingEdges(Vec<Entity>);

impl IncomingEdges {
    pub fn iter(&self) -> impl ExactSizeIterator<Item = Entity> + '_ {
        self.0.iter().copied()
    }
}
