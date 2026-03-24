# Finish the Conductance-Based LTC and Make It the Canonical Controller

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agent/PLANS.md` from the repository root and must be maintained in accordance with that file.

## Purpose / Big Picture

After this change, the project's `Ltc` controller will be a technically honest conductance-based Liquid Time-Constant controller throughout the product, not just in the internal cell update. A user will be able to run the same CLI and GUI workflows as today, but the LTC branch will have a locked equation, validated numerical behavior, backward-compatible genome loading, and diagnostics that expose real LTC state rather than a generic recurrent approximation.

The change is successful when three things are true: the LTC state update matches the chosen conductance-based equation in code and tests, old saved snapshots and genomes still load into the new layout without corruption, and end-to-end runs remain deterministic with the LTC family active.

## Progress

- [x] (2026-03-24 12:41Z) Read `.agent/PLANS.md`, `docs/SPECS.md`, `docs/TECH_SPEC.md`, `src/brain/ltc.rs`, `src/genetics/operators.rs`, and `src/io/snapshot.rs`.
- [x] (2026-03-24 12:41Z) Verified the current branch already contains a conductance-based LTC structure, legacy serde aliases/defaults, diagnostics hooks, and passing LTC-focused unit tests via `cargo test brain::ltc -- --nocapture`.
- [x] (2026-03-24 12:41Z) Authored this ExecPlan for finishing and hardening the LTC implementation as the canonical controller.
- [x] (2026-03-24 12:49Z) Documented the canonical conductance-based LTC membrane update in `src/brain/ltc.rs`, `docs/SPECS.md`, and `docs/TECH_SPEC.md`.
- [x] (2026-03-24 12:49Z) Refactored LTC stepping into explicit capacitance-scaling, recurrent-drive, and single-unfold helpers and added equilibrium, reversal-sign, and split-timestep tests; verified with `cargo test brain::ltc`, `cargo test genetics::operators`, and full `cargo test`.
- [ ] Audit genetics, serialization, diagnostics, and GUI inspection so every LTC-facing surface reflects the real parameterization.
- [ ] Run full regression tests and one artifact-producing headless run with LTC-enabled populations.

## Surprises & Discoveries

- Observation: the repository is already partway through the "make it real LTC" work.
  Evidence: `src/brain/ltc.rs` now defines conductance, gate mean/slope, reversal potential, leak, and capacitance terms instead of the earlier `tau`-only leaky RNN layout.

- Observation: backward compatibility for older serialized LTC genomes is already anticipated.
  Evidence: `serde(alias = "...")`, `#[serde(default)]`, and `ensure_layout()` in `src/brain/ltc.rs` normalize missing/new fields.

- Observation: current LTC tests prove determinism and serde compatibility, but they do not yet prove equation fidelity across numerical refinements.
  Evidence: existing tests cover known-sequence stepping, clamp ranges, factory wiring, diagnostics exposure, and legacy deserialization, but not timestep refinement or equilibrium behavior.

- Observation: the existing conductance update was already mathematically close to the target equation; the immediate value came from decomposing it into named helpers and proving its behavior with finer-grained tests.
  Evidence: after the helper refactor, the original golden tests still passed unchanged and the new equilibrium/reversal/refinement tests passed without changing external LTC interfaces.

## Decision Log

- Decision: keep the public `BrainGenome::Ltc` variant and current snapshot file shape.
  Rationale: the product already exposes LTC as a controller family; the goal is to make that family technically correct without breaking the broader CLI/GUI/data model.
  Date/Author: 2026-03-24 / Codex

- Decision: preserve backward compatibility for pre-conductance LTC snapshots and genomes.
  Rationale: existing artifacts should remain viewable and evolvable; the migration should happen at deserialize/validate time rather than through a one-off conversion script.
  Date/Author: 2026-03-24 / Codex

- Decision: keep the internal ODE unfold count fixed in code for v1 unless validation proves it must become configurable.
  Rationale: a fixed unfold count is already part of the technical spec and is simpler for deterministic testing; the immediate gap is correctness and validation, not tunability.
  Date/Author: 2026-03-24 / Codex

- Decision: treat "real LTC" in this repository as "conductance-based state update with sensory and recurrent gates, leak conductance, reversal potentials, and membrane capacitance," not as a vague recurrent family label.
  Rationale: this removes ambiguity and gives tests a concrete target.
  Date/Author: 2026-03-24 / Codex

## Outcomes & Retrospective

The first implementation slice is complete. The LTC update rule is now explicit in code and specs, and the test suite now covers equilibrium, reversal-potential sign, and split-timestep consistency in addition to the earlier determinism and serde checks. The remaining work is concentrated in semantic mutation, richer diagnostics, and artifact-level validation rather than core controller structure.

## Context and Orientation

The core LTC code lives in `src/brain/ltc.rs`. That file currently defines a conductance-based genome, state update, and controller factory. `src/brain/mod.rs` defines the shared controller traits and diagnostics snapshots. `src/genetics/operators.rs` performs flattened crossover and mutation over `LtcGenome`. `src/io/snapshot.rs` persists `AgentGenome` through snapshots, so any LTC genome change has artifact compatibility impact. `docs/SPECS.md` and `docs/TECH_SPEC.md` already claim the controller is conductance-based, so the implementation must now fully earn that claim.

In this plan, "conductance-based" means each connection contributes a nonnegative conductance multiplied by a gate value, and that conductance pulls membrane voltage toward a reversal potential. "Leak" means the passive pull toward a baseline voltage. "Membrane capacitance" means the inertia term that slows state change. "ODE unfold" means subdividing one simulation step into several smaller internal integration updates.

## Plan of Work

Milestone 1 establishes the canonical LTC equation and makes the implementation read like that equation. In `src/brain/ltc.rs`, split the current stepping logic into named helpers for sensory drive, recurrent drive, one unfold update, and readout. Add comments immediately above the update rule that spell out, in plain language, how numerator and denominator correspond to leak plus synaptic conductances. Update `docs/TECH_SPEC.md` and the LTC section in `docs/SPECS.md` so the written contract matches the exact implemented parameterization: sensory and recurrent weights are conductances, `mu` and `sigma` define gate position and sharpness, `erev` defines target voltage, `gleak` and `vleak` define passive leak, and `cm` sets state inertia.

Milestone 2 hardens numerical validation. Add unit tests in `src/brain/ltc.rs` for steady-state leak behavior, sign-sensitive reversal potential behavior, and timestep refinement. The timestep refinement test should compare one `dt` step against two `dt / 2` steps with the same starting state and assert they remain close within a fixed tolerance. Add a zero-input equilibrium test that proves the controller converges toward the leak-dominated equilibrium rather than drifting arbitrarily. Keep deterministic golden tests, but make them describe conductance behavior rather than generic recurrent stepping.

Milestone 3 audits genome lifecycle and evolution semantics. In `src/brain/ltc.rs`, keep `ensure_layout()` as the single place that upgrades legacy genomes, and add tests proving old-field aliases deserialize into valid conductance-based genomes with nonempty defaulted vectors. In `src/genetics/operators.rs`, stop treating all flattened LTC parameters as equivalent mutation targets; keep the flattened representation for crossover, but mutate with field-aware step sizes and then clamp by semantic range. Conductances should remain nonnegative, reversal potentials should stay in the bounded voltage range, and gate parameters should remain inside chosen operating bounds. This preserves valid LTC semantics during evolution instead of relying on broad post-hoc clipping alone.

Milestone 4 upgrades diagnostics and user-facing inspection. In `src/brain/mod.rs`, extend `LtcNetworkSnapshot` only as far as needed to expose real LTC internals: current state values, readout outputs, and the current sensory/recurrent conductance or gate activations for the inspected input. In `src/brain/ltc.rs`, make `inspect_network()` and `step_with_diagnostics()` populate those fields from the real conductance calculation, not just static weights. This gives the GUI a truthful picture of what the LTC is doing.

Milestone 5 runs regression and acceptance validation across the product. Re-run the full test suite, verify snapshot round-trip behavior, and run a short batch job from the CLI with LTC enabled. Acceptance requires deterministic outputs, successful load of legacy-compatible snapshots, and no regressions in mixed-family evolution or viewer loading.

## Concrete Steps

Run all commands from `/home/juul/rust/neural-survival`.

1. Validate the current LTC-focused baseline before edits.

    cargo test brain::ltc -- --nocapture

   Expected outcome: all LTC unit tests pass.

2. After Milestones 1 and 2, run the brain and genetics regression set.

    cargo test brain::ltc
    cargo test genetics::operators

   Expected outcome: new conductance-specific tests pass, including timestep-refinement and legacy-layout tests.

3. After Milestone 3, verify artifact compatibility and end-to-end determinism.

    cargo test
    cargo run -- batch --config configs/default.toml --seed 42 --generations 3 --out /tmp/ns-ltc-real

   Expected outcome: the full suite passes, and `/tmp/ns-ltc-real` contains `metrics.csv`, `fitness.svg`, `run_summary.json`, and `snapshot_gen_0003.json`.

4. Verify snapshot loading still works for the produced artifact.

    cargo run -- view --snapshot /tmp/ns-ltc-real/snapshot_gen_0003.json

   Expected outcome: the viewer or headless viewer stub loads without LTC deserialization errors.

## Validation and Acceptance

Acceptance is complete only when all checks below pass:

1. `src/brain/ltc.rs` documents the exact conductance-based update rule in code comments and implements it through explicit helper functions rather than one opaque block.
2. Unit tests prove:
   - deterministic known-sequence stepping
   - legacy serialized genome compatibility
   - clamp-range enforcement
   - leak-dominated equilibrium behavior
   - reversal-potential sign behavior
   - timestep refinement consistency for split `dt`
   - diagnostics expose real LTC state and conductance/gate data
3. Genetics tests confirm LTC mutation and crossover preserve valid semantic ranges and valid genome lengths.
4. Full `cargo test` passes.
5. A real batch run writes artifacts and a produced snapshot reloads successfully.

## Idempotence and Recovery

This work is safe to do incrementally. The plan is additive and test-driven: refactor helpers first, add tests second, then tighten mutation/diagnostics. If a refactor breaks determinism, stop, restore the previous helper behavior, and re-run only `cargo test brain::ltc` before expanding outward. Because snapshot compatibility is part of the goal, never remove the current serde aliases/defaulted fields until replacement compatibility tests exist and pass.

## Artifacts and Notes

The canonical LTC update represented in code should be recognizable as:

    state_next = (cm_scaled * state_prev
                  + gleak * vleak
                  + sensory_sum(conductance * erev)
                  + recurrent_sum(conductance * erev))
                 /
                 (cm_scaled
                  + gleak
                  + sensory_sum(conductance)
                  + recurrent_sum(conductance))

where each conductance is a nonnegative weight multiplied by a sigmoid gate.

The important proof artifact after implementation is not just passing compilation; it is a test set that shows the controller behaves like this equation under deterministic stepping and under legacy snapshot loading.

## Interfaces and Dependencies

In `src/brain/ltc.rs`, the end state must keep:

    pub struct LtcShape { pub input_size: usize, pub state_size: usize, pub output_size: usize }

    pub struct LtcGenome {
        pub shape: LtcShape,
        pub sensory_weight: Vec<f32>,
        pub sensory_mu: Vec<f32>,
        pub sensory_sigma: Vec<f32>,
        pub sensory_erev: Vec<f32>,
        pub recurrent_weight: Vec<f32>,
        pub recurrent_mu: Vec<f32>,
        pub recurrent_sigma: Vec<f32>,
        pub recurrent_erev: Vec<f32>,
        pub vleak_state: Vec<f32>,
        pub cm_state: Vec<f32>,
        pub gleak_state: Vec<f32>,
        pub weights_state_output: Vec<f32>,
        pub bias_output: Vec<f32>,
    }

`LtcGenome::validate()` must remain the canonical compatibility-normalization point. `LtcController::forward_outputs()` must delegate to smaller helpers that directly correspond to the conductance equation. `src/genetics/operators.rs` must continue to support flattened LTC crossover, but mutation should become field-aware rather than uniformly perturbing every scalar with the same range.

In `src/brain/mod.rs`, keep `ControllerKind::Ltc` and `BrainGenome::Ltc(LtcGenome)` unchanged so existing snapshots and mixed-family flows remain valid.

No new external crate is required for this work unless a deterministic numeric helper becomes necessary. Prefer staying within the current dependency set.

## Revision Note

2026-03-24: Created this ExecPlan after reviewing the current conductance-based LTC branch. The plan is intentionally framed as a completion and hardening effort, not a full rewrite, because the repository already contains the core conductance-style genome and state-update path.
2026-03-24: Updated the ExecPlan after the first implementation slice to record the completed equation-documentation refactor and the new equilibrium, reversal-sign, and split-timestep verification coverage.
