use bevy::app::{PluginGroup, PluginGroupBuilder};

use crate::{comment::CommentUiPlugin, toolbar::ToolbarPlugin};

pub mod toolbar;
pub mod comment;



pub struct UiPlugin;


impl PluginGroup for UiPlugin {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(CommentUiPlugin)

    }
}