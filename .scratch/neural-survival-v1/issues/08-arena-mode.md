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
