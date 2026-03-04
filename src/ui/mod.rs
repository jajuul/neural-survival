//! Viewer entrypoints.

use crate::io::snapshot::SnapshotFile;

/// Launch viewer from a snapshot payload.
///
/// Inputs: snapshot state loaded from disk.
/// Output: `Ok(())` after viewer exits.
/// Side effects: in this build profile, prints summary and returns.
pub fn launch_snapshot_viewer(snapshot: &SnapshotFile) -> anyhow::Result<()> {
    println!(
        "viewer stub: generation={}, population={}, world={}x{}",
        snapshot.generation,
        snapshot.population.len(),
        snapshot.world_state.width,
        snapshot.world_state.height
    );
    Ok(())
}
