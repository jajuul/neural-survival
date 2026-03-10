# ui

## Purpose
This module provides viewer entrypoints for loading and inspecting simulation snapshots.

## Behavior
- Exposes `launch_snapshot_viewer` as the `run`/`view` integration point.
- In `--features gui` builds, renders a Bevy viewer with:
  - directional agent rendering, so heading is visible directly in the scene (`LTC` triangles rotate with heading; `MLP` circles include a forward marker)
  - explicit `Pause`, `Step`, and `Continue` simulation controls
  - typed numeric fields for evolution continuation (`generations`, `world size`, `food count`, `food motion`, `food respawn`, `sensor noise`, `sensor dropout`, `episode steps`, `mutation rate`)
  - evaluation schedule controls for GUI-triggered continuation runs:
    - initial evaluation mode (`Solo`/`Shared`)
    - post-switch mode (`Solo`/`Shared`)
    - switch generation (`0` disables switching)
  - continue-evolution action that applies edited parameters to in-GUI evolution
  - live evolution progress visibility (`current/total generations`) during GUI-triggered continuation, updated every 10 generations (plus completion)
  - current day/night light state in the window title and selected-agent inspector (`day`/`night`, phase, visibility multiplier)
  - an in-viewer fitness-history plot showing best and mean fitness over generations from snapshot history and GUI-triggered continuation runs
  - click-to-select agent inspection with a live eye-bin chart for food, prey, and predator sensing, local core sensor values (`energy`, `speed`), controller outputs, and last-step energy-cost breakdown
  - a selected-agent network inspector that renders controller topology and live activations:
    - MLP: input, hidden, and output layers
    - LTC: input, recurrent state, output, plus recurrent edges
    - larger, high-contrast panel with brighter activation/weight colors so live changes remain readable during motion
    - textual legend for input channels, output meanings, and activation/weight colors so the plot is readable without guessing the column semantics
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
