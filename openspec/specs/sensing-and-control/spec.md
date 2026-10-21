# sensing-and-control Specification

## Purpose

What an Agent senses, how its MLP controller maps that to outputs, and the per-Agent telemetry a Showcase exposes.

## Requirements

### Requirement: Food Eye Bins

An Agent's field of view, centred on its heading, SHALL be split into the configured number of equal sectors, ordered from the clockwise-most (bin 0). Each bin SHALL read `1 − distance / vision range` for the nearest food in that sector within vision range, measured across torus edges, and 0 when there is none. Food at distance 0 SHALL read as 1.0 in the bin dead ahead.

#### Scenario: Bins activate for the nearest food in each sector

- **WHEN** a Prey heading 0 with a 90° field of view, 4 bins and vision range 20 has food at (30°, 10), (−10°, 5), (−20°, 15), (−40°, 21) and (180°, 5) relative to it, and the Showcase steps once
- **THEN** its telemetry has one eye, of kind Food, with bins [0, 0.75, 0, 0.5]

<!-- verified-by: crates/core/tests/telemetry.rs::food_eye_bins_activate_for_the_nearest_food_in_each_sector -->

#### Scenario: Bins follow the heading and see across the torus edge

- **WHEN** a Prey at x = 1 heading west (π), with 3 bins and vision range 20, has food at x = 45 in a 50-wide world
- **THEN** its bins read [0, 0.7, 0]

<!-- verified-by: crates/core/tests/telemetry.rs::eye_bins_follow_the_heading_and_see_across_the_torus_edge -->

#### Scenario: Food exactly on the Agent

- **WHEN** food lies exactly at a Prey's position, with 3 bins
- **THEN** its bins read [0, 1, 0]

<!-- verified-by: crates/core/tests/telemetry.rs::food_exactly_on_the_agent_reads_as_full_activation_dead_ahead -->

### Requirement: Core sensors

An Agent SHALL sense its energy normalised by the configured energy scale and capped at 1, and its speed normalised by max speed.

#### Scenario: Normalised energy and speed

- **WHEN** a Prey with energy 50, energy scale 200, idle drain 1 and max speed 2 steps once at throttle 0
- **THEN** its core sensors read energy 0.25 and speed 0
- **WHEN** it steps again
- **THEN** they read energy 49/200 and speed 0.125

<!-- verified-by: crates/core/tests/telemetry.rs::core_sensors_report_normalised_energy_and_speed -->

#### Scenario: Normalised energy saturates at one

- **WHEN** a Prey has energy 500 with energy scale 200
- **THEN** its core energy sensor reads 1.0

<!-- verified-by: crates/core/tests/telemetry.rs::normalised_energy_saturates_at_one -->

### Requirement: Controller input layout

The controller input vector SHALL be normalised energy, then normalised speed, then the food Eye Bins in bin order.

#### Scenario: Inputs are energy, speed, then food bins

- **WHEN** a still Prey with energy 50, energy scale 100 and 3 bins sees food dead ahead at distance 8 with vision range 20
- **THEN** its controller inputs are [0.5, 0, 0, 0.6, 0]

<!-- verified-by: crates/core/tests/telemetry.rs::controller_input_is_energy_then_speed_then_food_bins -->

### Requirement: MLP controllers

A randomly initialised Prey Genome SHALL hold an MLP whose layers are the controller input width, the configured hidden layer sizes, and 2 outputs. Weights SHALL be drawn uniformly within the Xavier bound `±sqrt(6 / (fan_in + fan_out))` and biases SHALL start at 0. Both outputs SHALL be tanh-activated, so they stay within [−1, 1].

#### Scenario: Shape and Xavier-scaled weights

- **WHEN** a Showcase is built with 3 Prey, 4 Eye Bins and hidden layers [30, 20]
- **THEN** each Genome is an MLP Prey with layer sizes [6, 30, 20, 2]
- **AND** every weight lies within its layer's Xavier bound, the largest reaches above 80% of it, and every bias is 0
- **AND** there is no Genome for Agent 3

<!-- verified-by: crates/core/tests/mlp_init.rs::random_prey_get_mlps_shaped_by_config_with_xavier_weights -->

#### Scenario: Different Prey get different weights

- **WHEN** a Showcase is built from the default config
- **THEN** the Genomes of Agents 0 and 1 differ

<!-- verified-by: crates/core/tests/mlp_init.rs::different_prey_get_different_initial_weights -->

#### Scenario: Outputs stay within the tanh range

- **WHEN** a 10-Prey Showcase runs its 50-step Episode
- **THEN** every living Agent's raw steering and throttle lie within [−1, 1] after every step

<!-- verified-by: crates/core/tests/mlp_init.rs::controller_outputs_stay_within_tanh_range -->

### Requirement: Telemetry

A Showcase SHALL expose, for each Agent by id after its first step, its Eye Bins per eye kind, its core sensors, its raw controller outputs, the applied steering and thrust, and the energy cost of each Energy Rule that step. Telemetry SHALL be absent before the first step and for unknown ids, and an Agent's last telemetry SHALL be kept unchanged after it dies.

#### Scenario: Absent before the first step and for unknown Agents

- **WHEN** a one-Prey Showcase has not stepped
- **THEN** Agent 0 has no telemetry
- **WHEN** it steps once
- **THEN** Agent 0 has telemetry and Agent 1 has none

<!-- verified-by: crates/core/tests/telemetry.rs::telemetry_is_absent_before_the_first_step_and_for_unknown_agents -->

#### Scenario: Raw and applied outputs

- **WHEN** a Prey outputs steering −0.5 and throttle −0.6
- **THEN** its telemetry reports raw steering −0.5, raw throttle −0.6, applied steering −0.5, and applied thrust 0.2

<!-- verified-by: crates/core/tests/telemetry.rs::telemetry_reports_raw_and_applied_outputs -->

#### Scenario: Energy cost per Energy Rule

- **WHEN** a Prey with idle drain 0.3 steps once
- **THEN** its telemetry lists exactly one energy cost: Idle Drain, 0.3

<!-- verified-by: crates/core/tests/telemetry.rs::telemetry_lists_energy_cost_per_energy_rule -->

#### Scenario: Last telemetry is kept after death

- **WHEN** a lone Prey with energy 1.0 and idle drain 0.6 dies on its second step
- **THEN** its telemetry reads core energy 0.4 / energy scale, from its last living step
- **AND** the Showcase has ended and the telemetry is still readable and unchanged

<!-- verified-by: crates/core/tests/telemetry.rs::last_telemetry_is_kept_after_the_agent_dies -->

#### Scenario: A dead Agent's telemetry is not overwritten while others live

- **WHEN** one of two Prey dies on step 2 while the other is kept fed, and the Showcase steps 3 more times
- **THEN** the Showcase has not ended and the dead Prey's telemetry equals what it was at death

<!-- verified-by: crates/core/tests/telemetry.rs::dead_agents_telemetry_is_not_overwritten_while_others_live -->
