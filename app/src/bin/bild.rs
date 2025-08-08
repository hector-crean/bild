
use bevy::{app::{App, AppExit}};

use bild_app::app::AppPlugin;

pub fn main() -> AppExit {
   App::new().add_plugins(AppPlugin).run()
}
