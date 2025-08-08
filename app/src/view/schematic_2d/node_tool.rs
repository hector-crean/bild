use bevy::{color::palettes, prelude::*};
use picking::double_click::DoubleClick;

use super::Schematic2DToolState;

/// Unique identifier for nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

/// Unique identifier for connection points
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionPointId(pub u64);

/// Types of connection points
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionType {
    Input,
    Output,
    Bidirectional,
}

/// Types of nodes in the schematic
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    Basic,
    Process,
    Decision,
    Terminal,
}

/// Connection point on a node
#[derive(Debug, Clone)]
pub struct ConnectionPoint {
    pub id: ConnectionPointId,
    pub local_position: Vec2, // Relative to node center
    pub point_type: ConnectionType,
    pub label: Option<String>,
}

/// Core schematic node component - view-independent data
#[derive(Component, Debug, Clone)]
pub struct SchematicNode {
    pub id: NodeId,
    pub node_type: NodeType,
    pub label: String,
    pub size: Vec2,
    pub connection_points: Vec<ConnectionPoint>,
}

impl SchematicNode {
    pub fn new(id: NodeId, node_type: NodeType, label: String) -> Self {
        let size = match node_type {
            NodeType::Basic => Vec2::new(120.0, 60.0),
            NodeType::Process => Vec2::new(140.0, 80.0),
            NodeType::Decision => Vec2::new(100.0, 100.0),
            NodeType::Terminal => Vec2::new(100.0, 40.0),
        };

        let connection_points = Self::default_connection_points(&node_type);

        Self {
            id,
            node_type,
            label,
            size,
            connection_points,
        }
    }

    fn default_connection_points(node_type: &NodeType) -> Vec<ConnectionPoint> {
        // We'll use a simple counter for now, in a real implementation this should be
        // managed by a proper resource or system
        static CONNECTION_ID_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        
        let next_id = || {
            ConnectionPointId(CONNECTION_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
        };

        match node_type {
            NodeType::Basic | NodeType::Process => vec![
                ConnectionPoint {
                    id: next_id(),
                    local_position: Vec2::new(-60.0, 0.0), // Left side
                    point_type: ConnectionType::Input,
                    label: Some("In".to_string()),
                },
                ConnectionPoint {
                    id: next_id(),
                    local_position: Vec2::new(60.0, 0.0), // Right side
                    point_type: ConnectionType::Output,
                    label: Some("Out".to_string()),
                },
            ],
            NodeType::Decision => vec![
                ConnectionPoint {
                    id: next_id(),
                    local_position: Vec2::new(0.0, 50.0), // Top
                    point_type: ConnectionType::Input,
                    label: Some("In".to_string()),
                },
                ConnectionPoint {
                    id: next_id(),
                    local_position: Vec2::new(-50.0, -50.0), // Bottom left
                    point_type: ConnectionType::Output,
                    label: Some("No".to_string()),
                },
                ConnectionPoint {
                    id: next_id(),
                    local_position: Vec2::new(50.0, -50.0), // Bottom right
                    point_type: ConnectionType::Output,
                    label: Some("Yes".to_string()),
                },
            ],
            NodeType::Terminal => vec![
                ConnectionPoint {
                    id: next_id(),
                    local_position: Vec2::new(0.0, 0.0), // Center
                    point_type: ConnectionType::Bidirectional,
                    label: None,
                },
            ],
        }
    }
}

/// States for the node tool within the 2D schematic view
#[derive(Default, SubStates, Debug, Clone, Eq, PartialEq, Hash)]
#[source(Schematic2DToolState = Schematic2DToolState::Node)]
pub enum NodeToolState {
    #[default]
    AwaitingPlacement,    // Waiting for click to place node
    Previewing,           // Showing node preview at cursor
    Editing,              // Editing existing node properties
    ChooseNodeType,       // Radial menu for node type selection
}

/// Component to mark a node as being previewed (not yet placed)
#[derive(Component)]
pub struct PreviewNode;

/// Resource to track the next node ID
#[derive(Resource)]
pub struct NodeIdCounter(pub u64);

impl Default for NodeIdCounter {
    fn default() -> Self {
        Self(1)
    }
}

/// Marker component for nodes that belong to the 2D schematic view
#[derive(Component)]
pub struct Schematic2DNode;

/// Node tool plugin - specific to 2D schematic view
pub struct NodeToolPlugin;

impl Plugin for NodeToolPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<NodeToolState>()
            .init_resource::<NodeIdCounter>()
            .add_systems(Startup, Self::setup)
            .add_systems(
                Update,
                (
                    Self::place_node_on_click
                        .run_if(in_state(NodeToolState::AwaitingPlacement))
                        .run_if(on_event::<Pointer<Click>>)
                        .run_if(not(on_event::<Pointer<DoubleClick>>)),
                    Self::debug_state_transition
                        .run_if(on_event::<StateTransitionEvent<NodeToolState>>),
                )
                    .run_if(in_state(Schematic2DToolState::Node)),
            )
            .add_systems(OnEnter(Schematic2DToolState::Node), Self::enter_node_tool)
            .add_systems(OnExit(Schematic2DToolState::Node), Self::exit_node_tool);
    }
}

impl NodeToolPlugin {
    fn setup() {
        info!("NodeToolPlugin: setup complete for 2D Schematic view");
    }

    fn debug_state_transition(
        mut state_reader: EventReader<StateTransitionEvent<NodeToolState>>,
    ) {
        for event in state_reader.read() {
            info!(
                "NodeToolState changed from {:?} to {:?}",
                event.exited, event.entered
            );
        }
    }

    fn enter_node_tool(
        mut next_state: ResMut<NextState<NodeToolState>>,
    ) {
        info!("Entering node tool in 2D Schematic view");
        next_state.set(NodeToolState::AwaitingPlacement);
    }

    fn exit_node_tool(
        mut commands: Commands,
        preview_nodes: Query<Entity, With<PreviewNode>>,
    ) {
        info!("Exiting node tool in 2D Schematic view");
        // Clean up any preview nodes
        for entity in &preview_nodes {
            commands.entity(entity).despawn();
        }
    }

    fn place_node_on_click(
        mut commands: Commands,
        mut pointer_click_events: EventReader<Pointer<Click>>,
        mut node_counter: ResMut<NodeIdCounter>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
        for click in pointer_click_events.read() {
            if let Some(position) = click.hit.position {
                let node_id = NodeId(node_counter.0);
                node_counter.0 += 1;

                let node = SchematicNode::new(
                    node_id,
                    NodeType::Basic,
                    format!("Node {}", node_id.0),
                );

                info!("Placing node at position: {:?}", position);

                // Create visual representation for 2D view
                let mesh = meshes.add(Rectangle::new(node.size.x, node.size.y));
                let material = materials.add(ColorMaterial::from(Color::from(palettes::tailwind::BLUE_400)));

                // Spawn the node entity with 2D-specific components
                commands.spawn((
                    node,
                    Schematic2DNode, // Marker for 2D schematic view
                    Mesh2d(mesh),
                    MeshMaterial2d(material),
                    Transform::from_translation(Vec3::new(position.x, position.y, 0.0)),
                ));
            }
        }
    }
} 