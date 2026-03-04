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
    pub initial_agent_energy: f32,
    pub base_energy_cost_per_second: f32,
    pub speed_energy_cost_factor: f32,
    pub network_energy_cost_factor: f32,
    pub perception_energy_cost_factor: f32,
    pub food_energy_gain: f32,
    pub food_collision_radius: f32,
    pub default_agent_max_speed: f32,
    pub max_turn_rate_radians: f32,
    pub default_perception_radius: f32,
    pub default_perception_channels: u16,
    pub default_network_size: usize,
    pub default_metabolic_efficiency: f32,
    pub controller_ltc_probability: f32,
    pub fitness_weight_survival: f32,
    pub fitness_weight_food: f32,
    pub fitness_weight_energy: f32,
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
            initial_agent_energy: 100.0,
            base_energy_cost_per_second: 0.5,
            speed_energy_cost_factor: 0.03,
            network_energy_cost_factor: 0.0008,
            perception_energy_cost_factor: 0.0005,
            food_energy_gain: 20.0,
            food_collision_radius: 1.2,
            default_agent_max_speed: 6.0,
            max_turn_rate_radians: std::f32::consts::PI,
            default_perception_radius: 15.0,
            default_perception_channels: 6,
            default_network_size: 64,
            default_metabolic_efficiency: 1.0,
            controller_ltc_probability: 0.5,
            fitness_weight_survival: 1.0,
            fitness_weight_food: 2.0,
            fitness_weight_energy: 0.5,
            rule_toggles: RuleToggles::default(),
        }
    }
}
