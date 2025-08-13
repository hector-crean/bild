use bevy::prelude::*;
use serde::{Serialize, Deserialize};    


#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetKind{
    Power,
    Ground,
    Signal,
    Analog,
    Digital,
}

/// Net : A collection of interconnected pins, forming a single electrical path.
/// Nets can be physically connected using wires or logically connected using net labels.
/// A net is also sometimes referred to as a wire, signal, or connection.

/// A named electrical net.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[require(Name)]
pub struct Net {
    pub net_type: NetKind,
}

impl Net {
    pub fn new(net_type: NetKind) -> Self {
        Self { net_type }
    }
}