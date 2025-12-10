use std::marker::PhantomData;

use bevy::prelude::*;
use bevy::ecs::query::QueryFilter;

use super::components::{
    CircuitNode, EdgeFrom, EdgeTo, IncomingEdges, OutgoingEdges,
    EdgeStartTransform, EdgeEndTransform,
};

/// Plugin to automatically propagate a component value from nodes to all connected edges.
///
/// When a node with a [`Propagate<C>`] component changes, the component `C` is automatically
/// added to all edges connected to that node (via `OutgoingEdges` and `IncomingEdges`).
///
/// The plugin will maintain the target component over graph changes, adding or removing
/// `C` when edges are added to or removed from a node with a [`Propagate<C>`] component,
/// or if the [`Propagate<C>`] component is added, changed or removed.
///
/// Optionally you can include a query filter `F` to restrict the nodes that propagate.
/// Note that the filter is not rechecked dynamically: changes to the filter state will not be
/// picked up until the [`Propagate`] component is touched, or the graph structure changes.
///
/// Individual nodes can be skipped with the [`PropagateOver`] component.
///
/// The schedule can be configured via [`GraphPropagatePlugin::new`].
/// You should be sure to schedule your logic relative to this set: making changes
/// that modify component values before this logic, and reading the propagated
/// values after it.
pub struct GraphPropagatePlugin<
    C: Component + Clone + PartialEq,
    F: QueryFilter = (),
> {
    _marker: PhantomData<fn() -> (C, F)>,
}

impl<C: Component + Clone + PartialEq, F: QueryFilter> Default for GraphPropagatePlugin<C, F> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<C: Component + Clone + PartialEq, F: QueryFilter> GraphPropagatePlugin<C, F> {
    /// Construct the plugin. The propagation systems will be placed in the Update schedule.
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// Causes the inner component to be added to this node and all connected edges.
#[derive(Component, Clone, PartialEq)]
pub struct Propagate<C: Component + Clone + PartialEq>(pub C);

/// Stops the output component being added to this node's edges.
/// The node itself will still have the component if it has [`Propagate<C>`].
#[derive(Component)]
pub struct PropagateOver<C>(PhantomData<fn() -> C>);

/// The set in which propagation systems are added. You can schedule your logic relative to this set.
#[derive(SystemSet, Clone, PartialEq, PartialOrd, Ord)]
pub struct PropagateSet<C: Component + Clone + PartialEq> {
    _p: PhantomData<fn() -> C>,
}

/// Internal struct for managing propagation on edges
#[derive(Component, Clone, PartialEq)]
pub struct Inherited<C: Component + Clone + PartialEq>(pub C);

impl<C> Default for PropagateOver<C> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<C: Component + Clone + PartialEq> std::fmt::Debug for PropagateSet<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PropagateSet")
            .field("_p", &self._p)
            .finish()
    }
}

impl<C: Component + Clone + PartialEq> Eq for PropagateSet<C> {}

impl<C: Component + Clone + PartialEq> std::hash::Hash for PropagateSet<C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self._p.hash(state);
    }
}

impl<C: Component + Clone + PartialEq> Default for PropagateSet<C> {
    fn default() -> Self {
        Self {
            _p: Default::default(),
        }
    }
}

impl<C: Component + Clone + PartialEq, F: QueryFilter + 'static> Plugin
    for GraphPropagatePlugin<C, F>
{
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_source::<C, F>,
                propagate_to_edges::<C, F>,
                propagate_new_edges::<C, F>,
                propagate_output::<C, F>,
            )
                .chain()
                .in_set(PropagateSet::<C>::default()),
        );
    }
}

/// Add/remove `Inherited::<C>` and `C` for nodes with a direct `Propagate::<C>`
pub fn update_source<C: Component + Clone + PartialEq, F: QueryFilter>(
    mut commands: Commands,
    changed: Query<
        (Entity, &Propagate<C>),
        (
            Or<(Changed<Propagate<C>>, Without<Inherited<C>>)>,
            With<CircuitNode>,
            F,
        ),
    >,
    mut removed: RemovedComponents<Propagate<C>>,
) {
    for (entity, source) in &changed {
        commands
            .entity(entity)
            .try_insert(Inherited(source.0.clone()));
    }

    for removed in removed.read() {
        if let Ok(mut commands) = commands.get_entity(removed) {
            commands.remove::<(Inherited<C>, C)>();
        }
    }
}

/// Propagate `Inherited::<C>` from nodes to their connected edges
/// 
/// By default propagates to both outgoing and incoming edges.
/// For direction-aware propagation, use `propagate_to_edges_directional` instead.
pub fn propagate_to_edges<C: Component + Clone + PartialEq, F: QueryFilter>(
    mut commands: Commands,
    changed_nodes: Query<
        (Entity, &Inherited<C>, &OutgoingEdges, &IncomingEdges),
        (
            Changed<Inherited<C>>,
            With<CircuitNode>,
            Without<PropagateOver<C>>,
            F,
        ),
    >,
    all_nodes: Query<
        (Entity, Option<&Inherited<C>>, &OutgoingEdges, &IncomingEdges),
        (With<CircuitNode>, F),
    >,
    edges: Query<(Entity, &EdgeFrom, &EdgeTo), (With<EdgeFrom>, With<EdgeTo>)>,
    mut removed: RemovedComponents<Inherited<C>>,
) {
    // Collect all edges that need to be updated
    let mut edges_to_update: Vec<(Entity, Option<Inherited<C>>)> = Vec::new();

    // Handle changed nodes - propagate to their connected edges (both directions)
    for (_node_entity, inherited, outgoing, incoming) in &changed_nodes {
        // Add all outgoing edges
        for edge_entity in outgoing.iter() {
            edges_to_update.push((edge_entity, Some(inherited.clone())));
        }

        // Add all incoming edges
        for edge_entity in incoming.iter() {
            edges_to_update.push((edge_entity, Some(inherited.clone())));
        }
    }

    // Handle removed Inherited components - need to clean up edges
    for removed_node in removed.read() {
        // Find all edges connected to this node and mark them for cleanup
        // We need to check if the edge should still have Inherited from its other endpoint
        for (edge_entity, edge_from, edge_to) in edges.iter() {
            let from_node = edge_from.0;
            let to_node = edge_to.0;

            // If this edge was connected to the removed node, check if it should still have Inherited
            if from_node == removed_node || to_node == removed_node {
                // Check if the other endpoint still has Inherited
                let other_node = if from_node == removed_node { to_node } else { from_node };
                let should_keep = all_nodes
                    .get(other_node)
                    .ok()
                    .and_then(|(_, inherited, _, _)| inherited.cloned())
                    .is_some();

                if !should_keep {
                    edges_to_update.push((edge_entity, None));
                }
            }
        }
    }

    // For edges connected to multiple nodes, prefer the value from the "from" node
    // Deduplicate by keeping the first value for each edge
    let mut seen_edges = std::collections::HashMap::new();
    for (edge_entity, maybe_inherited) in edges_to_update {
        // If we've already seen this edge, prefer keeping the existing value (first one wins)
        // or if we're removing, prioritize removal
        match seen_edges.get(&edge_entity) {
            Some(None) => {
                // Already marked for removal, keep it that way
                continue;
            }
            Some(Some(_)) => {
                // Already has a value, keep the first one
                if maybe_inherited.is_some() {
                    continue;
                }
            }
            None => {}
        }
        seen_edges.insert(edge_entity, maybe_inherited);
    }

    // Apply updates to edges
    for (edge_entity, maybe_inherited) in seen_edges {
        if let Some(inherited) = maybe_inherited {
            commands.entity(edge_entity).try_insert(inherited);
        } else {
            commands.entity(edge_entity).remove::<(Inherited<C>, C)>();
        }
    }
}

/// Propagate `Inherited::<C>` to newly added edges from their connected nodes
pub fn propagate_new_edges<C: Component + Clone + PartialEq, F: QueryFilter>(
    mut commands: Commands,
    new_edges: Query<
        (Entity, &EdgeFrom, &EdgeTo),
        (With<EdgeFrom>, With<EdgeTo>, Without<Inherited<C>>),
    >,
    nodes: Query<
        (Entity, &Inherited<C>),
        (With<CircuitNode>, Without<PropagateOver<C>>, F),
    >,
) {
    for (edge_entity, edge_from, edge_to) in &new_edges {
        // Prefer the value from the "from" node, but fall back to "to" node
        let inherited = nodes
            .get(edge_from.0)
            .ok()
            .map(|(_, inherited)| inherited.clone())
            .or_else(|| {
                nodes
                    .get(edge_to.0)
                    .ok()
                    .map(|(_, inherited)| inherited.clone())
            });

        if let Some(inherited) = inherited {
            commands.entity(edge_entity).try_insert(inherited);
        }
    }
}

/// Add `C` to edges with `Inherited::<C>`
pub fn propagate_output<C: Component + Clone + PartialEq, F: QueryFilter>(
    mut commands: Commands,
    changed: Query<
        (Entity, &Inherited<C>, Option<&C>),
        (Changed<Inherited<C>>, With<EdgeFrom>, With<EdgeTo>),
    >,
) {
    for (entity, inherited, maybe_current) in &changed {
        if maybe_current.is_some_and(|c| &inherited.0 == c) {
            continue;
        }

        commands.entity(entity).try_insert(inherited.0.clone());
    }
}

// ============================================================================
// TRANSFORM PROPAGATION
// ============================================================================

/// Plugin to automatically propagate Transform from nodes to edges.
///
/// This uses the same patterns as `GraphPropagatePlugin` but with direction-aware
/// propagation:
/// - Transform from `EdgeFrom` node → `EdgeStartTransform` on outgoing edges
/// - Transform from `EdgeTo` node → `EdgeEndTransform` on incoming edges
///
/// This is optimized for performance with batching and minimal queries.
pub struct EdgeTransformPropagatePlugin;

impl Plugin for EdgeTransformPropagatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                propagate_node_transforms_to_edges,
                propagate_transforms_to_new_edges,
            )
                .chain()
                .in_set(EdgeTransformPropagateSet),
        );
    }
}

/// System set for edge transform propagation
#[derive(SystemSet, Clone, PartialEq, PartialOrd, Ord, Debug, Hash, Eq)]
pub struct EdgeTransformPropagateSet;

/// Propagate Transform from nodes to connected edges (direction-aware)
/// 
/// Optimized version that:
/// - Removes redundant edge queries (relationship already tells us which edges connect)
/// - Batches updates for better performance
/// - Only processes nodes with Changed<Transform> to avoid unnecessary work
fn propagate_node_transforms_to_edges(
    mut commands: Commands,
    changed_nodes: Query<
        (Entity, &Transform, &OutgoingEdges, &IncomingEdges),
        (Changed<Transform>, With<CircuitNode>),
    >,
) {
    // Collect all updates first, then apply in batch
    // Use separate vectors since EdgeStartTransform and EdgeEndTransform are different types
    let mut start_updates: Vec<(Entity, EdgeStartTransform)> = Vec::new();
    let mut end_updates: Vec<(Entity, EdgeEndTransform)> = Vec::new();
    
    for (_node_entity, transform, outgoing, incoming) in &changed_nodes {
        // OutgoingEdges already contains edges where this node is the "from" node
        // No need to query and verify - the relationship system guarantees this
        for edge_entity in outgoing.iter() {
            start_updates.push((edge_entity, EdgeStartTransform(*transform)));
        }

        // IncomingEdges already contains edges where this node is the "to" node
        for edge_entity in incoming.iter() {
            end_updates.push((edge_entity, EdgeEndTransform(*transform)));
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

/// Propagate transforms to newly added edges
fn propagate_transforms_to_new_edges(
    mut commands: Commands,
    new_edges: Query<
        (Entity, &EdgeFrom, &EdgeTo),
        (
            With<EdgeFrom>,
            With<EdgeTo>,
            Or<(Without<EdgeStartTransform>, Without<EdgeEndTransform>)>,
        ),
    >,
    nodes: Query<&Transform, With<CircuitNode>>,
) {
    for (edge_entity, edge_from, edge_to) in &new_edges {
        let mut cmd = commands.entity(edge_entity);
        
        // Get transform from the "from" node
        if let Ok(from_transform) = nodes.get(edge_from.0) {
            cmd.insert(EdgeStartTransform(*from_transform));
        }

        // Get transform from the "to" node
        if let Ok(to_transform) = nodes.get(edge_to.0) {
            cmd.insert(EdgeEndTransform(*to_transform));
        }
    }
}

