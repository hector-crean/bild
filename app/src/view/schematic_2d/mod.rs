pub mod node_tool;

use bevy::prelude::*;
use node_tool::{NodeToolPlugin, NodeToolState};
use strum::{EnumIter, IntoEnumIterator};
use widget_3d::radial_menu::{OpenRadialMenu, RadialItemData, RadialMenuPosition};

use super::ViewState;

/// Tools specific to the 2D Schematic view
#[derive(Default, SubStates, Debug, Clone, Eq, PartialEq, Hash, EnumIter)]
#[source(ViewState = ViewState::Schematic2D)]
pub enum Schematic2DToolState {
    #[default]
    Node,     // Node placement/editing
    Edge,     // Edge drawing (to be implemented)
    Select,   // Selection and manipulation (to be implemented)
    Pan,      // Pan/zoom navigation (to be implemented)
}

impl From<Schematic2DToolState> for RadialItemData {
    fn from(state: Schematic2DToolState) -> Self {
        match state {
            Schematic2DToolState::Node => RadialItemData {
                label: "Node".to_string(),
                icon: "icons/node_tool.png".to_string(),
                color: Color::WHITE,
            },
            Schematic2DToolState::Edge => RadialItemData {
                label: "Edge".to_string(),
                icon: "icons/edge_tool.png".to_string(),
                color: Color::WHITE,
            },
            Schematic2DToolState::Select => RadialItemData {
                label: "Select".to_string(),
                icon: "icons/select_tool.png".to_string(),
                color: Color::WHITE,
            },
            Schematic2DToolState::Pan => RadialItemData {
                label: "Pan".to_string(),
                icon: "icons/pan_tool.png".to_string(),
                color: Color::WHITE,
            },
        }
    }
}

/// Plugin for the 2D Schematic view and its tools
pub struct Schematic2DViewPlugin;

impl Plugin for Schematic2DViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<Schematic2DToolState>()
            .add_sub_state::<NodeToolState>()
            .add_plugins(NodeToolPlugin)
            .add_systems(
                Update,
                Self::handle_tool_state_transition
                    .run_if(on_event::<StateTransitionEvent<Schematic2DToolState>>),
            )
            .add_systems(OnEnter(ViewState::Schematic2D), Self::enter_schematic_view)
            .add_systems(OnExit(ViewState::Schematic2D), Self::exit_schematic_view);
    }
}

impl Schematic2DViewPlugin {
    fn handle_tool_state_transition(
        mut state_reader: EventReader<StateTransitionEvent<Schematic2DToolState>>,
    ) {
        for event in state_reader.read() {
            info!(
                "Schematic2DToolState changed from {:?} to {:?}",
                event.exited, event.entered
            );
        }
    }

    fn enter_schematic_view(
        mut commands: Commands,
        windows: Query<&Window>,
        mut open_radial_menu: EventWriter<OpenRadialMenu>,
        mut next_tool_state: ResMut<NextState<Schematic2DToolState>>,
    ) {
        info!("Entering 2D Schematic view");
        
        // Set default tool for this view
        next_tool_state.set(Schematic2DToolState::Node);

        // Show tool selection radial menu
        if let Ok(window) = windows.single() {
            let items: Vec<RadialItemData> = Schematic2DToolState::iter()
                .map(|state| state.into())
                .collect();

            open_radial_menu.write(OpenRadialMenu {
                position: RadialMenuPosition::ScreenCenter,
                items,
            });
        }
    }

    fn exit_schematic_view() {
        info!("Exiting 2D Schematic view");
        // Cleanup view-specific resources if needed
    }
} 