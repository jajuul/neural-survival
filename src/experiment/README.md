# experiment

## Purpose
This module orchestrates deterministic multi-generation evolution runs.

## Behavior
- Initializes a mixed-family population from config and seed.
- Scores genomes through deterministic episodes selected by `evaluation_mode`:
  - `solo`: each genome gets its own private episode
  - `shared`: the full population competes over one shared food field during scoring
- Supports optional predator/prey ecology inside shared evaluation:
  - prey are scored from food, survival, and energy
  - predators are scored from captures, survival, and energy
  - predator/prey reproduction remains partitioned by role as well as controller family
- Supports scheduled regime switching with `evaluation_mode_after_switch` and
  `evaluation_mode_switch_after_generation`, so runs can follow curricula such as
  `solo -> shared @ 100`.
- Feeds controllers deterministic sensor vectors (core scalar cues plus separate food-eye, prey-eye, and predator-eye channels).
- Parallelizes `solo` evaluation deterministically across genomes, since each episode already has its own derived seed and no shared mutable world state.
- Reuses shared-world action/sensor buffers and consumes per-step sensor data from the simulation's spatially indexed fast path.
- Uses whatever world resource model is configured, including moving food clusters with phase-dependent concentration, while keeping evaluation deterministic from seed/config.
- Evolves generations using genetics operators.
- Produces per-generation best/mean fitness metrics and final population genomes.
- Provides `run_batch_to_dir` to persist metrics, fitness plot, summary, and snapshot artifacts.
- Provides progress-aware runner variants (`run_generations_with_progress`, `run_batch_to_dir_with_progress`) that report generation completion counts for CLI/UI feedback.
- Provides continuation helpers for evolving additional generations from an existing population (`continue_generations_from_population`) and rebuilding deterministic world state for that population (`world_state_for_population`).
- Embeds generation-metric history into snapshots so the GUI can render fitness trends without re-reading CSV files.

## Inputs
- `SimulationConfig`
- Run seed and generation count.

## Outputs
- `EvolutionRun` with generation metrics and final population.

## Side Effects
- Runner APIs are pure in-memory.
- `run_batch_to_dir` writes `metrics.csv`, `fitness.svg`, `run_summary.json`, and snapshot artifacts to disk.
