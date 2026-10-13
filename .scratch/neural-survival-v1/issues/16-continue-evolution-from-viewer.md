# 16: Continuing evolution from the viewer

**What to build:** From the viewer, train y more Generations in the background using the edited config, then show the new Generation.

**Blocked by:** 13: `run` command and continuing from a Snapshot; 15: Viewer config editor and edit saves

**Status:** ready-for-agent

- [ ] The user enters y and starts continuation; Evolution runs on a background thread over copies of the Genomes and edited config
- [ ] The Showcase keeps animating while training runs
- [ ] Progress shows every 10 Generations and the Fitness chart updates as training proceeds
- [ ] When training finishes, the viewer switches automatically to a Showcase of the new Generation
- [ ] Output goes to the loaded Snapshot's directory: a Snapshot every 10 Generations and at the end, plus rewritten `metrics.csv`, `fitness.svg`, and `run_summary.json` with the full history
- [ ] Results equal a headless continuation from the same Snapshot and config (ADR-0001)

## Comments

**From the 02 (Evolution library) implementation and review:**

- Progress timing and the `advance(0)` edge case are described in 13's Comments. The same rules apply here for a y of 0 and for the per-10-Generation Snapshots.
