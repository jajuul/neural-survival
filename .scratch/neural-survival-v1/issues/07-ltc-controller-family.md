# 07: LTC Controller Family

**What to build:** Add the conductance-based LTC Controller Family and the LTC-Prey and LTC-Predator Species. LTC networks are integrated with the semi-implicit (fused) solver over a configurable number of sub-steps per sim step.

**Blocked by:** 04: Viewer: `view` plays a Showcase

**Status:** ready-for-agent

- [ ] LTC networks have a configured neuron count, published-default initial parameters, and map the shared input vector to steering and throttle in [-1, 1]
- [ ] Each sim step runs the configured number of solver sub-steps (default about 6); the count is recorded in the resolved config
- [ ] Neuron state persists across steps within an Episode and resets at the start of each Episode
- [ ] LTC Species are bred only within themselves; Controller Family never mutates (ADR-0002)
- [ ] All four Species can coexist in one run
- [ ] Viewer draws LTC Agents with a distinct shape
- [ ] Integration tests cover state reset between Episodes, family isolation in breeding, and determinism with LTC present
