pub mod selection;
pub mod drag;

use std::marker::PhantomData;

use bevy::prelude::*;

use camera::controller::CameraSettings;

use drag::{three_d::DragTransform3dPlugin, two_d::DragTransform2dPlugin};
use selection::SelectionPlugin;


#[derive(Default)]
pub struct InteractiveMeshPlugin<T: CameraSettings> {
    phantom_camera: PhantomData<T>,
}



impl<T: CameraSettings> Plugin for InteractiveMeshPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_plugins((SelectionPlugin, DragTransform3dPlugin::<T>::default(), DragTransform2dPlugin::<T>::default()));
    }
}
