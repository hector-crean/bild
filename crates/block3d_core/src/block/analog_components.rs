use crate::face::Face;
use super::{Block3DLike, MaterialProperties};
use serde::{Serialize, Deserialize};
use bevy::prelude::*;
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

// ============================================================================
// PASSIVE COMPONENTS
// ============================================================================

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

impl Eq for Capacitor {}

impl Eq for Inductor {}

impl Eq for Diode {}

impl Eq for OpAmp {}

impl Eq for AnalogComponent {}

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

// ============================================================================
// INTEGRATED CIRCUITS
// ============================================================================

/// Operational Amplifier
#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OpAmp {
    pub size: (u32, u32, u32),
    pub faces: Vec<Face>,
    pub open_loop_gain: f64, // V/V
    pub bandwidth: Frequency,
    pub input_offset_voltage: ElectricPotential,
    pub input_bias_current: ElectricCurrent,
    pub slew_rate: f64, // V/µs
    pub supply_voltage_min: ElectricPotential,
    pub supply_voltage_max: ElectricPotential,
    pub package: PackageType,
    pub operating_temperature: ThermodynamicTemperature,
}

impl Default for OpAmp {
    fn default() -> Self {
        Self {
            size: (2, 2, 1),
            faces: Vec::new(),
            open_loop_gain: 1e6,
            bandwidth: Frequency::new::<hertz>(1e6),
            input_offset_voltage: ElectricPotential::new::<volt>(1e-6),
            input_bias_current: ElectricCurrent::new::<ampere>(1e-12),
            slew_rate: 1.0, // V/µs
            supply_voltage_min: ElectricPotential::new::<volt>(5.0),
            supply_voltage_max: ElectricPotential::new::<volt>(30.0),
            package: PackageType::SurfaceMount,
            operating_temperature: ThermodynamicTemperature::new::<kelvin>(298.15),
        }
    }
}

impl Block3DLike for OpAmp {
    fn size(&self) -> (u32, u32, u32) { self.size }
    fn faces(&self) -> impl Iterator<Item = Face> { self.faces.iter().cloned() }
    fn symbol(&self) -> String { "OpAmp".to_string() }
}

impl FrequencyDependent for OpAmp {
    fn bandwidth(&self) -> Option<Frequency> { Some(self.bandwidth) }
    fn self_resonant_frequency(&self) -> Option<Frequency> { None }
    fn impedance_at_frequency(&self, _frequency: Frequency) -> ElectricalResistance {
        ElectricalResistance::new::<ohm>(1e12) // Very high input impedance
    }
}

impl MaterialProperties for OpAmp {
    fn thermal_conductivity(&self) -> f32 { 148.0 } // Silicon die
    fn electrical_resistivity(&self) -> f32 { 1e5 }
    fn youngs_modulus(&self) -> f32 { 130e9 }
    fn poisson_ratio(&self) -> f32 { 0.27 }
    fn density(&self) -> f32 { 2329.0 }
    fn specific_heat(&self) -> f32 { 712.0 }
}

// ============================================================================
// CONVENIENCE CONSTRUCTORS
// ============================================================================

impl Resistor {
    pub fn new(resistance_ohms: f64, power_watts: f64, tolerance: f64) -> Self {
        Self {
            resistance: ElectricalResistance::new::<ohm>(resistance_ohms),
            power_rating: Power::new::<watt>(power_watts),
            tolerance,
            ..Default::default()
        }
    }
}

impl Capacitor {
    pub fn new(capacitance_farads: f64, voltage_rating: f64, tolerance: f64) -> Self {
        Self {
            capacitance: Capacitance::new::<farad>(capacitance_farads),
            voltage_rating: ElectricPotential::new::<volt>(voltage_rating),
            tolerance,
            ..Default::default()
        }
    }
}

impl Inductor {
    pub fn new(inductance_henries: f64, current_rating: f64) -> Self {
        Self {
            inductance: Inductance::new::<henry>(inductance_henries),
            current_rating: ElectricCurrent::new::<ampere>(current_rating),
            ..Default::default()
        }
    }
}

impl OpAmp {
    pub fn new(bandwidth_hz: f64, supply_voltage: f64) -> Self {
        Self {
            bandwidth: Frequency::new::<hertz>(bandwidth_hz),
            supply_voltage_max: ElectricPotential::new::<volt>(supply_voltage),
            ..Default::default()
        }
    }
}

// ============================================================================
// UNIFIED COMPONENT ENUM FOR WFC SOLVER
// ============================================================================

/// Unified enum containing all analog circuit components for use with WFC solver
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AnalogComponent {
    Resistor(Resistor),
    Capacitor(Capacitor),
    Inductor(Inductor),
    Diode(Diode),
    OpAmp(OpAmp),
}

impl Default for AnalogComponent {
    fn default() -> Self {
        Self::Resistor(Resistor::default())
    }
}

impl Block3DLike for AnalogComponent {
    fn size(&self) -> (u32, u32, u32) {
        match self {
            AnalogComponent::Resistor(r) => r.size(),
            AnalogComponent::Capacitor(c) => c.size(),
            AnalogComponent::Inductor(l) => l.size(),
            AnalogComponent::Diode(d) => d.size(),
            AnalogComponent::OpAmp(o) => o.size(),
        }
    }
    
    fn faces(&self) -> impl Iterator<Item = Face> {
        match self {
            AnalogComponent::Resistor(r) => r.faces().collect::<Vec<_>>().into_iter(),
            AnalogComponent::Capacitor(c) => c.faces().collect::<Vec<_>>().into_iter(),
            AnalogComponent::Inductor(l) => l.faces().collect::<Vec<_>>().into_iter(),
            AnalogComponent::Diode(d) => d.faces().collect::<Vec<_>>().into_iter(),
            AnalogComponent::OpAmp(o) => o.faces().collect::<Vec<_>>().into_iter(),
        }
    }
    
    fn symbol(&self) -> String {
        match self {
            AnalogComponent::Resistor(r) => r.symbol(),
            AnalogComponent::Capacitor(c) => c.symbol(),
            AnalogComponent::Inductor(l) => l.symbol(),
            AnalogComponent::Diode(d) => d.symbol(),
            AnalogComponent::OpAmp(o) => o.symbol(),
        }
    }
    
    fn ranking(&self) -> f32 {
        match self {
            AnalogComponent::Resistor(_) => 1.0,
            AnalogComponent::Capacitor(_) => 1.2,
            AnalogComponent::Inductor(_) => 1.1,
            AnalogComponent::Diode(_) => 0.8,
            AnalogComponent::OpAmp(_) => 0.5, // Lower ranking for complex ICs
        }
    }
}

impl MaterialProperties for AnalogComponent {
    fn thermal_conductivity(&self) -> f32 {
        match self {
            AnalogComponent::Resistor(r) => r.thermal_conductivity(),
            AnalogComponent::Capacitor(c) => c.thermal_conductivity(),
            AnalogComponent::Inductor(l) => l.thermal_conductivity(),
            AnalogComponent::Diode(d) => d.thermal_conductivity(),
            AnalogComponent::OpAmp(o) => o.thermal_conductivity(),
        }
    }
    
    fn electrical_resistivity(&self) -> f32 {
        match self {
            AnalogComponent::Resistor(r) => r.electrical_resistivity(),
            AnalogComponent::Capacitor(c) => c.electrical_resistivity(),
            AnalogComponent::Inductor(l) => l.electrical_resistivity(),
            AnalogComponent::Diode(d) => d.electrical_resistivity(),
            AnalogComponent::OpAmp(o) => o.electrical_resistivity(),
        }
    }
    
    fn youngs_modulus(&self) -> f32 {
        match self {
            AnalogComponent::Resistor(r) => r.youngs_modulus(),
            AnalogComponent::Capacitor(c) => c.youngs_modulus(),
            AnalogComponent::Inductor(l) => l.youngs_modulus(),
            AnalogComponent::Diode(d) => d.youngs_modulus(),
            AnalogComponent::OpAmp(o) => o.youngs_modulus(),
        }
    }
    
    fn poisson_ratio(&self) -> f32 {
        match self {
            AnalogComponent::Resistor(r) => r.poisson_ratio(),
            AnalogComponent::Capacitor(c) => c.poisson_ratio(),
            AnalogComponent::Inductor(l) => l.poisson_ratio(),
            AnalogComponent::Diode(d) => d.poisson_ratio(),
            AnalogComponent::OpAmp(o) => o.poisson_ratio(),
        }
    }
    
    fn density(&self) -> f32 {
        match self {
            AnalogComponent::Resistor(r) => r.density(),
            AnalogComponent::Capacitor(c) => c.density(),
            AnalogComponent::Inductor(l) => l.density(),
            AnalogComponent::Diode(d) => d.density(),
            AnalogComponent::OpAmp(o) => o.density(),
        }
    }
    
    fn specific_heat(&self) -> f32 {
        match self {
            AnalogComponent::Resistor(r) => r.specific_heat(),
            AnalogComponent::Capacitor(c) => c.specific_heat(),
            AnalogComponent::Inductor(l) => l.specific_heat(),
            AnalogComponent::Diode(d) => d.specific_heat(),
            AnalogComponent::OpAmp(o) => o.specific_heat(),
        }
    }
}

// Convenience constructors for the enum
impl AnalogComponent {
    pub fn resistor(resistance_ohms: f64, power_watts: f64, tolerance: f64) -> Self {
        Self::Resistor(Resistor::new(resistance_ohms, power_watts, tolerance))
    }
    
    pub fn capacitor(capacitance_farads: f64, voltage_rating: f64, tolerance: f64) -> Self {
        Self::Capacitor(Capacitor::new(capacitance_farads, voltage_rating, tolerance))
    }
    
    pub fn inductor(inductance_henries: f64, current_rating: f64) -> Self {
        Self::Inductor(Inductor::new(inductance_henries, current_rating))
    }
    
    pub fn op_amp(bandwidth_hz: f64, supply_voltage: f64) -> Self {
        Self::OpAmp(OpAmp::new(bandwidth_hz, supply_voltage))
    }
    
    pub fn diode() -> Self {
        Self::Diode(Diode::default())
    }
}

// Manual Hash implementations for all component structs, ignoring floating point fields
impl std::hash::Hash for Capacitor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.size.hash(state);
        self.faces.hash(state);
        self.dielectric.hash(state);
        self.package.hash(state);
    }
}

impl std::hash::Hash for Inductor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.size.hash(state);
        self.faces.hash(state);
        self.core_material.hash(state);
        self.package.hash(state);
    }
}

impl std::hash::Hash for Diode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.size.hash(state);
        self.faces.hash(state);
        self.package.hash(state);
    }
}

impl std::hash::Hash for OpAmp {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.size.hash(state);
        self.faces.hash(state);
        self.package.hash(state);
    }
}

impl std::hash::Hash for AnalogComponent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            AnalogComponent::Resistor(r) => r.hash(state),
            AnalogComponent::Capacitor(c) => c.hash(state),
            AnalogComponent::Inductor(l) => l.hash(state),
            AnalogComponent::Diode(d) => d.hash(state),
            AnalogComponent::OpAmp(o) => o.hash(state),
        }
    }
} 