use block3d_core::block::{
    Block3DLike, MaterialProperties,
    analog_components::{
        Resistor, Capacitor, Inductor, Diode, OpAmp,
        Resistive, Capacitive, Inductive, FrequencyDependent, 
        PowerRated, NoiseGenerating, Semiconductor,
        DielectricType, CoreMaterial, PackageType,
    }
};
use uom::si::f64::*;
use uom::si::{
    frequency::hertz,
    thermodynamic_temperature::kelvin,
    electric_potential::volt,
    electric_current::ampere,
};

fn main() {
    println!("Analog Circuit Components Example");
    println!("=================================");
    println!("Demonstrating individual component structs with trait-based properties");
    println!();

    // Create individual components using their specific structs
    let resistor_1k = Resistor::new(1000.0, 0.25, 5.0);
    let capacitor_1nf = Capacitor::new(1e-9, 50.0, 10.0);
    let inductor_1uh = Inductor::new(1e-6, 1.0);
    let diode_1n4148 = Diode::default(); // Use default silicon diode
    let op_amp_741 = OpAmp::new(1e6, 15.0);

    // Demonstrate trait-based property access
    println!("=== COMPONENT PROPERTIES ===");
    demonstrate_resistor(&resistor_1k);
    demonstrate_capacitor(&capacitor_1nf);
    demonstrate_inductor(&inductor_1uh);
    demonstrate_diode(&diode_1n4148);
    demonstrate_opamp(&op_amp_741);

    // Demonstrate frequency analysis
    println!("\n=== FREQUENCY ANALYSIS ===");
    frequency_analysis(&resistor_1k, &capacitor_1nf, &inductor_1uh, &op_amp_741);

    // Demonstrate noise analysis
    println!("\n=== NOISE ANALYSIS ===");
    noise_analysis(&resistor_1k);

    // Demonstrate grid functionality
    println!("\n=== GRID FUNCTIONALITY ===");
    grid_functionality_demo(&resistor_1k, &op_amp_741);

    // Demonstrate circuit composition
    println!("\n=== CIRCUIT COMPOSITION ===");
    rc_filter_analysis(&resistor_1k, &capacitor_1nf);

    // Demonstrate Bevy ECS compatibility
    println!("\n=== BEVY ECS COMPATIBILITY ===");
    println!("All components derive `Component` and can be used directly in Bevy ECS:");
    println!("- Resistor: Bevy Component ✓");
    println!("- Capacitor: Bevy Component ✓");  
    println!("- Inductor: Bevy Component ✓");
    println!("- Diode: Bevy Component ✓");
    println!("- OpAmp: Bevy Component ✓");
    println!("\nComponents can be added to entities with: commands.spawn(resistor_1k);");
}

fn demonstrate_resistor(resistor: &Resistor) {
    println!("\n--- 1kΩ Resistor ---");
    println!("Grid size: {:?}", resistor.size());
    println!("Package: {:?}", resistor.package);
    
    // Resistive properties
    println!("Resistance: {:.0} Ω", resistor.resistance().get::<uom::si::electrical_resistance::ohm>());
    println!("Tolerance: ±{:.1}%", resistor.tolerance());
    println!("Temp coefficient: {:.0} ppm/°C", resistor.temperature_coefficient());
    
    // Power properties
    println!("Power rating: {:.2} W", resistor.power_rating().get::<uom::si::power::watt>());
    println!("Max current: {:.3} A", resistor.current_rating().get::<ampere>());
    println!("Max voltage: {:.1} V", resistor.voltage_rating().get::<volt>());
    
    // Material properties
    println!("Thermal conductivity: {:.1} W/m·K", resistor.thermal_conductivity());
    println!("Density: {:.0} kg/m³", resistor.density());
}

fn demonstrate_capacitor(capacitor: &Capacitor) {
    println!("\n--- 1nF Capacitor ---");
    println!("Grid size: {:?}", capacitor.size());
    println!("Package: {:?}", capacitor.package);
    
    // Capacitive properties
    println!("Capacitance: {:.0} nF", capacitor.capacitance().get::<uom::si::capacitance::farad>() * 1e9);
    println!("Voltage rating: {:.0} V", capacitor.voltage_rating().get::<volt>());
    println!("Dielectric: {:?}", capacitor.dielectric_type());
    println!("ESR: {:.3} Ω", capacitor.equivalent_series_resistance().get::<uom::si::electrical_resistance::ohm>());
    
    // Frequency properties
    if let Some(srf) = capacitor.self_resonant_frequency() {
        println!("Self-resonant freq: {:.2e} Hz", srf.get::<hertz>());
    }
}

fn demonstrate_inductor(inductor: &Inductor) {
    println!("\n--- 1µH Inductor ---");
    println!("Grid size: {:?}", inductor.size());
    
    // Inductive properties
    println!("Inductance: {:.0} µH", inductor.inductance().get::<uom::si::inductance::henry>() * 1e6);
    println!("Current rating: {:.1} A", inductor.current_rating().get::<ampere>());
    println!("DC resistance: {:.1} Ω", inductor.dc_resistance().get::<uom::si::electrical_resistance::ohm>());
    println!("Core material: {:?}", inductor.core_material());
    
    // Frequency properties
    if let Some(srf) = inductor.self_resonant_frequency() {
        println!("Self-resonant freq: {:.2e} Hz", srf.get::<hertz>());
    }
}

fn demonstrate_diode(diode: &Diode) {
    println!("\n--- Silicon Diode ---");
    println!("Grid size: {:?}", diode.size());
    
    // Semiconductor properties
    println!("Forward voltage: {:.2} V", diode.forward_voltage().unwrap().get::<volt>());
    println!("Breakdown voltage: {:.0} V", diode.breakdown_voltage().unwrap().get::<volt>());
    println!("Forward current rating: {:.1} A", diode.forward_current_rating.get::<ampere>());
    println!("Junction capacitance: {:.0} pF", diode.junction_capacitance.get::<uom::si::capacitance::farad>() * 1e12);
    println!("Recovery time: {:.1} ns", diode.reverse_recovery_time);
}

fn demonstrate_opamp(opamp: &OpAmp) {
    println!("\n--- Operational Amplifier ---");
    println!("Grid size: {:?}", opamp.size());
    
    // OpAmp specific properties
    println!("Open-loop gain: {:.0} dB", 20.0 * opamp.open_loop_gain.log10());
    println!("Bandwidth: {:.0} MHz", opamp.bandwidth.get::<hertz>() / 1e6);
    println!("Input offset: {:.0} µV", opamp.input_offset_voltage.get::<volt>() * 1e6);
    println!("Input bias current: {:.0} pA", opamp.input_bias_current.get::<ampere>() * 1e12);
    println!("Slew rate: {:.1} V/µs", opamp.slew_rate);
    println!("Supply range: {:.0}V to {:.0}V", 
             opamp.supply_voltage_min.get::<volt>(),
             opamp.supply_voltage_max.get::<volt>());
}

fn frequency_analysis(resistor: &Resistor, capacitor: &Capacitor, inductor: &Inductor, opamp: &OpAmp) {
    let frequencies = [100.0, 1e3, 10e3, 100e3, 1e6, 10e6];
    
    println!("Frequency (Hz) | Resistor (Ω) | Capacitor (Ω) | Inductor (Ω) | OpAmp (Ω)");
    println!("---------------|---------------|----------------|---------------|----------");
    
    for freq_hz in frequencies {
        let freq = Frequency::new::<hertz>(freq_hz);
        
        // Only components that implement FrequencyDependent can be analyzed
        let r_impedance = resistor.resistance().get::<uom::si::electrical_resistance::ohm>();
        let c_impedance = capacitor.impedance_at_frequency(freq).get::<uom::si::electrical_resistance::ohm>();
        let l_impedance = inductor.impedance_at_frequency(freq).get::<uom::si::electrical_resistance::ohm>();
        let op_impedance = opamp.impedance_at_frequency(freq).get::<uom::si::electrical_resistance::ohm>();
        
        println!("{:10.0e} | {:11.2e} | {:12.2e} | {:11.2e} | {:8.2e}",
                 freq_hz, r_impedance, c_impedance, l_impedance, op_impedance);
    }
}

fn noise_analysis(resistor: &Resistor) {
    let room_temp = ThermodynamicTemperature::new::<kelvin>(298.15);
    let hot_temp = ThermodynamicTemperature::new::<kelvin>(373.15); // 100°C
    
    println!("Resistor thermal noise density:");
    println!("  At 25°C: {:.2} nV/√Hz", resistor.thermal_noise_density(room_temp) * 1e9);
    println!("  At 100°C: {:.2} nV/√Hz", resistor.thermal_noise_density(hot_temp) * 1e9);
}

fn grid_functionality_demo(resistor: &Resistor, opamp: &OpAmp) {
    println!("Grid placement functionality:");
    
    // Test single-unit component
    println!("Resistor size: {:?}", resistor.size());
    println!("Positions occupied at (0,0,0): {:?}", resistor.occupied_positions((0, 0, 0)));
    println!("Can place at (5,5,5): {}", resistor.can_place_at((5, 5, 5)));
    
    // Test multi-unit component
    println!("OpAmp size: {:?}", opamp.size());
    println!("Positions occupied at (0,0,0): {:?}", opamp.occupied_positions((0, 0, 0)));
    println!("Total positions: {}", opamp.occupied_positions((0, 0, 0)).len());
}

fn rc_filter_analysis(resistor: &Resistor, capacitor: &Capacitor) {
    println!("RC Low-pass Filter Analysis:");
    
    let r_value = resistor.resistance().get::<uom::si::electrical_resistance::ohm>();
    let c_value = capacitor.capacitance().get::<uom::si::capacitance::farad>();
    
    let cutoff_freq = 1.0 / (2.0 * std::f64::consts::PI * r_value * c_value);
    println!("  R = {:.0} Ω, C = {:.0} nF", r_value, c_value * 1e9);
    println!("  Theoretical cutoff: {:.2} Hz", cutoff_freq);
    
    // Account for parasitics
    let esr = capacitor.equivalent_series_resistance().get::<uom::si::electrical_resistance::ohm>();
    let actual_cutoff = 1.0 / (2.0 * std::f64::consts::PI * (r_value + esr) * c_value);
    println!("  With ESR ({:.3} Ω): {:.2} Hz", esr, actual_cutoff);
    
    // Power analysis
    let test_voltage = ElectricPotential::new::<volt>(5.0);
    let test_current = ElectricCurrent::new::<ampere>(0.005); // 5mA
    
    println!("  Safe operating area check at 5V, 5mA:");
    println!("    Resistor: {}", resistor.is_within_safe_operating_area(test_voltage, test_current));
    
    // Material properties for thermal simulation
    println!("  Thermal properties:");
    println!("    Resistor thermal conductivity: {:.1} W/m·K", resistor.thermal_conductivity());
    println!("    Capacitor thermal conductivity: {:.1} W/m·K", capacitor.thermal_conductivity());
} 