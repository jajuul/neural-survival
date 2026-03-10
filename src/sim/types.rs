use serde::{Deserialize, Serialize};

/// Number of non-vision scalar features included in controller input.
pub const CORE_SENSOR_FEATURES: usize = 2;
/// Maximum number of bins in one vision channel.
pub const MAX_VISION_CELLS: usize = crate::config::MAX_PERCEPTION_CHANNELS as usize;
/// Number of distinct vision channels currently exposed to controllers.
pub const VISION_CHANNEL_COUNT: usize = 3;
/// Fixed controller input width used by MLP/LTC genomes.
pub const CONTROLLER_SENSOR_INPUT_SIZE: usize =
    CORE_SENSOR_FEATURES + MAX_VISION_CELLS * VISION_CHANNEL_COUNT;

/// 2D vector type used for world-space positions and velocities.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    /// Construct a new vector from `x` and `y` components.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Build a unit vector that points in `angle_radians` direction.
    pub fn from_angle(angle_radians: f32) -> Self {
        Self {
            x: angle_radians.cos(),
            y: angle_radians.sin(),
        }
    }

    /// Multiply both vector components by `factor`.
    pub fn scale(self, factor: f32) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
        }
    }

    /// Add another vector to this vector.
    pub fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    /// Euclidean vector length.
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

/// Neural input vector placeholder for the controller API.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct SensorVector {
    pub values: [f32; CONTROLLER_SENSOR_INPUT_SIZE],
}

impl SensorVector {
    /// Build a fixed-width sensor vector by copying values into the front of the array and
    /// zero-filling the remainder.
    pub fn from_slice(values: &[f32]) -> Self {
        let mut output = [0.0; CONTROLLER_SENSOR_INPUT_SIZE];
        for (dst, src) in output.iter_mut().zip(values.iter().copied()) {
            *dst = src;
        }
        Self { values: output }
    }
}

/// Decomposed sensor inputs used for controller observability.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct SensorObservation {
    pub core_values: [f32; 2],
    pub food_eye_bins: Vec<f32>,
    pub prey_eye_bins: Vec<f32>,
    pub predator_eye_bins: Vec<f32>,
    pub eye_bins: Vec<f32>,
    pub input: SensorVector,
}

/// Neural output controls for one simulation step.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct ActionVector {
    pub steering: f32,
    pub throttle: f32,
}

fn default_eye_cell_angle_radians() -> f32 {
    crate::config::SimulationConfig::default().eye_fov_angle_radians
        / crate::config::SimulationConfig::default().default_perception_channels as f32
}

/// Ecological role used by sensing and future role-specific world rules.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    #[default]
    Prey,
    Predator,
}

/// Runtime state for one agent.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AgentState {
    pub id: u32,
    pub position: Vec2,
    pub heading_radians: f32,
    pub velocity: Vec2,
    pub energy: f32,
    pub alive: bool,
    pub max_speed: f32,
    pub perception_radius: f32,
    pub perception_channels: u16,
    #[serde(default = "default_eye_cell_angle_radians")]
    pub eye_cell_angle_radians: f32,
    pub network_size: usize,
    pub metabolic_efficiency: f32,
    #[serde(default)]
    pub role: AgentRole,
    pub food_consumed: u32,
    #[serde(default)]
    pub prey_captured: u32,
}

/// Runtime state for one food item.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct FoodState {
    pub id: u32,
    pub position: Vec2,
    pub radius: f32,
    pub energy_value: f32,
    #[serde(default)]
    pub motion_angle_radians: f32,
    #[serde(default)]
    pub cluster_id: Option<u32>,
    #[serde(default)]
    pub cluster_angle_radians: f32,
    #[serde(default)]
    pub cluster_distance_factor: f32,
}

/// Moving resource cluster that owns a region of food items.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct FoodClusterState {
    pub id: u32,
    pub position: Vec2,
    pub motion_angle_radians: f32,
    #[serde(default)]
    pub phase_offset: f32,
}

/// Entire mutable simulation world at a given step.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct WorldState {
    pub width: f32,
    pub height: f32,
    pub agents: Vec<AgentState>,
    pub food: Vec<FoodState>,
    #[serde(default)]
    pub food_clusters: Vec<FoodClusterState>,
    #[serde(default)]
    pub world_seed: u64,
    pub step_count: u64,
}

/// Deterministic lighting phase derived from world step count and config.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DayNightState {
    pub phase: f32,
    pub daylight: f32,
    pub visibility_multiplier: f32,
    pub is_night: bool,
}

/// Aggregated side effects for one world step.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct StepOutcome {
    pub consumed_food: usize,
    pub prey_captures: usize,
    pub deaths: usize,
}

/// Per-term energy accounting for one agent step.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct EnergyBreakdown {
    pub base_loss: f32,
    pub speed_loss: f32,
    pub network_loss: f32,
    pub perception_loss: f32,
    pub food_gain: f32,
    pub capture_gain: f32,
    pub total_loss: f32,
    pub net_delta: f32,
}

/// Per-agent telemetry captured during one world step.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct StepAgentTelemetry {
    pub energy: EnergyBreakdown,
    pub consumed_food: bool,
    pub captured_prey: bool,
    pub alive_after: bool,
}

/// Summary emitted after a headless episode for deterministic comparison.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct EpisodeSummary {
    pub step_count: u64,
    pub alive_agents: usize,
    pub consumed_food: u32,
    pub remaining_food: usize,
    pub total_energy: f32,
}
