
use serde::{Serialize, Deserialize};

use uom::si::f64::*;


// ============================================================================
// PHYSICAL PROPERTY TRAITS
// ============================================================================

/// Components that exhibit electrical resistance
pub trait Resistive {
    fn resistance(&self) -> ElectricalResistance;
    fn tolerance(&self) -> f64; // Percentage tolerance
    fn temperature_coefficient(&self) -> f64; // ppm/°C
}

/// Components that store electrical energy in an electric field
pub trait Capacitive {
    fn capacitance(&self) -> Capacitance;
    fn voltage_rating(&self) -> ElectricPotential;
    fn dielectric_type(&self) -> DielectricType;
    fn equivalent_series_resistance(&self) -> ElectricalResistance;
}

/// Components that store electrical energy in a magnetic field
pub trait Inductive {
    fn inductance(&self) -> Inductance;
    fn current_rating(&self) -> ElectricCurrent;
    fn dc_resistance(&self) -> ElectricalResistance;
    fn core_material(&self) -> CoreMaterial;
}

/// Components with frequency-dependent behavior
pub trait FrequencyDependent {
    fn bandwidth(&self) -> Option<Frequency>;
    fn self_resonant_frequency(&self) -> Option<Frequency>;
    fn impedance_at_frequency(&self, frequency: Frequency) -> ElectricalResistance;
}

/// Components that generate or handle power
pub trait PowerRated {
    fn power_rating(&self) -> Power;
    fn current_rating(&self) -> ElectricCurrent;
    fn voltage_rating(&self) -> ElectricPotential;
    fn is_within_safe_operating_area(&self, voltage: ElectricPotential, current: ElectricCurrent) -> bool;
}

/// Components that exhibit thermal noise
pub trait NoiseGenerating {
    fn thermal_noise_density(&self, temperature: ThermodynamicTemperature) -> f64; // V/√Hz
    fn flicker_noise_corner(&self) -> Option<Frequency>;
}

/// Components with semiconductor properties
pub trait Semiconductor {
    fn threshold_voltage(&self) -> Option<ElectricPotential>;
    fn forward_voltage(&self) -> Option<ElectricPotential>;
    fn breakdown_voltage(&self) -> Option<ElectricPotential>;
    fn junction_temperature(&self) -> Option<ThermodynamicTemperature>;
}

// ============================================================================
// SUPPORTING ENUMS
// ============================================================================

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DielectricType {
    Ceramic,
    Film,
    Electrolytic,
    Tantalum,
    Mica,
    Paper,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CoreMaterial {
    Air,
    Iron,
    Ferrite,
    Powdered,
    Laminated,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PackageType {
    ThroughHole,
    SurfaceMount,
    ChipOnBoard,
    BareChip,
}



