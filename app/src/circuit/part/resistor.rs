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
    Capacitive, FrequencyDependent, Inductive, NoiseGenerating, PackageType, PowerRated, Resistive, Semiconductor 
};
use circuit_physics_core::material_properties::MaterialProperties;





/// Fixed resistor component
#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Resistor {
    pub size: (u32, u32, u32),
    pub faces: Vec<Face>,
    pub resistance: ElectricalResistance,
    pub power_rating: Power,
    pub tolerance: f64, // Percentage
    pub temperature_coefficient: f64, // ppm/°C
    pub package: PackageType,
    pub operating_temperature: ThermodynamicTemperature,
}

impl Default for Resistor {
    fn default() -> Self {
        Self {
            size: (1, 1, 1),
            faces: Vec::new(),
            resistance: ElectricalResistance::new::<ohm>(1000.0),
            power_rating: Power::new::<watt>(0.25),
            tolerance: 5.0,
            temperature_coefficient: 100.0,
            package: PackageType::SurfaceMount,
            operating_temperature: ThermodynamicTemperature::new::<kelvin>(298.15),
        }
    }
}

impl Resistor {
    pub fn new(size: (u32, u32, u32), resistance: ElectricalResistance, power_rating: Power, tolerance: f64, temperature_coefficient: f64, package: PackageType, operating_temperature: ThermodynamicTemperature) -> Self {
        Self {
            size,
            faces: Vec::new(),
            resistance,
            power_rating,
            tolerance,
            temperature_coefficient,
            package,
            operating_temperature,
        }
    }
}

impl Block3DLike for Resistor {
    fn size(&self) -> (u32, u32, u32) { self.size }
    fn faces(&self) -> impl Iterator<Item = Face> { self.faces.iter().cloned() }
    fn symbol(&self) -> String { "R".to_string() }
}

impl Resistive for Resistor {
    fn resistance(&self) -> ElectricalResistance { self.resistance }
    fn tolerance(&self) -> f64 { self.tolerance }
    fn temperature_coefficient(&self) -> f64 { self.temperature_coefficient }
}

impl PowerRated for Resistor {
    fn power_rating(&self) -> Power { self.power_rating }
    fn current_rating(&self) -> ElectricCurrent {
        ElectricCurrent::new::<ampere>((self.power_rating.get::<watt>() / self.resistance.get::<ohm>()).sqrt())
    }
    fn voltage_rating(&self) -> ElectricPotential {
        ElectricPotential::new::<volt>((self.power_rating.get::<watt>() * self.resistance.get::<ohm>()).sqrt())
    }
    fn is_within_safe_operating_area(&self, voltage: ElectricPotential, current: ElectricCurrent) -> bool {
        let power = Power::new::<watt>(voltage.get::<volt>() * current.get::<ampere>());
        power <= self.power_rating && voltage <= self.voltage_rating() && current <= self.current_rating()
    }
}

impl NoiseGenerating for Resistor {
    fn thermal_noise_density(&self, temperature: ThermodynamicTemperature) -> f64 {
        let k_b = 1.380649e-23; // Boltzmann constant
        (4.0 * k_b * temperature.get::<kelvin>() * self.resistance.get::<ohm>()).sqrt()
    }
    fn flicker_noise_corner(&self) -> Option<Frequency> { None }
}

impl MaterialProperties for Resistor {
    fn thermal_conductivity(&self) -> f32 { 1.0 } // Generic resistor
    fn electrical_resistivity(&self) -> f32 { self.resistance.get::<ohm>() as f32 }
    fn youngs_modulus(&self) -> f32 { 1e9 }
    fn poisson_ratio(&self) -> f32 { 0.3 }
    fn density(&self) -> f32 { 2000.0 }
    fn specific_heat(&self) -> f32 { 1000.0 }
}

// Manual Hash implementation ignoring floating point fields
impl std::hash::Hash for Resistor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.size.hash(state);
        self.faces.hash(state);
        // Skip floating point fields for hashing
        self.package.hash(state);
    }
}

// Manual Eq implementation ignoring floating point fields
impl Eq for Resistor {}