use std::{fs, path::Path};

use thiserror::Error;

pub mod types;

pub use types::{EvaluationMode, MAX_PERCEPTION_CHANNELS, RuleToggles, SimulationConfig};

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

    if config.food_spawn_rate_per_second < 0.0 {
        return Err(ConfigError::Invalid {
            field: "food_spawn_rate_per_second",
            message: "must be >= 0".into(),
        });
    }

    if config.food_motion_speed < 0.0 {
        return Err(ConfigError::Invalid {
            field: "food_motion_speed",
            message: "must be >= 0".into(),
        });
    }

    if config.food_cluster_enabled && config.food_cluster_count == 0 {
        return Err(ConfigError::Invalid {
            field: "food_cluster_count",
            message: "must be >= 1 when food_cluster_enabled = true".into(),
        });
    }

    if config.food_cluster_radius <= 0.0 {
        return Err(ConfigError::Invalid {
            field: "food_cluster_radius",
            message: "must be > 0".into(),
        });
    }

    if config.food_cluster_motion_speed < 0.0 {
        return Err(ConfigError::Invalid {
            field: "food_cluster_motion_speed",
            message: "must be >= 0".into(),
        });
    }

    if !(0.0..=1.0).contains(&config.food_cluster_productivity_bias) {
        return Err(ConfigError::Invalid {
            field: "food_cluster_productivity_bias",
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

    if config.max_acceleration <= 0.0 {
        return Err(ConfigError::Invalid {
            field: "max_acceleration",
            message: "must be > 0".into(),
        });
    }

    if config.linear_drag < 0.0 {
        return Err(ConfigError::Invalid {
            field: "linear_drag",
            message: "must be >= 0".into(),
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

    if !(0.0..=1.0).contains(&config.predator_fraction) {
        return Err(ConfigError::Invalid {
            field: "predator_fraction",
            message: "must be in [0, 1]".into(),
        });
    }

    if config.predator_capture_radius < 0.0 {
        return Err(ConfigError::Invalid {
            field: "predator_capture_radius",
            message: "must be >= 0".into(),
        });
    }

    if config.predator_capture_energy_gain < 0.0 {
        return Err(ConfigError::Invalid {
            field: "predator_capture_energy_gain",
            message: "must be >= 0".into(),
        });
    }

    if config.sensor_noise_stddev < 0.0 {
        return Err(ConfigError::Invalid {
            field: "sensor_noise_stddev",
            message: "must be >= 0".into(),
        });
    }

    if !(0.0..=1.0).contains(&config.sensor_dropout_probability) {
        return Err(ConfigError::Invalid {
            field: "sensor_dropout_probability",
            message: "must be in [0, 1]".into(),
        });
    }

    if config.day_night_cycle_enabled && config.day_night_cycle_steps == 0 {
        return Err(ConfigError::Invalid {
            field: "day_night_cycle_steps",
            message: "must be >= 1 when day_night_cycle_enabled = true".into(),
        });
    }

    if !(0.0..=1.0).contains(&config.night_visibility_multiplier) {
        return Err(ConfigError::Invalid {
            field: "night_visibility_multiplier",
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

    if config.fitness_weight_predation < 0.0 {
        return Err(ConfigError::Invalid {
            field: "fitness_weight_predation",
            message: "must be >= 0".into(),
        });
    }

    if config.fitness_weight_energy < 0.0 {
        return Err(ConfigError::Invalid {
            field: "fitness_weight_energy",
            message: "must be >= 0".into(),
        });
    }

    if (config.fitness_weight_survival
        + config.fitness_weight_food
        + config.fitness_weight_predation
        + config.fitness_weight_energy)
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
    use super::{
        ConfigError, EvaluationMode, MAX_PERCEPTION_CHANNELS, SimulationConfig, parse_config_toml,
    };

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
        assert_eq!(parsed.food_spawn_rate_per_second, 0.0);
        assert_eq!(parsed.food_motion_speed, 1.0);
        assert!(!parsed.food_cluster_enabled);
        assert_eq!(parsed.food_cluster_count, 4);
        assert_eq!(parsed.food_cluster_radius, 28.0);
        assert_eq!(parsed.food_cluster_motion_speed, 0.8);
        assert_eq!(parsed.food_cluster_productivity_bias, 0.65);
        assert_eq!(parsed.elitism_fraction, 0.10);
        assert_eq!(parsed.tournament_size, 5);
        assert_eq!(parsed.mutation_rate, 0.15);
        assert_eq!(parsed.mlp_hidden_size, 8);
        assert_eq!(parsed.ltc_state_size, 8);
        assert_eq!(parsed.predator_fraction, 0.0);
        assert_eq!(parsed.predator_capture_radius, 1.8);
        assert_eq!(parsed.predator_capture_energy_gain, 25.0);
        assert_eq!(parsed.fitness_weight_predation, 2.0);
        assert!(parsed.day_night_cycle_enabled);
        assert_eq!(parsed.day_night_cycle_steps, 240);
        assert_eq!(parsed.night_visibility_multiplier, 0.35);
        assert_eq!(
            parsed.eye_fov_angle_radians,
            std::f32::consts::PI + std::f32::consts::FRAC_PI_4
        );
        assert_eq!(parsed.max_acceleration, 18.0);
        assert_eq!(parsed.linear_drag, 1.8);
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
    fn invalid_food_motion_speed_is_rejected() {
        let raw = r#"
            food_motion_speed = -0.1
        "#;

        let err = parse_config_toml(raw).expect_err("negative food motion speed must fail");
        assert!(matches!(
            err,
            ConfigError::Invalid {
                field: "food_motion_speed",
                ..
            }
        ));
    }

    #[test]
    fn invalid_food_spawn_rate_per_second_is_rejected() {
        let raw = r#"
            food_spawn_rate_per_second = -0.1
        "#;

        let err = parse_config_toml(raw).expect_err("negative fixed food spawn rate must fail");
        assert!(matches!(
            err,
            ConfigError::Invalid {
                field: "food_spawn_rate_per_second",
                ..
            }
        ));
    }

    #[test]
    fn invalid_food_cluster_count_is_rejected_when_enabled() {
        let raw = r#"
            food_cluster_enabled = true
            food_cluster_count = 0
        "#;

        let err =
            parse_config_toml(raw).expect_err("cluster count zero should be invalid when enabled");

        assert!(matches!(
            err,
            ConfigError::Invalid {
                field: "food_cluster_count",
                ..
            }
        ));
    }

    #[test]
    fn invalid_predator_capture_radius_is_rejected() {
        let raw = r#"
            predator_capture_radius = -0.1
        "#;

        let err = parse_config_toml(raw).expect_err("negative predator capture radius is invalid");

        assert!(matches!(
            err,
            ConfigError::Invalid {
                field: "predator_capture_radius",
                ..
            }
        ));
    }

    #[test]
    fn invalid_max_acceleration_is_rejected() {
        let raw = r#"
            max_acceleration = 0.0
        "#;

        let err = parse_config_toml(raw).expect_err("zero max_acceleration should be invalid");

        assert!(matches!(
            err,
            ConfigError::Invalid {
                field: "max_acceleration",
                ..
            }
        ));
    }

    #[test]
    fn invalid_linear_drag_is_rejected() {
        let raw = r#"
            linear_drag = -0.1
        "#;

        let err = parse_config_toml(raw).expect_err("negative linear_drag should be invalid");

        assert!(matches!(
            err,
            ConfigError::Invalid {
                field: "linear_drag",
                ..
            }
        ));
    }

    #[test]
    fn evaluation_mode_parses_from_toml() {
        let raw = r#"
            evaluation_mode = "shared"
        "#;

        let parsed = parse_config_toml(raw).expect("evaluation mode should parse");

        assert_eq!(parsed.evaluation_mode, EvaluationMode::Shared);
    }

    #[test]
    fn evaluation_schedule_parses_from_toml() {
        let raw = r#"
            evaluation_mode = "solo"
            evaluation_mode_after_switch = "shared"
            evaluation_mode_switch_after_generation = 100
        "#;

        let parsed = parse_config_toml(raw).expect("evaluation schedule should parse");

        assert_eq!(parsed.evaluation_mode, EvaluationMode::Solo);
        assert_eq!(parsed.evaluation_mode_after_switch, EvaluationMode::Shared);
        assert_eq!(parsed.evaluation_mode_switch_after_generation, 100);
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
    fn invalid_sensor_dropout_probability_is_rejected() {
        let raw = r#"
            sensor_dropout_probability = 1.1
        "#;

        let err = parse_config_toml(raw).expect_err("dropout probability above 1 must fail");
        assert!(matches!(
            err,
            ConfigError::Invalid {
                field: "sensor_dropout_probability",
                ..
            }
        ));
    }

    #[test]
    fn invalid_sensor_noise_stddev_is_rejected() {
        let raw = r#"
            sensor_noise_stddev = -0.01
        "#;

        let err = parse_config_toml(raw).expect_err("negative sensor noise stddev must fail");
        assert!(matches!(
            err,
            ConfigError::Invalid {
                field: "sensor_noise_stddev",
                ..
            }
        ));
    }

    #[test]
    fn invalid_day_night_cycle_steps_is_rejected_when_enabled() {
        let raw = r#"
            day_night_cycle_enabled = true
            day_night_cycle_steps = 0
        "#;

        let err = parse_config_toml(raw).expect_err("zero cycle length must fail when enabled");
        assert!(matches!(
            err,
            ConfigError::Invalid {
                field: "day_night_cycle_steps",
                ..
            }
        ));
    }

    #[test]
    fn invalid_night_visibility_multiplier_is_rejected() {
        let raw = r#"
            night_visibility_multiplier = 1.2
        "#;

        let err = parse_config_toml(raw).expect_err("night visibility > 1 must fail");
        assert!(matches!(
            err,
            ConfigError::Invalid {
                field: "night_visibility_multiplier",
                ..
            }
        ));
    }

    #[test]
    fn invalid_predator_fraction_is_rejected() {
        let raw = r#"
            predator_fraction = 1.2
        "#;

        let err = parse_config_toml(raw).expect_err("predator fraction > 1 must fail");
        assert!(matches!(
            err,
            ConfigError::Invalid {
                field: "predator_fraction",
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
