use bevy::app::{PluginGroup, PluginGroupBuilder};

use self::comment::CommentUiPlugin;

pub mod toolbar;
pub mod comment;


pub struct Widget2dPlugin;


impl PluginGroup for Widget2dPlugin {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(CommentUiPlugin)

    }
}