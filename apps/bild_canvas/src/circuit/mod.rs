pub mod motif;
pub mod net;
pub mod part;
pub mod pin;

pub mod graph;

pub mod commands;
pub mod graph_gizmos;
pub mod layer;
pub mod query;
pub mod relations;
pub mod solver;

use bevy::prelude::*;

use crate::circuit::{graph::{EdgeTransformPropagatePlugin, render::CircuitGraphRenderPlugin}, solver::CircuitSolverPlugin};

pub struct CircuitPlugin;

impl Plugin for CircuitPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EdgeTransformPropagatePlugin, CircuitGraphRenderPlugin));
    }
}