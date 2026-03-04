use rand::{SeedableRng, rngs::StdRng};

use crate::brain::ControllerFactory;
use crate::config::SimulationConfig;
use crate::fitness::{FitnessComponents, compute_fitness};
use crate::genetics::{
    AgentGenome, ScoredGenome, default_mixed_factory, evolve_population, initialize_population,
};
use crate::sim::{basic_sensor_vector, initialize_world, step_world_with_actions};

/// Per-generation aggregate metrics used for experiment reporting.
#[derive(Clone, Debug, PartialEq)]
pub struct GenerationMetrics {
    pub generation_index: u32,
    pub best_fitness: f32,
    pub mean_fitness: f32,
    pub population_size: usize,
}

/// In-memory result of a deterministic headless evolution run.
#[derive(Clone, Debug, PartialEq)]
pub struct EvolutionRun {
    pub generations: Vec<GenerationMetrics>,
    pub final_population: Vec<AgentGenome>,
}

/// Run deterministic headless evolution for `generations` rounds.
///
/// Inputs: validated simulation config, deterministic seed, and generation count.
/// Output: per-generation metrics and final generation genomes.
/// Side effects: none.
pub fn run_generations(config: &SimulationConfig, seed: u64, generations: u32) -> EvolutionRun {
    let factory = default_mixed_factory(config);
    let mut init_rng = StdRng::seed_from_u64(seed);
    let mut evolution_rng = StdRng::seed_from_u64(seed ^ 0x9E37_79B9_7F4A_7C15);

    let mut population = initialize_population(config, &mut init_rng, &factory);
    let mut metrics = Vec::with_capacity(generations as usize);

    for generation_index in 0..generations {
        let scored = score_population(config, seed, generation_index, &population, &factory);

        let best_fitness = scored
            .iter()
            .map(|entry| entry.fitness)
            .fold(f32::NEG_INFINITY, f32::max);
        let mean_fitness =
            scored.iter().map(|entry| entry.fitness).sum::<f32>() / scored.len() as f32;

        metrics.push(GenerationMetrics {
            generation_index,
            best_fitness,
            mean_fitness,
            population_size: population.len(),
        });

        if generation_index + 1 < generations {
            population = evolve_population(&scored, config, &mut evolution_rng);
        }
    }

    EvolutionRun {
        generations: metrics,
        final_population: population,
    }
}

fn score_population(
    config: &SimulationConfig,
    run_seed: u64,
    generation_index: u32,
    population: &[AgentGenome],
    factory: &impl ControllerFactory,
) -> Vec<ScoredGenome> {
    population
        .iter()
        .enumerate()
        .map(|(agent_index, genome)| {
            let eval_seed = per_agent_eval_seed(run_seed, generation_index, agent_index);
            let fitness = evaluate_genome(config, eval_seed, genome, factory);

            ScoredGenome {
                genome: genome.clone(),
                fitness,
            }
        })
        .collect()
}

fn evaluate_genome(
    config: &SimulationConfig,
    eval_seed: u64,
    genome: &AgentGenome,
    factory: &impl ControllerFactory,
) -> f32 {
    let mut world = initialize_world(config, eval_seed);
    world.agents.truncate(1);

    {
        let agent = &mut world.agents[0];
        agent.max_speed = genome.traits.max_speed;
        agent.perception_radius = genome.traits.perception_radius;
        agent.perception_channels = genome.traits.perception_channels;
        agent.metabolic_efficiency = genome.traits.metabolic_efficiency;
    }

    let mut controller = factory.build(&genome.brain);
    controller.reset();

    let mut survival_steps = 0;
    for _ in 0..config.episode_steps {
        if !world.agents[0].alive {
            break;
        }

        let sensor = basic_sensor_vector(&world.agents[0], &world, config);
        let action = controller.step(&sensor, config.timestep_seconds);
        step_world_with_actions(&mut world, config, &[action]);

        if world.agents[0].alive {
            survival_steps += 1;
        }
    }

    let agent = &world.agents[0];
    compute_fitness(
        FitnessComponents {
            survival_steps,
            food_consumed: agent.food_consumed,
            final_energy: agent.energy,
        },
        config,
    )
}

fn per_agent_eval_seed(run_seed: u64, generation_index: u32, agent_index: usize) -> u64 {
    run_seed
        ^ ((generation_index as u64 + 1).wrapping_mul(0x9E37_79B9))
        ^ ((agent_index as u64 + 1).wrapping_mul(0x517C_C1B7_2722_0A95))
}

#[cfg(test)]
mod tests {
    use crate::config::SimulationConfig;

    use super::run_generations;

    #[test]
    fn run_generations_is_deterministic_for_same_seed() {
        let config = SimulationConfig {
            population_size: 12,
            food_count: 32,
            episode_steps: 40,
            ..SimulationConfig::default()
        };

        let run_a = run_generations(&config, 42, 4);
        let run_b = run_generations(&config, 42, 4);

        assert_eq!(run_a, run_b);
    }

    #[test]
    fn run_generations_preserves_population_size_every_generation() {
        let config = SimulationConfig {
            population_size: 14,
            food_count: 30,
            episode_steps: 30,
            ..SimulationConfig::default()
        };

        let run = run_generations(&config, 9, 5);

        assert_eq!(run.generations.len(), 5);
        assert!(
            run.generations
                .iter()
                .all(|entry| entry.population_size == config.population_size)
        );
        assert_eq!(run.final_population.len(), config.population_size);
    }
}
