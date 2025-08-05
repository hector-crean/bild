use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Different types of time backends for simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeBackend {
    /// Real-time simulation synchronized with wall clock
    RealTime,
    /// Accelerated simulation (faster than real-time)
    Accelerated { factor: f64 },
    /// Decelerated simulation (slower than real-time)
    Decelerated { factor: f64 },
    /// Fixed time step simulation
    FixedStep { step_size: Duration },
    /// Event-driven simulation
    EventDriven,
    /// External time source (e.g., from API)
    External { source: String },
    /// Custom time backend
    Custom { name: String, config: serde_json::Value },
}

impl Default for TimeBackend {
    fn default() -> Self {
        TimeBackend::RealTime
    }
}

/// Simulation time representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationTime {
    pub timestamp: f64,           // Simulation timestamp in seconds
    pub wall_time: SystemTime,    // Wall clock time
    pub backend: TimeBackend,     // Time backend used
    pub step_count: u64,          // Number of simulation steps
    pub delta_time: Duration,     // Time since last step
}

impl SimulationTime {
    /// Create a new simulation time with current wall time
    pub fn now(backend: TimeBackend) -> Self {
        Self {
            timestamp: 0.0,
            wall_time: SystemTime::now(),
            backend,
            step_count: 0,
            delta_time: Duration::ZERO,
        }
    }
    
    /// Create simulation time from timestamp
    pub fn from_timestamp(timestamp: f64, backend: TimeBackend) -> Self {
        Self {
            timestamp,
            wall_time: SystemTime::now(),
            backend,
            step_count: 0,
            delta_time: Duration::ZERO,
        }
    }
    
    /// Advance simulation time by delta
    pub fn advance(&mut self, delta: Duration) {
        self.delta_time = delta;
        self.step_count += 1;
        
        match &self.backend {
            TimeBackend::RealTime => {
                self.timestamp += delta.as_secs_f64();
            }
            TimeBackend::Accelerated { factor } => {
                self.timestamp += delta.as_secs_f64() * factor;
            }
            TimeBackend::Decelerated { factor } => {
                self.timestamp += delta.as_secs_f64() * factor;
            }
            TimeBackend::FixedStep { step_size } => {
                self.timestamp += step_size.as_secs_f64();
            }
            TimeBackend::EventDriven => {
                // Event-driven time doesn't advance automatically
            }
            TimeBackend::External { .. } => {
                // External time source - would be updated by external system
            }
            TimeBackend::Custom { .. } => {
                // Custom time advancement logic
                self.timestamp += delta.as_secs_f64();
            }
        }
    }
    
    /// Get the current simulation time in seconds
    pub fn as_secs_f64(&self) -> f64 {
        self.timestamp
    }
    
    /// Get the current simulation time as duration
    pub fn as_duration(&self) -> Duration {
        Duration::from_secs_f64(self.timestamp)
    }
    
    /// Check if this is a real-time simulation
    pub fn is_real_time(&self) -> bool {
        matches!(self.backend, TimeBackend::RealTime)
    }
    
    /// Get the time acceleration factor
    pub fn acceleration_factor(&self) -> f64 {
        match &self.backend {
            TimeBackend::Accelerated { factor } => *factor,
            TimeBackend::Decelerated { factor } => *factor,
            _ => 1.0,
        }
    }
}

impl Default for SimulationTime {
    fn default() -> Self {
        Self::now(TimeBackend::RealTime)
    }
}

/// Time manager for coordinating different time backends
#[derive(Resource)]
pub struct TimeManager {
    pub current_time: SimulationTime,
    pub start_time: Instant,
    pub last_update: Instant,
    pub time_scale: f64,
    pub is_paused: bool,
}

impl TimeManager {
    pub fn new(backend: TimeBackend) -> Self {
        let now = Instant::now();
        Self {
            current_time: SimulationTime::now(backend),
            start_time: now,
            last_update: now,
            time_scale: 1.0,
            is_paused: false,
        }
    }
    
    /// Update the time manager
    pub fn update(&mut self) {
        if self.is_paused {
            return;
        }
        
        let now = Instant::now();
        let delta = now.duration_since(self.last_update);
        
        // Apply time scale
        let scaled_delta = Duration::from_secs_f64(
            delta.as_secs_f64() * self.time_scale
        );
        
        self.current_time.advance(scaled_delta);
        self.last_update = now;
    }
    
    /// Pause the simulation
    pub fn pause(&mut self) {
        self.is_paused = true;
    }
    
    /// Resume the simulation
    pub fn resume(&mut self) {
        self.is_paused = false;
        self.last_update = Instant::now();
    }
    
    /// Set the time scale
    pub fn set_time_scale(&mut self, scale: f64) {
        self.time_scale = scale.max(0.0);
    }
    
    /// Get the elapsed simulation time
    pub fn elapsed(&self) -> Duration {
        self.current_time.as_duration()
    }
    
    /// Get the elapsed wall time
    pub fn wall_elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for TimeManager {
    fn default() -> Self {
        Self::new(TimeBackend::RealTime)
    }
}

/// Plugin for time management
pub struct TimeManagerPlugin;

impl Plugin for TimeManagerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimeManager::default())
            .add_systems(Update, update_time_manager);
    }
}

/// System to update the time manager
fn update_time_manager(mut time_manager: ResMut<TimeManager>) {
    time_manager.update();
}

/// Time-related events
#[derive(Event)]
pub enum TimeEvent {
    /// Simulation time has advanced
    TimeAdvanced { delta: Duration, new_time: SimulationTime },
    /// Simulation has been paused
    Paused,
    /// Simulation has been resumed
    Resumed,
    /// Time scale has changed
    TimeScaleChanged { new_scale: f64 },
    /// Time backend has changed
    BackendChanged { new_backend: TimeBackend },
}

/// Time synchronization for distributed simulations
#[derive(Debug, Clone)]
pub struct TimeSync {
    pub local_time: SimulationTime,
    pub remote_times: Vec<(String, SimulationTime)>,
    pub sync_interval: Duration,
    pub last_sync: Instant,
}

impl TimeSync {
    pub fn new(sync_interval: Duration) -> Self {
        Self {
            local_time: SimulationTime::default(),
            remote_times: Vec::new(),
            sync_interval,
            last_sync: Instant::now(),
        }
    }
    
    /// Check if it's time to synchronize
    pub fn should_sync(&self) -> bool {
        self.last_sync.elapsed() >= self.sync_interval
    }
    
    /// Update remote time
    pub fn update_remote_time(&mut self, node_id: String, time: SimulationTime) {
        if let Some(existing) = self.remote_times.iter_mut().find(|(id, _)| *id == node_id) {
            existing.1 = time;
        } else {
            self.remote_times.push((node_id, time));
        }
    }
    
    /// Get the average remote time
    pub fn average_remote_time(&self) -> Option<SimulationTime> {
        if self.remote_times.is_empty() {
            return None;
        }
        
        let avg_timestamp = self.remote_times.iter()
            .map(|(_, time)| time.timestamp)
            .sum::<f64>() / self.remote_times.len() as f64;
        
        Some(SimulationTime::from_timestamp(avg_timestamp, self.local_time.backend.clone()))
    }
}

/// Serializable time sync data for network transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSyncData {
    pub local_time: SimulationTime,
    pub remote_times: Vec<(String, SimulationTime)>,
    pub sync_interval_secs: f64,
    pub last_sync_secs: f64,
}

impl From<&TimeSync> for TimeSyncData {
    fn from(time_sync: &TimeSync) -> Self {
        Self {
            local_time: time_sync.local_time.clone(),
            remote_times: time_sync.remote_times.clone(),
            sync_interval_secs: time_sync.sync_interval.as_secs_f64(),
            last_sync_secs: time_sync.last_sync.elapsed().as_secs_f64(),
        }
    }
}

impl From<TimeSyncData> for TimeSync {
    fn from(data: TimeSyncData) -> Self {
        Self {
            local_time: data.local_time,
            remote_times: data.remote_times,
            sync_interval: Duration::from_secs_f64(data.sync_interval_secs),
            last_sync: Instant::now() - Duration::from_secs_f64(data.last_sync_secs),
        }
    }
} 