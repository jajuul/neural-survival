# 13: `run` command and continuing from a Snapshot

**What to build:** The `run` command trains headlessly and then opens the viewer. Underneath, Evolution can continue from a Snapshot with results identical to an uninterrupted run.

**Blocked by:** 03: `batch` and run files; 04: Viewer: `view` plays a Showcase

**Status:** ready-for-agent

- [ ] Evolution constructed from a Snapshot and advanced M Generations matches an uninterrupted run of the same total length (ADR-0001)
- [ ] `neural-survival run --config <path> [--seed <u64>] --headless-generations <u32> --out <dir>` evolves exactly the requested Generations, writes the same files as `batch`, then opens the viewer on the final Snapshot
- [ ] A non-empty output directory is accepted: only `metrics.csv`, `fitness.svg`, `run_summary.json`, and `snapshot_gen_*.json` are removed first; other files are kept
- [ ] Prints the seed when generated and progress every 10 Generations
- [ ] A hidden option skips opening the window so tests can run `run` as a process
- [ ] CLI acceptance test covers the spec's train-then-view scenario (exact Generation count, full Fitness history in the final Snapshot) and stale-file clearing
- [ ] Integration test covers continuation equivalence
