use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bild_canvas::circuit::CircuitPlugin;
use bild_canvas::circuit::graph::{
    CircuitGraphCommandsExt, CircuitGraphManagerPlugin, CircuitGraphMessage, CircuitGraphQuery, CircuitNode, GraphEventPropagationConfig, PropagationType
};
use bild_canvas::circuit::layer::types::LayerId;
use camera::controller::CameraSettings;
use camera::controller::pan_orbit_camera::{OrbitCameraController, OrbitCameraControllerPlugin};
use camera::markers::MainCamera;
use geometry::representation::polyline::PolylinePlugin;
use interaction::InteractiveMeshPlugin;
use interaction::drag::three_d::Draggable3d;
use rand::Rng;

#[derive(Resource, Default, PartialEq, Eq)]
struct Camera3dSettingsImpl {
    locked: bool,
}

impl CameraSettings for Camera3dSettingsImpl {
    fn is_locked(&self) -> bool {
        self.locked
    }
    fn lock(&mut self) {
        self.locked = true;
    }
    fn unlock(&mut self) {
        self.locked = false;
    }
}

fn setup_camera_3d(mut commands: Commands) {
    let controller = OrbitCameraController::default();
    let transform = controller.generate_transform();

    // Camera setup without MeshPickingCamera which caused compilation issues
    commands.spawn((Camera3d::default(), MainCamera, controller, transform));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Camera controller plugin
        .add_plugins(OrbitCameraControllerPlugin::<Camera3dSettingsImpl>::default())
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .init_resource::<Camera3dSettingsImpl>()
        // Circuit visualization plugins
        .add_plugins(PolylinePlugin)
        .add_plugins((CircuitGraphManagerPlugin, CircuitPlugin))
        .add_plugins(MeshPickingPlugin)
        .add_plugins(InteractiveMeshPlugin::<Camera3dSettingsImpl>::default())
        // Configuration for how the graph looks
        .init_resource::<GraphEventStats>()
        .add_systems(Startup, (setup_camera_3d, spawn_circuit_graph))
        .add_systems(
            Update,
            (
                log_graph_events,
                visualize_propagation,
                handle_keyboard_input,
                fade_highlights,
            ),
        )
        .run();
}

// Configuration for stress testing
const STRESS_TEST: bool = true;
const NUM_NODES: usize = 50; // Adjust this to stress test different sizes
const EDGE_PROBABILITY: f64 = 0.02; // Probability of edge between any two nodes (for random graph)
const GRID_SIZE: usize = 50; // For grid graph: creates GRID_SIZE x GRID_SIZE nodes

#[derive(Clone, Copy)]
enum GraphType {
    Small,      // Original 6-node graph
    Random,     // Random graph with NUM_NODES nodes
    Grid,       // Regular grid graph
    ScaleFree,  // Scale-free network (preferential attachment)
}

fn spawn_circuit_graph(mut commands: Commands) {
    // Add a light source for better visibility of 3D objects
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 2000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let graph_type = if STRESS_TEST {
        GraphType::Grid // Change to Grid, ScaleFree, etc. to test different topologies
    } else {
        GraphType::Small
    };

    match graph_type {
        GraphType::Small => spawn_small_graph(&mut commands),
        GraphType::Random => spawn_random_graph(&mut commands, NUM_NODES, EDGE_PROBABILITY),
        GraphType::Grid => spawn_grid_graph(&mut commands, GRID_SIZE),
        GraphType::ScaleFree => spawn_scale_free_graph(&mut commands, NUM_NODES),
    }
}

fn spawn_small_graph(commands: &mut Commands) {
    // Original 6-node graph
    let node_a = commands
        .spawn((
            Transform::from_xyz(-1.0, 0.0, 0.0),
            GlobalTransform::default(),
            CircuitNode::default(),
            LayerId(0),
            Draggable3d::default(),
        ))
        .id();
    let node_b = commands
        .spawn((
            Transform::from_xyz(1.0, 0.0, 0.0),
            GlobalTransform::default(),
            CircuitNode::default(),
            LayerId(0),
            Draggable3d::default(),
        ))
        .id();
    let node_c = commands
        .spawn((
            Transform::from_xyz(0.0, 1.0, 0.0),
            GlobalTransform::default(),
            CircuitNode::default(),
            LayerId(0),
            Draggable3d::default(),
        ))
        .id();
    let node_d = commands
        .spawn((
            Transform::from_xyz(0.0, -1.0, 0.0),
            GlobalTransform::default(),
            CircuitNode::default(),
            LayerId(0),
            Draggable3d::default(),
        ))
        .id();
    let node_e = commands
        .spawn((
            Transform::from_xyz(0.0, 0.0, 1.0),
            GlobalTransform::default(),
            CircuitNode::default(),
            LayerId(0),
            Draggable3d::default(),
        ))
        .id();
    let node_f = commands
        .spawn((
            Transform::from_xyz(0.0, 0.0, -1.0),
            GlobalTransform::default(),
            CircuitNode::default(),
            LayerId(0),
            Draggable3d::default(),
        ))
        .id();

    commands.spawn_edge(node_a, node_b);
    commands.spawn_edge(node_a, node_c);
    commands.spawn_edge(node_a, node_d);
    commands.spawn_edge(node_b, node_c);
    commands.spawn_edge(node_b, node_d);
    commands.spawn_edge(node_c, node_d);
    commands.spawn_edge(node_c, node_e);
    commands.spawn_edge(node_c, node_f);
    commands.spawn_edge(node_d, node_e);
    commands.spawn_edge(node_d, node_f);
    commands.spawn_edge(node_e, node_f);
}

fn spawn_random_graph(commands: &mut Commands, num_nodes: usize, edge_probability: f64) {
    let mut rng = rand::thread_rng();
    let mut node_entities = Vec::with_capacity(num_nodes);

    // Spawn nodes in a roughly spherical distribution
    let radius = (num_nodes as f32).cbrt() * 2.0; // Scale radius with cube root of node count
    
    for i in 0..num_nodes {
        // Distribute nodes in 3D space
        let angle1 = (i as f32) * 2.0 * std::f32::consts::PI / (num_nodes as f32);
        let angle2 = (i as f32) * std::f32::consts::PI / (num_nodes as f32);
        let r = radius * (0.3 + 0.7 * (i as f32) / (num_nodes as f32));
        
        let x = r * angle1.cos() * angle2.sin();
        let y = r * angle2.cos();
        let z = r * angle1.sin() * angle2.sin();
        
        // Add some randomness to avoid perfect sphere
        let x = x + rng.gen_range(-0.5..0.5);
        let y = y + rng.gen_range(-0.5..0.5);
        let z = z + rng.gen_range(-0.5..0.5);

        let node = commands
            .spawn((
                Transform::from_xyz(x, y, z),
                GlobalTransform::default(),
                CircuitNode::default(),
                LayerId(0),
                Draggable3d::default(),
            ))
            .id();
        node_entities.push(node);
    }

    // Create edges with given probability
    let mut edge_count = 0;
    for i in 0..num_nodes {
        for j in (i + 1)..num_nodes {
            if rng.gen_bool(edge_probability) {
                commands.spawn_edge(node_entities[i], node_entities[j]);
                edge_count += 1;
            }
        }
    }
    
    println!("Spawned random graph: {} nodes, {} edges", num_nodes, edge_count);
}

fn spawn_grid_graph(commands: &mut Commands, grid_size: usize) {
    let mut node_entities = Vec::with_capacity(grid_size * grid_size);
    let spacing = 1.0;
    let offset = -(grid_size as f32) * spacing / 2.0;

    // Spawn nodes in a grid
    for y in 0..grid_size {
        for x in 0..grid_size {
            let node = commands
                .spawn((
                    Transform::from_xyz(
                        offset + x as f32 * spacing,
                        0.0,
                        offset + y as f32 * spacing,
                    ),
                    GlobalTransform::default(),
                    CircuitNode::default(),
                    LayerId(0),
                    Draggable3d::default(),
                ))
                .id();
            node_entities.push(node);
        }
    }

    // Connect nodes in a grid pattern (each node connects to right and bottom neighbors)
    let mut edge_count = 0;
    for y in 0..grid_size {
        for x in 0..grid_size {
            let idx = y * grid_size + x;
            
            // Connect to right neighbor
            if x < grid_size - 1 {
                commands.spawn_edge(node_entities[idx], node_entities[idx + 1]);
                edge_count += 1;
            }
            
            // Connect to bottom neighbor
            if y < grid_size - 1 {
                commands.spawn_edge(node_entities[idx], node_entities[idx + grid_size]);
                edge_count += 1;
            }
        }
    }
    
    println!("Spawned grid graph: {} nodes ({}x{}), {} edges", 
             grid_size * grid_size, grid_size, grid_size, edge_count);
}

fn spawn_scale_free_graph(commands: &mut Commands, num_nodes: usize) {
    let mut rng = rand::thread_rng();
    let mut node_entities = Vec::with_capacity(num_nodes);
    let mut degrees = Vec::with_capacity(num_nodes);
    
    let radius = (num_nodes as f32).cbrt() * 2.0;

    // Start with a small connected graph (3 nodes)
    for i in 0..3.min(num_nodes) {
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / 3.0;
        let node = commands
            .spawn((
                Transform::from_xyz(
                    angle.cos() * 2.0,
                    0.0,
                    angle.sin() * 2.0,
                ),
                GlobalTransform::default(),
                CircuitNode::default(),
                LayerId(0),
                Draggable3d::default(),
            ))
            .id();
        node_entities.push(node);
        degrees.push(0);
    }
    
    // Connect initial nodes
    if num_nodes >= 3 {
        commands.spawn_edge(node_entities[0], node_entities[1]);
        commands.spawn_edge(node_entities[1], node_entities[2]);
        commands.spawn_edge(node_entities[2], node_entities[0]);
        degrees[0] += 2;
        degrees[1] += 2;
        degrees[2] += 2;
    }

    // Add remaining nodes with preferential attachment
    for i in 3..num_nodes {
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / (num_nodes as f32);
        let r = radius * (0.3 + 0.7 * (i as f32) / (num_nodes as f32));
        let node = commands
            .spawn((
                Transform::from_xyz(
                    r * angle.cos(),
                    rng.gen_range(-1.0..1.0),
                    r * angle.sin(),
                ),
                GlobalTransform::default(),
                CircuitNode::default(),
                LayerId(0),
                Draggable3d::default(),
            ))
            .id();
        
        // Connect to existing nodes with probability proportional to their degree
        let total_degree: usize = degrees.iter().sum();
        let connections = rng.gen_range(1..=3.min(i)); // Connect to 1-3 existing nodes
        
        for _ in 0..connections {
            let mut target_idx = 0;
            let mut cumulative = 0;
            let threshold = rng.gen_range(0..total_degree);
            
            for (idx, &deg) in degrees.iter().enumerate() {
                cumulative += deg + 1; // +1 to ensure all nodes have some chance
                if cumulative > threshold {
                    target_idx = idx;
                    break;
                }
            }
            
            commands.spawn_edge(node_entities[target_idx], node);
            degrees[target_idx] += 1;
        }
        
        node_entities.push(node);
        degrees.push(connections);
    }
    
    let total_edges: usize = degrees.iter().sum::<usize>() / 2;
    println!("Spawned scale-free graph: {} nodes, ~{} edges", num_nodes, total_edges);
}

// ============================================================================
// Event System Integration
// ============================================================================

/// Statistics tracking for graph events
#[derive(Resource, Default, Debug)]
struct GraphEventStats {
    node_added_count: usize,
    node_removed_count: usize,
    edge_added_count: usize,
    edge_removed_count: usize,
    node_changed_count: usize,
    edge_changed_count: usize,
    propagation_count: usize,
}

/// Component to mark nodes that should be highlighted due to propagation
#[derive(Component, Default)]
struct PropagationHighlight {
    start_time: f32,
    duration: f32,
}

/// System to log graph events to console
fn log_graph_events(
    mut reader: MessageReader<CircuitGraphMessage>,
    mut stats: ResMut<GraphEventStats>,
) {
    for message in reader.read() {
        match message {
            CircuitGraphMessage::NodeAdded { entity, initial_neighbors } => {
                stats.node_added_count += 1;
                println!(
                    "✓ NodeAdded: entity={:?}, neighbors={}",
                    entity,
                    initial_neighbors.len()
                );
            }
            CircuitGraphMessage::NodeRemoved { entity, affected_edges } => {
                stats.node_removed_count += 1;
                println!(
                    "✗ NodeRemoved: entity={:?}, affected_edges={}",
                    entity,
                    affected_edges.len()
                );
            }
            CircuitGraphMessage::EdgeAdded { entity, from, to } => {
                stats.edge_added_count += 1;
                println!("+ EdgeAdded: edge={:?}, from={:?}, to={:?}", entity, from, to);
            }
            CircuitGraphMessage::EdgeRemoved { entity, from, to } => {
                stats.edge_removed_count += 1;
                println!("- EdgeRemoved: edge={:?}, from={:?}, to={:?}", entity, from, to);
            }
            CircuitGraphMessage::NodeChanged {
                entity,
                affected_neighbors,
                changes,
            } => {
                stats.node_changed_count += 1;
                println!(
                    "↻ NodeChanged: entity={:?}, affected_neighbors={}, transform_changed={}",
                    entity, affected_neighbors.len(), changes.transform_changed
                );
            }
            CircuitGraphMessage::EdgeChanged { entity, from, to, changes } => {
                stats.edge_changed_count += 1;
                println!(
                    "↻ EdgeChanged: edge={:?}, from={:?}, to={:?}, color_changed={}",
                    entity, from, to, changes.color_changed
                );
            }
            CircuitGraphMessage::PropagationTriggered {
                source,
                affected_nodes,
                propagation_type,
            } => {
                stats.propagation_count += 1;
                println!(
                    "🌊 PropagationTriggered: source={:?}, affected_nodes={}, type={:?}",
                    source, affected_nodes.len(), propagation_type
                );
            }
            CircuitGraphMessage::NodeConnected { node, edge, neighbor } => {
                println!(
                    "🔗 NodeConnected: node={:?}, edge={:?}, neighbor={:?}",
                    node, edge, neighbor
                );
            }
            CircuitGraphMessage::NodeDisconnected { node, edge, neighbor } => {
                println!(
                    "🔌 NodeDisconnected: node={:?}, edge={:?}, neighbor={:?}",
                    node, edge, neighbor
                );
            }
            // Legacy variants (if they exist)
            _ => {
                // Other message types are ignored or handled elsewhere
            }
        }
    }
}

/// System to visualize propagation by highlighting affected nodes
fn visualize_propagation(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut reader: MessageReader<CircuitGraphMessage>,
    nodes: Query<Entity, With<CircuitNode>>,
    time: Res<Time>,
) {
    // Create a highlight material (bright green/yellow)
    let highlight_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.8, 0.2), // Bright yellow-orange
        emissive: LinearRgba::new(0.5, 0.4, 0.1, 1.0),   // Glow effect
        metallic: 0.3,
        perceptual_roughness: 0.5,
        ..default()
    });

    for message in reader.read() {
        if let CircuitGraphMessage::PropagationTriggered {
            source: _,
            affected_nodes,
            propagation_type: _,
        } = message
        {
            for node_entity in affected_nodes {
                // Check if node exists
                if nodes.get(*node_entity).is_ok() {
                    // Apply highlight material
                    commands.entity(*node_entity).insert(MeshMaterial3d(highlight_material.clone()));
                    
                    // Add highlight component with timestamp
                    commands.entity(*node_entity).insert(PropagationHighlight {
                        start_time: time.elapsed_secs(),
                        duration: 2.0, // Highlight for 2 seconds
                    });
                }
            }
        }
    }
}

/// System to fade highlights back to original color over time
fn fade_highlights(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut nodes: Query<
        (Entity, &PropagationHighlight),
        (With<CircuitNode>, With<MeshMaterial3d<StandardMaterial>>),
    >,
    time: Res<Time>,
) {
    let default_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.9), // Light blue-gray (default node color)
        metallic: 0.3,
        perceptual_roughness: 0.5,
        ..default()
    });

    let current_time = time.elapsed_secs();
    
    for (entity, highlight) in nodes.iter_mut() {
        let elapsed = current_time - highlight.start_time;
        if elapsed >= highlight.duration {
            // Fade complete, restore default material
            commands.entity(entity).insert(MeshMaterial3d(default_material.clone()));
            commands.entity(entity).remove::<PropagationHighlight>();
        } else {
            // Fade between highlight and default
            let fade_factor = 1.0 - (elapsed / highlight.duration);
            let highlight_color = LinearRgba::new(1.0, 0.8, 0.2, 1.0);
            let default_color = LinearRgba::new(0.8, 0.8, 0.9, 1.0);
            
            let faded_color = LinearRgba::new(
                highlight_color.red * fade_factor + default_color.red * (1.0 - fade_factor),
                highlight_color.green * fade_factor + default_color.green * (1.0 - fade_factor),
                highlight_color.blue * fade_factor + default_color.blue * (1.0 - fade_factor),
                1.0,
            );
            
            let faded_material = materials.add(StandardMaterial {
                base_color: faded_color.into(),
                emissive: LinearRgba::new(0.5, 0.4, 0.1, 1.0) * fade_factor,
                metallic: 0.3,
                perceptual_roughness: 0.5,
                ..default()
            });
            
            commands.entity(entity).insert(MeshMaterial3d(faded_material));
        }
    }
}

/// System to handle keyboard input for interactive demonstrations
fn handle_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<GraphEventPropagationConfig>,
    graph_query: CircuitGraphQuery,
    nodes: Query<Entity, With<CircuitNode>>,
    mut graph_messages: MessageWriter<CircuitGraphMessage>,
) {
    // Spacebar: Trigger propagation from a random node
    if keyboard_input.just_pressed(KeyCode::Space) {
        let node_entities: Vec<Entity> = nodes.iter().collect();
        if !node_entities.is_empty() {
            let mut rng = rand::thread_rng();
            if let Some(&random_node) = node_entities.choose(&mut rng) {
                // Manually trigger a NodeChanged event to demonstrate propagation
                let affected_neighbors = graph_query.affected_neighbors(random_node);
                
                graph_messages.write(CircuitGraphMessage::NodeChanged {
                    entity: random_node,
                    affected_neighbors: affected_neighbors.clone(),
                    changes: bild_canvas::circuit::graph::NodeChangeSet {
                        transform_changed: true,
                        global_transform_changed: false,
                        layer_id_changed: false,
                    },
                });
                
                println!("🎯 Triggered propagation from node {:?} ({} neighbors)", random_node, affected_neighbors.len());
            }
        }
    }
    
    // Number keys: Switch propagation modes
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        config.default_node_propagation = PropagationType::ImmediateNeighbors;
        println!("📊 Propagation mode: ImmediateNeighbors");
    }
    if keyboard_input.just_pressed(KeyCode::Digit2) {
        config.default_node_propagation = PropagationType::ConnectedComponent;
        println!("📊 Propagation mode: ConnectedComponent");
    }
    if keyboard_input.just_pressed(KeyCode::Digit3) {
        config.default_node_propagation = PropagationType::Downstream;
        println!("📊 Propagation mode: Downstream");
    }
    if keyboard_input.just_pressed(KeyCode::Digit4) {
        config.default_node_propagation = PropagationType::Upstream;
        println!("📊 Propagation mode: Upstream");
    }
    if keyboard_input.just_pressed(KeyCode::Digit5) {
        config.default_node_propagation = PropagationType::LimitedDepth { depth: 2 };
        println!("📊 Propagation mode: LimitedDepth (depth=2)");
    }
    
    // Toggle propagation on/off
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        config.propagate_node_changes = !config.propagate_node_changes;
        println!("🔛 Propagation enabled: {}", config.propagate_node_changes);
    }
}

use rand::seq::SliceRandom;
