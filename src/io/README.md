# io

## Purpose
This module persists experiment artifacts (`metrics.csv`, `fitness.svg`, `run_summary.json`, snapshots).

## Behavior
- Validates output directory safety (must be absent or empty).
- Writes generation metrics CSV rows.
- Writes an SVG plot of best and mean fitness over generations.
- Writes run summary JSON with config hash and resolved config.
- Writes and reads snapshot JSON payloads, including embedded generation-metric history used by the GUI fitness plot.

## Inputs
- Generation metrics, run metadata, snapshot payloads, and output paths.

## Outputs
- `metrics.csv`
- `fitness.svg`
- `run_summary.json`
- `snapshot_gen_<n>.json`

## Side Effects
- Filesystem reads/writes in target output directory.
