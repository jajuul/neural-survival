Tier: full — adds GUI dependencies (`eframe`, `egui_plot`) and a new viewer component (ADR-0003).

Base: 2a2a8fe

# Proposal

## Why

Evolution runs headlessly and leaves Snapshots, but there is no way to watch how a saved Generation behaves. Ticket 04 adds `neural-survival view --snapshot <path>`: a window that plays a Showcase of the Snapshot's Generation, which every later viewer ticket (05, 07, 09–16) builds on.

## What Changes

- New `view --snapshot <path>` CLI command that loads a Snapshot and opens an `eframe`/`egui` window, paused on a Showcase of that Generation.
- New viewer crate holding the GUI. Its logic lives in a headless model with no `egui` types, so it can be tested without a window:
  - **Playback:** pause, continue, single-step while paused, simulation speed in steps per second, stop at Episode end, and restart that replays the Showcase identically. Speed changes only how many steps run per frame, never what a step computes (ADR-0001).
  - **Stats:** Generation, world step, alive count, food remaining, alive and total Agents per Species and per Controller Family, and the Evaluation Schedule in force for the Generation.
  - **Scene:** the shapes the world view draws. Each living Agent is drawn in its Role's colour and its Controller Family's shape, food as small dots, and anything overlapping a torus edge is also drawn on the opposite side.
  - **Fitness chart data:** one best and one mean line per Species from the Snapshot's metrics history, drawn with `egui_plot`.
- A thin `egui` layer renders the model. It is not tested automatically.
- A hidden `--no-window` flag on `view` loads the Snapshot and builds the viewer model, then exits without opening a window, so the CLI can be tested as a process.

## Out of scope

- Opening the window, painting, layout, and widget behaviour are not tested automatically (spec Testing Decisions › "The GUI is not tested automatically"). The human checks them by hand at step 8.
- Selecting an Agent and showing its telemetry (ticket 14, Inspector).
- Arena Mode and side-by-side Arenas (08), Energy Rule toggle display (05), Light Phase display (10), Food Clusters and Obstacles (09, 12).
- Predator colour and LTC shape beyond reserving them: only MLP Prey exist.
- Arena Mode change markers on the chart (08).
- The `run` command (13), config editing and edit saves (15), continuing evolution from the viewer (16).
- Any change to the core library's behaviour or public interface.

## Capabilities

### New Capabilities

- `viewer`: the `view` command and the viewer's headless model: playback controls, stats, the drawn scene, and the Fitness chart data.

### Modified Capabilities

None. The viewer reads the existing Showcase and Snapshot interfaces unchanged.

## Impact

- **New crate** `crates/viewer` (`neural-survival-viewer`), depending on `neural-survival-core`, `eframe` and `egui_plot`. The core crate keeps no GUI dependency, which `scripts/check-architecture.sh` already enforces.
- **Binary** `src/main.rs`: a new `view` subcommand. The root `Cargo.toml` and the workspace members change, and `Cargo.lock` grows considerably with the windowing stack.
- **Build time and system libraries:** `eframe` pulls in `winit` and a graphics backend. Building on Linux and WSL needs the usual X11/Wayland development packages, and opening the window under WSL needs WSLg.
- **No change** to `crates/core`, including the always-read paths `rng.rs` and `snapshot.rs`.
