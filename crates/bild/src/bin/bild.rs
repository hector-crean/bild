
use bevy::{app::{App, AppExit}};

use bild::app::AppPlugin;

pub fn main() -> AppExit {
   App::new().add_plugins(AppPlugin).run()
}
