# 06: Predators and Captures

**What to build:** Add the Predator Role and the MLP-Predator Species. Predators capture Prey at close range and never eat food. Both Roles sense each other through new Prey and Predator Eye Bins.

**Blocked by:** 03: `batch` and run files; 04: Viewer: `view` plays a Showcase

**Status:** ready-for-agent

- [ ] MLP-Predator Species with its own configured population size, bred only within itself; Role never mutates (ADR-0002)
- [ ] Controller input adds N Prey and N Predator Eye Bins after the food bins
- [ ] A Capture occurs when a Predator is within capture range of a Prey: the Prey dies, the Predator gains a fixed configured energy amount, then cannot capture again until a configured cooldown passes
- [ ] Predators never eat food
- [ ] Starting energy and Fitness weights are configured per Role; Predator energy gained counts Captures
- [ ] Prey keep going after every Predator has died
- [ ] `metrics.csv` and the metrics history record Captures for Predator Species
- [ ] Viewer draws Predators in a distinct colour
- [ ] Integration tests cover Capture, cooldown, no food for Predators, and the new Eye Bins
