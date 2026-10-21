//! An Episode: one simulated run of the world, stepped to its end. Evolution
//! scores Episodes; a Showcase plays one unscored.

use rand::Rng as _;
use std::f64::consts::TAU;

use crate::config::Config;
use crate::genome::Genome;
use crate::geometry::{Torus, Vec2};
use crate::rng::{Stream, stream_rng};
use crate::telemetry::AgentTelemetry;
use crate::world::{AgentId, Arena, Food, Placement};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Episode {
    config: Config,
    /// Indexed by `AgentId`.
    genomes: Vec<Genome>,
    step_count: u32,
    step_limit: u32,
    arenas: Vec<Arena>,
}

/// Where Agents and food start; anything not given is drawn from the seed.
#[derive(Debug, Default)]
pub(crate) struct Layout {
    /// One per Genome, in order.
    pub placements: Option<Vec<Placement>>,
    pub food: Option<Vec<Vec2>>,
}

impl Episode {
    /// Expects a valid config, Genomes whose networks fit it, and one
    /// placement per Genome when placements are given.
    pub fn new(
        config: &Config,
        seed: u64,
        genomes: Vec<Genome>,
        step_limit: u32,
        layout: Layout,
    ) -> Self {
        let world = Torus::new(&config.world);
        let mut rng = stream_rng(seed, Stream::Arena, 0);
        let placements: Vec<Placement> = match layout.placements {
            Some(placements) => {
                debug_assert_eq!(placements.len(), genomes.len());
                placements
                    .into_iter()
                    .map(|p| Placement {
                        position: world.wrap(p.position),
                        heading: p.heading,
                    })
                    .collect()
            }
            None => (0..genomes.len())
                .map(|_| Placement {
                    position: world.random_point(&mut rng),
                    heading: rng.gen_range(0.0..TAU),
                })
                .collect(),
        };
        let food: Vec<Vec2> = match layout.food {
            Some(positions) => positions.into_iter().map(|p| world.wrap(p)).collect(),
            None => (0..config.food.count)
                .map(|_| world.random_point(&mut rng))
                .collect(),
        };
        let food = food.into_iter().map(|position| Food { position }).collect();

        // Agent ids index `genomes` across every Arena of the Episode.
        let members = genomes
            .iter()
            .cloned()
            .zip(placements)
            .enumerate()
            .map(|(i, (genome, placement))| (AgentId(i as u32), genome, placement))
            .collect();
        Self {
            config: config.clone(),
            genomes,
            step_count: 0,
            step_limit,
            arenas: vec![Arena::new(config, members, food, rng)],
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

    pub fn run_to_end(&mut self) {
        while !self.has_ended() {
            self.step();
        }
    }

    /// The Episode ends at its step limit or when every Agent is dead.
    pub fn has_ended(&self) -> bool {
        self.step_count >= self.step_limit || self.arenas.iter().all(Arena::all_dead)
    }

    pub fn step_count(&self) -> u32 {
        self.step_count
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn arenas(&self) -> &[Arena] {
        &self.arenas
    }

    pub fn genome(&self, id: AgentId) -> Option<&Genome> {
        self.genomes.get(id.0 as usize)
    }

    pub fn telemetry(&self, id: AgentId) -> Option<&AgentTelemetry> {
        self.arenas.iter().find_map(|arena| arena.telemetry(id))
    }
}
