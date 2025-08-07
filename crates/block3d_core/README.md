# Block3D Core - Analog Circuit Design Framework

A comprehensive Rust framework for analog circuit design and physical simulation, built with trait-based composition and Bevy ECS compatibility.

## Overview

This framework provides a modular, trait-based approach to modeling analog circuit components for both grid-based layout and comprehensive physical simulations. Each component is represented by its own struct with relevant physical properties exposed through specialized traits.

## Architecture

### Core Traits

#### Grid and Connectivity
- **`Block3DLike`**: Core spatial and connectivity behavior for grid placement
  - Size and position management
  - Face-based connection system
  - Collision detection and placement validation

#### Physical Properties
- **`MaterialProperties`**: Basic material characteristics for thermal/mechanical simulation
- **`Resistive`**: Electrical resistance behavior with temperature coefficients
- **`Capacitive`**: Energy storage in electric fields with dielectric properties
- **`Inductive`**: Energy storage in magnetic fields with core materials
- **`Semiconductor`**: Junction-based devices with forward/breakdown voltages

#### Electrical Behavior
- **`FrequencyDependent`**: Impedance vs frequency, bandwidth, self-resonance
- **`PowerRated`**: Safe operating area, current/voltage/power limits
- **`NoiseGenerating`**: Thermal and flicker noise characteristics

### Advanced Physical Simulation Traits

#### Thermal Analysis
- **`ThermalBehavior`**: Junction-to-ambient thermal resistance, thermal capacitance
- **`TemperatureDependent`**: Parameter drift with temperature changes

#### Mechanical Reliability  
- **`MechanicalStress`**: Acceleration limits, resonant frequencies, shock resistance
- **`SolderJointReliability`**: Thermal cycling fatigue, CTE mismatch effects

#### Electromagnetic Effects
- **`ElectromagneticCompatibility`**: EMI emission/susceptibility, shielding effectiveness
- **`MagneticFieldSensitive`**: Magnetic field tolerance and sensitivity

#### Aging and Reliability
- **`AgingMechanisms`**: Arrhenius aging models, activation energies
- **`WearOut`**: Weibull distribution parameters, MTTF calculations
- **`LifetimePrediction`**: Multi-mechanism failure rate combination

#### Environmental Effects
- **`RadiationHardness`**: Total dose, SEU cross-sections, displacement damage
- **`EnvironmentalDegradation`**: Moisture sensitivity, corrosion, outgassing
- **`OpticalProperties`**: Spectral response, optical power handling (LEDs, photodiodes)

#### Advanced Physics
- **`MultiPhysicsCoupling`**: Electro-thermal-mechanical coupling coefficients
- **`NonlinearBehavior`**: Harmonic distortion, compression points
- **`ProcessVariation`**: Manufacturing variations, Monte Carlo analysis

## Component Types

### Passive Components
```rust
// Individual structs for each component type
let resistor = Resistor::new(1000.0, 0.25, 5.0); // 1kΩ, 0.25W, 5% tolerance
let capacitor = Capacitor::new(1e-9, 50.0, 10.0); // 1nF, 50V, 10% tolerance  
let inductor = Inductor::new(1e-6, 1.0); // 1µH, 1A rating
```

### Semiconductor Components
```rust
let diode = Diode::default(); // Standard silicon diode
let opamp = OpAmp::new(1e6, 15.0); // 1MHz bandwidth, ±15V supply
```

### Key Benefits

1. **Trait Composition**: Mix and match physical properties as needed
2. **Bevy ECS Ready**: All components derive `Component` for direct ECS use
3. **Type Safety**: Strong typing with units via `uom` crate
4. **Extensible**: Easy to add new component types and physical properties
5. **Simulation Ready**: Comprehensive physical models for multi-physics simulation

## Example Usage

### Basic Component Creation and Analysis
```rust
use block3d_core::block::analog_components::*;

// Create components
let resistor = Resistor::new(1000.0, 0.25, 5.0);
let capacitor = Capacitor::new(1e-9, 50.0, 10.0);

// Access electrical properties
let resistance = resistor.resistance();
let capacitance = capacitor.capacitance();

// Frequency analysis
let freq = Frequency::new::<hertz>(1000.0);
let r_impedance = resistance; // Resistor impedance is constant
let c_impedance = capacitor.impedance_at_frequency(freq);

// Thermal analysis  
let temp = ThermodynamicTemperature::new::<kelvin>(298.15);
let noise = resistor.thermal_noise_density(temp);

// Grid placement
let positions = resistor.occupied_positions((0, 0, 0));
assert_eq!(positions, vec![(0, 0, 0)]); // Single grid unit
```

### Multi-Physics Simulation
```rust
use block3d_core::block::physical_simulation_traits::*;

// Implement advanced traits for specific components
impl ThermalBehavior for Resistor {
    fn thermal_resistance_jc(&self) -> f32 { 100.0 } // K/W
    fn thermal_resistance_ca(&self) -> f32 { 200.0 } // K/W  
    fn thermal_capacitance(&self) -> f32 { 0.001 } // J/K
    fn max_junction_temperature(&self) -> ThermodynamicTemperature {
        ThermodynamicTemperature::new::<kelvin>(423.15) // 150°C
    }
}

// Perform thermal analysis
let power = Power::new::<watt>(0.1);
let ambient = ThermodynamicTemperature::new::<kelvin>(298.15);
let junction_temp = resistor.junction_temperature(power, ambient);
let is_safe = resistor.is_thermally_safe(power, ambient);
```

### Bevy ECS Integration
```rust
use bevy::prelude::*;

fn setup_circuit(mut commands: Commands) {
    // Spawn components as ECS entities
    commands.spawn(Resistor::new(1000.0, 0.25, 5.0));
    commands.spawn(Capacitor::new(1e-9, 50.0, 10.0));
    commands.spawn(OpAmp::new(1e6, 15.0));
}

fn thermal_simulation_system(
    query: Query<&Resistor, With<ThermalBehavior>>,
) {
    for resistor in query.iter() {
        // Perform thermal simulation
        let power = Power::new::<watt>(0.1);
        let ambient = ThermodynamicTemperature::new::<kelvin>(298.15);
        if !resistor.is_thermally_safe(power, ambient) {
            warn!("Resistor overheating!");
        }
    }
}
```

## Physical Simulation Capabilities

The framework supports comprehensive multi-physics simulations including:

- **Thermal**: Junction temperatures, thermal resistance networks, transient thermal analysis
- **Mechanical**: Vibration analysis, shock resistance, solder joint fatigue
- **Electrical**: AC/DC analysis, noise simulation, nonlinear behavior
- **Reliability**: Arrhenius aging, Weibull wear-out, multi-mechanism failure rates
- **Environmental**: Radiation effects, humidity/corrosion, chemical compatibility
- **Manufacturing**: Process variations, Monte Carlo analysis, corner case modeling

This makes it suitable for:
- Aerospace and defense applications
- Automotive electronics  
- Medical device design
- High-reliability industrial systems
- Academic research and education

## Future Extensions

The trait-based architecture makes it easy to add:
- New component types (MOSFETs, BJTs, transformers, etc.)
- Additional physical effects (piezoelectric, magnetostrictive, etc.)
- Advanced simulation algorithms (SPICE integration, FEA coupling, etc.)
- Manufacturing-specific models (wire bonding, die attach, etc.)

The framework provides a solid foundation for comprehensive analog circuit design and simulation while maintaining the flexibility needed for diverse applications. 