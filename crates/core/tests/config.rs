use neural_survival_core::{Config, EpisodePlan};

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
        "episodes",
        "episode_steps",
        "stages",
        "[fitness.prey]",
        "survival",
        "[ga]",
        "tournament_size",
        "elitism",
        "mutation_rate",
        "mutation_sigma",
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

#[test]
fn evaluation_schedule_stages_parse_and_override_from_their_generation() {
    let config = Config::from_toml_str(
        "[evaluation]\nepisodes = 2\nepisode_steps = 100\n\n\
         [[evaluation.stages]]\nfrom_generation = 10\nepisodes = 3\nepisode_steps = 200\n\n\
         [[evaluation.stages]]\nfrom_generation = 20\nepisodes = 4\nepisode_steps = 400\n",
    )
    .unwrap();
    let plan = |generation| config.evaluation.plan(generation);
    let expect = |episodes, episode_steps| EpisodePlan {
        episodes,
        episode_steps,
    };
    assert_eq!(plan(0), expect(2, 100));
    assert_eq!(plan(9), expect(2, 100));
    assert_eq!(plan(10), expect(3, 200));
    assert_eq!(plan(19), expect(3, 200));
    assert_eq!(plan(500), expect(4, 400));
    assert_eq!(
        Config::from_toml_str(&config.to_toml_string()).unwrap(),
        config
    );
}

#[test]
fn evolution_settings_are_validated() {
    for (toml, field) in [
        (
            "[[evaluation.stages]]\nfrom_generation = 5\nepisodes = 1\nepisode_steps = 9\n\n\
             [[evaluation.stages]]\nfrom_generation = 5\nepisodes = 1\nepisode_steps = 9\n",
            "evaluation.stages",
        ),
        ("[evaluation]\nepisodes = 0\n", "evaluation.episodes"),
        (
            "[evaluation]\nepisode_steps = 0\n",
            "evaluation.episode_steps",
        ),
        (
            "[[evaluation.stages]]\nfrom_generation = 3\nepisodes = 0\nepisode_steps = 9\n",
            "evaluation.episodes",
        ),
        (
            "[[evaluation.stages]]\nfrom_generation = 3\nepisodes = 1\nepisode_steps = 0\n",
            "evaluation.episode_steps",
        ),
        ("[ga]\ntournament_size = 0\n", "ga.tournament_size"),
        ("[ga]\nmutation_rate = 1.5\n", "ga.mutation_rate"),
        ("[ga]\nmutation_sigma = -0.1\n", "ga.mutation_sigma"),
    ] {
        let err = Config::from_toml_str(toml).unwrap_err();
        assert!(err.to_string().contains(field), "{toml}: {err}");
    }
}
