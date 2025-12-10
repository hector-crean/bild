// use bevy::prelude::*;
// use uom::si::f64::*;
use uom::si::f32::{ElectricCurrent, Power, ThermodynamicTemperature};





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
