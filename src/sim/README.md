# sim

## Purpose
This module implements the deterministic fixed-step simulation core for the 2D toroidal world.

## Behavior
- Initializes agent and food state deterministically from a seed.
- Advances the world one fixed step at a time either with stub controller actions (`step_world`) or caller-provided controller actions (`step_world_with_actions` / `step_world_with_actions_and_telemetry`).
- Applies toroidal wrapping, optional food drift motion, food collision handling, deterministic optional food respawn, deterministic fixed-rate food spawning, and energy loss/gain updates.
- Uses acceleration-driven movement with persistent linear momentum and configurable drag instead of writing speed directly from throttle each step.
- Supports optional deterministic food clusters: moving cluster centers with per-cluster phase offsets that tighten or spread food density over the day/night cycle.
- Supports an optional deterministic day/night cycle derived from `world.step_count`; the current light level is exposed as a reusable helper and can reduce sensing visibility at night.
- Builds controller sensor vectors with Shorelark-inspired eyes: food, prey, and predators are binned by relative angle inside a configurable field-of-view and bounded by perception range/channel traits.
- Uses fixed-width controller sensor arrays (`energy`, `speed`, `food bins`, `prey bins`, `predator bins`) for the evaluation/runtime path to avoid per-step heap allocation churn.
- Uses a coarse toroidal spatial index per world snapshot so eye queries, predator captures, and food-collision checks prune to nearby food/agent buckets instead of scanning the full world each step.
- Defers consumed-food removal until after the per-agent loop and uses `swap_remove` so frequent food collisions do not incur O(n) vector shifts.
- Exposes only local core scalars (`energy`, `speed`) to controllers; absolute `x/y` position is not part of the controller input.
- Supports deterministic sensor corruption via configurable Gaussian noise and dropout probabilities.
- Eye geometry is trait-driven: agents evolve their perception range, number of eye bins, and per-eye angular width.
- Supports optional predator/prey capture rules where predators gain energy from prey collisions and prey die on capture.
- Skips predator/prey eye-channel work when the target ecological role is absent from the current world snapshot.
- Keeps cluster-assigned food attached to moving cluster centers and recolors/relayouts respawned food deterministically from world seed + step count.
- Exposes decomposed sensor observations (food bins, prey bins, predator bins, and the flattened controller-facing channel stack), current day/night state, and per-term energy telemetry so the GUI can inspect what a selected agent sensed and paid for on the last step.
- Produces aggregate episode summaries for reproducibility checks.

## Inputs
- `SimulationConfig` with world geometry, timing, food respawn/spawn-rate/motion, optional food-cluster parameters, predator/prey capture parameters, motion parameters (`max_acceleration`, `linear_drag`), day/night visibility, sensor corruption, and energy parameters.
- Deterministic `seed` for world initialization.

## Outputs
- Updated `WorldState` per step.
- `StepOutcome` (food consumed, prey captures, and deaths) per step.
- `SensorObservation`, `EnergyBreakdown`, and `StepAgentTelemetry` for observability-aware callers.
- `EpisodeSummary` for headless episode comparisons.

## Side Effects
- No external side effects; functions mutate in-memory world state only.
