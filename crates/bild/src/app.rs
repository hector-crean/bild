

use bevy::prelude::*;
use camera::controller::{pan_orbit_camera::{OrbitCameraController, OrbitCameraControllerPlugin}, CameraSettings};
use camera::markers::MainCamera;
use duplex::event_channel::GlobalEventChannel;
use once_cell::sync::Lazy;
use picking::double_click::DoubleClickPlugin;
use ui_3d::Ui3dPlugin;

use crate::event::{BildInEvent, BildOutEvent};
// #[cfg(not(target_arch = "wasm32"))]
use ui::{toolbar::ToolbarPlugin, UiPlugin};
use crate::tool::ToolPlugin;
#[cfg(not(target_arch = "wasm32"))]
use crate::tool::ToolState;


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
                OrbitCameraControllerPlugin::<CameraSettingsImpl>::default(),
                ToolPlugin,
                MeshPickingPlugin,
                DoubleClickPlugin,
                Ui3dPlugin,
                UiPlugin,
            ));
        
         

        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins(ToolbarPlugin::<ToolState>::default());

        app.add_systems(Startup, (Self::setup_camera))
            .add_systems(
                Update,
                (
                    BildInEvent::handle.run_if(on_event::<BildInEvent>),
                    BildOutEvent::handle.run_if(on_event::<BildOutEvent>),
                ),
            );

    }
}

impl AppPlugin {
    fn configure_window() -> Window {
        Window {
            title: "Crayon".into(),
            name: Some("bevy.app".into()), 
            canvas: Some("#crayon-canvas".into()),
            resolution: (500., 300.).into(),
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

    fn setup_camera(mut commands: Commands) {
        let controller = OrbitCameraController::default();
        let transform = controller.generate_camera_transform();

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
}





#[derive(Resource, Default, PartialEq, Eq)]
pub struct CameraSettingsImpl {
    is_locked: bool,
}

impl CameraSettings for CameraSettingsImpl {
    fn is_locked(&self) -> bool {
        false
    }

    fn lock(&mut self) {
        self.is_locked = true;
    }

    fn unlock(&mut self) {
        self.is_locked = false;
    }
}