use serde::{Serialize, Deserialize};
// use bevy::prelude::*;
// use uom::si::f64::*;
use uom::si::{
    acceleration::meter_per_second_squared, electric_current::ampere, electric_potential::volt, electrical_resistance::ohm, f32::{Acceleration, ElectricCurrent, ElectricField, ElectricalResistance, Frequency, MagneticFluxDensity, Power, ThermodynamicTemperature, Time}, frequency::hertz, luminous_intensity::candela, magnetic_flux_density::tesla, power::watt, pressure::pascal, thermodynamic_temperature::kelvin, time::second
};





// ============================================================================
// OPTICAL AND PHOTONIC TRAITS
// ============================================================================

/// Components that interact with light (photodiodes, LEDs, optocouplers)
pub trait OpticalProperties {
    /// Spectral response range (nm)
    fn spectral_range(&self) -> (f32, f32);
    
    /// Peak sensitivity wavelength (nm)
    fn peak_wavelength(&self) -> f32;
    
    /// Optical power handling capability (W)
    fn max_optical_power(&self) -> Power;
    
    /// Dark current at specified temperature
    fn dark_current(&self, temperature: ThermodynamicTemperature) -> ElectricCurrent;
}
