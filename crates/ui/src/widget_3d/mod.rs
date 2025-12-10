use bevy::app::{PluginGroup, PluginGroupBuilder};

use self::radial_menu::RadialMenuPlugin;

pub mod cursor;
pub mod radial_menu;
pub mod worldspace_ui_node;


pub struct Widget3dPlugin;


impl PluginGroup for Widget3dPlugin {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(RadialMenuPlugin)
    }
}