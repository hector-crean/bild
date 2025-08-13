use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::circuit::relations::{OnNet, OfPart, Pins};

/// A pin/port on a part.
///
/// Pin : A specific electrical connection point on a part.
/// Each pin has a defined purpose, like input, output, power, or ground.
/// Pins are used to connect parts together to form a circuit.
/// 
/// Keep this component minimal. Add semantics/constraints via separate components
/// like `PinRole`, `PinDomain`, etc.
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component, Serialize, Deserialize)]
#[require(PinRole::Passive, PinDomain::Mixed, PinLabel(None))]
pub struct Pin {
    /// Human-readable pin name
    pub name: String,
    /// 1-based index for ordering
    pub index: u8,
}

impl Pin {
    /// Construct a new pin. Panics if `index` is not 1-based.
    pub fn new(name: impl Into<String>, index: u8) -> Self {
        assert!(index >= 1, "Pin index must be 1-based");
        Self { name: name.into(), index }
    }

    /// Owning part entity, via `OfPart` relationship.
    pub fn owning_part(world: &World, pin: Entity) -> Option<Entity> {
        world.get::<OfPart>(pin).map(|rel| rel.0)
    }

    /// Connected net entity, if any, via `OnNet` relationship.
    pub fn connected_net(world: &World, pin: Entity) -> Option<Entity> {
        world.get::<OnNet>(pin).map(|rel| rel.0)
    }
}

// ============================================================================
// Semantics and constraints as separate components
// ============================================================================

/// Electrical role of a pin. Useful for DRC and auto-wiring.
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component, Serialize, Deserialize)]
pub enum PinRole {
    Input,
    Output,
    InOut,
    #[default]
    Passive,
    Power,
    Ground,
    OpenCollector,
    TriState,
    NoConnect,
}

/// Logical polarity for digital or control pins.
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
#[reflect(Component, Serialize, Deserialize)]
pub enum PinPolarity {
    ActiveHigh,
    ActiveLow,
}

/// Electrical domain classification.
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component, Serialize, Deserialize)]
pub enum PinDomain {
    Analog,
    Digital,
    #[default]
    Mixed,
}

/// Allowed voltage range for this pin (in volts).
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
#[reflect(Component, Serialize, Deserialize)]
pub struct PinVoltageRange {
    pub min_volts: f32,
    pub max_volts: f32,
}

/// Maximum continuous current this pin is intended to carry (in amperes).
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
#[reflect(Component, Serialize, Deserialize)]
pub struct PinCurrentLimit {
    pub max_amps: f32,
}

/// Expected or required pin impedance (in ohms).
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
#[reflect(Component, Serialize, Deserialize)]
pub struct PinImpedance {
    pub ohms: f32,
}

/// PCB/library pad identifier (e.g., "1", "A1", "GND").
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component, Serialize, Deserialize)]
pub struct PinPad {
    pub pad: String,
}

/// Logical grouping (e.g., bus membership like "D[0..7]" or bank name).
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component, Serialize, Deserialize)]
pub struct PinGroup {
    pub group: String,
}

/// Optional override for displayed label.
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component, Serialize, Deserialize)]
pub struct PinLabel(pub Option<String>);

/// Optional color hint for rendering.
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
#[reflect(Component, Serialize, Deserialize)]
pub struct PinColor(pub Color);

// ============================================================================
// Convenience helpers
// ============================================================================

/// Return all pins owned by `part` (stored order), if the reverse index is present.
pub fn pins_of_part(world: &World, part: Entity) -> Option<Vec<Entity>> {
    world.get::<Pins>(part).map(|pins| pins.iter().collect())
}




pub trait PinQueryExt {
    fn owning_part(&self, pin: Entity) -> Option<Entity>;      // via `OfPart`
    fn connected_net(&self, pin: Entity) -> Option<Entity>;    // via `ConnectedToNet`
    fn pins_of_part(&self, part: Entity) -> Vec<Entity>;       // via `Pins`
  }