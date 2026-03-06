use rand::{Rng, rngs::StdRng};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::brain::{
    BrainGenome, ControllerFactory, ControllerKind, ControllerStepOutput, NeuralController,
};
use crate::sim::{ActionVector, SensorVector};

/// Layer sizes for a simple LTC network.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LtcShape {
    pub input_size: usize,
    pub state_size: usize,
    pub output_size: usize,
}

impl LtcShape {
    /// Validate shape invariants required for matrix-vector operations.
    pub fn validate(self) -> Result<(), LtcGenomeError> {
        if self.input_size == 0 {
            return Err(LtcGenomeError::InvalidShape {
                field: "input_size",
                message: "must be >= 1".into(),
            });
        }

        if self.state_size == 0 {
            return Err(LtcGenomeError::InvalidShape {
                field: "state_size",
                message: "must be >= 1".into(),
            });
        }

        if self.output_size == 0 {
            return Err(LtcGenomeError::InvalidShape {
                field: "output_size",
                message: "must be >= 1".into(),
            });
        }

        Ok(())
    }

    /// Total scalar parameter count for this shape.
    pub fn parameter_count(self) -> usize {
        self.state_size * self.input_size
            + self.state_size * self.state_size
            + self.state_size
            + self.state_size
            + self.output_size * self.state_size
            + self.output_size
    }
}

/// Serialized LTC parameters with explicit state time constants.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LtcGenome {
    pub shape: LtcShape,
    /// Row-major `[state][input]` matrix.
    pub weights_input_state: Vec<f32>,
    /// Row-major `[state][state]` matrix.
    pub weights_state_state: Vec<f32>,
    pub bias_state: Vec<f32>,
    pub tau_state: Vec<f32>,
    /// Row-major `[output][state]` matrix.
    pub weights_state_output: Vec<f32>,
    pub bias_output: Vec<f32>,
}

/// Construction/validation errors for LTC genomes.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum LtcGenomeError {
    #[error("invalid shape field `{field}`: {message}")]
    InvalidShape {
        field: &'static str,
        message: String,
    },
    #[error("invalid parameter vector length: expected {expected}, got {actual}")]
    InvalidParameterLength { expected: usize, actual: usize },
}

impl LtcGenome {
    /// Build a random LTC genome and clamp values into stable stepping ranges.
    pub fn random(shape: LtcShape, rng: &mut StdRng) -> Self {
        shape
            .validate()
            .expect("factory should only construct valid LTC shapes");

        let mut genome = Self {
            shape,
            weights_input_state: (0..shape.state_size * shape.input_size)
                .map(|_| rng.gen_range(-1.0..=1.0))
                .collect(),
            weights_state_state: (0..shape.state_size * shape.state_size)
                .map(|_| rng.gen_range(-0.8..=0.8))
                .collect(),
            bias_state: (0..shape.state_size)
                .map(|_| rng.gen_range(-0.5..=0.5))
                .collect(),
            tau_state: (0..shape.state_size)
                .map(|_| rng.gen_range(0.5..=2.5))
                .collect(),
            weights_state_output: (0..shape.output_size * shape.state_size)
                .map(|_| rng.gen_range(-1.0..=1.0))
                .collect(),
            bias_output: (0..shape.output_size)
                .map(|_| rng.gen_range(-0.5..=0.5))
                .collect(),
        };
        genome.clamp_stable_ranges();
        genome
    }

    /// Construct a genome from a flattened parameter vector.
    pub fn from_flat_parameters(
        shape: LtcShape,
        parameters: &[f32],
    ) -> Result<Self, LtcGenomeError> {
        shape.validate()?;

        let expected = shape.parameter_count();
        if parameters.len() != expected {
            return Err(LtcGenomeError::InvalidParameterLength {
                expected,
                actual: parameters.len(),
            });
        }

        let mut cursor = 0;

        let input_state_len = shape.state_size * shape.input_size;
        let weights_input_state = parameters[cursor..cursor + input_state_len].to_vec();
        cursor += input_state_len;

        let state_state_len = shape.state_size * shape.state_size;
        let weights_state_state = parameters[cursor..cursor + state_state_len].to_vec();
        cursor += state_state_len;

        let bias_state = parameters[cursor..cursor + shape.state_size].to_vec();
        cursor += shape.state_size;

        let tau_state = parameters[cursor..cursor + shape.state_size].to_vec();
        cursor += shape.state_size;

        let state_output_len = shape.output_size * shape.state_size;
        let weights_state_output = parameters[cursor..cursor + state_output_len].to_vec();
        cursor += state_output_len;

        let bias_output = parameters[cursor..cursor + shape.output_size].to_vec();

        Ok(Self {
            shape,
            weights_input_state,
            weights_state_state,
            bias_state,
            tau_state,
            weights_state_output,
            bias_output,
        })
    }

    /// Flatten this genome into deterministic row-major parameter order.
    pub fn to_flat_parameters(&self) -> Vec<f32> {
        let mut flat = Vec::with_capacity(self.parameter_count());
        flat.extend_from_slice(&self.weights_input_state);
        flat.extend_from_slice(&self.weights_state_state);
        flat.extend_from_slice(&self.bias_state);
        flat.extend_from_slice(&self.tau_state);
        flat.extend_from_slice(&self.weights_state_output);
        flat.extend_from_slice(&self.bias_output);
        flat
    }

    /// Number of parameters in this genome.
    pub fn parameter_count(&self) -> usize {
        self.shape.parameter_count()
    }

    /// Clamp parameters to ranges that keep discrete stepping numerically stable.
    pub fn clamp_stable_ranges(&mut self) {
        for value in &mut self.weights_input_state {
            *value = value.clamp(-3.0, 3.0);
        }
        for value in &mut self.weights_state_state {
            *value = value.clamp(-2.0, 2.0);
        }
        for value in &mut self.bias_state {
            *value = value.clamp(-3.0, 3.0);
        }
        for value in &mut self.tau_state {
            *value = value.clamp(0.1, 10.0);
        }
        for value in &mut self.weights_state_output {
            *value = value.clamp(-3.0, 3.0);
        }
        for value in &mut self.bias_output {
            *value = value.clamp(-3.0, 3.0);
        }
    }

    /// Validate internal vector lengths against shape.
    pub fn validate(&self) -> Result<(), LtcGenomeError> {
        self.shape.validate()?;

        if self.weights_input_state.len() != self.shape.state_size * self.shape.input_size {
            return Err(LtcGenomeError::InvalidParameterLength {
                expected: self.shape.state_size * self.shape.input_size,
                actual: self.weights_input_state.len(),
            });
        }

        if self.weights_state_state.len() != self.shape.state_size * self.shape.state_size {
            return Err(LtcGenomeError::InvalidParameterLength {
                expected: self.shape.state_size * self.shape.state_size,
                actual: self.weights_state_state.len(),
            });
        }

        if self.bias_state.len() != self.shape.state_size {
            return Err(LtcGenomeError::InvalidParameterLength {
                expected: self.shape.state_size,
                actual: self.bias_state.len(),
            });
        }

        if self.tau_state.len() != self.shape.state_size {
            return Err(LtcGenomeError::InvalidParameterLength {
                expected: self.shape.state_size,
                actual: self.tau_state.len(),
            });
        }

        if self.weights_state_output.len() != self.shape.output_size * self.shape.state_size {
            return Err(LtcGenomeError::InvalidParameterLength {
                expected: self.shape.output_size * self.shape.state_size,
                actual: self.weights_state_output.len(),
            });
        }

        if self.bias_output.len() != self.shape.output_size {
            return Err(LtcGenomeError::InvalidParameterLength {
                expected: self.shape.output_size,
                actual: self.bias_output.len(),
            });
        }

        Ok(())
    }
}

/// Runtime LTC controller with persistent hidden state.
#[derive(Clone, Debug)]
pub struct LtcController {
    genome: LtcGenome,
    state: Vec<f32>,
}

impl LtcController {
    /// Build a controller from a validated LTC genome.
    pub fn new(mut genome: LtcGenome) -> Result<Self, LtcGenomeError> {
        genome.validate()?;
        genome.clamp_stable_ranges();

        Ok(Self {
            state: vec![0.0; genome.shape.state_size],
            genome,
        })
    }

    fn forward_outputs(&mut self, input: &SensorVector, dt: f32) -> Vec<f32> {
        let shape = self.genome.shape;
        let dt = dt.max(0.0);

        let mut input_buffer = vec![0.0; shape.input_size];
        for (dst, src) in input_buffer.iter_mut().zip(input.values.iter().copied()) {
            *dst = src;
        }

        let previous_state = self.state.clone();

        for state_idx in 0..shape.state_size {
            let input_row_start = state_idx * shape.input_size;
            let input_drive = self.genome.weights_input_state
                [input_row_start..input_row_start + shape.input_size]
                .iter()
                .zip(input_buffer.iter())
                .map(|(w, x)| w * x)
                .sum::<f32>();

            let recurrent_row_start = state_idx * shape.state_size;
            let recurrent_drive = self.genome.weights_state_state
                [recurrent_row_start..recurrent_row_start + shape.state_size]
                .iter()
                .zip(previous_state.iter())
                .map(|(w, x)| w * x.tanh())
                .sum::<f32>();

            let target = input_drive + recurrent_drive + self.genome.bias_state[state_idx];
            let tau = self.genome.tau_state[state_idx].clamp(0.1, 10.0);
            let derivative = (-previous_state[state_idx] + target) / tau;
            self.state[state_idx] = previous_state[state_idx] + dt * derivative;
        }

        let mut outputs = vec![0.0; shape.output_size];
        for (output_idx, output_value) in outputs.iter_mut().enumerate() {
            let row_start = output_idx * shape.state_size;
            let weighted_sum = self.genome.weights_state_output
                [row_start..row_start + shape.state_size]
                .iter()
                .zip(self.state.iter())
                .map(|(w, s)| w * s.tanh())
                .sum::<f32>();

            *output_value = (weighted_sum + self.genome.bias_output[output_idx]).tanh();
        }

        outputs
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

impl NeuralController for LtcController {
    fn reset(&mut self) {
        self.state.fill(0.0);
    }

    fn step(&mut self, input: &SensorVector, dt: f32) -> ActionVector {
        self.step_with_observability(input, dt).action
    }

    fn step_with_observability(&mut self, input: &SensorVector, dt: f32) -> ControllerStepOutput {
        let outputs = self.forward_outputs(input, dt);
        Self::step_output_from_signals(&outputs)
    }

    fn parameter_count(&self) -> usize {
        self.genome.parameter_count()
    }

    fn controller_kind(&self) -> ControllerKind {
        ControllerKind::Ltc
    }
}

/// Factory for creating LTC genomes and runtime controllers.
#[derive(Clone, Copy, Debug)]
pub struct LtcControllerFactory {
    pub shape: LtcShape,
}

impl LtcControllerFactory {
    /// Create a factory with deterministic defaults.
    pub const fn new(shape: LtcShape) -> Self {
        Self { shape }
    }
}

impl ControllerFactory for LtcControllerFactory {
    fn random_genome(&self, rng: &mut StdRng) -> BrainGenome {
        BrainGenome::Ltc(LtcGenome::random(self.shape, rng))
    }

    fn build(&self, genome: &BrainGenome) -> Box<dyn NeuralController> {
        match genome {
            BrainGenome::Ltc(ltc) => Box::new(
                LtcController::new(ltc.clone())
                    .expect("LTC factory received invalid LTC genome shape/parameter sizes"),
            ),
            BrainGenome::Mlp(_) => {
                panic!("LTC factory cannot build MLP genomes");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};

    use crate::brain::{ControllerFactory, ControllerKind, NeuralController};
    use crate::sim::SensorVector;

    use super::{LtcController, LtcControllerFactory, LtcGenome, LtcGenomeError, LtcShape};

    #[test]
    fn deterministic_state_evolution_matches_known_sequence() {
        let shape = LtcShape {
            input_size: 1,
            state_size: 1,
            output_size: 2,
        };

        let params = [
            1.0, // input->state
            0.0, // state->state
            0.0, // state bias
            1.0, // tau
            1.0, -1.0, // state->output
            0.0, 0.0, // output bias
        ];

        let genome = LtcGenome::from_flat_parameters(shape, &params)
            .expect("known LTC parameter vector must be valid");
        let mut controller =
            LtcController::new(genome).expect("known genome must construct controller");

        let action_1 = controller.step(&SensorVector { values: vec![0.5] }, 0.1);
        let action_2 = controller.step(&SensorVector { values: vec![0.5] }, 0.1);
        let action_3 = controller.step(&SensorVector { values: vec![0.0] }, 0.1);

        assert!((action_1.steering - 0.0499169).abs() < 1e-5);
        assert!((action_1.throttle - 0.4750416).abs() < 1e-5);
        assert!((action_2.steering - 0.0944330).abs() < 1e-5);
        assert!((action_2.throttle - 0.4527835).abs() < 1e-5);
        assert!((action_3.steering - 0.0850860).abs() < 1e-5);
        assert!((action_3.throttle - 0.4574570).abs() < 1e-5);
    }

    #[test]
    fn clamp_stable_ranges_limits_extreme_values() {
        let shape = LtcShape {
            input_size: 1,
            state_size: 1,
            output_size: 1,
        };

        let params = [100.0, -99.0, 88.0, 0.001, -77.0, 66.0];
        let mut genome = LtcGenome::from_flat_parameters(shape, &params)
            .expect("parameter vector should match LTC shape");

        genome.clamp_stable_ranges();

        assert_eq!(genome.weights_input_state[0], 3.0);
        assert_eq!(genome.weights_state_state[0], -2.0);
        assert_eq!(genome.bias_state[0], 3.0);
        assert_eq!(genome.tau_state[0], 0.1);
        assert_eq!(genome.weights_state_output[0], -3.0);
        assert_eq!(genome.bias_output[0], 3.0);
    }

    #[test]
    fn from_flat_parameters_rejects_wrong_length() {
        let shape = LtcShape {
            input_size: 2,
            state_size: 2,
            output_size: 2,
        };

        let err = LtcGenome::from_flat_parameters(shape, &[0.0, 1.0]).expect_err("must fail");
        assert_eq!(
            err,
            LtcGenomeError::InvalidParameterLength {
                expected: shape.parameter_count(),
                actual: 2,
            }
        );
    }

    #[test]
    fn factory_builds_ltc_controller_with_expected_kind() {
        let shape = LtcShape {
            input_size: 4,
            state_size: 3,
            output_size: 2,
        };
        let mut rng = StdRng::seed_from_u64(321);
        let factory = LtcControllerFactory::new(shape);
        let genome = factory.random_genome(&mut rng);

        let controller = factory.build(&genome);

        assert_eq!(controller.controller_kind(), ControllerKind::Ltc);
        assert_eq!(controller.parameter_count(), shape.parameter_count());
    }

    #[test]
    fn step_with_observability_matches_action_mapping() {
        let shape = LtcShape {
            input_size: 1,
            state_size: 1,
            output_size: 2,
        };
        let genome =
            LtcGenome::from_flat_parameters(shape, &[1.0, 0.0, 0.0, 1.0, 1.0, -1.0, 0.0, 0.0])
                .expect("parameter vector should match LTC shape");
        let mut controller = LtcController::new(genome).expect("genome should construct");

        let output = controller.step_with_observability(&SensorVector { values: vec![0.5] }, 0.1);

        assert!((output.action.steering - output.steering_raw).abs() < 1e-6);
        assert!((output.action.throttle - output.throttle_applied).abs() < 1e-6);
        assert!((output.throttle_applied - 0.4750416).abs() < 1e-5);
    }
}
