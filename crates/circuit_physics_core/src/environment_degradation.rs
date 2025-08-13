use serde::{Serialize, Deserialize};
// use bevy::prelude::*;
// use uom::si::f64::*;
use uom::si::{
    acceleration::meter_per_second_squared, electric_current::ampere, electric_potential::volt, electrical_resistance::ohm, f32::{Acceleration, ElectricCurrent, ElectricField, ElectricalResistance, Frequency, MagneticFluxDensity, Power, ThermodynamicTemperature, Time}, frequency::hertz, luminous_intensity::candela, magnetic_flux_density::tesla, power::watt, pressure::pascal, thermodynamic_temperature::kelvin, time::second
};






// ============================================================================
// CHEMICAL AND ENVIRONMENTAL TRAITS
// ============================================================================

/// Components susceptible to environmental degradation
pub trait EnvironmentalDegradation {
    /// Humidity sensitivity level (1-6 per IPC standards)
    fn moisture_sensitivity_level(&self) -> u8;
    
    /// Corrosion resistance rating
    fn corrosion_resistance(&self) -> CorrosionResistance;
    
    /// Outgassing characteristics (TML, CVCM percentages)
    fn outgassing_properties(&self) -> (f32, f32); // (TML, CVCM)
    
    /// Chemical compatibility with common materials
    fn chemical_compatibility(&self, material: ChemicalMaterial) -> bool;
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum CorrosionResistance {
    Excellent,
    Good,
    Fair,
    Poor,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum ChemicalMaterial {
    Silicone,
    Epoxy,
    Polyurethane,
    Fluorocarbon,
    Alcohol,
    Acetone,
    Water,
}
