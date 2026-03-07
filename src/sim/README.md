# sim

## Purpose
This module implements the deterministic fixed-step simulation core for the 2D toroidal world.

## Behavior
- Initializes agent and food state deterministically from a seed.
- Advances the world one fixed step at a time either with stub controller actions (`step_world`) or caller-provided controller actions (`step_world_with_actions` / `step_world_with_actions_and_telemetry`).
- Applies toroidal wrapping, food collision handling, deterministic optional food respawn, and energy loss/gain updates.
- Builds controller sensor vectors with Shorelark-inspired eyes: food and other agents are binned by relative angle inside a configurable field-of-view and bounded by perception range/channel traits.
- Eye geometry is trait-driven: agents evolve their perception range, number of eye bins, and per-eye angular width.
- Exposes decomposed sensor observations (food bins, agent bins, and the flattened controller-facing channel stack) and per-term energy telemetry so the GUI can inspect what a selected agent sensed and paid for on the last step.
- Produces aggregate episode summaries for reproducibility checks.

## Inputs
- `SimulationConfig` with world geometry, timing, food respawn, and energy parameters.
- Deterministic `seed` for world initialization.

## Outputs
- Updated `WorldState` per step.
- `StepOutcome` (food consumed and deaths) per step.
- `SensorObservation`, `EnergyBreakdown`, and `StepAgentTelemetry` for observability-aware callers.
- `EpisodeSummary` for headless episode comparisons.

## Side Effects
- No external side effects; functions mutate in-memory world state only.
