
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





// ============================================================================
// SEMICONDUCTOR COMPONENTS
// ============================================================================

/// Diode component
#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Diode {
    pub size: (u32, u32, u32),
    pub faces: Vec<Face>,
    pub forward_voltage: ElectricPotential,
    pub reverse_breakdown_voltage: ElectricPotential,
    pub forward_current_rating: ElectricCurrent,
    pub reverse_recovery_time: f64, // nanoseconds
    pub junction_capacitance: Capacitance,
    pub package: PackageType,
    pub operating_temperature: ThermodynamicTemperature,
}

impl Default for Diode {
    fn default() -> Self {
        Self {
            size: (1, 1, 1),
            faces: Vec::new(),
            forward_voltage: ElectricPotential::new::<volt>(0.7),
            reverse_breakdown_voltage: ElectricPotential::new::<volt>(100.0),
            forward_current_rating: ElectricCurrent::new::<ampere>(1.0),
            reverse_recovery_time: 10.0,
            junction_capacitance: Capacitance::new::<farad>(10e-12), // 10pF
            package: PackageType::SurfaceMount,
            operating_temperature: ThermodynamicTemperature::new::<kelvin>(298.15),
        }
    }
}

impl Block3DLike for Diode {
    fn size(&self) -> (u32, u32, u32) { self.size }
    fn faces(&self) -> impl Iterator<Item = Face> { self.faces.iter().cloned() }
    fn symbol(&self) -> String { "D".to_string() }
}

impl Semiconductor for Diode {
    fn threshold_voltage(&self) -> Option<ElectricPotential> { Some(self.forward_voltage) }
    fn forward_voltage(&self) -> Option<ElectricPotential> { Some(self.forward_voltage) }
    fn breakdown_voltage(&self) -> Option<ElectricPotential> { Some(self.reverse_breakdown_voltage) }
    fn junction_temperature(&self) -> Option<ThermodynamicTemperature> { Some(self.operating_temperature) }
}

impl PowerRated for Diode {
    fn power_rating(&self) -> Power {
        Power::new::<watt>(self.forward_voltage.get::<volt>() * self.forward_current_rating.get::<ampere>())
    }
    fn current_rating(&self) -> ElectricCurrent { self.forward_current_rating }
    fn voltage_rating(&self) -> ElectricPotential { self.reverse_breakdown_voltage }
    fn is_within_safe_operating_area(&self, voltage: ElectricPotential, current: ElectricCurrent) -> bool {
        current <= self.forward_current_rating && voltage <= self.reverse_breakdown_voltage
    }
}

impl MaterialProperties for Diode {
    fn thermal_conductivity(&self) -> f32 { 148.0 } // Silicon
    fn electrical_resistivity(&self) -> f32 { 1e5 }
    fn youngs_modulus(&self) -> f32 { 130e9 }
    fn poisson_ratio(&self) -> f32 { 0.27 }
    fn density(&self) -> f32 { 2329.0 }
    fn specific_heat(&self) -> f32 { 712.0 }
}
