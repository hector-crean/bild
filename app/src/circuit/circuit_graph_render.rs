

use bevy::prelude::*;
use geometry::representation::polyline::prelude::*;
// use interaction::drag::three_d::Draggable3d;

use super::circuit_graph::{EdgeFrom, EdgeTo, EdgeColor, CircuitEdgeQuery, CircuitNodeQuery, CircuitNode};

#[derive(Resource, Clone)]
pub struct GraphRenderConfig {
    pub default_color: Color,
    pub width: f32,
    pub draw_nodes: bool,
    pub node_radius: f32,
    pub node_color: Color,
    pub node_segments: usize,
}

impl Default for GraphRenderConfig {
    fn default() -> Self {
        Self {
            default_color: Color::WHITE,
            width: 2.0,
            draw_nodes: true,
            node_radius: 0.12,
            node_color: Color::srgb(1.0, 0.6, 0.2),
            node_segments: 16,
        }
    }
}

// Rendering is attached directly to the edge/node entities; no indirection entities needed.

pub struct GraphRenderPlugin;

impl Plugin for GraphRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphRenderConfig>()
            .add_systems(Update, (
                spawn_edge_polylines,
                update_edge_polylines,
                cleanup_edge_polylines,
                spawn_node_meshes,
            ));
    }
}

fn spawn_edge_polylines(
    mut commands: Commands,
    config: Res<GraphRenderConfig>,
    mut polylines: ResMut<Assets<Polyline>>,
    mut materials: ResMut<Assets<PolylineMaterial>>,
    edges: Query<CircuitEdgeQuery, Without<PolylineHandle>>,
    transforms: Query<&GlobalTransform>,
) {
    let edge_count = edges.iter().count();
    if edge_count > 0 {
        info!("Found {edge_count} edges without polylines, spawning...");
    }

    for edge in &edges {
        let (Ok(ta), Ok(tb)) = (transforms.get(edge.edge_from.0), transforms.get(edge.edge_to.0)) else { continue; };
        let a = ta.translation();
        let b = tb.translation();
        info!("Spawning edge polyline for edge {:?}: {} -> {}", edge.entity, edge.edge_from.0, edge.edge_to.0);
        let polyline = polylines.add(Polyline { vertices: vec![a, b] });
        let material = materials.add(PolylineMaterial { width: config.width, color: config.default_color.to_linear(), depth_bias: -0.01, perspective: false });
        commands.entity(edge.entity).insert((
            PolylineHandle(polyline),
            PolylineMaterialHandle(material),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ));
    }
}

fn update_edge_polylines(
    edges: Query<CircuitEdgeQuery, (With<PolylineHandle>, With<PolylineMaterialHandle>)>,
    transforms: Query<&GlobalTransform, Changed<GlobalTransform>>,
    mut polylines: ResMut<Assets<Polyline>>,
    mut materials: ResMut<Assets<PolylineMaterial>>,
) {
    let changed_transforms = transforms.iter().count();
    if changed_transforms > 0 {
        info!("Detected {} changed transforms, updating edge polylines...", changed_transforms);
    }

    for edge in &edges {
        if transforms.get(edge.edge_from.0).is_err() && transforms.get(edge.edge_to.0).is_err() {
            continue;
        }
        if let Some(polyline_h) = edge.polyline {
            if let Some(polyline) = polylines.get_mut(&polyline_h.0) {
                if let (Ok(ta), Ok(tb)) = (transforms.get(edge.edge_from.0), transforms.get(edge.edge_to.0)) {
                    let a = ta.translation();
                    let b = tb.translation();
                    polyline.vertices = vec![a, b];
                    info!("Updated edge polyline vertices: {} -> {}", edge.edge_from.0, edge.edge_to.0);
                }
            }
        }
        if let (Some(material_h), Some(color)) = (edge.material, edge.edge_color) {
            if let Some(mat) = materials.get_mut(&material_h.0) {
                mat.color = color.0.to_linear();
            }
        }
    }
}

fn cleanup_edge_polylines(
    mut commands: Commands,
    edges_missing: Query<Entity, Or<(Without<EdgeFrom>, Without<EdgeTo>)>>,
) {
    for edge in &edges_missing {
        commands
            .entity(edge)
            .remove::<(PolylineHandle, PolylineMaterialHandle, Visibility, InheritedVisibility, ViewVisibility)>();
    }
}

fn spawn_node_meshes(
    mut commands: Commands,
    config: Res<GraphRenderConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    nodes: Query<CircuitNodeQuery, Without<Mesh3d>>,
) {
    if !config.draw_nodes { return; }

    for node in &nodes {
        // Only render nodes that have connections (degree > 0) or are junctions
        let has_connections = node.incoming_edges.is_some() || node.outgoing_edges.is_some();
        let is_junction = matches!(node.node_kind, CircuitNode::Branch);
        
        if !has_connections && !is_junction { continue; }

        let center = node.global_transform.translation();
        info!("Spawning node mesh for {:?} at {}", node.entity, center);
        let material = materials.add(StandardMaterial {
            base_color: config.node_color,
            unlit: true,
            ..default()
        });
        commands.entity(node.entity).insert((
            Mesh3d(meshes.add(Circle::new(config.node_radius))),
            MeshMaterial3d(material),
        ));
    }
}

// No node update system required: the mesh lives on the node and follows its Transform.

// circle_vertices no longer needed; nodes are rendered as mesh circles


