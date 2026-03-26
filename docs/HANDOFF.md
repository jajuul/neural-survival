# Handoff Notes

## Purpose
This file is a practical handoff for the next contributor or a new ChatGPT account. It summarizes the current state of the repository, the most recent changes, the important architectural facts that are easy to miss, and the most likely next tasks.

This file should be updated whenever the project direction changes materially or when a new contributor would otherwise need chat history to understand current priorities.

## Current State
- The repo builds and tests cleanly after the most recent code changes for:
  - `cargo test --lib`
  - `cargo test --test cli_workflows`
  - `cargo test --test io_artifacts`
  - `cargo check --features gui`
- The simulation currently supports:
  - MLP and LTC controller families
  - prey and predator roles
  - fixed-rate food spawning
  - optional moving food clusters
  - optional day/night sensing attenuation
  - configurable round obstacles
  - GUI continuation of evolution from snapshots
- The current fixed controller input width is `42`:
  - `2` core scalars
  - `10` food bins
  - `10` prey bins
  - `10` predator bins
  - `10` obstacle bins

## Recent Commits
- `f9eb53c` `Add configurable round obstacles`
- `da9e9a6` `Color predators red in viewer`
- `dff6edc` `Add fixed-rate food spawning`
- `3a130ea` `Format LTC and operator code`
- `248234a` `Optimize solo simulation hot path`

## Current Working Tree
At the time this note was written, these local changes were not yet committed:
- modified: `docs/TECH_SPEC.md`
- untracked: `docs/BEHAVIOR_SPEC.md`
- untracked: `viewer_snapshots/`

If those are still present later, treat them as intentional until verified otherwise.

## Important Architectural Facts

### Simulation Core
- Main world stepping lives in `src/sim/world.rs`.
- `WorldState` now includes `obstacles`.
- Obstacles are internal static circles. They do not behave as toroidal clones across edges.
- Agent movement uses obstacle-aware substeps to avoid tunneling through small obstacles.
- Food placement and food motion are pushed out of obstacle interiors.

### Sensors
- Sensor/vector contracts live in `src/sim/types.rs`.
- Controller-facing inputs are fixed-width and zero-fill inactive bins.
- Expanding the sensor surface changes controller input width and can break old serialized genomes unless an upgrade path exists.

### Snapshot Compatibility
- Snapshot I/O lives in `src/io/snapshot.rs`.
- Older snapshots are upgraded on load so MLP/LTC genomes with older input widths still build after adding new sensor channels.
- Similar upgrade logic also exists in controller factory build paths as a safety net.

### Evolution and Fitness
- Evolution orchestration lives in `src/experiment/runner.rs`.
- Controller families do not interbreed.
- Roles also reproduce in separate pools, so breeding is partitioned by `(ControllerKind, AgentRole)`.
- Evaluation can be `solo`, `shared`, or scheduled `solo -> shared`.

### GUI
- Viewer logic is in `src/ui/mod.rs`.
- The GUI exposes a large runtime config surface for continuation runs.
- Some GUI-editable fields are initialization-oriented and do not retroactively rewrite all already-evolved genomes in place.

## Current Spec Documents
- `docs/SPECS.md` is the consolidated specification.
- `docs/TECH_SPEC.md` is the technical companion.
- `docs/BEHAVIOR_SPEC.md` is the behavior companion.

These documents should stay aligned. If only one is updated, the next contributor will lose time resolving drift.

## Known Context And Open Questions

### LTC Fitness
The LTC family did not appear frozen genetically, but it underperformed under some fitness settings. A likely product-level issue is fitness shaping rather than a total LTC implementation failure.

What was observed:
- LTC could improve when survival/energy shaping was removed and food collection was the only reward.
- The current task is not necessarily “fix LTC math” first. It may be “pick a better environment + fitness regime.”

Reasonable next investigation areas:
- simpler fitness shaping
- stronger ecological pressure so survival is not saturated
- comparative runs with and without energy reward

### Obstacles
Obstacles were added recently. The current implementation is intended to be useful and deterministic, not a final ecology design.

Reasonable follow-ups:
- tune default obstacle parameters
- decide whether obstacle rendering should be more visually prominent
- decide whether obstacle interaction should carry any explicit energy or fitness implication

## Likely Next Tasks
- Commit the updated behavior and tech specs if they are still uncommitted.
- Update the top-level `README.md` with a concise current quickstart and feature summary.
- Add one short “current research questions” section somewhere checked in if LTC-vs-MLP behavior remains the main investigation thread.
- Decide whether `viewer_snapshots/` should remain an ad hoc working directory or become a documented artifact location.
- If GUI workflow is important, do one manual visual pass on:
  - obstacle rendering
  - selected-agent obstacle eye bins
  - continuation controls after loading old snapshots

## Fast Orientation Commands
Run these from the repository root:

```bash
cargo test --lib
cargo test --test cli_workflows
cargo test --test io_artifacts
cargo check --features gui
```

Useful code-navigation starting points:
- `src/sim/world.rs`
- `src/sim/types.rs`
- `src/experiment/runner.rs`
- `src/io/snapshot.rs`
- `src/ui/mod.rs`
- `configs/default.toml`

## Notes For A New ChatGPT Account
- Read `AGENTS.md` first.
- Read `docs/SPECS.md`, `docs/TECH_SPEC.md`, and `docs/BEHAVIOR_SPEC.md` before changing behavior.
- Check `git status --short` immediately so you know whether you are inheriting local docs/snapshots that were never committed.
- If changing sensor width again, preserve or extend snapshot upgrade behavior instead of breaking old snapshots silently.
