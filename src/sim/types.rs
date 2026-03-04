use serde::{Deserialize, Serialize};

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
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct SensorVector {
    pub values: Vec<f32>,
}

/// Neural output controls for one simulation step.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct ActionVector {
    pub steering: f32,
    pub throttle: f32,
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
    pub network_size: usize,
    pub metabolic_efficiency: f32,
    pub food_consumed: u32,
}

/// Runtime state for one food item.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct FoodState {
    pub id: u32,
    pub position: Vec2,
    pub radius: f32,
    pub energy_value: f32,
}

/// Entire mutable simulation world at a given step.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct WorldState {
    pub width: f32,
    pub height: f32,
    pub agents: Vec<AgentState>,
    pub food: Vec<FoodState>,
    pub step_count: u64,
}

/// Aggregated side effects for one world step.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct StepOutcome {
    pub consumed_food: usize,
    pub deaths: usize,
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
