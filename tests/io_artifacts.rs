use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use neural_survival::config::SimulationConfig;
use neural_survival::experiment::{GenerationMetrics, run_batch_to_dir};
use neural_survival::genetics::AgentGenome;
use neural_survival::io::snapshot::{SnapshotFile, read_snapshot, write_snapshot};
use neural_survival::sim::initialize_world;

fn temp_output_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after UNIX_EPOCH")
        .as_nanos();
    std::env::temp_dir().join(format!("neural-survival-{prefix}-{nanos}"))
}

#[test]
fn batch_run_writes_metrics_summary_and_snapshot() {
    let out_dir = temp_output_dir("batch-artifacts");
    let config = SimulationConfig {
        population_size: 10,
        food_count: 20,
        episode_steps: 20,
        ..SimulationConfig::default()
    };

    let run = run_batch_to_dir(&config, 55, 3, &out_dir).expect("batch run should succeed");

    assert_eq!(run.generations.len(), 3);

    let metrics_path = out_dir.join("metrics.csv");
    let plot_path = out_dir.join("fitness.svg");
    let summary_path = out_dir.join("run_summary.json");
    let snapshot_path = out_dir.join("snapshot_gen_0003.json");

    assert!(metrics_path.exists());
    assert!(plot_path.exists());
    assert!(summary_path.exists());
    assert!(snapshot_path.exists());

    let metrics_content = fs::read_to_string(&metrics_path).expect("metrics file must be readable");
    assert!(metrics_content.contains("generation,best_fitness,mean_fitness,survival_min"));
    let plot_content = fs::read_to_string(&plot_path).expect("plot file must be readable");
    assert!(plot_content.contains("best fitness"));
    assert!(plot_content.contains("mean fitness"));

    let snapshot = read_snapshot(&snapshot_path).expect("snapshot should deserialize");
    assert_eq!(snapshot.generation, 3);
    assert_eq!(snapshot.metrics_history.len(), 3);
    assert_eq!(snapshot.population.len(), config.population_size);
}

#[test]
fn snapshot_round_trip_preserves_payload() {
    let out_dir = temp_output_dir("snapshot-roundtrip");
    fs::create_dir_all(&out_dir).expect("temporary output directory should be creatable");

    let config = SimulationConfig {
        population_size: 4,
        food_count: 8,
        ..SimulationConfig::default()
    };

    let world = initialize_world(&config, 99);
    let snapshot = SnapshotFile {
        generation: 2,
        seed: 99,
        resolved_config: config,
        metrics_history: vec![GenerationMetrics {
            generation_index: 1,
            best_fitness: 2.0,
            mean_fitness: 1.5,
            survival_min: 5,
            survival_mean: 6.0,
            survival_max: 7,
            population_size: 4,
            controller_mlp: 2,
            controller_ltc: 2,
        }],
        population: Vec::<AgentGenome>::new(),
        world_state: world,
    };

    let path = write_snapshot(&out_dir, 2, &snapshot).expect("snapshot write should succeed");
    let loaded = read_snapshot(&path).expect("snapshot read should succeed");

    assert_eq!(loaded, snapshot);
}
