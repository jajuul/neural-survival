//! A Showcase: an unscored Episode of the population that a caller steps and
//! observes.

use thiserror::Error;

use crate::config::{Config, ConfigError};
use crate::episode::{Episode, Layout};
use crate::genome::{Genome, NetworkShapeError, check_network_shapes, random_genomes};
use crate::geometry::Vec2;
use crate::rng::{Stream, derive_seed};
use crate::snapshot::Snapshot;
use crate::telemetry::AgentTelemetry;
use crate::world::{AgentId, Arena, Placement};

#[derive(Debug, Error)]
pub enum ShowcaseError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error("got {placements} placements for {genomes} genomes")]
    PlacementCount { genomes: usize, placements: usize },
    #[error(transparent)]
    NetworkShape(#[from] NetworkShapeError),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Showcase {
    episode: Episode,
}

/// Builds a Showcase; anything not given is drawn from the seed.
pub struct ShowcaseBuilder<'a> {
    config: &'a Config,
    seed: u64,
    genomes: Option<Vec<Genome>>,
    layout: Layout,
    step_limit: Option<u32>,
}

impl<'a> ShowcaseBuilder<'a> {
    /// Use these Genomes instead of randomly initialised ones.
    pub fn genomes(mut self, genomes: Vec<Genome>) -> Self {
        self.genomes = Some(genomes);
        self
    }

    /// Start Agents here instead of at random, one per Genome in order.
    pub fn placements(mut self, placements: Vec<Placement>) -> Self {
        self.layout.placements = Some(placements);
        self
    }

    /// Start with food here instead of `food.count` random items.
    pub fn food(mut self, positions: Vec<Vec2>) -> Self {
        self.layout.food = Some(positions);
        self
    }

    pub fn build(self) -> Result<Showcase, ShowcaseError> {
        let config = self.config;
        config.validate()?;
        let genomes = match self.genomes {
            Some(genomes) => genomes,
            None => random_genomes(config, self.seed),
        };
        check_network_shapes(config, &genomes)?;
        if let Some(placements) = &self.layout.placements
            && placements.len() != genomes.len()
        {
            return Err(ShowcaseError::PlacementCount {
                genomes: genomes.len(),
                placements: placements.len(),
            });
        }
        let step_limit = self
            .step_limit
            .unwrap_or(config.evaluation.plan(0).episode_steps);
        Ok(Showcase {
            episode: Episode::new(config, self.seed, genomes, step_limit, self.layout),
        })
    }
}

impl Showcase {
    /// A Showcase of randomly initialised Genomes in a random layout.
    pub fn new(config: &Config, seed: u64) -> Result<Self, ShowcaseError> {
        Self::builder(config, seed).build()
    }

    /// A Showcase of a Snapshot's Genomes, seeded from its run seed and
    /// Generation and as long as that Generation's Episodes.
    pub fn from_snapshot(snapshot: &Snapshot) -> Result<Self, ShowcaseError> {
        let seed = derive_seed(snapshot.seed, Stream::Showcase, snapshot.generation.into());
        let mut builder = Self::builder(&snapshot.config, seed).genomes(snapshot.genomes.clone());
        builder.step_limit = Some(
            snapshot
                .config
                .evaluation
                .plan(snapshot.generation)
                .episode_steps,
        );
        builder.build()
    }

    pub fn builder(config: &Config, seed: u64) -> ShowcaseBuilder<'_> {
        ShowcaseBuilder {
            config,
            seed,
            genomes: None,
            layout: Layout::default(),
            step_limit: None,
        }
    }

    /// Advance one simulation step; does nothing once the Episode has ended.
    pub fn step(&mut self) {
        self.episode.step();
    }

    /// Advance up to `steps` steps, stopping early if the Episode ends.
    pub fn step_many(&mut self, steps: u32) {
        for _ in 0..steps {
            self.step();
        }
    }

    /// The Episode ends at its step limit or when every Agent is dead.
    pub fn has_ended(&self) -> bool {
        self.episode.has_ended()
    }

    pub fn step_count(&self) -> u32 {
        self.episode.step_count()
    }

    pub fn arenas(&self) -> &[Arena] {
        self.episode.arenas()
    }

    /// The config this Showcase was built with.
    pub fn config(&self) -> &Config {
        self.episode.config()
    }

    pub fn genome(&self, id: AgentId) -> Option<&Genome> {
        self.episode.genome(id)
    }

    /// The Agent's telemetry from its last living step: `None` before the
    /// first step or for an unknown id.
    pub fn telemetry(&self, id: AgentId) -> Option<&AgentTelemetry> {
        self.episode.telemetry(id)
    }
}
