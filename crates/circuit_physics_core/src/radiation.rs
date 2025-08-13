use serde::{Serialize, Deserialize};
// use bevy::prelude::*;
// use uom::si::f64::*;
use uom::si::{
    acceleration::meter_per_second_squared, electric_current::ampere, electric_potential::volt, electrical_resistance::ohm, f32::{Acceleration, ElectricCurrent, ElectricField, ElectricalResistance, Frequency, MagneticFluxDensity, Power, ThermodynamicTemperature, Time}, frequency::hertz, luminous_intensity::candela, magnetic_flux_density::tesla, power::watt, pressure::pascal, thermodynamic_temperature::kelvin, time::second
};






// ============================================================================
// RADIATION EFFECTS TRAITS
// ============================================================================

/// Components susceptible to radiation effects
pub trait RadiationHardness {
    /// Total ionizing dose tolerance (rad or Gy)
    fn total_dose_tolerance(&self) -> f32;
    
    /// Single event upset cross-section (cm²)
    fn seu_cross_section(&self) -> f32;
    
    /// Displacement damage threshold (MeV·cm²/g)
    fn displacement_damage_threshold(&self) -> f32;
    
    /// Check if component can survive radiation environment
    fn radiation_survivability(&self, total_dose: f32, particle_flux: f32) -> bool {
        total_dose <= self.total_dose_tolerance()
    }
}