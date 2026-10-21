# run-files Specification

## Purpose

The `batch` CLI command and the library functions behind it: preparing an output directory, recording evolution with periodic Snapshots, and writing the run files `metrics.csv`, `fitness.svg` and `run_summary.json`.

## Requirements

### Requirement: Batch command

`neural-survival batch --config <path> [--seed <u64>] --generations <u32> --out <dir>` SHALL evolve headlessly for the given number of Generations and write the run files and Snapshots to the output directory. When `--seed` is omitted it SHALL generate one and print `Generated seed: <seed>`. It SHALL print `Generation <g> (<completed>/<total>): …` at every progress report.

#### Scenario: A headless batch run writes the required artifacts

- **WHEN** the user runs `neural-survival batch --config config.toml --seed 42 --generations 10 --out results` with no `results` directory
- **THEN** the command succeeds
- **AND** `results` contains `metrics.csv`, `fitness.svg`, `run_summary.json` and `snapshot_gen_10.json`

<!-- verified-by: tests/cli.rs::a_headless_batch_run_writes_the_required_artifacts -->

#### Scenario: An omitted seed is generated and printed

- **WHEN** the user runs `batch` without `--seed`
- **THEN** stdout has a line `Generated seed: <n>` and `run_summary.json` records seed `<n>`

<!-- verified-by: tests/cli.rs::an_omitted_seed_is_generated_and_printed -->

#### Scenario: A given seed is not reported as generated

- **WHEN** the user runs `batch` with `--seed 42`
- **THEN** stdout does not contain `Generated seed`

<!-- verified-by: tests/cli.rs::a_given_seed_is_not_reported_as_generated -->

#### Scenario: Progress is printed every 10 Generations and on completion

- **WHEN** the user runs `batch` for 25 Generations
- **THEN** stdout's progress lines begin `Generation 10 (10/25)`, `Generation 20 (20/25)` and `Generation 25 (25/25)`, in that order

<!-- verified-by: tests/cli.rs::progress_is_printed_every_10_generations_and_on_completion -->

#### Scenario: Snapshots every 10 Generations and for the final Generation

- **WHEN** the user runs `batch` for 25 Generations
- **THEN** the output directory holds exactly `snapshot_gen_10.json`, `snapshot_gen_20.json` and `snapshot_gen_25.json`

<!-- verified-by: tests/cli.rs::snapshots_are_written_every_10_generations_and_for_the_final_generation -->

#### Scenario: An invalid config fails with its error

- **WHEN** the config file contains the unknown key `widht` and the user runs `batch`
- **THEN** the command fails, stderr contains `widht`, and no output directory is created

<!-- verified-by: tests/cli.rs::an_invalid_config_fails_with_its_error -->

### Requirement: Output directory must be absent or empty

The output directory SHALL be created when absent, accepted when empty, and refused when it contains anything, with an error explaining it must be absent or empty, leaving its contents untouched.

#### Scenario: A batch run rejects a non-empty output directory

- **WHEN** `results` contains `notes.txt` and the user runs `batch --out results`
- **THEN** the command fails, stderr contains `must be absent or empty`, and `results` still holds only `notes.txt`

<!-- verified-by: tests/cli.rs::a_batch_run_rejects_a_non_empty_output_directory -->
<!-- verified-by: crates/core/tests/run_files.rs::a_non_empty_output_directory_is_refused_and_left_alone -->

#### Scenario: An existing empty output directory is accepted

- **WHEN** `results` exists and is empty and the user runs `batch --out results`
- **THEN** the command succeeds and writes `results/metrics.csv`

<!-- verified-by: tests/cli.rs::a_batch_run_accepts_an_existing_empty_output_directory -->
<!-- verified-by: crates/core/tests/run_files.rs::an_empty_output_directory_is_accepted -->

#### Scenario: An absent output directory is created

- **WHEN** the output directory `absent/nested` does not exist and is prepared
- **THEN** it exists as a directory

<!-- verified-by: crates/core/tests/run_files.rs::an_absent_output_directory_is_created -->

### Requirement: Recording evolution

Recording SHALL advance an Evolution by the requested Generations, save `snapshot_gen_<n>.json` (named by the current, not yet evaluated Generation) at every progress report before completion, then save the final Snapshot and write the run files covering the full metrics history. It SHALL do so even for 0 Generations. If saving fails, evolution SHALL still complete and the first failure SHALL be reported.

#### Scenario: Snapshot file names

- **WHEN** Snapshot file names are asked for Generations 0 and 120
- **THEN** they are `snapshot_gen_0.json` and `snapshot_gen_120.json`

<!-- verified-by: crates/core/tests/run_files.rs::snapshot_files_are_named_by_generation -->

#### Scenario: Recording a fresh run

- **WHEN** a fresh Evolution with seed 5 is recorded for 25 Generations
- **THEN** progress reports Generations 10, 20, 25
- **AND** the directory holds `snapshot_gen_10.json`, `snapshot_gen_20.json` and `snapshot_gen_25.json`, the last equal to the Evolution's final Snapshot and the first equal to a Snapshot of a separate 10-Generation run
- **AND** `metrics.csv`, `fitness.svg` and `run_summary.json` exist, with 25 metrics rows

<!-- verified-by: crates/core/tests/run_files.rs::recording_saves_a_snapshot_every_10_generations_and_at_the_end -->

#### Scenario: Recording a continuation

- **WHEN** an Evolution already at Generation 25 is recorded for 20 more
- **THEN** the Snapshots are `snapshot_gen_30.json`, `snapshot_gen_40.json`, `snapshot_gen_45.json`
- **AND** `metrics.csv` has 45 rows, the full history

<!-- verified-by: crates/core/tests/run_files.rs::recording_a_continuation_names_snapshots_by_absolute_generation -->

#### Scenario: Recording zero Generations

- **WHEN** a fresh Evolution is recorded for 0 Generations
- **THEN** the directory holds exactly `fitness.svg`, `metrics.csv`, `run_summary.json` and `snapshot_gen_0.json`

<!-- verified-by: crates/core/tests/run_files.rs::recording_zero_generations_still_writes_the_current_generation -->

#### Scenario: Recording into a missing directory

- **WHEN** an Evolution is recorded for 25 Generations into a directory that does not exist
- **THEN** recording fails with a Snapshot error naming `snapshot_gen_10.json`
- **AND** the Evolution is at Generation 25 and the directory still does not exist

<!-- verified-by: crates/core/tests/run_files.rs::recording_into_a_missing_directory_reports_the_failure_after_evolving -->

### Requirement: metrics.csv

`metrics.csv` SHALL have the header `generation,species,episodes,episode_steps,best_fitness,mean_fitness,alive_at_end,food_eaten` and one row per Generation per Species, in Generation order, matching the metrics history. Rewriting the run files SHALL replace them.

#### Scenario: One row per Generation per Species

- **WHEN** the run files are written after 3 Generations of 1 Episode of 20 steps
- **THEN** `metrics.csv` has the header and 3 rows for Generations 0, 1, 2, species `MLP-Prey`, Episodes 1, steps 20, and the metrics history's values

<!-- verified-by: crates/core/tests/run_files.rs::metrics_csv_has_one_row_per_generation_per_species -->

#### Scenario: Rewriting replaces the files

- **WHEN** the run files are written after 2 Generations and again after 3
- **THEN** `metrics.csv` has 3 rows

<!-- verified-by: crates/core/tests/run_files.rs::run_files_are_replaced_when_rewritten -->

### Requirement: run_summary.json

`run_summary.json` SHALL record the seed, the resolved config, the number of evaluated Generations, and the metrics rows of the last evaluated Generation.

#### Scenario: Summary after 4 Generations

- **WHEN** the run files are written for seed 11 after 4 Generations
- **THEN** `run_summary.json` has seed 11, generations 4, the config, and final metrics equal to Generation 3's rows

<!-- verified-by: crates/core/tests/run_files.rs::run_summary_records_the_resolved_config_seed_and_final_generation -->

#### Scenario: Summary of an unevaluated run

- **WHEN** the run files are written with an empty metrics history
- **THEN** `run_summary.json` has generations 0 and an empty final metrics list

<!-- verified-by: crates/core/tests/run_files.rs::run_summary_of_an_unevaluated_run_has_no_generations -->

### Requirement: fitness.svg

`fitness.svg` SHALL plot one best and one mean Fitness line per Species, with Generations evenly spaced left to right and Fitness scaled linearly upwards, every point strictly inside the chart. It SHALL label the first and last Generation and the lowest and highest Fitness (two decimals) level with those points, and list each line in a legend, best above mean, at least 12px apart. With no history it SHALL say `No Generations evaluated yet` and draw no lines.

#### Scenario: Best and mean lines per Species

- **WHEN** the chart is drawn for Generations 3–5 with best Fitness 5, 9, 7 and mean 3, 5, 1
- **THEN** it is an SVG with `MLP-Prey best` and `MLP-Prey mean` lines of 3 points each, all inside the chart
- **AND** the points are evenly spaced left to right at matching x for both lines
- **AND** 9 is at the top, 1 at the bottom, and 5, 3 and 7 at a half, a quarter and three quarters of the way up

<!-- verified-by: crates/core/tests/run_files.rs::fitness_chart_plots_best_and_mean_per_species -->

#### Scenario: Axis labels

- **WHEN** the chart is drawn for the same history
- **THEN** it has text labels `3`, `5`, `1.00` and `9.00`
- **AND** `9.00` and `1.00` sit within 5px of the highest and lowest points' heights

<!-- verified-by: crates/core/tests/run_files.rs::fitness_chart_labels_the_generation_and_fitness_range -->

#### Scenario: Legend

- **WHEN** the chart is drawn for the same history
- **THEN** the legend labels `MLP-Prey best` and `MLP-Prey mean` lie inside the chart, best above mean by at least 12px

<!-- verified-by: crates/core/tests/run_files.rs::fitness_chart_legend_lists_each_line_top_to_bottom -->

#### Scenario: A single flat Generation

- **WHEN** the chart is drawn for one Generation with best and mean Fitness both 5
- **THEN** the best line has one point, inside the chart

<!-- verified-by: crates/core/tests/run_files.rs::fitness_chart_of_a_single_flat_generation_draws_inside_the_chart -->

#### Scenario: No history

- **WHEN** the chart is drawn with an empty metrics history
- **THEN** it contains `No Generations evaluated yet` and no line

<!-- verified-by: crates/core/tests/run_files.rs::fitness_chart_without_history_says_so -->
