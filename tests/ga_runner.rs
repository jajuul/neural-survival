use neural_survival::config::SimulationConfig;
use neural_survival::experiment::run_generations;

#[test]
fn ga_run_produces_non_empty_generation_metrics() {
    let config = SimulationConfig {
        population_size: 16,
        food_count: 40,
        episode_steps: 35,
        ..SimulationConfig::default()
    };

    let run = run_generations(&config, 101, 6);

    assert_eq!(run.generations.len(), 6);
    assert!(run.generations.iter().any(|entry| entry.best_fitness > 0.0));
    assert!(
        run.generations
            .windows(2)
            .any(|pair| (pair[0].mean_fitness - pair[1].mean_fitness).abs() > f32::EPSILON)
    );
}

#[test]
fn ga_run_is_repeatable_for_same_seed() {
    let config = SimulationConfig {
        population_size: 12,
        food_count: 30,
        episode_steps: 25,
        ..SimulationConfig::default()
    };

    let run_a = run_generations(&config, 2025, 4);
    let run_b = run_generations(&config, 2025, 4);

    assert_eq!(run_a.generations, run_b.generations);
    assert_eq!(run_a.final_population, run_b.final_population);
}
