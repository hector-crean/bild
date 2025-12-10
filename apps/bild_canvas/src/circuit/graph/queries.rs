use bevy::{ecs::{query::QueryData, system::SystemParam}, prelude::*};
use std::collections::{HashMap, HashSet, VecDeque};

use crate::circuit::layer::types::LayerId;
use super::components::{
    CircuitEdge, CircuitNode, EdgeFrom, EdgeTo, IncomingEdges, OutgoingEdges, EdgeColor,
    EdgeStartTransform, EdgeEndTransform,
};

#[derive(QueryData)]
pub struct CircuitEdgeQueryData {
	pub entity: Entity,
	pub edge_from: &'static EdgeFrom,
	pub edge_to: &'static EdgeTo,
	pub edge_kind: &'static CircuitEdge,
	pub edge_color: Option<&'static EdgeColor>,
	pub start_transform: Option<&'static EdgeStartTransform>,
	pub end_transform: Option<&'static EdgeEndTransform>,
	pub polyline_mesh: Option<&'static Mesh3d>,
	pub polyline_material: Option<&'static MeshMaterial3d<StandardMaterial>>
}

#[derive(QueryData)]
pub struct CircuitNodeQueryData {
    pub entity: Entity,
    pub node_kind: &'static CircuitNode,
    pub incoming_edges: &'static IncomingEdges,
    pub outgoing_edges: &'static OutgoingEdges,
    pub transform: &'static Transform,
	pub global_transform: &'static GlobalTransform,
	pub layer_id: Option<&'static LayerId>,
}

/// Query data for detecting changes in node connectivity.
/// Useful for tracking before/after states when edges are added or removed.
#[derive(QueryData)]
pub struct CircuitNodeDeltaQueryData {
	pub entity: Entity,
	pub incoming_edges: &'static IncomingEdges,
	pub outgoing_edges: &'static OutgoingEdges,
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

// ============================================================================
// Graph system param and algorithms
// ============================================================================

#[derive(SystemParam)]
pub struct CircuitGraphQuery<'w, 's> {
	pub edges_from: Query<'w, 's, &'static EdgeFrom>,
	pub edges_to: Query<'w, 's, &'static EdgeTo>,
	pub outgoing_index: Query<'w, 's, &'static OutgoingEdges>,
	pub incoming_index: Query<'w, 's, &'static IncomingEdges>,
    /// Convenience edge query: iterate only entities that are valid edges (have both endpoints)
    pub edges_q: Query<'w, 's, (
        Entity,
        &'static EdgeFrom,
        &'static EdgeTo,
    ), With<CircuitEdge>>,
    /// Convenience node query: iterate entities that participate as a node (incoming and/or outgoing edges)
    pub nodes_q: Query<'w, 's, (
        Entity,
        Option<&'static OutgoingEdges>,
        Option<&'static IncomingEdges>,
    ), (With<CircuitNode>, Or<(With<OutgoingEdges>, With<IncomingEdges>)>)>,
}

impl<'w, 's> CircuitGraphQuery<'w, 's> {
    /// Iterate all edge entities that have both `EdgeFrom` and `EdgeTo`.
    pub fn edges_iter(
        &self,
    ) -> impl Iterator<Item = (Entity, EdgeFrom, EdgeTo)> + '_ {
        self.edges_q
            .iter()
            .map(|(e, from, to)| (e, *from, *to))
    }

    /// Iterate all node entities that have any connections (incoming and/or outgoing).
    pub fn nodes_iter(
        &self,
    ) -> impl Iterator<Item = (Entity, Option<&OutgoingEdges>, Option<&IncomingEdges>)> + '_ {
        self.nodes_q.iter()
    }

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

	/// Get all neighbors that should receive updates (undirected neighbors).
	/// This is useful for propagation systems that need to notify all connected nodes.
	pub fn affected_neighbors(&self, node: Entity) -> Vec<Entity> {
		self.undirected_neighbors(node)
	}

	/// Get nodes in a BFS-limited subgraph starting from `start` with optional depth limit.
	/// If `depth` is `None`, returns the full connected component.
	pub fn affected_subgraph(&self, start: Entity, depth: Option<usize>) -> Vec<Entity> {
		match depth {
			None => self.connected_component_undirected(start),
			Some(max_depth) => {
				let mut result = Vec::new();
				let mut queue = VecDeque::new();
				let mut visited = HashSet::new();
				queue.push_back((start, 0));
				visited.insert(start);

				while let Some((current, d)) = queue.pop_front() {
					if d > max_depth {
						continue;
					}
					result.push(current);
					if d < max_depth {
						for neighbor in self.undirected_neighbors(current) {
							if visited.insert(neighbor) {
								queue.push_back((neighbor, d + 1));
							}
						}
					}
				}
				result
			}
		}
	}

	/// Collect all nodes in the connected component containing `start` (undirected).
	/// This treats the graph as undirected for connectivity purposes.
	pub fn connected_component_undirected(&self, start: Entity) -> Vec<Entity> {
		let mut queue = VecDeque::new();
		let mut visited = HashSet::new();
		queue.push_back(start);
		visited.insert(start);
		
		while let Some(current) = queue.pop_front() {
			for neighbor in self.undirected_neighbors(current) {
				if visited.insert(neighbor) {
					queue.push_back(neighbor);
				}
			}
		}
		visited.into_iter().collect()
	}

	/// Collect all nodes that are upstream (predecessors) of `node`.
	/// Uses reverse BFS to find all nodes that can reach this node.
	pub fn upstream_nodes(&self, node: Entity) -> Vec<Entity> {
		let mut queue = VecDeque::new();
		let mut visited = HashSet::new();
		queue.push_back(node);
		visited.insert(node);
		
		while let Some(current) = queue.pop_front() {
			for predecessor in self.predecessors(current) {
				if visited.insert(predecessor) {
					queue.push_back(predecessor);
				}
			}
		}
		visited.into_iter().filter(|&n| n != node).collect()
	}

	/// Collect all nodes that are downstream (successors) of `node`.
	/// Uses forward BFS to find all nodes reachable from this node.
	pub fn downstream_nodes(&self, node: Entity) -> Vec<Entity> {
		let mut queue = VecDeque::new();
		let mut visited = HashSet::new();
		queue.push_back(node);
		visited.insert(node);
		
		while let Some(current) = queue.pop_front() {
			for neighbor in self.neighbors(current) {
				if visited.insert(neighbor) {
					queue.push_back(neighbor);
				}
			}
		}
		visited.into_iter().filter(|&n| n != node).collect()
	}

	/// Batch collect neighbors for multiple nodes efficiently.
	/// Returns a map from node entity to its neighbors.
	pub fn collect_neighbors_batch(&self, nodes: impl Iterator<Item = Entity>) -> HashMap<Entity, Vec<Entity>> {
		nodes.map(|node| (node, self.affected_neighbors(node))).collect()
	}

	/// Get all edges connected to a node (both incoming and outgoing).
	pub fn connected_edges(&self, node: Entity) -> Vec<Entity> {
		let mut edges = Vec::new();
		edges.extend(self.outgoing_edges(node));
		edges.extend(self.incoming_edges(node));
		edges
	}

	/// Check if two nodes are in the same connected component (undirected).
	pub fn same_component(&self, a: Entity, b: Entity) -> bool {
		if a == b {
			return true;
		}
		let component = self.connected_component_undirected(a);
		component.contains(&b)
	}
}

// ============================================================================
// Iterator types
// ============================================================================

pub struct BfsIter<'a> {
    graph: &'a CircuitGraphQuery<'a, 'a>,
	queue: VecDeque<Entity>,
	visited: HashSet<Entity>,
}

impl<'a> BfsIter<'a> {
    fn new(graph: &'a CircuitGraphQuery, start: Entity) -> Self {
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
    graph: &'a CircuitGraphQuery<'a, 'a>,
	stack: Vec<Entity>,
	visited: HashSet<Entity>,
}

impl<'a> DfsIter<'a> {
    fn new(graph: &'a CircuitGraphQuery, start: Entity) -> Self {
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

