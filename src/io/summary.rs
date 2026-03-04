use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::SimulationConfig;

/// Serialized run metadata stored in `run_summary.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunSummary {
    pub seed: u64,
    pub generations: u32,
    pub config_hash: String,
    pub resolved_config: SimulationConfig,
    pub fitness_weight_survival: f32,
    pub fitness_weight_food: f32,
    pub fitness_weight_energy: f32,
    pub runner: String,
}

/// Compute deterministic FNV-1a config hash string.
///
/// Inputs: simulation config.
/// Output: stable lowercase hex hash.
/// Side effects: none.
pub fn config_hash(config: &SimulationConfig) -> anyhow::Result<String> {
    let payload = serde_json::to_vec(config)?;

    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in payload {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }

    Ok(format!("{hash:016x}"))
}

/// Write run summary JSON to output directory.
///
/// Inputs: output directory and summary payload.
/// Output: written file path.
/// Side effects: writes JSON file to disk.
pub fn write_run_summary(out_dir: &Path, summary: &RunSummary) -> anyhow::Result<PathBuf> {
    let path = out_dir.join("run_summary.json");
    let bytes = serde_json::to_vec_pretty(summary)?;
    fs::write(&path, bytes)?;
    Ok(path)
}
