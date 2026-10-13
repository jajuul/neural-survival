//! Feed-forward MLP controller with tanh on every layer.

use rand::Rng as _;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::rng::Rng;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NetworkError {
    #[error("an MLP needs at least an input and an output layer, got {0} layer sizes")]
    TooFewLayers(usize),
    #[error("MLP layer sizes must be at least 1")]
    EmptyLayer,
    #[error("MLP with layer sizes {layer_sizes:?} needs {expected} parameters, got {actual}")]
    ParamCount {
        layer_sizes: Vec<usize>,
        expected: usize,
        actual: usize,
    },
}

/// An MLP's shape and parameters. `params` holds, for each layer in turn, its
/// `outputs × inputs` weights in row-major order followed by its `outputs`
/// biases.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mlp {
    layer_sizes: Vec<usize>,
    params: Vec<f64>,
}

impl Mlp {
    pub fn from_params(layer_sizes: Vec<usize>, params: Vec<f64>) -> Result<Self, NetworkError> {
        if layer_sizes.len() < 2 {
            return Err(NetworkError::TooFewLayers(layer_sizes.len()));
        }
        if layer_sizes.contains(&0) {
            return Err(NetworkError::EmptyLayer);
        }
        let expected = param_count(&layer_sizes);
        if params.len() != expected {
            return Err(NetworkError::ParamCount {
                layer_sizes,
                expected,
                actual: params.len(),
            });
        }
        Ok(Self {
            layer_sizes,
            params,
        })
    }

    /// Xavier-uniform weights in `±sqrt(6 / (fan_in + fan_out))`, zero biases.
    pub(crate) fn random(layer_sizes: Vec<usize>, rng: &mut Rng) -> Self {
        let mut params = Vec::with_capacity(param_count(&layer_sizes));
        for pair in layer_sizes.windows(2) {
            let (fan_in, fan_out) = (pair[0], pair[1]);
            let bound = (6.0 / (fan_in + fan_out) as f64).sqrt();
            params.extend((0..fan_in * fan_out).map(|_| rng.gen_range(-bound..=bound)));
            params.extend(std::iter::repeat_n(0.0, fan_out));
        }
        Self {
            layer_sizes,
            params,
        }
    }

    pub fn layer_sizes(&self) -> &[usize] {
        &self.layer_sizes
    }

    pub fn params(&self) -> &[f64] {
        &self.params
    }

    pub fn input_width(&self) -> usize {
        self.layer_sizes[0]
    }

    pub fn output_width(&self) -> usize {
        self.layer_sizes[self.layer_sizes.len() - 1]
    }

    pub(crate) fn forward(&self, input: &[f64]) -> Vec<f64> {
        debug_assert_eq!(input.len(), self.input_width());
        let mut activations = input.to_vec();
        let mut offset = 0;
        for pair in self.layer_sizes.windows(2) {
            let (n_in, n_out) = (pair[0], pair[1]);
            let weights = &self.params[offset..offset + n_in * n_out];
            let biases = &self.params[offset + n_in * n_out..offset + n_in * n_out + n_out];
            activations = (0..n_out)
                .map(|o| {
                    let row = &weights[o * n_in..(o + 1) * n_in];
                    let sum: f64 = row.iter().zip(&activations).map(|(w, x)| w * x).sum();
                    (sum + biases[o]).tanh()
                })
                .collect();
            offset += n_in * n_out + n_out;
        }
        activations
    }
}

fn param_count(layer_sizes: &[usize]) -> usize {
    layer_sizes.windows(2).map(|p| p[0] * p[1] + p[1]).sum()
}
