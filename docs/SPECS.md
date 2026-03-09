# Neural Survival v1 Specification

## Summary
`neural-survival` is a native Rust desktop application for evolving neural-network-controlled agents ("animals") in a simulated world.

Version 1 focuses on:
- Fast, deterministic, headless evolution runs.
- Visual inspection of evolved populations in a GUI.
- Side-by-side support for feed-forward MLP and Liquid Time-Constant (LTC) controllers.
- In-viewer continuation of evolution from loaded snapshots.

## Goals
1. Evolve agents with a genetic algorithm in a 2D toroidal environment.
2. Compare MLP and LTC controller families under the same world rules.
3. Model mutable agent traits that affect sensing, movement, and energy economics.
4. Allow world-rule toggles for energy consumption factors.
5. Support configurable food replenishment during episodes.
6. Support running N headless generations before opening the GUI simulation view.
7. Export per-generation metrics and run summaries for offline analysis.
8. Surface generation progress in CLI/GUI during long evolution runs.

## Non-Goals (v1)
1. Predator-prey behavior.
2. Multi-food edibility behavior in live simulation (only schema hooks are included).
3. Topology-evolving neuroevolution methods (for example NEAT).
4. GPU/distributed training.

## Product Scope

### Runtime and Delivery
- Target: native desktop application.
- Language: Rust (edition 2024).
- Rendering/UI stack: Bevy.
- Workflow: CLI-driven experiment runs and GUI visualization.

### Core Simulation
- World geometry: 2D continuous toroidal (wrap-around edges).
- Time model: fixed-step simulation.
- Initial food model: stationary food points spawned uniformly at random and optionally respawned on consumption according to configurable probability.
- Agent kinematics: steering plus bounded velocity.

### Neural Controllers
- Controller families required in v1:
  - Feed-forward MLP.
  - Conductance-based LTC network.
- Inference backend: custom lightweight deterministic Rust implementation.
- Both controller families must be usable through one shared controller interface.
- Controller capacity knobs are runtime-configurable via config:
  - `mlp_hidden_size`
  - `ltc_state_size`
- The LTC controller uses gated sensory/recurrent conductances with leak and membrane-capacitance terms, advanced by a fixed-count internal ODE unfold each simulation step.

### Evolution
- Genetic algorithm strategy:
  - Elitism.
  - Tournament selection.
  - Crossover.
  - Mutation.
- Fitness evaluation mode is configurable:
  - `solo`: each genome is scored in a private deterministic episode
  - `shared`: the current generation is scored inside a shared controller-driven world, so agents compete indirectly through food depletion
- Controller families evolve in separate pools (MLP and LTC do not interbreed).
- Reproducibility: strict seeded determinism for same seed/config/platform assumptions.

## User Workflows

### 1) Train then visualize
1. User runs `run` command with config, seed, and `headless_generations = N`.
2. Application evolves population headlessly for N generations.
3. Application launches GUI with generation-N population snapshot.
4. GUI shows the historical best/mean fitness plot carried in the snapshot and updates it if the user continues evolution from the viewer.

### 2) Batch experiments
1. User runs headless batch for many generations.
2. Application outputs metrics and run summary files.
3. User compares runs across seeds/configs offline.
4. Output directory must be absent or empty (non-empty directories are rejected to prevent artifact mixing).

### 3) Interactive inspection
1. User opens saved snapshot in GUI.
2. User inspects movement, survival, and top-level statistics.
3. User can pause/resume, adjust simulation speed, and toggle active world rules.
4. User can continue evolution from the loaded population for a typed number of generations, with editable continuation parameters (`world size`, `food count`, `food motion`, `food respawn`, `sensor noise`, `sensor dropout`, `episode steps`, `mutation rate`) and a selectable evaluation schedule (`solo`, `shared`, or `solo -> shared @ N`).
5. User can click an agent in the GUI to inspect its current food/agent eye-bin activations, controller outputs, last-step energy-cost breakdown, and live neural-network topology/activations.

## Public Interfaces and Types

### Controller Interfaces
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

### Genomes and Traits
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

### Config and Rule Toggles
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

### CLI Surface
- `neural-survival run --config <path> --seed <u64> --headless-generations <u32> --out <dir>`
- `neural-survival batch --config <path> --seed <u64> --generations <u32> --out <dir>`
- `neural-survival view --snapshot <path>`

## Simulation Specification

### World and Entities
- Food entities are points with collision radius and optional drift motion controlled by `food_motion_speed`.
- Consumed food can respawn immediately at a new random position, controlled by `food_respawn_probability` in `[0, 1]`.
- Agents have position, heading, velocity, energy, controller state, and trait values.
- World wraps in both axes.

### Sensors and Actions
- Sensor vector includes:
  - Core normalized scalar cues (`energy`, `speed`).
  - Shorelark-inspired "eye" bins: angular perception cells within a configurable field-of-view and perception range.
  - Eye bins sense both food and other agents.
  - Optional deterministic sensor corruption controlled by `sensor_noise_stddev` and `sensor_dropout_probability`.
  - Controller input uses separate channel stacks for food bins and agent bins, preserving type information for future extensions such as edible-agent versus predator-agent channels.
  - Per-agent channel count trait controls active eye bin count.
  - Per-agent `eye_cell_angle_radians` trait controls how much angle each eye/bin covers.
  - Total visible field of view is derived from evolved eye count and evolved per-eye angle.
  - Fixed controller input width where inactive eye bins are zero-filled (current width: `2 + 10 food + 10 agent = 22` values).
- Action vector controls:
  - Steering direction change.
  - Throttle/drive magnitude.

### Energy Model
For each fixed step:
`E(t+dt) = E(t) - base_cost*dt - speed_term - network_term - perception_term + food_gain`

- `speed_term` applies only when `energy_by_speed = true`.
- `network_term` applies only when `energy_by_network_size = true`.
- `perception_term` applies only when `energy_by_perception_size = true`.
- Agent dies when energy `<= 0`.

### Fitness
Per-agent episode fitness is weighted from:
- Survival duration.
- Food consumed.
- Remaining energy.

Fitness attribution is always per agent. The source episode depends on the effective evaluation mode for that generation: private single-agent episodes in `solo`, shared-population episodes in `shared`.

Evaluation mode can be scheduled across a run:
- `evaluation_mode`: initial mode
- `evaluation_mode_after_switch`: mode used after the switch point
- `evaluation_mode_switch_after_generation`: generation index where the switch becomes active
- `0` disables switching and keeps `evaluation_mode` for the whole run

Weights are config-driven and exported in run metadata.

## Genetic Algorithm Specification
- Population is evolved in discrete generations.
- Selection: tournament selection.
- Retention: elitism by top fitness percentile/count.
- Reproduction: crossover followed by mutation.
- Species isolation: parent selection and reproduction are partitioned by `ControllerKind`.
- Trait genes and brain genes are both subject to mutation and crossover.
- Mutations are bounded to valid ranges.
- All random draws use deterministic seeded RNG streams.

## GUI Specification (Bevy)

### Visual Encoding
- MLP-controlled agents: circles with a forward marker so heading is visible.
- LTC-controlled agents: triangles rotated to show heading directly.
- Food: small static points.

### Live Statistics Panel
- Current generation index.
- Alive agent count.
- Food remaining.
- Best and mean fitness.
- Controller-family distribution (MLP/LTC).
- Active world-rule toggle states.
- Evolution continuation progress while GUI-triggered evolution is running.
- Selected-agent inspector:
  - click-to-select agent inspection
  - live eye-bin chart for active perception channels, split into food and nearby-agent activations
  - core sensor values (`energy`, `speed`)
  - raw controller outputs (`steering`, `throttle_raw`) and applied throttle
  - last-step energy breakdown (`base`, `speed`, `network`, `perception`, `food`, `net`)
  - live network inspector panel showing the selected controller topology and current node activations
    - MLP: input -> hidden -> output layers
    - LTC: input -> recurrent state -> output, including recurrent connections
    - enlarged, high-contrast rendering so topology and activation changes remain readable while the simulation is running
    - panel text identifies input-channel layout, output meanings, and color semantics
  - selected agent remains inspectable after death using the last captured telemetry

### Controls
- Pause simulation.
- Single-step one simulation tick while paused.
- Continue simulation (resume).
- Simulation speed multiplier.
- Toggle energy rule flags.
- Restart from seed/config.
- Load snapshot for replay/inspection.
- Continuation evaluation schedule controls:
  - initial evaluation mode
  - post-switch evaluation mode
  - switch generation (`0` disables the schedule)
- View in-GUI fitness trend plot for best and mean fitness across generations.
- View live network topology/activation plot for the selected agent.
- Typed continuation controls:
  - generations to evolve
  - world width / world height
  - food count
  - food motion speed
  - food respawn probability
  - sensor noise standard deviation
  - sensor dropout probability
  - episode steps
  - mutation rate
- Run continuation evolution from GUI controls.
- Click an agent to select it; click empty world space to clear selection.

## Data Export

### Required Artifacts
1. `metrics.csv`
   - One row per generation.
   - Includes generation, best/mean fitness, survival stats, controller distribution, config hash.
2. `fitness.svg`
   - Line chart for best and mean fitness vs generation.
   - Written for headless runs to provide quick visual trend inspection.
3. `run_summary.json`
   - Seed, resolved config, fitness weights, runtime metadata.
4. `snapshot_gen_<n>.json`
   - Population genomes, minimal viewer start state, and embedded generation-metric history for in-GUI plotting.

### Progress Feedback
- CLI `run` and `batch` print generation progress in increments of 10 generations (and always on completion).
- GUI continuation status/title displays progress with the same 10-generation cadence (and exact completion).

## Repository Architecture (Planned)
- `src/main.rs`: CLI entry and mode dispatch.
- `src/config/`: config schema, defaults, validation.
- `src/sim/`: world model and fixed-step systems.
- `src/brain/`: controller trait + MLP/LTC implementations.
- `src/genetics/`: selection, crossover, mutation.
- `src/fitness/`: scoring logic.
- `src/experiment/`: generation loop orchestration.
- `src/ui/`: Bevy rendering and controls.
- `src/io/`: metrics/snapshot serialization.
- `tests/`: integration coverage.

## Testing and Acceptance

### Unit Tests
1. Deterministic MLP forward pass for known parameters/input.
2. Deterministic LTC stepping for known sequence.
3. Mutation bounds enforcement for each trait gene.
4. Crossover behavior correctness.
5. Energy toggle flags include/exclude terms correctly.

### Integration Tests
1. Same seed/config yields same aggregate generation metrics.
2. One-generation GA smoke test produces valid next generation.
3. Snapshot serialization round-trip preserves genomes/state.
4. CLI batch run creates expected output artifacts.

### Acceptance Scenarios
1. Run `batch` for 10 generations and verify all required outputs exist and parse.
2. Run `run --headless-generations 20` and verify GUI starts from generation-20 snapshot.
3. Toggle network-size energy rule in GUI and observe measurable population behavior/stat changes.

## Defaults and Assumptions
- Population size: 128.
- Food count: 400.
- World size: 300 x 300.
- Fixed timestep: 0.1 s.
- Episode steps: 300.
- Elitism: top 10%.
- Tournament size: 5.
- Mutation rate: 0.15 per gene.
- Default energy coefficients in `configs/default.toml`:
  - `base_energy_cost_per_second = 0.005`
  - `speed_energy_cost_factor = 0.0003`
  - `network_energy_cost_factor = 0.000008`
  - `perception_energy_cost_factor = 0.000005`
- Eye FOV angle: 1.0 radians.
- Default perception radius/channels: `10.0` / `3` (with max channels `10`).
- Default controller sizes: `mlp_hidden_size = 8`, `ltc_state_size = 8`.
- Default fitness weights: survival `0.50`, food `10.0`, energy `0.5`.
- Default energy rule toggles in `configs/default.toml`: speed/network/perception all disabled.
- v1 includes one active food type in simulation, with trait fields prepared for future multi-food compatibility.
