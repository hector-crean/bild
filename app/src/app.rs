

use bevy::prelude::*;
use bild_core::BildCorePlugin;
use camera::controller::{pan_orbit_camera::{OrbitCameraController, OrbitCameraControllerPlugin}, CameraSettings};
use camera::markers::MainCamera;
use duplex::event_channel::GlobalEventChannel;
use geometry::representation::polyline::PolylinePlugin;
use interaction::InteractiveMeshPlugin;
use once_cell::sync::Lazy;
use picking::{double_click::DoubleClickPlugin, PickingExtraPlugin};
use styles::StylesPlugin;
use widget_3d::Ui3dPlugin;
use bevy::picking::pointer::PointerInteraction;
use crate::{event::{BildInEvent, BildOutEvent}, tool::GroundPlane, view::ViewPlugin};
// #[cfg(not(target_arch = "wasm32"))]
use widget_2d::{toolbar::ToolbarPlugin, UiPlugin};
use crate::tool::ToolPlugin;
#[cfg(not(target_arch = "wasm32"))]
use crate::tool::ToolState;
use pane_layout::{PaneLayoutPlugin, RootPaneLayoutNode};
use pane_layout::prelude::*;
use viewport_2d::Viewport2dPanePlugin;
use viewport_3d::Viewport3dPanePlugin;

pub static GLOBAL_EVENT_CHANNEL: Lazy<GlobalEventChannel<BildInEvent, BildOutEvent>> =
    Lazy::new(GlobalEventChannel::new);


use duplex::create_duplex_plugin;


use bevy::app::App;
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowTheme};
use bevy::winit::WinitSettings;

use std::collections::HashSet;
use std::time::Duration;
use std::collections::HashMap;
use bevy::input::common_conditions::input_just_pressed;
use bevy::picking::events::Pointer;
use bevy::picking::pointer::PointerButton;



pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Set up console error logging
        console_error_panic_hook::set_once();

        let (duplex_plugin, js_rx, js_tx) = create_duplex_plugin::<BildInEvent, BildOutEvent>();

        GLOBAL_EVENT_CHANNEL.set_sender(js_tx);
        GLOBAL_EVENT_CHANNEL.set_receiver(js_rx);

        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Self::configure_window()),
                    ..default()
                })
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..Default::default()
                })
                .build(),
        );
        app.insert_resource(WinitSettings {
            focused_mode: bevy::winit::UpdateMode::Reactive { wait: Duration::from_millis(10), react_to_device_events: true, react_to_user_events: true, react_to_window_events: true },
            unfocused_mode: bevy::winit::UpdateMode::reactive_low_power(Duration::from_millis(10)),
        })
        .insert_resource(CameraSettingsImpl::default());


        app
           .add_plugins((
                duplex_plugin,
                StylesPlugin,
                OrbitCameraControllerPlugin::<CameraSettingsImpl>::default(),
                PolylinePlugin,
                ToolPlugin,
                ViewPlugin,  // Add view system
                PaneLayoutPlugin, // Add pane layout system
                Viewport3dPanePlugin, // Register 3D viewport pane
                Viewport2dPanePlugin, // Register 2D viewport pane
                MeshPickingPlugin,
                PickingExtraPlugin,
                Ui3dPlugin,
                UiPlugin,
                GroundPlanePlugin,
                InteractiveMeshPlugin::<CameraSettingsImpl>::default(),
            ));
            app.add_plugins(BildCorePlugin);
        
        // Ensure a pane root exists for the layout system
        app.add_systems(PreStartup, Self::spawn_pane_root);

         

        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins(ToolbarPlugin::<ToolState>::default());

        app.add_systems(Startup, (Self::setup_camera));


        app.add_systems(
                Update,
                (
                    BildInEvent::handle.run_if(on_event::<BildInEvent>),
                    BildOutEvent::handle.run_if(on_event::<BildOutEvent>),
                ),
            );

        app.add_systems(Update, draw_mesh_intersections);

    }
}

impl AppPlugin {
    fn configure_window() -> Window {
        Window {
            title: "bild".into(),
            name: Some("bild.app".into()), 
            canvas: Some("#bild-canvas".into()),
            resolution: (1920., 1080.).into(),
            present_mode: PresentMode::AutoVsync,
            fit_canvas_to_parent: true,
            prevent_default_event_handling: true,
            window_theme: Some(WindowTheme::Dark),
            enabled_buttons: bevy::window::EnabledButtons {
                maximize: false,
                ..Default::default()
            },
            visible: true,
            ..default()
        }
    }

    fn setup_camera(
        mut commands: Commands,
        windows: Query<Entity, With<Window>>,
        asset_server: Res<AssetServer>,
    ) {
        let controller = OrbitCameraController::default();
        let transform = controller.generate_transform();

        // Add camera
        commands.spawn((
            Camera3d::default(),
            MainCamera,
            controller,
            transform,
        ));

        // Add ambient light with improved settings
        commands.insert_resource(AmbientLight {
            color: Color::srgb(0.9, 0.9, 1.0), // Slightly blue-tinted white for better atmosphere
            brightness: 0.4,                  // Increased brightness for better visibility
            ..default()
        });

        // Add primary directional light (sun-like)
        commands.spawn((
            DirectionalLight {
                color: Color::srgb(1.0, 0.95, 0.9), // Warm sunlight color
                illuminance: 12000.0,              // More realistic illuminance value
                shadows_enabled: true,
                shadow_depth_bias: 0.02,           // Reduce shadow acne
                shadow_normal_bias: 0.6,           // Improve shadow quality
                ..default()
            },
            Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ));

        // Add secondary fill light (opposite direction, lower intensity)
        commands.spawn((
            DirectionalLight {
                color: Color::srgb(0.8, 0.85, 1.0), // Cooler color for fill light
                illuminance: 3000.0,               // Lower intensity than main light
                shadows_enabled: false,            // No shadows from fill light
                ..default()
            },
            Transform::from_xyz(-3.0, 5.0, -3.0).looking_at(Vec3::ZERO, Vec3::Y),
        ));
    }

    fn spawn_pane_root(mut commands: Commands) {
        commands.spawn(RootPaneLayoutNode);
    }
}





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




/// A system that draws hit indicators for every pointer.
fn draw_mesh_intersections(pointers: Query<&PointerInteraction>, mut gizmos: Gizmos) {
    for (point, normal) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position.zip(hit.normal))
    {
        gizmos.sphere(point, 0.05, bevy::color::palettes::tailwind::RED_500);
        gizmos.arrow(point, point + normal.normalize() * 0.5, bevy::color::palettes::tailwind::PINK_500);
    }
}
