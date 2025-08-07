use block3d_algorithm::wfc::{
    graph::WFCGraph, heuristics::weighted_random_heuristic::WeightedRandomHeuristic,
    solver::WFCSolver,
};
use block3d_core::block::AnalogComponent;
use block3d_core::block::Block3DLike;
use std::collections::HashSet;
use uom::si::{capacitance::farad, electrical_resistance::ohm, inductance::henry};

fn main() {
    let dimensions = (3, 3, 1);

    let graph = WFCGraph::<AnalogComponent>::grid_graph(dimensions);

    // Create a set of circuit components for the solver
    let mut component_set = HashSet::<AnalogComponent>::new();

    // Add various analog circuit components
    component_set.insert(AnalogComponent::resistor(1000.0, 0.25, 5.0)); // 1kΩ
    component_set.insert(AnalogComponent::resistor(10000.0, 0.25, 5.0)); // 10kΩ
    component_set.insert(AnalogComponent::capacitor(1e-9, 50.0, 10.0)); // 1nF
    component_set.insert(AnalogComponent::capacitor(1e-6, 25.0, 20.0)); // 1µF
    component_set.insert(AnalogComponent::inductor(1e-6, 1.0)); // 1µH
    component_set.insert(AnalogComponent::op_amp(1e6, 15.0)); // Op-amp
    component_set.insert(AnalogComponent::diode()); // Generic diode

    println!(
        "Starting WFC circuit layout with {} component types",
        component_set.len()
    );

    let invariants = vec![];
    let heuristic = Box::new(WeightedRandomHeuristic);
    let mut solver = WFCSolver::new(graph, component_set, invariants, heuristic, vec![]);

    match solver.solve() {
        Ok(_solution) => {
            println!("Circuit layout solution found!");
            println!("Final circuit layout:");

            // Print the solution in a more readable format
            for node_weight in solver.graph.node_weights() {
                let component_info = match &node_weight.block {
                    AnalogComponent::Resistor(r) => format!("{}Ω", r.resistance.get::<ohm>()),
                    AnalogComponent::Capacitor(c) => {
                        format!("{}F", c.capacitance.get::<farad>())
                    }
                    AnalogComponent::Inductor(l) => format!("{}H", l.inductance.get::<henry>()),
                    AnalogComponent::Diode(_) => "Diode".to_string(),
                    AnalogComponent::OpAmp(_) => "Op-Amp".to_string(),
                };

                // println!(
                //     "Position {:?}: {} ({})",
                //     node_weight.position,
                //     component_info
                // );
            }
        }
        Err(e) => println!("Error solving circuit layout: {}", e),
    }
}
