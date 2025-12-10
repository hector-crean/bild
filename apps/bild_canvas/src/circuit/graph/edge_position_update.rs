use bevy::prelude::*;

use super::{
    CircuitGraphMessage, GraphSystemSet,
    components::{CircuitNode, EdgeEndTransform, EdgeStartTransform},
    queries::CircuitGraphQuery,
};

/// Plugin that consumes `CircuitGraphMessage` events and updates edge positions.
///
/// This plugin listens to graph events and automatically updates `EdgeStartTransform`
/// and `EdgeEndTransform` components on edges when:
/// - Nodes are added or changed (transform updates)
/// - Edges are added (initial transform setup)
/// - Topology changes occur (node connections)
///
/// This is an event-driven alternative to the query-based `EdgeTransformPropagatePlugin`.
pub struct EdgePositionUpdatePlugin;

impl Plugin for EdgePositionUpdatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_edge_positions_from_messages
                .in_set(GraphSystemSet::Consumption)
                .after(GraphSystemSet::GraphSync),
        );
    }
}

/// System that consumes `CircuitGraphMessage` events and updates edge transforms
fn update_edge_positions_from_messages(
    mut commands: Commands,
    mut reader: MessageReader<CircuitGraphMessage>,
    graph_query: CircuitGraphQuery,
    nodes: Query<&Transform, With<CircuitNode>>,
) {
    // Collect all updates first to batch them
    let mut start_updates: Vec<(Entity, EdgeStartTransform)> = Vec::new();
    let mut end_updates: Vec<(Entity, EdgeEndTransform)> = Vec::new();

    for message in reader.read() {
        match message {
            CircuitGraphMessage::NodeChanged {
                entity,
                affected_neighbors: _,
                changes,
            } => {
                // Only update if transform actually changed
                if !changes.transform_changed && !changes.global_transform_changed {
                    continue;
                }

                // Get the node's transform
                let Ok(node_transform) = nodes.get(*entity) else {
                    continue;
                };

                // Use relationship queries for efficient edge lookup
                // Update outgoing edges (this node is the "from" node)
                for edge_entity in graph_query.outgoing_edges(*entity) {
                    start_updates.push((edge_entity, EdgeStartTransform(*node_transform)));
                }

                // Update incoming edges (this node is the "to" node)
                for edge_entity in graph_query.incoming_edges(*entity) {
                    end_updates.push((edge_entity, EdgeEndTransform(*node_transform)));
                }
            }

            CircuitGraphMessage::NodeAdded {
                entity,
                initial_neighbors: _,
            } => {
                // Initialize transforms for all edges connected to this new node
                let Ok(node_transform) = nodes.get(*entity) else {
                    continue;
                };

                // Update outgoing edges
                for edge_entity in graph_query.outgoing_edges(*entity) {
                    start_updates.push((edge_entity, EdgeStartTransform(*node_transform)));
                }

                // Update incoming edges
                for edge_entity in graph_query.incoming_edges(*entity) {
                    end_updates.push((edge_entity, EdgeEndTransform(*node_transform)));
                }
            }

            CircuitGraphMessage::EdgeAdded { entity, from, to } => {
                // Initialize transforms for the new edge from both endpoints
                if let Ok(from_transform) = nodes.get(*from) {
                    start_updates.push((*entity, EdgeStartTransform(*from_transform)));
                }

                if let Ok(to_transform) = nodes.get(*to) {
                    end_updates.push((*entity, EdgeEndTransform(*to_transform)));
                }

                // If we couldn't get transforms, the nodes might not exist yet
                // This is okay - the edge will be updated when the nodes are added/changed
            }

            CircuitGraphMessage::NodeConnected {
                node,
                edge,
                neighbor: _,
            } => {
                // When topology changes, update the edge's transforms
                // This handles cases where EdgeFrom/EdgeTo are modified
                let edge_from_opt = graph_query.edges_from.get(*edge).ok();
                let edge_to_opt = graph_query.edges_to.get(*edge).ok();

                if let (Some(edge_from), Some(edge_to)) = (edge_from_opt, edge_to_opt) {
                    // Update start transform if this node is the "from" node
                    if edge_from.0 == *node {
                        if let Ok(node_transform) = nodes.get(*node) {
                            start_updates.push((*edge, EdgeStartTransform(*node_transform)));
                        }
                    }

                    // Update end transform if this node is the "to" node
                    if edge_to.0 == *node {
                        if let Ok(node_transform) = nodes.get(*node) {
                            end_updates.push((*edge, EdgeEndTransform(*node_transform)));
                        }
                    }
                }
            }

            CircuitGraphMessage::NodeDisconnected {
                node,
                edge,
                neighbor: _,
            } => {
                // When a node is disconnected, update the edge's transforms based on current topology
                let edge_from_opt = graph_query.edges_from.get(*edge).ok();
                let edge_to_opt = graph_query.edges_to.get(*edge).ok();

                if let (Some(edge_from), Some(edge_to)) = (edge_from_opt, edge_to_opt) {
                    // Update start transform if this node is the "from" node
                    if edge_from.0 == *node {
                        if let Ok(node_transform) = nodes.get(*node) {
                            start_updates.push((*edge, EdgeStartTransform(*node_transform)));
                        }
                    }

                    // Update end transform if this node is the "to" node
                    if edge_to.0 == *node {
                        if let Ok(node_transform) = nodes.get(*node) {
                            end_updates.push((*edge, EdgeEndTransform(*node_transform)));
                        }
                    }
                }
            }

            // Other message types don't affect edge positions
            CircuitGraphMessage::NodeRemoved { .. }
            | CircuitGraphMessage::EdgeRemoved { .. }
            | CircuitGraphMessage::EdgeChanged { .. }
            | CircuitGraphMessage::PropagationTriggered { .. } => {}
        }
    }

    // Apply all updates in batch
    for (edge_entity, transform_component) in start_updates {
        commands.entity(edge_entity).insert(transform_component);
    }
    for (edge_entity, transform_component) in end_updates {
        commands.entity(edge_entity).insert(transform_component);
    }
}
