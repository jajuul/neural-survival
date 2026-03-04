use neural_survival::config::SimulationConfig;
use neural_survival::sim::{initialize_world, run_headless_episode, run_steps};

#[test]
fn same_seed_and_config_produce_identical_world_state() {
    let config = SimulationConfig {
        population_size: 16,
        food_count: 40,
        timestep_seconds: 0.05,
        ..SimulationConfig::default()
    };

    let mut run_a = initialize_world(&config, 42);
    let mut run_b = initialize_world(&config, 42);

    run_steps(&mut run_a, &config, 50);
    run_steps(&mut run_b, &config, 50);

    assert_eq!(run_a, run_b);
}

#[test]
fn same_seed_and_config_produce_identical_episode_summary() {
    let config = SimulationConfig {
        episode_steps: 100,
        population_size: 24,
        food_count: 60,
        ..SimulationConfig::default()
    };

    let run_a = run_headless_episode(&config, 2026);
    let run_b = run_headless_episode(&config, 2026);

    assert_eq!(run_a, run_b);
}
