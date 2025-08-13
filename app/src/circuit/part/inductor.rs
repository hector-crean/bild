use block3d_core::block::Block3DLike;
use block3d_core::face::Face;
use serde::{Serialize, Deserialize};
use bevy::prelude::*;
use strum::EnumIter;
use uom::si::f64::*;
use uom::si::{
    electric_potential::volt,
    electric_current::ampere,
    electrical_resistance::ohm,
    capacitance::farad,
    inductance::henry,
    frequency::hertz,
    power::watt,
    thermodynamic_temperature::kelvin,
};
use circuit_physics_core::physical::{
    Capacitive, CoreMaterial, FrequencyDependent, Inductive, NoiseGenerating, PackageType, PowerRated, Resistive, Semiconductor 
};

use circuit_physics_core::material_properties::MaterialProperties;




/// Fixed inductor component
#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Inductor {
    pub size: (u32, u32, u32),
    pub faces: Vec<Face>,
    pub inductance: Inductance,
    pub current_rating: ElectricCurrent,
    pub dc_resistance: ElectricalResistance,
    pub core_material: CoreMaterial,
    pub package: PackageType,
    pub operating_temperature: ThermodynamicTemperature,
}

impl Default for Inductor {
    fn default() -> Self {
        Self {
            size: (1, 1, 1),
            faces: Vec::new(),
            inductance: Inductance::new::<henry>(1e-6), // 1µH
            current_rating: ElectricCurrent::new::<ampere>(1.0),
            dc_resistance: ElectricalResistance::new::<ohm>(0.1),
            core_material: CoreMaterial::Ferrite,
            package: PackageType::SurfaceMount,
            operating_temperature: ThermodynamicTemperature::new::<kelvin>(298.15),
        }
    }
}

impl Block3DLike for Inductor {
    fn size(&self) -> (u32, u32, u32) { self.size }
    fn faces(&self) -> impl Iterator<Item = Face> { self.faces.iter().cloned() }
    fn symbol(&self) -> String { "L".to_string() }
}

impl Inductive for Inductor {
    fn inductance(&self) -> Inductance { self.inductance }
    fn current_rating(&self) -> ElectricCurrent { self.current_rating }
    fn dc_resistance(&self) -> ElectricalResistance { self.dc_resistance }
    fn core_material(&self) -> CoreMaterial { self.core_material }
}

impl FrequencyDependent for Inductor {
    fn bandwidth(&self) -> Option<Frequency> { None }
    fn self_resonant_frequency(&self) -> Option<Frequency> {
        // Rough approximation with parasitic capacitance
        let parasitic_c = 1e-12; // 1pF typical
        Some(Frequency::new::<hertz>(1.0 / (2.0 * std::f64::consts::PI * 
            (self.inductance.get::<henry>() * parasitic_c).sqrt())))
    }
    fn impedance_at_frequency(&self, frequency: Frequency) -> ElectricalResistance {
        let omega = 2.0 * std::f64::consts::PI * frequency.get::<hertz>();
        let reactance = omega * self.inductance.get::<henry>();
        ElectricalResistance::new::<ohm>(reactance)
    }
}

impl MaterialProperties for Inductor {
    fn thermal_conductivity(&self) -> f32 { 50.0 } // Copper wire + ferrite
    fn electrical_resistivity(&self) -> f32 { self.dc_resistance.get::<ohm>() as f32 }
    fn youngs_modulus(&self) -> f32 { 100e9 }
    fn poisson_ratio(&self) -> f32 { 0.35 }
    fn density(&self) -> f32 { 5000.0 }
    fn specific_heat(&self) -> f32 { 500.0 }
}
