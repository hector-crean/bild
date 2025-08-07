pub mod analog_components;
pub mod physical_simulation_traits;

// Re-export the unified analog component enum
pub use analog_components::AnalogComponent;

use serde::{Deserialize, Serialize};
use strum::AsRefStr;
use strum::EnumProperty;
use crate::face::Face;
use strum::EnumIter;


/// Core trait for objects that can exist in a 3D grid and connect to other blocks
/// This trait focuses purely on spatial and connectivity concerns
pub trait Block3DLike: Clone + Default {
    /// The size this block occupies in grid units (width, height, depth)
    fn size(&self) -> (u32, u32, u32);
    
    /// The connection faces this block exposes to neighboring blocks
    fn faces(&self) -> impl Iterator<Item = Face>;

    /// Ranking for weighted random heuristic
    fn ranking(&self) -> f32 { 
        1.0
    }

    fn symbol(&self) -> String;
    
    /// Check if this block can be placed at a given position without conflicts
    fn can_place_at(&self, _position: (i32, i32, i32)) -> bool {
        true // Default implementation allows placement anywhere
    }
    
    /// Get all the grid positions this block would occupy if placed at the given position
    fn occupied_positions(&self, position: (i32, i32, i32)) -> Vec<(i32, i32, i32)> {
        let (width, height, depth) = self.size();
        let (x, y, z) = position;
        
        let mut positions = Vec::new();
        for dx in 0..width as i32 {
            for dy in 0..height as i32 {
                for dz in 0..depth as i32 {
                    positions.push((x + dx, y + dy, z + dz));
                }
            }
        }
        positions
    }
    
    /// Check if this block can connect to another block through the given faces
    fn can_connect_to<T: Block3DLike>(&self, other: &T, my_face: &Face, other_face: &Face) -> bool {
        my_face.can_connect_to(other_face)
    }
}

/// Separate trait for material/physical properties needed for physics simulation
pub trait MaterialProperties {
    fn thermal_conductivity(&self) -> f32;
    fn electrical_resistivity(&self) -> f32;
    fn youngs_modulus(&self) -> f32;
    fn poisson_ratio(&self) -> f32;
    fn density(&self) -> f32;
    fn specific_heat(&self) -> f32;
}

/// Optional trait for blocks that have a specific material/construction type
/// This can be used for legacy compatibility or when you need material-based grouping
pub trait HasMaterialType {
    type MaterialType;
    fn material_type(&self) -> Self::MaterialType;
}



