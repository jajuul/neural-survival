# sim

## Purpose
This module implements the deterministic fixed-step simulation core for the 2D toroidal world.

## Behavior
- Initializes agent and food state deterministically from a seed.
- Advances the world one fixed step at a time using stub controller actions.
- Applies toroidal wrapping, food collision handling, and energy loss/gain updates.
- Produces aggregate episode summaries for reproducibility checks.

## Inputs
- `SimulationConfig` with world geometry, timing, and energy parameters.
- Deterministic `seed` for world initialization.

## Outputs
- Updated `WorldState` per step.
- `StepOutcome` (food consumed and deaths) per step.
- `EpisodeSummary` for headless episode comparisons.

## Side Effects
- No external side effects; functions mutate in-memory world state only.
