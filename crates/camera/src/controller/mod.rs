pub mod pan_orbit_camera;

use bevy::{prelude::*, render::{camera::Camera, view::RenderLayers}};

pub trait CameraController: Component
where
    Self: Sized,
{
    fn update_camera_transform_system(
        query: Query<(&Self, &mut Transform), (Or<(Changed<Self>, Added<Self>)>, With<Camera3d>)>,
    );
}



pub trait CameraSettings: Resource + Default + PartialEq + Eq {
    fn is_locked(&self) -> bool;
    fn lock(&mut self);
    fn unlock(&mut self);
}