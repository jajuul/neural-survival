# io

## Purpose
This module persists experiment artifacts (`metrics.csv`, `run_summary.json`, snapshots).

## Behavior
- Validates output directory safety (must be absent or empty).
- Writes generation metrics CSV rows.
- Writes run summary JSON with config hash and resolved config.
- Writes and reads snapshot JSON payloads.

## Inputs
- Generation metrics, run metadata, snapshot payloads, and output paths.

## Outputs
- `metrics.csv`
- `run_summary.json`
- `snapshot_gen_<n>.json`

## Side Effects
- Filesystem reads/writes in target output directory.
