# Disclaimer
AI coding test. All code made with AI in this repo.

# neural-survival

`neural-survival` is a native Rust application for evolving neural-network-controlled agents in a deterministic 2D world, then inspecting the resulting populations in a GUI.

The project currently supports:
- MLP and LTC controller families
- prey and predator roles
- fixed-rate food spawning
- optional moving food clusters
- optional day/night sensing attenuation
- configurable round obstacles
- headless batch runs with exported artifacts
- GUI snapshot viewing and in-viewer continuation of evolution

## Quickstart

From the repository root:

```bash
cargo test --lib
cargo test --test cli_workflows
cargo test --test io_artifacts
cargo check --features gui
```

Run a headless batch:

```bash
cargo run --features gui -- batch \
  --config configs/default.toml \
  --seed 42 \
  --generations 50 \
  --out out/batch_42
```

Run headless generations and then open the viewer:

```bash
cargo run --features gui -- run \
  --config configs/default.toml \
  --seed 42 \
  --headless-generations 50 \
  --out out/run_42
```

Open an existing snapshot directly:

```bash
cargo run --features gui -- view \
  --snapshot out/run_42/snapshot_gen_0050.json
```

## CLI Commands

- `run`
  - evolves headlessly for `N` generations
  - writes artifacts
  - opens the GUI on the final snapshot
- `batch`
  - evolves headlessly only
  - writes artifacts and exits
- `view`
  - opens the GUI from a saved snapshot

Both `run` and `batch` accept an optional `--seed`. If omitted, one is generated and printed so the run can be reproduced later.

## Output Artifacts

Headless runs write:
- `metrics.csv`
- `fitness.svg`
- `run_summary.json`
- `snapshot_gen_<n>.json`

The GUI can also save viewer snapshots named like:
- `snapshot_gen_<n>_step_<m>.json`

Snapshots include the resolved config, seed, saved world state, saved population, and metric history needed to reopen the viewer later.

## GUI Summary

The GUI supports:
- pause, continue, and single-step
- simulation speed control
- selected-agent inspection
- live network visualization
- energy-rule toggles
- saving snapshots
- editing continuation parameters
- continuing evolution from a loaded snapshot

The selected-agent inspector shows:
- food, prey, predator, and obstacle eye bins
- core sensor values
- raw controller outputs
- last-step energy breakdown
- controller-family topology/activation view

## Current Config Surface

The checked-in default config is [configs/default.toml](configs/default.toml). It includes the main runtime knobs for:
- world size and timestep
- food count, respawn, spawn rate, and motion
- obstacle count and obstacle radius range
- food clusters
- predator/prey settings
- sensor noise/dropout
- day/night settings
- GA settings
- fitness weights
- controller sizing

## Important Notes

- The simulation is deterministic for the same seed and config, subject to platform/toolchain assumptions.
- Controller input width is currently `42`.
- Older snapshots are upgraded on load when possible so previously saved genomes still build after sensor-surface expansions.
- Some GUI-editable continuation fields are initialization-oriented and do not retroactively rewrite all already-evolved genomes in place.

## Documentation Map

- [docs/SPECS.md](docs/SPECS.md): consolidated product specification
- [docs/BEHAVIOR_SPEC.md](docs/BEHAVIOR_SPEC.md): user-visible behavior
- [docs/TECH_SPEC.md](docs/TECH_SPEC.md): internal technical contracts
- [docs/HANDOFF.md](docs/HANDOFF.md): current state and likely next tasks

## Main Code Areas

- `src/sim`: world stepping, sensing, obstacles, food, and energy logic
- `src/brain`: MLP and LTC controllers
- `src/genetics`: genomes, mutation, crossover, and population evolution
- `src/experiment`: generation loops and evaluation orchestration
- `src/io`: artifacts and snapshots
- `src/ui`: Bevy viewer
- `src/config`: TOML config and validation
