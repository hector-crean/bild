pub mod comment_tool;
pub mod transform_tool;
pub mod markup_tool;
pub mod block_tool;
use bevy::prelude::*;

use block3d_core::block::lego_block::LegoBlock;
use block_tool::{BlockToolPlugin, BlockToolState};
use comment_tool::{ CommentToolPlugin, CommentToolState};
use layer::Layer;
use markup_tool::{MarkupToolPlugin, MarkupToolState};
use transform_tool::{TransformToolPlugin, TransformToolState};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use ui::toolbar::ToolbarState;

use crate::{app::CameraSettingsImpl, event::BildOutEvent};

// #[derive(
//     Debug, Clone, Copy, Eq, PartialEq, Hash, States, Component,
//     strum::EnumIter, strum::IntoStaticStr,
//     serde::Serialize, serde::Deserialize, TS,
// )]
// #[serde(tag = "type", content = "data")]
// pub enum ToolState {
//     Move(TransformTool),
//     Comment(CommentTool),
// }

// impl Default for ToolState {
//     fn default() -> Self {
//         Self::Move(TransformTool::default())
//     }
// }

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
pub enum ToolState {
    #[default]
    #[strum(props(icon = "icons/navigation_48px.png"))]
    Transform,
    #[strum(props(icon = "icons/message-circle_48px.png"))]
    Comment,
    #[strum(props(icon = "icons/pen_48px.png"))]
    Markup,
    #[strum(props(icon = "icons/box_48px.png"))]
    Block,
}

impl ToolbarState for ToolState {
    fn get_icon(&self) -> &'static str {
        match self {
            ToolState::Transform => "icons/navigation_48px.png",
            ToolState::Comment => "icons/message-circle_48px.png",
            ToolState::Markup => "icons/pen_48px.png",
            ToolState::Block => "icons/box_48px.png",
        }
    }
}



#[derive(Component, Default)]
pub struct GroundPlane;

impl Layer for GroundPlane {}


pub struct ToolPlugin;

impl Plugin for ToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ToolState>()
            .add_sub_state::<CommentToolState>()
            .add_sub_state::<TransformToolState>()
            .add_sub_state::<MarkupToolState>()
            .add_sub_state::<BlockToolState>()
            .add_plugins((CommentToolPlugin, TransformToolPlugin::<CameraSettingsImpl>::default(), MarkupToolPlugin, BlockToolPlugin::<LegoBlock, GroundPlane>::default()))
            .add_systems(
                Update,
                (
                    handle_tool_state_transition.run_if(on_event::<StateTransitionEvent<ToolState>>),
                ),
            );
    }
}

fn handle_tool_state_transition(
    tool_state: Res<State<ToolState>>,
    mut rdr: EventReader<StateTransitionEvent<ToolState>>,
    mut crayon_out: EventWriter<BildOutEvent>,
) {
    info!("Tool state changed to: {:?}", tool_state.get());
    for StateTransitionEvent { exited, entered } in rdr.read() {
        match (exited, entered) {
            (_, Some(to)) => {
                crayon_out.write(BildOutEvent::ToolChanged(*to));
            }
            _ => {
                error!("Tool state transition event is missing exited or entered state");
            }
        }
    }
}




