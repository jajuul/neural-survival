use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::experiment::GenerationMetrics;

#[derive(Serialize)]
struct MetricsRow<'a> {
    generation: u32,
    best_fitness: f32,
    mean_fitness: f32,
    survival_min: u32,
    survival_mean: f32,
    survival_max: u32,
    alive_count: usize,
    controller_mlp: usize,
    controller_ltc: usize,
    config_hash: &'a str,
}

/// Write generation metrics to `metrics.csv`.
///
/// Inputs: output directory, per-generation metrics, and config hash.
/// Output: written file path.
/// Side effects: writes CSV file to disk.
pub fn write_metrics_csv(
    out_dir: &Path,
    metrics: &[GenerationMetrics],
    config_hash: &str,
) -> anyhow::Result<PathBuf> {
    let path = out_dir.join("metrics.csv");
    let mut writer = csv::Writer::from_path(&path)?;

    for metric in metrics {
        writer.serialize(MetricsRow {
            generation: metric.generation_index,
            best_fitness: metric.best_fitness,
            mean_fitness: metric.mean_fitness,
            survival_min: metric.survival_min,
            survival_mean: metric.survival_mean,
            survival_max: metric.survival_max,
            alive_count: metric.population_size,
            controller_mlp: metric.controller_mlp,
            controller_ltc: metric.controller_ltc,
            config_hash,
        })?;
    }

    writer.flush()?;
    Ok(path)
}
