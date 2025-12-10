use bevy::prelude::*;
use super::components::{EdgeFrom, EdgeTo};
use super::queries::CircuitGraphQuery;

// ============================================================================
// Spawning convenience
// ============================================================================

/// Spawn an edge entity related to `from` (as `EdgeFrom`) and targeting `to` (as `EdgeTo`).
pub fn spawn_edge(commands: &mut Commands, from: Entity, to: Entity) {
    commands.entity(from).with_related_entities::<EdgeFrom>(|rel| {
        rel.spawn(EdgeTo(to));
    });
}

/// Remove the edge entity `from -> to` if it exists.
pub fn remove_edge(commands: &mut Commands, graph: &CircuitGraphQuery, from: Entity, to: Entity) {
	if let Some(edge) = graph.find_edge(from, to) {
		commands.entity(edge).despawn();
	}
}

// ============================================================================
// Commands extensions
// ============================================================================

pub trait CircuitGraphCommandsExt {
	fn spawn_edge(&mut self, from: Entity, to: Entity);
	fn spawn_edges<I: IntoIterator<Item = (Entity, Entity)>>(&mut self, pairs: I);
	fn remove_edge(&mut self, graph: &CircuitGraphQuery, from: Entity, to: Entity);
	fn remove_all_outgoing(&mut self, graph: &CircuitGraphQuery, node: Entity);
	fn remove_all_incoming(&mut self, graph: &CircuitGraphQuery, node: Entity);
}

impl<'w, 's> CircuitGraphCommandsExt for Commands<'w, 's> {
	fn spawn_edge(&mut self, from: Entity, to: Entity) { spawn_edge(self, from, to) }

	fn spawn_edges<I: IntoIterator<Item = (Entity, Entity)>>(&mut self, pairs: I) {
		for (from, to) in pairs { spawn_edge(self, from, to); }
	}

	fn remove_edge(&mut self, graph: &CircuitGraphQuery, from: Entity, to: Entity) { remove_edge(self, graph, from, to) }

	fn remove_all_outgoing(&mut self, graph: &CircuitGraphQuery, node: Entity) {
		for edge in graph.outgoing_edges(node) { self.entity(edge).despawn(); }
	}

	fn remove_all_incoming(&mut self, graph: &CircuitGraphQuery, node: Entity) {
		for edge in graph.incoming_edges(node) { self.entity(edge).despawn(); }
	}
}

pub trait CircuitGraphEntityCommandsExt<'a> {
	fn connect_to(&mut self, to: Entity) -> &mut Self;
	fn disconnect_from(&mut self, graph: &CircuitGraphQuery, to: Entity) -> &mut Self;
	fn clear_outgoing(&mut self, graph: &CircuitGraphQuery) -> &mut Self;
	fn clear_incoming(&mut self, graph: &CircuitGraphQuery) -> &mut Self;
}

impl<'a> CircuitGraphEntityCommandsExt<'a> for bevy::ecs::system::EntityCommands<'a> {
	fn connect_to(&mut self, to: Entity) -> &mut Self {
        self.with_related_entities::<EdgeFrom>(|rel| { rel.spawn(EdgeTo(to)); });
		self
	}

	fn disconnect_from(&mut self, graph: &CircuitGraphQuery, to: Entity) -> &mut Self {
		let from = self.id();
		if let Some(edge) = graph.find_edge(from, to) {
			self.commands().entity(edge).despawn();
		}
		self
	}

	fn clear_outgoing(&mut self, graph: &CircuitGraphQuery) -> &mut Self {
		let from = self.id();
		for edge in graph.outgoing_edges(from) { self.commands().entity(edge).despawn(); }
		self
	}

	fn clear_incoming(&mut self, graph: &CircuitGraphQuery) -> &mut Self {
		let node = self.id();
		for edge in graph.incoming_edges(node) { self.commands().entity(edge).despawn(); }
		self
	}
}

