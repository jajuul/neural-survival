# world-simulation Specification

## Purpose

The headless world a Showcase steps: a single Arena on a 2D torus with MLP Prey and stationary food, movement, eating, idle drain, death, Episode end, and determinism (ADR-0001).

## Requirements

### Requirement: Showcase construction

A Showcase SHALL be built from a config and seed, containing one Arena with the configured number of Prey, each alive with the configured starting energy and placed inside the world, and the configured number of food items. A Showcase builder SHALL accept explicit Genomes, placements and food, and SHALL reject inputs that do not fit the config.

#### Scenario: A new Showcase holds the configured Prey and food

- **WHEN** a Showcase is built with seed 1 from a 50 × 40 world config with 4 Prey and 6 food
- **THEN** its step count is 0 and it has not ended
- **AND** it has one Arena with 4 Agents and 6 food items
- **AND** every Agent is alive, has the configured starting energy, and lies within x ∈ [0, 50), y ∈ [0, 40)

<!-- verified-by: crates/core/tests/showcase.rs::showcase_starts_with_configured_prey_and_food -->

#### Scenario: Placement count must match the Genomes

- **WHEN** a Showcase is built for 4 Prey with an empty list of placements
- **THEN** building fails with a placement-count error reporting 4 Genomes and 0 placements

<!-- verified-by: crates/core/tests/showcase.rs::builder_rejects_placements_that_do_not_match_genomes -->

#### Scenario: A Genome's network must fit the config

- **WHEN** a Showcase is built with a Genome whose network has one more input than the config's controller input width
- **THEN** building fails with a network-shape error for Genome index 0

<!-- verified-by: crates/core/tests/showcase.rs::builder_rejects_genomes_whose_network_does_not_fit_the_config -->

#### Scenario: An invalid config is rejected

- **WHEN** a Showcase is built from a config with `sensors.eye_bins = 0`
- **THEN** building fails with a config error

<!-- verified-by: crates/core/tests/showcase.rs::showcase_rejects_an_invalid_config -->

### Requirement: Movement by steering and thrust

Each step, an Agent's steering output in [-1, 1] SHALL change its heading by steering × turn rate, and its throttle output in [-1, 1] SHALL map linearly to thrust in [0, 1]. Speed SHALL increase by thrust × max acceleration, then lose the configured linear drag fraction, and SHALL never exceed max speed. The Agent then moves forward along its heading by its speed.

#### Scenario: Steering turns the heading

- **WHEN** a Prey with heading 1.0, turn rate 0.3 and constant steering 0.5 is stepped twice
- **THEN** its heading is 1.15 after the first step and 1.30 after the second

<!-- verified-by: crates/core/tests/movement.rs::steering_turns_heading_by_turn_rate -->

#### Scenario: Throttle gives thrust with drag

- **WHEN** a Prey at (10, 10) heading east, with max acceleration 0.5, drag 0.1 and constant throttle 0, is stepped twice
- **THEN** after one step its speed is 0.25 and it is at (10.25, 10)
- **AND** after two steps its speed is 0.475 and its x is 10.725

<!-- verified-by: crates/core/tests/movement.rs::throttle_maps_to_forward_thrust_with_drag -->

#### Scenario: Full reverse throttle gives no thrust

- **WHEN** a Prey at rest outputs throttle −1 for 5 steps
- **THEN** its speed stays 0 and it does not move

<!-- verified-by: crates/core/tests/movement.rs::full_reverse_throttle_gives_no_thrust -->

#### Scenario: Speed is capped

- **WHEN** a Prey with max speed 2.0 outputs full throttle for 50 steps
- **THEN** its speed never exceeds 2.0 and ends at 2.0

<!-- verified-by: crates/core/tests/movement.rs::speed_is_capped_at_max_speed -->

### Requirement: The world is a torus

Positions SHALL wrap around every edge of the world, so an Agent leaving one edge reappears at the opposite edge and always lies within [0, width) × [0, height).

#### Scenario: Agents wrap at the east and south edges

- **WHEN** a Prey at full speed heads east from x = 49 in a 50-wide world
- **THEN** it is at x = 49.5 after one step and x = 0.45 after the next
- **WHEN** a Prey at full speed heads south (−y) from y = 0.2 in a 40-high world
- **THEN** after one step it is at y = 39.7

<!-- verified-by: crates/core/tests/movement.rs::agents_wrap_around_torus_edges -->

#### Scenario: Positions stay inside the world

- **WHEN** a steering, full-throttle Prey is stepped 500 times in a 50 × 40 world
- **THEN** after every step its position lies within [0, 50) × [0, 40)

<!-- verified-by: crates/core/tests/movement.rs::positions_stay_inside_the_world_over_a_long_run -->

### Requirement: Prey eat food on contact

A Prey SHALL eat every food item whose distance from it, measured across torus edges, is within the sum of its body radius and the food radius, gaining that food's energy. Eaten food SHALL respawn immediately at a new position inside the world; uneaten food stays where it is.

#### Scenario: Touching food is eaten

- **WHEN** a still Prey with body radius 2, energy 50 and idle drain 0.5 is 2.5 units from food of radius 1 and energy 30, and the Showcase steps once
- **THEN** the Prey's energy is 79.5

<!-- verified-by: crates/core/tests/food_and_energy.rs::prey_eat_touching_food_and_gain_its_energy -->

#### Scenario: Food out of contact is not eaten

- **WHEN** the same Prey is 3.1 units from the food and the Showcase steps once
- **THEN** the Prey's energy is 49.5 and the food has not moved

<!-- verified-by: crates/core/tests/food_and_energy.rs::food_out_of_contact_is_not_eaten -->

#### Scenario: Food is eaten across the torus edge

- **WHEN** a still Prey at x = 0.5 touches food at x = 49.5 in a 50-wide world
- **THEN** after one step its energy is 50 + 30 − 0.5 = 79.5

<!-- verified-by: crates/core/tests/food_and_energy.rs::prey_eat_food_across_the_torus_edge -->

#### Scenario: Eaten food respawns immediately elsewhere

- **WHEN** a Prey eats one of two food items in one step
- **THEN** there are still 2 food items
- **AND** the eaten one has a new position, the other is unchanged, and both lie inside the world

<!-- verified-by: crates/core/tests/food_and_energy.rs::eaten_food_respawns_immediately_elsewhere -->

### Requirement: Idle drain and death

Each step, every living Agent SHALL lose the configured idle drain. An Agent whose energy reaches zero SHALL die with energy 0, and a dead Agent SHALL never change again.

#### Scenario: Idle drain reduces energy

- **WHEN** a still Prey with energy 10 and idle drain 0.25 steps 4 times without food
- **THEN** its energy is 9.0

<!-- verified-by: crates/core/tests/food_and_energy.rs::idle_drain_reduces_energy_each_step -->

#### Scenario: An Agent at zero energy dies and stays dead

- **WHEN** a Prey starting with energy 1.0 and idle drain 0.4 steps 3 times, while a second Prey is kept fed
- **THEN** the first Prey is alive after 2 steps and dead with energy 0 after the third
- **AND** after 5 more steps it is unchanged, and the Showcase has not ended

<!-- verified-by: crates/core/tests/food_and_energy.rs::agent_at_zero_energy_dies_and_stays_dead -->

### Requirement: Episode end

A Showcase SHALL end when its step count reaches the Episode step limit or when every Agent is dead. Stepping an ended Showcase SHALL change nothing.

#### Scenario: The Showcase ends at the step limit

- **WHEN** a Showcase with a 30-step limit steps 29 times
- **THEN** it has not ended
- **WHEN** it steps once more
- **THEN** it has ended with step count 30
- **AND** a further step leaves its step count and world unchanged

<!-- verified-by: crates/core/tests/showcase.rs::showcase_ends_at_step_limit_and_stops_advancing -->

#### Scenario: The Showcase ends when every Agent is dead

- **WHEN** a single Prey with energy 1.0 and idle drain 0.5 steps twice without food
- **THEN** the Showcase has not ended after the first step and has ended after the second, with step count 2

<!-- verified-by: crates/core/tests/food_and_energy.rs::showcase_ends_when_every_agent_is_dead -->

#### Scenario: Stepping in batches stops at Episode end

- **WHEN** a Showcase with a 400-step limit is asked to step 10,000 times at once
- **THEN** it has ended with a step count of at most 400

<!-- verified-by: crates/core/tests/determinism.rs::step_many_stops_at_episode_end -->

### Requirement: Determinism

Two Showcases built from the same seed and config SHALL produce identical world state and telemetry after the same number of steps, whether stepped one at a time or in batches. Different seeds SHALL give different worlds.

#### Scenario: Same seed gives identical state

- **WHEN** two Showcases are built with seed 1234 from the same 8-Prey config and each stepped 200 times
- **THEN** their Arenas are equal and the telemetry of every Agent is equal

<!-- verified-by: crates/core/tests/determinism.rs::same_seed_and_config_give_identical_state_after_n_steps -->

#### Scenario: Single steps match batches

- **WHEN** one Showcase steps 250 times one at a time and another with the same seed steps in batches of 1, 100 and 149
- **THEN** both have step count 250 and are equal

<!-- verified-by: crates/core/tests/determinism.rs::stepping_one_at_a_time_matches_stepping_in_batches -->

#### Scenario: Different seeds give different worlds

- **WHEN** Showcases are built from the same config with seeds 1 and 2
- **THEN** their Arenas differ

<!-- verified-by: crates/core/tests/determinism.rs::different_seeds_give_different_worlds -->

#### Scenario: The world changes over time

- **WHEN** a Showcase steps 50 times
- **THEN** its Arenas differ from their starting state

<!-- verified-by: crates/core/tests/determinism.rs::the_world_actually_changes_over_time -->
