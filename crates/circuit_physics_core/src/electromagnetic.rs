use serde::{Serialize, Deserialize};
// use bevy::prelude::*;
// use uom::si::f64::*;
use uom::si::{
    acceleration::meter_per_second_squared, electric_current::ampere, electric_potential::volt, electrical_resistance::ohm, f32::{Acceleration, ElectricCurrent, ElectricField, ElectricalResistance, Frequency, MagneticFluxDensity, Power, ThermodynamicTemperature, Time}, frequency::hertz, luminous_intensity::candela, magnetic_flux_density::tesla, power::watt, pressure::pascal, thermodynamic_temperature::kelvin, time::second
};



// ============================================================================
// ELECTROMAGNETIC COMPATIBILITY (EMC) TRAITS
// ============================================================================

/// Components that generate or are susceptible to electromagnetic interference
pub trait ElectromagneticCompatibility {
    /// Electromagnetic emission level (dBµV/m at specified distance and frequency)
    fn emission_level(&self, frequency: Frequency) -> f32;
    
    /// Electromagnetic susceptibility threshold (V/m)
    fn susceptibility_threshold(&self, frequency: Frequency) -> ElectricField;
    
    /// Shielding effectiveness (dB)
    fn shielding_effectiveness(&self) -> f32;
    
    /// Check if component meets EMC requirements
    fn meets_emc_limits(&self, frequency: Frequency, limit_dbmv: f32) -> bool {
        self.emission_level(frequency) <= limit_dbmv
    }
}

/// Components that exhibit magnetic field sensitivity
pub trait MagneticFieldSensitive {
    /// Maximum magnetic field the component can tolerate (Tesla)
    fn max_magnetic_field(&self) -> MagneticFluxDensity;
    
    /// Magnetic field sensitivity coefficient
    fn magnetic_sensitivity(&self) -> f32; // % change per Tesla
    
    /// Check if component is safe in given magnetic field
    fn is_magnetically_safe(&self, field: MagneticFluxDensity) -> bool {
        field <= self.max_magnetic_field()
    }
}
