# Tasks

## 1. Acceptance tests (Codex, fresh session; the human approves)

- [x] 1.1 Re-run the step 1 checklist on `specs/viewer/spec.md` and send any scenario that can't become a test back to step 1. Verify: the checklist report lists no unresolved failures.
- [x] 1.2 From the delta spec only, write `tests/acceptance_04_viewer_plays_a_showcase.rs`: CLI scenarios run the binary as a process, and model scenarios use the `neural_survival_viewer` public interface, which may not exist yet. Write "speed never changes what happens" as a `proptest` property over speeds and frame lengths, adding `proptest` as a dev-dependency. Link each scenario with `<!-- verified-by: ... -->`. Verify: the file compiles once the crate exists, and every scenario has a link.
- [x] 1.3 Commit the tests, then commit `openspec/changes/04-viewer-plays-a-showcase/acceptance-approved` naming that commit and file once the human approves. Verify: `git log` shows both commits.

## 2. Viewer crate scaffold

- [x] 2.1 Create `crates/viewer` (`neural-survival-viewer`) with `#![forbid(unsafe_code)]`, `model` and `app` modules, and `eframe` plus the matching `egui_plot` dependency. Add it to the workspace members and as a dependency of the root package. Verify: `cargo build --workspace --locked` succeeds and `scripts/check-architecture.sh` passes, so core still has no GUI dependency.

## 3. Playback model

- [x] 3.1 Implement `Viewer::open(Snapshot)`, which starts paused at step 0 on `Showcase::from_snapshot`, and the getters for paused, step, ended, speed and Showcase. Verify: the "opens paused" acceptance test passes.
- [x] 3.2 Implement `frame(dt)` with the step budget (0.25 s cap, remainder carried, cleared on pause), `play`, `pause`, `single_step`, and `set_speed` clamped to [1, 1000] with a default of 30. Verify: the pause and continue, single-step, speed, and "speed never changes what happens" acceptance tests pass.
- [x] 3.3 Stop at Episode end and implement `restart()` by rebuilding from the Snapshot. Verify: the Episode-end and restart acceptance tests pass.

## 4. Stats, scene and chart data

- [x] 4.1 Implement `stats()`: Generation, step, alive, food, per-Species and per-family alive and total, and the Evaluation Schedule plan for the Generation. Verify: the stats acceptance tests pass.
- [x] 4.2 Implement the public `Scene::of(&Showcase)` and `Viewer::scene()`, which delegates to it: living Agents as family shape and role colour with heading, food dots, and edge copies for left, right, top, bottom and corners. Verify: the scene acceptance tests pass.
- [x] 4.3 Implement `fitness_series()`: best and mean points per Species from the metrics history. Verify: the chart acceptance tests pass.

## 5. GUI and CLI

- [x] 5.1 Implement `app`: an `eframe::App` that calls `frame(dt)`, paints the scene scaled to the panel, and shows a side panel with stats, pause/continue/step/restart buttons and a speed slider, plus an `egui_plot` chart of `fitness_series()`. Add `run(viewer)` to open the window (design decision 1). Verify: `cargo clippy --workspace --all-targets -- -D warnings` is clean, and `cargo run -- view --snapshot <file>` opens the window paused on that Generation (checked by hand).
- [x] 5.2 Add `view --snapshot <path>` and the hidden `--no-window` flag to `src/main.rs`. A load failure prints `error: …` and exits non-zero before any window opens. Verify: the CLI acceptance tests pass and `neural-survival --help` doesn't show `--no-window`.
- [x] 5.3 Document `view` and the Linux/WSL GUI build prerequisites in `README.md`. Verify: the documented commands run as written.

## 6. Gate

- [x] 6.1 Commit, run `scripts/mutants-ticket.sh 2a2a8fe`, and add tests for any missed mutants in the model. For survivors in `app.rs`, which no test exercises, propose `accepted-mutants` entries to the human. Verify: every survivor is caught or proposed.
- [x] 6.2 Run `scripts/review-gate.sh 2a2a8fe 04-viewer-plays-a-showcase` on the committed change and report its recorded results. Verify: `review/gate.md` shows every check passing.

## Workflow follow-up

- Steps 5–7 (review, verify, walkthrough) run in fresh Codex sessions.
- After the human approves at step 8: `/opsx:archive`, set the ticket to `Status: resolved`, and append findings for later tickets (13, 14, 15, 16) to their `## Comments`.
