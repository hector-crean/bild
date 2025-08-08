pub mod comment;
pub mod simulation;
pub mod time;
pub mod selection;

pub mod actions;
pub mod keybinding;
pub mod utils;


use bevy::prelude::*;

use crate::{
    actions::ActionsPlugin, keybinding::KeybindingPlugin, selection::SelectionPlugin,
    utils::CoreUtilsPlugin,
};

/// Crate prelude.
pub mod prelude {
    pub use crate::{
        actions::{ActionAppExt, ActionWorldExt},
        keybinding::{Keybinding, KeybindingAppExt},
        selection::EditorSelection,
        utils::IntoBoxedScene,
    };
}

/// Core plugin for the editor.
#[derive(Default)]
pub struct BildCorePlugin;

impl Plugin for BildCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ActionsPlugin,
            KeybindingPlugin,
            SelectionPlugin,
            CoreUtilsPlugin,
        ));
    }
}
