pub mod analog_components;
pub mod physical_simulation_traits;

// Re-export the unified analog component enum
pub use analog_components::AnalogComponent;

use serde::{Deserialize, Serialize};
use strum::AsRefStr;
use strum::EnumProperty;
use crate::face::Face;
use strum::EnumIter;

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default, Hash, Eq, PartialEq, EnumIter, strum::Display, EnumProperty, AsRefStr)]
pub enum BlockKind {
    /// Silicon die - the main processing unit
    #[strum(props(
        icon = "icons/chip_48px.png",
        color = "#2E2E2E",  // Dark silicon
        thermal_conductivity = "148.0",  // W/m·K for silicon
        electrical_resistivity = "1e5",  // Ω·m for lightly doped silicon
        youngs_modulus = "130e9",        // Pa
        poisson_ratio = "0.27",
        density = "2329.0",              // kg/m³
        specific_heat = "712.0"          // J/kg·K
    ))]
    SiliconDie,
    
    /// Substrate/package substrate
    #[strum(props(
        icon = "icons/substrate_48px.png",
        color = "#8B4513",  // Brown
        thermal_conductivity = "20.0",
        electrical_resistivity = "1e12",
        youngs_modulus = "25e9",
        poisson_ratio = "0.25",
        density = "1900.0",
        specific_heat = "1000.0"
    ))]
    Substrate,
    
    /// Heat spreader (usually copper or aluminum)
    #[strum(props(
        icon = "icons/heat_spreader_48px.png",
        color = "#B87333",  // Copper color
        thermal_conductivity = "400.0",  // Copper
        electrical_resistivity = "1.7e-8",
        youngs_modulus = "110e9",
        poisson_ratio = "0.34",
        density = "8960.0",
        specific_heat = "385.0"
    ))]
    HeatSpreader,
    
    /// Thermal Interface Material (TIM)
    #[strum(props(
        icon = "icons/tim_48px.png",
        color = "#C0C0C0",  // Silver
        thermal_conductivity = "5.0",
        electrical_resistivity = "1e10",
        youngs_modulus = "1e6",
        poisson_ratio = "0.45",
        density = "2000.0",
        specific_heat = "800.0"
    ))]
    ThermalInterfaceMaterial,
    
    /// Heat sink fins
    #[strum(props(
        icon = "icons/heat_sink_48px.png",
        color = "#A9A9A9",  // Aluminum
        thermal_conductivity = "237.0",  // Aluminum
        electrical_resistivity = "2.8e-8",
        youngs_modulus = "70e9",
        poisson_ratio = "0.33",
        density = "2700.0",
        specific_heat = "900.0"
    ))]
    HeatSink,
    
    /// Wire bonds or solder balls
    #[strum(props(
        icon = "icons/interconnect_48px.png",
        color = "#FFD700",  // Gold
        thermal_conductivity = "318.0",  // Gold
        electrical_resistivity = "2.4e-8",
        youngs_modulus = "78e9",
        poisson_ratio = "0.42",
        density = "19300.0",
        specific_heat = "129.0"
    ))]
    Interconnect,
    
    /// PCB/Circuit board
    #[strum(props(
        icon = "icons/pcb_48px.png",
        color = "#228B22",  // Green PCB
        thermal_conductivity = "0.3",   // FR4
        electrical_resistivity = "1e14",
        youngs_modulus = "22e9",
        poisson_ratio = "0.28",
        density = "1850.0",
        specific_heat = "1100.0"
    ))]
    CircuitBoard,
    
    /// Underfill material
    #[strum(props(
        icon = "icons/underfill_48px.png",
        color = "#DDA0DD",  // Plum
        thermal_conductivity = "0.8",
        electrical_resistivity = "1e12",
        youngs_modulus = "5e9",
        poisson_ratio = "0.35",
        density = "1800.0",
        specific_heat = "1200.0"
    ))]
    Underfill,
    
    /// Air/void space for thermal analysis
    #[strum(props(
        icon = "icons/air_48px.png",
        color = "#87CEEB",  // Light blue
        thermal_conductivity = "0.026",  // Air
        electrical_resistivity = "1e16",
        youngs_modulus = "0.0",
        poisson_ratio = "0.0",
        density = "1.2",
        specific_heat = "1005.0"
    ))]
    #[default]
    Air,
}

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



