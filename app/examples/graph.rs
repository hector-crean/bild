use bevy::color::palettes;
use bevy::math::VectorSpace;
use bevy::prelude::*;
use bild_app::circuit::circuit_graph_render::{GraphRenderConfig, GraphRenderPlugin};
use camera::controller::pan_orbit_camera::{OrbitCameraController, OrbitCameraControllerPlugin};
use camera::controller::CameraSettings;
use camera::markers::MainCamera;
use camera_2d::{EditorCamera2d, EditorCamera2dPlugin};
use geometry::representation::polyline::PolylinePlugin;
use interaction::InteractiveMeshPlugin;
use interaction::drag::two_d::Draggable2d;
use interaction::drag::three_d::{DragTransform3dPlugin, Draggable3d};
use bild_app::circuit::circuit_graph::{CircuitGraph, CircuitGraphCommandsExt, EdgeColor};
use bild_app::circuit::graph_gizmos::{GraphGizmosPlugin, GraphGizmosConfig};


#[derive(Resource, Default, PartialEq, Eq)]
struct Camera3dSettingsImpl { locked: bool }

impl CameraSettings for Camera3dSettingsImpl {
    fn is_locked(&self) -> bool { self.locked }
    fn lock(&mut self) { self.locked = true; }
    fn unlock(&mut self) { self.locked = false; }
}

fn setup_camera_3d(mut commands: Commands) {
    let controller = OrbitCameraController::default();
    let transform = controller.generate_transform();
    commands.spawn((Camera3d::default(), MeshPickingCamera, MainCamera, controller, transform));
}



fn main() {
    App::new()
        .add_plugins((DefaultPlugins, OrbitCameraControllerPlugin::<Camera3dSettingsImpl>::default()))
        .init_resource::<Camera3dSettingsImpl>()
            .add_plugins(DragTransform3dPlugin::<Camera3dSettingsImpl>::default())
        .add_plugins((PolylinePlugin, GraphRenderPlugin))
        .insert_resource(GraphRenderConfig {
            node_color: palettes::tailwind::YELLOW_50.into(), // default line color when no EdgeColor present
            default_color: palettes::tailwind::ORANGE_50.into(), // node sphere color
            draw_nodes: true,
            node_radius: 0.2,
            node_segments: 50,
            width: 1.0
        })
        // .insert_resource(GraphGizmosConfig {
        //     line_color: palettes::tailwind::YELLOW_50.into(), 
        //     node_color: palettes::tailwind::ORANGE_50.into(), 
        //     draw_nodes: true
        //     })
        .add_systems(Startup, (setup_camera_3d, spawn_circuit_graph))
        .run();
}




fn spawn_circuit_graph(mut commands: Commands) {
    // Nodes must have a Transform/GlobalTransform so gizmos know where to draw
    let node_a = commands.spawn((Transform::from_xyz(-1.0, 0.0, 0.0), GlobalTransform::default())).id();
    let node_b = commands.spawn((Transform::from_xyz( 1.0, 0.0, 0.0), GlobalTransform::default())).id();
    let node_c = commands.spawn((Transform::from_xyz( 0.0, 1.0, 0.0), GlobalTransform::default())).id();
    let node_d = commands.spawn((Transform::from_xyz( 0.0, -1.0, 0.0), GlobalTransform::default())).id();
    let node_e = commands.spawn((Transform::from_xyz( 0.0, 0.0, 1.0), GlobalTransform::default())).id();
    let node_f = commands.spawn((Transform::from_xyz( 0.0, 0.0, -1.0), GlobalTransform::default())).id();

    // Edges (A->B, A->C)
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

