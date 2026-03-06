use std::{fs, path::Path};

use thiserror::Error;

pub mod types;

pub use types::{MAX_PERCEPTION_CHANNELS, RuleToggles, SimulationConfig};

/// Errors returned while loading or validating configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse config TOML: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("invalid config field `{field}`: {message}")]
    Invalid {
        field: &'static str,
        message: String,
    },
}

/// Load [`SimulationConfig`] from a TOML file path and validate field constraints.
pub fn load_config(path: &Path) -> Result<SimulationConfig, ConfigError> {
    let raw = fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.display().to_string(),
        source,
    })?;

    parse_config_toml(&raw)
}

/// Parse [`SimulationConfig`] from TOML text and validate field constraints.
pub fn parse_config_toml(raw: &str) -> Result<SimulationConfig, ConfigError> {
    let config: SimulationConfig = toml::from_str(raw)?;
    validate_config(&config)?;
    Ok(config)
}

/// Validate runtime-safe ranges for configuration fields.
pub fn validate_config(config: &SimulationConfig) -> Result<(), ConfigError> {
    if config.world_width <= 0.0 {
        return Err(ConfigError::Invalid {
            field: "world_width",
            message: "must be > 0".into(),
        });
    }

    if config.world_height <= 0.0 {
        return Err(ConfigError::Invalid {
            field: "world_height",
            message: "must be > 0".into(),
        });
    }

    if config.timestep_seconds <= 0.0 {
        return Err(ConfigError::Invalid {
            field: "timestep_seconds",
            message: "must be > 0".into(),
        });
    }

    if config.episode_steps == 0 {
        return Err(ConfigError::Invalid {
            field: "episode_steps",
            message: "must be >= 1".into(),
        });
    }

    if config.population_size == 0 {
        return Err(ConfigError::Invalid {
            field: "population_size",
            message: "must be >= 1".into(),
        });
    }

    if config.food_count == 0 {
        return Err(ConfigError::Invalid {
            field: "food_count",
            message: "must be >= 1".into(),
        });
    }

    if !(0.0..=1.0).contains(&config.food_respawn_probability) {
        return Err(ConfigError::Invalid {
            field: "food_respawn_probability",
            message: "must be in [0, 1]".into(),
        });
    }

    if !(0.0..=1.0).contains(&config.elitism_fraction) || config.elitism_fraction == 0.0 {
        return Err(ConfigError::Invalid {
            field: "elitism_fraction",
            message: "must be in (0, 1]".into(),
        });
    }

    if config.tournament_size < 2 {
        return Err(ConfigError::Invalid {
            field: "tournament_size",
            message: "must be >= 2".into(),
        });
    }

    if config.tournament_size > config.population_size {
        return Err(ConfigError::Invalid {
            field: "tournament_size",
            message: "must be <= population_size".into(),
        });
    }

    if !(0.0..=1.0).contains(&config.mutation_rate) {
        return Err(ConfigError::Invalid {
            field: "mutation_rate",
            message: "must be in [0, 1]".into(),
        });
    }

    if config.initial_agent_energy <= 0.0 {
        return Err(ConfigError::Invalid {
            field: "initial_agent_energy",
            message: "must be > 0".into(),
        });
    }

    if config.base_energy_cost_per_second < 0.0 {
        return Err(ConfigError::Invalid {
            field: "base_energy_cost_per_second",
            message: "must be >= 0".into(),
        });
    }

    if config.speed_energy_cost_factor < 0.0 {
        return Err(ConfigError::Invalid {
            field: "speed_energy_cost_factor",
            message: "must be >= 0".into(),
        });
    }

    if config.network_energy_cost_factor < 0.0 {
        return Err(ConfigError::Invalid {
            field: "network_energy_cost_factor",
            message: "must be >= 0".into(),
        });
    }

    if config.perception_energy_cost_factor < 0.0 {
        return Err(ConfigError::Invalid {
            field: "perception_energy_cost_factor",
            message: "must be >= 0".into(),
        });
    }

    if config.food_energy_gain <= 0.0 {
        return Err(ConfigError::Invalid {
            field: "food_energy_gain",
            message: "must be > 0".into(),
        });
    }

    if config.food_collision_radius <= 0.0 {
        return Err(ConfigError::Invalid {
            field: "food_collision_radius",
            message: "must be > 0".into(),
        });
    }

    if config.eye_fov_angle_radians <= 0.0 || config.eye_fov_angle_radians > std::f32::consts::TAU {
        return Err(ConfigError::Invalid {
            field: "eye_fov_angle_radians",
            message: "must be in (0, 2π]".into(),
        });
    }

    if config.default_agent_max_speed <= 0.0 {
        return Err(ConfigError::Invalid {
            field: "default_agent_max_speed",
            message: "must be > 0".into(),
        });
    }

    if config.max_turn_rate_radians <= 0.0 {
        return Err(ConfigError::Invalid {
            field: "max_turn_rate_radians",
            message: "must be > 0".into(),
        });
    }

    if config.default_perception_radius <= 0.0 {
        return Err(ConfigError::Invalid {
            field: "default_perception_radius",
            message: "must be > 0".into(),
        });
    }

    if config.default_perception_channels == 0 {
        return Err(ConfigError::Invalid {
            field: "default_perception_channels",
            message: "must be >= 1".into(),
        });
    }

    if config.default_perception_channels > MAX_PERCEPTION_CHANNELS {
        return Err(ConfigError::Invalid {
            field: "default_perception_channels",
            message: format!(
                "must be <= MAX_PERCEPTION_CHANNELS ({})",
                MAX_PERCEPTION_CHANNELS
            ),
        });
    }

    if config.default_network_size == 0 {
        return Err(ConfigError::Invalid {
            field: "default_network_size",
            message: "must be >= 1".into(),
        });
    }

    if config.mlp_hidden_size == 0 {
        return Err(ConfigError::Invalid {
            field: "mlp_hidden_size",
            message: "must be >= 1".into(),
        });
    }

    if config.ltc_state_size == 0 {
        return Err(ConfigError::Invalid {
            field: "ltc_state_size",
            message: "must be >= 1".into(),
        });
    }

    if config.default_metabolic_efficiency <= 0.0 {
        return Err(ConfigError::Invalid {
            field: "default_metabolic_efficiency",
            message: "must be > 0".into(),
        });
    }

    if !(0.0..=1.0).contains(&config.controller_ltc_probability) {
        return Err(ConfigError::Invalid {
            field: "controller_ltc_probability",
            message: "must be in [0, 1]".into(),
        });
    }

    if config.fitness_weight_survival < 0.0 {
        return Err(ConfigError::Invalid {
            field: "fitness_weight_survival",
            message: "must be >= 0".into(),
        });
    }

    if config.fitness_weight_food < 0.0 {
        return Err(ConfigError::Invalid {
            field: "fitness_weight_food",
            message: "must be >= 0".into(),
        });
    }

    if config.fitness_weight_energy < 0.0 {
        return Err(ConfigError::Invalid {
            field: "fitness_weight_energy",
            message: "must be >= 0".into(),
        });
    }

    if (config.fitness_weight_survival + config.fitness_weight_food + config.fitness_weight_energy)
        <= 0.0
    {
        return Err(ConfigError::Invalid {
            field: "fitness_weight_*",
            message: "at least one fitness weight must be > 0".into(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ConfigError, MAX_PERCEPTION_CHANNELS, SimulationConfig, parse_config_toml};

    #[test]
    fn empty_toml_uses_spec_defaults() {
        let parsed = parse_config_toml("").expect("empty TOML should fall back to defaults");

        assert_eq!(parsed, SimulationConfig::default());
        assert_eq!(parsed.world_width, 300.0);
        assert_eq!(parsed.world_height, 300.0);
        assert_eq!(parsed.timestep_seconds, 0.1);
        assert_eq!(parsed.episode_steps, 300);
        assert_eq!(parsed.population_size, 128);
        assert_eq!(parsed.food_count, 400);
        assert_eq!(parsed.food_respawn_probability, 1.0);
        assert_eq!(parsed.elitism_fraction, 0.10);
        assert_eq!(parsed.tournament_size, 5);
        assert_eq!(parsed.mutation_rate, 0.15);
        assert_eq!(parsed.mlp_hidden_size, 8);
        assert_eq!(parsed.ltc_state_size, 8);
        assert_eq!(
            parsed.eye_fov_angle_radians,
            std::f32::consts::PI + std::f32::consts::FRAC_PI_4
        );
    }

    #[test]
    fn partial_toml_overrides_default_fields() {
        let raw = r#"
            world_width = 512.0
            mutation_rate = 0.2
        "#;

        let parsed = parse_config_toml(raw).expect("partial TOML should parse");

        assert_eq!(parsed.world_width, 512.0);
        assert_eq!(parsed.mutation_rate, 0.2);
        assert_eq!(parsed.population_size, 128);
    }

    #[test]
    fn invalid_tournament_size_is_rejected() {
        let raw = r#"
            population_size = 4
            tournament_size = 8
        "#;

        let err = parse_config_toml(raw).expect_err("invalid tournament size should fail");

        match err {
            ConfigError::Invalid { field, .. } => assert_eq!(field, "tournament_size"),
            _ => panic!("expected ConfigError::Invalid"),
        }
    }

    #[test]
    fn invalid_food_respawn_probability_is_rejected() {
        let raw = r#"
            food_respawn_probability = 1.5
        "#;

        let err = parse_config_toml(raw).expect_err("respawn probability above 1 must fail");
        assert!(matches!(
            err,
            ConfigError::Invalid {
                field: "food_respawn_probability",
                ..
            }
        ));
    }

    #[test]
    fn invalid_default_perception_channels_above_max_is_rejected() {
        let raw = format!(
            "
            default_perception_channels = {}
        ",
            MAX_PERCEPTION_CHANNELS + 1
        );

        let err = parse_config_toml(&raw).expect_err("too many perception channels should fail");

        match err {
            ConfigError::Invalid { field, .. } => assert_eq!(field, "default_perception_channels"),
            _ => panic!("expected ConfigError::Invalid"),
        }
    }

    #[test]
    fn invalid_hidden_layer_size_is_rejected() {
        let raw = r#"
            mlp_hidden_size = 0
        "#;

        let err = parse_config_toml(raw).expect_err("zero hidden size should fail");

        match err {
            ConfigError::Invalid { field, .. } => assert_eq!(field, "mlp_hidden_size"),
            _ => panic!("expected ConfigError::Invalid"),
        }
    }

    #[test]
    fn invalid_ltc_state_size_is_rejected() {
        let raw = r#"
            ltc_state_size = 0
        "#;

        let err = parse_config_toml(raw).expect_err("zero ltc state size should fail");

        match err {
            ConfigError::Invalid { field, .. } => assert_eq!(field, "ltc_state_size"),
            _ => panic!("expected ConfigError::Invalid"),
        }
    }
}
