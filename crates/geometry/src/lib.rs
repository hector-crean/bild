//! Geometric primitives extending Bevy's shape system
//! 
//! This crate provides additional geometric primitives that implement Bevy's
//! `Primitive2d`, `Primitive3d`, `Measured2d`, and `Measured3d` traits.
//! 
//! It also provides a granular material system where each geometry representation
//! is a simple Asset type with its own specific material trait and plugin system 
//! (e.g., `SDFMaterial`, `PointCloudMaterial`).

pub mod primitives;
pub mod traits;
pub mod representation;

pub mod prelude {
    // Re-export Bevy's primitive traits
    pub use bevy::math::primitives::{Primitive2d, Primitive3d, Measured2d, Measured3d};
    
    // Re-export our custom traits
    pub use crate::traits::*;
    
}