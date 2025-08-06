#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::{
    asset::{load_internal_asset, uuid_handle},
    prelude::*,
};
use material::PolylineMaterialPlugin;
use polyline::{PolylineBasePlugin, PolylineRenderPlugin};

pub mod material;
pub mod polyline;

pub mod prelude {
    pub use super::material::{PolylineMaterial, PolylineMaterialHandle};
    pub use super::polyline::{Polyline, PolylineBundle, PolylineHandle};
    pub use super::PolylinePlugin;
}
pub struct PolylinePlugin;

pub const SHADER_HANDLE: Handle<Shader> = uuid_handle!("b180bfe9-10c8-48fe-b27a-dfa41436d7d0");

impl Plugin for PolylinePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(
            app,
            SHADER_HANDLE,
            "shaders/polyline.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins((
            PolylineBasePlugin,
            PolylineRenderPlugin,
            PolylineMaterialPlugin,
        ));
    }
}