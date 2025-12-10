use bevy::prelude::*;
use geometry::representation::polyline::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PolylinePlugin))
        .add_systems(Startup, (setup_camera, spawn_polyline))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_polyline(
    mut commands: Commands,
    mut polylines: ResMut<Assets<Polyline>>,
    mut materials: ResMut<Assets<PolylineMaterial>>,
) {
    // Simple zig-zag polyline in the X-Y plane
    let vertices = vec![
        Vec3::new(-2.0, 0.0, 0.0),
        Vec3::new(-1.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
    ];

    let polyline = polylines.add(Polyline { vertices });

    let mut mat = PolylineMaterial::default();
    mat.color = Color::srgb(0.95, 0.35, 0.35).to_linear();
    mat.width = 6.0; // pixels at near plane

    let material = materials.add(mat);

    commands.spawn((
        PolylineHandle(polyline),
        PolylineMaterialHandle(material),
        Transform::default(),
    ));
}

