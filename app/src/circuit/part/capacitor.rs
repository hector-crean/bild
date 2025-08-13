
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
    Capacitive, FrequencyDependent, Inductive, NoiseGenerating, PackageType, PowerRated, Resistive, Semiconductor, DielectricType
};

use circuit_physics_core::material_properties::MaterialProperties;




/// Fixed capacitor component
#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Capacitor {
    pub size: (u32, u32, u32),
    pub faces: Vec<Face>,
    pub capacitance: Capacitance,
    pub voltage_rating: ElectricPotential,
    pub tolerance: f64,
    pub dielectric: DielectricType,
    pub esr: ElectricalResistance, // Equivalent Series Resistance
    pub package: PackageType,
    pub operating_temperature: ThermodynamicTemperature,
}

impl Default for Capacitor {
    fn default() -> Self {
        Self {
            size: (1, 1, 1),
            faces: Vec::new(),
            capacitance: Capacitance::new::<farad>(1e-9), // 1nF
            voltage_rating: ElectricPotential::new::<volt>(50.0),
            tolerance: 10.0,
            dielectric: DielectricType::Ceramic,
            esr: ElectricalResistance::new::<ohm>(0.01),
            package: PackageType::SurfaceMount,
            operating_temperature: ThermodynamicTemperature::new::<kelvin>(298.15),
        }
    }
}

impl Block3DLike for Capacitor {
    fn size(&self) -> (u32, u32, u32) { self.size }
    fn faces(&self) -> impl Iterator<Item = Face> { self.faces.iter().cloned() }
    fn symbol(&self) -> String { "C".to_string() }
}

impl Capacitive for Capacitor {
    fn capacitance(&self) -> Capacitance { self.capacitance }
    fn voltage_rating(&self) -> ElectricPotential { self.voltage_rating }
    fn dielectric_type(&self) -> DielectricType { self.dielectric }
    fn equivalent_series_resistance(&self) -> ElectricalResistance { self.esr }
}

impl FrequencyDependent for Capacitor {
    fn bandwidth(&self) -> Option<Frequency> { None }
    fn self_resonant_frequency(&self) -> Option<Frequency> {
        // Rough approximation: f = 1/(2π√(LC)) where L is parasitic inductance
        let parasitic_l = 1e-9; // 1nH typical
        Some(Frequency::new::<hertz>(1.0 / (2.0 * std::f64::consts::PI * 
            (parasitic_l * self.capacitance.get::<farad>()).sqrt())))
    }
    fn impedance_at_frequency(&self, frequency: Frequency) -> ElectricalResistance {
        let omega = 2.0 * std::f64::consts::PI * frequency.get::<hertz>();
        let reactance = 1.0 / (omega * self.capacitance.get::<farad>());
        ElectricalResistance::new::<ohm>(reactance)
    }
}

impl MaterialProperties for Capacitor {
    fn thermal_conductivity(&self) -> f32 { 0.5 }
    fn electrical_resistivity(&self) -> f32 { 1e14 } // High for dielectric
    fn youngs_modulus(&self) -> f32 { 5e9 }
    fn poisson_ratio(&self) -> f32 { 0.3 }
    fn density(&self) -> f32 { 3000.0 }
    fn specific_heat(&self) -> f32 { 800.0 }
}