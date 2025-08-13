
pub mod diode;
pub mod resistor;
pub mod inductor;
pub mod capacitor;



use block3d_core::block::Block3DLike;
use block3d_core::face::Face;
use circuit_physics_core::physical::PackageType;
use serde::{Serialize, Deserialize};
use bevy::prelude::*;
use strum::EnumIter;
use uom::si::f64::*;
use uom::si::{
    electric_potential::volt,
    electric_current::ampere,
    electrical_resistance::ohm,
    capacitance::farad,
    inductance::henry,
    frequency::hertz,
    power::watt,
    thermodynamic_temperature::kelvin,
};
use circuit_physics_core::{
    physical::{
        Resistive, Capacitive, Inductive, FrequencyDependent, PowerRated, NoiseGenerating, Semiconductor 
    },
    material_properties::MaterialProperties
};
use interaction::drag::two_d::Draggable2d;

use diode::Diode;
use capacitor::Capacitor;
use inductor::Inductor;
use resistor::Resistor;
















// ============================================================================
// UNIFIED COMPONENT ENUM FOR WFC SOLVER
// ============================================================================

/// Unified enum containing all analog circuit components for use with WFC solver
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, EnumIter, Component)]
#[require(Transform, Draggable2d)]
pub enum Part {
    Resistor(Resistor),
    Capacitor(Capacitor),
    Inductor(Inductor),
    Diode(Diode),
}

impl Default for Part {
    fn default() -> Self {
        Self::Resistor(Resistor::default())
    }
}



impl Block3DLike for Part {
    fn size(&self) -> (u32, u32, u32) {
        match self {
            Part::Resistor(r) => r.size(),
            Part::Capacitor(c) => c.size(),
            Part::Inductor(l) => l.size(),
            Part::Diode(d) => d.size(),
        }
    }
    
    fn faces(&self) -> impl Iterator<Item = Face> {
        match self {
            Part::Resistor(r) => r.faces().collect::<Vec<_>>().into_iter(),
            Part::Capacitor(c) => c.faces().collect::<Vec<_>>().into_iter(),
            Part::Inductor(l) => l.faces().collect::<Vec<_>>().into_iter(),
            Part::Diode(d) => d.faces().collect::<Vec<_>>().into_iter(),
        }
    }
    
    fn symbol(&self) -> String {
        match self {
            Part::Resistor(r) => r.symbol(),
            Part::Capacitor(c) => c.symbol(),
            Part::Inductor(l) => l.symbol(),
            Part::Diode(d) => d.symbol(),
        }
    }
    
    fn ranking(&self) -> f32 {
        match self {
            Part::Resistor(_) => 1.0,
            Part::Capacitor(_) => 1.2,
            Part::Inductor(_) => 1.1,
            Part::Diode(_) => 0.8,
        }
    }
}

impl MaterialProperties for Part {
    fn thermal_conductivity(&self) -> f32 {
        match self {
            Part::Resistor(r) => r.thermal_conductivity(),
            Part::Capacitor(c) => c.thermal_conductivity(),
            Part::Inductor(l) => l.thermal_conductivity(),
            Part::Diode(d) => d.thermal_conductivity(),
        }
    }
    
    fn electrical_resistivity(&self) -> f32 {
        match self {
            Part::Resistor(r) => r.electrical_resistivity(),
            Part::Capacitor(c) => c.electrical_resistivity(),
            Part::Inductor(l) => l.electrical_resistivity(),
            Part::Diode(d) => d.electrical_resistivity(),
        }
    }
    
    fn youngs_modulus(&self) -> f32 {
        match self {
            Part::Resistor(r) => r.youngs_modulus(),
            Part::Capacitor(c) => c.youngs_modulus(),
            Part::Inductor(l) => l.youngs_modulus(),
            Part::Diode(d) => d.youngs_modulus(),
        }
    }
    
    fn poisson_ratio(&self) -> f32 {
        match self {
            Part::Resistor(r) => r.poisson_ratio(),
            Part::Capacitor(c) => c.poisson_ratio(),
            Part::Inductor(l) => l.poisson_ratio(),
            Part::Diode(d) => d.poisson_ratio(),
        }
    }
    
    fn density(&self) -> f32 {
        match self {
            Part::Resistor(r) => r.density(),
            Part::Capacitor(c) => c.density(),
            Part::Inductor(l) => l.density(),
            Part::Diode(d) => d.density(),
        }
    }
    
    fn specific_heat(&self) -> f32 {
        match self {
            Part::Resistor(r) => r.specific_heat(),
            Part::Capacitor(c) => c.specific_heat(),
            Part::Inductor(l) => l.specific_heat(),
            Part::Diode(d) => d.specific_heat(),
        }
    }
}

// Convenience constructors for the enum
impl Part {
    pub fn resistor() -> Self {
        Self::Resistor(Resistor::default())
    }
    
    pub fn capacitor() -> Self {
        Self::Capacitor(Capacitor::default())
    }
    
    pub fn inductor() -> Self {
        Self::Inductor(Inductor::default())
    }
    
   
    
    pub fn diode() -> Self {
        Self::Diode(Diode::default())
    }
}

// Manual Hash implementations for all component structs, ignoring floating point fields
impl std::hash::Hash for Capacitor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.size.hash(state);
        self.faces.hash(state);
        self.dielectric.hash(state);
        self.package.hash(state);
    }
}

impl std::hash::Hash for Inductor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.size.hash(state);
        self.faces.hash(state);
        self.core_material.hash(state);
        self.package.hash(state);
    }
}

impl std::hash::Hash for Diode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.size.hash(state);
        self.faces.hash(state);
        self.package.hash(state);
    }
}


impl std::hash::Hash for Part {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Part::Resistor(r) => r.hash(state),
            Part::Capacitor(c) => c.hash(state),
            Part::Inductor(l) => l.hash(state),
            Part::Diode(d) => d.hash(state),
        }
    }
} 







