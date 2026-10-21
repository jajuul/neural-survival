# Design

## Context

See proposal.md for why. The core library already exposes everything the viewer needs: `Showcase::from_snapshot`, `step`, `has_ended`, `step_count`, `arenas()` (Agents with role, family, position, heading, alive; food), `Snapshot::load`, the metrics history, and `config.evaluation.plan(generation)`. It doesn't expose the body and food radii a Showcase was built with, so this change adds `Showcase::config()` (decision 4). The binary has one subcommand, `batch`. `scripts/check-architecture.sh` fails if the core crate gains a GUI dependency and requires `#![forbid(unsafe_code)]` in every crate root.

Relies on ADR-0001 (playback speed must never change what a step computes) and ADR-0003 (`eframe`/`egui` with `egui_plot`, over a headless core that never depends on the GUI).

## Goals / Non-Goals

**Goals:**
- Every behaviour in `specs/viewer/spec.md` is testable without a display.
- The `egui` code only renders model state and forwards input; it makes no decisions worth testing.
- Later tickets (Inspector, config editor, continuation) can extend the model instead of the rendering code.

**Non-Goals:**
- Automated tests of the window, painting or widgets.
- Any change to `crates/core` beyond the read-only `Showcase::config()` getter.

## Decisions

### 1. A new `crates/viewer` crate, split into a model and a GUI layer

`neural-survival-viewer` holds two parts:
- `model`, with no `egui` types: `Viewer` (Snapshot, Showcase, playback state, speed, step budget), `Stats`, `Scene`, and `FitnessSeries`. All the spec's scenarios run against this through the crate's public interface.
- `app`: the `eframe::App` that, each frame, passes the elapsed time to `Viewer::frame(dt)`, paints `Scene` with `egui::Painter`, shows `Stats` in a side panel, draws `FitnessSeries` with `egui_plot`, and maps buttons and the speed slider to `Viewer` methods.

The binary builds the `Viewer` itself, so load errors surface before any window opens, then calls `neural_survival_viewer::run(viewer)` to open the window.

*Alternatives:*
- **Model in core.** Rejected. Playback timing and draw shapes are presentation concerns, and putting them in core blurs ADR-0003's line.
- **Viewer code in the binary crate.** Rejected. Integration tests can't call into a binary's modules, and later tickets (14–16) grow the viewer substantially.
- **A separate model crate without `eframe`.** Faster test builds, but a third crate for a few hundred lines. Revisit if build times become painful.

### 2. Restart rebuilds from the Snapshot

`Viewer` keeps the Snapshot, and `restart()` calls `Showcase::from_snapshot` again. The existing snapshots spec already guarantees this gives an identical Showcase, so the core needs no `restart`. The library interface in `spec.md` lists a `restart` on Showcase. Leaving it out is deliberate and can be revisited when edits (ticket 15) make the Showcase's config differ from the Snapshot's.

*Alternative:* clone the initial `Showcase` on open. That works too, but it keeps a second full world in memory and adds nothing.

### 3. Fixed steps from a step budget

`frame(dt)` adds `min(dt, 0.25) × speed` to a fractional step budget while playing, runs `floor(budget)` steps and keeps the remainder. Pausing clears the budget. Speed is clamped to [1, 1000] steps per second, with a default of 30. Steps run only through `Showcase::step`, so ADR-0001 holds by construction, and the spec's "speed never changes what happens" scenarios check it.

*Alternative:* step once per rendered frame and change the frame rate. Rejected because speed would then depend on the display's refresh rate.

### 4. The scene is world-space shapes, with edge copies

`Scene` lists `AgentShape { position, radius, heading, colour, shape }` and `FoodDot { position, radius, colour }` in world units. The painter only scales world units to the panel. `Scene::of(&Showcase)` is public, and `Viewer::scene()` returns `Scene::of(self.showcase())`. This lets tests build exact layouts with the core's existing `Showcase::builder(...).placements(...).food(...)`, because a Snapshot carries no world layout. Edge copies are computed in the model: for each item, add offsets of ±width when it overlaps the left or right edge and ±height for the top or bottom, plus the diagonal copy when it overlaps both. Colours are model constants (`ROLE_COLOURS`, `FOOD_COLOUR`), stored as RGB triples so the model stays free of `egui`. MLP is a circle; the shape enum reserves a triangle for LTC (ticket 07). Radii come from the Showcase's own config through a new read-only `Showcase::config()`, because `Scene::of` takes only the Showcase and the radii aren't otherwise public. Found during implementation; the human approved the getter.

### 5. Stats and Fitness series are derived on demand

`Viewer::stats()` and `Viewer::fitness_series()` compute their values from the Showcase, the Snapshot and its config whenever they're asked. Per-Species counts group Agents by `Species { family, role }` in `Species::ALL` order. Per-family counts group by `ControllerFamily`. With a few hundred Agents, recomputing every frame costs nothing.

### 6. A hidden `--no-window` flag for CLI tests

`view --no-window` builds the `Viewer`, prints the ready line, and exits. It's hidden from `--help` with `#[arg(hide = true)]`. This matches the spec's plan for `run`, which has "a test-only way to skip opening the window", and lets `tests/cli.rs` cover loading and errors as a process.

### 7. Dependencies

`eframe` with its default `glow` backend, and the `egui_plot` version that matches it. Both are pinned to the newest compatible pair when the code is implemented. They are added only to `crates/viewer/Cargo.toml`. The root package adds `neural-survival-viewer`.

## Risks / Trade-offs

- [Build time and `Cargo.lock` grow considerably with `winit`/`glow`] → Accepted for a GUI app. The core crate's tests are unaffected. Decision 1 notes a model-only crate as the escape hatch.
- [Building needs X11/Wayland development libraries; opening the window under WSL needs WSLg] → `README.md` documents the packages. Automated tests never open a window.
- [The window isn't covered by automated tests] → It only renders model state. The human checks it at step 8 against the ticket's checklist.
- [Edge copies of a large `vision_range` aren't drawn] → Only bodies and food are drawn now. Vision cones are a later Inspector concern (ticket 14).
- [`cargo mutants` on `app.rs` will find survivors, because no test runs the GUI] → Exclude `crates/viewer/src/app.rs` from mutation testing for this change, or propose its survivors for `accepted-mutants`. Decide when the gate runs.

## Open Questions

- The exact Prey and food colours, and the side-panel layout. These are cosmetic and can be tuned during the step 8 check without changing any spec.
