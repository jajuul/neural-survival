# 08: Arena Mode

**What to build:** Add Arena Mode: Separated (one full-size Arena per Controller Family, all built from the same Episode seed) or Shared (all Species in one Arena). The mode is part of the config and can change at any Generation boundary in either direction, without automatic scaling.

**Blocked by:** 03: `batch` and run files; 06: Predators and Captures; 07: LTC Controller Family

**Status:** ready-for-agent

- [ ] Separated mode gives each Controller Family its own Arena containing its Prey and Predator Species
- [ ] All Arenas in one Episode have identical food and spawn layouts
- [ ] Shared mode puts all Species in one Arena of the configured size, with no automatic scaling of food or area
- [ ] Changing Arena Mode between Generations takes effect at the next Generation
- [ ] Each metrics row records the Arena Mode; `fitness.svg` and the viewer chart mark mode changes
- [ ] Viewer shows Separated Arenas side by side
- [ ] Integration tests cover identical layouts across Separated Arenas and mode switching

## Comments

**From the 02 (Evolution library) implementation and review:**

- **Agent ids must stay global across Arenas.** `Evolution::run_generation` (`crates/core/src/evolution.rs`) adds each Agent's score to `fitness[agent.id.0]`, where `fitness` is indexed in the order of `self.populations.concat()`. `Episode::genome` (`crates/core/src/episode.rs`) looks Genomes up by `AgentId` in the same way. When the Episode's Agents are split across Separated Arenas, keep ids numbered across the whole Episode, not restarting in each Arena, or remap them before scoring. Add a test where a later Arena's Agents get the right Fitness.
- **Arena random streams are not split yet.** `Episode::new` draws spawn layout and food layout from `stream_rng(seed, Stream::Arena, 0)`, then hands that same rng to the Arena for food respawns. For "identical food and spawn layouts", every Separated Arena should draw its layout from the same Episode-seed stream. Decide whether respawn draws are also shared (the ecologies stay controlled until they diverge) or split per Arena (`Stream::Arena, arena_index`).
- **Spawn layouts can differ in size.** Spawn placements are drawn per Genome, but each Controller Family's Arena may hold a different number of Agents, because population is configured per Species. Decide what "identical spawn layouts" means when the counts differ, for example drawing positions in the same order so the shorter list is a prefix of the longer one.
- **Adding `arena_mode` to `MetricsRow` changes the Snapshot schema.** `MetricsRow` is serialised into Snapshots and Snapshot load uses `deny_unknown_fields`. Either bump `SNAPSHOT_FORMAT_VERSION` (`crates/core/src/snapshot.rs`) or give the field a serde default, and say which in the commit.
