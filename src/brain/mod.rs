//! Neural controller interfaces and implementations.

use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use crate::sim::{ActionVector, SensorVector};

pub mod mlp;

pub use mlp::{MlpController, MlpControllerFactory, MlpGenome, MlpGenomeError, MlpShape};

/// Supported controller families.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ControllerKind {
    Mlp,
    Ltc,
}

/// Serialized representation of controller parameters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BrainGenome {
    Mlp(MlpGenome),
}

/// Runtime controller behavior shared across controller families.
pub trait NeuralController {
    /// Reset controller internal state to episode-initial values.
    fn reset(&mut self);

    /// Compute one action from sensor inputs for a fixed simulation step.
    fn step(&mut self, input: &SensorVector, dt: f32) -> ActionVector;

    /// Number of scalar parameters used by this controller instance.
    fn parameter_count(&self) -> usize;

    /// Controller family identifier.
    fn controller_kind(&self) -> ControllerKind;
}

/// Factory trait for generating and instantiating controller genomes.
pub trait ControllerFactory {
    /// Create a new random genome using the provided deterministic RNG.
    fn random_genome(&self, rng: &mut StdRng) -> BrainGenome;

    /// Build a runtime controller from a serialized genome.
    fn build(&self, genome: &BrainGenome) -> Box<dyn NeuralController>;
}
