//! Genetic operators and population evolution utilities.

pub mod genome;
pub mod operators;
pub mod population;

pub use genome::{AgentGenome, TraitGenome};
pub use population::{
    ScoredGenome, default_mixed_factory, evolve_population, initialize_population,
};
