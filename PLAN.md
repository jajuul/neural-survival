# Build Neural Survival v1 from `docs/SPECS.md`

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agent/PLANS.md` from the repository root. Any contributor implementing this plan must re-read `.agent/PLANS.md` before starting each milestone and must keep this file self-contained.

## Purpose / Big Picture

After this change, a user can run deterministic evolutionary experiments on neural-network agents and then inspect the evolved population in a desktop simulation viewer. The user will be able to compare two controller families, feed-forward multilayer perceptrons (MLP) and liquid time-constant (LTC) networks, under identical world rules and random seed settings.

The implementation is successful when three things are demonstrably true: headless evolution runs produce stable outputs for the same seed, GUI visualization shows evolved agents with distinct shapes by network type, and experiment artifacts (`metrics.csv`, `run_summary.json`, and snapshots) are emitted and reloadable.

## Progress

- [x] (2026-03-04 08:00Z) Read `docs/SPECS.md`, `AGENTS.md`, and `.agent/PLANS.md`; established implementation scope and constraints.
- [x] (2026-03-04 08:00Z) Authored a decision-complete ExecPlan in `PLAN.md`.
- [x] (2026-03-04 08:29Z) Reconciled plan/spec ambiguities for seed ownership, output directory semantics, snapshot restart behavior, and metrics schema.
- [x] (2026-03-04 08:53Z) Implemented workspace/module scaffolding, dependency setup, and config loading; verified with `cargo test` and `cargo run -- --help`.
- [x] (2026-03-04 08:58Z) Implemented deterministic simulation core with seeded world initialization, fixed-step movement/wrapping, food collisions, and toggle-driven energy model; verified with unit and integration tests.
- [x] (2026-03-04 09:10Z) Implemented neural controller framework and deterministic MLP controller (genome encode/decode + factory + golden tests) and validated MLP-driven simulation determinism.
- [ ] Implement LTC controller with deterministic step behavior.
- [ ] Implement GA generation loop, mutation/crossover/selection, and fitness scoring.
- [ ] Implement artifact export and snapshot reload.
- [ ] Implement Bevy viewer, HUD, and runtime controls.
- [ ] Implement CLI flows (`run`, `batch`, `view`) and complete unit/integration tests.
- [ ] Validate acceptance scenarios and record final retrospective.

## Surprises & Discoveries

- Observation: The repository is intentionally minimal (`src/main.rs` only), so all architecture in this plan is additive and no migration compatibility layer is needed.
  Evidence: `find . -maxdepth 3 -type f` shows only base Rust project files plus docs.

- Observation: `AGENTS.md` references `docs/SPECS.md`, and that file now exists and is the authoritative product specification.
  Evidence: `docs/SPECS.md` contains full v1 spec and `.agent/SPECS.md` is not the canonical source.

- Observation: Sandbox-restricted runs could not resolve crates.io, but escalated execution successfully downloaded dependencies and completed verification.
  Evidence: Initial failures reported `Could not resolve host: index.crates.io`; subsequent escalated `cargo test` passed with 6 tests and `cargo run -- --help` displayed expected subcommands.

- Observation: The MLP golden test constants were initially incorrect at the fifth decimal place and required recomputation to align with exact tanh math.
  Evidence: `brain::mlp::tests::deterministic_forward_matches_known_values` failed once, then passed after updating expected steering/throttle constants.

## Decision Log

- Decision: Use a two-binary behavior model inside one crate via subcommands (`run`, `batch`, `view`) instead of separate crates.
  Rationale: Keeps startup complexity low while preserving clear user workflows and testability.
  Date/Author: 2026-03-04 / Codex

- Decision: Treat deterministic behavior as a first-class requirement and route all randomness through seeded `ChaCha8Rng` instances passed explicitly through systems.
  Rationale: Reproducibility is a core product promise and must not depend on implicit global RNG use.
  Date/Author: 2026-03-04 / Codex

- Decision: Implement a lightweight in-house numeric inference layer for MLP and LTC, avoiding heavyweight ML frameworks.
  Rationale: Direct genome mutation, deterministic stepping, and low dependency burden are more important than training-framework features in v1.
  Date/Author: 2026-03-04 / Codex

- Decision: Implement GUI with Bevy and keep simulation logic in a UI-independent `sim` module.
  Rationale: The same deterministic simulation must run headless and rendered, and this separation enforces reuse.
  Date/Author: 2026-03-04 / Codex

- Decision: Define one active food type in runtime behavior while preserving trait fields (`food_preference_mask`) for forward compatibility.
  Rationale: Matches v1 scope while preventing breaking schema changes when adding multi-food edibility later.
  Date/Author: 2026-03-04 / Codex

- Decision: Make CLI `--seed` the single authoritative run seed and keep `SimulationConfig` seed-free.
  Rationale: `docs/SPECS.md` defines `--seed` on `run` and `batch`; one authority avoids conflicting values and preserves deterministic reproducibility.
  Date/Author: 2026-03-04 / Codex

- Decision: Require output directory creation to fail when destination exists and is non-empty.
  Rationale: This prevents silent artifact mixing without extending the published CLI with extra overwrite flags.
  Date/Author: 2026-03-04 / Codex

- Decision: Include seed and resolved config payload in snapshots, and define viewer restart to use those embedded values.
  Rationale: This makes `view --snapshot` self-contained and keeps "restart from seed/config" behavior unambiguous.
  Date/Author: 2026-03-04 / Codex

- Decision: Add `bevy` as an optional dependency behind a `gui` Cargo feature during early milestones.
  Rationale: Keeps baseline dependency declared while allowing headless milestone work to compile without pulling full GUI dependency graph by default.
  Date/Author: 2026-03-04 / Codex

- Decision: Use a deterministic, math-based stub action policy (`agent_id` + `step_count`) for Milestone 2 instead of RNG-driven actions.
  Rationale: This isolates world stepping behavior from controller implementation while still exercising movement and energy dynamics in a repeatable way.
  Date/Author: 2026-03-04 / Codex

- Decision: Add `step_world_with_actions` and `basic_sensor_vector` hooks in `sim` so controller-driven episodes can be tested without coupling simulation core to any specific controller implementation.
  Rationale: Maintains separation of concerns while enabling Milestone 3 end-to-end MLP validation.
  Date/Author: 2026-03-04 / Codex

## Outcomes & Retrospective

Milestone 1 is complete. The repository now has module scaffolding across planned domains, CLI subcommands (`run`, `batch`, `view`) with required seed flags on headless modes, validated TOML configuration types with spec defaults, and module-level README documentation. Acceptance commands were executed successfully: unit tests pass and CLI help output shows expected command surface.

Milestone 2 is complete. The simulation core now includes `WorldState`, `AgentState`, and `FoodState` with deterministic initialization from `ChaCha8Rng`, a fixed-step update loop, toroidal wrapping, collision-driven food consumption, and energy updates gated by runtime rule toggles. Verification includes unit tests for wrapping/collision/energy toggles and integration tests proving deterministic world progression and identical episode summaries for identical seed/config pairs.

Milestone 3 is complete. The `brain` module now defines `NeuralController`, `ControllerFactory`, `ControllerKind`, and serializable `BrainGenome` with an MLP variant. The MLP implementation includes deterministic feed-forward inference, flattened parameter encode/decode routines, random genome generation, and golden-value unit tests. End-to-end tests now run complete MLP-driven simulation episodes and verify repeatable episode summaries for fixed seeds.

## Context and Orientation

This repository currently contains a minimal Rust executable with no domain modules. The target behavior is described in `docs/SPECS.md`, and implementation discipline is defined in `AGENTS.md` and `.agent/PLANS.md`.

Key concepts used in this plan are defined here:

`Toroidal world` means the 2D world wraps around at both edges, so moving past the right edge re-enters from the left, and moving past the top re-enters from the bottom.

`Fixed-step simulation` means simulation state always advances by a constant `dt` (for example `0.1` seconds) rather than using variable frame time; this enables deterministic comparisons.

`Genome` means the encoded parameters that produce one agent, including neural-controller parameters and scalar trait values.

`Snapshot` means a serialized file containing enough state to load and visualize a generation without recomputing prior generations.

The codebase will be organized under `src/` with domain modules:

- `src/config/` for `SimulationConfig` parsing and validation.
- `src/sim/` for world state and stepping logic.
- `src/brain/` for neural controller interfaces and implementations.
- `src/genetics/` for selection, crossover, and mutation.
- `src/fitness/` for score computation.
- `src/experiment/` for generation orchestration.
- `src/io/` for CSV/JSON/snapshot persistence.
- `src/ui/` for Bevy rendering and controls.

Each module directory will include a `README.md` describing purpose, inputs, outputs, and side effects, per `AGENTS.md`.

## Plan of Work

The implementation proceeds in nine milestones. Each milestone creates an independently testable improvement and keeps all prior tests passing.

### Milestone 1: Workspace and module scaffolding

Create the module tree and baseline dependencies so the project compiles with empty stubs. Add command-line parsing with `clap` for `run`, `batch`, and `view`, where `--seed` is required for `run` and `batch` and is the authoritative random seed for the entire run. Add `SimulationConfig` and `RuleToggles` schemas with default values from `docs/SPECS.md`, and implement TOML load/validation without a seed field in config. Create `configs/default.toml` in this milestone so later validation commands are executable as written. Add module `README.md` files at creation time.

Acceptance for this milestone is `cargo test` passing with scaffolding tests, and `cargo run -- --help` showing all subcommands.

### Milestone 2: Deterministic simulation core

Implement world types in `src/sim/` (`WorldState`, `AgentState`, `FoodState`) and a fixed-step update function that handles movement, toroidal wrapping, food collisions, and energy update equations with runtime rule toggles. Keep controller outputs as stub actions at first to validate world behavior in isolation.

Add unit tests for toroidal wrapping, collision consumption, and per-rule energy terms. Add integration tests ensuring deterministic state progression from a fixed seed and config.

Acceptance is a headless single-episode run producing stable final world statistics across repeated executions.

### Milestone 3: Neural controller framework + MLP

Define `NeuralController`, `ControllerKind`, and `ControllerFactory` in `src/brain/mod.rs`. Implement `src/brain/mlp.rs` with deterministic matrix-vector inference and bounded activation outputs mapped into `ActionVector`. Define `BrainGenome` structure for MLP weights and biases and encode/decode routines.

Add deterministic golden tests for MLP forward passes with known weights and inputs.

Acceptance is a simulation episode using MLP controllers end-to-end without panic and with repeatable results for fixed seed.

### Milestone 4: LTC prototype and production integration

First add a small prototype implementation in `src/brain/ltc.rs` that validates step dynamics and numerical stability for fixed `dt`. Use this milestone to prove deterministic behavior and prevent unstable parameter ranges from entering the runtime path.

After prototype validation, integrate LTC into `ControllerFactory`, generation initialization, and controller-kind-based rendering metadata. Add tests for reproducible LTC state evolution over a fixed input sequence and bound-clamped parameter initialization.

Acceptance is both MLP and LTC controller families running in the same generation and producing deterministic episode metrics.

### Milestone 5: Genetics and fitness loop

Implement `src/fitness/mod.rs` to compute weighted fitness from survival time, food consumed, and final energy. Implement `src/genetics/` with elitism retention, tournament selection, crossover, and mutation over both brain and trait genes, including strict bounds enforcement.

Implement `src/experiment/runner.rs` generation orchestration that evaluates populations, sorts by fitness, generates next population, and records per-generation metrics.

Acceptance is a deterministic multi-generation headless run that creates a non-empty fitness progression and preserves configured population size every generation.

### Milestone 6: Artifact export and snapshot reload

Implement `src/io/metrics.rs` to write `metrics.csv`, `src/io/summary.rs` to write `run_summary.json`, and `src/io/snapshot.rs` to write/read `snapshot_gen_<n>.json`. Include a config hash and seed in all artifacts. Define `metrics.csv` to include explicit survival statistics fields (`survival_min`, `survival_mean`, `survival_max`) in addition to fitness and controller distribution metrics. Include resolved config and seed in snapshot payloads so `view --snapshot` and in-GUI restart can run without external config lookup. For output safety, fail with a clear error when `--out` already exists and is non-empty.

Add integration tests for snapshot roundtrip and artifact presence/content checks after a short batch run.

Acceptance is `batch` writing all artifacts and `view --snapshot` loading without recomputation.

### Milestone 7: Bevy viewer and controls

Implement `src/ui/` Bevy app setup with world rendering systems. Render MLP agents as circles and LTC agents as triangles. Render food as static points. Add HUD panel with generation index, alive count, food remaining, best/mean fitness, controller distribution, and active rule toggles.

Add controls for pause/resume, speed multiplier, rule toggles, restart from seed/config, and load snapshot. In viewer mode loaded from snapshot, define restart behavior to use the snapshot-embedded seed and resolved config.

Acceptance is a manual run where toggling a rule visibly affects behavior and statistics in-session.

### Milestone 8: CLI orchestration and end-to-end behavior

Wire subcommands:

`run`: headless evolve for `--headless-generations N`, then start GUI from resulting snapshot.

`batch`: evolve for `--generations N` and export artifacts without GUI.

`view`: open GUI directly from snapshot file.

Add integration tests for CLI happy paths and key failure paths (missing config, invalid snapshot).

Acceptance is all three workflows from `docs/SPECS.md` functioning from command line.

### Milestone 9: Hardening and documentation pass

Write or refine function docs on non-obvious and high-impact boundaries, especially deterministic RNG flow, energy model computations, and LTC update equations. Ensure every module has `README.md`. Stabilize error messages and add final developer notes in `docs/SPECS.md` if behavior changed.

Acceptance is full test pass and manual acceptance scenarios completed with artifacts captured in this plan.

## Concrete Steps

Run all commands from repository root: `/home/juul/rust/neural-survival`.

1. Add dependencies and baseline scaffolding.

    cargo add clap --features derive
    cargo add serde --features derive
    cargo add serde_json toml csv
    cargo add rand rand_chacha thiserror
    cargo add bevy

   Create default config used by the validation commands:

    mkdir -p configs
    cat > configs/default.toml <<'EOF'
    world_width = 200.0
    world_height = 200.0
    timestep_seconds = 0.1
    episode_steps = 300
    population_size = 128
    food_count = 400
    elitism_fraction = 0.10
    tournament_size = 5
    mutation_rate = 0.15

    [rule_toggles]
    energy_by_speed = true
    energy_by_network_size = true
    energy_by_perception_size = true
    EOF

2. Verify CLI skeleton compiles and shows commands.

    cargo run -- --help

   Expected output excerpt:

    Usage: neural-survival <COMMAND>
    Commands:
      run
      batch
      view

3. Run test suite incrementally after each milestone.

    cargo test

   Expected output excerpt (counts will grow as implementation proceeds):

    running X tests
    test result: ok. X passed; 0 failed

4. Validate deterministic behavior during milestones 2, 5, and 8.

    cargo run -- batch --config configs/default.toml --seed 42 --generations 3 --out /tmp/ns-a
    cargo run -- batch --config configs/default.toml --seed 42 --generations 3 --out /tmp/ns-b

   Expected behavior: `metrics.csv` and `run_summary.json` are byte-identical or semantically identical where timestamps are intentionally excluded from hashed payload fields.

5. Validate run-then-view behavior at milestones 7 and 8.

    cargo run -- run --config configs/default.toml --seed 42 --headless-generations 20 --out /tmp/ns-run

   Expected behavior: headless phase completes, then GUI opens with generation 20 population and restart uses seed/config embedded in the loaded snapshot.

## Validation and Acceptance

Acceptance is complete only when all checks below pass:

1. Unit tests cover deterministic MLP forward pass, deterministic LTC stepping, trait mutation bounds, crossover behavior, and per-rule energy term toggling.

2. Integration tests cover deterministic seeded generation stats, snapshot read/write roundtrip, and CLI artifact production.

3. Manual GUI checks confirm:
   - Agent shape matches controller kind (circle for MLP, triangle for LTC).
   - HUD metrics update during simulation.
   - Toggling network-size energy rule changes observed behavior over time.

4. User workflow checks confirm:
   - `batch` mode emits required artifacts.
   - `run --headless-generations N` fast-forwards then opens viewer at generation `N`.
   - `view --snapshot` loads prior run data.

5. Determinism checks confirm repeated same-seed runs produce identical generation metrics under same platform/toolchain assumptions.

## Idempotence and Recovery

All steps are additive and safe to rerun. Re-running scaffold and module creation steps should overwrite only intended generated files in the working tree. If a milestone fails partway, fix code and rerun `cargo test`; no database or destructive migration exists in v1.

For artifact outputs, if an output directory already exists and is non-empty, implementations must fail with a clear message. This prevents accidental data mixing across experiment runs without changing the public CLI contract.

If GUI issues block progress in headless environments, continue validating with `batch` and unit/integration tests, then perform GUI validation on a machine with graphics support before closing the milestone.

## Artifacts and Notes

Record milestone evidence snippets here as work proceeds. Keep excerpts short.

Example expected artifact tree after a short batch run:

    /tmp/ns-a/
      metrics.csv
      run_summary.json
      snapshot_gen_0003.json

Example expected `metrics.csv` header:

    generation,best_fitness,mean_fitness,survival_min,survival_mean,survival_max,alive_count,controller_mlp,controller_ltc,config_hash

This section must be updated during implementation with real command outputs and any deviations from expected behavior.

## Interfaces and Dependencies

The following interfaces and types must exist by end of implementation.

In `src/brain/mod.rs`, define:

    pub trait NeuralController {
        fn reset(&mut self);
        fn step(&mut self, input: &SensorVector, dt: f32) -> ActionVector;
        fn parameter_count(&self) -> usize;
        fn controller_kind(&self) -> ControllerKind;
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
    pub enum ControllerKind { Mlp, Ltc }

In `src/sim/types.rs`, define:

    pub struct SensorVector { pub values: Vec<f32> }
    pub struct ActionVector { pub steering: f32, pub throttle: f32 }
    pub struct AgentState { /* id, position, heading, velocity, energy, traits, genome */ }
    pub struct FoodState { /* id, position, energy_value */ }
    pub struct WorldState { /* dimensions, agents, food, step_count */ }

In `src/config/types.rs`, define:

    pub struct RuleToggles {
        pub energy_by_speed: bool,
        pub energy_by_network_size: bool,
        pub energy_by_perception_size: bool,
    }

    pub struct SimulationConfig {
        pub world_width: f32,
        pub world_height: f32,
        pub timestep_seconds: f32,
        pub episode_steps: u32,
        pub population_size: usize,
        pub food_count: usize,
        pub elitism_fraction: f32,
        pub tournament_size: usize,
        pub mutation_rate: f32,
        pub rule_toggles: RuleToggles,
    }

The seed is not part of `SimulationConfig`; parse it from CLI (`--seed`) and pass it explicitly through runner entrypoints.

In `src/genetics/genome.rs`, define:

    pub struct TraitGenome {
        pub max_speed: f32,
        pub perception_radius: f32,
        pub perception_channels: u16,
        pub metabolic_efficiency: f32,
        pub food_preference_mask: u32,
    }

    pub enum BrainGenome {
        Mlp(MlpGenome),
        Ltc(LtcGenome),
    }

    pub struct AgentGenome {
        pub controller_kind: ControllerKind,
        pub brain: BrainGenome,
        pub traits: TraitGenome,
    }

In `src/experiment/runner.rs`, define:

    pub struct GenerationMetrics { /* generation index and aggregate stats */ }
    pub fn run_batch(config: &SimulationConfig, seed: u64, out_dir: &std::path::Path) -> anyhow::Result<()>;
    pub fn run_headless_then_view(config: &SimulationConfig, seed: u64, generations: u32, out_dir: &std::path::Path) -> anyhow::Result<()>;

In `src/io/snapshot.rs`, define:

    pub struct SnapshotFile {
        pub generation: u32,
        pub seed: u64,
        pub resolved_config: SimulationConfig,
        pub population: Vec<AgentGenome>,
        pub world_state: WorldState,
    }

Dependencies to add in `Cargo.toml`:

- `bevy` for rendering and runtime systems.
- `clap` for CLI.
- `serde`, `serde_json`, `toml`, and `csv` for configuration and artifacts.
- `rand` and `rand_chacha` for deterministic seeded RNG.
- `thiserror` (and optionally `anyhow`) for structured error handling.

## Revision Note

2026-03-04: Initial creation of this ExecPlan based on `docs/SPECS.md` at user request. No implementation changes have started yet.
2026-03-04: Plan revised to remove seed ambiguity (`--seed` is authoritative), define non-empty output directory failure semantics, add explicit creation of `configs/default.toml`, define snapshot-backed restart behavior in viewer mode, and expand metrics schema to include survival stats. This change was made to align implementation guidance with `docs/SPECS.md` and reduce integration risk.
2026-03-04: Plan updated after Milestone 1 implementation to reflect completed scaffolding work, document crates.io DNS limitations blocking acceptance command execution, and record the temporary `bevy` optional-feature dependency choice for staged delivery.
2026-03-04: Plan updated after successful escalated verification to mark Milestone 1 complete and replace temporary acceptance blockers with actual passing command evidence.
2026-03-04: Plan updated after Milestone 2 implementation to mark deterministic simulation core complete, record deterministic stub-action design choice, and capture passing unit/integration test evidence for fixed-step world behavior.
2026-03-04: Plan updated after Milestone 3 implementation to mark controller framework + MLP complete, record simulation hooks for controller-driven stepping, and capture passing MLP unit/integration determinism evidence.
