

use bevy::prelude::*;
use bild_core::BildCorePlugin;
// use camera::controller::pan_orbit_camera::OrbitCameraController;
use duplex::event_channel::GlobalEventChannel;
use geometry::representation::polyline::PolylinePlugin;
use once_cell::sync::Lazy;
use picking::PickingExtraPlugin;
use styles::StylesPlugin;
use widget_3d::Ui3dPlugin;
use crate::{
    event::{BildInEvent, BildOutEvent},
    view::{schematic_2d::Schematic2dPlugin},
};
// use crate::circuit::commands::CommandsCircuitExt;
// use crate::circuit::part::Part;
// #[cfg(not(target_arch = "wasm32"))]
use widget_2d::UiPlugin;
// #[cfg(not(target_arch = "wasm32"))]
// use crate::tool::ToolState;

pub static GLOBAL_EVENT_CHANNEL: Lazy<GlobalEventChannel<BildInEvent, BildOutEvent>> =
    Lazy::new(GlobalEventChannel::new);


use duplex::create_duplex_plugin;


use bevy::app::App;
use bevy::window::{PresentMode, WindowTheme};
use bevy::winit::WinitSettings;

use std::time::Duration;
 



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
        });


        app
           .add_plugins((
            BildCorePlugin,
                duplex_plugin,
                StylesPlugin,
                PolylinePlugin,
                MeshPickingPlugin,
                PickingExtraPlugin,
                Ui3dPlugin,
                UiPlugin,
                //views
                // Layout3dViewPlugin,
                Schematic2dPlugin {},
            ));
        


        app.add_systems(
                Update,
                (
                    BildInEvent::handle.run_if(on_event::<BildInEvent>),
                    BildOutEvent::handle.run_if(on_event::<BildOutEvent>),
                ),
            );

        // Demo removed; add your own Startup systems as needed.


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
}

