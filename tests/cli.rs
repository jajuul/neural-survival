//! Acceptance tests: the CLI binary run as a process.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

/// A tiny, fast world so a few Generations take well under a second.
const TINY_CONFIG: &str = r#"
[world]
width = 40.0
height = 40.0

[food]
count = 5

[population]
mlp_prey = 4

[mlp]
hidden_layers = []

[evaluation]
episodes = 1
episode_steps = 20

[ga]
tournament_size = 2
elitism = 1
"#;

struct Workspace {
    dir: TempDir,
}

impl Workspace {
    fn new() -> Self {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("config.toml"), TINY_CONFIG).unwrap();
        Self { dir }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.path().join(name)
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_neural-survival"))
            .current_dir(self.dir.path())
            .args(args)
            .output()
            .unwrap()
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        stdout(output),
        stderr(output)
    );
}

fn file_names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

const BATCH_42: &[&str] = &[
    "batch",
    "--config",
    "config.toml",
    "--seed",
    "42",
    "--generations",
    "10",
    "--out",
    "results",
];

/// Long enough to pass two progress intervals.
const BATCH_25: &[&str] = &[
    "batch",
    "--config",
    "config.toml",
    "--seed",
    "1",
    "--generations",
    "25",
    "--out",
    "results",
];

#[test]
fn a_headless_batch_run_writes_the_required_artifacts() {
    let workspace = Workspace::new();

    let output = workspace.run(BATCH_42);

    assert_success(&output);
    let results = workspace.path("results");
    for name in [
        "metrics.csv",
        "fitness.svg",
        "run_summary.json",
        "snapshot_gen_10.json",
    ] {
        assert!(
            results.join(name).is_file(),
            "missing {name}; found {:?}",
            file_names(&results)
        );
    }
}

#[test]
fn a_batch_run_rejects_a_non_empty_output_directory() {
    let workspace = Workspace::new();
    std::fs::create_dir(workspace.path("results")).unwrap();
    std::fs::write(workspace.path("results/notes.txt"), "keep me").unwrap();

    let output = workspace.run(BATCH_42);

    assert!(!output.status.success(), "batch should fail");
    assert!(
        stderr(&output).contains("must be absent or empty"),
        "stderr: {}",
        stderr(&output)
    );
    assert_eq!(file_names(&workspace.path("results")), ["notes.txt"]);
}

#[test]
fn a_batch_run_accepts_an_existing_empty_output_directory() {
    let workspace = Workspace::new();
    std::fs::create_dir(workspace.path("results")).unwrap();

    let output = workspace.run(BATCH_42);

    assert_success(&output);
    assert!(workspace.path("results/metrics.csv").is_file());
}

#[test]
fn an_omitted_seed_is_generated_and_printed() {
    let workspace = Workspace::new();

    let output = workspace.run(&[
        "batch",
        "--config",
        "config.toml",
        "--generations",
        "1",
        "--out",
        "results",
    ]);

    assert_success(&output);
    let printed: u64 = stdout(&output)
        .lines()
        .find_map(|line| line.strip_prefix("Generated seed: "))
        .expect("the seed is printed")
        .trim()
        .parse()
        .unwrap();
    let summary = std::fs::read_to_string(workspace.path("results/run_summary.json")).unwrap();
    assert!(
        summary.contains(&format!("\"seed\": {printed}")),
        "summary: {summary}"
    );
}

#[test]
fn a_given_seed_is_not_reported_as_generated() {
    let workspace = Workspace::new();

    let output = workspace.run(BATCH_42);

    assert_success(&output);
    assert!(!stdout(&output).contains("Generated seed"));
}

#[test]
fn progress_is_printed_every_10_generations_and_on_completion() {
    let workspace = Workspace::new();

    let output = workspace.run(BATCH_25);

    assert_success(&output);
    let stdout = stdout(&output);
    let progress: Vec<&str> = stdout
        .lines()
        .filter_map(|line| line.strip_prefix("Generation "))
        .map(|rest| rest.split(':').next().unwrap())
        .collect();
    assert_eq!(progress, ["10 (10/25)", "20 (20/25)", "25 (25/25)"]);
}

#[test]
fn snapshots_are_written_every_10_generations_and_for_the_final_generation() {
    let workspace = Workspace::new();

    let output = workspace.run(BATCH_25);

    assert_success(&output);
    let snapshots: Vec<String> = file_names(&workspace.path("results"))
        .into_iter()
        .filter(|name| name.starts_with("snapshot_gen_"))
        .collect();
    assert_eq!(
        snapshots,
        [
            "snapshot_gen_10.json",
            "snapshot_gen_20.json",
            "snapshot_gen_25.json"
        ]
    );
}

#[test]
fn an_invalid_config_fails_with_its_error() {
    let workspace = Workspace::new();
    std::fs::write(workspace.path("config.toml"), "[world]\nwidht = 3.0\n").unwrap();

    let output = workspace.run(BATCH_42);

    assert!(!output.status.success(), "batch should fail");
    assert!(
        stderr(&output).contains("widht"),
        "stderr: {}",
        stderr(&output)
    );
    assert!(!workspace.path("results").exists());
}
