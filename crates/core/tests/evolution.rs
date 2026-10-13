mod common;

use common::{assert_close, tiny_config};
use neural_survival_core::{Config, ControllerFamily, EvaluationStage, Evolution, MLP_PREY, Role};

fn small_config() -> Config {
    let mut config = tiny_config();
    config.population.mlp_prey = 6;
    config.food.count = 10;
    config.prey.starting_energy = 5.0;
    config.energy.idle_drain = 0.1;
    config.mlp.hidden_layers = vec![3];
    config.evaluation.episode_steps = 30;
    config
}

#[test]
fn evolution_starts_at_generation_zero_with_configured_population() {
    let config = small_config();
    let evolution = Evolution::new(config, 42).unwrap();

    assert_eq!(evolution.generation(), 0);
    assert_eq!(evolution.seed(), 42);
    assert_eq!(evolution.genomes(MLP_PREY).len(), 6);
    assert!(evolution.metrics().is_empty());
    for genome in evolution.genomes(MLP_PREY) {
        assert_eq!(genome.role, Role::Prey);
        assert_eq!(genome.family(), ControllerFamily::Mlp);
    }
}

#[test]
fn species_population_sizes_and_identity_stay_fixed_across_generations() {
    let config = small_config();
    let mut evolution = Evolution::new(config, 42).unwrap();
    let before = evolution.genomes(MLP_PREY).to_vec();

    evolution.advance(3, |_| {});

    assert_eq!(evolution.generation(), 3);
    let after = evolution.genomes(MLP_PREY);
    assert_eq!(after.len(), 6);
    assert_ne!(after, &before[..]);
    for genome in after {
        assert_eq!(genome.role, Role::Prey);
        assert_eq!(genome.family(), ControllerFamily::Mlp);
    }
}

/// Every Agent touches every food item each step: the body reaches across
/// the whole 10×10 torus, so each step each Prey eats all food.
fn feast_config() -> Config {
    let mut config = small_config();
    config.world.width = 10.0;
    config.world.height = 10.0;
    config.agent.body_radius = 8.0;
    config.population.mlp_prey = 3;
    config.food.count = 2;
    config.food.energy = 5.0;
    config.prey.starting_energy = 100.0;
    config.evaluation.episode_steps = 5;
    config.evaluation.episodes = 2;
    config.fitness.prey.survival = 2.0;
    config.fitness.prey.energy = 0.5;
    config
}

#[test]
fn fitness_is_weighted_steps_alive_plus_energy_gained() {
    let mut evolution = Evolution::new(feast_config(), 1).unwrap();
    evolution.advance(1, |_| {});

    // 5 steps alive, 2 food × 5 energy × 5 steps = 50 gained: 2·5 + 0.5·50.
    let row = &evolution.metrics()[0];
    assert_eq!(row.generation, 0);
    assert_eq!(row.species, MLP_PREY);
    assert_close(row.best_fitness, 35.0);
    assert_close(row.mean_fitness, 35.0);
    assert_close(row.alive_at_end, 3.0);
    // 3 Prey × 2 food × 5 steps per Episode.
    assert_close(row.food_eaten, 30.0);
}

#[test]
fn steps_alive_stop_counting_at_death() {
    let mut config = small_config();
    config.food.count = 0;
    config.prey.starting_energy = 1.0;
    config.energy.idle_drain = 0.3;
    config.fitness.prey.survival = 2.0;
    let mut evolution = Evolution::new(config, 1).unwrap();
    evolution.advance(1, |_| {});

    // Energy 0.7, 0.4, 0.1 after three steps; the fourth kills.
    let row = &evolution.metrics()[0];
    assert_close(row.best_fitness, 6.0);
    assert_close(row.mean_fitness, 6.0);
    assert_close(row.alive_at_end, 0.0);
}

fn immortal_config() -> Config {
    let mut config = small_config();
    config.food.count = 0;
    config.prey.starting_energy = 100.0;
    config.energy.idle_drain = 0.01;
    config.fitness.prey.survival = 1.0;
    config.fitness.prey.energy = 0.0;
    config
}

#[test]
fn evaluation_schedule_sets_episode_count_and_length_per_generation() {
    let mut config = immortal_config();
    config.evaluation.episodes = 1;
    config.evaluation.episode_steps = 4;
    config.evaluation.stages = vec![EvaluationStage {
        from_generation: 2,
        episodes: 3,
        episode_steps: 7,
    }];
    let mut evolution = Evolution::new(config, 9).unwrap();
    evolution.advance(3, |_| {});

    let rows: Vec<_> = evolution
        .metrics()
        .iter()
        .map(|r| (r.generation, r.episodes, r.episode_steps, r.best_fitness))
        .collect();
    assert_eq!(rows, [(0, 1, 4, 4.0), (1, 1, 4, 4.0), (2, 3, 7, 7.0)]);
}

#[test]
fn progress_fires_at_every_tenth_generation_and_on_completion() {
    let mut config = small_config();
    config.evaluation.episode_steps = 2;
    let mut evolution = Evolution::new(config, 3).unwrap();
    let mut seen = Vec::new();
    evolution.advance(25, |p| {
        assert_eq!(p.latest.len(), 1);
        assert_eq!(p.latest[0].generation + 1, p.generation);
        seen.push((p.completed, p.total, p.generation));
    });
    assert_eq!(seen, [(10, 25, 10), (20, 25, 20), (25, 25, 25)]);

    // Continuing from Generation 25 reports at Generations 30 and 40, then 45.
    seen.clear();
    evolution.advance(20, |p| seen.push((p.completed, p.total, p.generation)));
    assert_eq!(seen, [(5, 20, 30), (15, 20, 40), (20, 20, 45)]);
}

#[test]
fn metrics_history_has_one_row_per_species_per_generation() {
    let mut evolution = Evolution::new(small_config(), 4).unwrap();
    evolution.advance(4, |_| {});
    let generations: Vec<u32> = evolution.metrics().iter().map(|r| r.generation).collect();
    assert_eq!(generations, [0, 1, 2, 3]);
    assert!(evolution.metrics().iter().all(|r| r.species == MLP_PREY));
}
