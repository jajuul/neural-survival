# experiment

## Purpose
This module orchestrates deterministic multi-generation evolution runs.

## Behavior
- Initializes a mixed-family population from config and seed.
- Scores each genome through deterministic episode evaluation.
- Evolves generations using genetics operators.
- Produces per-generation best/mean fitness metrics and final population genomes.
- Provides `run_batch_to_dir` to persist metrics, summary, and snapshot artifacts.

## Inputs
- `SimulationConfig`
- Run seed and generation count.

## Outputs
- `EvolutionRun` with generation metrics and final population.

## Side Effects
- No external side effects in Milestone 5.
