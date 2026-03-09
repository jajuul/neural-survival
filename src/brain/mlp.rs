use rand::{Rng, rngs::StdRng};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::brain::{
    BrainGenome, ControllerDiagnostics, ControllerFactory, ControllerKind,
    ControllerNetworkSnapshot, ControllerStepOutput, MlpNetworkSnapshot, NeuralController,
};
use crate::sim::{ActionVector, SensorVector};

/// Layer sizes for a single-hidden-layer feed-forward MLP.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MlpShape {
    pub input_size: usize,
    pub hidden_size: usize,
    pub output_size: usize,
}

impl MlpShape {
    /// Validate shape invariants required for matrix-vector operations.
    pub fn validate(self) -> Result<(), MlpGenomeError> {
        if self.input_size == 0 {
            return Err(MlpGenomeError::InvalidShape {
                field: "input_size",
                message: "must be >= 1".into(),
            });
        }

        if self.hidden_size == 0 {
            return Err(MlpGenomeError::InvalidShape {
                field: "hidden_size",
                message: "must be >= 1".into(),
            });
        }

        if self.output_size == 0 {
            return Err(MlpGenomeError::InvalidShape {
                field: "output_size",
                message: "must be >= 1".into(),
            });
        }

        Ok(())
    }

    /// Total scalar parameter count for this shape.
    pub fn parameter_count(self) -> usize {
        self.hidden_size * self.input_size
            + self.hidden_size
            + self.output_size * self.hidden_size
            + self.output_size
    }
}

/// Serialized MLP parameters (weights and biases).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MlpGenome {
    pub shape: MlpShape,
    /// Row-major `[hidden][input]` matrix.
    pub weights_input_hidden: Vec<f32>,
    pub bias_hidden: Vec<f32>,
    /// Row-major `[output][hidden]` matrix.
    pub weights_hidden_output: Vec<f32>,
    pub bias_output: Vec<f32>,
}

/// Construction/validation errors for MLP genomes.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum MlpGenomeError {
    #[error("invalid shape field `{field}`: {message}")]
    InvalidShape {
        field: &'static str,
        message: String,
    },
    #[error("invalid parameter vector length: expected {expected}, got {actual}")]
    InvalidParameterLength { expected: usize, actual: usize },
}

impl MlpGenome {
    /// Build a random MLP genome for a validated shape.
    pub fn random(shape: MlpShape, rng: &mut StdRng, min_weight: f32, max_weight: f32) -> Self {
        shape
            .validate()
            .expect("factory should only construct valid MLP shapes");

        let mut sample = || rng.gen_range(min_weight..=max_weight);

        Self {
            shape,
            weights_input_hidden: (0..shape.hidden_size * shape.input_size)
                .map(|_| sample())
                .collect(),
            bias_hidden: (0..shape.hidden_size).map(|_| sample()).collect(),
            weights_hidden_output: (0..shape.output_size * shape.hidden_size)
                .map(|_| sample())
                .collect(),
            bias_output: (0..shape.output_size).map(|_| sample()).collect(),
        }
    }

    /// Construct a genome from a single flattened parameter vector.
    pub fn from_flat_parameters(
        shape: MlpShape,
        parameters: &[f32],
    ) -> Result<Self, MlpGenomeError> {
        shape.validate()?;

        let expected = shape.parameter_count();
        if parameters.len() != expected {
            return Err(MlpGenomeError::InvalidParameterLength {
                expected,
                actual: parameters.len(),
            });
        }

        let mut cursor = 0;

        let input_hidden_len = shape.hidden_size * shape.input_size;
        let weights_input_hidden = parameters[cursor..cursor + input_hidden_len].to_vec();
        cursor += input_hidden_len;

        let bias_hidden = parameters[cursor..cursor + shape.hidden_size].to_vec();
        cursor += shape.hidden_size;

        let hidden_output_len = shape.output_size * shape.hidden_size;
        let weights_hidden_output = parameters[cursor..cursor + hidden_output_len].to_vec();
        cursor += hidden_output_len;

        let bias_output = parameters[cursor..cursor + shape.output_size].to_vec();

        Ok(Self {
            shape,
            weights_input_hidden,
            bias_hidden,
            weights_hidden_output,
            bias_output,
        })
    }

    /// Flatten this genome into deterministic row-major parameter order.
    pub fn to_flat_parameters(&self) -> Vec<f32> {
        let mut flat = Vec::with_capacity(self.parameter_count());
        flat.extend_from_slice(&self.weights_input_hidden);
        flat.extend_from_slice(&self.bias_hidden);
        flat.extend_from_slice(&self.weights_hidden_output);
        flat.extend_from_slice(&self.bias_output);
        flat
    }

    /// Number of parameters in this genome.
    pub fn parameter_count(&self) -> usize {
        self.shape.parameter_count()
    }

    /// Validate internal vector lengths against shape.
    pub fn validate(&self) -> Result<(), MlpGenomeError> {
        self.shape.validate()?;

        if self.weights_input_hidden.len() != self.shape.hidden_size * self.shape.input_size {
            return Err(MlpGenomeError::InvalidParameterLength {
                expected: self.shape.hidden_size * self.shape.input_size,
                actual: self.weights_input_hidden.len(),
            });
        }

        if self.bias_hidden.len() != self.shape.hidden_size {
            return Err(MlpGenomeError::InvalidParameterLength {
                expected: self.shape.hidden_size,
                actual: self.bias_hidden.len(),
            });
        }

        if self.weights_hidden_output.len() != self.shape.output_size * self.shape.hidden_size {
            return Err(MlpGenomeError::InvalidParameterLength {
                expected: self.shape.output_size * self.shape.hidden_size,
                actual: self.weights_hidden_output.len(),
            });
        }

        if self.bias_output.len() != self.shape.output_size {
            return Err(MlpGenomeError::InvalidParameterLength {
                expected: self.shape.output_size,
                actual: self.bias_output.len(),
            });
        }

        Ok(())
    }
}

/// Runtime feed-forward MLP controller.
#[derive(Clone, Debug)]
pub struct MlpController {
    genome: MlpGenome,
}

impl MlpController {
    /// Build a controller from a validated MLP genome.
    pub fn new(genome: MlpGenome) -> Result<Self, MlpGenomeError> {
        genome.validate()?;
        Ok(Self { genome })
    }

    fn forward_snapshot(&self, input: &SensorVector) -> MlpNetworkSnapshot {
        let shape = self.genome.shape;

        let mut input_buffer = vec![0.0; shape.input_size];
        for (dst, src) in input_buffer.iter_mut().zip(input.values.iter().copied()) {
            *dst = src;
        }

        let mut hidden = vec![0.0; shape.hidden_size];
        for (hidden_idx, hidden_value) in hidden.iter_mut().enumerate() {
            let row_start = hidden_idx * shape.input_size;
            let weighted_sum = self.genome.weights_input_hidden
                [row_start..row_start + shape.input_size]
                .iter()
                .zip(input_buffer.iter())
                .map(|(w, x)| w * x)
                .sum::<f32>();

            *hidden_value = (weighted_sum + self.genome.bias_hidden[hidden_idx]).tanh();
        }

        let mut outputs = vec![0.0; shape.output_size];
        for (output_idx, output_value) in outputs.iter_mut().enumerate() {
            let row_start = output_idx * shape.hidden_size;
            let weighted_sum = self.genome.weights_hidden_output
                [row_start..row_start + shape.hidden_size]
                .iter()
                .zip(hidden.iter())
                .map(|(w, x)| w * x)
                .sum::<f32>();

            *output_value = (weighted_sum + self.genome.bias_output[output_idx]).tanh();
        }

        MlpNetworkSnapshot {
            input_values: input_buffer,
            hidden_values: hidden,
            output_values: outputs,
            weights_input_hidden: self.genome.weights_input_hidden.clone(),
            weights_hidden_output: self.genome.weights_hidden_output.clone(),
        }
    }

    fn step_output_from_signals(outputs: &[f32]) -> ControllerStepOutput {
        let steering_raw = outputs.first().copied().unwrap_or(0.0).clamp(-1.0, 1.0);
        let throttle_raw = outputs.get(1).copied().unwrap_or(0.0).clamp(-1.0, 1.0);
        let throttle_applied = ((throttle_raw + 1.0) * 0.5).clamp(0.0, 1.0);

        ControllerStepOutput {
            action: ActionVector {
                steering: steering_raw,
                throttle: throttle_applied,
            },
            steering_raw,
            throttle_raw,
            throttle_applied,
        }
    }
}

impl NeuralController for MlpController {
    fn reset(&mut self) {
        // Feed-forward MLP has no hidden state to reset.
    }

    fn step(&mut self, input: &SensorVector, _dt: f32) -> ActionVector {
        self.step_with_observability(input, 0.0).action
    }

    fn step_with_observability(&mut self, input: &SensorVector, _dt: f32) -> ControllerStepOutput {
        let snapshot = self.forward_snapshot(input);
        Self::step_output_from_signals(&snapshot.output_values)
    }

    fn inspect_network(&self, input: &SensorVector) -> ControllerNetworkSnapshot {
        ControllerNetworkSnapshot::Mlp(self.forward_snapshot(input))
    }

    fn step_with_diagnostics(&mut self, input: &SensorVector, _dt: f32) -> ControllerDiagnostics {
        let snapshot = self.forward_snapshot(input);
        ControllerDiagnostics {
            output: Self::step_output_from_signals(&snapshot.output_values),
            network: ControllerNetworkSnapshot::Mlp(snapshot),
        }
    }

    fn parameter_count(&self) -> usize {
        self.genome.parameter_count()
    }

    fn controller_kind(&self) -> ControllerKind {
        ControllerKind::Mlp
    }
}

/// Factory for creating MLP genomes and runtime controllers.
#[derive(Clone, Copy, Debug)]
pub struct MlpControllerFactory {
    pub shape: MlpShape,
    pub min_weight: f32,
    pub max_weight: f32,
}

impl MlpControllerFactory {
    /// Create a factory with deterministic defaults.
    pub const fn new(shape: MlpShape) -> Self {
        Self {
            shape,
            min_weight: -1.0,
            max_weight: 1.0,
        }
    }
}

impl ControllerFactory for MlpControllerFactory {
    fn random_genome(&self, rng: &mut StdRng) -> BrainGenome {
        BrainGenome::Mlp(MlpGenome::random(
            self.shape,
            rng,
            self.min_weight,
            self.max_weight,
        ))
    }

    fn build(&self, genome: &BrainGenome) -> Box<dyn NeuralController> {
        match genome {
            BrainGenome::Mlp(mlp) => Box::new(
                MlpController::new(mlp.clone())
                    .expect("MLP factory received invalid MLP genome shape/parameter sizes"),
            ),
            BrainGenome::Ltc(_) => {
                panic!("MLP factory cannot build LTC genomes");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};

    use crate::brain::{
        ControllerFactory, ControllerKind, ControllerNetworkSnapshot, NeuralController,
    };
    use crate::sim::SensorVector;

    use super::{MlpController, MlpControllerFactory, MlpGenome, MlpGenomeError, MlpShape};

    #[test]
    fn genome_round_trip_flat_parameters() {
        let shape = MlpShape {
            input_size: 3,
            hidden_size: 2,
            output_size: 2,
        };
        let params: Vec<f32> = (0..shape.parameter_count())
            .map(|idx| idx as f32 * 0.1 - 0.2)
            .collect();

        let genome = MlpGenome::from_flat_parameters(shape, &params)
            .expect("parameter vector should match MLP shape");

        assert_eq!(genome.to_flat_parameters(), params);
    }

    #[test]
    fn from_flat_parameters_rejects_wrong_length() {
        let shape = MlpShape {
            input_size: 4,
            hidden_size: 3,
            output_size: 2,
        };

        let err = MlpGenome::from_flat_parameters(shape, &[0.0, 1.0]).expect_err("must fail");
        assert_eq!(
            err,
            MlpGenomeError::InvalidParameterLength {
                expected: shape.parameter_count(),
                actual: 2,
            }
        );
    }

    #[test]
    fn deterministic_forward_matches_known_values() {
        let shape = MlpShape {
            input_size: 2,
            hidden_size: 2,
            output_size: 2,
        };
        let params = [
            0.5, -0.25, -0.75, 0.1, // input -> hidden
            0.1, -0.2, // hidden bias
            0.3, -0.4, 0.2, 0.6, // hidden -> output
            0.05, -0.1, // output bias
        ];

        let genome = MlpGenome::from_flat_parameters(shape, &params)
            .expect("known MLP parameter vector must be valid");
        let mut controller =
            MlpController::new(genome).expect("known genome must construct controller");

        let action = controller.step(&SensorVector::from_slice(&[1.0, -1.0]), 0.1);

        assert!((action.steering - 0.5153911).abs() < 1e-5);
        assert!((action.throttle - 0.2969760).abs() < 1e-5);
    }

    #[test]
    fn factory_builds_mlp_controller_with_expected_kind() {
        let shape = MlpShape {
            input_size: 4,
            hidden_size: 3,
            output_size: 2,
        };
        let mut rng = StdRng::seed_from_u64(123);
        let factory = MlpControllerFactory::new(shape);
        let genome = factory.random_genome(&mut rng);

        let controller = factory.build(&genome);

        assert_eq!(controller.controller_kind(), ControllerKind::Mlp);
        assert_eq!(controller.parameter_count(), shape.parameter_count());
    }

    #[test]
    fn step_with_observability_matches_action_mapping() {
        let shape = MlpShape {
            input_size: 2,
            hidden_size: 2,
            output_size: 2,
        };
        let genome = MlpGenome::from_flat_parameters(
            shape,
            &[
                0.5, -0.25, -0.75, 0.1, 0.1, -0.2, 0.3, -0.4, 0.2, 0.6, 0.05, -0.1,
            ],
        )
        .expect("parameter vector should match MLP shape");
        let mut controller = MlpController::new(genome).expect("genome should construct");

        let output =
            controller.step_with_observability(&SensorVector::from_slice(&[1.0, -1.0]), 0.1);

        assert!((output.action.steering - output.steering_raw).abs() < 1e-6);
        assert!((output.action.throttle - output.throttle_applied).abs() < 1e-6);
        assert!((output.throttle_applied - 0.2969760).abs() < 1e-5);
    }

    #[test]
    fn step_with_diagnostics_exposes_layer_shapes() {
        let shape = MlpShape {
            input_size: 2,
            hidden_size: 2,
            output_size: 2,
        };
        let genome = MlpGenome::from_flat_parameters(
            shape,
            &[
                0.5, -0.25, -0.75, 0.1, 0.1, -0.2, 0.3, -0.4, 0.2, 0.6, 0.05, -0.1,
            ],
        )
        .expect("parameter vector should match MLP shape");
        let mut controller = MlpController::new(genome).expect("genome should construct");

        let diagnostics =
            controller.step_with_diagnostics(&SensorVector::from_slice(&[1.0, -1.0]), 0.1);

        let ControllerNetworkSnapshot::Mlp(network) = diagnostics.network else {
            panic!("expected MLP network snapshot");
        };

        assert_eq!(network.input_values.len(), 2);
        assert_eq!(network.hidden_values.len(), 2);
        assert_eq!(network.output_values.len(), 2);
        assert_eq!(network.weights_input_hidden.len(), 4);
        assert_eq!(network.weights_hidden_output.len(), 4);
    }
}
