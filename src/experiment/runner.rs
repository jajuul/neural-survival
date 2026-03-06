use rand::{SeedableRng, rngs::StdRng};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::brain::{ControllerFactory, ControllerKind};
use crate::config::SimulationConfig;
use crate::fitness::{FitnessComponents, compute_fitness};
use crate::genetics::{
    AgentGenome, ScoredGenome, default_mixed_factory, evolve_population, initialize_population,
};
use crate::io::ensure_output_dir;
use crate::io::metrics::write_metrics_csv;
use crate::io::plots::write_fitness_svg;
use crate::io::snapshot::{SnapshotFile, write_snapshot};
use crate::io::summary::{RunSummary, config_hash, write_run_summary};
use crate::sim::{basic_sensor_vector, initialize_world, step_world_with_actions};

/// Per-generation aggregate metrics used for experiment reporting.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenerationMetrics {
    pub generation_index: u32,
    pub best_fitness: f32,
    pub mean_fitness: f32,
    pub survival_min: u32,
    pub survival_mean: f32,
    pub survival_max: u32,
    pub population_size: usize,
    pub controller_mlp: usize,
    pub controller_ltc: usize,
}

/// In-memory result of a deterministic headless evolution run.
#[derive(Clone, Debug, PartialEq)]
pub struct EvolutionRun {
    pub generations: Vec<GenerationMetrics>,
    pub final_population: Vec<AgentGenome>,
}

fn run_generations_internal<F>(
    config: &SimulationConfig,
    seed: u64,
    generations: u32,
    mut on_generation_complete: F,
) -> EvolutionRun
where
    F: FnMut(u32, u32),
{
    let factory = default_mixed_factory(config);
    let mut init_rng = StdRng::seed_from_u64(seed);

    let mut population = initialize_population(config, &mut init_rng, &factory);
    let mut metrics = Vec::with_capacity(generations as usize);

    for generation_index in 0..generations {
        let evaluated = evaluate_population(config, seed, generation_index, &population, &factory);
        let scored = to_scored_genomes(&evaluated);
        metrics.push(build_generation_metrics(
            generation_index,
            &population,
            &evaluated,
            &scored,
        ));
        on_generation_complete(generation_index + 1, generations);

        if generation_index + 1 < generations {
            let mut evolution_rng = evolution_rng_for_generation(seed, generation_index);
            population = evolve_population(&scored, config, &mut evolution_rng);
        }
    }

    EvolutionRun {
        generations: metrics,
        final_population: population,
    }
}

/// Run deterministic headless evolution for `generations` rounds.
///
/// Inputs: validated simulation config, deterministic seed, and generation count.
/// Output: per-generation metrics and final generation genomes.
/// Side effects: none.
pub fn run_generations(config: &SimulationConfig, seed: u64, generations: u32) -> EvolutionRun {
    run_generations_internal(config, seed, generations, |_completed, _total| {})
}

/// Run deterministic headless evolution and report per-generation progress.
///
/// Inputs: config, seed, generation count, and progress callback.
/// Output: per-generation metrics and final generation genomes.
/// Side effects: invokes progress callback once per generation.
pub fn run_generations_with_progress<F>(
    config: &SimulationConfig,
    seed: u64,
    generations: u32,
    on_generation_complete: F,
) -> EvolutionRun
where
    F: FnMut(u32, u32),
{
    run_generations_internal(config, seed, generations, on_generation_complete)
}

/// Continue evolving from an existing generation for additional generation steps.
///
/// Inputs: config, run seed, generation index represented by `population`, initial population,
/// and number of additional generations to advance.
/// Output: metrics for each evolved generation and the newly evolved final population.
/// Side effects: none.
pub fn continue_generations_from_population(
    config: &SimulationConfig,
    seed: u64,
    start_generation_index: u32,
    population: &[AgentGenome],
    generations_to_advance: u32,
) -> EvolutionRun {
    let factory = default_mixed_factory(config);
    let mut population = population.to_vec();
    let mut metrics = Vec::with_capacity(generations_to_advance as usize);

    for offset in 0..generations_to_advance {
        let generation_index = start_generation_index + offset;
        let evaluated = evaluate_population(config, seed, generation_index, &population, &factory);
        let scored = to_scored_genomes(&evaluated);
        metrics.push(build_generation_metrics(
            generation_index,
            &population,
            &evaluated,
            &scored,
        ));

        let mut evolution_rng = evolution_rng_for_generation(seed, generation_index);
        population = evolve_population(&scored, config, &mut evolution_rng);
    }

    EvolutionRun {
        generations: metrics,
        final_population: population,
    }
}

/// Run a deterministic batch evolution and persist required artifacts.
///
/// Inputs: config, seed, generation count, and output directory.
/// Output: in-memory run result matching persisted artifacts.
/// Side effects: writes metrics, summary, and snapshot files.
pub fn run_batch_to_dir(
    config: &SimulationConfig,
    seed: u64,
    generations: u32,
    out_dir: &Path,
) -> anyhow::Result<EvolutionRun> {
    run_batch_to_dir_with_progress(config, seed, generations, out_dir, |_completed, _total| {})
}

/// Run a deterministic batch evolution, persist artifacts, and report per-generation progress.
///
/// Inputs: config, seed, generation count, output directory, and progress callback.
/// Output: in-memory run result matching persisted artifacts.
/// Side effects: writes metrics/summary/snapshot files and invokes progress callback.
pub fn run_batch_to_dir_with_progress<F>(
    config: &SimulationConfig,
    seed: u64,
    generations: u32,
    out_dir: &Path,
    on_generation_complete: F,
) -> anyhow::Result<EvolutionRun>
where
    F: FnMut(u32, u32),
{
    ensure_output_dir(out_dir)?;

    let run = run_generations_with_progress(config, seed, generations, on_generation_complete);
    let cfg_hash = config_hash(config)?;

    write_metrics_csv(out_dir, &run.generations, &cfg_hash)?;
    write_fitness_svg(out_dir, &run.generations)?;
    write_run_summary(
        out_dir,
        &RunSummary {
            seed,
            generations,
            config_hash: cfg_hash,
            resolved_config: config.clone(),
            fitness_weight_survival: config.fitness_weight_survival,
            fitness_weight_food: config.fitness_weight_food,
            fitness_weight_energy: config.fitness_weight_energy,
            runner: "neural-survival".into(),
        },
    )?;

    let snapshot = SnapshotFile {
        generation: generations,
        seed,
        resolved_config: config.clone(),
        metrics_history: run.generations.clone(),
        population: run.final_population.clone(),
        world_state: world_state_for_population(config, seed, generations, &run.final_population),
    };
    write_snapshot(out_dir, generations, &snapshot)?;

    Ok(run)
}

fn evaluate_population(
    config: &SimulationConfig,
    run_seed: u64,
    generation_index: u32,
    population: &[AgentGenome],
    factory: &impl ControllerFactory,
) -> Vec<EvaluatedGenome> {
    population
        .iter()
        .enumerate()
        .map(|(agent_index, genome)| {
            let eval_seed = per_agent_eval_seed(run_seed, generation_index, agent_index);
            let components = evaluate_genome(config, eval_seed, genome, factory);
            let fitness = compute_fitness(components, config);

            EvaluatedGenome {
                genome: genome.clone(),
                fitness,
                components,
            }
        })
        .collect()
}

fn evaluate_genome(
    config: &SimulationConfig,
    eval_seed: u64,
    genome: &AgentGenome,
    factory: &impl ControllerFactory,
) -> FitnessComponents {
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
    FitnessComponents {
        survival_steps,
        food_consumed: agent.food_consumed,
        final_energy: agent.energy,
    }
}

fn per_agent_eval_seed(run_seed: u64, generation_index: u32, agent_index: usize) -> u64 {
    run_seed
        ^ ((generation_index as u64 + 1).wrapping_mul(0x9E37_79B9))
        ^ ((agent_index as u64 + 1).wrapping_mul(0x517C_C1B7_2722_0A95))
}

/// Build deterministic world state for a specific generation/population pair.
///
/// Inputs: config, run seed, generation index, and population genomes.
/// Output: initialized world with trait-projected agent parameters.
/// Side effects: none.
pub fn world_state_for_population(
    config: &SimulationConfig,
    seed: u64,
    generation_index: u32,
    population: &[AgentGenome],
) -> crate::sim::WorldState {
    let mut world = initialize_world(config, seed ^ generation_index as u64);
    world.agents.truncate(population.len());

    for (agent, genome) in world.agents.iter_mut().zip(population.iter()) {
        agent.max_speed = genome.traits.max_speed;
        agent.perception_radius = genome.traits.perception_radius;
        agent.perception_channels = genome.traits.perception_channels;
        agent.metabolic_efficiency = genome.traits.metabolic_efficiency;
    }

    world
}

fn to_scored_genomes(evaluated: &[EvaluatedGenome]) -> Vec<ScoredGenome> {
    evaluated
        .iter()
        .map(|entry| ScoredGenome {
            genome: entry.genome.clone(),
            fitness: entry.fitness,
        })
        .collect::<Vec<_>>()
}

fn build_generation_metrics(
    generation_index: u32,
    population: &[AgentGenome],
    evaluated: &[EvaluatedGenome],
    scored: &[ScoredGenome],
) -> GenerationMetrics {
    let best_fitness = scored
        .iter()
        .map(|entry| entry.fitness)
        .fold(f32::NEG_INFINITY, f32::max);
    let mean_fitness = scored.iter().map(|entry| entry.fitness).sum::<f32>() / scored.len() as f32;
    let survival_min = evaluated
        .iter()
        .map(|entry| entry.components.survival_steps)
        .min()
        .unwrap_or(0);
    let survival_max = evaluated
        .iter()
        .map(|entry| entry.components.survival_steps)
        .max()
        .unwrap_or(0);
    let survival_mean = evaluated
        .iter()
        .map(|entry| entry.components.survival_steps as f32)
        .sum::<f32>()
        / evaluated.len() as f32;
    let controller_mlp = evaluated
        .iter()
        .filter(|entry| entry.genome.controller_kind == ControllerKind::Mlp)
        .count();
    let controller_ltc = evaluated
        .iter()
        .filter(|entry| entry.genome.controller_kind == ControllerKind::Ltc)
        .count();

    GenerationMetrics {
        generation_index,
        best_fitness,
        mean_fitness,
        survival_min,
        survival_mean,
        survival_max,
        population_size: population.len(),
        controller_mlp,
        controller_ltc,
    }
}

fn evolution_rng_for_generation(run_seed: u64, generation_index: u32) -> StdRng {
    StdRng::seed_from_u64(
        run_seed
            ^ 0x9E37_79B9_7F4A_7C15
            ^ ((generation_index as u64 + 1).wrapping_mul(0xD6E8_FEB8_6659_FD93)),
    )
}

#[derive(Clone, Debug, PartialEq)]
struct EvaluatedGenome {
    genome: AgentGenome,
    fitness: f32,
    components: FitnessComponents,
}

#[cfg(test)]
mod tests {
    use crate::config::SimulationConfig;

    use super::{continue_generations_from_population, run_generations};

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

    #[test]
    fn continue_generations_from_population_matches_single_full_run() {
        let config = SimulationConfig {
            population_size: 10,
            food_count: 24,
            episode_steps: 30,
            ..SimulationConfig::default()
        };

        let seed = 77;
        let full_run = run_generations(&config, seed, 5);
        let split_prefix = run_generations(&config, seed, 2);
        let continued = continue_generations_from_population(
            &config,
            seed,
            1,
            &split_prefix.final_population,
            3,
        );

        assert_eq!(continued.final_population, full_run.final_population);
        assert_eq!(continued.generations.len(), 3);
    }
}
