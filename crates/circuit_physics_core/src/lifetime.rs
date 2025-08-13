use serde::{Serialize, Deserialize};
// use bevy::prelude::*;
// use uom::si::f64::*;
use uom::si::{
    acceleration::meter_per_second_squared, electric_current::ampere, electric_potential::volt, electrical_resistance::ohm, f32::{Acceleration, ElectricCurrent, ElectricField, ElectricalResistance, Frequency, MagneticFluxDensity, Power, ThermodynamicTemperature, Time}, frequency::hertz, luminous_intensity::candela, magnetic_flux_density::tesla, power::watt, pressure::pascal, thermodynamic_temperature::kelvin, time::second
};




// ============================================================================
// AGING AND RELIABILITY TRAITS
// ============================================================================

/// Components that exhibit aging mechanisms
pub trait AgingMechanisms {
    /// Activation energy for primary aging mechanism (eV)
    fn activation_energy(&self) -> f32;
    
    /// Stress acceleration factor
    fn stress_acceleration_factor(&self, stress_level: f32, reference_level: f32) -> f32;
    
    /// Time to failure based on Arrhenius model
    fn time_to_failure_arrhenius(&self, temperature: ThermodynamicTemperature, stress: f32) -> Time {
        let k_b = 8.617e-5; // Boltzmann constant in eV/K
        let temp_k = temperature.get::<kelvin>();
        let reference_temp = 298.15; // 25°C
        
        let acceleration = ((self.activation_energy() / k_b) * 
                          (1.0/reference_temp - 1.0/temp_k)).exp();
        let stress_factor = self.stress_acceleration_factor(stress, 1.0) ;
        
        // Base lifetime at reference conditions (assume 10 years)
        let base_lifetime: f32 = 10.0 * 365.0 * 24.0 * 3600.0; // seconds
        
        Time::new::<second>(base_lifetime / (acceleration * stress_factor))
    }
}

/// Components with wear-out mechanisms
pub trait WearOut {
    /// Wear-out distribution parameters (Weibull shape and scale)
    fn weibull_parameters(&self) -> (f32, f32); // (beta, eta)
    
    /// Calculate reliability at given time
    fn reliability_at_time(&self, time: Time) -> f32 {
        let (beta, eta) = self.weibull_parameters();
        let t = time.get::<second>() as f32;
        (-((t / eta).powf(beta))).exp()
    }
    
    // Mean time to failure
    // fn mean_time_to_failure(&self) -> Time {
    //     let (beta, eta) = self.weibull_parameters();
    //     let gamma_factor = gamma::gamma(1.0 + 1.0/beta);
    //     Time::new::<second>((eta * gamma_factor))
    // }
}
