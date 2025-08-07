use bevy::app::{PluginGroup, PluginGroupBuilder};

pub mod double_click;



pub struct PickingExtraPlugin;

   
impl PluginGroup for PickingExtraPlugin {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(double_click::DoubleClickPlugin)

    }
}
