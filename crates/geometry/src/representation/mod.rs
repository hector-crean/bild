//! Geometry representation types and their rendering infrastructure
//! 
//! This module provides a granular material system where each geometry representation
//! is a simple Asset type with its own specific material trait and plugin system.
//! Uses Bevy's Extract trait for proper render world data extraction.

use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::RenderApp;

pub mod sdf;
pub mod pointcloud;
pub mod voxel_grid;
pub mod heightfields;
pub mod metaballs;
pub mod octrees;
pub mod wireframes;
pub mod splines;
pub mod nurbs;
pub mod bezier_curve;
pub mod brep;
pub mod csg;
pub mod mesh;
pub mod meshlets;
pub mod polyline;