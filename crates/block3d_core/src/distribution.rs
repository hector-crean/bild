use rand::distributions::{Distribution, Standard, Uniform};
use rand::Rng;
use crate::block::{Block3DLike, analog_components::*};
use crate::face::Face;
use uom::si::f64::*;
use uom::si::{
    electric_potential::volt,
    electrical_resistance::ohm,
    capacitance::farad,
    inductance::henry,
    thermodynamic_temperature::kelvin,
    power::watt,
    electric_current::ampere,
    frequency::hertz,
};

/// A distribution that generates random blocks implementing Block3DLike
pub struct BlockDistribution {
    /// Weights for different block types (Resistor, Capacitor, Inductor, Diode, OpAmp)
    pub weights: [f32; 5],
    /// Size range for generated blocks (min, max) for each dimension
    pub size_range: ((u32, u32), (u32, u32), (u32, u32)),
}

impl Default for BlockDistribution {
    fn default() -> Self {
        Self {
            // Equal probability for all component types
            weights: [1.0, 1.0, 1.0, 1.0, 1.0],
            // Default size range: 1-3 units in each dimension
            size_range: ((1, 3), (1, 3), (1, 3)),
        }
    }
}

impl BlockDistribution {
    /// Create a new distribution with custom weights
    pub fn with_weights(weights: [f32; 5]) -> Self {
        Self {
            weights,
            ..Default::default()
        }
    }
    
    /// Create a distribution favoring passive components (R, L, C)
    pub fn passive_heavy() -> Self {
        Self::with_weights([3.0, 3.0, 3.0, 1.0, 0.5]) // Favor R, L, C
    }
    
    /// Create a distribution favoring active components
    pub fn active_heavy() -> Self {
        Self::with_weights([1.0, 1.0, 1.0, 2.0, 3.0]) // Favor diodes and op-amps
    }
    
    /// Set the size range for generated blocks
    pub fn with_size_range(mut self, size_range: ((u32, u32), (u32, u32), (u32, u32))) -> Self {
        self.size_range = size_range;
        self
    }
    
    /// Generate a random size within the configured range
    fn random_size<R: Rng + ?Sized>(&self, rng: &mut R) -> (u32, u32, u32) {
        let width = rng.gen_range(self.size_range.0.0..=self.size_range.0.1);
        let height = rng.gen_range(self.size_range.1.0..=self.size_range.1.1);
        let depth = rng.gen_range(self.size_range.2.0..=self.size_range.2.1);
        (width, height, depth)
    }
}





impl Distribution<AnalogComponent> for BlockDistribution {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> AnalogComponent {
        // Weighted selection of block type
        let total_weight: f32 = self.weights.iter().sum();
        let mut threshold = rng.gen_range(0.0..1.0) * total_weight;
        
        let size = self.random_size(rng);
        
        for (i, &weight) in self.weights.iter().enumerate() {
            threshold -= weight;
            if threshold <= 0.0 {
                return match i {
                    0 => AnalogComponent::Resistor(random_resistor(rng, size)),
                    1 => AnalogComponent::Capacitor(random_capacitor(rng, size)),
                    2 => AnalogComponent::Inductor(random_inductor(rng, size)),
                    3 => AnalogComponent::Diode(random_diode(rng, size)),
                    // 4 => AnalogComponent::OpAmp(random_opamp(rng, size)),
                    _ => unreachable!(),
                };
            }
        }
        
        // Fallback (shouldn't happen)
        AnalogComponent::Resistor(random_resistor(rng, size))
    }
}

/// Generate a random resistor with realistic values
fn random_resistor<R: Rng + ?Sized>(rng: &mut R, size: (u32, u32, u32)) -> Resistor {
    // E24 series resistor values (common standard values)
    let e24_values = [
        1.0, 1.1, 1.2, 1.3, 1.5, 1.6, 1.8, 2.0, 2.2, 2.4, 2.7, 3.0,
        3.3, 3.6, 3.9, 4.3, 4.7, 5.1, 5.6, 6.2, 6.8, 7.5, 8.2, 9.1
    ];
    
    let base_value = e24_values[rng.gen_range(0..e24_values.len())];
    let multiplier = 10.0_f64.powi(rng.gen_range(0..7)); // 1Ω to 1MΩ
    let resistance = base_value * multiplier;
    
    let power_rating = match rng.gen_range(0..4) {
        0 => Power::new::<watt>(0.125),
        1 => Power::new::<watt>(0.25),
        2 => Power::new::<watt>(0.5),
        _ => Power::new::<watt>(1.0),
    };
    
    Resistor {
        size,
        faces: Vec::new(),
        resistance: ElectricalResistance::new::<ohm>(resistance),
        power_rating,
        tolerance: rng.gen_range(1.0..20.0), // 1% to 20% tolerance
        temperature_coefficient: rng.gen_range(-200.0..200.0), // ppm/°C
        package: if rng.gen_bool(0.7) { PackageType::SurfaceMount } else { PackageType::ThroughHole },
        operating_temperature: ThermodynamicTemperature::new::<kelvin>(rng.gen_range(273.0..398.0)),
    }
}

/// Generate a random capacitor with realistic values
fn random_capacitor<R: Rng + ?Sized>(rng: &mut R, size: (u32, u32, u32)) -> Capacitor {
    // Common capacitor values
    let base_values = [1.0, 2.2, 4.7, 10.0, 22.0, 47.0, 100.0, 220.0, 470.0];
    let base_value = base_values[rng.gen_range(0..base_values.len())];
    let multiplier = 10.0_f64.powi(rng.gen_range(-12..-3)); // pF to mF range
    let capacitance = base_value * multiplier;
    
    let voltage_rating = match rng.gen_range(0..6) {
        0 => ElectricPotential::new::<volt>(6.3),
        1 => ElectricPotential::new::<volt>(10.0),
        2 => ElectricPotential::new::<volt>(16.0),
        3 => ElectricPotential::new::<volt>(25.0),
        4 => ElectricPotential::new::<volt>(50.0),
        _ => ElectricPotential::new::<volt>(100.0),
    };
    
    Capacitor {
        size,
        faces: Vec::new(),
        capacitance: Capacitance::new::<farad>(capacitance),
        voltage_rating,
        tolerance: rng.gen_range(5.0..20.0),
        dielectric: match rng.gen_range(0..4) {
            0 => DielectricType::Ceramic,
            1 => DielectricType::Electrolytic,
            2 => DielectricType::Film,
            _ => DielectricType::Tantalum,
        },
        esr: ElectricalResistance::new::<ohm>(rng.gen_range(0.001..1.0)),
        package: if rng.gen_bool(0.6) { PackageType::SurfaceMount } else { PackageType::ThroughHole },
        operating_temperature: ThermodynamicTemperature::new::<kelvin>(rng.gen_range(273.0..398.0)),
    }
}

/// Generate a random inductor with realistic values
fn random_inductor<R: Rng + ?Sized>(rng: &mut R, size: (u32, u32, u32)) -> Inductor {
    let base_values = [1.0, 2.2, 4.7, 10.0, 22.0, 47.0, 100.0, 220.0, 470.0];
    let base_value = base_values[rng.gen_range(0..base_values.len())];
    let multiplier = 10.0_f64.powi(rng.gen_range(-9..-3)); // nH to mH range
    let inductance = base_value * multiplier;
    
    Inductor {
        size,
        faces: Vec::new(),
        inductance: Inductance::new::<henry>(inductance),
        current_rating: ElectricCurrent::new::<ampere>(rng.gen_range(0.1..10.0)),
        dc_resistance: ElectricalResistance::new::<ohm>(rng.gen_range(0.01..10.0)),
        core_material: match rng.gen_range(0..4) {
            0 => CoreMaterial::Air,
            1 => CoreMaterial::Ferrite,
            2 => CoreMaterial::Iron,
            _ => CoreMaterial::Laminated,
        },
        package: if rng.gen_bool(0.5) { PackageType::SurfaceMount } else { PackageType::ThroughHole },
        operating_temperature: ThermodynamicTemperature::new::<kelvin>(rng.gen_range(273.0..398.0)),
    }
}

/// Generate a random diode
fn random_diode<R: Rng + ?Sized>(rng: &mut R, size: (u32, u32, u32)) -> Diode {
    Diode {
        size,
        faces: Vec::new(),
        forward_voltage: ElectricPotential::new::<volt>(rng.gen_range(0.3..3.3)),
        reverse_breakdown_voltage: ElectricPotential::new::<volt>(rng.gen_range(5.0..1000.0)),
        forward_current_rating: ElectricCurrent::new::<ampere>(rng.gen_range(0.001..10.0)),
        reverse_recovery_time: rng.gen_range(1.0..100.0), // ns
        junction_capacitance: Capacitance::new::<farad>(rng.gen_range(1e-12..100e-12)), // pF range
        package: if rng.gen_bool(0.7) { PackageType::SurfaceMount } else { PackageType::ThroughHole },
        operating_temperature: ThermodynamicTemperature::new::<kelvin>(rng.gen_range(273.0..398.0)),
    }
}

/// Generate a random op-amp
// fn random_opamp<R: Rng>(rng: &mut R, size: (u32, u32, u32)) -> OpAmp {
//     OpAmp {
//         size,
//         faces: Vec::new(),
//         supply_voltage: ElectricPotential::new::<volt>(rng.gen_range(3.0..15.0)),
//         input_offset_voltage: ElectricPotential::new::<volt>(rng.gen_range(0.001..0.01)),
//         gain_bandwidth_product: Frequency::new::<hertz>(rng.gen_range(1e6..100e6)),
//         slew_rate: rng.gen_range(0.1..100.0), // V/μs
//         package: PackageType::SurfaceMount, // Op-amps are typically SMT
//         operating_temperature: ThermodynamicTemperature::new::<kelvin>(rng.gen_range(273.0..398.0)),
//     }
// }

/// Convenience function to generate a vector of random blocks
pub fn generate_random_blocks<R: Rng + ?Sized>(
    rng: &mut R, 
    count: usize, 
    distribution: &BlockDistribution
) -> Vec<AnalogComponent> {
    (0..count)
        .map(|_| distribution.sample(rng))
        .collect()
}

/// Generate blocks with a specific distribution pattern
pub fn generate_circuit_blocks<R: Rng + ?Sized>(rng: &mut R, count: usize) -> Vec<AnalogComponent> {
    let distribution = BlockDistribution::passive_heavy()
        .with_size_range(((1, 2), (1, 2), (1, 1))); // Smaller components
    generate_random_blocks(rng, count, &distribution)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_block_distribution() {
        let mut rng = thread_rng();
        let distribution = BlockDistribution::default();
        
        let blocks = generate_random_blocks(&mut rng, 10, &distribution);
        assert_eq!(blocks.len(), 10);
        
        // Test that all blocks implement Block3DLike
        for block in blocks {
            assert!(block.size().0 > 0);
            assert!(block.size().1 > 0);
            assert!(block.size().2 > 0);
            assert!(!block.symbol().is_empty());
        }
    }
    
    #[test]
    fn test_passive_heavy_distribution() {
        let mut rng = thread_rng();
        let blocks = generate_circuit_blocks(&mut rng, 20);
        
        let passive_count = blocks.iter()
            .filter(|b| matches!(b, AnalogComponent::Resistor(_) | AnalogComponent::Capacitor(_) | AnalogComponent::Inductor(_)))
            .count();
            
        // Should have more passive than active components
        assert!(passive_count > blocks.len() / 2);
    }
} 