use serde::{Deserialize, Serialize};

/// Runtime switches for optional energy model terms.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct RuleToggles {
    pub energy_by_speed: bool,
    pub energy_by_network_size: bool,
    pub energy_by_perception_size: bool,
}

impl Default for RuleToggles {
    fn default() -> Self {
        Self {
            energy_by_speed: true,
            energy_by_network_size: true,
            energy_by_perception_size: true,
        }
    }
}

/// Static simulation configuration loaded from TOML.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct SimulationConfig {
    pub world_width: f32,
    pub world_height: f32,
    pub timestep_seconds: f32,
    pub episode_steps: u32,
    pub population_size: usize,
    pub food_count: usize,
    pub elitism_fraction: f32,
    pub tournament_size: usize,
    pub mutation_rate: f32,
    pub rule_toggles: RuleToggles,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            world_width: 200.0,
            world_height: 200.0,
            timestep_seconds: 0.1,
            episode_steps: 300,
            population_size: 128,
            food_count: 400,
            elitism_fraction: 0.10,
            tournament_size: 5,
            mutation_rate: 0.15,
            rule_toggles: RuleToggles::default(),
        }
    }
}
