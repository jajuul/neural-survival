//! Fitness score computation utilities.

use crate::config::SimulationConfig;
use crate::sim::AgentRole;

/// Components contributing to one agent's fitness score.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FitnessComponents {
    pub survival_steps: u32,
    pub food_consumed: u32,
    pub prey_captured: u32,
    pub final_energy: f32,
}

/// Compute scalar fitness from weighted episode components.
///
/// Inputs: `components` from one episode and `config` fitness weights.
/// Output: deterministic scalar fitness value.
/// Side effects: none.
pub fn compute_fitness(
    role: AgentRole,
    components: FitnessComponents,
    config: &SimulationConfig,
) -> f32 {
    let survival = components.survival_steps as f32;
    let food = components.food_consumed as f32;
    let predation = components.prey_captured as f32;
    let energy = components.final_energy.max(0.0);

    let role_reward = match role {
        AgentRole::Prey => config.fitness_weight_food * food,
        AgentRole::Predator => config.fitness_weight_predation * predation,
    };

    config.fitness_weight_survival * survival + role_reward + config.fitness_weight_energy * energy
}

#[cfg(test)]
mod tests {
    use crate::config::SimulationConfig;

    use crate::sim::AgentRole;

    use super::{FitnessComponents, compute_fitness};

    #[test]
    fn compute_fitness_uses_config_weights() {
        let config = SimulationConfig {
            fitness_weight_survival: 1.0,
            fitness_weight_food: 3.0,
            fitness_weight_energy: 0.25,
            ..SimulationConfig::default()
        };

        let fitness = compute_fitness(
            AgentRole::Prey,
            FitnessComponents {
                survival_steps: 10,
                food_consumed: 2,
                prey_captured: 0,
                final_energy: 8.0,
            },
            &config,
        );

        assert!((fitness - 18.0).abs() < 1e-6);
    }

    #[test]
    fn predator_fitness_uses_capture_weight_instead_of_food_weight() {
        let config = SimulationConfig {
            fitness_weight_survival: 1.0,
            fitness_weight_food: 100.0,
            fitness_weight_predation: 3.0,
            fitness_weight_energy: 0.5,
            ..SimulationConfig::default()
        };

        let fitness = compute_fitness(
            AgentRole::Predator,
            FitnessComponents {
                survival_steps: 4,
                food_consumed: 9,
                prey_captured: 2,
                final_energy: 6.0,
            },
            &config,
        );

        assert!((fitness - 13.0).abs() < 1e-6);
    }
}
