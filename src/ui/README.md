# ui

## Purpose
This module provides viewer entrypoints for loading and inspecting simulation snapshots.

## Behavior
- Exposes `launch_snapshot_viewer` as the `run`/`view` integration point.
- In `--features gui` builds, renders a Bevy viewer with:
  - directional agent rendering, so heading is visible directly in the scene (`LTC` triangles rotate with heading; `MLP` circles include a forward marker)
  - start/stop simulation control
  - typed numeric fields for evolution continuation (`generations`, `world size`, `food count`, `food respawn`, `episode steps`, `mutation rate`)
  - an evaluation-mode toggle (`Solo`/`Shared`) that applies to GUI-triggered continuation runs
  - continue-evolution action that applies edited parameters to in-GUI evolution
  - live evolution progress visibility (`current/total generations`) during GUI-triggered continuation, updated every 10 generations (plus completion)
  - an in-viewer fitness-history plot showing best and mean fitness over generations from snapshot history and GUI-triggered continuation runs
  - click-to-select agent inspection with a live eye-bin chart for both food and agent sensing, core sensor values, controller outputs, and last-step energy-cost breakdown
  - controller-driven viewer stepping, so GUI movement and observability use the same neural controllers rather than stub actions
  - speed/rule toggles and restart from loaded snapshot state
- In non-GUI builds, keeps a viewer stub so CLI workflows remain testable in headless environments.

## Inputs
- `SnapshotFile` payloads.

## Outputs
- Viewer lifecycle completion (`Ok(())`) or an error.

## Side Effects
- In GUI mode, opens a window, mutates in-memory runtime state, and handles user interaction.
- In non-GUI mode, prints a viewer summary and returns.
