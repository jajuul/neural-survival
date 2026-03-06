use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use neural_survival::app::execute_command;
use neural_survival::cli::{BatchArgs, Commands, RunArgs, ViewArgs};

fn temp_output_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after UNIX_EPOCH")
        .as_nanos();
    std::env::temp_dir().join(format!("neural-survival-{prefix}-{nanos}"))
}

#[test]
fn batch_command_writes_expected_artifacts() {
    let out = temp_output_dir("cli-batch");

    execute_command(Commands::Batch(BatchArgs {
        config: PathBuf::from("configs/default.toml"),
        seed: 123,
        generations: 3,
        out: out.clone(),
    }))
    .expect("batch command should succeed");

    assert!(out.join("metrics.csv").exists());
    assert!(out.join("fitness.svg").exists());
    assert!(out.join("run_summary.json").exists());
    assert!(out.join("snapshot_gen_0003.json").exists());
}

#[test]
fn run_command_fast_forwards_and_creates_snapshot() {
    let out = temp_output_dir("cli-run");

    execute_command(Commands::Run(RunArgs {
        config: PathBuf::from("configs/default.toml"),
        seed: 777,
        headless_generations: 2,
        out: out.clone(),
    }))
    .expect("run command should succeed");

    assert!(out.join("fitness.svg").exists());
    assert!(out.join("snapshot_gen_0002.json").exists());
}

#[test]
fn view_command_errors_on_missing_snapshot() {
    let result = execute_command(Commands::View(ViewArgs {
        snapshot: PathBuf::from("/tmp/does-not-exist-snapshot.json"),
    }));

    assert!(result.is_err());
}
