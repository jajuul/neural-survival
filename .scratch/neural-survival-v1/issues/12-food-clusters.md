# 12: Food Clusters

**What to build:** Add optional Food Clusters: drifting regions around which food spawns, with density following the Light Phase.

**Blocked by:** 10: Day/night and sensor noise; 11: Food motion and replenishment

**Spec:** User Stories 53–54, 83; Implementation Decisions › Food, Day/night; Testing Decisions › Seam 1 (`../spec.md`)

**Status:** ready-for-agent

- [ ] When enabled, food spawns around drifting Food Clusters
- [ ] Cluster density changes with the Light Phase
- [ ] Clusters combine with food motion and replenishment settings
- [ ] Viewer draws cluster overlays when enabled
- [ ] Integration tests cover spawning near clusters and density varying with Light Phase
