# 04: Viewer: `view` plays a Showcase

**What to build:** The `view --snapshot <path>` command opens an `eframe`/`egui` window (ADR-0003) showing a Showcase of the Snapshot's Generation. The GUI is a thin layer over the library's Showcase interface. Simulation speed changes how often steps run, never what a step computes (ADR-0001).

**Blocked by:** 02: Evolution library

**Spec:** User Stories 6, 69–78, 81–82; Implementation Decisions › Workspace shape, The library's public interface (Showcase), Determinism; Testing Decisions › The GUI is not tested automatically; Acceptance Scenarios › Snapshot inspection and continuation (A saved snapshot can be opened for inspection) (`../spec.md`)

**Change:** 04-viewer-plays-a-showcase

**Status:** ready-for-agent

- [ ] `neural-survival view --snapshot <path>` opens the viewer paused on a Showcase of that Generation
- [ ] World view draws Agents (Prey colour, MLP shape) and food, with wrap-around handled
- [ ] Pause, continue, single-step while paused, and simulation speed control work
- [ ] The Showcase stops when its Episode ends; restart replays it identically
- [ ] Stats panel shows Generation, world step, alive count, food remaining, per-Species and per-family counts, and the active Evaluation Schedule
- [ ] A best/mean Fitness chart (`egui_plot`) shows one line pair per Species from the Snapshot's history
