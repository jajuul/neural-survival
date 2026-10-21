# Walkthrough: 04-viewer-plays-a-showcase

## Review summary

Full tier, already declared and confirmed. Change range: `2a2a8fe..04d8cad`; checked source: `b180cc9`. At walkthrough start, HEAD was `04d8cad`. The gate and step 5 both name `b180cc9`; verification names the identical gate range. `git diff --name-only b180cc9 HEAD -- . ':!openspec/changes/04-viewer-plays-a-showcase/review/'` is empty: subsequent commits changed only review records. The unrelated uncommitted `AGENTS.md` edit is excluded.

- [Gate](gate.md), `2a2a8fe..b180cc9`: **PASS** formatting, Clippy, workspace tests, architecture, OpenSpec validation, traceability, acceptance-test lock, tier and mutation testing. **SKIPPED** dependency audit (`no deny.toml`). **FAIL** none. Mutation PASS has underlying exit 2: all **20 surviving signatures are accepted**, comprising 19 GUI signatures and one mixed-Role stats predicate. This is an exception-policy pass, not complete mutation detection; verification maps every signature.
- [Code review](code-review.md): Standards **0**, Spec **0**, Bugs **0 actionable findings**. The previous heading-marker edge-copy finding is resolved. The report qualifies the built-in review's incomplete-mutation statement with final mutation evidence.
- [Verification](verification.md): **12 Verified, 0 Partially verified, 0 Failed, 0 Unverified** requirements; all 27 scenarios have approved acceptance evidence. These statuses cover headless behavior.

**Largest open risk:** no recorded manual GUI run establishes that the window, repaint loop, controls and painting work together. The accepted `run → Ok(())` mutant illustrates that automated tests can pass without opening any window.

**Verdict: ready for the human's targeted reading and step 8 manual checks; approval remains pending.**

## What changed

`view --snapshot <path>` now opens a saved Generation as a paused Showcase, with playback, restart, stats and Fitness history. A Snapshot supplies Genomes and seed, not saved world state: opening constructs a fresh, deterministic Episode. This path does not evolve or score the population (`src/main.rs::view`, `crates/core/src/showcase.rs::Showcase::from_snapshot`).

## How it is structured

- **CLI:** `src/main.rs::view` loads the Snapshot, constructs `Viewer`, prints readiness and transfers ownership to `run`. Hidden `--no-window` returns before the window call.
- **Model:** `crates/viewer/src/model.rs::Viewer` owns the retained Snapshot, current Showcase, pause/speed state and playback clock. Callers get shared references to the Snapshot and Showcase. `stats`, `scene` and `fitness_series` derive presentation data on demand; the GUI receives owned values, not mutable simulation access.
- **Window:** `crates/viewer/src/app.rs::App` owns `Viewer`. `App::ui` supplies frame time; `controls` forwards input and displays stats; `world` scales/clips the world-space scene; `chart` draws best/mean series.
- **Core:** `crates/core/src/showcase.rs::Showcase` still owns a private Episode. The new `config()` delegates to `crates/core/src/episode.rs::Episode::config`, exposing a shared config reference for radii and dimensions.

New dependency edges: root binary → viewer; viewer → core, `eframe 0.37`, `egui_plot 0.38`; `app` → `model`. The model uses core types and no GUI types. Core has no reverse dependency; recorded Architecture PASS corroborates this (`crates/viewer/Cargo.toml`, root `Cargo.toml`, `scripts/check-architecture.sh`). Model tests avoid window creation, although building the viewer still compiles its GUI dependencies.

The committed scope also includes review infrastructure: `scripts/mutants-ticket.sh` lets all workspace tests detect each mutant; `scripts/review-gate.sh` runs mutations only with `--mutants`; the review skills allow later review-only commits. These changes are covered by step 5's report.

## Runtime paths

**Open, play, restart:**

1. `src/main.rs::view` calls `Snapshot::load`, then `model.rs::Viewer::open`. `showcase.rs::Showcase::from_snapshot` derives the Showcase seed from run seed and Generation, uses saved Genomes and selects that Generation's Episode limit.
2. `app.rs::run` moves the Viewer into `App`. It opens paused at step 0. Continue calls `Viewer::play`; a subsequent `App::ui` supplies `stable_dt` to `Viewer::frame`.
3. `Viewer::frame` caps elapsed time at 0.25 s, multiplies by speed and advances whole steps due through `Showcase::step_many`. `episode.rs::Episode::step` ignores steps after the limit or all Agents die. Frame time never enters simulation-step computation.
4. `App::world` consumes `Viewer::scene`; `Scene::of` filters dead Agents and adds torus-edge copies, including heading-marker reach. `App::chart` consumes Snapshot-history points from `Viewer::fitness_series`.
5. Restart calls `Viewer::restart`, reconstructs from the retained Snapshot and clears playback time through `pause`. Approved tests `restart_after_the_end` and `speed_never_changes_what_happens` provide replay and scheduling evidence (see verification).

**Missing or unsupported Snapshot:** `snapshot.rs::Snapshot::load` returns a path-bearing I/O error or `UnsupportedVersion` before Viewer/window construction. `src/main.rs::view` propagates it; `main` prints `error: …` to stderr and returns failure. The CLI acceptance tests cover both. A later `eframe::run_native` error also propagates through `app.rs::run`, but platform window failures lack executed evidence.

## Design and ADR conformance

ADR-0001's fixed-step boundary and ADR-0003's dependency direction are preserved by the paths above. No ADR deviation found.

One representation differs from design decision 3: `Viewer::frame` retains cumulative steps-worth of time plus `steps_run`, rather than a bounded fractional remainder. Pause resets both; speed changes preserve carried time. Step 5 found playback compliant; verification records missing long-duration/non-finite-time evidence. Restart-from-Snapshot, world-space scene data, the reserved triangle and on-demand stats/chart derivation match the design.

Deployment prose differs: design anticipates development-library prerequisites; README describes runtime-loaded libraries and runtime packages. Verification establishes neither a clean Linux/WSL installation nor those platform claims.

## Risks and approval checks

The GUI boundary owns consequential timing, repaint, widget and coordinate conversions (`App::{ui,controls,world}`); headless equality tests cannot establish their visible behavior. Run normal `view --snapshot` and check opening paused, every control, speed, Episode end/restart, stats, best/mean lines, heading direction and edge/corner painting against the ticket's checklist. No GUI run or gate rerun was performed for this walkthrough.

Other limits are detailed in [verification](verification.md): no dependency audit for the larger GUI tree; single-Species fixtures cannot establish mixed-Role grouping (`Viewer::stats`, accepted mutant deferred to ticket 06); cumulative-clock extremes remain untested (`Viewer::frame`). There are no outstanding step 5 findings to repeat.

## Read in full

**None.** The diff touches no active pattern in `docs/agents/always-read.txt`: `crates/core/src/rng.rs`, `crates/core/src/snapshot.rs` and `docs/adr/*` are unchanged. Commented script suggestions are not active entries.

## Inspect these

1. **`crates/viewer/src/app.rs:9–179` — `run`, `App::{ui,controls,chart,world}`.** Read the untested GUI boundary alongside the manual run: window ownership, repaint/dispatch order, button enabling, chart labels, scaling, y inversion and clipping. This carries the largest evidence gap.
2. **`crates/viewer/src/model.rs:19–110` — `Viewer` playback and restart.** Check cumulative-clock accounting, pause resets, speed bounds and reconstruction from the immutable retained Snapshot. These determine scheduling and replay without changing simulation calculations.
3. **`crates/viewer/src/model.rs:246–355` — `HEADING_MARKER_LENGTH`, `Scene::of`, `edge_copies`.** Check shared marker reach and Cartesian edge/corner offsets. This is where the former wrap finding was fixed; `crates/viewer/tests/scene.rs::a_heading_marker_crossing_an_edge_is_copied` supplies the regression evidence.

Follow-up walkthroughs can trace a specific scenario, explain a boundary choice, or locate the test supporting a behavior; missing GUI evidence remains explicit.
