pub mod styles;
pub mod context_menu;
pub mod widget_2d;
pub mod widget_3d;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            styles::StylesPlugin,
            context_menu::ContextMenuPlugin,
            widget_2d::Widget2dPlugin,
            widget_3d::Widget3dPlugin,
        ));
    }
}

