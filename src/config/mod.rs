use std::{fs, path::Path};

use thiserror::Error;

pub mod types;

pub use types::{RuleToggles, SimulationConfig};

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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ConfigError, SimulationConfig, parse_config_toml};

    #[test]
    fn empty_toml_uses_spec_defaults() {
        let parsed = parse_config_toml("").expect("empty TOML should fall back to defaults");

        assert_eq!(parsed, SimulationConfig::default());
        assert_eq!(parsed.world_width, 200.0);
        assert_eq!(parsed.world_height, 200.0);
        assert_eq!(parsed.timestep_seconds, 0.1);
        assert_eq!(parsed.episode_steps, 300);
        assert_eq!(parsed.population_size, 128);
        assert_eq!(parsed.food_count, 400);
        assert_eq!(parsed.elitism_fraction, 0.10);
        assert_eq!(parsed.tournament_size, 5);
        assert_eq!(parsed.mutation_rate, 0.15);
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
}
