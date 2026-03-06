use serde::{Deserialize, Serialize};

use crate::brain::{BrainGenome, ControllerKind};

fn default_eye_cell_angle_radians() -> f32 {
    crate::config::SimulationConfig::default().eye_fov_angle_radians
        / crate::config::SimulationConfig::default().default_perception_channels as f32
}

/// Evolvable scalar trait genes for one agent.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraitGenome {
    pub max_speed: f32,
    pub perception_radius: f32,
    pub perception_channels: u16,
    #[serde(default = "default_eye_cell_angle_radians")]
    pub eye_cell_angle_radians: f32,
    pub metabolic_efficiency: f32,
    pub food_preference_mask: u32,
}

/// Full evolvable genome for one agent.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentGenome {
    pub controller_kind: ControllerKind,
    pub brain: BrainGenome,
    pub traits: TraitGenome,
}
