pub mod selection;
pub mod drag;

use std::marker::PhantomData;

use bevy::prelude::*;

use camera::controller::CameraSettings;

use drag::DragTransformPlugin;
use selection::SelectionPlugin;

pub struct InteractiveMeshPlugin<T: CameraSettings> {
    phantom_camera: PhantomData<T>,
}

impl<T: CameraSettings> Plugin for InteractiveMeshPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SelectionPlugin,
            DragTransformPlugin::<T>::default(),
        ));
    }
}
