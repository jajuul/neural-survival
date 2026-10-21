# snapshots Specification

## Purpose

Saving a Generation as a versioned JSON Snapshot, loading it back, continuing evolution from it, and playing it as a Showcase.

## Requirements

### Requirement: Snapshot contents

A Snapshot SHALL hold the format version, the run seed, the Generation index, the resolved config, every Genome, and the metrics history, and no world state.

#### Scenario: A Snapshot of Generation 3

- **WHEN** an Evolution with seed 5 advances 3 Generations and is snapshotted
- **THEN** the Snapshot holds the current format version, seed 5, Generation 3, the config, the current MLP-Prey Genomes, and the 3 metrics rows

<!-- verified-by: crates/core/tests/snapshot.rs::snapshot_holds_the_generation_without_world_state -->

### Requirement: Save and load

A Snapshot SHALL round-trip unchanged through saving to and loading from a JSON file. Loading SHALL fail with a clear error naming both versions when the format version is unsupported, and naming the path when the file is missing.

#### Scenario: Round trip

- **WHEN** a Snapshot is saved and loaded again
- **THEN** the loaded Snapshot equals the original

<!-- verified-by: crates/core/tests/snapshot.rs::snapshot_round_trips_through_save_and_load -->

#### Scenario: Unsupported version

- **WHEN** a Snapshot saved with format version 999 is loaded
- **THEN** loading fails with an unsupported-version error whose message contains 999 and the supported version

<!-- verified-by: crates/core/tests/snapshot.rs::unsupported_snapshot_version_is_rejected_clearly -->

#### Scenario: Missing file

- **WHEN** a Snapshot is loaded from `does_not_exist.json`, which does not exist
- **THEN** the error message contains `does_not_exist.json`

<!-- verified-by: crates/core/tests/snapshot.rs::loading_a_missing_snapshot_names_the_path -->

### Requirement: Continuing from a Snapshot

Evolution SHALL be constructible from a Snapshot, and continuing it SHALL give exactly the result of an uninterrupted run (ADR-0001). A Snapshot whose Species population does not match its config SHALL be rejected.

#### Scenario: A resumed run matches an uninterrupted one

- **WHEN** one run with seed 31 advances 5 Generations, and another advances 2, is saved, loaded, rebuilt from the Snapshot and advances 3 more
- **THEN** the resumed run is at Generation 5 and equals the uninterrupted one

<!-- verified-by: crates/core/tests/snapshot.rs::continuing_a_saved_snapshot_matches_an_uninterrupted_run -->

#### Scenario: Wrong population size

- **WHEN** a Snapshot for 5 MLP Prey has one Genome removed and Evolution is built from it
- **THEN** building fails with a population-size error expecting 5 and finding 4, whose message names MLP-Prey

<!-- verified-by: crates/core/tests/snapshot.rs::snapshot_with_wrong_population_size_is_rejected -->

### Requirement: Showcase from a Snapshot

A Showcase SHALL be constructible from a Snapshot, playing its Genomes in order, seeded from the run seed and Generation, and lasting the Episode step limit the Evaluation Schedule gives that Generation.

#### Scenario: The Snapshot's Genomes, deterministically

- **WHEN** a Showcase is built from a Generation-2 Snapshot of 5 Prey
- **THEN** it has 5 Agents playing the Snapshot's Genomes in order
- **AND** two Showcases from the same Snapshot are equal after 10 steps
- **AND** changing the Snapshot's Generation or run seed gives a different starting world

<!-- verified-by: crates/core/tests/snapshot.rs::showcase_from_snapshot_plays_its_genomes_with_a_seed_from_run_seed_and_generation -->

#### Scenario: Episode length follows the Snapshot's Generation

- **WHEN** a Generation-1 Snapshot's schedule gives Generation 1 a 9-step Episode, and its Showcase is asked to step 1000 times
- **THEN** it stops at step 9

<!-- verified-by: crates/core/tests/snapshot.rs::showcase_from_snapshot_lasts_as_long_as_its_generations_episodes -->
