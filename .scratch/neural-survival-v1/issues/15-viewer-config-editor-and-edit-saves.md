# 15: Viewer config editor and edit saves

**What to build:** A config editor in the viewer covering the full config, and saving the current Generation with the edited config.

**Blocked by:** 14: Inspector

**Spec:** User Stories 94–96, 103; Implementation Decisions › The library's public interface (Config), Viewer saves (`../spec.md`)

**Status:** ready-for-agent

- [ ] Editor covers world, food, Obstacles, Food Clusters, GA, Fitness, sensors, Predator/Prey settings, Arena Mode, Evaluation Schedule, and Energy Rules
- [ ] Initialisation-only fields (network shape, Eye Bin count, Generation-0 ranges) are clearly marked as not rewriting evolved Genomes
- [ ] Rule toggles made in the viewer are the same setting as in the editor
- [ ] Saving writes `snapshot_gen_<n>_edit_<k>.json` next to the loaded Snapshot, using the smallest unused k ≥ 1, containing the current Genomes and the edited config
- [ ] An edit save reopens with `view` and shows the edited config

## Comments

**From the 04 (Showcase viewer) implementation and review:**

- `crates/viewer/src/model.rs::Viewer` retains the opening Snapshot, and `restart()` rebuilds from it. `stats()` reads its config and `fitness_series()` reads its metrics; `Scene::of` reads the current Showcase config. Introduce explicit ownership for edited config and decide how restart, displayed Evaluation Schedule, and saved Snapshot remain consistent. The current `Showcase::config()` returns a shared reference and is not an editing interface.
- `Viewer` currently retains no loaded file path (`src/main.rs::view` passes only the Snapshot). Preserve the source directory explicitly for edit saves and smallest-unused-index allocation.
