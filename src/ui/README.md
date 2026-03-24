# ui

## Purpose
This module provides viewer entrypoints for loading and inspecting simulation snapshots.

## Behavior
- Exposes `launch_snapshot_viewer` as the `run`/`view` integration point.
- In `--features gui` builds, renders a Bevy viewer with:
  - directional agent rendering, so heading is visible directly in the scene (`LTC` triangles rotate with heading; `MLP` circles include a forward marker)
  - explicit `Pause`, `Step`, and `Continue` simulation controls
  - grouped typed editor fields for the full GUI continuation config surface:
    - world, timing, food, clustering, and day/night parameters
    - GA and fitness-weight parameters
    - default trait, controller-capacity, predator/prey, and sensor parameters
    - rule toggles (`energy_by_speed`, `energy_by_network_size`, `energy_by_perception_size`)
  - evaluation schedule controls for GUI-triggered continuation runs:
    - initial evaluation mode (`Solo`/`Shared`)
    - post-switch mode (`Solo`/`Shared`)
    - switch generation (`0` disables switching)
  - continue-evolution action that applies the edited runtime config to the next in-GUI evolution run
  - explicit note in the control panel that default/architecture fields affect newly initialized agents rather than rewriting already-evolved genomes in place
  - live evolution progress visibility (`current/total generations`) during GUI-triggered continuation, updated every 10 generations (plus completion)
  - current day/night light state in the window title and selected-agent inspector (`day`/`night`, phase, visibility multiplier)
  - faint moving food-cluster region markers plus cluster-colored food sprites so resource regions are visible in the world
  - an in-viewer fitness-history plot showing best and mean fitness over generations from snapshot history and GUI-triggered continuation runs
  - a `Save` button that writes the current viewer state to `viewer_snapshots/` in the working directory and shows the exact saved path in the control panel
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
