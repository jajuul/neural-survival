# config Specification

## Purpose

How a run's settings are read from TOML, defaulted, validated, and written back out as the resolved config.

## Requirements

### Requirement: Config parses from TOML with defaults

The library SHALL parse a config from TOML text, filling every field the text omits with its default value.

#### Scenario: An empty file yields the fully defaulted config

- **WHEN** an empty string is parsed as a config
- **THEN** the result equals the default config

<!-- verified-by: crates/core/tests/config.rs::empty_file_yields_fully_defaulted_config -->

#### Scenario: A partial file overrides only the fields it gives

- **WHEN** the text `[world]` / `width = 12.5` is parsed
- **THEN** `world.width` is 12.5
- **AND** `world.height` and every `food` setting keep their default values

<!-- verified-by: crates/core/tests/config.rs::partial_file_overrides_only_given_fields -->

### Requirement: Unknown keys are rejected

The library SHALL reject a config containing a key it does not recognise, at any nesting level, with an error that names the key.

#### Scenario: Unknown top-level key

- **WHEN** the text `wrold_size = 3` is parsed
- **THEN** parsing fails with an error message containing `wrold_size`

<!-- verified-by: crates/core/tests/config.rs::unknown_top_level_key_is_an_error_naming_the_key -->

#### Scenario: Unknown nested key

- **WHEN** the text `[food]` / `energyy = 3.0` is parsed
- **THEN** parsing fails with an error message containing `energyy`

<!-- verified-by: crates/core/tests/config.rs::unknown_nested_key_is_an_error_naming_the_key -->

### Requirement: Invalid values are rejected

The library SHALL reject a config whose values are out of range, with an error naming the offending field by its dotted path.

#### Scenario: Out-of-range world and sensor values

- **WHEN** a config with `sensors.eye_bins = 0` is parsed
- **THEN** parsing fails with an error message containing `sensors.eye_bins`
- **WHEN** a config with `world.width = -1.0` is parsed
- **THEN** parsing fails with an error message containing `world.width`

<!-- verified-by: crates/core/tests/config.rs::invalid_values_are_rejected_naming_the_field -->

#### Scenario: Out-of-range evolution settings

- **WHEN** a config is parsed with any one of: two Evaluation Schedule stages starting at the same Generation, `evaluation.episodes = 0`, `evaluation.episode_steps = 0`, a stage with 0 Episodes or 0 steps, `ga.tournament_size = 0`, `ga.mutation_rate = 1.5`, or `ga.mutation_sigma = -0.1`
- **THEN** parsing fails with an error naming the field: `evaluation.stages`, `evaluation.episodes`, `evaluation.episode_steps`, `ga.tournament_size`, `ga.mutation_rate`, or `ga.mutation_sigma` respectively

<!-- verified-by: crates/core/tests/config.rs::evolution_settings_are_validated -->

### Requirement: The resolved config serialises back out

The library SHALL serialise a resolved config to TOML that contains every field and parses back to an equal config.

#### Scenario: A modified config round-trips

- **WHEN** a default config with `world.width = 33.0`, `mlp.hidden_layers = [4, 3]` and `sensors.eye_bins = 7` is serialised and parsed again
- **THEN** the serialised text contains `hidden_layers`
- **AND** the parsed config equals the original

<!-- verified-by: crates/core/tests/config.rs::resolved_config_serialises_back_out_and_round_trips -->

#### Scenario: Defaults serialise every field

- **WHEN** the default config is serialised
- **THEN** the text contains every section (`[world]`, `[food]`, `[population]`, `[prey]`, `[agent]`, `[energy]`, `[sensors]`, `[mlp]`, `[evaluation]`, `[fitness.prey]`, `[ga]`) and every field within them

<!-- verified-by: crates/core/tests/config.rs::defaults_serialise_every_field -->

### Requirement: Evaluation Schedule stages

The Evaluation Schedule SHALL give a base Episode count and Episode step limit, optionally overridden by stages that apply from a given Generation onwards; the latest stage whose starting Generation has been reached applies.

#### Scenario: Stages override from their Generation

- **WHEN** the base schedule is 2 Episodes of 100 steps, with stages from Generation 10 (3 × 200) and from Generation 20 (4 × 400)
- **THEN** Generations 0 and 9 get 2 × 100, Generations 10 and 19 get 3 × 200, and Generation 500 gets 4 × 400
- **AND** the config round-trips through TOML unchanged

<!-- verified-by: crates/core/tests/config.rs::evaluation_schedule_stages_parse_and_override_from_their_generation -->
