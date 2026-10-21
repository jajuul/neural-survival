# 16: Continuing evolution from the viewer

**What to build:** From the viewer, train y more Generations in the background using the edited config, then show the new Generation.

**Blocked by:** 13: `run` command and continuing from a Snapshot; 15: Viewer config editor and edit saves

**Spec:** User Stories 94, 97–102; Implementation Decisions › GUI continuation, Artifact writing, Determinism; Acceptance Scenarios › Snapshot inspection and continuation (The user continues evolution from the GUI) (`../spec.md`)

**Status:** ready-for-agent

- [ ] The user enters y and starts continuation; Evolution runs on a background thread over copies of the Genomes and edited config
- [ ] The Showcase keeps animating while training runs
- [ ] Progress shows every 10 Generations and the Fitness chart updates as training proceeds
- [ ] When training finishes, the viewer switches automatically to a Showcase of the new Generation
- [ ] Output goes to the loaded Snapshot's directory: a Snapshot every 10 Generations and at the end, plus rewritten `metrics.csv`, `fitness.svg`, and `run_summary.json` with the full history
- [ ] Results equal a headless continuation from the same Snapshot and config (ADR-0001)

## Comments

**From the 02 (Evolution library) implementation and review:**

- Progress timing and the `advance(0)` edge case are described in 13's Comments. The same rules apply here for a y of 0 and for the per-10-Generation Snapshots.

**From the 03 (`batch` and run files) implementation and review:**

- **`record_evolution` covers the output requirement** (`crates/core/src/run_files.rs`). Call it on the background thread with the loaded Snapshot's directory. It doesn't require the directory to be empty, and it rewrites the run files with the full history. A failed write stops further writes but not the evolution, and the first error comes back at the end, so surface it in the viewer. The progress callback receives `Progress { evolution, latest, .. }`, which can feed the chart.

**From the 04 (Showcase viewer) implementation and review:**

- `crates/viewer/src/app.rs::App` owns a single `model.rs::Viewer`; model state has no background worker or progress channel. Keep training off the UI thread, pass owned copies to it, and define how progress/results return to the model and request repaints. `App::ui` currently requests continuous repaint only while the Showcase plays and has not ended; a paused/ended Showcase must still receive training updates.
- Replacing the Generation must replace the retained Snapshot as well as the Showcase: `Viewer::restart`, `stats` and `fitness_series` all depend on Snapshot state. Preserve full metrics history so restart and chart remain consistent after continuation.
