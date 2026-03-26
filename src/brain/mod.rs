//! Neural controller interfaces and implementations.

use rand::Rng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use crate::sim::{ActionVector, SensorVector};

pub mod ltc;
pub mod mlp;

pub use ltc::{LtcController, LtcControllerFactory, LtcGenome, LtcGenomeError, LtcShape};
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
    Ltc(LtcGenome),
}

/// Controller outputs for one fixed simulation step, including pre-remap signals.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ControllerStepOutput {
    pub action: ActionVector,
    pub steering_raw: f32,
    pub throttle_raw: f32,
    pub throttle_applied: f32,
}

/// Per-step controller diagnostics used by the GUI inspector.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ControllerDiagnostics {
    pub output: ControllerStepOutput,
    pub network: ControllerNetworkSnapshot,
}

/// Family-specific network topology and activation snapshot.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum ControllerNetworkSnapshot {
    Mlp(MlpNetworkSnapshot),
    Ltc(LtcNetworkSnapshot),
    #[default]
    Unavailable,
}

/// Feed-forward network activations and weights for one MLP step.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MlpNetworkSnapshot {
    pub input_values: Vec<f32>,
    pub hidden_values: Vec<f32>,
    pub output_values: Vec<f32>,
    pub weights_input_hidden: Vec<f32>,
    pub weights_hidden_output: Vec<f32>,
}

/// Recurrent network activations and weights for one LTC step.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LtcNetworkSnapshot {
    pub input_values: Vec<f32>,
    pub state_values: Vec<f32>,
    pub output_values: Vec<f32>,
    pub sensory_weights: Vec<f32>,
    pub recurrent_weights: Vec<f32>,
    pub readout_weights: Vec<f32>,
}

/// Runtime controller behavior shared across controller families.
pub trait NeuralController: Send + Sync {
    /// Reset controller internal state to episode-initial values.
    fn reset(&mut self);

    /// Compute one action from sensor inputs for a fixed simulation step.
    fn step(&mut self, input: &SensorVector, dt: f32) -> ActionVector;

    /// Compute one action and expose raw control signals for observability.
    fn step_with_observability(&mut self, input: &SensorVector, dt: f32) -> ControllerStepOutput {
        let action = self.step(input, dt);
        ControllerStepOutput {
            action,
            steering_raw: action.steering,
            throttle_raw: (action.throttle * 2.0 - 1.0).clamp(-1.0, 1.0),
            throttle_applied: action.throttle,
        }
    }

    /// Inspect the controller's current topology and activation state for a given input.
    fn inspect_network(&self, _input: &SensorVector) -> ControllerNetworkSnapshot {
        ControllerNetworkSnapshot::Unavailable
    }

    /// Compute one action and expose both raw signals and network activations.
    fn step_with_diagnostics(&mut self, input: &SensorVector, dt: f32) -> ControllerDiagnostics {
        let output = self.step_with_observability(input, dt);
        let network = self.inspect_network(input);
        ControllerDiagnostics { output, network }
    }

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

/// Factory that can produce and build both controller families.
#[derive(Clone, Debug)]
pub struct MixedControllerFactory {
    pub mlp_factory: MlpControllerFactory,
    pub ltc_factory: LtcControllerFactory,
    pub ltc_probability: f32,
}

/// Upgrade a serialized brain genome to a newer controller input width in place.
pub fn upgrade_brain_genome_input_size(brain: &mut BrainGenome, target_input_size: usize) {
    match brain {
        BrainGenome::Mlp(mlp) => mlp.upgrade_input_size(target_input_size),
        BrainGenome::Ltc(ltc) => ltc.upgrade_input_size(target_input_size),
    }
}

impl MixedControllerFactory {
    /// Construct a mixed-family factory with bounded LTC sampling probability.
    pub fn new(
        mlp_factory: MlpControllerFactory,
        ltc_factory: LtcControllerFactory,
        ltc_probability: f32,
    ) -> Self {
        Self {
            mlp_factory,
            ltc_factory,
            ltc_probability: ltc_probability.clamp(0.0, 1.0),
        }
    }
}

impl ControllerFactory for MixedControllerFactory {
    fn random_genome(&self, rng: &mut StdRng) -> BrainGenome {
        if rng.gen_bool(self.ltc_probability as f64) {
            self.ltc_factory.random_genome(rng)
        } else {
            self.mlp_factory.random_genome(rng)
        }
    }

    fn build(&self, genome: &BrainGenome) -> Box<dyn NeuralController> {
        match genome {
            BrainGenome::Mlp(_) => self.mlp_factory.build(genome),
            BrainGenome::Ltc(_) => self.ltc_factory.build(genome),
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};

    use super::{
        ControllerFactory, ControllerKind, LtcControllerFactory, LtcShape, MixedControllerFactory,
        MlpControllerFactory, MlpShape,
    };

    #[test]
    fn mixed_factory_can_force_only_mlp_generation() {
        let mixed = MixedControllerFactory::new(
            MlpControllerFactory::new(MlpShape {
                input_size: 4,
                hidden_size: 6,
                output_size: 2,
            }),
            LtcControllerFactory::new(LtcShape {
                input_size: 4,
                state_size: 5,
                output_size: 2,
            }),
            0.0,
        );
        let mut rng = StdRng::seed_from_u64(7);
        let genome = mixed.random_genome(&mut rng);
        let controller = mixed.build(&genome);

        assert_eq!(controller.controller_kind(), ControllerKind::Mlp);
    }

    #[test]
    fn mixed_factory_can_force_only_ltc_generation() {
        let mixed = MixedControllerFactory::new(
            MlpControllerFactory::new(MlpShape {
                input_size: 4,
                hidden_size: 6,
                output_size: 2,
            }),
            LtcControllerFactory::new(LtcShape {
                input_size: 4,
                state_size: 5,
                output_size: 2,
            }),
            1.0,
        );
        let mut rng = StdRng::seed_from_u64(7);
        let genome = mixed.random_genome(&mut rng);
        let controller = mixed.build(&genome);

        assert_eq!(controller.controller_kind(), ControllerKind::Ltc);
    }
}
