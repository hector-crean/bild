

use bevy::{ecs::system::SystemParam, prelude::*};
use std::collections::{HashMap, HashSet, VecDeque};

// ============================================================================
// GENERIC GRAPH RELATIONSHIP COMPONENTS
// ============================================================================

/// Edge component: indicates the source node (`from`).
#[derive(Component, Debug, Clone, Copy)]
#[relationship(relationship_target = OutgoingEdges)]
pub struct EdgeFrom(#[relationship] pub Entity);

/// Edge component: indicates the target node (`to`).
#[derive(Component, Debug, Clone, Copy)]
#[relationship(relationship_target = IncomingEdges)]
pub struct EdgeTo(#[relationship] pub Entity);

/// Reverse index: all edges that start at a node (outgoing).
#[derive(Component, Debug, Default)]
#[relationship_target(relationship = EdgeFrom)]
pub struct OutgoingEdges(Vec<Entity>);

impl OutgoingEdges {
	pub fn iter(&self) -> impl ExactSizeIterator<Item = Entity> + '_ { self.0.iter().copied() }
}

/// Reverse index: all edges that end at a node (incoming).
#[derive(Component, Debug, Default)]
#[relationship_target(relationship = EdgeTo)]
pub struct IncomingEdges(Vec<Entity>);

impl IncomingEdges {
	pub fn iter(&self) -> impl ExactSizeIterator<Item = Entity> + '_ { self.0.iter().copied() }
}

// ============================================================================
// CLEAN QUERY EXTENSION TRAITS
// ============================================================================

/// Query helpers for graph navigation
pub trait GraphQueryExt<'w, 's> {
	fn outgoing_edges_of(&self, node: Entity) -> Vec<Entity>;
	fn incoming_edges_of(&self, node: Entity) -> Vec<Entity>;
	fn neighbors_of(&self, node: Entity) -> Vec<Entity>;
}

impl<'w, 's> GraphQueryExt<'w, 's> for (
	Query<'w, 's, &'static OutgoingEdges>,
	Query<'w, 's, &'static EdgeTo>,
	Query<'w, 's, &'static IncomingEdges>,
) {
	fn outgoing_edges_of(&self, node: Entity) -> Vec<Entity> {
		let (outgoing, _to, _incoming) = self;
		outgoing.relationship_sources::<OutgoingEdges>(node).collect()
	}

	fn incoming_edges_of(&self, node: Entity) -> Vec<Entity> {
		let (_outgoing, _to, incoming) = self;
		incoming.relationship_sources::<IncomingEdges>(node).collect()
	}

	fn neighbors_of(&self, node: Entity) -> Vec<Entity> {
		let (outgoing, to, _incoming) = self;
		outgoing
			.relationship_sources::<OutgoingEdges>(node)
			.filter_map(|edge| to.get(edge).ok().map(|edge_to| edge_to.0))
			.collect()
	}
}



/// Graph system param and algorithms

#[derive(SystemParam)]
pub struct Graph<'w, 's> {
	pub edges_from: Query<'w, 's, &'static EdgeFrom>,
	pub edges_to: Query<'w, 's, &'static EdgeTo>,
	pub outgoing_index: Query<'w, 's, &'static OutgoingEdges>,
	pub incoming_index: Query<'w, 's, &'static IncomingEdges>,
}

impl<'w, 's> Graph<'w, 's> {
	/// Iterate outgoing edge entities from a node
	pub fn outgoing_edges(&self, node: Entity) -> impl Iterator<Item = Entity> + '_ {
		self.outgoing_index.relationship_sources::<OutgoingEdges>(node)
	}

	/// Iterate incoming edge entities to a node
	pub fn incoming_edges(&self, node: Entity) -> impl Iterator<Item = Entity> + '_ {
		self.incoming_index.relationship_sources::<IncomingEdges>(node)
	}

	/// Iterate neighbor node entities reachable via outgoing edges
	pub fn neighbors(&self, node: Entity) -> impl Iterator<Item = Entity> + '_ {
		self.outgoing_edges(node)
			.filter_map(|edge| self.edges_to.get(edge).ok().map(|edge_to| edge_to.0))
	}

	/// Iterate predecessor node entities (via incoming edges)
	pub fn predecessors(&self, node: Entity) -> impl Iterator<Item = Entity> + '_ {
		self.incoming_edges(node)
			.filter_map(|edge| self.edges_from.get(edge).ok().map(|edge_from| edge_from.0))
	}

	/// Out-degree (number of outgoing edges)
	pub fn out_degree(&self, node: Entity) -> usize { self.outgoing_edges(node).count() }

	/// In-degree (number of incoming edges)
	pub fn in_degree(&self, node: Entity) -> usize { self.incoming_edges(node).count() }

	/// Iterate (edge, neighbor) pairs for outgoing edges
	pub fn neighbors_with_edges(&self, node: Entity) -> impl Iterator<Item = (Entity, Entity)> + '_ {
		self.outgoing_edges(node)
			.filter_map(|edge| self.edges_to.get(edge).ok().map(|edge_to| (edge, edge_to.0)))
	}

	/// Return neighbors treating the graph as undirected (successors ∪ predecessors)
	pub fn undirected_neighbors(&self, node: Entity) -> Vec<Entity> {
		let mut set: HashSet<Entity> = HashSet::new();
		for n in self.neighbors(node) { set.insert(n); }
		for p in self.predecessors(node) { set.insert(p); }
		set.into_iter().collect()
	}

	/// Find the edge entity connecting `from -> to`, if present
	pub fn find_edge(&self, from: Entity, to: Entity) -> Option<Entity> {
		self.outgoing_edges(from)
			.find(|&edge| self.edges_to.get(edge).ok().map(|et| et.0) == Some(to))
	}

	/// Whether `to` is reachable from `start` (directed reachability)
	pub fn is_reachable(&self, start: Entity, to: Entity) -> bool {
		if start == to { return true; }
		self.bfs_path(start, to).is_some()
	}

	/// Breadth-first search from `start` to `goal`, returning the path of node entities if found.
	pub fn bfs_path(&self, start: Entity, goal: Entity) -> Option<Vec<Entity>> {
		if start == goal { return Some(vec![start]); }
		let mut queue = VecDeque::new();
		let mut visited: HashSet<Entity> = HashSet::new();
		let mut parent: HashMap<Entity, Entity> = HashMap::new();
		queue.push_back(start);
		visited.insert(start);
		while let Some(current) = queue.pop_front() {
			for neighbor in self.neighbors(current) {
				if !visited.contains(&neighbor) {
					visited.insert(neighbor);
					parent.insert(neighbor, current);
					if neighbor == goal {
						let mut path = vec![goal];
						let mut node = goal;
						while let Some(&p) = parent.get(&node) {
							path.push(p);
							if p == start { break; }
							node = p;
						}
						path.reverse();
						return Some(path);
					}
					queue.push_back(neighbor);
				}
			}
		}
		None
	}

	/// Collect the connected component containing `start` using BFS over outgoing edges.
	pub fn connected_component(&self, start: Entity) -> Vec<Entity> {
		let mut queue = VecDeque::new();
		let mut visited: HashSet<Entity> = HashSet::new();
		queue.push_back(start);
		visited.insert(start);
		while let Some(current) = queue.pop_front() {
			for neighbor in self.neighbors(current) {
				if visited.insert(neighbor) { queue.push_back(neighbor); }
			}
		}
		visited.into_iter().collect()
	}

	/// Iterator over nodes in BFS order starting at `start`.
    pub fn bfs_iter(&self, start: Entity) -> BfsIter<'_> {
		BfsIter::new(self, start)
	}

	/// Iterator over nodes in DFS (pre-order) starting at `start`.
    pub fn dfs_iter(&self, start: Entity) -> DfsIter<'_> {
		DfsIter::new(self, start)
	}
}

// ============================================================================
// Iterator types
// =========================================================================

pub struct BfsIter<'a> {
    graph: &'a Graph<'a, 'a>,
	queue: VecDeque<Entity>,
	visited: HashSet<Entity>,
}

impl<'a> BfsIter<'a> {
    fn new(graph: &'a Graph, start: Entity) -> Self {
		let mut queue = VecDeque::new();
		let mut visited = HashSet::new();
		queue.push_back(start);
		visited.insert(start);
		Self { graph, queue, visited }
	}
}

impl<'a> Iterator for BfsIter<'a> {
	type Item = Entity;
	fn next(&mut self) -> Option<Self::Item> {
		let current = self.queue.pop_front()?;
		for neighbor in self.graph.neighbors(current) {
			if self.visited.insert(neighbor) {
				self.queue.push_back(neighbor);
			}
		}
		Some(current)
	}
}

pub struct DfsIter<'a> {
    graph: &'a Graph<'a, 'a>,
	stack: Vec<Entity>,
	visited: HashSet<Entity>,
}

impl<'a> DfsIter<'a> {
    fn new(graph: &'a Graph, start: Entity) -> Self {
		Self { graph, stack: vec![start], visited: HashSet::new() }
	}
}

impl<'a> Iterator for DfsIter<'a> {
	type Item = Entity;
	fn next(&mut self) -> Option<Self::Item> {
		while let Some(current) = self.stack.pop() {
			if self.visited.insert(current) {
				for neighbor in self.graph.neighbors(current) {
					self.stack.push(neighbor);
				}
				return Some(current);
			}
		}
		None
	}
}

// ============================================================================
// Spawning convenience
// =========================================================================

/// Spawn an edge entity related to `from` (as `EdgeFrom`) and targeting `to` (as `EdgeTo`).
pub fn spawn_edge(commands: &mut Commands, from: Entity, to: Entity) {
	commands.entity(from).with_related_entities::<EdgeFrom>(|rel| {
		rel.spawn(EdgeTo(to));
	});
}

/// Remove the edge entity `from -> to` if it exists.
pub fn remove_edge(commands: &mut Commands, graph: &Graph, from: Entity, to: Entity) {
	if let Some(edge) = graph.find_edge(from, to) {
		commands.entity(edge).despawn();
	}
}

// ============================================================================
// Commands extensions
// =========================================================================

pub trait GraphCommandsExt {
	fn spawn_edge(&mut self, from: Entity, to: Entity);
	fn spawn_edges<I: IntoIterator<Item = (Entity, Entity)>>(&mut self, pairs: I);
	fn remove_edge(&mut self, graph: &Graph, from: Entity, to: Entity);
	fn remove_all_outgoing(&mut self, graph: &Graph, node: Entity);
	fn remove_all_incoming(&mut self, graph: &Graph, node: Entity);
}

impl<'w, 's> GraphCommandsExt for Commands<'w, 's> {
	fn spawn_edge(&mut self, from: Entity, to: Entity) { spawn_edge(self, from, to) }

	fn spawn_edges<I: IntoIterator<Item = (Entity, Entity)>>(&mut self, pairs: I) {
		for (from, to) in pairs { spawn_edge(self, from, to); }
	}

	fn remove_edge(&mut self, graph: &Graph, from: Entity, to: Entity) { remove_edge(self, graph, from, to) }

	fn remove_all_outgoing(&mut self, graph: &Graph, node: Entity) {
		for edge in graph.outgoing_edges(node) { self.entity(edge).despawn(); }
	}

	fn remove_all_incoming(&mut self, graph: &Graph, node: Entity) {
		for edge in graph.incoming_edges(node) { self.entity(edge).despawn(); }
	}
}

pub trait GraphEntityCommandsExt<'a> {
	fn connect_to(&mut self, to: Entity) -> &mut Self;
	fn disconnect_from(&mut self, graph: &Graph, to: Entity) -> &mut Self;
	fn clear_outgoing(&mut self, graph: &Graph) -> &mut Self;
	fn clear_incoming(&mut self, graph: &Graph) -> &mut Self;
}

impl<'a> GraphEntityCommandsExt<'a> for bevy::ecs::system::EntityCommands<'a> {
	fn connect_to(&mut self, to: Entity) -> &mut Self {
		self.with_related_entities::<EdgeFrom>(|rel| { rel.spawn(EdgeTo(to)); });
		self
	}

	fn disconnect_from(&mut self, graph: &Graph, to: Entity) -> &mut Self {
		let from = self.id();
		if let Some(edge) = graph.find_edge(from, to) {
			self.commands().entity(edge).despawn();
		}
		self
	}

	fn clear_outgoing(&mut self, graph: &Graph) -> &mut Self {
		let from = self.id();
		for edge in graph.outgoing_edges(from) { self.commands().entity(edge).despawn(); }
		self
	}

	fn clear_incoming(&mut self, graph: &Graph) -> &mut Self {
		let node = self.id();
		for edge in graph.incoming_edges(node) { self.commands().entity(edge).despawn(); }
		self
	}
}
