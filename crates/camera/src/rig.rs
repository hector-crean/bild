use bevy::{prelude::*, render::{camera::Camera, view::RenderLayers}};


pub trait CameraRig: Resource {
    fn add_camera(&mut self, camera: Camera);
    fn remove_camera(&mut self, camera: &Camera);
    fn update(&mut self, cameras: &mut Query<&mut Camera>);
    //add/remove controller marker components? We want there to be only one controller component per camera bundle?
}

pub struct DefaultCameraRig {
    cameras: Vec<Entity>,
}

impl Default for DefaultCameraRig {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultCameraRig {
    pub fn new() -> Self {
        Self {
            cameras: Vec::new(),
        }
    }
}

