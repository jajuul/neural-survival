//! Species: fixed breeding pools, one per Controller Family and Role (ADR-0002).

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::genome::{ControllerFamily, Genome, Role};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Species {
    pub family: ControllerFamily,
    pub role: Role,
}

pub const MLP_PREY: Species = Species {
    family: ControllerFamily::Mlp,
    role: Role::Prey,
};

impl Species {
    /// Every Species, in the fixed order used for Genomes, Agents, and metrics.
    pub const ALL: [Species; 1] = [MLP_PREY];

    pub fn of(genome: &Genome) -> Self {
        Self {
            family: genome.family(),
            role: genome.role,
        }
    }

    /// The Species' fixed population size from the config.
    pub fn population(self, config: &Config) -> usize {
        match (self.family, self.role) {
            (ControllerFamily::Mlp, Role::Prey) => config.population.mlp_prey,
        }
    }
}

impl std::fmt::Display for Species {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let family = match self.family {
            ControllerFamily::Mlp => "MLP",
        };
        let role = match self.role {
            Role::Prey => "Prey",
        };
        write!(f, "{family}-{role}")
    }
}
