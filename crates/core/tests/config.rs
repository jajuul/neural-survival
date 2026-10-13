use neural_survival_core::Config;

#[test]
fn empty_file_yields_fully_defaulted_config() {
    let config = Config::from_toml_str("").unwrap();
    assert_eq!(config, Config::default());
}

#[test]
fn partial_file_overrides_only_given_fields() {
    let config = Config::from_toml_str("[world]\nwidth = 12.5\n").unwrap();
    assert_eq!(config.world.width, 12.5);
    assert_eq!(config.world.height, Config::default().world.height);
    assert_eq!(config.food, Config::default().food);
}

#[test]
fn unknown_top_level_key_is_an_error_naming_the_key() {
    let err = Config::from_toml_str("wrold_size = 3\n").unwrap_err();
    assert!(err.to_string().contains("wrold_size"), "{err}");
}

#[test]
fn unknown_nested_key_is_an_error_naming_the_key() {
    let err = Config::from_toml_str("[food]\nenergyy = 3.0\n").unwrap_err();
    assert!(err.to_string().contains("energyy"), "{err}");
}

#[test]
fn resolved_config_serialises_back_out_and_round_trips() {
    let mut config = Config::default();
    config.world.width = 33.0;
    config.mlp.hidden_layers = vec![4, 3];
    config.sensors.eye_bins = 7;

    let text = config.to_toml_string();
    assert!(text.contains("hidden_layers"), "{text}");
    assert_eq!(Config::from_toml_str(&text).unwrap(), config);
}

#[test]
fn defaults_serialise_every_field() {
    let text = Config::default().to_toml_string();
    for key in [
        "[world]",
        "width",
        "height",
        "drag",
        "[food]",
        "count",
        "energy",
        "radius",
        "[population]",
        "mlp_prey",
        "[prey]",
        "starting_energy",
        "[agent]",
        "max_speed",
        "max_acceleration",
        "turn_rate",
        "body_radius",
        "vision_range",
        "field_of_view",
        "[energy]",
        "idle_drain",
        "[sensors]",
        "eye_bins",
        "energy_scale",
        "[mlp]",
        "hidden_layers",
        "[evaluation]",
        "episode_steps",
    ] {
        assert!(text.contains(key), "missing {key} in:\n{text}");
    }
}

#[test]
fn invalid_values_are_rejected_naming_the_field() {
    let err = Config::from_toml_str("[sensors]\neye_bins = 0\n").unwrap_err();
    assert!(err.to_string().contains("sensors.eye_bins"), "{err}");

    let err = Config::from_toml_str("[world]\nwidth = -1.0\n").unwrap_err();
    assert!(err.to_string().contains("world.width"), "{err}");
}
