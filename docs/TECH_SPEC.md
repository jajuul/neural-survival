# Neural Survival v1 Technical Specification

## Purpose
This document describes how `neural-survival` is expected to work internally. It captures the technical contracts, simulation model, controller interfaces, configuration surface, and repository structure that support the user-visible behavior described in `docs/BEHAVIOR_SPEC.md`.

## Runtime and Delivery
- Target platform: native desktop application.
- Primary language: Rust edition 2024.
- Rendering and UI: Bevy.
- Workflow model: CLI-driven experiment runs and GUI visualization.

## Core Simulation Model

### World
- The world is a 2D continuous torus with wrap-around edges.
- Simulation advances in fixed time steps.
- Food spawns uniformly at random in the initial world state.
- Food may remain stationary or drift according to `food_motion_speed`.
- Food may respawn immediately on consumption according to `food_respawn_probability`.
- Food may also spawn according to `food_spawn_rate_per_second`, a deterministic foods-per-simulated-second schedule.

### Agents
- Agents carry position, heading, velocity, energy, controller state, and evolved trait values.
- Movement is driven by steering and bounded velocity.
- Agents die when energy is less than or equal to zero.

### Day and Night
- The optional day/night cycle is deterministic.
- The phase is derived from `world.step_count`.
- In v1, day/night only affects sensing visibility.
- Night reduces effective perception range and eye-bin intensity through `night_visibility_multiplier`.

## Neural Controllers

### Supported Families
- Feed-forward MLP.
- Conductance-based LTC network.

### Shared Contracts
Both controller families must be usable through one shared controller interface:

```rust
pub trait NeuralController {
    fn reset(&mut self);
    fn step(&mut self, input: &SensorVector, dt: f32) -> ActionVector;
    fn step_with_observability(
        &mut self,
        input: &SensorVector,
        dt: f32,
    ) -> ControllerStepOutput;
    fn inspect_network(&self, input: &SensorVector) -> ControllerNetworkSnapshot;
    fn step_with_diagnostics(
        &mut self,
        input: &SensorVector,
        dt: f32,
    ) -> ControllerDiagnostics;
    fn parameter_count(&self) -> usize;
    fn controller_kind(&self) -> ControllerKind;
}

pub enum ControllerKind {
    Mlp,
    Ltc,
}

pub struct ControllerStepOutput {
    pub action: ActionVector,
    pub steering_raw: f32,
    pub throttle_raw: f32,
    pub throttle_applied: f32,
}

pub struct ControllerDiagnostics {
    pub output: ControllerStepOutput,
    pub network: ControllerNetworkSnapshot,
}

pub trait ControllerFactory {
    fn random_genome(&self, rng: &mut StdRng) -> BrainGenome;
    fn build(&self, genome: &BrainGenome) -> Box<dyn NeuralController>;
}
```

### Capacity Knobs
- `mlp_hidden_size`
- `ltc_state_size`

### LTC Stepping
The LTC controller uses gated sensory and recurrent conductances with leak and membrane-capacitance terms. Each simulation step advances the LTC state by running a fixed-count internal ODE unfold.

For each recurrent state neuron, one unfold computes the next membrane value as a conductance-weighted average:

    v_next =
        (cm_scaled * v_prev
         + gleak * vleak
         + sum_sensory(g * erev)
         + sum_recurrent(g * erev))
        /
        (cm_scaled
         + gleak
         + sum_sensory(g)
         + sum_recurrent(g))

where `g = weight * sigmoid((source - mu) * sigma)`. Sensory and recurrent weights are nonnegative conductance magnitudes, `mu` and `sigma` control gate position and sharpness, `erev` is the reversal potential each synapse pulls toward, `gleak` and `vleak` define passive leak, and `cm` is the membrane-capacitance or inertia term. The readout layer remains a deterministic `tanh` projection from recurrent state to steering/throttle outputs.

## Genomes and Traits
Agents evolve both brain parameters and mutable traits:

```rust
pub struct AgentGenome {
    pub brain: BrainGenome,
    pub traits: TraitGenome,
}

pub struct TraitGenome {
    pub max_speed: f32,
    pub perception_radius: f32,
    pub perception_channels: u16,
    pub eye_cell_angle_radians: f32,
    pub metabolic_efficiency: f32,
    pub food_preference_mask: u32,
}
```

## Configuration and Rule Toggles

```rust
pub struct SimulationConfig {
    // world, timing, food, GA, and fitness fields
    pub evaluation_mode: EvaluationMode,
    pub evaluation_mode_after_switch: EvaluationMode,
    pub evaluation_mode_switch_after_generation: u32,
}

pub enum EvaluationMode {
    Solo,
    Shared,
}

pub struct RuleToggles {
    pub energy_by_speed: bool,
    pub energy_by_network_size: bool,
    pub energy_by_perception_size: bool,
}
```

The config owns the world, timing, food, genetic algorithm, controller sizing, and fitness settings. Evaluation mode can be scheduled across a run with an initial mode, an optional post-switch mode, and a generation index where the switch becomes active. A switch value of `0` disables scheduling and keeps the initial mode for the entire run.

## CLI Surface
- `neural-survival run --config <path> --seed <u64> --headless-generations <u32> --out <dir>`
- `neural-survival batch --config <path> --seed <u64> --generations <u32> --out <dir>`
- `neural-survival view --snapshot <path>`

## Sensor and Action Model

### Sensor Vector
The controller input is a fixed-width vector composed of:
- normalized scalar cues for `energy` and `speed`
- food eye-bin activations
- agent eye-bin activations

The current input width is `22`, organized as `2 + 10 food + 10 agent`. Inactive eye bins are zero-filled so all controllers receive the same input width regardless of the evolved active channel count.

Eye-bin behavior:
- eye bins are angular perception cells limited by range and field of view
- eye bins sense both food and other agents
- the channel count trait controls the active number of eye bins
- `eye_cell_angle_radians` controls angular coverage per eye bin
- total visible field of view is derived from active eye count and per-eye angle
- day/night can attenuate effective sensing
- deterministic sensor corruption may be applied through `sensor_noise_stddev` and `sensor_dropout_probability`

### Action Vector
The controller output drives:
- steering direction change
- throttle or drive magnitude

## Energy Model
For each fixed step:

`E(t+dt) = E(t) - base_cost*dt - speed_term - network_term - perception_term + food_gain`

Term activation:
- `speed_term` applies only when `energy_by_speed = true`
- `network_term` applies only when `energy_by_network_size = true`
- `perception_term` applies only when `energy_by_perception_size = true`

## Fitness Model
Per-agent fitness is a weighted combination of:
- survival duration
- food consumed
- remaining energy

Fitness is always attributed per agent. The episode source depends on the effective evaluation mode for that generation:
- `solo`: private single-agent deterministic episode
- `shared`: shared world episode where agents compete indirectly through food depletion

The effective evaluation mode can switch during a run using:
- `evaluation_mode`
- `evaluation_mode_after_switch`
- `evaluation_mode_switch_after_generation`

Fitness weights are config-driven and must be exported in run metadata.

## Genetic Algorithm
- Population evolves in discrete generations.
- Selection uses tournament selection.
- Retention uses elitism by top fitness percentile or count.
- Reproduction uses crossover followed by mutation.
- Parent selection and reproduction are partitioned by `ControllerKind`.
- Both trait genes and brain genes participate in crossover and mutation.
- Mutations are bounded to valid ranges.
- All random draws use deterministic seeded RNG streams.

Implementation target details:
- headless throughput should be optimized for deterministic runs
- `solo` evaluation may be parallelized deterministically across genomes
- sensing should use fixed-width controller inputs and a coarse toroidal spatial index to reduce repeated full-world scans

## GUI Technical Requirements

### Visual Encoding
- MLP-controlled agents are circles with a forward marker.
- LTC-controlled agents are triangles rotated by heading.
- Food is rendered as small points.

### Live Statistics and Inspection
The GUI must surface:
- generation index
- alive agent count
- food remaining
- best and mean fitness
- controller-family distribution
- active rule toggles
- light phase and visibility multiplier when day/night is enabled
- continuation progress

The selected-agent inspector must support:
- eye-bin charts split into food and agent activations
- core sensor display
- current light-state display
- raw controller outputs and applied throttle
- last-step energy-cost breakdown
- live network topology and activation visualization
- post-death inspection using the last captured telemetry

### Controls
The GUI must provide:
- pause and resume
- single-step while paused
- simulation speed multiplier
- rule toggle controls
- restart from seed and config
- snapshot load
- continuation evaluation schedule controls
- typed continuation controls for generations, world size, food count, food spawn rate, food motion speed, food respawn probability, sensor noise, sensor dropout, episode steps, and mutation rate

## Data Export

### Required Artifacts
1. `metrics.csv`
   - one row per generation
   - includes generation, best fitness, mean fitness, survival stats, controller distribution, and config hash
2. `fitness.svg`
   - line chart for best and mean fitness versus generation
3. `run_summary.json`
   - seed, resolved config, fitness weights, and runtime metadata
4. `snapshot_gen_<n>.json`
   - population genomes
   - minimal viewer start state
   - embedded generation-metric history

### Progress Feedback
- CLI `run` and `batch` print progress every 10 generations and on completion.
- GUI continuation reports progress at the same cadence and on completion.

## Repository Architecture
- `src/main.rs`: CLI entry and mode dispatch
- `src/config/`: config schema, defaults, validation
- `src/sim/`: world model and fixed-step systems
- `src/brain/`: controller trait plus MLP and LTC implementations
- `src/genetics/`: selection, crossover, mutation
- `src/fitness/`: scoring logic
- `src/experiment/`: generation loop orchestration
- `src/ui/`: Bevy rendering and controls
- `src/io/`: metrics and snapshot serialization
- `tests/`: integration coverage

## Testing and Acceptance Targets

### Unit Tests
1. Deterministic MLP forward pass for known parameters and input.
2. Deterministic LTC stepping for a known sequence.
3. Mutation bounds enforcement for each trait gene.
4. Crossover behavior correctness.
5. Energy toggle flags include and exclude terms correctly.

### Integration Tests
1. The same seed and config yield the same aggregate generation metrics.
2. A one-generation GA smoke test produces a valid next generation.
3. Snapshot serialization round-trips genomes and state.
4. A CLI batch run creates the expected output artifacts.

## Defaults and Assumptions
- population size: 128
- food count: 400
- world size: 300 x 300
- fixed timestep: 0.1 seconds
- episode steps: 300
- elitism: top 10%
- tournament size: 5
- mutation rate: 0.15 per gene
- `base_energy_cost_per_second = 0.005`
- `speed_energy_cost_factor = 0.0003`
- `network_energy_cost_factor = 0.000008`
- `perception_energy_cost_factor = 0.000005`
- default perception radius: `10.0`
- default perception channels: `3`
- maximum channels: `10`
- default controller sizes: `mlp_hidden_size = 8`, `ltc_state_size = 8`
- default fitness weights: survival `0.50`, food `10.0`, energy `0.5`
- default energy rule toggles: speed, network, and perception disabled
- v1 includes one active food type, with schema fields reserved for future multi-food support
