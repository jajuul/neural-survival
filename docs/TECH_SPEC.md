# Neural Survival v1 Technical Specification

## Purpose
This document describes how `neural-survival` works internally today. It captures the technical contracts, current simulation model, controller interfaces, configuration surface, artifact behavior, and viewer capabilities that support the user-visible behavior described in `docs/BEHAVIOR_SPEC.md`.

## Runtime and Delivery
- Target platform: native desktop application.
- Primary language: Rust edition 2024.
- Rendering and UI: Bevy behind the `gui` Cargo feature.
- Workflow model: CLI-driven experiment runs and GUI visualization.

## Core Simulation Model

### World
- The world is a 2D continuous torus with wrap-around edges.
- Simulation advances in fixed time steps.
- `WorldState` stores:
  - world size
  - agents
  - food
  - optional food clusters
  - optional static round obstacles
  - world seed
  - current step count

Food behavior:
- initial food spawns deterministically from the run seed
- food may remain stationary or drift according to `food_motion_speed`
- food may respawn immediately on consumption according to `food_respawn_probability`
- food may also spawn according to `food_spawn_rate_per_second`, interpreted as foods per simulated second
- food may belong to moving clusters when `food_cluster_enabled = true`

Obstacle behavior:
- obstacles are static round objects generated from `obstacle_count`, `obstacle_radius_min`, and `obstacle_radius_max`
- obstacles do not wrap as toroidal clones; they are ordinary internal objects inside the toroidal world
- agent movement is blocked by obstacles
- food placement and food motion are pushed out of obstacle interiors

### Agents
- Agents carry position, heading, velocity, energy, alive/dead state, and projected trait values.
- Trait-projected runtime fields include:
  - `max_speed`
  - `perception_radius`
  - `perception_channels`
  - `eye_cell_angle_radians`
  - `metabolic_efficiency`
  - `role`
- Roles are:
  - `prey`
  - `predator`
- Prey consume food.
- Predators capture prey within `predator_capture_radius` and gain `predator_capture_energy_gain`.

### Kinematics
- Actions are steering plus throttle.
- Steering rotates heading by `max_turn_rate_radians * dt`.
- Throttle requests forward acceleration bounded by `max_acceleration`.
- Velocity persists across steps and is damped by `linear_drag`.
- Velocity magnitude is clamped by the agent's `max_speed`.
- Movement uses obstacle-aware substeps so fast agents do not tunnel through small obstacles in a single step.

### Day and Night
- The optional day/night cycle is deterministic and derived from `world.step_count`.
- `night_visibility_multiplier` attenuates sensing at night.
- Day/night currently affects sensing visibility only; it does not directly alter movement or energy cost terms.

## Neural Controllers

### Supported Families
- Feed-forward MLP.
- Conductance-based LTC network.

### Shared Contracts
Both controller families implement one shared runtime interface:

```rust
pub trait NeuralController: Send + Sync {
    fn reset(&mut self);
    fn step(&mut self, input: &SensorVector, dt: f32) -> ActionVector;
    fn step_with_observability(&mut self, input: &SensorVector, dt: f32) -> ControllerStepOutput;
    fn inspect_network(&self, input: &SensorVector) -> ControllerNetworkSnapshot;
    fn step_with_diagnostics(&mut self, input: &SensorVector, dt: f32) -> ControllerDiagnostics;
    fn parameter_count(&self) -> usize;
    fn controller_kind(&self) -> ControllerKind;
}
```

Factories build runtime controllers from serialized genomes:

```rust
pub trait ControllerFactory {
    fn random_genome(&self, rng: &mut StdRng) -> BrainGenome;
    fn build(&self, genome: &BrainGenome) -> Box<dyn NeuralController>;
}
```

### Capacity Knobs
- `mlp_hidden_size`
- `ltc_state_size`

### LTC Stepping
The LTC controller uses gated sensory and recurrent conductances with leak and membrane-capacitance terms. Each simulation step advances recurrent state by a fixed number of internal ODE unfolds.

Each unfold computes the next membrane value as a conductance-weighted average of:
- prior state
- leak pull
- sensory synapses
- recurrent synapses

where each synaptic conductance is `weight * sigmoid((source - mu) * sigma)` and each synapse pulls toward a configured reversal potential.

### Legacy Genome Compatibility
Controller input width is fixed for newly created genomes, but old snapshots may have older input shapes. On snapshot load, serialized MLP/LTC genomes are upgraded in memory by appending zero-initialized obstacle-channel input weights so old snapshots continue to load after sensor-surface expansions.

## Genomes and Traits

```rust
pub struct AgentGenome {
    pub controller_kind: ControllerKind,
    pub brain: BrainGenome,
    pub traits: TraitGenome,
}

pub struct TraitGenome {
    pub max_speed: f32,
    pub perception_radius: f32,
    pub perception_channels: u16,
    pub eye_cell_angle_radians: f32,
    pub metabolic_efficiency: f32,
    pub role: AgentRole,
    pub food_preference_mask: u32,
}
```

Evolution is partitioned by:
- `ControllerKind`
- `AgentRole`

That means crossover and selection occur only inside one `(ControllerKind, AgentRole)` pool.

## Sensor and Action Model

### Sensor Vector
Controller input is fixed-width and currently has width `42`.

Layout:
- `2` core scalars: `energy`, `speed`
- `10` food eye bins
- `10` prey eye bins
- `10` predator eye bins
- `10` obstacle eye bins

Inactive bins are zero-filled so all controllers receive the same input width regardless of the evolved active channel count.

Eye-bin behavior:
- bins cover relative angle within the active field of view
- `perception_channels` determines how many bins are active
- `eye_cell_angle_radians` determines angle coverage per bin
- `perception_radius` determines maximum sensing range
- day/night visibility scales effective range and activation strength
- deterministic sensor corruption can be applied through:
  - `sensor_noise_stddev`
  - `sensor_dropout_probability`

The GUI observability layer exposes decomposed `food`, `prey`, `predator`, and `obstacle` bin arrays plus the flattened controller-facing vector.

### Action Vector
Controllers output:
- steering
- throttle

The GUI also exposes:
- raw steering output
- raw throttle output
- applied throttle after remapping/clamping

## Energy Model
Per fixed step:

`E(t + dt) = E(t) - base_loss - speed_loss - network_loss - perception_loss + food_gain + capture_gain`

Term activation:
- `speed_loss` applies only if `energy_by_speed = true`
- `network_loss` applies only if `energy_by_network_size = true`
- `perception_loss` applies only if `energy_by_perception_size = true`

Agents die when resulting energy is `<= 0`.

## Fitness Model
Per-agent fitness is config-weighted and role-aware:
- survival duration
- food consumed
- prey captured
- remaining energy

Weights:
- `fitness_weight_survival`
- `fitness_weight_food`
- `fitness_weight_predation`
- `fitness_weight_energy`

Episode source depends on the effective evaluation mode:
- `solo`: each genome is scored in a private deterministic episode
- `shared`: the generation is scored in a shared controller-driven world

Evaluation mode can switch during a run using:
- `evaluation_mode`
- `evaluation_mode_after_switch`
- `evaluation_mode_switch_after_generation`

## Genetic Algorithm
- Population evolves in discrete generations.
- Selection uses tournament selection.
- Retention uses elitism.
- Reproduction uses crossover followed by mutation.
- Mutation and crossover touch both brain genes and trait genes.
- Randomness is deterministic and routed through seeded RNG streams.

Implementation details that matter operationally:
- `solo` evaluation may run in deterministic parallel
- the simulation uses fixed-width sensor vectors to reduce per-step allocation churn
- a coarse spatial index prunes food and agent sensing/collision work in multi-agent worlds
- single-agent evaluation has special fast paths to avoid building the full spatial index when possible

## Configuration Surface
`SimulationConfig` owns the runtime-editable knobs for:
- world size and timestep
- episode length
- population size
- food count, respawn, spawn rate, and motion
- obstacle count and obstacle radius range
- food cluster settings
- GA settings
- energy settings
- fitness weights
- controller sizing
- predator/prey settings
- sensor noise/dropout
- day/night settings
- evaluation-mode schedule
- energy-rule toggles

Validation enforces:
- positive world dimensions and episode steps
- bounded probabilities and fractions
- bounded obstacle radius range
- obstacle size must fit inside the world
- positive or nonnegative numeric constraints depending on field semantics

## CLI Surface
- `neural-survival run --config <path> [--seed <u64>] --headless-generations <u32> --out <dir>`
- `neural-survival batch --config <path> [--seed <u64>] --generations <u32> --out <dir>`
- `neural-survival view --snapshot <path>`

Rules:
- `run` and `batch` print generation progress every 10 generations plus completion
- output directory creation fails if the target exists and is non-empty
- when `run` completes headless generations, it loads the final snapshot and launches the viewer

## Artifact and Snapshot Behavior
Headless outputs include:
- `metrics.csv`
- `fitness.svg`
- `run_summary.json`
- `snapshot_gen_<n>.json`

Viewer-originated snapshot saves use:
- `snapshot_gen_<n>_step_<m>.json`

Snapshots contain:
- generation index
- seed
- resolved config
- metrics history
- serialized population
- saved world state

## GUI Technical Requirements

### Visual Encoding
- MLP-controlled agents are circles with a forward marker.
- LTC-controlled agents are triangles rotated by heading.
- Predators are colored red.
- Prey keep controller-family coloration.
- Food is rendered as small points.
- Food clusters are rendered as faint circular overlays.
- Obstacles are rendered as red circular overlays.

### Live Statistics and Inspection
The GUI surfaces:
- generation index
- world step
- alive agent count
- food remaining
- best and mean fitness
- controller-family counts
- active evaluation schedule
- active energy-rule toggles
- day/night state when enabled
- continuation progress

The selected-agent inspector supports:
- food/prey/predator/obstacle eye-bin charts
- core sensor display
- raw controller outputs and applied throttle
- last-step energy-cost breakdown
- current light-state display
- live network topology and activation visualization
- post-death inspection using the last captured telemetry

### Controls
The GUI provides:
- pause
- continue
- single-step while paused
- simulation speed multiplier
- rule-toggle controls
- restart from the snapshot seed/config state
- save snapshot
- continuation evaluation-schedule controls
- typed continuation controls for the full runtime config surface

## Repository Orientation
Main implementation areas:
- `src/sim` for deterministic world stepping and sensing
- `src/brain` for MLP/LTC controllers and factories
- `src/genetics` for genomes, mutation, crossover, and population evolution
- `src/experiment` for generation loops and evaluation orchestration
- `src/io` for artifact and snapshot I/O
- `src/ui` for the Bevy viewer
- `src/config` for TOML config and validation
