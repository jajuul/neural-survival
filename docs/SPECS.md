# Neural Survival v1 Specification

## Summary
`neural-survival` is a native Rust desktop application for evolving neural-network-controlled agents ("animals") in a simulated world.

Version 1 focuses on:
- Fast, deterministic, headless evolution runs.
- Visual inspection of evolved populations in a GUI.
- Side-by-side support for feed-forward MLP and Liquid Time-Constant (LTC) controllers.

## Goals
1. Evolve agents with a genetic algorithm in a 2D toroidal environment.
2. Compare MLP and LTC controller families under the same world rules.
3. Model mutable agent traits that affect sensing, movement, and energy economics.
4. Allow world-rule toggles for energy consumption factors.
5. Support running N headless generations before opening the GUI simulation view.
6. Export per-generation metrics and run summaries for offline analysis.

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
- Initial food model: stationary food points spawned uniformly at random.
- Agent kinematics: steering plus bounded velocity.

### Neural Controllers
- Controller families required in v1:
  - Feed-forward MLP.
  - LTC network.
- Inference backend: custom lightweight deterministic Rust implementation.
- Both controller families must be usable through one shared controller interface.

### Evolution
- Genetic algorithm strategy:
  - Elitism.
  - Tournament selection.
  - Crossover.
  - Mutation.
- Reproducibility: strict seeded determinism for same seed/config/platform assumptions.

## User Workflows

### 1) Train then visualize
1. User runs `run` command with config, seed, and `headless_generations = N`.
2. Application evolves population headlessly for N generations.
3. Application launches GUI with generation-N population snapshot.

### 2) Batch experiments
1. User runs headless batch for many generations.
2. Application outputs metrics and run summary files.
3. User compares runs across seeds/configs offline.

### 3) Interactive inspection
1. User opens saved snapshot in GUI.
2. User inspects movement, survival, and top-level statistics.
3. User toggles active world rules and reruns from seed/config.

## Public Interfaces and Types

### Controller Interfaces
```rust
pub trait NeuralController {
    fn reset(&mut self);
    fn step(&mut self, input: &SensorVector, dt: f32) -> ActionVector;
    fn parameter_count(&self) -> usize;
    fn controller_kind(&self) -> ControllerKind;
}

pub enum ControllerKind {
    Mlp,
    Ltc,
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
    pub metabolic_efficiency: f32,
    pub food_preference_mask: u32,
}
```

### Config and Rule Toggles
```rust
pub struct SimulationConfig {
    // world, timing, food, GA, and fitness fields
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
- Food entities are stationary points with collision radius.
- Agents have position, heading, velocity, energy, controller state, and trait values.
- World wraps in both axes.

### Sensors and Actions
- Sensor vector includes:
  - Direction and distance cues for nearby edible food.
  - Direction and distance cue for nearest other agent.
  - Internal state cue (normalized energy).
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

Weights are config-driven and exported in run metadata.

## Genetic Algorithm Specification
- Population is evolved in discrete generations.
- Selection: tournament selection.
- Retention: elitism by top fitness percentile/count.
- Reproduction: crossover followed by mutation.
- Trait genes and brain genes are both subject to mutation and crossover.
- Mutations are bounded to valid ranges.
- All random draws use deterministic seeded RNG streams.

## GUI Specification (Bevy)

### Visual Encoding
- MLP-controlled agents: circles.
- LTC-controlled agents: triangles.
- Food: small static points.

### Live Statistics Panel
- Current generation index.
- Alive agent count.
- Food remaining.
- Best and mean fitness.
- Controller-family distribution (MLP/LTC).
- Active world-rule toggle states.

### Controls
- Pause/resume.
- Simulation speed multiplier.
- Toggle energy rule flags.
- Restart from seed/config.
- Load snapshot for replay/inspection.

## Data Export

### Required Artifacts
1. `metrics.csv`
   - One row per generation.
   - Includes generation, best/mean fitness, survival stats, controller distribution, config hash.
2. `run_summary.json`
   - Seed, resolved config, fitness weights, runtime metadata.
3. `snapshot_gen_<n>.json`
   - Population genomes and minimal state for viewer playback/start.

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
- v1 includes one active food type in simulation, with trait fields prepared for future multi-food compatibility.
