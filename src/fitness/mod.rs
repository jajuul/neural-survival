//! Fitness score computation utilities.

use crate::config::SimulationConfig;

/// Components contributing to one agent's fitness score.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FitnessComponents {
    pub survival_steps: u32,
    pub food_consumed: u32,
    pub final_energy: f32,
}

/// Compute scalar fitness from weighted episode components.
///
/// Inputs: `components` from one episode and `config` fitness weights.
/// Output: deterministic scalar fitness value.
/// Side effects: none.
pub fn compute_fitness(components: FitnessComponents, config: &SimulationConfig) -> f32 {
    let survival = components.survival_steps as f32;
    let food = components.food_consumed as f32;
    let energy = components.final_energy.max(0.0);

    config.fitness_weight_survival * survival
        + config.fitness_weight_food * food
        + config.fitness_weight_energy * energy
}

#[cfg(test)]
mod tests {
    use crate::config::SimulationConfig;

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
            FitnessComponents {
                survival_steps: 10,
                food_consumed: 2,
                final_energy: 8.0,
            },
            &config,
        );

        assert!((fitness - 18.0).abs() < 1e-6);
    }
}
