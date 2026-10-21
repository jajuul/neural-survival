use neural_survival_core::{
    Config, Genome, Mlp, Network, NetworkShapeError, Role, Showcase, ShowcaseError,
};

fn small_config() -> Config {
    let mut config = Config::default();
    config.world.width = 50.0;
    config.world.height = 40.0;
    config.population.mlp_prey = 4;
    config.food.count = 6;
    config.evaluation.episode_steps = 30;
    config
}

#[test]
fn showcase_starts_with_configured_prey_and_food() {
    let config = small_config();
    let showcase = Showcase::new(&config, 1).unwrap();

    assert_eq!(showcase.step_count(), 0);
    assert!(!showcase.has_ended());
    let arenas = showcase.arenas();
    assert_eq!(arenas.len(), 1);
    let arena = &arenas[0];
    assert_eq!(arena.agents().len(), 4);
    assert_eq!(arena.food().len(), 6);
    for agent in arena.agents() {
        assert!(agent.alive);
        assert_eq!(agent.energy, config.prey.starting_energy);
        assert!(agent.position.x >= 0.0 && agent.position.x < 50.0);
        assert!(agent.position.y >= 0.0 && agent.position.y < 40.0);
    }
}

#[test]
fn showcase_ends_at_step_limit_and_stops_advancing() {
    let config = small_config();
    let mut showcase = Showcase::new(&config, 1).unwrap();

    for _ in 0..29 {
        showcase.step();
    }
    assert!(!showcase.has_ended());
    showcase.step();
    assert!(showcase.has_ended());
    assert_eq!(showcase.step_count(), 30);

    let before = showcase.arenas().to_vec();
    showcase.step();
    assert_eq!(showcase.step_count(), 30);
    assert_eq!(showcase.arenas(), &before[..]);
}

#[test]
fn builder_rejects_placements_that_do_not_match_genomes() {
    let config = small_config();
    let result = Showcase::builder(&config, 1).placements(vec![]).build();
    assert!(matches!(
        result,
        Err(ShowcaseError::PlacementCount {
            genomes: 4,
            placements: 0
        })
    ));
}

#[test]
fn builder_rejects_genomes_whose_network_does_not_fit_the_config() {
    let config = small_config();
    let inputs = config.controller_input_width() + 1;
    let mlp = Mlp::from_params(vec![inputs, 2], vec![0.0; inputs * 2 + 2]).unwrap();
    let genome = Genome::new(Role::Prey, Network::Mlp(mlp));
    let result = Showcase::builder(&config, 1).genomes(vec![genome]).build();
    assert!(matches!(
        result,
        Err(ShowcaseError::NetworkShape(NetworkShapeError {
            index: 0,
            ..
        }))
    ));
}

#[test]
fn showcase_rejects_an_invalid_config() {
    let mut config = small_config();
    config.sensors.eye_bins = 0;
    assert!(matches!(
        Showcase::new(&config, 1),
        Err(ShowcaseError::Config(_))
    ));
}

#[test]
fn showcase_exposes_the_config_it_was_built_with() {
    let mut config = small_config();
    config.agent.body_radius = 1.25;
    config.food.radius = 0.75;
    let showcase = Showcase::new(&config, 1).unwrap();

    assert_eq!(showcase.config(), &config);
}
