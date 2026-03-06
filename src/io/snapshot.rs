use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::SimulationConfig;
use crate::experiment::GenerationMetrics;
use crate::genetics::AgentGenome;
use crate::sim::WorldState;

/// Snapshot payload persisted for later viewer reload.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SnapshotFile {
    pub generation: u32,
    pub seed: u64,
    pub resolved_config: SimulationConfig,
    #[serde(default)]
    pub metrics_history: Vec<GenerationMetrics>,
    pub population: Vec<AgentGenome>,
    pub world_state: WorldState,
}

/// Write snapshot JSON to output directory.
///
/// Inputs: output directory, generation index, and snapshot payload.
/// Output: written file path (`snapshot_gen_<n>.json`).
/// Side effects: writes JSON file to disk.
pub fn write_snapshot(
    out_dir: &Path,
    generation: u32,
    snapshot: &SnapshotFile,
) -> anyhow::Result<PathBuf> {
    let path = out_dir.join(format!("snapshot_gen_{generation:04}.json"));
    let bytes = serde_json::to_vec_pretty(snapshot)?;
    fs::write(&path, bytes)?;
    Ok(path)
}

/// Read snapshot JSON from disk.
///
/// Inputs: snapshot file path.
/// Output: parsed snapshot payload.
/// Side effects: reads file from disk.
pub fn read_snapshot(path: &Path) -> anyhow::Result<SnapshotFile> {
    let bytes = fs::read(path)?;
    let snapshot = serde_json::from_slice(&bytes)?;
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use crate::config::SimulationConfig;
    use crate::sim::initialize_world;

    use super::SnapshotFile;

    #[test]
    fn snapshot_defaults_missing_metrics_history() {
        let config = SimulationConfig {
            population_size: 2,
            food_count: 2,
            ..SimulationConfig::default()
        };
        let world = initialize_world(&config, 7);
        let json = serde_json::json!({
            "generation": 3,
            "seed": 7,
            "resolved_config": config,
            "population": [],
            "world_state": world
        });

        let snapshot: SnapshotFile =
            serde_json::from_value(json).expect("legacy snapshot shape should deserialize");

        assert!(snapshot.metrics_history.is_empty());
    }
}
