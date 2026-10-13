mod common;

use std::path::PathBuf;

use common::tiny_config;
use neural_survival_core::{
    Config, Evolution, EvolutionError, MLP_PREY, SNAPSHOT_FORMAT_VERSION, Showcase, Snapshot,
    SnapshotError,
};

fn small_config() -> Config {
    let mut config = tiny_config();
    config.population.mlp_prey = 5;
    config.food.count = 12;
    config.prey.starting_energy = 4.0;
    config.energy.idle_drain = 0.1;
    config.mlp.hidden_layers = vec![3];
    config.evaluation.episodes = 2;
    config.evaluation.episode_steps = 25;
    config
}

fn temp_path(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_file(&path);
    path
}

#[test]
fn snapshot_holds_the_generation_without_world_state() {
    let config = small_config();
    let mut evolution = Evolution::new(config.clone(), 5).unwrap();
    evolution.advance(3, |_| {});
    let snapshot = evolution.snapshot();

    assert_eq!(snapshot.format_version, SNAPSHOT_FORMAT_VERSION);
    assert_eq!(snapshot.seed, 5);
    assert_eq!(snapshot.generation, 3);
    assert_eq!(snapshot.config, config);
    assert_eq!(snapshot.genomes, evolution.genomes(MLP_PREY));
    assert_eq!(snapshot.metrics, evolution.metrics());
    assert_eq!(snapshot.metrics.len(), 3);
}

#[test]
fn snapshot_round_trips_through_save_and_load() {
    let mut evolution = Evolution::new(small_config(), 6).unwrap();
    evolution.advance(2, |_| {});
    let snapshot = evolution.snapshot();
    let path = temp_path("round_trip.json");

    snapshot.save(&path).unwrap();
    let loaded = Snapshot::load(&path).unwrap();
    assert_eq!(loaded, snapshot);
}

#[test]
fn unsupported_snapshot_version_is_rejected_clearly() {
    let mut evolution = Evolution::new(small_config(), 6).unwrap();
    evolution.advance(1, |_| {});
    let mut snapshot = evolution.snapshot();
    snapshot.format_version = 999;
    let path = temp_path("future_version.json");
    snapshot.save(&path).unwrap();

    let error = Snapshot::load(&path).unwrap_err();
    assert!(matches!(
        error,
        SnapshotError::UnsupportedVersion { found: 999, .. }
    ));
    let message = error.to_string();
    assert!(message.contains("999"), "{message}");
    assert!(
        message.contains(&SNAPSHOT_FORMAT_VERSION.to_string()),
        "{message}"
    );
}

#[test]
fn loading_a_missing_snapshot_names_the_path() {
    let path = temp_path("does_not_exist.json");
    let message = Snapshot::load(&path).unwrap_err().to_string();
    assert!(message.contains("does_not_exist.json"), "{message}");
}

#[test]
fn same_seed_and_config_give_identical_genomes_and_metrics() {
    let mut a = Evolution::new(small_config(), 77).unwrap();
    let mut b = Evolution::new(small_config(), 77).unwrap();
    a.advance(4, |_| {});
    b.advance(4, |_| {});
    assert_eq!(a.genomes(MLP_PREY), b.genomes(MLP_PREY));
    assert_eq!(a.metrics(), b.metrics());

    let mut c = Evolution::new(small_config(), 78).unwrap();
    c.advance(4, |_| {});
    assert_ne!(a.genomes(MLP_PREY), c.genomes(MLP_PREY));
}

#[test]
fn continuing_a_saved_snapshot_matches_an_uninterrupted_run() {
    let mut uninterrupted = Evolution::new(small_config(), 31).unwrap();
    uninterrupted.advance(5, |_| {});

    let mut first = Evolution::new(small_config(), 31).unwrap();
    first.advance(2, |_| {});
    let path = temp_path("continuation.json");
    first.snapshot().save(&path).unwrap();
    let mut resumed = Evolution::from_snapshot(Snapshot::load(&path).unwrap()).unwrap();
    resumed.advance(3, |_| {});

    assert_eq!(resumed.generation(), 5);
    assert_eq!(resumed, uninterrupted);
}

#[test]
fn snapshot_with_wrong_population_size_is_rejected() {
    let evolution = Evolution::new(small_config(), 1).unwrap();
    let mut snapshot = evolution.snapshot();
    snapshot.genomes.pop();

    let error = Evolution::from_snapshot(snapshot).unwrap_err();
    let message = error.to_string();
    assert!(message.contains("MLP-Prey"), "{message}");
    assert!(matches!(
        error,
        EvolutionError::PopulationSize {
            expected: 5,
            actual: 4,
            ..
        }
    ));
}

#[test]
fn showcase_from_snapshot_plays_its_genomes_with_a_seed_from_run_seed_and_generation() {
    let mut evolution = Evolution::new(small_config(), 12).unwrap();
    evolution.advance(2, |_| {});
    let snapshot = evolution.snapshot();

    let showcase = Showcase::from_snapshot(&snapshot).unwrap();
    let agents = showcase.arenas()[0].agents();
    assert_eq!(agents.len(), 5);
    for (i, agent) in agents.iter().enumerate() {
        assert_eq!(showcase.genome(agent.id), Some(&snapshot.genomes[i]));
    }

    // The same Snapshot always gives the same Showcase.
    let mut a = Showcase::from_snapshot(&snapshot).unwrap();
    let mut b = Showcase::from_snapshot(&snapshot).unwrap();
    a.step_many(10);
    b.step_many(10);
    assert_eq!(a, b);

    // Another Generation or run seed lays the world out differently.
    let mut other_generation = snapshot.clone();
    other_generation.generation += 1;
    let mut other_seed = snapshot.clone();
    other_seed.seed += 1;
    for other in [other_generation, other_seed] {
        let other = Showcase::from_snapshot(&other).unwrap();
        assert_ne!(other.arenas(), showcase.arenas());
    }
}

#[test]
fn showcase_from_snapshot_lasts_as_long_as_its_generations_episodes() {
    let mut config = small_config();
    config.prey.starting_energy = 100.0;
    config.evaluation.stages = vec![neural_survival_core::EvaluationStage {
        from_generation: 1,
        episodes: 1,
        episode_steps: 9,
    }];
    let mut evolution = Evolution::new(config, 3).unwrap();
    evolution.advance(1, |_| {});

    let mut showcase = Showcase::from_snapshot(&evolution.snapshot()).unwrap();
    showcase.step_many(1000);
    assert_eq!(showcase.step_count(), 9);
}
