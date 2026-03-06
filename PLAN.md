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
- [x] (2026-03-04 09:14Z) Implemented LTC controller with deterministic recurrent stepping, stable parameter clamping, mixed-family factory integration, and mixed MLP/LTC deterministic episode tests.
- [x] (2026-03-04 09:19Z) Implemented fitness scoring, genome types, mutation/crossover/tournament selection, and deterministic multi-generation GA runner with population-size and determinism tests.
- [x] (2026-03-04 09:23Z) Implemented artifact export (`metrics.csv`, `run_summary.json`, `snapshot_gen_<n>.json`), non-empty output-dir guard, snapshot read/write roundtrip, and batch artifact integration tests.
- [x] (2026-03-04 09:37Z) Implemented Bevy viewer path behind `gui` feature with MLP/LTC shape encoding (circle/triangle), food rendering, keyboard controls (pause/speed/rule toggles/restart), and live stats in window title.
- [x] (2026-03-04 09:26Z) Implemented CLI execution flows (`run`, `batch`, `view`) backed by config loading, batch artifact writing, snapshot loading, and integration tests across both headless and GUI-capable builds.
- [x] (2026-03-04 09:39Z) Validated acceptance-style CLI scenarios (`batch`, `run`, `view`) with real artifact outputs and recorded final retrospective; remaining gap is manual interactive GUI behavior verification on a desktop session.
- [x] (2026-03-04 10:45Z) Added GUI generation-continuation controls (variable generation count + evolve action), plus experiment APIs to continue evolution from snapshot populations and rebuild viewer state/visuals deterministically; verified with `cargo check --features gui` and full `cargo test`.
- [x] (2026-03-04 14:10Z) Reworked GUI continuation controls to typed numeric inputs and added editable continuation parameters (`world size`, `food count`, `episode steps`, `mutation rate`) that feed directly into in-GUI evolution runs; expanded default world size to 300x300 with unchanged population defaults.
- [x] (2026-03-04 14:45Z) Added live generation-progress visibility during evolution in both CLI (`run`/`batch` progress counters) and GUI (status text + title progress while continuation jobs run incrementally per generation).
- [x] (2026-03-04 18:15Z) Enforced species-isolated GA evolution (MLP and LTC evolve in separate pools with no interbreeding) and throttled progress feedback display cadence to every 10 generations (plus completion) for CLI and GUI progress indicators.
- [x] (2026-03-04 19:05Z) Implemented Shorelark-inspired food vision ("eyes"): angular food-intensity bins with configurable FOV, trait-driven active channel count/range, fixed-width controller sensor vectors, updated controller input shapes, and passing sensor/config validation tests.
- [x] (2026-03-04 19:30Z) Exposed controller layer sizes in TOML/config (`mlp_hidden_size`, `ltc_state_size`), validated non-zero bounds, and wired mixed-controller factory setup to use these configurable sizes.
- [x] (2026-03-04 19:55Z) Added automatic `fitness.svg` artifact generation (best + mean fitness trends per generation) for headless `run`/`batch` outputs, with integration tests verifying plot presence.
- [x] (2026-03-04 20:20Z) Added in-GUI fitness plotting by embedding generation history in snapshots and rendering best/mean fitness lines in the Bevy viewer, including continuation-time plot updates and backward-compatible snapshot deserialization.
- [x] (2026-03-06 00:00Z) Added selected-agent observability in the GUI by switching viewer stepping from stub actions to real controller execution, capturing sensor/output/energy telemetry, rendering a selected-agent inspector with eye-bin activity, and validating with `cargo fmt`, `cargo check --features gui`, and `cargo test`.
- [x] (2026-03-06 00:40Z) Added deterministic food respawn with configurable `food_respawn_probability`, exposed it in TOML and GUI continuation controls, and validated the new respawn behavior with simulation/config tests.
- [x] (2026-03-06 02:10Z) Replaced the prior LTC-inspired leaky RNN with a conductance-based LTC implementation (gated sensory/recurrent synapses, leak, capacitance, fixed internal ODE unfolds), added legacy snapshot/genome upgrade handling, and validated with `cargo fmt`, `cargo check --features gui`, and `cargo test`.

## Surprises & Discoveries

- Observation: The repository is intentionally minimal (`src/main.rs` only), so all architecture in this plan is additive and no migration compatibility layer is needed.
  Evidence: `find . -maxdepth 3 -type f` shows only base Rust project files plus docs.

- Observation: `AGENTS.md` references `docs/SPECS.md`, and that file now exists and is the authoritative product specification.
  Evidence: `docs/SPECS.md` contains full v1 spec and `.agent/SPECS.md` is not the canonical source.

- Observation: Sandbox-restricted runs could not resolve crates.io, but escalated execution successfully downloaded dependencies and completed verification.
  Evidence: Initial failures reported `Could not resolve host: index.crates.io`; subsequent escalated `cargo test` passed with 6 tests and `cargo run -- --help` displayed expected subcommands.

- Observation: The MLP golden test constants were initially incorrect at the fifth decimal place and required recomputation to align with exact tanh math.
  Evidence: `brain::mlp::tests::deterministic_forward_matches_known_values` failed once, then passed after updating expected steering/throttle constants.

- Observation: LTC golden outputs initially missed a second tanh nonlinearity in the output path and required corrected expected constants.
  Evidence: `brain::ltc::tests::deterministic_state_evolution_matches_known_sequence` failed once, then passed after recomputing expected values using the exact controller equations.

- Observation: End-to-end GA determinism required deriving per-agent evaluation seeds from run seed + generation + agent index instead of reusing one seed per generation.
  Evidence: deterministic-run tests are stable only after adding a fixed seed derivation function in `experiment::runner`.

- Observation: Metrics export and summary hashing remain deterministic only when run metadata excludes wall-clock timestamps.
  Evidence: repeated same-seed batch runs produce identical generation metrics and stable config hashes in tests.

- Observation: CLI workflow integration tests are reliable in headless CI only when the viewer call path is decoupled behind a non-GUI fallback.
  Evidence: `tests/cli_workflows.rs` exercises `run`, `batch`, and `view` without requiring a graphics context.

- Observation: Default Bevy feature set pulled in audio system dependencies (`alsa`) unavailable in this environment, causing GUI build failures.
  Evidence: `cargo check --features gui` failed in `alsa-sys` until Bevy features were reduced to a render/window subset.

- Observation: Snapshot `generation` values represent CLI run-count semantics while persisted population genomes correspond to the pre-next-evolution population, so GUI continuation needs an internal population-generation index to avoid skipping/repeating evolution steps.
  Evidence: continuity test coverage in `experiment::runner` passes only when continuation uses explicit generation-index tracking tied to the starting population.

- Observation: Keyboard shortcuts for simulation controls conflict with typed UI input unless input focus is explicitly tracked and shortcuts are gated while fields are active.
  Evidence: numeric entry produced rule-toggle side effects until `keyboard_controls` was conditioned on `active_input.is_none()`.

- Observation: GUI progress is not observable when continuation runs in one blocking call on the main thread; per-frame single-generation stepping is required to surface live progress.
  Evidence: status/title updates only become visible when evolution is processed incrementally via a persistent job state in `ViewerRuntime`.

- Observation: Simply preventing mixed-brain crossover is insufficient to guarantee species separation; parent sampling must be partitioned by controller family to avoid mixed pools at selection time.
  Evidence: species isolation is robust only after splitting `evolve_population` into independent MLP/LTC sub-evolution groups.

- Observation: Evolving trait-level `perception_channels` conflicts with static MLP/LTC input shapes unless sensor width is fixed; a compatibility layer is needed.
  Evidence: Existing snapshots/genomes carry fixed input dimensions, so the new eye model had to keep a constant input width while activating only the first `perception_channels` bins.

- Observation: The current GUI viewer loop still advances with `sim::step_world`, which uses deterministic stub actions instead of evolved controllers.
  Evidence: `src/ui/mod.rs` calls `step_world` from `advance_simulation`, and `src/sim/world.rs` defines `step_world` as a wrapper around `step_world_with_actions` that supplies `stub_action`.

- Observation: The original “LTC” implementation was only a leaky recurrent state model with fixed per-neuron time constants, not a conductance-based liquid time-constant cell.
  Evidence: Old `src/brain/ltc.rs` updated state by `(-x + target) / tau` with linear input/recurrent drives and no gated synaptic conductances or leak/capacitance terms.

- Observation: Existing Rust ODE crates (`ode_solvers`, `diffsol`, `differential_equations`) are broader and heavier than the controller loop needs; the LTC step only requires a tiny fixed-size, per-agent deterministic integrator.
  Evidence: The refactor succeeded with a fixed-count in-house unfold and all `cargo test` / `cargo check --features gui` validations passed without introducing a new dependency.

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

- Decision: Implement LTC as a stable Euler-stepped recurrent state model with per-neuron time constants and bounded parameter clamping.
  Rationale: Provides deterministic temporal dynamics with explicit numerical stability constraints suitable for mutation-based evolution.
  Date/Author: 2026-03-04 / Codex

- Decision: Add `MixedControllerFactory` to generate and build both MLP and LTC genomes through one shared factory interface.
  Rationale: Enables mixed-family population initialization and controller-kind metadata extraction for later UI/distribution reporting.
  Date/Author: 2026-03-04 / Codex

- Decision: Evaluate genomes in single-agent deterministic episodes during Milestone 5 instead of full-population shared-world episodes.
  Rationale: This keeps fitness attribution and reproducibility straightforward while still exercising controller + world integration for selection pressure.
  Date/Author: 2026-03-04 / Codex

- Decision: Enforce output-directory safety by rejecting existing non-empty `--out` directories at I/O layer entry.
  Rationale: Prevents accidental artifact mixing while preserving the existing CLI contract without extra overwrite flags.
  Date/Author: 2026-03-04 / Codex

- Decision: Introduce an explicit `app::execute_command` orchestration layer that main, tests, and future UIs can share.
  Rationale: Keeps command execution testable without spawning subprocesses and centralizes command side effects.
  Date/Author: 2026-03-04 / Codex

- Decision: Restrict Bevy dependency features in `Cargo.toml` to avoid mandatory audio backends for GUI compilation.
  Rationale: Enables deterministic GUI-path compile checks in environments without system `alsa` packages while preserving 2D rendering capabilities.
  Date/Author: 2026-03-04 / Codex

- Decision: For GUI "continue evolution", track two counters: user-facing snapshot generation (`generation`) and internal population generation index (`population_generation_index`) used for deterministic GA continuation seeding.
  Rationale: This preserves existing snapshot compatibility and avoids semantic churn in CLI artifact naming while enabling correct in-GUI continuation behavior from loaded populations.
  Date/Author: 2026-03-04 / Codex

- Decision: Use typed numeric fields for continuation controls instead of increment/decrement buttons, and apply a bounded subset of config parameters (`world_width`, `world_height`, `food_count`, `episode_steps`, `mutation_rate`) to continuation runs.
  Rationale: Typed entry is faster for large values (e.g., 1000 generations) and the selected parameters materially affect evolution behavior while keeping the UI manageable.
  Date/Author: 2026-03-04 / Codex

- Decision: Introduce progress-capable runner entrypoints and an incremental GUI evolution job loop instead of blocking continuation calls.
  Rationale: The user requirement is explicit live generation progress visibility in both terminal and GUI, which needs callback-based reporting in CLI and frame-by-frame progression in viewer mode.
  Date/Author: 2026-03-04 / Codex

- Decision: Partition evolution by `ControllerKind` and preserve per-species counts generation-to-generation.
  Rationale: This fully enforces "no interbreeding" semantics while retaining mixed-species experiments under one shared simulation run.
  Date/Author: 2026-03-04 / Codex

- Decision: Implement Shorelark-inspired food eyes as angular intensity bins over a configurable FOV (`eye_fov_angle_radians`) with fixed maximum bin capacity and per-agent active channel count.
  Rationale: This preserves deterministic, evolvable vision behavior while keeping controller genome shapes stable and backward-compatible with existing snapshots.
  Date/Author: 2026-03-04 / Codex

- Decision: Keep controller input/output dimensions fixed in code and expose only hidden/state sizes through config.
  Rationale: Input size depends on sensor contract and output size maps to steering/throttle controls; exposing hidden/state layers provides flexibility without breaking controller-action interface guarantees.
  Date/Author: 2026-03-04 / Codex

- Decision: Emit a static `fitness.svg` plot artifact during headless runs instead of introducing an in-GUI plotting dependency first.
  Rationale: This provides immediate, lightweight trend visibility for average/max fitness while preserving deterministic headless workflow and avoiding UI complexity.
  Date/Author: 2026-03-04 / Codex

- Decision: Feed the GUI plot from `SnapshotFile.metrics_history` instead of parsing `metrics.csv` at viewer startup.
  Rationale: This keeps `view --snapshot` self-contained, preserves deterministic replay behavior, and lets in-GUI continuation append new points without depending on sidecar files.
  Date/Author: 2026-03-04 / Codex

- Decision: Implement GUI observability only after making the viewer simulation controller-driven, and store only the selected agent's last-step telemetry in the runtime.
  Rationale: Eye bins, action outputs, and energy costs are only meaningful if they correspond to the controller that actually moved the agent. Persisting one selected agent's latest telemetry satisfies the inspector requirement without adding unnecessary per-frame history for the entire population.
  Date/Author: 2026-03-06 / Codex

- Decision: Replace the prior Euler-stepped leaky RNN with a conductance-based LTC cell and keep the ODE integration in-house instead of adding a general solver crate.
  Rationale: The user explicitly asked for a real LTC. A conductance-based implementation with gated sensory/recurrent synapses, leak, capacitance, and fixed internal unfolds matches LTC behavior materially better, while a custom integrator keeps per-agent stepping deterministic, dependency-light, and fast for tiny state vectors.
  Date/Author: 2026-03-06 / Codex

## Outcomes & Retrospective

Milestone 1 is complete. The repository now has module scaffolding across planned domains, CLI subcommands (`run`, `batch`, `view`) with required seed flags on headless modes, validated TOML configuration types with spec defaults, and module-level README documentation. Acceptance commands were executed successfully: unit tests pass and CLI help output shows expected command surface.

Milestone 2 is complete. The simulation core now includes `WorldState`, `AgentState`, and `FoodState` with deterministic initialization from `ChaCha8Rng`, a fixed-step update loop, toroidal wrapping, collision-driven food consumption, and energy updates gated by runtime rule toggles. Verification includes unit tests for wrapping/collision/energy toggles and integration tests proving deterministic world progression and identical episode summaries for identical seed/config pairs.

Milestone 3 is complete. The `brain` module now defines `NeuralController`, `ControllerFactory`, `ControllerKind`, and serializable `BrainGenome` with an MLP variant. The MLP implementation includes deterministic feed-forward inference, flattened parameter encode/decode routines, random genome generation, and golden-value unit tests. End-to-end tests now run complete MLP-driven simulation episodes and verify repeatable episode summaries for fixed seeds.

Milestone 4 is complete. The `brain` module now includes an LTC controller prototype with deterministic recurrent stepping, serialized LTC genomes, bounded parameter clamping for stable integration, and a mixed-family factory that produces both MLP and LTC genomes. Verification includes LTC unit tests for deterministic step outputs and clamping behavior, plus an integration test proving mixed MLP/LTC episodes produce repeatable metrics and a non-zero count of both controller kinds in one population.

Milestone 5 is complete. The repository now includes a fitness module, genetics genome/types/operators (`elitism`, `tournament selection`, `crossover`, `mutation`), and an experiment runner that executes deterministic multi-generation evolution and emits in-memory generation metrics. Verification includes unit tests for fitness weighting, mutation bounds, crossover behavior, population sizing, runner determinism, and integration tests that confirm non-empty fitness progression and stable same-seed outputs.

Milestone 6 is complete. The `io` module now writes metrics CSV, run summaries, and snapshots and can read snapshots back for reload workflows. The runner now exposes `run_batch_to_dir` for artifact-producing headless runs, and tests verify snapshot roundtrip and expected artifact creation for a short batch run.

Milestone 8 is complete for command-line workflows. CLI command handlers now run end-to-end for `batch` and `run` workflows and load snapshots for `view`, with integration tests covering happy paths and missing-snapshot failure.

Milestone 7 is complete at the implementation level. The GUI path now launches a Bevy app (when compiled with `--features gui`), renders MLP and LTC agents with distinct geometry (circle and triangle), renders food points, supports pause/resume and simulation speed control, allows runtime rule toggling and restart from snapshot seed/config state, and exposes live run statistics in the window title for continuous inspection.

Final retrospective: the core v1 pipeline is now implemented end-to-end for deterministic headless evolution, artifact persistence, snapshot reload, mixed-family controller execution, and CLI orchestration. Automated verification covers deterministic behavior, GA mechanics, artifact generation, snapshot roundtrip, and command workflows. The outstanding verification gap is manual desktop interaction testing of the Bevy window controls and visual behavior on a machine with active graphics display support.

Post-plan enhancement: GUI now supports continuing evolution by a variable number of generations directly in the viewer, with deterministic continuation APIs in `experiment::runner` and viewer-side visual rebuilds when controller families change after evolution. This closes a key interactive workflow gap that previously required returning to CLI runs.

Post-plan enhancement (v2): GUI continuation now supports typed numeric input and in-panel parameter editing for continuation runs, and project defaults now use a larger 300x300 world without increasing the default population size.

Post-plan enhancement (v3): generation-progress reporting is now visible during evolution in both command-line and GUI workflows, improving observability for long runs.

Post-plan enhancement (v4): MLP and LTC species now evolve independently without cross-species mating, and user-facing generation feedback is throttled to every 10 generations to reduce progress noise on long runs.

Post-plan enhancement (v5): controllers now receive Shorelark-inspired eye inputs (food intensity binned by relative angle and perception range), with fixed-width sensor vectors, new FOV configuration, and trait-driven active eye channel counts.

Post-plan enhancement (v6): model-capacity knobs for `MLP` and `LTC` are now runtime-configurable through TOML (`mlp_hidden_size`, `ltc_state_size`) with validation and factory wiring.

Post-plan enhancement (v7): headless runs now export `fitness.svg` to visualize best and mean fitness trends over generations without external tooling.

Post-plan enhancement (v8): the Bevy viewer now renders an internal fitness chart for best and mean fitness, sourced from snapshot-embedded metric history and extended during GUI continuation.

Post-plan enhancement (v9): the Bevy viewer now runs real controllers in-frame and exposes a selected-agent inspector with eye-bin activations, controller outputs, and energy-cost breakdown, bringing GUI simulation behavior into alignment with headless controller-driven execution.

Post-plan enhancement (v10): food now respawns deterministically on consumption according to configurable probability, with the same parameter available in both TOML and GUI continuation controls.

Post-plan enhancement (v11): the LTC controller is now a conductance-based liquid network with gated sensory/recurrent synapses, leak conductance, membrane capacitance, fixed internal ODE unfolds, and backward-compatible upgrade handling for older serialized genomes.

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
2026-03-04: Plan updated after Milestone 4 implementation to mark LTC prototype/integration complete, record stable-parameter and mixed-factory design decisions, and capture passing LTC + mixed-family determinism evidence.
2026-03-04: Plan updated after Milestone 5 implementation to mark GA/fitness loop complete, record deterministic per-agent evaluation seeding and single-agent evaluation design decisions, and capture passing population/determinism progression tests.
2026-03-04: Plan updated after Milestone 6 implementation to mark artifact I/O complete, record output-directory safety and deterministic hash decisions, and capture passing snapshot/artifact integration tests.
2026-03-04: Plan updated after CLI orchestration implementation to mark Milestone 8 headless workflows complete, record command-execution layering decisions, and capture passing CLI integration tests while deferring full Bevy viewer behavior to Milestone 7.
2026-03-04: Plan updated after Bevy viewer implementation to mark Milestone 7 complete, record GUI feature-scoping decisions, and capture passing default-test + `--features gui` compile evidence.
2026-03-04: Plan updated after acceptance-style command validation to mark final plan step complete and document the remaining manual GUI-interaction verification gap.
2026-03-04: Plan updated after GUI generation-continuation implementation to record the new viewer controls, note population-generation indexing semantics discovered during continuation work, and capture passing `cargo check --features gui` + `cargo test` verification evidence.
2026-03-04: Plan updated after typed GUI-input and continuation-parameter editing work to document focus/shortcut interactions, configurable continuation fields, and larger world-size defaults.
2026-03-04: Plan updated after progress-visibility work to document incremental GUI continuation jobs and CLI generation-progress reporting callbacks.
2026-03-04: Plan updated after species-isolated evolution and feedback-throttling changes to document the no-interbreeding evolution partitioning and 10-generation progress cadence.
2026-03-04: Plan updated after Shorelark-inspired eye-sensor implementation to document fixed-width vision-bin sensing, FOV configuration, and compatibility constraints between evolvable perception traits and static controller genome shapes.
2026-03-04: Plan updated after introducing config-driven controller layer sizes so network capacity can be tuned in `configs/default.toml` without code edits.
2026-03-04: Plan updated after adding headless fitness-plot artifact generation (`fitness.svg`) and corresponding artifact tests.
2026-03-04: Plan updated after embedding generation history in snapshots and rendering GUI fitness plots from that history.
