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
