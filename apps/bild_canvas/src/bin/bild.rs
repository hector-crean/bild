
use bevy::{app::{App, AppExit}};

use bild_canvas::app::AppPlugin;

pub fn main() -> AppExit {
   App::new().add_plugins(AppPlugin).run()
}
