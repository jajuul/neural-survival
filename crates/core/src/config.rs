//! TOML config. Every field has a default, so parsing yields a fully resolved
//! config; unknown keys are rejected so a misspelt setting is never ignored.
//! Angles are in radians and rates are per simulation step.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid config: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("invalid config: `{field}` {reason}")]
    Invalid {
        field: &'static str,
        reason: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub world: WorldConfig,
    pub food: FoodConfig,
    pub population: PopulationConfig,
    pub prey: RoleConfig,
    pub agent: AgentConfig,
    pub energy: EnergyConfig,
    pub sensors: SensorConfig,
    pub mlp: MlpConfig,
    pub evaluation: EvaluationConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WorldConfig {
    pub width: f64,
    pub height: f64,
    /// Fraction of speed lost each step, in [0, 1].
    pub drag: f64,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            width: 200.0,
            height: 200.0,
            drag: 0.1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FoodConfig {
    pub count: usize,
    pub energy: f64,
    pub radius: f64,
}

impl Default for FoodConfig {
    fn default() -> Self {
        Self {
            count: 40,
            energy: 30.0,
            radius: 1.0,
        }
    }
}

/// Number of Genomes per Species.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PopulationConfig {
    pub mlp_prey: usize,
}

impl Default for PopulationConfig {
    fn default() -> Self {
        Self { mlp_prey: 20 }
    }
}

/// Settings that differ per Role.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RoleConfig {
    pub starting_energy: f64,
}

impl Default for RoleConfig {
    fn default() -> Self {
        Self {
            starting_energy: 100.0,
        }
    }
}

/// Body and senses shared by every Agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AgentConfig {
    pub max_speed: f64,
    pub max_acceleration: f64,
    /// Heading change at full steering, in radians.
    pub turn_rate: f64,
    pub body_radius: f64,
    pub vision_range: f64,
    /// Total angular width of the eyes, centred on the heading, in radians.
    pub field_of_view: f64,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_speed: 2.0,
            max_acceleration: 0.5,
            turn_rate: 0.3,
            body_radius: 2.0,
            vision_range: 40.0,
            field_of_view: 2.0 * std::f64::consts::FRAC_PI_3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EnergyConfig {
    pub idle_drain: f64,
}

impl Default for EnergyConfig {
    fn default() -> Self {
        Self { idle_drain: 0.1 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SensorConfig {
    /// Sectors per eye; fixed for the run.
    pub eye_bins: usize,
    /// Energy at which the normalised energy sensor reads 1.
    pub energy_scale: f64,
}

impl Default for SensorConfig {
    fn default() -> Self {
        Self {
            eye_bins: 5,
            energy_scale: 200.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MlpConfig {
    pub hidden_layers: Vec<usize>,
}

impl Default for MlpConfig {
    fn default() -> Self {
        Self {
            hidden_layers: vec![8],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EvaluationConfig {
    pub episode_steps: u32,
}

impl Default for EvaluationConfig {
    fn default() -> Self {
        Self {
            episode_steps: 1000,
        }
    }
}

impl Config {
    pub fn from_toml_str(text: &str) -> Result<Self, ConfigError> {
        let config: Config = toml::from_str(text)?;
        config.validate()?;
        Ok(config)
    }

    pub fn to_toml_string(&self) -> String {
        toml::to_string(self).expect("config always serialises")
    }

    /// Width of every controller's input vector: normalised energy,
    /// normalised speed, then the food Eye Bins.
    pub fn controller_input_width(&self) -> usize {
        2 + self.sensors.eye_bins
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        let checks: [(bool, &'static str, &'static str); 13] = [
            (self.world.width > 0.0, "world.width", "must be positive"),
            (self.world.height > 0.0, "world.height", "must be positive"),
            (
                (0.0..=1.0).contains(&self.world.drag),
                "world.drag",
                "must be in [0, 1]",
            ),
            (
                self.food.energy >= 0.0,
                "food.energy",
                "must not be negative",
            ),
            (
                self.food.radius >= 0.0,
                "food.radius",
                "must not be negative",
            ),
            (
                self.agent.max_speed >= 0.0,
                "agent.max_speed",
                "must not be negative",
            ),
            (
                self.agent.body_radius >= 0.0,
                "agent.body_radius",
                "must not be negative",
            ),
            (
                self.agent.vision_range > 0.0,
                "agent.vision_range",
                "must be positive",
            ),
            (
                self.agent.field_of_view > 0.0 && self.agent.field_of_view <= std::f64::consts::TAU,
                "agent.field_of_view",
                "must be in (0, 2π]",
            ),
            (
                self.energy.idle_drain > 0.0,
                "energy.idle_drain",
                "must be positive",
            ),
            (
                self.sensors.eye_bins > 0,
                "sensors.eye_bins",
                "must be at least 1",
            ),
            (
                self.sensors.energy_scale > 0.0,
                "sensors.energy_scale",
                "must be positive",
            ),
            (
                self.mlp.hidden_layers.iter().all(|&n| n > 0),
                "mlp.hidden_layers",
                "sizes must be at least 1",
            ),
        ];
        match checks.into_iter().find(|(ok, _, _)| !ok) {
            Some((_, field, reason)) => Err(ConfigError::Invalid { field, reason }),
            None => Ok(()),
        }
    }
}
