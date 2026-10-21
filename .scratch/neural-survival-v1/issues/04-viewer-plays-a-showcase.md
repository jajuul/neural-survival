# 04: Viewer: `view` plays a Showcase

**What to build:** The `view --snapshot <path>` command opens an `eframe`/`egui` window (ADR-0003) showing a Showcase of the Snapshot's Generation. The GUI is a thin layer over the library's Showcase interface. Simulation speed changes how often steps run, never what a step computes (ADR-0001).

**Blocked by:** 02: Evolution library

**Spec:** User Stories 6, 69–78, 81–82; Implementation Decisions › Workspace shape, The library's public interface (Showcase), Determinism; Testing Decisions › The GUI is not tested automatically; Acceptance Scenarios › Snapshot inspection and continuation (A saved snapshot can be opened for inspection) (`../spec.md`)

**Change:** 04-viewer-plays-a-showcase

**Status:** resolved

- [x] `neural-survival view --snapshot <path>` opens the viewer paused on a Showcase of that Generation
- [x] World view draws Agents (Prey colour, MLP shape) and food, with wrap-around handled
- [x] Pause, continue, single-step while paused, and simulation speed control work
- [x] The Showcase stops when its Episode ends; restart replays it identically
- [x] Stats panel shows Generation, world step, alive count, food remaining, per-Species and per-family counts, and the active Evaluation Schedule
- [x] A best/mean Fitness chart (`egui_plot`) shows one line pair per Species from the Snapshot's history

## Comments

**2026-10-21 — Step 8 approval and archive:** The human reported “Looks good” after the GUI run instructions and requested archive, then confirmed spec sync. Archived at `openspec/changes/archive/2026-10-21-04-viewer-plays-a-showcase/`; current behavior is in `openspec/specs/viewer/spec.md`. This records human approval, not an agent-observed GUI run. Review evidence and its dependency-audit, GUI automation and future multi-Species limitations remain in the archived review reports.
