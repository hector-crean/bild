use bevy::{picking::pointer::PointerInteraction, prelude::*};

use crate::view::layout_3d::tool::GroundPlane;
pub struct GroundPlanePlugin;

impl Plugin for GroundPlanePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, GroundPlane::spawn)
            .add_systems(Update, GroundPlane::draw_cursor);
    }
}


impl GroundPlane {
    fn spawn(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
        commands.spawn((
            GroundPlane,
            Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::new(1000., 1000.)))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, 0.5), // Semi-transparent white
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::default(),
            Pickable::IGNORE, // Disable picking for the ground plane.

        ));
    }
    fn draw_cursor(
        camera_query: Single<(&Camera, &GlobalTransform)>,
        ground: Single<&GlobalTransform, With<Self>>,
        windows: Query<&Window>,
        mut gizmos: Gizmos,
    ) {
        let Ok(windows) = windows.single() else {
            return;
        };

        let (camera, camera_transform) = *camera_query;

        let Some(cursor_position) = windows.cursor_position() else {
            return;
        };

        // Calculate a ray pointing from the camera into the world based on the cursor's position.
        let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
            return;
        };

        // Calculate if and where the ray is hitting the ground plane.
        let Some(distance) =
            ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up()))
        else {
            return;
        };
        let point = ray.get_point(distance);

        // Draw a circle just above the ground plane at that position.
        gizmos.circle(
            Isometry3d::new(
                point + ground.up() * 0.01,
                Quat::from_rotation_arc(Vec3::Z, ground.up().as_vec3()),
            ),
            0.2,
            Color::WHITE,
        );
    }
}



