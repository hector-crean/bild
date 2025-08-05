use bevy::prelude::*;
use std::marker::PhantomData;

pub trait Layer: Component + Default {

}

pub struct LayerPlugin<T: Layer> {
    phantom: PhantomData<T>,
}

impl<T: Layer> Plugin for LayerPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::spawn)
            .add_systems(Update, Self::draw_cursor);
    }
}



impl<T: Layer> LayerPlugin<T> {
    fn spawn(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        commands.spawn((
            T::default(),
            Transform::from_translation(Vec3::ZERO),
            Mesh3d(meshes.add(Plane3d::default())),
            MeshMaterial3d(materials.add(StandardMaterial::default())),
            // PickingBehavior::IGNORE,
        ));
    }

    fn draw_cursor(
        camera_query: Single<(&Camera, &GlobalTransform)>,
        ground: Single<&GlobalTransform, With<T>>,
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
