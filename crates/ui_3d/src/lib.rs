use bevy::app::{PluginGroup, PluginGroupBuilder};

use crate::radial_menu::RadialMenuPlugin;

pub mod cursor;
pub mod radial_menu;
pub mod worldspace_ui_node;


pub struct Ui3dPlugin;


impl PluginGroup for Ui3dPlugin {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(RadialMenuPlugin)
    }
}