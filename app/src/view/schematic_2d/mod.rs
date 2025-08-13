
use bevy::prelude::*;
use camera_2d::{EditorCamera2d, EditorCamera2dPlugin};
use interaction::{drag::two_d::{Drag2dSettings}, InteractiveMeshPlugin};
use camera::controller::CameraSettings;
use crate::circuit::{part::Part, commands::CommandsCircuitExt};





pub struct Schematic2dPlugin { }

impl Plugin for Schematic2dPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Camera2dSettingsImpl>()
            .add_plugins(EditorCamera2dPlugin)
            .add_plugins(InteractiveMeshPlugin::<Camera2dSettingsImpl>::default())
            .add_systems(Startup, (setup_camera_2d, init_drag_settings, setup_demo_circuit));
    }
}


fn setup_demo_circuit(mut commands: Commands) {
    // Nets
    let vcc = commands.spawn_net("VCC");
    let gnd = commands.spawn_net("GND");
    let n1 = commands.spawn_net("N1");

    // Parts with pins
    let (_r1, r1_pins) = commands.spawn_part_with_pins("R1", Part::resistor(), &["1", "2"]);
    let (_c1, c1_pins) = commands.spawn_part_with_pins("C1", Part::capacitor(), &["1", "2"]);

    // Connect pins to nets: VCC -> R1.1 -> N1 -> C1.1 -> GND
    commands.connect_pin_to_net(r1_pins[0], vcc);
    commands.connect_pin_to_net(r1_pins[1], n1);
    commands.connect_pin_to_net(c1_pins[0], n1);
    commands.connect_pin_to_net(c1_pins[1], gnd);
}




#[derive(Resource, Default, PartialEq, Eq)]
struct Camera2dSettingsImpl { locked: bool }

impl CameraSettings for Camera2dSettingsImpl {
    fn is_locked(&self) -> bool { self.locked }
    fn lock(&mut self) { self.locked = true; }
    fn unlock(&mut self) { self.locked = false; }
}

fn setup_camera_2d(mut commands: Commands) {
    commands.spawn((Camera2d, EditorCamera2d::default(), Transform::from_xyz(0.0, 0.0, 999.0)));
}

fn init_drag_settings(mut settings: ResMut<Drag2dSettings>) {
    settings.enabled = true;
    settings.grid_snapping = Some(Vec2::splat(10.0));
    settings.axis_constraints = None;
}






