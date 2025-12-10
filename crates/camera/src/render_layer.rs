use bevy::{camera::visibility::RenderLayers, prelude::*};

pub const BACKGROUND: RenderLayers = RenderLayers::layer(1);
pub const FOREGROUND: RenderLayers = RenderLayers::layer(2);
