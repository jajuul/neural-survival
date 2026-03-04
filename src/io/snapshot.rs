use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::SimulationConfig;
use crate::genetics::AgentGenome;
use crate::sim::WorldState;

/// Snapshot payload persisted for later viewer reload.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SnapshotFile {
    pub generation: u32,
    pub seed: u64,
    pub resolved_config: SimulationConfig,
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
