pub mod schematic_2d;

use bevy::prelude::*;
use schematic_2d::{Schematic2DViewPlugin, Schematic2DToolState};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::event::BildOutEvent;

/// Top-level view states - defines what type of content we're viewing/editing
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    Eq,
    PartialEq,
    Hash,
    States,
    Component,
    strum::EnumIter,
    strum::IntoStaticStr,
    strum::EnumProperty,
    Serialize,
    Deserialize,
    TS,
)]
pub enum ViewState {
    #[default]
    #[strum(props(icon = "icons/model_3d_48px.png", label = "3D Model"))]
    Model3D,        // Existing 3D model view
    #[strum(props(icon = "icons/schematic_2d_48px.png", label = "2D Schematic"))]
    Schematic2D,    // New 2D schematic view
    // Future views can be added here:
    // TextEditor,     // Text/code editing view
    // Timeline,       // Timeline/animation view
}

impl ViewState {
    pub fn get_icon(&self) -> &'static str {
        match self {
            ViewState::Model3D => "icons/model_3d_48px.png",
            ViewState::Schematic2D => "icons/schematic_2d_48px.png",
        }
    }

    pub fn get_label(&self) -> &'static str {
        match self {
            ViewState::Model3D => "3D Model",
            ViewState::Schematic2D => "2D Schematic",
        }
    }
}

pub struct ViewPlugin;

impl Plugin for ViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ViewState>()
            .add_sub_state::<Schematic2DToolState>()
            .add_plugins(Schematic2DViewPlugin)
            .add_systems(
                Update,
                handle_view_state_transition
                    .run_if(on_event::<StateTransitionEvent<ViewState>>),
            );
    }
}

fn handle_view_state_transition(
    view_state: Res<State<ViewState>>,
    mut rdr: EventReader<StateTransitionEvent<ViewState>>,
    mut bild_out: EventWriter<BildOutEvent>,
) {
    info!("View state changed to: {:?}", view_state.get());
    for StateTransitionEvent { exited, entered } in rdr.read() {
        match (exited, entered) {
            (_, Some(to)) => {
                // We could send a ViewChanged event here if needed
                info!("Switched to view: {:?}", to);
            }
            _ => {
                error!("View state transition event is missing exited or entered state");
            }
        }
    }
} 