# experiment

## Purpose
This module orchestrates deterministic multi-generation evolution runs.

## Behavior
- Initializes a mixed-family population from config and seed.
- Scores each genome through deterministic episode evaluation.
- Feeds controllers deterministic sensor vectors (core scalar cues plus eye-bin food vision).
- Evolves generations using genetics operators.
- Produces per-generation best/mean fitness metrics and final population genomes.
- Provides `run_batch_to_dir` to persist metrics, summary, and snapshot artifacts.
- Provides progress-aware runner variants (`run_generations_with_progress`, `run_batch_to_dir_with_progress`) that report generation completion counts for CLI/UI feedback.
- Provides continuation helpers for evolving additional generations from an existing population (`continue_generations_from_population`) and rebuilding deterministic world state for that population (`world_state_for_population`).

## Inputs
- `SimulationConfig`
- Run seed and generation count.

## Outputs
- `EvolutionRun` with generation metrics and final population.

## Side Effects
- Runner APIs are pure in-memory.
- `run_batch_to_dir` writes metrics/summary/snapshot artifacts to disk.
