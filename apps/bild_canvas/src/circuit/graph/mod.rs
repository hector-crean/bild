pub mod components;
pub mod queries;
pub mod commands;
pub mod propagate;
pub mod render;

// Re-export commonly used items for convenience
pub use components::*;
pub use queries::*;
pub use commands::*;
pub use propagate::*;

use bevy::prelude::*;

/// Change set tracking what properties changed on a node
#[derive(Clone, Debug, Reflect, Default)]
pub struct NodeChangeSet {
    pub transform_changed: bool,
    pub global_transform_changed: bool,
    pub layer_id_changed: bool,
}

/// Change set tracking what properties changed on an edge
#[derive(Clone, Debug, Reflect, Default)]
pub struct EdgeChangeSet {
    pub edge_kind_changed: bool,
    pub color_changed: bool,
    pub weight_changed: bool,
    pub start_transform_changed: bool,
    pub end_transform_changed: bool,
}

/// Type of graph propagation event
#[derive(Clone, Debug, Reflect, PartialEq, Eq)]
pub enum PropagationType {
    ImmediateNeighbors,
    ConnectedComponent,
    Downstream,
    Upstream,
    LimitedDepth { depth: usize },
}

#[derive(Message, Event, Clone, Debug, Reflect)]
pub enum CircuitGraphMessage {
    // Structure changes
    NodeAdded {
        entity: Entity,
        initial_neighbors: Vec<Entity>,
    },
    NodeRemoved {
        entity: Entity,
        affected_edges: Vec<Entity>,
    },
    EdgeAdded {
        entity: Entity,
        from: Entity,
        to: Entity,
    },
    EdgeRemoved {
        entity: Entity,
        from: Entity,
        to: Entity,
    },
    
    // Topology changes (affects graph structure)
    NodeConnected {
        node: Entity,
        edge: Entity,
        neighbor: Entity,
    },
    NodeDisconnected {
        node: Entity,
        edge: Entity,
        neighbor: Entity,
    },
    
    // Property changes
    NodeChanged {
        entity: Entity,
        affected_neighbors: Vec<Entity>,
        changes: NodeChangeSet,
    },
    EdgeChanged {
        entity: Entity,
        from: Entity,
        to: Entity,
        changes: EdgeChangeSet,
    },
    
    // Propagation events (for graph algorithms)
    PropagationTriggered {
        source: Entity,
        affected_nodes: Vec<Entity>,
        propagation_type: PropagationType,
    },

}

#[derive(Message, Event, Clone, Debug, Reflect)]
pub enum CircuitGraphCommand {
    AddNode(Entity),
    RemoveNode(Entity),
    AddEdge(Entity),
    RemoveEdge(Entity),
}

use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use petgraph::{
    Directed,
    graph::{EdgeIndex, NodeIndex},
    visit::EdgeRef,
};

#[derive(Clone)]
pub struct NodeState<N: Debug> {
    _phantom: PhantomData<N>,
}

impl<N: Debug> NodeState<N> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct EdgeState<E: Debug> {
    _phantom: PhantomData<E>,
}

impl<E: Debug> EdgeState<E> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

#[derive(Clone, Resource)]
pub struct Graph<N: Debug, E: Debug>(pub petgraph::Graph<NodeState<N>, EdgeState<E>, Directed>);

impl<N, E> Default for Graph<N, E>
where
    N: Debug + Send + Sync + 'static,
    E: Debug + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<N, E> Deref for Graph<N, E>
where
    N: Debug,
    E: Debug,
{
    type Target = petgraph::Graph<NodeState<N>, EdgeState<E>, Directed>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<N, E> DerefMut for Graph<N, E>
where
    N: Debug,
    E: Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<N, E> Graph<N, E>
where
    N: Debug + Send + Sync + 'static,
    E: Debug + Send + Sync + 'static,
{
    pub fn inner(&self) -> &petgraph::Graph<NodeState<N>, EdgeState<E>, Directed> {
        &self.0
    }

    pub fn new() -> Self {
        Self(petgraph::Graph::new())
    }

    /// Call this from your `on_node_added` system
    pub fn register_node(
        &mut self,
        index: &mut GraphEntityIndex<N, E>,
        entity: Entity,
        state: NodeState<N>,
    ) -> Option<NodeIndex> {
        if index.node_of_entity.contains_key(&entity) {
            return None;
        }

        let ni = self.add_node(state);
        index.node_of_entity.insert(entity, ni);
        index.entity_of_node.insert(ni, entity);
        Some(ni)
    }

    /// Call this from your `on_node_removed` system
    /// Returns a list of edge entities that were removed as a result of removing this node
    pub fn unregister_node(
        &mut self,
        index: &mut GraphEntityIndex<N, E>,
        entity: Entity,
    ) -> Vec<Entity> {
        use petgraph::Direction;
        let mut removed_edges = Vec::new();
        if let Some(ni) = index.node_of_entity.remove(&entity) {
            index.entity_of_node.remove(&ni);

            // Clean up edge mappings for edges that are about to be destroyed by remove_node
            for edge_ref in self
                .edges_directed(ni, Direction::Outgoing)
                .collect::<Vec<_>>()
            {
                let eid = edge_ref.id();
                if let Some(ent) = index.entity_of_edge.remove(&eid) {
                    index.edge_of_entity.remove(&ent);
                    removed_edges.push(ent);
                }
            }
            for edge_ref in self
                .edges_directed(ni, Direction::Incoming)
                .collect::<Vec<_>>()
            {
                let eid = edge_ref.id();
                if let Some(ent) = index.entity_of_edge.remove(&eid) {
                    index.edge_of_entity.remove(&ent);
                    removed_edges.push(ent);
                }
            }
            let _ = self.remove_node(ni);
        }
        removed_edges
    }

    /// Call this from your `on_edge_added` or `changed` system
    pub fn sync_edge(
        &mut self,
        index: &mut GraphEntityIndex<N, E>,
        edge_entity: Entity,
        from_entity: Entity,
        to_entity: Entity,
        state: EdgeState<E>,
    ) {
        // 1. Remove old mapping if exists (handle updates)
        if let Some(old_ei) = index.edge_of_entity.remove(&edge_entity) {
            index.entity_of_edge.remove(&old_ei);
            let _ = self.remove_edge(old_ei);
        }

        // 2. Validate endpoints exist in graph
        let Some(a) = index.node_of_entity.get(&from_entity).copied() else {
            return;
        };
        let Some(b) = index.node_of_entity.get(&to_entity).copied() else {
            return;
        };

        // 3. Add new edge
        let ei = self.add_edge(a, b, state);
        index.edge_of_entity.insert(edge_entity, ei);
        index.entity_of_edge.insert(ei, edge_entity);
    }

    /// Call this from `on_edge_removed`
    pub fn unregister_edge(&mut self, index: &mut GraphEntityIndex<N, E>, edge_entity: Entity) {
        if let Some(ei) = index.edge_of_entity.remove(&edge_entity) {
            index.entity_of_edge.remove(&ei);
            let _ = self.remove_edge(ei);
        }
    }
}

#[derive(Resource)]
pub struct GraphEntityIndex<N: Debug + Send + Sync + 'static, E: Debug + Send + Sync + 'static> {
    pub node_of_entity: std::collections::HashMap<Entity, NodeIndex>,
    pub entity_of_node: std::collections::HashMap<NodeIndex, Entity>,
    pub edge_of_entity: std::collections::HashMap<Entity, EdgeIndex>,
    pub entity_of_edge: std::collections::HashMap<EdgeIndex, Entity>,
    _phantom: PhantomData<(N, E)>,
}

impl<N, E> Default for GraphEntityIndex<N, E>
where
    N: Debug + Send + Sync + 'static,
    E: Debug + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            node_of_entity: Default::default(),
            entity_of_node: Default::default(),
            edge_of_entity: Default::default(),
            entity_of_edge: Default::default(),
            _phantom: PhantomData,
        }
    }
}

/// Configuration for graph event propagation behavior
#[derive(Resource, Clone, Debug, Reflect)]
pub struct GraphEventPropagationConfig {
    /// Enable automatic propagation for node changes
    pub propagate_node_changes: bool,
    /// Enable automatic propagation for edge changes
    pub propagate_edge_changes: bool,
    /// Default propagation mode for node changes
    pub default_node_propagation: PropagationType,
    /// Default propagation mode for edge changes
    pub default_edge_propagation: PropagationType,
}

impl Default for GraphEventPropagationConfig {
    fn default() -> Self {
        Self {
            propagate_node_changes: true,
            propagate_edge_changes: false,
            default_node_propagation: PropagationType::ImmediateNeighbors,
            default_edge_propagation: PropagationType::ImmediateNeighbors,
        }
    }
}

/// System sets for ordering graph systems
#[derive(SystemSet, Clone, Debug, Hash, PartialEq, Eq)]
pub enum GraphSystemSet {
    /// First: Detect changes to graph structure and components
    ChangeDetection,
    /// Second: Generate events from detected changes
    EventGeneration,
    /// Third: Propagate events through the graph
    EventPropagation,
    /// Fourth: Sync with petgraph resource (for solver compatibility)
    GraphSync,
    /// Last: For other plugins to consume events
    Consumption,
}

pub struct CircuitGraphManagerPlugin;

impl Plugin for CircuitGraphManagerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CircuitGraphMessage>()
            .register_type::<NodeChangeSet>()
            .register_type::<EdgeChangeSet>()
            .register_type::<PropagationType>()
            .add_message::<CircuitGraphMessage>()
            .register_type::<CircuitGraphCommand>()
            .add_message::<CircuitGraphCommand>()
            .init_resource::<Graph<CircuitNode, CircuitEdge>>()
            .init_resource::<GraphEntityIndex<CircuitNode, CircuitEdge>>()
            .init_resource::<GraphEventPropagationConfig>()
            .init_resource::<PropagationBuffer>()
            .add_systems(
                Update,
                (
                    // Change Detection
                    detect_node_changes,
                    detect_edge_changes,
                    detect_topology_changes,
                    detect_property_changes,
                )
                    .chain()
                    .in_set(GraphSystemSet::ChangeDetection),
            )
            .add_systems(
                Update,
                (
                    // Event Propagation (if enabled)
                    // Split into read and write to avoid resource conflicts
                    read_propagation_messages,
                    write_propagation_events,
                )
                    .chain()
                    .in_set(GraphSystemSet::EventPropagation)
                    .after(GraphSystemSet::ChangeDetection),
            )
            .add_systems(
                Update,
                (
                    // Graph Sync (petgraph)
                    sync_graph_nodes,
                    sync_graph_edges,
                )
                    .chain()
                    .in_set(GraphSystemSet::GraphSync)
                    .after(GraphSystemSet::EventPropagation),
            );
    }
}

// ============================================================================
// Change Detection Systems
// ============================================================================

/// Detect node additions and removals
fn detect_node_changes(
    graph_query: CircuitGraphQuery,
        mut graph_messages: MessageWriter<CircuitGraphMessage>,
    added_nodes: Query<CircuitNodeQueryData, Added<CircuitNode>>,
    mut removed_nodes: RemovedComponents<CircuitNode>,
) {
    // Handle added nodes
    for node_data in added_nodes.iter() {
        let neighbors = graph_query.affected_neighbors(node_data.entity);
        graph_messages.write(CircuitGraphMessage::NodeAdded {
            entity: node_data.entity,
            initial_neighbors: neighbors,
        });
    }

    // Handle removed nodes
    for entity in removed_nodes.read() {
        // Collect affected edges before they're removed
        let affected_edges = graph_query.connected_edges(entity);
        graph_messages.write(CircuitGraphMessage::NodeRemoved {
            entity,
            affected_edges,
        });
    }
}

/// Detect edge additions and removals
fn detect_edge_changes(
        mut graph_messages: MessageWriter<CircuitGraphMessage>,
    added_edges: Query<CircuitEdgeQueryData, Added<CircuitEdge>>,
    mut removed_edges: RemovedComponents<CircuitEdge>,
    edges: Query<(&EdgeFrom, &EdgeTo), With<CircuitEdge>>,
) {
    // Handle added edges
    for edge_data in added_edges.iter() {
        graph_messages.write(CircuitGraphMessage::EdgeAdded {
            entity: edge_data.entity,
            from: edge_data.edge_from.0,
            to: edge_data.edge_to.0,
        });
    }

    // Handle removed edges
    for entity in removed_edges.read() {
        // Try to get edge endpoints before removal
        if let Ok((from, to)) = edges.get(entity) {
            graph_messages.write(CircuitGraphMessage::EdgeRemoved {
                entity,
                from: from.0,
                to: to.0,
            });
        }
    }
}

/// Detect topology changes (relationship changes: EdgeFrom/EdgeTo modified)
fn detect_topology_changes(
        mut graph_messages: MessageWriter<CircuitGraphMessage>,
    changed_edges: Query<
        (Entity, &EdgeFrom, &EdgeTo),
        Or<(Changed<EdgeFrom>, Changed<EdgeTo>)>,
    >,
) {
    for (edge_entity, from, to) in changed_edges.iter() {
        // Emit connection events for both endpoints
        graph_messages.write(CircuitGraphMessage::NodeConnected {
            node: from.0,
            edge: edge_entity,
            neighbor: to.0,
        });
        graph_messages.write(CircuitGraphMessage::NodeConnected {
            node: to.0,
            edge: edge_entity,
            neighbor: from.0,
        });
    }
}

/// Detect property changes on nodes and edges
fn detect_property_changes(
    graph_query: CircuitGraphQuery,
    mut graph_messages: MessageWriter<CircuitGraphMessage>,
    changed_nodes: Query<
        CircuitNodeQueryData,
        Or<(Changed<Transform>, Changed<GlobalTransform>)>,
    >,
    changed_edges: Query<
        (Entity, &EdgeFrom, &EdgeTo),
        Or<(
            Changed<CircuitEdge>,
            Changed<EdgeColor>,
            Changed<EdgeWeight>,
            Changed<EdgeStartTransform>,
            Changed<EdgeEndTransform>,
        )>,
    >,
) {
    // Handle node property changes
    for node_data in changed_nodes.iter() {
        let mut changes = NodeChangeSet::default();
        // Note: We can't easily detect which specific component changed from QueryData alone,
        // so we mark all that could have changed. In practice, Changed<T> filters already
        // limit this to actual changes.
        changes.transform_changed = true;
        changes.global_transform_changed = true;
        changes.layer_id_changed = true;

        let affected_neighbors = graph_query.affected_neighbors(node_data.entity);
        graph_messages.write(CircuitGraphMessage::NodeChanged {
            entity: node_data.entity,
            affected_neighbors,
            changes,
        });
    }

    // Handle edge property changes
    for (edge_entity, from, to) in changed_edges.iter() {
        let mut changes = EdgeChangeSet::default();
        // Similar to nodes, mark all potential changes
        changes.edge_kind_changed = true;
        changes.color_changed = true;
        changes.weight_changed = true;
        changes.start_transform_changed = true;
        changes.end_transform_changed = true;

        graph_messages.write(CircuitGraphMessage::EdgeChanged {
            entity: edge_entity,
            from: from.0,
            to: to.0,
            changes,
        });
    }
}

// ============================================================================
// Event Propagation System
// ============================================================================

/// Resource to buffer propagation events to avoid Res/ResMut conflicts
#[derive(Resource, Default)]
struct PropagationBuffer {
    events: Vec<CircuitGraphMessage>,
}

/// Read messages and compute propagation targets (read-only phase)
fn read_propagation_messages(
    config: Res<GraphEventPropagationConfig>,
    graph_query: CircuitGraphQuery,
    mut reader: MessageReader<CircuitGraphMessage>,
    mut buffer: ResMut<PropagationBuffer>,
) {
    if !config.propagate_node_changes && !config.propagate_edge_changes {
        return;
    }

    buffer.events.clear();

    for message in reader.read() {
        match message {
            CircuitGraphMessage::NodeChanged { entity, affected_neighbors: _, changes: _ }
            | CircuitGraphMessage::NodeAdded { entity, initial_neighbors: _ } => {
                if config.propagate_node_changes {
                    let affected_nodes = match config.default_node_propagation {
                        PropagationType::ImmediateNeighbors => {
                            graph_query.affected_neighbors(*entity)
                        }
                        PropagationType::ConnectedComponent => {
                            graph_query.connected_component_undirected(*entity)
                        }
                        PropagationType::Downstream => {
                            graph_query.downstream_nodes(*entity)
                        }
                        PropagationType::Upstream => {
                            graph_query.upstream_nodes(*entity)
                        }
                        PropagationType::LimitedDepth { depth } => {
                            graph_query.affected_subgraph(*entity, Some(depth))
                        }
                    };

                    buffer.events.push(CircuitGraphMessage::PropagationTriggered {
                        source: *entity,
                        affected_nodes,
                        propagation_type: config.default_node_propagation.clone(),
                    });
                }
            }
            CircuitGraphMessage::EdgeChanged { entity, from, to, changes: _ } => {
                if config.propagate_edge_changes {
                    // For edges, propagate to both endpoints
                    let affected_nodes = match config.default_edge_propagation {
                        PropagationType::ImmediateNeighbors => {
                            let mut nodes = graph_query.affected_neighbors(*from);
                            nodes.extend(graph_query.affected_neighbors(*to));
                            nodes.sort();
                            nodes.dedup();
                            nodes
                        }
                        PropagationType::ConnectedComponent => {
                            // Union of both components
                            let mut from_comp = graph_query.connected_component_undirected(*from);
                            let to_comp = graph_query.connected_component_undirected(*to);
                            from_comp.extend(to_comp);
                            from_comp.sort();
                            from_comp.dedup();
                            from_comp
                        }
                        PropagationType::Downstream => {
                            let mut nodes = graph_query.downstream_nodes(*from);
                            nodes.extend(graph_query.downstream_nodes(*to));
                            nodes.sort();
                            nodes.dedup();
                            nodes
                        }
                        PropagationType::Upstream => {
                            let mut nodes = graph_query.upstream_nodes(*from);
                            nodes.extend(graph_query.upstream_nodes(*to));
                            nodes.sort();
                            nodes.dedup();
                            nodes
                        }
                        PropagationType::LimitedDepth { depth } => {
                            let mut nodes = graph_query.affected_subgraph(*from, Some(depth));
                            nodes.extend(graph_query.affected_subgraph(*to, Some(depth)));
                            nodes.sort();
                            nodes.dedup();
                            nodes
                        }
                    };

                    buffer.events.push(CircuitGraphMessage::PropagationTriggered {
                        source: *entity,
                        affected_nodes,
                        propagation_type: config.default_edge_propagation.clone(),
                    });
                }
            }
            _ => {
                // Other message types don't trigger propagation
            }
        }
    }
}

/// Write buffered propagation events (write-only phase)
fn write_propagation_events(
    mut graph_messages: MessageWriter<CircuitGraphMessage>,
    mut buffer: ResMut<PropagationBuffer>,
) {
    for event in buffer.events.drain(..) {
        graph_messages.write(event);
    }
}

// ============================================================================
// Graph Sync Systems (petgraph)
// ============================================================================

/// Sync added/removed nodes with petgraph resource
fn sync_graph_nodes(
    mut graph: ResMut<Graph<CircuitNode, CircuitEdge>>,
    mut index: ResMut<GraphEntityIndex<CircuitNode, CircuitEdge>>,
    mut reader: MessageReader<CircuitGraphMessage>,
) {
    for message in reader.read() {
        match message {
            CircuitGraphMessage::NodeAdded { entity, initial_neighbors: _ } => {
                graph.register_node(&mut index, *entity, NodeState::new());
            }
            CircuitGraphMessage::NodeRemoved { entity, affected_edges: _ } => {
                graph.unregister_node(&mut index, *entity);
            }
            _ => {}
        }
    }
}

/// Sync added/removed edges with petgraph resource
fn sync_graph_edges(
    mut graph: ResMut<Graph<CircuitNode, CircuitEdge>>,
    mut index: ResMut<GraphEntityIndex<CircuitNode, CircuitEdge>>,
    mut reader: MessageReader<CircuitGraphMessage>,
    edges: Query<(&EdgeFrom, &EdgeTo), With<CircuitEdge>>,
) {
    for message in reader.read() {
        match message {
            CircuitGraphMessage::EdgeAdded { entity, from, to } => {
                graph.sync_edge(&mut index, *entity, *from, *to, EdgeState::new());
            }
            CircuitGraphMessage::EdgeRemoved { entity, from: _, to: _ } => {
                graph.unregister_edge(&mut index, *entity);
            }
            CircuitGraphMessage::NodeConnected { node: _, edge, neighbor: _ } => {
                // Re-sync the edge when its topology changes
                if let Ok((from, to)) = edges.get(*edge) {
                    graph.sync_edge(&mut index, *edge, from.0, to.0, EdgeState::new());
                }
            }
            _ => {}
        }
    }
}
