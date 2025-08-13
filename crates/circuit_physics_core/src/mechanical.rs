use serde::{Serialize, Deserialize};
// use bevy::prelude::*;
// use uom::si::f64::*;
use uom::si::{
    acceleration::meter_per_second_squared, electric_current::ampere, electric_potential::volt, electrical_resistance::ohm, f32::{Acceleration, ElectricCurrent, ElectricField, ElectricalResistance, Frequency, MagneticFluxDensity, Power, ThermodynamicTemperature, Time}, frequency::hertz, luminous_intensity::candela, magnetic_flux_density::tesla, power::watt, pressure::pascal, thermodynamic_temperature::kelvin, time::second
};


// ============================================================================
// MECHANICAL STRESS AND VIBRATION TRAITS
// ============================================================================

/// Components that can experience mechanical stress and vibration
pub trait MechanicalStress {
    /// Maximum acceleration the component can withstand (g's)
    fn max_acceleration(&self) -> Acceleration;
    
    /// Resonant frequency for vibration analysis (Hz)
    fn resonant_frequency(&self) -> Option<Frequency>;
    
    /// Mechanical quality factor (Q-factor)
    fn mechanical_q_factor(&self) -> f32;
    
    /// Shock resistance (g's for specified duration)
    fn shock_resistance(&self) -> (Acceleration, Time);
    
    /// Check if component can survive given acceleration
    fn can_survive_acceleration(&self, acceleration: Acceleration) -> bool {
        acceleration <= self.max_acceleration()
    }
}

/// Components susceptible to solder joint fatigue and mechanical failure
pub trait SolderJointReliability {
    /// Coefficient of thermal expansion mismatch (ppm/°C)
    fn cte_mismatch(&self) -> f32;
    
    /// Solder joint geometry factor
    fn joint_geometry_factor(&self) -> f32;
    
    /// Predicted cycles to failure for thermal cycling
    fn thermal_cycles_to_failure(&self, temp_range: f32, cycle_time: Time) -> u32 {
        // Simplified Coffin-Manson model
        let delta_t = temp_range;
        let n_f = 1000.0 * (delta_t / 100.0).powf(-2.0) * self.joint_geometry_factor();
        n_f as u32
    }
}