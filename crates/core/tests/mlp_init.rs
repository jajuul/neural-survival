use neural_survival_core::{AgentId, Config, ControllerFamily, Network, Role, Showcase};

#[test]
fn random_prey_get_mlps_shaped_by_config_with_xavier_weights() {
    let mut config = Config::default();
    config.population.mlp_prey = 3;
    config.sensors.eye_bins = 4;
    config.mlp.hidden_layers = vec![30, 20];
    let showcase = Showcase::new(&config, 77).unwrap();

    for id in 0..3 {
        let genome = showcase.genome(AgentId(id)).unwrap();
        assert_eq!(genome.role, Role::Prey);
        assert_eq!(genome.family(), ControllerFamily::Mlp);
        let Network::Mlp(mlp) = &genome.network;
        assert_eq!(mlp.layer_sizes(), &[6, 30, 20, 2]);

        let mut offset = 0;
        for pair in mlp.layer_sizes().windows(2) {
            let (fan_in, fan_out) = (pair[0], pair[1]);
            let bound = (6.0 / (fan_in + fan_out) as f64).sqrt();
            let weights = &mlp.params()[offset..offset + fan_in * fan_out];
            let biases =
                &mlp.params()[offset + fan_in * fan_out..offset + fan_in * fan_out + fan_out];
            assert!(
                weights.iter().all(|w| w.abs() <= bound),
                "weight beyond ±{bound}"
            );
            // Uniform over the bound: the largest of many draws lands near it.
            let largest = weights.iter().fold(0.0_f64, |m, w| m.max(w.abs()));
            assert!(
                largest > 0.8 * bound,
                "weights not spread to ±{bound}: {largest}"
            );
            assert!(biases.iter().all(|&b| b == 0.0));
            offset += fan_in * fan_out + fan_out;
        }
        assert_eq!(offset, mlp.params().len());
    }
    assert!(showcase.genome(AgentId(3)).is_none());
}

#[test]
fn different_prey_get_different_initial_weights() {
    let config = Config::default();
    let showcase = Showcase::new(&config, 77).unwrap();
    assert_ne!(showcase.genome(AgentId(0)), showcase.genome(AgentId(1)));
}

#[test]
fn controller_outputs_stay_within_tanh_range() {
    let mut config = Config::default();
    config.population.mlp_prey = 10;
    config.evaluation.episode_steps = 50;
    let mut showcase = Showcase::new(&config, 5).unwrap();
    while !showcase.has_ended() {
        showcase.step();
        for agent in showcase.arenas()[0].agents().iter().filter(|a| a.alive) {
            let raw = showcase.telemetry(agent.id).unwrap().raw_outputs;
            assert!(raw.steering.abs() <= 1.0 && raw.throttle.abs() <= 1.0);
        }
    }
}
