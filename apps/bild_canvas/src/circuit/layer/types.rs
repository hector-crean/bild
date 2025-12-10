use bevy::prelude::*;

/// Stable identifier for a fabrication layer
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Reflect, PartialOrd, Ord, Component)]
pub struct LayerId(pub u16);



/// Marks which layer an entity belongs to (geometry, parts, traces)
#[derive(Component, Copy, Clone, Debug, Reflect)]
pub struct OnLayer(pub LayerId);

/// Types of vias between layers
#[derive(Clone, Copy, Debug, Reflect)]
pub enum ViaKind {
    Through, // spans entire stack-up
    Blind,   // from outer layer to an inner layer
    Buried,  // between inner layers only
}

/// Connectivity element between layers
#[derive(Component, Debug, Reflect)]
pub struct Via {
    pub from: LayerId,
    pub to: LayerId,
    pub kind: ViaKind,
    /// Nominal via diameter (world units)
    pub diameter: f32,
}


