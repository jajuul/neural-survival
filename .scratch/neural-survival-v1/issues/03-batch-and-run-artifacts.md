# 03: `batch` and run files

**What to build:** The `batch` CLI command: evolve headlessly for a fixed number of Generations and write the run artifacts. Artifact writing is a library function (metrics history + resolved config + seed → directory) so later the GUI continuation can reuse it.

**Blocked by:** 02: Evolution library

**Status:** ready-for-agent

- [ ] `neural-survival batch --config <path> [--seed <u64>] --generations <u32> --out <dir>` runs headless evolution
- [ ] Fails with an error explaining the output directory must be absent or empty when it exists and is non-empty
- [ ] Prints the generated seed when `--seed` is omitted
- [ ] Prints progress every 10 Generations and on completion
- [ ] Writes `metrics.csv` (one row per Generation per Species: best Fitness, mean Fitness, alive at Episode end, food eaten), `fitness.svg` (best and mean per Species), and `run_summary.json` (including the resolved config and seed)
- [ ] Writes `snapshot_gen_<n>.json` every 10 Generations and for the final Generation
- [ ] CLI acceptance tests run the binary as a process and cover the spec's two batch scenarios
