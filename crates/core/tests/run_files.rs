mod common;

use std::path::{Path, PathBuf};

use common::tiny_config;
use neural_survival_core::{
    Config, Evolution, MetricsRow, RunFilesError, Snapshot, prepare_output_dir, record_evolution,
    snapshot_file_name, write_run_files,
};

fn small_config() -> Config {
    let mut config = tiny_config();
    config.population.mlp_prey = 4;
    config.food.count = 8;
    config.prey.starting_energy = 4.0;
    config.energy.idle_drain = 0.1;
    config.evaluation.episode_steps = 20;
    config.ga.tournament_size = 2;
    config.ga.elitism = 1;
    config
}

/// A fresh, absent directory for one test.
fn temp_dir(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("run_files")
        .join(name);
    let _ = std::fs::remove_dir_all(&path);
    path
}

fn file_names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn snapshot_names(dir: &Path) -> Vec<String> {
    file_names(dir)
        .into_iter()
        .filter(|name| name.starts_with("snapshot_gen_"))
        .collect()
}

fn read(dir: &Path, name: &str) -> String {
    std::fs::read_to_string(dir.join(name)).unwrap()
}

#[test]
fn snapshot_files_are_named_by_generation() {
    assert_eq!(snapshot_file_name(0), "snapshot_gen_0.json");
    assert_eq!(snapshot_file_name(120), "snapshot_gen_120.json");
}

#[test]
fn an_absent_output_directory_is_created() {
    let dir = temp_dir("absent").join("nested");

    prepare_output_dir(&dir).unwrap();

    assert!(dir.is_dir());
}

#[test]
fn an_empty_output_directory_is_accepted() {
    let dir = temp_dir("empty");
    std::fs::create_dir_all(&dir).unwrap();

    prepare_output_dir(&dir).unwrap();
}

#[test]
fn a_non_empty_output_directory_is_refused_and_left_alone() {
    let dir = temp_dir("non_empty");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("notes.txt"), "keep").unwrap();

    let error = prepare_output_dir(&dir).unwrap_err();

    assert!(matches!(error, RunFilesError::OutputDirNotEmpty { .. }));
    assert!(error.to_string().contains("must be absent or empty"));
    assert_eq!(file_names(&dir), ["notes.txt"]);
}

#[test]
fn metrics_csv_has_one_row_per_generation_per_species() {
    let mut evolution = Evolution::new(small_config(), 3).unwrap();
    evolution.advance(3, |_| {});
    let dir = temp_dir("metrics_csv");
    prepare_output_dir(&dir).unwrap();

    write_run_files(&dir, evolution.config(), 3, evolution.metrics()).unwrap();

    let csv = read(&dir, "metrics.csv");
    let mut lines = csv.lines();
    assert_eq!(
        lines.next().unwrap(),
        "generation,species,episodes,episode_steps,best_fitness,mean_fitness,alive_at_end,food_eaten"
    );
    let rows: Vec<Vec<&str>> = lines.map(|line| line.split(',').collect()).collect();
    assert_eq!(rows.len(), 3);
    for (row, expected) in rows.iter().zip(evolution.metrics()) {
        assert_eq!(row[0], expected.generation.to_string());
        assert_eq!(row[1], "MLP-Prey");
        assert_eq!(row[2], "1");
        assert_eq!(row[3], "20");
        let numbers: Vec<f64> = row[4..]
            .iter()
            .map(|value| value.parse().unwrap())
            .collect();
        assert_eq!(
            numbers,
            [
                expected.best_fitness,
                expected.mean_fitness,
                expected.alive_at_end,
                expected.food_eaten
            ]
        );
    }
    assert_eq!(
        rows.iter().map(|row| row[0]).collect::<Vec<_>>(),
        ["0", "1", "2"]
    );
}

#[test]
fn run_summary_records_the_resolved_config_seed_and_final_generation() {
    let config = small_config();
    let mut evolution = Evolution::new(config.clone(), 11).unwrap();
    evolution.advance(4, |_| {});
    let dir = temp_dir("run_summary");
    prepare_output_dir(&dir).unwrap();

    write_run_files(&dir, &config, 11, evolution.metrics()).unwrap();

    let summary: serde_json::Value = serde_json::from_str(&read(&dir, "run_summary.json")).unwrap();
    assert_eq!(summary["seed"], 11);
    assert_eq!(summary["generations"], 4);
    let recorded: Config = serde_json::from_value(summary["config"].clone()).unwrap();
    assert_eq!(recorded, config);
    let final_metrics: Vec<MetricsRow> =
        serde_json::from_value(summary["final_metrics"].clone()).unwrap();
    assert_eq!(final_metrics, evolution.metrics()[3..]);
}

#[test]
fn run_summary_of_an_unevaluated_run_has_no_generations() {
    let dir = temp_dir("run_summary_empty");
    prepare_output_dir(&dir).unwrap();

    write_run_files(&dir, &small_config(), 1, &[]).unwrap();

    let summary: serde_json::Value = serde_json::from_str(&read(&dir, "run_summary.json")).unwrap();
    assert_eq!(summary["generations"], 0);
    assert_eq!(summary["final_metrics"], serde_json::json!([]));
}

/// The `points` of the polyline for `series`, as (x, y) pairs.
fn polyline(svg: &str, series: &str) -> Vec<(f64, f64)> {
    let start = svg
        .find(&format!(r#"data-series="{series}""#))
        .unwrap_or_else(|| panic!("no {series} line in {svg}"));
    let points = &svg[start..];
    let points = &points[points.find(r#"points=""#).unwrap() + 8..];
    let points = &points[..points.find('"').unwrap()];
    points
        .split_whitespace()
        .map(|pair| {
            let (x, y) = pair.split_once(',').unwrap();
            (x.parse().unwrap(), y.parse().unwrap())
        })
        .collect()
}

fn row(generation: u32, best: f64, mean: f64) -> MetricsRow {
    MetricsRow {
        generation,
        species: neural_survival_core::MLP_PREY,
        episodes: 1,
        episode_steps: 10,
        best_fitness: best,
        mean_fitness: mean,
        alive_at_end: 0.0,
        food_eaten: 0.0,
    }
}

/// The chart's drawing area from its `width` and `height` attributes.
fn chart_size(svg: &str) -> (f64, f64) {
    let attribute = |name: &str| -> f64 {
        let start = svg.find(&format!(r#" {name}=""#)).unwrap() + name.len() + 3;
        svg[start..start + svg[start..].find('"').unwrap()]
            .parse()
            .unwrap()
    };
    (attribute("width"), attribute("height"))
}

/// Whether `point` lies strictly inside the chart, not on or past its edge.
fn inside(svg: &str, (x, y): (f64, f64)) -> bool {
    let (width, height) = chart_size(svg);
    0.0 < x && x < width && 0.0 < y && y < height
}

/// The y of the `<text>` element whose content is `label`.
fn text_y(svg: &str, label: &str) -> f64 {
    let end = svg.find(&format!(">{label}</text>")).unwrap();
    let start = svg[..end].rfind("<text").unwrap();
    let y = &svg[start..end];
    let y = &y[y.find(r#" y=""#).unwrap() + 4..];
    y[..y.find('"').unwrap()].parse().unwrap()
}

/// A continued run's history: Generations 3 to 5, Fitness between 1 and 9.
fn continued_history() -> [MetricsRow; 3] {
    [row(3, 5.0, 3.0), row(4, 9.0, 5.0), row(5, 7.0, 1.0)]
}

fn chart_of(name: &str, metrics: &[MetricsRow]) -> String {
    let dir = temp_dir(name);
    prepare_output_dir(&dir).unwrap();
    write_run_files(&dir, &small_config(), 1, metrics).unwrap();
    read(&dir, "fitness.svg")
}

#[test]
fn fitness_chart_plots_best_and_mean_per_species() {
    let svg = chart_of("fitness_svg", &continued_history());

    assert!(svg.starts_with("<svg"));
    assert!(svg.trim_end().ends_with("</svg>"));
    let best = polyline(&svg, "MLP-Prey best");
    let mean = polyline(&svg, "MLP-Prey mean");
    assert_eq!(best.len(), 3);
    assert_eq!(mean.len(), 3);
    for &point in best.iter().chain(&mean) {
        assert!(inside(&svg, point), "{point:?} is outside the chart");
    }
    // Generations run left to right, evenly spaced.
    assert!(best[0].0 < best[1].0 && best[1].0 < best[2].0);
    assert!(((best[1].0 - best[0].0) - (best[2].0 - best[1].0)).abs() < 0.01);
    assert_eq!(
        best.iter().map(|p| p.0).collect::<Vec<_>>(),
        mean.iter().map(|p| p.0).collect::<Vec<_>>()
    );
    // Higher Fitness is drawn higher up (smaller y), linearly: 9 is the top,
    // 1 the bottom, 5 halfway, 3 a quarter and 7 three quarters of the way up.
    let top = best[1].1;
    let bottom = mean[2].1;
    assert!(top < bottom);
    let at = |fraction: f64| bottom + fraction * (top - bottom);
    assert!((best[0].1 - at(0.5)).abs() < 0.01);
    assert!((mean[1].1 - at(0.5)).abs() < 0.01);
    assert!((mean[0].1 - at(0.25)).abs() < 0.01);
    assert!((best[2].1 - at(0.75)).abs() < 0.01);
}

#[test]
fn fitness_chart_labels_the_generation_and_fitness_range() {
    let svg = chart_of("fitness_svg_ticks", &continued_history());

    for label in ["3", "5", "1.00", "9.00"] {
        assert!(
            svg.contains(&format!(">{label}</text>")),
            "no {label} label"
        );
    }
    let best = polyline(&svg, "MLP-Prey best");
    let mean = polyline(&svg, "MLP-Prey mean");
    // The Fitness labels sit level with the highest and lowest points.
    assert!((text_y(&svg, "9.00") - best[1].1).abs() < 5.0);
    assert!((text_y(&svg, "1.00") - mean[2].1).abs() < 5.0);
}

#[test]
fn fitness_chart_legend_lists_each_line_top_to_bottom() {
    let svg = chart_of("fitness_svg_legend", &continued_history());

    let best = text_y(&svg, "MLP-Prey best");
    let mean = text_y(&svg, "MLP-Prey mean");
    assert!(inside(&svg, (1.0, best)) && inside(&svg, (1.0, mean)));
    // At least a line of 12px text apart, best first.
    assert!(mean - best >= 12.0, "best at {best}, mean at {mean}");
}

#[test]
fn fitness_chart_of_a_single_flat_generation_draws_inside_the_chart() {
    let svg = chart_of("fitness_svg_flat", &[row(0, 5.0, 5.0)]);

    let point = polyline(&svg, "MLP-Prey best");
    assert_eq!(point.len(), 1);
    assert!(
        inside(&svg, point[0]),
        "{:?} is outside the chart",
        point[0]
    );
}

#[test]
fn fitness_chart_without_history_says_so() {
    let dir = temp_dir("fitness_svg_empty");
    prepare_output_dir(&dir).unwrap();

    write_run_files(&dir, &small_config(), 1, &[]).unwrap();

    let svg = read(&dir, "fitness.svg");
    assert!(svg.contains("No Generations evaluated yet"));
    assert!(!svg.contains("<polyline"));
}

#[test]
fn recording_saves_a_snapshot_every_10_generations_and_at_the_end() {
    let dir = temp_dir("record_fresh");
    prepare_output_dir(&dir).unwrap();
    let mut evolution = Evolution::new(small_config(), 5).unwrap();
    let mut reported = Vec::new();

    record_evolution(&mut evolution, 25, &dir, |progress| {
        reported.push(progress.generation)
    })
    .unwrap();

    assert_eq!(reported, [10, 20, 25]);
    assert_eq!(
        snapshot_names(&dir),
        [
            "snapshot_gen_10.json",
            "snapshot_gen_20.json",
            "snapshot_gen_25.json"
        ]
    );
    assert_eq!(
        Snapshot::load(dir.join("snapshot_gen_25.json")).unwrap(),
        evolution.snapshot()
    );
    let mut ten = Evolution::new(small_config(), 5).unwrap();
    ten.advance(10, |_| {});
    assert_eq!(
        Snapshot::load(dir.join("snapshot_gen_10.json")).unwrap(),
        ten.snapshot()
    );
    for name in ["metrics.csv", "fitness.svg", "run_summary.json"] {
        assert!(dir.join(name).is_file(), "missing {name}");
    }
    assert_eq!(read(&dir, "metrics.csv").lines().count(), 1 + 25);
}

#[test]
fn recording_a_continuation_names_snapshots_by_absolute_generation() {
    let mut evolution = Evolution::new(small_config(), 5).unwrap();
    evolution.advance(25, |_| {});
    let dir = temp_dir("record_continued");
    prepare_output_dir(&dir).unwrap();

    record_evolution(&mut evolution, 20, &dir, |_| {}).unwrap();

    assert_eq!(
        snapshot_names(&dir),
        [
            "snapshot_gen_30.json",
            "snapshot_gen_40.json",
            "snapshot_gen_45.json"
        ]
    );
    // The run files carry the full history, not just this call's Generations.
    assert_eq!(read(&dir, "metrics.csv").lines().count(), 1 + 45);
}

#[test]
fn recording_zero_generations_still_writes_the_current_generation() {
    let dir = temp_dir("record_zero");
    prepare_output_dir(&dir).unwrap();
    let mut evolution = Evolution::new(small_config(), 5).unwrap();

    record_evolution(&mut evolution, 0, &dir, |_| {}).unwrap();

    assert_eq!(
        file_names(&dir),
        [
            "fitness.svg",
            "metrics.csv",
            "run_summary.json",
            "snapshot_gen_0.json"
        ]
    );
}

#[test]
fn recording_into_a_missing_directory_reports_the_failure_after_evolving() {
    let dir = temp_dir("record_missing");
    let mut evolution = Evolution::new(small_config(), 5).unwrap();

    let error = record_evolution(&mut evolution, 25, &dir, |_| {}).unwrap_err();

    assert!(matches!(error, RunFilesError::Snapshot(_)), "{error}");
    // The first failure is reported, not a later one.
    assert!(
        error.to_string().contains("snapshot_gen_10.json"),
        "{error}"
    );
    assert_eq!(evolution.generation(), 25);
    assert!(!dir.exists());
}

#[test]
fn run_files_are_replaced_when_rewritten() {
    let dir = temp_dir("rewrite");
    prepare_output_dir(&dir).unwrap();
    let mut evolution = Evolution::new(small_config(), 5).unwrap();
    evolution.advance(2, |_| {});
    write_run_files(&dir, evolution.config(), 5, evolution.metrics()).unwrap();
    evolution.advance(1, |_| {});

    write_run_files(&dir, evolution.config(), 5, evolution.metrics()).unwrap();

    assert_eq!(read(&dir, "metrics.csv").lines().count(), 1 + 3);
}
