# 06: Predators and Captures

**What to build:** Add the Predator Role and the MLP-Predator Species. Predators capture Prey at close range and never eat food. Both Roles sense each other through new Prey and Predator Eye Bins.

**Blocked by:** 03: `batch` and run files; 04: Viewer: `view` plays a Showcase

**Spec:** User Stories 13, 25–26, 40, 44, 48–50, 60, 82, 106 (Captures); Implementation Decisions › Species, Input vector layout, Fitness, Capture; Testing Decisions › Seam 1 (World rules: Capture, Predators don't eat food) (`../spec.md`)

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

**From the 03 (`batch` and run files) implementation and review:**

- **`metrics.csv` has no Captures column yet.** `metrics_csv` in `crates/core/src/run_files.rs` writes the `MetricsRow` fields. When Captures go into `MetricsRow`, add the column there and extend `metrics_csv_has_one_row_per_generation_per_species` (`crates/core/tests/run_files.rs`). Adding the field also changes the Snapshot schema; see 08's note on `arena_mode`.
- **Chart colours are paired with Species by `zip`.** `fitness_svg` zips `Species::ALL` with `SERIES_COLOURS`, which has 4 entries. A new Species gets the next colour automatically. With more than one Species, add a test that each Species' lines have different strokes.

**From the 04 (viewer plays a Showcase) implementation:**

- **Per-Species stats can't tell Species of one family apart yet.** `Viewer::stats` (`crates/viewer/src/model.rs`) counts an Agent for a Species when its family *and* Role match. `cargo mutants` survives `&&` → `||` there because MLP-Prey is the only Species. Once MLP-Predator exists, add a test to `crates/viewer/tests/` that a Showcase with both Roles reports separate alive and total counts per Species. Also check that Predators get their own `ROLE_COLOURS` entry in the scene, and that `fitness_series` gives one line pair per Species.
