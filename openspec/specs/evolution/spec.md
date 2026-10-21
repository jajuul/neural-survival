# evolution Specification

## Purpose

Advancing Generations in the headless library: evaluating Genomes over the Episodes the Evaluation Schedule prescribes, scoring Fitness, breeding each Species with the genetic algorithm, reporting progress, and recording metrics. Only the MLP-Prey Species exists so far.

## Requirements

### Requirement: Evolution starts at Generation 0

Evolution SHALL be constructible from a resolved config and a seed, starting at Generation 0 with the configured population of randomly initialised Genomes per Species and an empty metrics history.

#### Scenario: A fresh Evolution

- **WHEN** an Evolution is built with seed 42 from a config with 6 MLP Prey
- **THEN** it reports that config, Generation 0 and seed 42, and holds no metrics
- **AND** the MLP-Prey Species holds 6 Genomes, each an MLP Prey

<!-- verified-by: crates/core/tests/evolution.rs::evolution_starts_at_generation_zero_with_configured_population -->

### Requirement: Species are fixed

Each Species' population size and the Role and Controller Family of its Genomes SHALL stay fixed across Generations (ADR-0002).

#### Scenario: Population and identity persist

- **WHEN** an Evolution with 6 MLP Prey advances 3 Generations
- **THEN** it is at Generation 3 with 6 MLP Prey Genomes, which differ from the starting ones

<!-- verified-by: crates/core/tests/evolution.rs::species_population_sizes_and_identity_stay_fixed_across_generations -->

### Requirement: Evaluation follows the Evaluation Schedule

Each Generation SHALL run the number of Episodes, each with the step limit, that the Evaluation Schedule gives for that Generation.

#### Scenario: A stage changes Episode count and length

- **WHEN** Prey that cannot die, scored 1 per step survived, evolve 3 Generations under a schedule of 1 × 4 steps with a stage from Generation 2 of 3 × 7 steps
- **THEN** the metrics rows record (Generation, Episodes, steps, best Fitness) as (0, 1, 4, 4), (1, 1, 4, 4), (2, 3, 7, 7)

<!-- verified-by: crates/core/tests/evolution.rs::evaluation_schedule_sets_episode_count_and_length_per_generation -->

### Requirement: Fitness

An Agent's Fitness in an Episode SHALL be `w_survival × steps alive + w_energy × energy gained`, with the weights of its Role from config. Steps alive SHALL stop counting at death. A Genome's Fitness SHALL be the mean over its Generation's Episodes.

#### Scenario: Weighted survival plus energy gained

- **WHEN** 3 Prey that eat both food items (5 energy each) every step evolve 1 Generation of 2 Episodes of 5 steps, with survival weight 2 and energy weight 0.5
- **THEN** Generation 0's MLP-Prey row reports best and mean Fitness 35 (2 × 5 + 0.5 × 50), 3 alive at Episode end, and 30 food eaten

<!-- verified-by: crates/core/tests/evolution.rs::fitness_is_weighted_steps_alive_plus_energy_gained -->

#### Scenario: Steps alive stop at death

- **WHEN** Prey with energy 1.0, idle drain 0.3, no food and survival weight 2 are evaluated for one Generation
- **THEN** best and mean Fitness are 6 (3 steps alive × 2) and none are alive at Episode end

<!-- verified-by: crates/core/tests/evolution.rs::steps_alive_stop_counting_at_death -->

### Requirement: Genetic algorithm per Species

Each Species SHALL breed its next Generation by: copying its top-k Genomes by Fitness unchanged as elites, first; filling the rest with offspring of two parents each chosen by tournament selection of the configured size; uniform crossover taking each parameter from one parent; and Gaussian mutation of each parameter with the configured rate and sigma.

#### Scenario: Elitism copies exactly the top k unchanged

- **WHEN** 8 Genomes breed one Generation with mutation rate 1 and elitism 0, 1 or 3
- **THEN** exactly that many children equal a parent, and they lead the next Generation

<!-- verified-by: crates/core/tests/genetic_algorithm.rs::elitism_copies_exactly_the_top_k_unchanged -->

#### Scenario: Elites are the fittest Genomes

- **WHEN** 6 Genomes breed with elitism 1, tournament size 6 and no mutation
- **THEN** every child equals the same parent Genome

<!-- verified-by: crates/core/tests/genetic_algorithm.rs::elites_are_the_fittest_genomes -->

#### Scenario: Small tournaments let weaker Genomes breed

- **WHEN** 6 Genomes breed with elitism 0, tournament size 1 and no mutation
- **THEN** not all children are equal

<!-- verified-by: crates/core/tests/genetic_algorithm.rs::small_tournaments_let_weaker_genomes_breed -->

#### Scenario: Uniform crossover

- **WHEN** 6 Genomes breed with elitism 0, tournament size 1 and no mutation
- **THEN** every parameter of every child equals that parameter in some parent
- **AND** at least one child equals no parent

<!-- verified-by: crates/core/tests/genetic_algorithm.rs::uniform_crossover_takes_each_parameter_from_a_parent -->

#### Scenario: Gaussian mutation has the configured sigma

- **WHEN** a lone Genome breeds with mutation rate 1 and sigma 0.5
- **THEN** every parameter changes, the changes have mean within ±0.1 and standard deviation within [0.4, 0.6]

<!-- verified-by: crates/core/tests/genetic_algorithm.rs::gaussian_mutation_has_configured_sigma -->

#### Scenario: Mutation rate sets the share of mutated parameters

- **WHEN** a lone Genome breeds with mutation rate 0.25
- **THEN** between 18% and 32% of its parameters change

<!-- verified-by: crates/core/tests/genetic_algorithm.rs::mutation_rate_sets_the_share_of_mutated_parameters -->

### Requirement: Progress reporting

Advancing SHALL call a progress callback whenever the current Generation reaches a multiple of 10, and once on completion, reporting the Generations completed in this call, the total requested, the current Generation, and the latest metrics rows.

#### Scenario: Progress at every tenth Generation and on completion

- **WHEN** an Evolution advances 25 Generations from Generation 0
- **THEN** progress reports (completed, total, Generation) as (10, 25, 10), (20, 25, 20), (25, 25, 25), each with one latest row for the previous Generation
- **WHEN** it then advances 20 more
- **THEN** progress reports (5, 20, 30), (15, 20, 40), (20, 20, 45)

<!-- verified-by: crates/core/tests/evolution.rs::progress_fires_at_every_tenth_generation_and_on_completion -->

### Requirement: Metrics history

Evolution SHALL keep a metrics history with one row per Species per evaluated Generation, holding best Fitness, mean Fitness, Agents alive at Episode end, food eaten, and the Episode count and step limit used.

#### Scenario: One row per Species per Generation

- **WHEN** an Evolution advances 4 Generations
- **THEN** its metrics rows are for Generations 0, 1, 2, 3, all for MLP-Prey

<!-- verified-by: crates/core/tests/evolution.rs::metrics_history_has_one_row_per_species_per_generation -->

### Requirement: Evolution is deterministic

The same seed and config SHALL give identical Genomes and metrics; a different seed SHALL give different Genomes.

#### Scenario: Same seed, same result

- **WHEN** two Evolutions with seed 77 and the same config each advance 4 Generations, and a third with seed 78 does the same
- **THEN** the first two have equal Genomes and metrics, and the third's Genomes differ

<!-- verified-by: crates/core/tests/snapshot.rs::same_seed_and_config_give_identical_genomes_and_metrics -->
