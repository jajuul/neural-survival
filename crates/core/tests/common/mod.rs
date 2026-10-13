#![allow(dead_code)]

use neural_survival_core::{Config, Genome, Mlp, Network, Role};

/// A tiny world with no hidden layers, so a Genome's outputs are set by its
/// output biases alone.
pub fn tiny_config() -> Config {
    let mut config = Config::default();
    config.world.width = 50.0;
    config.world.height = 40.0;
    config.world.drag = 0.1;
    config.agent.max_speed = 2.0;
    config.agent.max_acceleration = 0.5;
    config.agent.turn_rate = 0.3;
    config.energy.idle_drain = 0.01;
    config.food.count = 0;
    config.mlp.hidden_layers = vec![];
    config.evaluation.episode_steps = 1000;
    config
}

/// A Prey whose controller ignores its inputs and always outputs the given
/// raw steering and throttle. Requires `config.mlp.hidden_layers` to be empty.
pub fn constant_prey(config: &Config, steering: f64, throttle: f64) -> Genome {
    assert!(config.mlp.hidden_layers.is_empty());
    let inputs = config.controller_input_width();
    // One layer: 2×inputs weights (row-major), then 2 biases.
    let mut params = vec![0.0; 2 * inputs];
    params.push(steering.atanh());
    params.push(throttle.atanh());
    let mlp = Mlp::from_params(vec![inputs, 2], params).unwrap();
    Genome::new(Role::Prey, Network::Mlp(mlp))
}

pub fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {expected}, got {actual}"
    );
}

/// A Prey whose raw steering is `tanh(input[input_index])` and whose throttle
/// is a constant. Requires `config.mlp.hidden_layers` to be empty.
pub fn probe_prey(config: &Config, input_index: usize, throttle: f64) -> Genome {
    assert!(config.mlp.hidden_layers.is_empty());
    let inputs = config.controller_input_width();
    let mut params = vec![0.0; 2 * inputs + 2];
    params[input_index] = 1.0;
    params[2 * inputs + 1] = throttle.atanh();
    let mlp = Mlp::from_params(vec![inputs, 2], params).unwrap();
    Genome::new(Role::Prey, Network::Mlp(mlp))
}
