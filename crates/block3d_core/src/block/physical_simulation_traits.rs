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
    
    /// Mean time to failure
    fn mean_time_to_failure(&self) -> Time {
        let (beta, eta) = self.weibull_parameters();
        let gamma_factor = gamma::gamma(1.0 + 1.0/beta);
        Time::new::<second>((eta * gamma_factor))
    }
}

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

// ============================================================================
// MULTI-PHYSICS COUPLING TRAITS
// ============================================================================

/// Components that exhibit coupled electro-thermal-mechanical behavior
pub trait MultiPhysicsCoupling {
    /// Piezoresistive coefficient (change in resistance per unit strain)
    fn piezoresistive_coefficient(&self) -> f32;
    
    /// Thermoelectric coefficient (Seebeck coefficient in V/K)
    fn thermoelectric_coefficient(&self) -> f32;
    
    /// Electrostrictive coefficient (strain per unit electric field squared)
    fn electrostrictive_coefficient(&self) -> f32;
    
    /// Calculate resistance change due to mechanical strain
    fn resistance_change_strain(&self, strain: f32, base_resistance: ElectricalResistance) -> ElectricalResistance {
        let delta_r = self.piezoresistive_coefficient() * strain * base_resistance.get::<ohm>() as f32;
        ElectricalResistance::new::<ohm>(base_resistance.get::<ohm>() + delta_r)
    }
}

// ============================================================================
// NONLINEAR BEHAVIOR TRAITS
// ============================================================================

/// Components with nonlinear electrical characteristics
pub trait NonlinearBehavior {
    /// Harmonic distortion coefficients
    fn harmonic_distortion_coefficients(&self) -> Vec<f32>;
    
    /// Intermodulation distortion characteristics
    fn intermodulation_distortion(&self, f1: Frequency, f2: Frequency) -> f32;
    
    /// Compression point (1dB compression)
    fn compression_point(&self) -> Power;
    
    /// Third-order intercept point
    fn third_order_intercept(&self) -> Power;
}

// ============================================================================
// PROCESS VARIATION TRAITS
// ============================================================================

/// Components with manufacturing process variations
pub trait ProcessVariation {
    /// Statistical distribution of primary parameter
    fn parameter_distribution(&self) -> ParameterDistribution;
    
    /// Process corners (SS, TT, FF, etc.)
    fn process_corners(&self) -> Vec<ProcessCorner>;
    
    /// Monte Carlo parameter generation
    fn generate_monte_carlo_parameters(&self, n_samples: usize) -> Vec<f32>;
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ParameterDistribution {
    pub distribution_type: DistributionType,
    pub mean: f32,
    pub std_dev: f32,
    pub min_value: f32,
    pub max_value: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum DistributionType {
    Normal,
    LogNormal,
    Uniform,
    Weibull,
    Exponential,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum ProcessCorner {
    SlowSlow,    // SS - Slow NMOS, Slow PMOS
    TypicalTypical, // TT - Typical
    FastFast,    // FF - Fast NMOS, Fast PMOS
    SlowFast,    // SF - Slow NMOS, Fast PMOS
    FastSlow,    // FS - Fast NMOS, Slow PMOS
}

// ============================================================================
// LIFETIME PREDICTION TRAITS
// ============================================================================

/// Comprehensive lifetime prediction combining multiple failure mechanisms
pub trait LifetimePrediction {
    /// Combine multiple failure mechanisms using competing risk model
    fn combined_failure_rate(&self, mechanisms: Vec<f32>) -> f32 {
        mechanisms.iter().sum()
    }
    
    /// System-level reliability with redundancy
    fn system_reliability(&self, component_reliabilities: Vec<f32>, redundancy_type: RedundancyType) -> f32 {
        match redundancy_type {
            RedundancyType::Series => component_reliabilities.iter().product(),
            RedundancyType::Parallel => 1.0 - component_reliabilities.iter().map(|r| 1.0 - r).product::<f32>(),
            RedundancyType::KOutOfN(k, _n) => {
                // Simplified k-out-of-n calculation
                // In practice, this would use binomial distribution
                if component_reliabilities.len() >= k {
                    component_reliabilities.iter().take(k).product()
                } else {
                    0.0
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum RedundancyType {
    Series,
    Parallel,
    KOutOfN(usize, usize), // (k, n) - k out of n must work
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Gamma function approximation for Weibull calculations
mod gamma {
    pub fn gamma(x: f32) -> f32 {
        if x == 1.0 {
            1.0
        } else if x == 2.0 {
            1.0
        } else if x == 1.5 {
            0.886227 // Γ(1.5) = √π/2
        } else {
            // Stirling's approximation for other values
            (2.0 * std::f32::consts::PI / x).sqrt() * (x / std::f32::consts::E).powf(x)
        }
    }
} 