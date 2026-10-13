# 03: `batch` and run files

**What to build:** The `batch` CLI command: evolve headlessly for a fixed number of Generations and write the run artifacts. Artifact writing is a library function (metrics history + resolved config + seed → directory) so later the GUI continuation can reuse it.

**Blocked by:** 02: Evolution library

**Spec:** User Stories 1–2, 7–8, 12, 104–107; Implementation Decisions › Artifact writing, CLI; Testing Decisions › Seam 2; Acceptance Scenarios › Batch experiment outputs (`../spec.md`)

**Status:** resolved

- [x] `neural-survival batch --config <path> [--seed <u64>] --generations <u32> --out <dir>` runs headless evolution
- [x] Fails with an error explaining the output directory must be absent or empty when it exists and is non-empty
- [x] Prints the generated seed when `--seed` is omitted
- [x] Prints progress every 10 Generations and on completion
- [x] Writes `metrics.csv` (one row per Generation per Species: best Fitness, mean Fitness, alive at Episode end, food eaten), `fitness.svg` (best and mean per Species), and `run_summary.json` (including the resolved config and seed)
- [x] Writes `snapshot_gen_<n>.json` every 10 Generations and for the final Generation
- [x] CLI acceptance tests run the binary as a process and cover the spec's two batch scenarios

## Comments

**From the 03 implementation and review:**

- **Run files live in `crates/core/src/run_files.rs`.** `write_run_files(dir, config, seed, metrics)` is the library function the spec asks for. `record_evolution(evolution, generations, dir, on_progress)` wraps `Evolution::advance`. It saves a Snapshot whenever the callback fires before completion, and after advancing it saves the final Snapshot and writes the run files, even for 0 Generations. `prepare_output_dir` does `batch`'s absent-or-empty check.
- **`snapshot_gen_<n>` is named after the current Generation**, the bred and not yet evaluated one that `Evolution::snapshot` holds. After `--generations 10`, the metrics cover Generations 0 to 9 and the final file is `snapshot_gen_10.json`.
- **`Progress` now carries `evolution: &Evolution`** so callbacks can save Snapshots.
- **Deliberate extras:** `metrics.csv` also has `episodes` and `episode_steps` columns, recording the Evaluation Schedule in force, and `run_summary.json` also has `generations` and `final_metrics`.
- **Mutation testing:** `cargo mutants` on the diff: 54 caught, 1 unviable, 0 missed.
