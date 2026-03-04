# ui

## Purpose
This module provides viewer entrypoints for loading and inspecting simulation snapshots.

## Behavior
- Exposes `launch_snapshot_viewer` as the `run`/`view` integration point.
- In `--features gui` builds, renders a Bevy viewer with:
  - start/stop simulation control
  - generation-step controls to continue evolution by a variable generation count
  - speed/rule toggles and restart from loaded snapshot state
- In non-GUI builds, keeps a viewer stub so CLI workflows remain testable in headless environments.

## Inputs
- `SnapshotFile` payloads.

## Outputs
- Viewer lifecycle completion (`Ok(())`) or an error.

## Side Effects
- In GUI mode, opens a window, mutates in-memory runtime state, and handles user interaction.
- In non-GUI mode, prints a viewer summary and returns.
