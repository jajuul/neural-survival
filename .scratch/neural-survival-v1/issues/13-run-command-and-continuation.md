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

## Comments

**From the 02 (Evolution library) implementation and review:**

- **Continuation equivalence is already covered.** `Evolution::from_snapshot` exists, and `continuing_a_saved_snapshot_matches_an_uninterrupted_run` (`crates/core/tests/snapshot.rs`) checks it through save and load on disk. The first and last checkboxes here then only need wiring into `run`, plus a CLI-level check if you want one.
- **Progress is counted from absolute Generation numbers.** `Evolution::advance` calls back whenever the current Generation is a multiple of `PROGRESS_INTERVAL` (10), and once on completion. A run continued from Generation 25 reports at 30, 40 and then 45. If you hang "write a Snapshot every 10 Generations" off this callback, the files come out as `snapshot_gen_30`, `snapshot_gen_40` and then the final `snapshot_gen_45`.
- **`advance(0)` never calls the callback.** Decide what `--headless-generations 0` should do: reject it, or still write the run files and a Generation-0 Snapshot. Don't rely on the completion callback for the final files, because with 0 it never fires.

**From the 03 (`batch` and run files) implementation and review:**

- **Reuse `record_evolution`** (`crates/core/src/run_files.rs`). It already does "a Snapshot every 10 Generations and at the end", then writes `metrics.csv`, `fitness.svg`, and `run_summary.json`. It handles 0 Generations by writing a Generation-0 Snapshot and the run files. `batch` in `src/main.rs` shows the wiring, including seed printing and `print_progress`.
- **Clearing stale files needs the file names.** The run-file names are private constants in `run_files.rs` (`METRICS_FILE`, `FITNESS_CHART_FILE`, `RUN_SUMMARY_FILE`), and `snapshot_file_name` is public. Add a library function such as `clear_run_files(dir)` next to `prepare_output_dir` instead of hard-coding the names in the CLI. It must leave `snapshot_gen_<n>_edit_<k>.json` (ticket 15) alone, or decide explicitly that `snapshot_gen_*` covers edits too.
- **CLI tests:** `tests/cli.rs` has a `Workspace` helper (tempdir plus a tiny config) to reuse for `run`.
