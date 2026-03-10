use rand::{Rng, rngs::StdRng};

use crate::brain::{
    BrainGenome, ControllerFactory, ControllerKind, LtcControllerFactory, LtcShape,
    MixedControllerFactory, MlpControllerFactory, MlpShape,
};
use crate::config::SimulationConfig;
use crate::sim::{AgentRole, CONTROLLER_SENSOR_INPUT_SIZE};

use super::genome::AgentGenome;
use super::operators::{crossover, default_traits, mutate};

/// Genome with computed fitness for one generation.
#[derive(Clone, Debug, PartialEq)]
pub struct ScoredGenome {
    pub genome: AgentGenome,
    pub fitness: f32,
}

/// Build a mixed-family factory from config defaults.
pub fn default_mixed_factory(config: &SimulationConfig) -> MixedControllerFactory {
    MixedControllerFactory::new(
        MlpControllerFactory::new(MlpShape {
            input_size: CONTROLLER_SENSOR_INPUT_SIZE,
            hidden_size: config.mlp_hidden_size,
            output_size: 2,
        }),
        LtcControllerFactory::new(LtcShape {
            input_size: CONTROLLER_SENSOR_INPUT_SIZE,
            state_size: config.ltc_state_size,
            output_size: 2,
        }),
        config.controller_ltc_probability,
    )
}

/// Generate initial population genomes from deterministic RNG.
///
/// Inputs: config, RNG, and controller factory.
/// Output: a population of `config.population_size` genomes.
/// Side effects: none.
pub fn initialize_population(
    config: &SimulationConfig,
    rng: &mut StdRng,
    factory: &impl ControllerFactory,
) -> Vec<AgentGenome> {
    let traits = default_traits(config);

    (0..config.population_size)
        .map(|_| {
            let mut traits = traits.clone();
            traits.role = if rng.gen_bool(config.predator_fraction as f64) {
                AgentRole::Predator
            } else {
                AgentRole::Prey
            };
            let brain = factory.random_genome(rng);
            let controller_kind = match &brain {
                BrainGenome::Mlp(_) => ControllerKind::Mlp,
                BrainGenome::Ltc(_) => ControllerKind::Ltc,
            };

            AgentGenome {
                controller_kind,
                brain,
                traits,
            }
        })
        .collect()
}

/// Build next generation using elitism + tournament selection + crossover + mutation.
///
/// Inputs: scored current generation, config, and deterministic RNG.
/// Output: new generation genomes with preserved population size.
/// Side effects: none.
pub fn evolve_population(
    scored: &[ScoredGenome],
    config: &SimulationConfig,
    rng: &mut StdRng,
) -> Vec<AgentGenome> {
    assert!(
        !scored.is_empty(),
        "evolve_population requires non-empty scored set"
    );

    let mlp_scored = scored
        .iter()
        .filter(|entry| {
            entry.genome.controller_kind == ControllerKind::Mlp
                && entry.genome.traits.role == AgentRole::Prey
        })
        .cloned()
        .collect::<Vec<_>>();
    let mlp_predator_scored = scored
        .iter()
        .filter(|entry| {
            entry.genome.controller_kind == ControllerKind::Mlp
                && entry.genome.traits.role == AgentRole::Predator
        })
        .cloned()
        .collect::<Vec<_>>();
    let ltc_scored = scored
        .iter()
        .filter(|entry| {
            entry.genome.controller_kind == ControllerKind::Ltc
                && entry.genome.traits.role == AgentRole::Prey
        })
        .cloned()
        .collect::<Vec<_>>();
    let ltc_predator_scored = scored
        .iter()
        .filter(|entry| {
            entry.genome.controller_kind == ControllerKind::Ltc
                && entry.genome.traits.role == AgentRole::Predator
        })
        .cloned()
        .collect::<Vec<_>>();

    let mut next = Vec::with_capacity(config.population_size);
    next.extend(evolve_species_group(
        &mlp_scored,
        mlp_scored.len(),
        config,
        rng,
    ));
    next.extend(evolve_species_group(
        &mlp_predator_scored,
        mlp_predator_scored.len(),
        config,
        rng,
    ));
    next.extend(evolve_species_group(
        &ltc_scored,
        ltc_scored.len(),
        config,
        rng,
    ));
    next.extend(evolve_species_group(
        &ltc_predator_scored,
        ltc_predator_scored.len(),
        config,
        rng,
    ));

    debug_assert_eq!(next.len(), config.population_size);
    next
}

fn evolve_species_group(
    scored: &[ScoredGenome],
    target_count: usize,
    config: &SimulationConfig,
    rng: &mut StdRng,
) -> Vec<AgentGenome> {
    if target_count == 0 {
        return Vec::new();
    }

    let mut ranked = scored.to_vec();
    ranked.sort_by(|a, b| b.fitness.total_cmp(&a.fitness));

    let mut elite_count =
        ((target_count as f32 * config.elitism_fraction).round() as usize).clamp(1, target_count);
    elite_count = elite_count.min(ranked.len());

    let mut next = ranked
        .iter()
        .take(elite_count)
        .map(|entry| entry.genome.clone())
        .collect::<Vec<_>>();

    let tournament_size = config.tournament_size.min(ranked.len()).max(1);
    while next.len() < target_count {
        let parent_a = tournament_select(&ranked, tournament_size, rng);
        let parent_b = tournament_select(&ranked, tournament_size, rng);

        let mut child = crossover(parent_a, parent_b, rng);
        mutate(&mut child, rng, config);
        next.push(child);
    }

    next
}

fn tournament_select<'a>(
    ranked: &'a [ScoredGenome],
    size: usize,
    rng: &mut StdRng,
) -> &'a AgentGenome {
    let mut best_idx = rng.gen_range(0..ranked.len());
    for _ in 1..size {
        let idx = rng.gen_range(0..ranked.len());
        if ranked[idx].fitness > ranked[best_idx].fitness {
            best_idx = idx;
        }
    }

    &ranked[best_idx].genome
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};

    use crate::config::SimulationConfig;

    use super::{ScoredGenome, default_mixed_factory, evolve_population, initialize_population};

    #[test]
    fn initialize_population_uses_config_population_size() {
        let config = SimulationConfig {
            population_size: 10,
            ..SimulationConfig::default()
        };
        let factory = default_mixed_factory(&config);
        let mut rng = StdRng::seed_from_u64(1);

        let population = initialize_population(&config, &mut rng, &factory);

        assert_eq!(population.len(), 10);
    }

    #[test]
    fn evolve_population_preserves_population_size() {
        let config = SimulationConfig {
            population_size: 12,
            tournament_size: 4,
            ..SimulationConfig::default()
        };
        let factory = default_mixed_factory(&config);
        let mut init_rng = StdRng::seed_from_u64(2);
        let population = initialize_population(&config, &mut init_rng, &factory);

        let scored = population
            .into_iter()
            .enumerate()
            .map(|(idx, genome)| ScoredGenome {
                genome,
                fitness: idx as f32,
            })
            .collect::<Vec<_>>();

        let mut evo_rng = StdRng::seed_from_u64(3);
        let next = evolve_population(&scored, &config, &mut evo_rng);

        assert_eq!(next.len(), config.population_size);
    }

    #[test]
    fn evolve_population_preserves_species_counts() {
        let config = SimulationConfig {
            population_size: 24,
            tournament_size: 4,
            ..SimulationConfig::default()
        };
        let factory = default_mixed_factory(&config);
        let mut init_rng = StdRng::seed_from_u64(44);
        let population = initialize_population(&config, &mut init_rng, &factory);

        let mlp_before = population
            .iter()
            .filter(|entry| entry.controller_kind == crate::brain::ControllerKind::Mlp)
            .count();
        let ltc_before = population
            .iter()
            .filter(|entry| entry.controller_kind == crate::brain::ControllerKind::Ltc)
            .count();
        let prey_before = population
            .iter()
            .filter(|entry| entry.traits.role == crate::sim::AgentRole::Prey)
            .count();
        let predator_before = population
            .iter()
            .filter(|entry| entry.traits.role == crate::sim::AgentRole::Predator)
            .count();

        let scored = population
            .into_iter()
            .enumerate()
            .map(|(idx, genome)| ScoredGenome {
                genome,
                fitness: idx as f32,
            })
            .collect::<Vec<_>>();

        let mut evo_rng = StdRng::seed_from_u64(81);
        let next = evolve_population(&scored, &config, &mut evo_rng);

        let mlp_after = next
            .iter()
            .filter(|entry| entry.controller_kind == crate::brain::ControllerKind::Mlp)
            .count();
        let ltc_after = next
            .iter()
            .filter(|entry| entry.controller_kind == crate::brain::ControllerKind::Ltc)
            .count();
        let prey_after = next
            .iter()
            .filter(|entry| entry.traits.role == crate::sim::AgentRole::Prey)
            .count();
        let predator_after = next
            .iter()
            .filter(|entry| entry.traits.role == crate::sim::AgentRole::Predator)
            .count();

        assert_eq!(mlp_after, mlp_before);
        assert_eq!(ltc_after, ltc_before);
        assert_eq!(prey_after, prey_before);
        assert_eq!(predator_after, predator_before);
    }

    #[test]
    fn default_mixed_factory_uses_configured_layer_sizes() {
        let config = SimulationConfig {
            mlp_hidden_size: 13,
            ltc_state_size: 11,
            ..SimulationConfig::default()
        };
        let factory = default_mixed_factory(&config);

        assert_eq!(factory.mlp_factory.shape.hidden_size, 13);
        assert_eq!(factory.ltc_factory.shape.state_size, 11);
    }
}
