pub mod net;
pub mod part;
pub mod pin;
pub mod trace;

pub mod circuit_graph;
pub mod circuit_graph_render;
pub mod commands;
pub mod graph_gizmos;
pub mod query;
pub mod relations;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;



