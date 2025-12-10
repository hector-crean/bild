use bevy::prelude::*;

use super::components::{
    CircuitNode, EdgeStartTransform, EdgeEndTransform,
};

pub struct CircuitGraphRenderPlugin;

impl Plugin for CircuitGraphRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BatchedEdgeMesh>()
            .add_systems(Update, (
                spawn_node_meshes,
                update_batched_edge_mesh,
            ).chain());
    }
}

/// Resource to hold a single batched mesh for all edges
#[derive(Resource)]
struct BatchedEdgeMesh {
    mesh_handle: Option<Handle<Mesh>>,
    material_handle: Option<Handle<StandardMaterial>>,
    entity: Option<Entity>,
    edge_count: usize,
}

impl Default for BatchedEdgeMesh {
    fn default() -> Self {
        Self {
            mesh_handle: None,
            material_handle: None,
            entity: None,
            edge_count: 0,
        }
    }
}

/// Spawn mesh entities for nodes that don't have them yet
/// Nodes already benefit from automatic batching when sharing mesh/material handles
fn spawn_node_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    nodes: Query<Entity, (With<CircuitNode>, Without<Mesh3d>)>,
) {
    // Create a shared material for all nodes - this enables automatic batching
    let node_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.9), // Light blue-gray
        metallic: 0.3,
        perceptual_roughness: 0.5,
        ..default()
    });

    // Create a shared sphere mesh - this enables automatic batching
    let sphere_mesh = meshes.add(Sphere::new(0.1));

    // Insert shared mesh and material - Bevy will automatically batch these
    for node_entity in nodes.iter() {
        commands.entity(node_entity).insert((
            Mesh3d(sphere_mesh.clone()),
            MeshMaterial3d(node_material.clone()),
        ));
    }
}

/// Update a single batched mesh containing all edges
/// This reduces edge rendering from N draw calls to 1 draw call
fn update_batched_edge_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut batched_mesh: ResMut<BatchedEdgeMesh>,
    edges: Query<
        (&EdgeStartTransform, &EdgeEndTransform),
        Or<(Changed<EdgeStartTransform>, Changed<EdgeEndTransform>)>,
    >,
    all_edges: Query<(&EdgeStartTransform, &EdgeEndTransform), (With<EdgeStartTransform>, With<EdgeEndTransform>)>,
) {
    // Check if we need to update (any edge changed or mesh doesn't exist)
    let needs_update = !edges.is_empty() || batched_mesh.mesh_handle.is_none();
    
    if !needs_update {
        return;
    }

    // Collect all edge positions into a single polyline mesh
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut current_index = 0u32;

    for (start_transform, end_transform) in all_edges.iter() {
        // Add two vertices for this edge
        positions.push(start_transform.0.translation.to_array());
        positions.push(end_transform.0.translation.to_array());
        
        // Add indices for the line segment
        indices.push(current_index);
        indices.push(current_index + 1);
        
        current_index += 2;
    }

    if positions.is_empty() {
        // No edges to render - despawn batched entity if it exists
        if let Some(entity) = batched_mesh.entity {
            commands.entity(entity).despawn();
        }
        batched_mesh.mesh_handle = None;
        batched_mesh.entity = None;
        batched_mesh.edge_count = 0;
        return;
    }

    // Create a single batched mesh for all edges using LineList topology
    // This allows us to render all edges in a single draw call
    use bevy::render::render_resource::PrimitiveTopology;
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::Indices;
    
    let batched_mesh_data = Mesh::new(
        PrimitiveTopology::LineList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_indices(Indices::U32(indices));
    
    let mesh_handle = meshes.add(batched_mesh_data);
    batched_mesh.edge_count = all_edges.iter().len();
    
    // Get or create the shared material
    let edge_material = batched_mesh.material_handle.get_or_insert_with(|| {
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 0.5),
            unlit: true, // Lines typically don't need lighting
            ..default()
        })
    }).clone();

    // Update or create the batched edge entity
    if let Some(existing_entity) = batched_mesh.entity {
        // Update existing entity's mesh
        commands.entity(existing_entity).insert(Mesh3d(mesh_handle.clone()));
    } else {
        // Create new batched edge entity
        let entity = commands.spawn((
            Mesh3d(mesh_handle.clone()),
            MeshMaterial3d(edge_material),
            BatchedEdgeRenderer, // Marker component
        )).id();
        
        batched_mesh.entity = Some(entity);
    }
    
    batched_mesh.mesh_handle = Some(mesh_handle);
}

/// Marker component for the batched edge renderer entity
#[derive(Component)]
struct BatchedEdgeRenderer;