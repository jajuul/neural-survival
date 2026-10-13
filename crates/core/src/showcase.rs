//! A Showcase: an unscored Episode of the population that a caller steps and
//! observes.

use rand::Rng as _;
use std::f64::consts::TAU;
use thiserror::Error;

use crate::config::{Config, ConfigError};
use crate::genome::{CONTROLLER_OUTPUTS, ControllerFamily, Genome, Role};
use crate::geometry::{Torus, Vec2};
use crate::rng::{Stream, stream_rng};
use crate::telemetry::AgentTelemetry;
use crate::world::{AgentId, Arena, Food, Placement};

#[derive(Debug, Error)]
pub enum ShowcaseError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error("got {placements} placements for {genomes} genomes")]
    PlacementCount { genomes: usize, placements: usize },
    #[error(
        "genome {index} has a {inputs}-input, {outputs}-output network; \
         the config needs {expected_inputs} inputs and {CONTROLLER_OUTPUTS} outputs"
    )]
    NetworkShape {
        index: usize,
        inputs: usize,
        outputs: usize,
        expected_inputs: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Showcase {
    config: Config,
    /// Indexed by `AgentId`.
    genomes: Vec<Genome>,
    step_count: u32,
    arenas: Vec<Arena>,
}

/// Builds a Showcase; anything not given is drawn from the seed.
pub struct ShowcaseBuilder<'a> {
    config: &'a Config,
    seed: u64,
    genomes: Option<Vec<Genome>>,
    placements: Option<Vec<Placement>>,
    food: Option<Vec<Vec2>>,
}

impl<'a> ShowcaseBuilder<'a> {
    /// Use these Genomes instead of randomly initialised ones.
    pub fn genomes(mut self, genomes: Vec<Genome>) -> Self {
        self.genomes = Some(genomes);
        self
    }

    /// Start Agents here instead of at random, one per Genome in order.
    pub fn placements(mut self, placements: Vec<Placement>) -> Self {
        self.placements = Some(placements);
        self
    }

    /// Start with food here instead of `food.count` random items.
    pub fn food(mut self, positions: Vec<Vec2>) -> Self {
        self.food = Some(positions);
        self
    }

    pub fn build(self) -> Result<Showcase, ShowcaseError> {
        let config = self.config;
        config.validate()?;
        let genomes = match self.genomes {
            Some(genomes) => genomes,
            None => {
                let mut rng = stream_rng(self.seed, Stream::Genomes, 0);
                (0..config.population.mlp_prey)
                    .map(|_| Genome::random(Role::Prey, ControllerFamily::Mlp, config, &mut rng))
                    .collect()
            }
        };
        let expected_inputs = config.controller_input_width();
        for (index, genome) in genomes.iter().enumerate() {
            let (inputs, outputs) = (genome.network.input_width(), genome.network.output_width());
            if inputs != expected_inputs || outputs != CONTROLLER_OUTPUTS {
                return Err(ShowcaseError::NetworkShape {
                    index,
                    inputs,
                    outputs,
                    expected_inputs,
                });
            }
        }

        let world = Torus::new(&config.world);
        let mut rng = stream_rng(self.seed, Stream::Arena, 0);
        let placements = match self.placements {
            Some(placements) if placements.len() != genomes.len() => {
                return Err(ShowcaseError::PlacementCount {
                    genomes: genomes.len(),
                    placements: placements.len(),
                });
            }
            Some(placements) => placements
                .into_iter()
                .map(|p| Placement {
                    position: world.wrap(p.position),
                    heading: p.heading,
                })
                .collect(),
            None => (0..genomes.len())
                .map(|_| Placement {
                    position: world.random_point(&mut rng),
                    heading: rng.gen_range(0.0..TAU),
                })
                .collect::<Vec<_>>(),
        };
        let food = match self.food {
            Some(positions) => positions.into_iter().map(|p| world.wrap(p)).collect(),
            None => (0..config.food.count)
                .map(|_| world.random_point(&mut rng))
                .collect::<Vec<_>>(),
        };
        let food = food.into_iter().map(|position| Food { position }).collect();

        let members = genomes
            .iter()
            .cloned()
            .zip(placements)
            .enumerate()
            .map(|(i, (genome, placement))| (AgentId(i as u32), genome, placement))
            .collect();
        Ok(Showcase {
            config: config.clone(),
            genomes,
            step_count: 0,
            arenas: vec![Arena::new(config, members, food, rng)],
        })
    }
}

impl Showcase {
    /// A Showcase of randomly initialised Genomes in a random layout.
    pub fn new(config: &Config, seed: u64) -> Result<Self, ShowcaseError> {
        Self::builder(config, seed).build()
    }

    pub fn builder(config: &Config, seed: u64) -> ShowcaseBuilder<'_> {
        ShowcaseBuilder {
            config,
            seed,
            genomes: None,
            placements: None,
            food: None,
        }
    }

    /// Advance one simulation step; does nothing once the Episode has ended.
    pub fn step(&mut self) {
        if self.has_ended() {
            return;
        }
        for arena in &mut self.arenas {
            arena.step(&self.config);
        }
        self.step_count += 1;
    }

    /// Advance up to `steps` steps, stopping early if the Episode ends.
    pub fn step_many(&mut self, steps: u32) {
        for _ in 0..steps {
            self.step();
        }
    }

    /// The Episode ends at its step limit or when every Agent is dead.
    pub fn has_ended(&self) -> bool {
        self.step_count >= self.config.evaluation.episode_steps
            || self.arenas.iter().all(Arena::all_dead)
    }

    pub fn step_count(&self) -> u32 {
        self.step_count
    }

    pub fn arenas(&self) -> &[Arena] {
        &self.arenas
    }

    pub fn genome(&self, id: AgentId) -> Option<&Genome> {
        self.genomes.get(id.0 as usize)
    }

    /// The Agent's telemetry from its last living step: `None` before the
    /// first step or for an unknown id.
    pub fn telemetry(&self, id: AgentId) -> Option<&AgentTelemetry> {
        self.arenas.iter().find_map(|arena| arena.telemetry(id))
    }
}
