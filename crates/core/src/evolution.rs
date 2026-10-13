//! Evolution: evaluates each Generation over the Episodes its Evaluation
//! Schedule prescribes, then breeds every Species separately.

use thiserror::Error;

use crate::config::{Config, ConfigError};
use crate::episode::{Episode, Layout};
use crate::ga;
use crate::genome::{Genome, NetworkShapeError, check_network_shapes, random_genomes};
use crate::metrics::MetricsRow;
use crate::rng::{Stream, derive_seed, stream_rng};
use crate::snapshot::{SNAPSHOT_FORMAT_VERSION, Snapshot};
use crate::species::Species;

#[derive(Debug, Error)]
pub enum EvolutionError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    NetworkShape(#[from] NetworkShapeError),
    #[error("{species} has {actual} genomes but the config sets its population to {expected}")]
    PopulationSize {
        species: Species,
        expected: usize,
        actual: usize,
    },
}

/// `advance` reports progress whenever the Generation reaches a multiple of
/// this, and on completion.
pub const PROGRESS_INTERVAL: u32 = 10;

/// Reported by `advance`; see `PROGRESS_INTERVAL`.
#[derive(Debug, Clone, Copy)]
pub struct Progress<'a> {
    /// Generations completed so far in this call.
    pub completed: u32,
    /// Generations this call will advance.
    pub total: u32,
    /// The Generation now current.
    pub generation: u32,
    /// The latest metrics rows, one per Species.
    pub latest: &'a [MetricsRow],
    /// The Evolution as of now, for example to save a Snapshot.
    pub evolution: &'a Evolution,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Evolution {
    config: Config,
    seed: u64,
    generation: u32,
    /// One population per Species, in `Species::ALL` order.
    populations: Vec<Vec<Genome>>,
    metrics: Vec<MetricsRow>,
}

impl Evolution {
    /// Generation 0, with randomly initialised Genomes drawn from `seed`.
    pub fn new(config: Config, seed: u64) -> Result<Self, EvolutionError> {
        config.validate()?;
        let genomes = random_genomes(&config, seed);
        Self::from_parts(config, seed, 0, genomes, Vec::new())
    }

    /// Continue from a saved Generation.
    pub fn from_snapshot(snapshot: Snapshot) -> Result<Self, EvolutionError> {
        Self::from_parts(
            snapshot.config,
            snapshot.seed,
            snapshot.generation,
            snapshot.genomes,
            snapshot.metrics,
        )
    }

    fn from_parts(
        config: Config,
        seed: u64,
        generation: u32,
        genomes: Vec<Genome>,
        metrics: Vec<MetricsRow>,
    ) -> Result<Self, EvolutionError> {
        config.validate()?;
        check_network_shapes(&config, &genomes)?;
        let populations = Species::ALL
            .iter()
            .map(|&species| {
                let members: Vec<Genome> = genomes
                    .iter()
                    .filter(|g| Species::of(g) == species)
                    .cloned()
                    .collect();
                let expected = species.population(&config);
                if members.len() != expected {
                    return Err(EvolutionError::PopulationSize {
                        species,
                        expected,
                        actual: members.len(),
                    });
                }
                Ok(members)
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            config,
            seed,
            generation,
            populations,
            metrics,
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// The current Generation: the one whose Genomes are held, not yet evaluated.
    pub fn generation(&self) -> u32 {
        self.generation
    }

    pub fn genomes(&self, species: Species) -> &[Genome] {
        let index = species_index(species);
        &self.populations[index]
    }

    /// One row per Species per evaluated Generation, oldest first.
    pub fn metrics(&self) -> &[MetricsRow] {
        &self.metrics
    }

    /// The current Generation, ready to save or to continue from.
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            format_version: SNAPSHOT_FORMAT_VERSION,
            seed: self.seed,
            generation: self.generation,
            config: self.config.clone(),
            genomes: self.populations.concat(),
            metrics: self.metrics.clone(),
        }
    }

    /// Evaluate and breed `generations` Generations, calling `on_progress`
    /// whenever the Generation reaches a multiple of `PROGRESS_INTERVAL`, and
    /// on completion.
    pub fn advance(&mut self, generations: u32, mut on_progress: impl FnMut(&Progress)) {
        for completed in 1..=generations {
            self.run_generation();
            if self.generation.is_multiple_of(PROGRESS_INTERVAL) || completed == generations {
                let latest = &self.metrics[self.metrics.len() - Species::ALL.len()..];
                on_progress(&Progress {
                    completed,
                    total: generations,
                    generation: self.generation,
                    latest,
                    evolution: self,
                });
            }
        }
    }

    fn run_generation(&mut self) {
        let plan = self.config.evaluation.plan(self.generation);
        let generation_seed = derive_seed(self.seed, Stream::Generation, self.generation.into());
        let genomes = self.populations.concat();
        let mut fitness = vec![0.0; genomes.len()];
        let mut alive = [0u32; Species::ALL.len()];
        let mut food_eaten = [0u32; Species::ALL.len()];

        for episode in 0..plan.episodes {
            let episode_seed = derive_seed(generation_seed, Stream::Episode, episode.into());
            let mut episode = Episode::new(
                &self.config,
                episode_seed,
                genomes.clone(),
                plan.episode_steps,
                Layout::default(),
            );
            episode.run_to_end();
            for agent in episode.arenas().iter().flat_map(|arena| arena.agents()) {
                fitness[agent.id.0 as usize] +=
                    self.config.fitness.weights(agent.role).score(agent);
                let species = species_index(Species {
                    family: agent.family,
                    role: agent.role,
                });
                alive[species] += u32::from(agent.alive);
                food_eaten[species] += agent.food_eaten;
            }
        }

        let episodes = f64::from(plan.episodes);
        for value in &mut fitness {
            *value /= episodes;
        }
        let mut offset = 0;
        for (index, &species) in Species::ALL.iter().enumerate() {
            let population = &self.populations[index];
            let scores = &fitness[offset..offset + population.len()];
            offset += population.len();
            self.metrics.push(MetricsRow {
                generation: self.generation,
                species,
                episodes: plan.episodes,
                episode_steps: plan.episode_steps,
                best_fitness: scores.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                mean_fitness: scores.iter().sum::<f64>() / scores.len() as f64,
                alive_at_end: f64::from(alive[index]) / episodes,
                food_eaten: f64::from(food_eaten[index]) / episodes,
            });
            if !population.is_empty() {
                let mut rng = stream_rng(generation_seed, Stream::Breeding, index as u64);
                self.populations[index] = ga::breed(population, scores, &self.config.ga, &mut rng);
            }
        }
        self.generation += 1;
    }
}

fn species_index(species: Species) -> usize {
    Species::ALL
        .iter()
        .position(|&s| s == species)
        .expect("Species::ALL lists every Species")
}
