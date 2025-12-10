

use bevy::prelude::*;
use std::{fmt::Debug, marker::PhantomData, ops::{Deref, DerefMut}};

use petgraph::{graph::{NodeIndex, EdgeIndex}, Directed};
use petgraph::visit::EdgeRef;
use bevy::tasks::{AsyncComputeTaskPool, Task};

use crate::circuit::graph::{
    CircuitEdge, CircuitEdgeQueryData, CircuitNode, CircuitNodeQueryData, EdgeFrom, EdgeTo,
};


use crate::compute::{BackendJob, ComputeMsg, ComputeCmd};

use thiserror::Error;










#[derive(Debug, Error)]
pub enum GraphSolverError {
    #[error("No valid states available for node {0:?}")]
    NoValidStates(NodeIndex),
    
    #[error("Propagation failed: {0:?}")]
    PropagationFailed(String),
    
    #[error("Incomplete collapse for node {0:?}")]
    IncompleteCollapse(NodeIndex),
    
    #[error("Invalid state: {0:?}")]
    InvalidState(String),

    #[error("No valid states after applying invariants for node {0:?}")]
    NoValidStatesAfterInvariants(NodeIndex),

    #[error("Heuristic failed to select state for node {0:?}")]
    HeuristicFailure(NodeIndex),

    #[error("Multiple states remain for uncollapsed node {0:?}: expected 1, found {1}")]
    MultipleStatesRemain(NodeIndex, usize),

    #[error("No solution found")]
    NoSolution,

    #[error("Node not found in graph")]
    NodeNotFound(NodeIndex),

    #[error("Node not found at position {0:?}")]
    NodeNotFoundAtPosition((usize, usize, usize)),
}

#[derive(Clone)]
pub struct NodeState<N: Debug> {
    _phantom: PhantomData<N>
}

impl<N: Debug> NodeState<N> {
    pub fn new() -> Self { Self { _phantom: PhantomData } }
}

#[derive(Clone)]
pub struct EdgeState<E: Debug> {
    _phantom: PhantomData<E>
}

impl<E: Debug> EdgeState<E> {
    pub fn new() -> Self { Self { _phantom: PhantomData } }
}






#[derive(Clone)]
pub struct Graph<N: Debug, E: Debug>(pub petgraph::Graph<NodeState<N>, EdgeState<E>, Directed>);

impl<N, E> Default for Graph<N,E> where N: Debug, E: Debug {
    fn default() -> Self {
        Self::new()
    }
}

impl<N, E> Deref for Graph<N,E> where N: Debug, E: Debug {
    type Target = petgraph::Graph<NodeState<N>, EdgeState<E>, Directed>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<N, E> DerefMut for Graph<N,E> where N: Debug, E: Debug {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<N, E> Graph<N,E> where N: Debug, E: Debug {
    pub fn inner(&self) -> &petgraph::Graph<NodeState<N>, EdgeState<E>, Directed> {
        &self.0
    } 

    pub fn new() -> Self { Self(petgraph::Graph::new()) }
}



#[derive(Resource)]
pub struct GraphSolver<N: Debug + Send + Sync + 'static, E: Debug + Send + Sync + 'static> {
    pub graph: Graph<N,E>,
    pub backend: SolveBackend,
    pub job: Option<Task<Result<SolveResult, GraphSolverError>>>,
    pub last_result: Option<Result<SolveResult, GraphSolverError>>,
}

impl<N, E> Default for GraphSolver<N,E> where N: Debug + Send + Sync + 'static, E: Debug + Send + Sync + 'static {
    fn default() -> Self {
        Self {
            graph: Graph::new(),
            backend: SolveBackend::CpuAsync,
            job: None,
            last_result: None,
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
where N: Debug + Send + Sync + 'static, E: Debug + Send + Sync + 'static 
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

impl<N, E> GraphSolver<N,E> where N: Debug + Send + Sync + 'static, E: Debug + Send + Sync + 'static {
   pub fn solve(&mut self) -> Result<(), GraphSolverError> {
    // TODO: Implement solver
    Ok(())
   }

    pub fn set_backend(&mut self, backend: SolveBackend) { self.backend = backend; }

    pub fn snapshot(&self) -> GraphSnapshot {
        GraphSnapshot {
            node_count: self.graph.node_count(),
            edge_count: self.graph.edge_count(),
        }
    }

    pub fn start(&mut self) -> bool {
        match self.backend {
            SolveBackend::CpuSync => {
                let snapshot = self.snapshot();
                self.last_result = Some(Self::compute(snapshot));
                true
            }
            SolveBackend::CpuAsync => {
                if self.job.is_some() { return false; }
                let snapshot = self.snapshot();
                let task = AsyncComputeTaskPool::get().spawn(async move {
                    Self::compute(snapshot)
                });
                self.job = Some(task);
                true
            }
        }
    }

    pub fn poll(&mut self) -> Option<Result<SolveResult, GraphSolverError>> {
        if let Some(done) = self.last_result.take() { return Some(done); }
        if let Some(task) = self.job.as_mut() {
            if let Some(done) = bevy::tasks::futures_lite::future::block_on(
                bevy::tasks::futures_lite::future::poll_once(task)
            ) {
                self.job = None;
                return Some(done);
            }
        }
        None
    }

    fn compute(_snapshot: GraphSnapshot) -> Result<SolveResult, GraphSolverError> {
        Ok(SolveResult { changed: false })
    }

    // --- GENERIC SYNC API ---

    /// Call this from your `on_node_added` system
    pub fn register_node(&mut self, index: &mut GraphEntityIndex<N, E>, entity: Entity, state: NodeState<N>) -> Option<NodeIndex> {
        if index.node_of_entity.contains_key(&entity) { return None; }
        
        let ni = self.graph.add_node(state);
        index.node_of_entity.insert(entity, ni);
        index.entity_of_node.insert(ni, entity);
        Some(ni)
    }

    /// Call this from your `on_node_removed` system
    pub fn unregister_node(&mut self, index: &mut GraphEntityIndex<N, E>, entity: Entity) {
        use petgraph::Direction;
        if let Some(ni) = index.node_of_entity.remove(&entity) {
            index.entity_of_node.remove(&ni);
            
            // Clean up edge mappings for edges that are about to be destroyed by remove_node
            for edge_ref in self.graph.edges_directed(ni, Direction::Outgoing).collect::<Vec<_>>() {
                let eid = edge_ref.id();
                if let Some(ent) = index.entity_of_edge.remove(&eid) {
                    index.edge_of_entity.remove(&ent);
                }
            }
            for edge_ref in self.graph.edges_directed(ni, Direction::Incoming).collect::<Vec<_>>() {
                let eid = edge_ref.id();
                if let Some(ent) = index.entity_of_edge.remove(&eid) {
                    index.edge_of_entity.remove(&ent);
                }
            }
            let _ = self.graph.remove_node(ni);
        }
    }

    /// Call this from your `on_edge_added` or `changed` system
    pub fn sync_edge(
        &mut self, 
        index: &mut GraphEntityIndex<N, E>, 
        edge_entity: Entity, 
        from_entity: Entity, 
        to_entity: Entity,
        state: EdgeState<E>
    ) {
        // 1. Remove old mapping if exists (handle updates)
        if let Some(old_ei) = index.edge_of_entity.remove(&edge_entity) {
            index.entity_of_edge.remove(&old_ei);
            let _ = self.graph.remove_edge(old_ei);
        }

        // 2. Validate endpoints exist in graph
        let Some(a) = index.node_of_entity.get(&from_entity).copied() else { return; };
        let Some(b) = index.node_of_entity.get(&to_entity).copied() else { return; };

        // 3. Add new edge
        let ei = self.graph.add_edge(a, b, state);
        index.edge_of_entity.insert(edge_entity, ei);
        index.entity_of_edge.insert(ei, edge_entity);
    }

    /// Call this from `on_edge_removed`
    pub fn unregister_edge(&mut self, index: &mut GraphEntityIndex<N, E>, edge_entity: Entity) {
        if let Some(ei) = index.edge_of_entity.remove(&edge_entity) {
            index.entity_of_edge.remove(&ei);
            let _ = self.graph.remove_edge(ei);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SolveBackend {
    CpuSync,
    CpuAsync,
}

impl Default for SolveBackend { fn default() -> Self { SolveBackend::CpuAsync } }

#[derive(Clone, Debug, Default)]
pub struct GraphSnapshot {
    pub node_count: usize,
    pub edge_count: usize,
}

#[derive(Clone, Debug, Default)]
pub struct SolveResult {
    pub changed: bool,
}








pub struct CircuitSolverPlugin;

impl Plugin for CircuitSolverPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphSolver<CircuitNode, CircuitEdge>>()
            .init_resource::<GraphEntityIndex<CircuitNode, CircuitEdge>>()
            .add_systems(Startup, initial_sync)
            .add_systems(
                Update,
                (
                    sync_nodes_added,
                    sync_nodes_removed,
                    sync_edges_added,
                    sync_edges_removed,
                    sync_edges_endpoints_changed,
                ),
            );
    }
}

fn initial_sync(
    mut solver: ResMut<GraphSolver<CircuitNode, CircuitEdge>>,
    mut index: ResMut<GraphEntityIndex<CircuitNode, CircuitEdge>>,
    nodes: Query<CircuitNodeQueryData>,
    edges: Query<CircuitEdgeQueryData>,
) {
    // 1. Reset generic solver
    solver.graph = crate::circuit::solver::Graph::default(); // Assumes we can assign or reset
    *index = GraphEntityIndex::default();

    // 2. Add all existing nodes
    for n in nodes.iter() {
        solver.register_node(&mut index, n.entity, NodeState::new());
    }

    // 3. Add all existing edges
    for e in edges.iter() {
        solver.sync_edge(
            &mut index,
            e.entity,
            e.edge_from.0,
            e.edge_to.0,
            EdgeState::new(),
        );
    }
}

fn sync_nodes_added(
    mut solver: ResMut<GraphSolver<CircuitNode, CircuitEdge>>,
    mut index: ResMut<GraphEntityIndex<CircuitNode, CircuitEdge>>,
    query: Query<CircuitNodeQueryData, Added<CircuitNode>>,
) {
    for item in query.iter() {
        solver.register_node(&mut index, item.entity, NodeState::new());
    }
}

fn sync_nodes_removed(
    mut solver: ResMut<GraphSolver<CircuitNode, CircuitEdge>>,
    mut index: ResMut<GraphEntityIndex<CircuitNode, CircuitEdge>>,
    mut removed: RemovedComponents<CircuitNode>,
) {
    for entity in removed.read() {
        solver.unregister_node(&mut index, entity);
    }
}

fn sync_edges_added(
    mut solver: ResMut<GraphSolver<CircuitNode, CircuitEdge>>,
    mut index: ResMut<GraphEntityIndex<CircuitNode, CircuitEdge>>,
    query: Query<CircuitEdgeQueryData, Added<CircuitEdge>>,
) {
    for item in query.iter() {
        solver.sync_edge(
            &mut index,
            item.entity,
            item.edge_from.0,
            item.edge_to.0,
            EdgeState::new(),
        );
    }
}

fn sync_edges_endpoints_changed(
    mut solver: ResMut<GraphSolver<CircuitNode, CircuitEdge>>,
    mut index: ResMut<GraphEntityIndex<CircuitNode, CircuitEdge>>,
    query: Query<CircuitEdgeQueryData, Or<(Changed<EdgeFrom>, Changed<EdgeTo>)>>,
) {
    for item in query.iter() {
        solver.sync_edge(
            &mut index,
            item.entity,
            item.edge_from.0,
            item.edge_to.0,
            EdgeState::new(),
        );
    }
}

fn sync_edges_removed(
    mut solver: ResMut<GraphSolver<CircuitNode, CircuitEdge>>,
    mut index: ResMut<GraphEntityIndex<CircuitNode, CircuitEdge>>,
    mut removed: RemovedComponents<CircuitEdge>,
) {
    for entity in removed.read() {
        solver.unregister_edge(&mut index, entity);
    }
}



