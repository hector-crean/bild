use serde::{Serialize, Deserialize};
// use bevy::prelude::*;
// use uom::si::f64::*;
use uom::si::{
    acceleration::meter_per_second_squared, electric_current::ampere, electric_potential::volt, electrical_resistance::ohm, f32::{Acceleration, ElectricCurrent, ElectricField, ElectricalResistance, Frequency, MagneticFluxDensity, Power, ThermodynamicTemperature, Time}, frequency::hertz, luminous_intensity::candela, magnetic_flux_density::tesla, power::watt, pressure::pascal, thermodynamic_temperature::kelvin, time::second
};

// ============================================================================
// THERMAL SIMULATION TRAITS
// ============================================================================

/// Components that exhibit thermal behavior and can be thermally simulated
pub trait ThermalBehavior {
    /// Thermal resistance from junction to case (K/W)
    fn thermal_resistance_jc(&self) -> f32;
    
    /// Thermal resistance from case to ambient (K/W)
    fn thermal_resistance_ca(&self) -> f32;
    
    /// Thermal capacitance for transient analysis (J/K)
    fn thermal_capacitance(&self) -> f32;
    
    /// Maximum junction temperature (K)
    fn max_junction_temperature(&self) -> ThermodynamicTemperature;
    
    /// Current junction temperature based on power dissipation
    fn junction_temperature(&self, power: Power, ambient_temp: ThermodynamicTemperature) -> ThermodynamicTemperature {
        let temp_rise = power.get::<watt>() * (self.thermal_resistance_jc() + self.thermal_resistance_ca());
        ThermodynamicTemperature::new::<kelvin>(ambient_temp.get::<kelvin>() + temp_rise)
    }
    
    /// Check if component is within thermal limits
    fn is_thermally_safe(&self, power: Power, ambient_temp: ThermodynamicTemperature) -> bool {
        self.junction_temperature(power, ambient_temp) <= self.max_junction_temperature()
    }
}

/// Components that have temperature-dependent electrical characteristics
pub trait TemperatureDependent {
    /// Temperature coefficient of primary parameter (ppm/°C or %/°C)
    fn temperature_coefficient(&self) -> f32;
    
    /// Reference temperature for specifications (typically 25°C)
    fn reference_temperature(&self) -> ThermodynamicTemperature {
        ThermodynamicTemperature::new::<kelvin>(298.15)
    }
    
    /// Calculate parameter drift due to temperature
    fn parameter_drift(&self, current_temp: ThermodynamicTemperature) -> f32 {
        let temp_diff = current_temp.get::<kelvin>() - self.reference_temperature().get::<kelvin>();
        self.temperature_coefficient() * temp_diff as f32 / 1e6 // Convert ppm to fraction
    }
}
