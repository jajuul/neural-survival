# ui

## Purpose
This module provides viewer entrypoints for loading and inspecting simulation snapshots.

## Behavior
- Exposes `launch_snapshot_viewer` as the `run`/`view` integration point.
- Current build profile uses a non-GUI viewer stub so CLI workflows remain testable in headless environments.

## Inputs
- `SnapshotFile` payloads.

## Outputs
- Viewer lifecycle completion (`Ok(())`) or an error.

## Side Effects
- Prints viewer summary in the current stub implementation.
