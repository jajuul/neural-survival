use rand::{Rng, rngs::StdRng};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::brain::{
    BrainGenome, ControllerDiagnostics, ControllerFactory, ControllerKind,
    ControllerNetworkSnapshot, ControllerStepOutput, LtcNetworkSnapshot, NeuralController,
};
use crate::sim::{ActionVector, SensorVector};

const LTC_ODE_UNFOLDS: usize = 6;
const MIN_CONDUCTANCE: f32 = 0.001;
const MAX_CONDUCTANCE: f32 = 3.0;
const MIN_SIGMA: f32 = 0.1;
const MAX_SIGMA: f32 = 10.0;
const MIN_MU: f32 = -1.5;
const MAX_MU: f32 = 1.5;
const MIN_EREV: f32 = -1.0;
const MAX_EREV: f32 = 1.0;
const MIN_VLEAK: f32 = -1.0;
const MAX_VLEAK: f32 = 1.0;
const MIN_CM: f32 = 0.05;
const MAX_CM: f32 = 5.0;
const MIN_OUTPUT_WEIGHT: f32 = -3.0;
const MAX_OUTPUT_WEIGHT: f32 = 3.0;
const MIN_OUTPUT_BIAS: f32 = -3.0;
const MAX_OUTPUT_BIAS: f32 = 3.0;
const DENOMINATOR_EPSILON: f32 = 1e-6;

/// Layer sizes for a conductance-based LTC network.
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
        4 * self.state_size * self.input_size
            + 4 * self.state_size * self.state_size
            + 3 * self.state_size
            + self.output_size * self.state_size
            + self.output_size
    }
}

/// Serialized conductance-based LTC parameters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LtcGenome {
    pub shape: LtcShape,
    /// Row-major `[state][input]` sensory conductance magnitudes.
    #[serde(alias = "weights_input_state")]
    pub sensory_weight: Vec<f32>,
    /// Row-major `[state][input]` sensory gate means.
    #[serde(default)]
    pub sensory_mu: Vec<f32>,
    /// Row-major `[state][input]` sensory gate slopes.
    #[serde(default)]
    pub sensory_sigma: Vec<f32>,
    /// Row-major `[state][input]` sensory reversal potentials.
    #[serde(default)]
    pub sensory_erev: Vec<f32>,
    /// Row-major `[state][state]` recurrent conductance magnitudes.
    #[serde(alias = "weights_state_state")]
    pub recurrent_weight: Vec<f32>,
    /// Row-major `[state][state]` recurrent gate means.
    #[serde(default)]
    pub recurrent_mu: Vec<f32>,
    /// Row-major `[state][state]` recurrent gate slopes.
    #[serde(default)]
    pub recurrent_sigma: Vec<f32>,
    /// Row-major `[state][state]` recurrent reversal potentials.
    #[serde(default)]
    pub recurrent_erev: Vec<f32>,
    /// Per-state leak reversal potential.
    #[serde(alias = "bias_state")]
    pub vleak_state: Vec<f32>,
    /// Per-state membrane capacitance / inertia term.
    #[serde(alias = "tau_state")]
    pub cm_state: Vec<f32>,
    /// Per-state leak conductance.
    #[serde(default)]
    pub gleak_state: Vec<f32>,
    /// Row-major `[output][state]` readout matrix.
    #[serde(alias = "weights_state_output")]
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
    #[error("invalid parameter vector length for `{field}`: expected {expected}, got {actual}")]
    InvalidParameterLength {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
}

impl LtcGenome {
    /// Build a random LTC genome and clamp values into stable stepping ranges.
    pub fn random(shape: LtcShape, rng: &mut StdRng) -> Self {
        shape
            .validate()
            .expect("factory should only construct valid LTC shapes");

        let sensory_len = shape.state_size * shape.input_size;
        let recurrent_len = shape.state_size * shape.state_size;
        let output_len = shape.output_size * shape.state_size;

        let mut genome = Self {
            shape,
            sensory_weight: (0..sensory_len)
                .map(|_| rng.gen_range(0.05..=1.0))
                .collect(),
            sensory_mu: (0..sensory_len).map(|_| rng.gen_range(0.0..=1.0)).collect(),
            sensory_sigma: (0..sensory_len).map(|_| rng.gen_range(1.0..=8.0)).collect(),
            sensory_erev: (0..sensory_len)
                .map(|_| if rng.gen_bool(0.5) { 1.0 } else { -1.0 })
                .collect(),
            recurrent_weight: (0..recurrent_len)
                .map(|_| rng.gen_range(0.05..=1.0))
                .collect(),
            recurrent_mu: (0..recurrent_len)
                .map(|_| rng.gen_range(-0.5..=0.5))
                .collect(),
            recurrent_sigma: (0..recurrent_len)
                .map(|_| rng.gen_range(1.0..=8.0))
                .collect(),
            recurrent_erev: (0..recurrent_len)
                .map(|_| if rng.gen_bool(0.5) { 1.0 } else { -1.0 })
                .collect(),
            vleak_state: (0..shape.state_size)
                .map(|_| rng.gen_range(-0.2..=0.2))
                .collect(),
            cm_state: (0..shape.state_size)
                .map(|_| rng.gen_range(0.4..=0.8))
                .collect(),
            gleak_state: (0..shape.state_size)
                .map(|_| rng.gen_range(0.05..=1.0))
                .collect(),
            weights_state_output: (0..output_len).map(|_| rng.gen_range(-1.0..=1.0)).collect(),
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
                field: "flat_parameters",
                expected,
                actual: parameters.len(),
            });
        }

        let sensory_len = shape.state_size * shape.input_size;
        let recurrent_len = shape.state_size * shape.state_size;
        let output_len = shape.output_size * shape.state_size;

        let mut cursor = 0;

        let sensory_weight = parameters[cursor..cursor + sensory_len].to_vec();
        cursor += sensory_len;
        let sensory_mu = parameters[cursor..cursor + sensory_len].to_vec();
        cursor += sensory_len;
        let sensory_sigma = parameters[cursor..cursor + sensory_len].to_vec();
        cursor += sensory_len;
        let sensory_erev = parameters[cursor..cursor + sensory_len].to_vec();
        cursor += sensory_len;

        let recurrent_weight = parameters[cursor..cursor + recurrent_len].to_vec();
        cursor += recurrent_len;
        let recurrent_mu = parameters[cursor..cursor + recurrent_len].to_vec();
        cursor += recurrent_len;
        let recurrent_sigma = parameters[cursor..cursor + recurrent_len].to_vec();
        cursor += recurrent_len;
        let recurrent_erev = parameters[cursor..cursor + recurrent_len].to_vec();
        cursor += recurrent_len;

        let vleak_state = parameters[cursor..cursor + shape.state_size].to_vec();
        cursor += shape.state_size;
        let cm_state = parameters[cursor..cursor + shape.state_size].to_vec();
        cursor += shape.state_size;
        let gleak_state = parameters[cursor..cursor + shape.state_size].to_vec();
        cursor += shape.state_size;

        let weights_state_output = parameters[cursor..cursor + output_len].to_vec();
        cursor += output_len;
        let bias_output = parameters[cursor..cursor + shape.output_size].to_vec();

        Ok(Self {
            shape,
            sensory_weight,
            sensory_mu,
            sensory_sigma,
            sensory_erev,
            recurrent_weight,
            recurrent_mu,
            recurrent_sigma,
            recurrent_erev,
            vleak_state,
            cm_state,
            gleak_state,
            weights_state_output,
            bias_output,
        })
    }

    /// Flatten this genome into deterministic row-major parameter order.
    pub fn to_flat_parameters(&self) -> Vec<f32> {
        let mut normalized = self.clone();
        normalized.ensure_layout();

        let mut flat = Vec::with_capacity(normalized.parameter_count());
        flat.extend_from_slice(&normalized.sensory_weight);
        flat.extend_from_slice(&normalized.sensory_mu);
        flat.extend_from_slice(&normalized.sensory_sigma);
        flat.extend_from_slice(&normalized.sensory_erev);
        flat.extend_from_slice(&normalized.recurrent_weight);
        flat.extend_from_slice(&normalized.recurrent_mu);
        flat.extend_from_slice(&normalized.recurrent_sigma);
        flat.extend_from_slice(&normalized.recurrent_erev);
        flat.extend_from_slice(&normalized.vleak_state);
        flat.extend_from_slice(&normalized.cm_state);
        flat.extend_from_slice(&normalized.gleak_state);
        flat.extend_from_slice(&normalized.weights_state_output);
        flat.extend_from_slice(&normalized.bias_output);
        flat
    }

    /// Number of parameters in this genome.
    pub fn parameter_count(&self) -> usize {
        self.shape.parameter_count()
    }

    /// Clamp parameters to ranges that keep conductance-based stepping stable.
    pub fn clamp_stable_ranges(&mut self) {
        self.ensure_layout();

        for value in &mut self.sensory_weight {
            *value = value.clamp(MIN_CONDUCTANCE, MAX_CONDUCTANCE);
        }
        for value in &mut self.sensory_mu {
            *value = value.clamp(MIN_MU, MAX_MU);
        }
        for value in &mut self.sensory_sigma {
            *value = value.clamp(MIN_SIGMA, MAX_SIGMA);
        }
        for value in &mut self.sensory_erev {
            *value = value.clamp(MIN_EREV, MAX_EREV);
        }
        for value in &mut self.recurrent_weight {
            *value = value.clamp(MIN_CONDUCTANCE, MAX_CONDUCTANCE);
        }
        for value in &mut self.recurrent_mu {
            *value = value.clamp(MIN_MU, MAX_MU);
        }
        for value in &mut self.recurrent_sigma {
            *value = value.clamp(MIN_SIGMA, MAX_SIGMA);
        }
        for value in &mut self.recurrent_erev {
            *value = value.clamp(MIN_EREV, MAX_EREV);
        }
        for value in &mut self.vleak_state {
            *value = value.clamp(MIN_VLEAK, MAX_VLEAK);
        }
        for value in &mut self.cm_state {
            *value = value.clamp(MIN_CM, MAX_CM);
        }
        for value in &mut self.gleak_state {
            *value = value.clamp(MIN_CONDUCTANCE, MAX_CONDUCTANCE);
        }
        for value in &mut self.weights_state_output {
            *value = value.clamp(MIN_OUTPUT_WEIGHT, MAX_OUTPUT_WEIGHT);
        }
        for value in &mut self.bias_output {
            *value = value.clamp(MIN_OUTPUT_BIAS, MAX_OUTPUT_BIAS);
        }
    }

    /// Validate internal vector lengths against shape, normalizing legacy serialized genomes.
    pub fn validate(&mut self) -> Result<(), LtcGenomeError> {
        self.shape.validate()?;
        self.ensure_layout();

        let sensory_len = self.shape.state_size * self.shape.input_size;
        let recurrent_len = self.shape.state_size * self.shape.state_size;
        let output_len = self.shape.output_size * self.shape.state_size;

        validate_len("sensory_weight", &self.sensory_weight, sensory_len)?;
        validate_len("sensory_mu", &self.sensory_mu, sensory_len)?;
        validate_len("sensory_sigma", &self.sensory_sigma, sensory_len)?;
        validate_len("sensory_erev", &self.sensory_erev, sensory_len)?;
        validate_len("recurrent_weight", &self.recurrent_weight, recurrent_len)?;
        validate_len("recurrent_mu", &self.recurrent_mu, recurrent_len)?;
        validate_len("recurrent_sigma", &self.recurrent_sigma, recurrent_len)?;
        validate_len("recurrent_erev", &self.recurrent_erev, recurrent_len)?;
        validate_len("vleak_state", &self.vleak_state, self.shape.state_size)?;
        validate_len("cm_state", &self.cm_state, self.shape.state_size)?;
        validate_len("gleak_state", &self.gleak_state, self.shape.state_size)?;
        validate_len(
            "weights_state_output",
            &self.weights_state_output,
            output_len,
        )?;
        validate_len("bias_output", &self.bias_output, self.shape.output_size)?;
        Ok(())
    }

    fn ensure_layout(&mut self) {
        let sensory_len = self.shape.state_size * self.shape.input_size;
        let recurrent_len = self.shape.state_size * self.shape.state_size;
        let sensory_weight = self.sensory_weight.clone();
        let recurrent_weight = self.recurrent_weight.clone();

        fill_or_preserve(&mut self.sensory_mu, sensory_len, 0.0);
        fill_or_preserve(&mut self.sensory_sigma, sensory_len, 1.0);
        fill_or_preserve_with(&mut self.sensory_erev, sensory_len, |idx| {
            legacy_erev(sensory_weight.get(idx).copied().unwrap_or(1.0))
        });

        fill_or_preserve(&mut self.recurrent_mu, recurrent_len, 0.0);
        fill_or_preserve(&mut self.recurrent_sigma, recurrent_len, 1.0);
        fill_or_preserve_with(&mut self.recurrent_erev, recurrent_len, |idx| {
            legacy_erev(recurrent_weight.get(idx).copied().unwrap_or(1.0))
        });

        fill_or_preserve(&mut self.gleak_state, self.shape.state_size, 1.0);
    }
}

fn validate_len(
    field: &'static str,
    values: &[f32],
    expected: usize,
) -> Result<(), LtcGenomeError> {
    if values.len() == expected {
        Ok(())
    } else {
        Err(LtcGenomeError::InvalidParameterLength {
            field,
            expected,
            actual: values.len(),
        })
    }
}

fn fill_or_preserve(values: &mut Vec<f32>, expected: usize, fill: f32) {
    if values.is_empty() {
        values.resize(expected, fill);
    }
}

fn fill_or_preserve_with<F>(values: &mut Vec<f32>, expected: usize, mut fill: F)
where
    F: FnMut(usize) -> f32,
{
    if values.is_empty() {
        values.extend((0..expected).map(&mut fill));
    }
}

fn legacy_erev(weight: f32) -> f32 {
    if weight >= 0.0 { 1.0 } else { -1.0 }
}

fn sigmoid(value: f32) -> f32 {
    1.0 / (1.0 + (-value.clamp(-60.0, 60.0)).exp())
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
        let elapsed = dt.max(1e-4);

        let input_buffer = self.input_buffer(input);
        let sensory_drive = self.compute_sensory_drive(&input_buffer);
        let cm_scale = self.compute_cm_scale(elapsed);

        for _ in 0..LTC_ODE_UNFOLDS {
            let previous_state = self.state.clone();
            self.state = self.unfold_state_once(&previous_state, &sensory_drive, &cm_scale);
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

    fn compute_cm_scale(&self, elapsed: f32) -> Vec<f32> {
        self.genome
            .cm_state
            .iter()
            .map(|cm| cm / (elapsed / LTC_ODE_UNFOLDS as f32))
            .collect()
    }

    fn input_buffer(&self, input: &SensorVector) -> Vec<f32> {
        let shape = self.genome.shape;
        let mut input_buffer = vec![0.0; shape.input_size];
        for (dst, src) in input_buffer.iter_mut().zip(input.values.iter().copied()) {
            *dst = src;
        }
        input_buffer
    }

    fn unfold_state_once(
        &self,
        previous_state: &[f32],
        sensory_drive: &SynapseDrive,
        cm_scale: &[f32],
    ) -> Vec<f32> {
        let shape = self.genome.shape;
        let mut next_state = vec![0.0; shape.state_size];

        for state_idx in 0..shape.state_size {
            let recurrent_drive = self.compute_recurrent_drive(previous_state, state_idx);

            // One LTC unfold solves the membrane voltage update as a conductance-weighted average.
            // The numerator collects "pulls" toward leak and synaptic reversal potentials, while
            // the denominator collects the total conductance plus the capacitance-scaled inertia.
            let numerator = cm_scale[state_idx] * previous_state[state_idx]
                + self.genome.gleak_state[state_idx] * self.genome.vleak_state[state_idx]
                + sensory_drive.numerator[state_idx]
                + recurrent_drive.numerator;
            let denominator = cm_scale[state_idx]
                + self.genome.gleak_state[state_idx]
                + sensory_drive.denominator[state_idx]
                + recurrent_drive.denominator;

            next_state[state_idx] = numerator / denominator.max(DENOMINATOR_EPSILON);
        }

        next_state
    }

    fn readout_outputs(&self) -> Vec<f32> {
        let shape = self.genome.shape;
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

    fn network_snapshot(
        &self,
        input: &SensorVector,
        output_values: Vec<f32>,
    ) -> LtcNetworkSnapshot {
        LtcNetworkSnapshot {
            input_values: self.input_buffer(input),
            state_values: self.state.clone(),
            output_values,
            sensory_weights: self.genome.sensory_weight.clone(),
            recurrent_weights: self.genome.recurrent_weight.clone(),
            readout_weights: self.genome.weights_state_output.clone(),
        }
    }

    fn compute_sensory_drive(&self, input: &[f32]) -> SynapseDrive {
        let shape = self.genome.shape;
        let mut numerator = vec![0.0; shape.state_size];
        let mut denominator = vec![0.0; shape.state_size];

        for state_idx in 0..shape.state_size {
            let sensory_row_start = state_idx * shape.input_size;
            for input_idx in 0..shape.input_size {
                let index = sensory_row_start + input_idx;
                let gate = sigmoid(
                    (input[input_idx] - self.genome.sensory_mu[index])
                        * self.genome.sensory_sigma[index],
                );
                let conductance = self.genome.sensory_weight[index] * gate;
                numerator[state_idx] += conductance * self.genome.sensory_erev[index];
                denominator[state_idx] += conductance;
            }
        }

        SynapseDrive {
            numerator,
            denominator,
        }
    }

    fn compute_recurrent_drive(&self, previous_state: &[f32], state_idx: usize) -> SynapseDriveScalar {
        let shape = self.genome.shape;
        let recurrent_row_start = state_idx * shape.state_size;
        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for recurrent_idx in 0..shape.state_size {
            let index = recurrent_row_start + recurrent_idx;
            let gate = sigmoid(
                (previous_state[recurrent_idx] - self.genome.recurrent_mu[index])
                    * self.genome.recurrent_sigma[index],
            );
            let conductance = self.genome.recurrent_weight[index] * gate;
            numerator += conductance * self.genome.recurrent_erev[index];
            denominator += conductance;
        }

        SynapseDriveScalar {
            numerator,
            denominator,
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

#[derive(Clone, Debug)]
struct SynapseDrive {
    numerator: Vec<f32>,
    denominator: Vec<f32>,
}

#[derive(Clone, Copy, Debug)]
struct SynapseDriveScalar {
    numerator: f32,
    denominator: f32,
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

    fn inspect_network(&self, input: &SensorVector) -> ControllerNetworkSnapshot {
        ControllerNetworkSnapshot::Ltc(self.network_snapshot(input, self.readout_outputs()))
    }

    fn step_with_diagnostics(&mut self, input: &SensorVector, dt: f32) -> ControllerDiagnostics {
        let outputs = self.forward_outputs(input, dt);
        ControllerDiagnostics {
            output: Self::step_output_from_signals(&outputs),
            network: ControllerNetworkSnapshot::Ltc(self.network_snapshot(input, outputs)),
        }
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

    use crate::brain::{
        ControllerFactory, ControllerKind, ControllerNetworkSnapshot, NeuralController,
    };
    use crate::sim::SensorVector;

    use super::{LtcController, LtcControllerFactory, LtcGenome, LtcGenomeError, LtcShape};

    fn single_state_genome(parameters: [f32; 15]) -> LtcGenome {
        LtcGenome::from_flat_parameters(
            LtcShape {
                input_size: 1,
                state_size: 1,
                output_size: 2,
            },
            &parameters,
        )
        .expect("single-state parameter vector should match LTC shape")
    }

    #[test]
    fn deterministic_state_evolution_matches_known_sequence() {
        let mut controller = LtcController::new(single_state_genome([
            1.0,   // sensory_weight
            0.0,   // sensory_mu
            1.0,   // sensory_sigma
            1.0,   // sensory_erev
            0.001, // recurrent_weight
            0.0,   // recurrent_mu
            1.0,   // recurrent_sigma
            0.0,   // recurrent_erev
            0.0,   // vleak
            1.0,   // cm
            1.0,   // gleak
            1.0, -1.0, // state->output
            0.0, 0.0, // output bias
        ]))
        .expect("known genome must construct controller");

        let action_1 = controller.step(&SensorVector::from_slice(&[0.0]), 1.0);
        let action_2 = controller.step(&SensorVector::from_slice(&[0.0]), 1.0);

        assert!((action_1.steering - 0.2365018).abs() < 1e-5);
        assert!((action_1.throttle - 0.3817491).abs() < 1e-5);
        assert!((action_2.steering - 0.2919969).abs() < 1e-5);
        assert!((action_2.throttle - 0.3540015).abs() < 1e-5);
    }

    #[test]
    fn clamp_stable_ranges_limits_extreme_values() {
        let shape = LtcShape {
            input_size: 1,
            state_size: 1,
            output_size: 1,
        };

        let params = [
            100.0, -99.0, 88.0, -77.0, 66.0, -55.0, 44.0, -33.0, 22.0, 0.001, -11.0, -77.0, 66.0,
        ];
        let mut genome = LtcGenome::from_flat_parameters(shape, &params)
            .expect("parameter vector should match LTC shape");

        genome.clamp_stable_ranges();

        assert_eq!(genome.sensory_weight[0], 3.0);
        assert_eq!(genome.sensory_mu[0], -1.5);
        assert_eq!(genome.sensory_sigma[0], 10.0);
        assert_eq!(genome.sensory_erev[0], -1.0);
        assert_eq!(genome.recurrent_weight[0], 3.0);
        assert_eq!(genome.recurrent_mu[0], -1.5);
        assert_eq!(genome.recurrent_sigma[0], 10.0);
        assert_eq!(genome.recurrent_erev[0], -1.0);
        assert_eq!(genome.vleak_state[0], 1.0);
        assert_eq!(genome.cm_state[0], 0.05);
        assert_eq!(genome.gleak_state[0], 0.001);
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
                field: "flat_parameters",
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
    fn legacy_serialized_genome_deserializes_with_defaults() {
        let legacy_json = r#"{
            "shape": { "input_size": 1, "state_size": 1, "output_size": 2 },
            "weights_input_state": [0.8],
            "weights_state_state": [0.2],
            "bias_state": [0.1],
            "tau_state": [0.7],
            "weights_state_output": [0.4, -0.6],
            "bias_output": [0.0, 0.0]
        }"#;

        let genome: LtcGenome =
            serde_json::from_str(legacy_json).expect("legacy genome should deserialize");
        let mut controller = LtcController::new(genome).expect("legacy genome should upgrade");

        let output = controller.step_with_observability(&SensorVector::from_slice(&[0.5]), 0.1);

        assert!(output.steering_raw.is_finite());
        assert!(output.throttle_applied.is_finite());
    }

    #[test]
    fn step_with_observability_matches_action_mapping() {
        let mut controller = LtcController::new(single_state_genome([
            1.0, 0.0, 1.0, 1.0, 0.001, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, -1.0, 0.0, 0.0,
        ]))
        .expect("genome should construct");

        let output = controller.step_with_observability(&SensorVector::from_slice(&[0.0]), 1.0);

        assert!((output.action.steering - output.steering_raw).abs() < 1e-6);
        assert!((output.action.throttle - output.throttle_applied).abs() < 1e-6);
        assert!((output.steering_raw - 0.2365018).abs() < 1e-5);
        assert!((output.throttle_applied - 0.3817491).abs() < 1e-5);
    }

    #[test]
    fn step_with_diagnostics_exposes_state_snapshot() {
        let mut controller = LtcController::new(single_state_genome([
            1.0, 0.0, 1.0, 1.0, 0.001, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, -1.0, 0.0, 0.0,
        ]))
        .expect("genome should construct");

        let diagnostics = controller.step_with_diagnostics(&SensorVector::from_slice(&[0.0]), 1.0);

        let ControllerNetworkSnapshot::Ltc(network) = diagnostics.network else {
            panic!("expected LTC network snapshot");
        };

        assert_eq!(network.input_values.len(), 1);
        assert_eq!(network.state_values.len(), 1);
        assert_eq!(network.output_values.len(), 2);
        assert_eq!(network.sensory_weights.len(), 1);
        assert_eq!(network.recurrent_weights.len(), 1);
        assert_eq!(network.readout_weights.len(), 2);
    }

    #[test]
    fn zero_input_converges_toward_leak_equilibrium() {
        let mut controller = LtcController::new(single_state_genome([
            0.001, 1.5, 10.0, 0.0, 0.001, 1.5, 10.0, 0.0, 0.25, 1.0, 1.5, 1.0, 0.0, 0.0, 0.0,
        ]))
        .expect("genome should construct");

        for _ in 0..20 {
            controller.step(&SensorVector::from_slice(&[0.0]), 1.0);
        }

        assert!((controller.state[0] - 0.25).abs() < 0.02);
    }

    #[test]
    fn reversal_potential_sign_controls_state_direction() {
        let positive = single_state_genome([
            3.0, 0.0, 10.0, 1.0, 0.001, 1.5, 10.0, 0.0, 0.0, 1.0, 0.001, 1.0, 0.0, 0.0, 0.0,
        ]);
        let negative = single_state_genome([
            3.0, 0.0, 10.0, -1.0, 0.001, 1.5, 10.0, 0.0, 0.0, 1.0, 0.001, 1.0, 0.0, 0.0, 0.0,
        ]);

        let mut positive_controller = LtcController::new(positive).expect("genome should construct");
        let mut negative_controller = LtcController::new(negative).expect("genome should construct");

        positive_controller.step(&SensorVector::from_slice(&[1.0]), 1.0);
        negative_controller.step(&SensorVector::from_slice(&[1.0]), 1.0);

        assert!(positive_controller.state[0] > 0.0);
        assert!(negative_controller.state[0] < 0.0);
    }

    #[test]
    fn split_timestep_tracks_single_step_closely() {
        let genome = single_state_genome([
            1.2, 0.1, 6.0, 1.0, 0.6, -0.1, 5.0, -0.5, 0.05, 0.8, 0.7, 1.0, 0.0, 0.0, 0.0,
        ]);
        let mut one_step = LtcController::new(genome.clone()).expect("genome should construct");
        let mut two_half_steps = LtcController::new(genome).expect("genome should construct");
        let input = SensorVector::from_slice(&[0.35]);

        one_step.step(&input, 0.5);
        two_half_steps.step(&input, 0.25);
        two_half_steps.step(&input, 0.25);

        assert!((one_step.state[0] - two_half_steps.state[0]).abs() < 0.03);
    }
}
