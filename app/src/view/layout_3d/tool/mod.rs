pub mod block_tool;
pub mod comment_tool;
pub mod markup_tool;
pub mod transform_tool;
use bevy::{color::palettes, prelude::*};

use block_tool::{BlockToolPlugin, BlockToolState};
use block3d_core::block::{Block3DLike};
use comment_tool::{CommentToolPlugin, CommentToolState};
use layer::Layer;
use markup_tool::{MarkupToolPlugin, MarkupToolState};
use serde::{Deserialize, Serialize};
use transform_tool::{TransformToolPlugin, TransformToolState};
use ts_rs::TS;
use widget_2d::toolbar::ToolbarState;
use widget_3d::radial_menu::RadialItemData;

use crate::{circuit::part::Part, event::BildOutEvent, view::layout_3d::CameraSettingsImpl};

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

pub struct  ToolPlugin;

impl Plugin for ToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ToolState>()
            .add_sub_state::<CommentToolState>()
            .add_sub_state::<TransformToolState>()
            .add_sub_state::<MarkupToolState>()
            .add_sub_state::<BlockToolState>()
            .add_plugins((
                CommentToolPlugin,
                TransformToolPlugin::<CameraSettingsImpl, GroundPlane>::default(),
                MarkupToolPlugin,
                BlockToolPlugin::<Part, GroundPlane>::default(),
            ))
            .add_systems(
                Update,
                (
                    handle_tool_state_transition
                        .run_if(on_event::<StateTransitionEvent<ToolState>>),
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

// Newtype wrapper to work around orphan rule
#[derive(Debug, Clone)]
pub struct RadialMenuItem<T>(pub T);

// Trait for converting to RadialItemData that we can implement locally
pub trait ToRadialItem {
    fn to_radial_item(self) -> RadialItemData;
}

impl<T> From<RadialMenuItem<T>> for RadialItemData
where
    T: ToRadialItem,
{
    fn from(item: RadialMenuItem<T>) -> Self {
        item.0.to_radial_item()
    }
}

impl ToRadialItem for Part {
    fn to_radial_item(self) -> RadialItemData {
        match self {
            Part::Resistor(_) => RadialItemData {
                icon: "resistor".to_string(),
                color: palettes::css::ORANGE.into(),
                label: "Resistor".to_string(),
            },
            Part::Capacitor(_) => RadialItemData {
                icon: "capacitor".to_string(),
                color: palettes::css::BLUE.into(),
                label: "Capacitor".to_string(),
            },
            Part::Inductor(_) => RadialItemData {
                icon: "inductor".to_string(),
                color: palettes::css::GREEN.into(),
                label: "Inductor".to_string(),
            },
            Part::Diode(_) => RadialItemData {
                icon: "diode".to_string(),
                color: palettes::css::RED.into(),
                label: "Diode".to_string(),
            },
          
        }
    }
}
