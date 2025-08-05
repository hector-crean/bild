use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::time::{SimulationTime, TimeBackend};

/// Trait for simulation backends
pub trait SimulationBackend: Send + Sync + 'static {
    type Config: Clone + Send + Sync + 'static;
    type State: Clone + Send + Sync + 'static;
    type Result: Clone + Send + Sync + 'static;
    type Error: std::error::Error + Send + Sync + 'static;

    /// Initialize the simulation backend
    fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error>;
    
    /// Run a single simulation step
    fn step(&mut self, state: &Self::State, time: SimulationTime) -> Result<Self::Result, Self::Error>;
    
    /// Run multiple simulation steps
    fn run_steps(&mut self, state: &Self::State, time: SimulationTime, steps: usize) -> Result<Vec<Self::Result>, Self::Error>;
    
    /// Get the time backend used by this simulation
    fn time_backend(&self) -> TimeBackend;
    
    /// Check if the backend is ready for simulation
    fn is_ready(&self) -> bool;
    
 
}
