# experiment

## Purpose
This module orchestrates deterministic multi-generation evolution runs.

## Behavior
- Initializes a mixed-family population from config and seed.
- Scores the full population through deterministic shared-world episode evaluation, so genomes compete over the same food field during fitness assignment.
- Feeds controllers deterministic sensor vectors (core scalar cues plus signed eye-bin vision derived from food and nearby agents).
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
