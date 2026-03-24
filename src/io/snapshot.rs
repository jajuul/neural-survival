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
    write_snapshot_to_path(&path, snapshot)?;
    Ok(path)
}

/// Write snapshot JSON to an explicit file path.
///
/// Inputs: destination file path and snapshot payload.
/// Output: none.
/// Side effects: creates parent directories if needed and writes JSON file to disk.
pub fn write_snapshot_to_path(path: &Path, snapshot: &SnapshotFile) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let bytes = serde_json::to_vec_pretty(snapshot)?;
    fs::write(path, bytes)?;
    Ok(())
}

/// Write a GUI-originated snapshot to a dedicated directory without overwriting prior saves.
///
/// Inputs: viewer output directory, generation index, world step, and snapshot payload.
/// Output: written file path (`snapshot_gen_<n>_step_<m>[_<k>].json`).
/// Side effects: creates output directory and writes JSON file to disk.
pub fn write_viewer_snapshot(
    out_dir: &Path,
    generation: u32,
    world_step: u64,
    snapshot: &SnapshotFile,
) -> anyhow::Result<PathBuf> {
    fs::create_dir_all(out_dir)?;

    for suffix in 0u32.. {
        let file_name = if suffix == 0 {
            format!("snapshot_gen_{generation:04}_step_{world_step:06}.json")
        } else {
            format!("snapshot_gen_{generation:04}_step_{world_step:06}_{suffix}.json")
        };
        let path = out_dir.join(file_name);
        if path.exists() {
            continue;
        }

        write_snapshot_to_path(&path, snapshot)?;
        return Ok(path);
    }

    unreachable!("u32 suffix space should be effectively unbounded for snapshot saves")
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
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::config::SimulationConfig;
    use crate::sim::initialize_world;

    use super::{SnapshotFile, write_viewer_snapshot};

    fn temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("neural-survival-{label}-{unique}"));
        fs::create_dir_all(&path).expect("temp test dir should be creatable");
        path
    }

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

    #[test]
    fn write_viewer_snapshot_uses_unique_file_names() {
        let out_dir = temp_dir("viewer-snapshot");
        let config = SimulationConfig {
            population_size: 2,
            food_count: 2,
            ..SimulationConfig::default()
        };
        let world = initialize_world(&config, 7);
        let snapshot = SnapshotFile {
            generation: 3,
            seed: 7,
            resolved_config: config,
            metrics_history: Vec::new(),
            population: Vec::new(),
            world_state: world,
        };

        let first = write_viewer_snapshot(&out_dir, 3, 12, &snapshot)
            .expect("first viewer snapshot write should succeed");
        let second = write_viewer_snapshot(&out_dir, 3, 12, &snapshot)
            .expect("second viewer snapshot write should succeed");

        assert_ne!(first, second);
        assert!(first.exists());
        assert!(second.exists());

        let _ = fs::remove_dir_all(out_dir);
    }
}
