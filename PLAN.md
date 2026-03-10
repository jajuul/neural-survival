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
- [x] (2026-03-06 22:01Z) Switched generation scoring from private single-agent episodes to deterministic shared-world population episodes with synchronous action collection, added a competition-focused unit test, and revalidated with `cargo fmt`, `cargo check --features gui`, and `cargo test`.
- [x] (2026-03-06 22:20Z) Extended the eye model to sense nearby agents as well as food, kept controller input width stable by feeding signed `food-agent` eye bins, updated the GUI inspector to show decomposed food/agent activations, and revalidated with `cargo fmt`, `cargo check --features gui`, and `cargo test`.
- [x] (2026-03-07 00:10Z) Reworked eyes to use separate food and agent controller channels, added configurable `evaluation_mode` (`solo`/`shared`) with a GUI toggle for continuation runs, restored `solo` as the default evaluation regime, and revalidated with `cargo fmt`, `cargo check --features gui`, and `cargo test`.
- [x] (2026-03-07 00:45Z) Added scheduled evaluation-mode switching (`evaluation_mode`, `evaluation_mode_after_switch`, `evaluation_mode_switch_after_generation`) to config, runner, and GUI so curricula such as `solo -> shared @ 100` are reproducible without manual toggling.
- [x] (2026-03-07 01:15Z) Removed absolute position from controller inputs, added deterministic sensor noise/dropout plus moving food as configurable simulation parameters, exposed the new knobs in the GUI, and revalidated with `cargo fmt`, `cargo check --features gui`, and `cargo test`.
- [x] (2026-03-09 00:00Z) Added explicit `Pause`, `Step`, and `Continue` GUI controls, plus a selected-agent network inspector that renders real-time MLP/LTC topology activations from live controller diagnostics; validated with `cargo fmt`, `cargo check --features gui`, and `cargo test`.
- [x] (2026-03-09 00:30Z) Optimized evolution throughput by switching controller inputs to fixed-width sensor arrays, adding per-step spatial indexing for food/agent sensing, reusing shared-evaluation action/sensor buffers, and parallelizing deterministic `solo` evaluation with `rayon`; validated with `cargo fmt`, `cargo check --features gui`, and `cargo test`.
- [x] (2026-03-10 00:20Z) Implemented milestone 1 of temporal ecology pressure: added a deterministic day/night cycle with configurable night visibility, applied light-level scaling to sensing, surfaced current light state in the GUI title/inspector, and validated with `cargo fmt`, `cargo check --features gui`, and `cargo test`.
- [x] (2026-03-10 01:10Z) Implemented milestone 2 of temporal ecology pressure: added prey/predator roles to genomes and runtime state, upgraded eyes to typed `food`/`prey`/`predator` channels, preserved `(ControllerKind, AgentRole)` pools during evolution, added predator capture rules plus role-aware fitness scoring, and validated with `cargo fmt`, `cargo check --features gui`, and `cargo test`.
- [x] (2026-03-10 02:00Z) Implemented milestone 3 of temporal ecology pressure: added deterministic moving food clusters with per-cluster phase offsets, phase-driven spatial concentration, cluster-aware respawn/layout, and GUI cluster overlays plus food coloring; validated with `cargo fmt`, `cargo check --features gui`, and `cargo test`.

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

- Observation: Shared-world evaluation changes selection pressure immediately even without explicit agent sensors, because food availability becomes non-stationary through competition.
  Evidence: `experiment::runner::tests::shared_world_evaluation_introduces_food_competition` now proves two colocated agents can only claim one shared food item during evaluation.

- Observation: Adding direct agent sensing did not require changing controller input width; feeding signed `food_bin - agent_bin` values preserves existing genome shapes and snapshot compatibility.
  Evidence: Initial implementation used a signed merge, but controller behavior regressed (LTC spinning) after shared evaluation + agent sensing were combined.

- Observation: Separate typed food/prey/predator eye channels are a better fit than a generic agent channel once ecological roles become explicit.
  Evidence: The revised sensor layout preserves all three modalities explicitly and now uses a local-only `32`-input controller vector (`energy`, `speed`, `10` food bins, `10` prey bins, `10` predator bins), matching the role-aware world rules without aliasing semantics.

- Observation: Phase-sensitive food clusters are easier to keep deterministic when productivity changes spatial concentration rather than teleporting food counts between clusters.
  Evidence: The implemented model keeps food assigned to moving cluster centers and varies cluster radius by phase, which satisfied the density-change proof tests while preserving stable resource-region identities and snapshot compatibility.

- Observation: Live network visualization in the GUI requires controller-specific activation snapshots captured during stepping; action outputs alone are not enough to reconstruct hidden/state activations after the fact.
  Evidence: The new viewer inspector only became possible after extending `NeuralController` with `inspect_network`/`step_with_diagnostics` and storing the selected agent's latest network snapshot in `ViewerRuntime`.

- Observation: Shared-world sensing cost was dominated by repeated full-world food/agent scans and per-call sensor `Vec` allocations.
  Evidence: Prior to the refactor, `basic_sensor_vector` rebuilt decomposed `Vec`s via `sensor_observation` for every agent step, and shared evaluation re-scanned all food/agents once per alive controller each tick.

- Observation: Night visibility tests are brittle if they assume a target exactly on the reduced visibility boundary still contributes signal.
  Evidence: The first `night_visibility_reduces_sensor_strength` test failed because a target at exactly `perception_radius * night_visibility_multiplier` correctly yielded zero contribution under the implemented falloff rule.

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

- Decision: Milestone 5 initially used single-agent deterministic episodes instead of full-population shared-world episodes.
  Rationale: That was the simplest way to keep fitness attribution and reproducibility straightforward for the first GA implementation.
  Date/Author: 2026-03-04 / Codex

- Decision: Replace private single-agent evaluation with synchronous shared-world population evaluation while keeping per-agent fitness attribution and the rest of the GA unchanged.
  Rationale: This is the smallest code change that introduces competition-driven temporal non-stationarity, which is more likely to reward recurrent controllers without rewriting species separation, metrics, or artifact formats.
  Date/Author: 2026-03-06 / Codex

- Decision: Represent direct agent sensing in the controller input as signed eye bins (`food - agents`) and keep the decomposed food/agent activations only in observability structures.
  Rationale: This was the smallest backward-compatible experiment, but it aliased two different perceptual causes into one scalar and made controller behavior harder to interpret.
  Date/Author: 2026-03-06 / Codex

- Decision: Replace the signed-merged eye input with separate food and agent channel stacks, and add `evaluation_mode` so `solo` remains the default while `shared` stays available for comparison.
  Rationale: Backward compatibility is no longer a goal. Separate channels preserve information needed for future extensions such as edible/prey/predator distinctions, and a switchable evaluation regime makes behavior comparisons reproducible instead of conflating sensor and scoring changes.
  Date/Author: 2026-03-07 / Codex

- Decision: Promote `solo -> shared` curricula to explicit config/runtime schedule fields instead of manual GUI timing.
  Rationale: The observed LTC behavior depends materially on when shared-world pressure is introduced, so the regime transition must be encoded in config and reused by both CLI and GUI runs to remain reproducible.
  Date/Author: 2026-03-07 / Codex

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

- Decision: Extend the controller interface with explicit network-diagnostics hooks and render only the selected agent's topology/activations in the GUI.
  Rationale: Full-population network snapshots every frame would add unnecessary overhead and UI noise. The selected-agent path keeps observability focused and cheap while still exposing the exact hidden/state activity that produced the visible action.
  Date/Author: 2026-03-09 / Codex

- Decision: Keep GUI observability on the richer `SensorObservation` path, but move evaluation/controller stepping onto fixed-width `SensorVector` arrays backed by a per-step spatial index and deterministic parallel `solo` evaluation.
  Rationale: This preserves the detailed UI data model while cutting the main headless overheads without changing controller behavior or output determinism.
  Date/Author: 2026-03-09 / Codex

- Decision: For milestone 1, apply day/night only to sensing visibility and GUI reporting, not to movement, energy, or food dynamics.
  Rationale: This isolates the first temporal mechanic to one subsystem, keeps the milestone independently testable, and avoids conflating lighting with later predator/cluster/momentum changes.
  Date/Author: 2026-03-10 / Codex

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

Post-plan enhancement (v6): the viewer now includes explicit `Pause`, `Step`, and `Continue` controls plus a selected-agent network inspector that visualizes MLP hidden/output activations or LTC recurrent-state/output activations live during simulation.

Post-plan enhancement (v7): headless evolution now runs faster by using fixed-width sensor arrays, a coarse toroidal spatial index for eye queries, reusable per-step action/sensor buffers in shared evaluation, and deterministic parallelism for `solo` scoring.

## Next Planned Milestone: Temporal Ecology Pressure

The next feature set should make the environment materially more temporal so recurrent controllers have a clearer opportunity to outperform feed-forward controllers. After this milestone, the user should be able to run the same simulation with four new behavior-shaping systems enabled: a deterministic day/night cycle that reduces visibility at night, explicit predator/prey ecology with typed sensing channels, moving food clusters whose abundance changes by phase, and momentum-based movement where throttle affects acceleration instead of instantly setting speed. The expected visible result is that agents must remember recent observations, track moving resource zones, and react differently to prey versus predators over multiple steps rather than only frame-by-frame.

This work touches the same core areas repeatedly, so the repository orientation matters. `src/config/types.rs` and `src/config/mod.rs` own validated runtime parameters. `src/sim/types.rs` and `src/sim/world.rs` define agent/world state, sensing, food motion, collisions, and energy. `src/genetics/genome.rs`, `src/genetics/operators.rs`, and `src/genetics/population.rs` define what traits are inherited and how population partitions are preserved. `src/experiment/runner.rs` is where evaluation mode and per-role fitness attribution must remain deterministic. `src/ui/mod.rs` is the viewer surface that will need to expose the current day/night phase, typed eye bins, and momentum-driven motion clearly enough to verify behavior by eye.

The first design decision for this milestone is to keep the new temporal mechanics deterministic and config-driven rather than emergent from random timers. The day/night system should use a fixed cycle length measured in simulation steps, with a smooth phase variable in `[0, 1)` derived from `world.step_count`. Visibility should not switch abruptly from full day to full night; instead the simulation should compute a scalar `visibility_multiplier` from the phase and apply it to perception range and eye intensity. A simple sinusoidal or cosine-shaped cycle is sufficient, but the plan should treat the result as one canonical helper in `src/sim/world.rs` so every system reads the same light level. The config surface should include at least `day_night_cycle_steps`, `night_visibility_multiplier`, and a boolean enable/disable flag so old experiments remain reproducible.

The second design decision is to make predator/prey roles explicit runtime categories rather than trying to infer them from controller kind. Controller family (`MLP` vs `LTC`) and ecological role (`prey` vs `predator`) should remain orthogonal so comparisons stay meaningful. The clean implementation path is to add a new `AgentRole` enum and store it on `TraitGenome` and `AgentState`, then preserve that role across reproduction exactly the same way controller family is preserved today. That implies the evolution partition will become role-aware as well: reproduction should operate within `(ControllerKind, AgentRole)` pools so predators do not silently turn into prey through crossover or mutation. Sensing should expand from the current two vision channels (`food`, `agents`) to three channels: `food`, `prey_agents`, and `predator_agents`. The flattened controller input should stay fixed-width with zero-filled inactive bins, but the width will increase accordingly. The GUI selected-agent inspector should show these channels distinctly, and the selected-agent summary should include the current role.

Predator/prey behavior must be meaningful at the world-rules level or the extra channels are cosmetic. The simplest robust rule set is: prey consume food as before; predators gain energy by colliding with prey; prey die or lose a large fixed energy amount on predator collision; predator-predator and prey-prey collisions do not transfer energy. Because role-specific behavior changes what “good fitness” means, `src/fitness/mod.rs` and `src/experiment/runner.rs` will need role-aware scoring. Prey fitness should still emphasize food, survival, and final energy. Predator fitness should emphasize successful prey captures, survival, and final energy, with config-driven weights so experiments can tune predation pressure explicitly. The metrics output should report prey/predator counts and role-specific mean/best fitness the same way it already splits by controller family.

The third design decision is to move from independent food particles to deterministic food clusters. The concrete model should add cluster centers to `WorldState`, each center moving slowly with its own heading while individual food items are spawned around it. Clusters should also be phase-sensitive: for example, some cluster types are more productive during the day and some at night, or cluster intensity follows the same day/night curve but with per-cluster phase offsets. The purpose is to create persistent, migrating “resource regions” that are not fully captured by one instantaneous sensor reading. The implementation should avoid expensive per-food global updates by letting foods inherit cluster motion or be periodically re-sampled near cluster centers. `src/sim/world.rs` should expose one helper that advances cluster centers and one helper that repopulates food around active clusters deterministically from seed + step count. The GUI should make clusters or their influence visible enough to debug, either by drawing faint region markers or by coloring food according to cluster membership.

The fourth system is momentum-driven movement. The current kinematics effectively make throttle choose speed directly each step. To increase temporal dependence, `src/sim/world.rs::apply_kinematics` should be replaced with a model where throttle drives acceleration, velocity persists across steps, and drag slowly damps motion. Steering should ideally influence angular velocity or turning acceleration rather than directly writing heading, but the minimal acceptable version is persistent linear momentum with configurable drag and acceleration limits. The config surface should include at least `max_acceleration`, `linear_drag`, and optionally `angular_drag` or `turn_acceleration` if steering is also made persistent. This change needs careful test coverage because it alters every episode. The acceptance case should demonstrate that an agent continues moving after throttle drops, slows down gradually under drag, and cannot reverse direction instantly.

Implementation should proceed in four milestones, each independently testable. First, add day/night phase computation and make sensor range/intensity scale with light level. The proof is a unit test in `src/sim/world.rs` showing the same agent/food geometry yields lower eye activation at night than in the day, plus a GUI indicator in `src/ui/mod.rs` showing current phase and visibility factor. Second, add `AgentRole`, typed channels, and predator/prey collision + fitness rules. The proof is a deterministic shared-world test where a predator consumes a prey and receives role-appropriate reward while prey metrics reflect the loss. Third, add moving food clusters and phase-dependent cluster productivity. The proof is a deterministic test that cluster centers move and food density around them changes across the day/night cycle, plus a GUI visual cue for cluster regions. Fourth, replace direct-speed kinematics with persistent acceleration/momentum and update controller interpretation of throttle accordingly. The proof is a unit test for inertia/drag and a short GUI smoke run where agents visibly glide rather than stop instantly.

The most likely integration risk is feature interaction between typed channels, role-aware scoring, and shared evaluation. To manage that risk, the plan should preserve switchability in config: each of the new mechanics should be individually togglable so regressions can be isolated. Day/night can be disabled. Predator/prey can be disabled by setting predator fraction or capture gain to zero. Cluster behavior can fall back to the current uniform food placement. Momentum can be disabled by choosing very high drag and legacy-equivalent acceleration. That keeps the milestone idempotent and makes recovery straightforward if one mechanic destabilizes training.

Validation should remain end-to-end. After implementation, run `cargo fmt`, `cargo check --features gui`, and `cargo test`. Then run a representative CLI experiment from the repo root, for example `cargo run --release -- run --config configs/default.toml --seed 42 --headless-generations 50 --out /tmp/ns-temporal`, and verify that the title/metrics mention day-night phase, role-aware statistics, and the resulting snapshot can be opened with `cargo run --release --features gui -- view --snapshot /tmp/ns-temporal/snapshot_gen_50.json`. In the GUI, the user should be able to see the day/night indicator change over time, select an agent and observe typed eye bins, and verify that motion has inertia.

## Planned Progress for Temporal Ecology Pressure

- [x] Add validated config for day/night phase (`day_night_cycle_enabled`, `day_night_cycle_steps`, `night_visibility_multiplier`).
- [x] Add day/night visibility scaling to sensing and surface current phase/visibility in the GUI.
- [x] Extend runtime and genome state with ecological role and any cluster state required for deterministic food-region motion.
- [x] Replace generic agent-eye sensing with typed `food`, `prey`, and `predator` channels and update the GUI inspector accordingly.
- [x] Implement predator/prey collision rules and role-aware fitness accounting in shared evaluation.
- [x] Add deterministic moving food clusters with phase-dependent productivity and a visible debug representation in the GUI.
- [ ] Replace direct-speed kinematics with persistent acceleration/momentum and add unit tests for inertia/drag behavior.
- [x] Re-run `cargo fmt`, `cargo check --features gui`, and `cargo test`, then record outcomes and any design adjustments in this plan.

Post-plan enhancement (v6): model-capacity knobs for `MLP` and `LTC` are now runtime-configurable through TOML (`mlp_hidden_size`, `ltc_state_size`) with validation and factory wiring.

Post-plan enhancement (v7): headless runs now export `fitness.svg` to visualize best and mean fitness trends over generations without external tooling.

Post-plan enhancement (v8): the Bevy viewer now renders an internal fitness chart for best and mean fitness, sourced from snapshot-embedded metric history and extended during GUI continuation.

Post-plan enhancement (v9): the Bevy viewer now runs real controllers in-frame and exposes a selected-agent inspector with eye-bin activations, controller outputs, and energy-cost breakdown, bringing GUI simulation behavior into alignment with headless controller-driven execution.

Post-plan enhancement (v10): food now respawns deterministically on consumption according to configurable probability, with the same parameter available in both TOML and GUI continuation controls.

Post-plan enhancement (v11): the LTC controller is now a conductance-based liquid network with gated sensory/recurrent synapses, leak conductance, membrane capacitance, fixed internal ODE unfolds, and backward-compatible upgrade handling for older serialized genomes.

Post-plan enhancement (v12): generation scoring now runs the full population in one shared controller-driven world per evaluation seed, introducing indirect competition through shared food depletion while preserving per-agent fitness reporting and species-isolated reproduction.

Post-plan enhancement (v13): eyes now sense nearby agents directly in addition to food, with the GUI inspector showing separate food/agent activations.

Post-plan enhancement (v14): controller input now uses separate fixed-width food and agent channel stacks, and GA evaluation mode is configurable between `solo` and `shared` from both TOML and the GUI continuation controls.

Post-plan enhancement (v15): controller input now uses only local core scalars (`energy`, `speed`) plus food/agent eye channels (`22` inputs total), while food motion and deterministic sensor noise/dropout are configurable from both TOML and the GUI continuation controls.

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
2026-03-06: Plan updated after replacing single-agent evaluation with shared-world population evaluation to document the new competition semantics, synchronous action-collection design, and passing shared-food-competition test.
2026-03-06: Plan updated after adding agent sensing to the eye model to document the signed fixed-width controller input, separate GUI food/agent observability channels, and passing sensor-model tests.
2026-03-07: Plan updated after introducing separate food/agent controller channels and switchable `evaluation_mode` to document the new fixed-width sensor layout, the GUI evaluation toggle, and the deliberate return to `solo` as the default regime.
2026-03-07: Plan updated after removing absolute position from controller inputs and adding configurable food motion plus deterministic sensor noise/dropout to better reward temporal controllers.
2026-03-10: Plan updated to add the next planned milestone for temporal ecology pressure: phased day/night visibility, predator/prey typed sensing and role-aware fitness, moving food clusters, and momentum-based movement. This change was made to turn the user’s “make LTC clearly better” goal into a concrete, staged implementation path.
2026-03-10: Plan updated after milestone 1 of temporal ecology pressure to document the implemented day/night config, night-visibility sensing scaling, GUI light indicator, and the edge-case discovery around visibility-boundary tests.
2026-03-10: Plan updated after milestone 2 of temporal ecology pressure to document explicit `AgentRole` support, typed `food/prey/predator` eye channels, role-preserving reproduction pools, predator capture rules, and role-aware fitness scoring.
