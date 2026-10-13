//! Genomes: the heritable unit, fixed to one Role and Controller Family.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::Config;
use crate::mlp::Mlp;
use crate::rng::{Rng, Stream, stream_rng};
use crate::species::Species;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Prey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControllerFamily {
    Mlp,
}

/// A Genome's network parameters, one variant per Controller Family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Network {
    Mlp(Mlp),
}

impl Network {
    pub fn family(&self) -> ControllerFamily {
        match self {
            Network::Mlp(_) => ControllerFamily::Mlp,
        }
    }

    pub fn input_width(&self) -> usize {
        match self {
            Network::Mlp(mlp) => mlp.input_width(),
        }
    }

    pub fn output_width(&self) -> usize {
        match self {
            Network::Mlp(mlp) => mlp.output_width(),
        }
    }

    /// Raw outputs `[steering, throttle]`, each in [-1, 1].
    pub(crate) fn act(&self, input: &[f64]) -> [f64; 2] {
        match self {
            Network::Mlp(mlp) => {
                let out = mlp.forward(input);
                [out[0], out[1]]
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Genome {
    pub role: Role,
    pub network: Network,
}

impl Genome {
    pub fn new(role: Role, network: Network) -> Self {
        Self { role, network }
    }

    pub fn family(&self) -> ControllerFamily {
        self.network.family()
    }

    /// An offspring by uniform crossover of two Genomes of the same Species.
    pub(crate) fn crossover(&self, other: &Genome, rng: &mut Rng) -> Genome {
        debug_assert_eq!(self.role, other.role);
        let network = match (&self.network, &other.network) {
            (Network::Mlp(a), Network::Mlp(b)) => Network::Mlp(a.crossover(b, rng)),
        };
        Genome {
            role: self.role,
            network,
        }
    }

    pub(crate) fn mutate(&mut self, rate: f64, sigma: f64, rng: &mut Rng) {
        match &mut self.network {
            Network::Mlp(mlp) => mlp.mutate(rate, sigma, rng),
        }
    }

    /// A Generation-0 Genome with a freshly initialised network.
    pub(crate) fn random(
        role: Role,
        family: ControllerFamily,
        config: &Config,
        rng: &mut Rng,
    ) -> Self {
        let network = match family {
            ControllerFamily::Mlp => {
                let mut sizes = vec![config.controller_input_width()];
                sizes.extend(&config.mlp.hidden_layers);
                sizes.push(CONTROLLER_OUTPUTS);
                Network::Mlp(Mlp::random(sizes, rng))
            }
        };
        Self { role, network }
    }
}

/// Every controller outputs steering and throttle.
pub const CONTROLLER_OUTPUTS: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error(
    "genome {index} has a {inputs}-input, {outputs}-output network; \
     the config needs {expected_inputs} inputs and {CONTROLLER_OUTPUTS} outputs"
)]
pub struct NetworkShapeError {
    pub index: usize,
    pub inputs: usize,
    pub outputs: usize,
    pub expected_inputs: usize,
}

/// Every Genome's network must fit the config's controller input and output widths.
pub(crate) fn check_network_shapes(
    config: &Config,
    genomes: &[Genome],
) -> Result<(), NetworkShapeError> {
    let expected_inputs = config.controller_input_width();
    for (index, genome) in genomes.iter().enumerate() {
        let (inputs, outputs) = (genome.network.input_width(), genome.network.output_width());
        if inputs != expected_inputs || outputs != CONTROLLER_OUTPUTS {
            return Err(NetworkShapeError {
                index,
                inputs,
                outputs,
                expected_inputs,
            });
        }
    }
    Ok(())
}

/// Randomly initialised Generation-0 Genomes for every Species, in
/// `Species::ALL` order, each Species drawing from its own stream.
pub(crate) fn random_genomes(config: &Config, seed: u64) -> Vec<Genome> {
    Species::ALL
        .iter()
        .enumerate()
        .flat_map(|(index, &species)| {
            let mut rng = stream_rng(seed, Stream::Genomes, index as u64);
            (0..species.population(config))
                .map(|_| Genome::random(species.role, species.family, config, &mut rng))
                .collect::<Vec<_>>()
        })
        .collect()
}
