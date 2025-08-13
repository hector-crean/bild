pub mod ground_plane;
pub mod tool;

use bevy::{
    asset::uuid::Uuid,
    feathers::theme::ThemedText,
    picking::{
        PickingSystems,
        input::{mouse_pick_events, touch_pick_events},
        pointer::{Location, PointerId, PointerInput, PointerInteraction},
    },
    prelude::*,
    render::{
        camera::{NormalizedRenderTarget, RenderTarget},
        render_resource::{Extent3d, TextureFormat, TextureUsages},
        view::RenderLayers,
    },
    scene2::{CommandsSpawnScene, bsn, on},
    ui::ui_layout_system,
};
use camera::{
    controller::{
        CameraSettings,
        pan_orbit_camera::{OrbitCameraController, OrbitCameraControllerPlugin},
    },
    markers::MainCamera,
};
use camera_3d::prelude::{DefaultEditorCamPlugins, EditorCam};
use infinite_grid::{InfiniteGrid, InfiniteGridPlugin, InfiniteGridSettings};
use interaction::InteractiveMeshPlugin;
use pane_layout::{
    Active, PaneView, pointer_id_from_entity, prelude::*, render_target_picking_passthrough,
    update_render_target_size,
};
use styles::Theme;
#[cfg(not(target_arch = "wasm32"))]
use widget_2d::toolbar::ToolbarPlugin;

#[cfg(not(target_arch = "wasm32"))]
use crate::view::layout_3d::tool::ToolState;
use crate::view::layout_3d::{ground_plane::GroundPlanePlugin, tool::ToolPlugin};

#[derive(Resource, Default, PartialEq, Eq)]
pub struct CameraSettingsImpl {
    is_locked: bool,
}

impl CameraSettings for CameraSettingsImpl {
    fn is_locked(&self) -> bool {
        self.is_locked
    }

    fn lock(&mut self) {
        self.is_locked = true;
    }

    fn unlock(&mut self) {
        self.is_locked = false;
    }
}

/// Plugin for the 3D Viewport pane.
pub struct Layout3dViewPlugin;

impl Plugin for Layout3dViewPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<InfiniteGridPlugin>() {
            app.add_plugins(InfiniteGridPlugin);
        }

        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins(ToolbarPlugin::<ToolState>::default());

        app.add_plugins((
            InteractiveMeshPlugin::<CameraSettingsImpl>::default(),
            GroundPlanePlugin,
            ToolPlugin,
            OrbitCameraControllerPlugin::<CameraSettingsImpl>::default(),
        ))
        .add_systems(Startup, (setup_view, setup_scene))
        .add_systems(Update, draw_mesh_intersections);
    }
}

fn setup_view(mut commands: Commands) {
    let controller = OrbitCameraController::default();
    let transform = controller.generate_transform();

    // Add ambient light with improved settings
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.9, 0.9, 1.0), // Slightly blue-tinted white for better atmosphere
        brightness: 0.4,                   // Increased brightness for better visibility
        ..default()
    });

    // Add primary directional light (sun-like)
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.9), // Warm sunlight color
            illuminance: 12000.0,               // More realistic illuminance value
            shadows_enabled: true,
            shadow_depth_bias: 0.02, // Reduce shadow acne
            shadow_normal_bias: 0.6, // Improve shadow quality
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Add secondary fill light (opposite direction, lower intensity)
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.8, 0.85, 1.0), // Cooler color for fill light
            illuminance: 3000.0,                // Lower intensity than main light
            shadows_enabled: false,             // No shadows from fill light
            ..default()
        },
        Transform::from_xyz(-3.0, 5.0, -3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let camera_id = commands
        .spawn((
            Camera3d::default(),
            MeshPickingCamera,
            MainCamera,
            controller,
            transform,
        ))
        .id();
}

fn setup_scene(mut commands: Commands, theme: Res<Theme>) {
    commands.spawn((
        InfiniteGrid,
        InfiniteGridSettings {
            x_axis_color: theme.viewport.x_axis_color,
            z_axis_color: theme.viewport.z_axis_color,
            major_line_color: theme.viewport.grid_major_line_color,
            minor_line_color: theme.viewport.grid_minor_line_color,
            ..default()
        },
        // RenderLayers::layer(1),
    ));
}


/// A system that draws hit indicators for every pointer.
fn draw_mesh_intersections(pointers: Query<&PointerInteraction>, mut gizmos: Gizmos) {
    for (point, normal) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position.zip(hit.normal))
    {
        gizmos.sphere(point, 0.05, bevy::color::palettes::tailwind::RED_500);
        gizmos.arrow(
            point,
            point + normal.normalize() * 0.5,
            bevy::color::palettes::tailwind::PINK_500,
        );
    }
}
