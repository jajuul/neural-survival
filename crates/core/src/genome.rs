//! Genomes: the heritable unit, fixed to one Role and Controller Family.

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::mlp::Mlp;
use crate::rng::Rng;

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
