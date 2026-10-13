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

## Comments

**From the 02 (Evolution library) implementation and review:**

- **Averaging over Episodes isn't fully tested.** `Evolution::run_generation` (`crates/core/src/evolution.rs`) divides each Genome's summed Fitness by the Episode count. But `fitness_is_weighted_steps_alive_plus_energy_gained` (`crates/core/tests/evolution.rs`) runs Episodes that all score the same, so the test catches a sum but not "last Episode" or "best Episode". This ticket changes Fitness (per-Role weights, energy gained from Captures), so add a test whose Episodes score differently and check that the reported Fitness is their mean. That probably needs a seam that isn't public yet, for example per-Genome or per-Episode Fitness exposed by Evolution, or a way to fix each Episode's layout. Agree the seam before adding it.
- **Mutation testing found two lines that can't be tested yet.** `cargo mutants` on the issue 02 diff found two mutations no test can catch while MLP-Prey is the only Species: the per-Species `offset` slicing of `fitness` in `Evolution::run_generation`, and `species_index` (both in `crates/core/src/evolution.rs`). Once MLP-Predator exists, make sure a test fails if Predator Fitness or metrics land on the wrong Species. Re-run `cargo mutants --workspace --in-diff <diff>` to confirm.
