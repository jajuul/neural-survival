use rand::{Rng, rngs::StdRng};

use crate::brain::{BrainGenome, ControllerKind, LtcGenome, MlpGenome};
use crate::config::{MAX_PERCEPTION_CHANNELS, SimulationConfig};

use super::genome::{AgentGenome, TraitGenome};

const MAX_SPEED_BOUNDS: (f32, f32) = (1.0, 12.0);
const PERCEPTION_RADIUS_BOUNDS: (f32, f32) = (2.0, 40.0);
const PERCEPTION_CHANNELS_BOUNDS: (u16, u16) = (1, MAX_PERCEPTION_CHANNELS);
const METABOLIC_EFFICIENCY_BOUNDS: (f32, f32) = (0.5, 2.0);

/// Build trait defaults from the simulation config.
pub fn default_traits(config: &SimulationConfig) -> TraitGenome {
    TraitGenome {
        max_speed: config.default_agent_max_speed,
        perception_radius: config.default_perception_radius,
        perception_channels: config.default_perception_channels,
        metabolic_efficiency: config.default_metabolic_efficiency,
        food_preference_mask: 1,
    }
}

/// Crossover two parent genomes into one child genome.
///
/// Inputs: parent genomes and deterministic RNG.
/// Output: child genome with mixed brain/trait genes.
/// Side effects: none.
pub fn crossover(parent_a: &AgentGenome, parent_b: &AgentGenome, rng: &mut StdRng) -> AgentGenome {
    assert_eq!(
        parent_a.controller_kind, parent_b.controller_kind,
        "crossover requires matching controller kinds"
    );

    let traits = TraitGenome {
        max_speed: if rng.gen_bool(0.5) {
            parent_a.traits.max_speed
        } else {
            parent_b.traits.max_speed
        },
        perception_radius: if rng.gen_bool(0.5) {
            parent_a.traits.perception_radius
        } else {
            parent_b.traits.perception_radius
        },
        perception_channels: if rng.gen_bool(0.5) {
            parent_a.traits.perception_channels
        } else {
            parent_b.traits.perception_channels
        },
        metabolic_efficiency: if rng.gen_bool(0.5) {
            parent_a.traits.metabolic_efficiency
        } else {
            parent_b.traits.metabolic_efficiency
        },
        food_preference_mask: if rng.gen_bool(0.5) {
            parent_a.traits.food_preference_mask
        } else {
            parent_b.traits.food_preference_mask
        },
    };

    let brain = crossover_brain(&parent_a.brain, &parent_b.brain, rng);
    let controller_kind = match &brain {
        BrainGenome::Mlp(_) => ControllerKind::Mlp,
        BrainGenome::Ltc(_) => ControllerKind::Ltc,
    };

    AgentGenome {
        controller_kind,
        brain,
        traits,
    }
}

/// Mutate one genome in-place using config mutation settings.
///
/// Inputs: mutable genome, deterministic RNG, and config mutation rate.
/// Output: none.
/// Side effects: mutates genes in place.
pub fn mutate(genome: &mut AgentGenome, rng: &mut StdRng, config: &SimulationConfig) {
    let rate = config.mutation_rate as f64;

    if rng.gen_bool(rate) {
        genome.traits.max_speed = (genome.traits.max_speed + rng.gen_range(-1.0..=1.0))
            .clamp(MAX_SPEED_BOUNDS.0, MAX_SPEED_BOUNDS.1);
    }

    if rng.gen_bool(rate) {
        genome.traits.perception_radius = (genome.traits.perception_radius
            + rng.gen_range(-2.0..=2.0))
        .clamp(PERCEPTION_RADIUS_BOUNDS.0, PERCEPTION_RADIUS_BOUNDS.1);
    }

    if rng.gen_bool(rate) {
        let delta: i32 = rng.gen_range(-2..=2);
        let next = genome.traits.perception_channels as i32 + delta;
        genome.traits.perception_channels = next.clamp(
            PERCEPTION_CHANNELS_BOUNDS.0 as i32,
            PERCEPTION_CHANNELS_BOUNDS.1 as i32,
        ) as u16;
    }

    if rng.gen_bool(rate) {
        genome.traits.metabolic_efficiency = (genome.traits.metabolic_efficiency
            + rng.gen_range(-0.15..=0.15))
        .clamp(METABOLIC_EFFICIENCY_BOUNDS.0, METABOLIC_EFFICIENCY_BOUNDS.1);
    }

    if rng.gen_bool(rate) {
        let bit_index = rng.gen_range(0..8);
        genome.traits.food_preference_mask ^= 1 << bit_index;
    }

    mutate_brain(&mut genome.brain, rng, config.mutation_rate);
    genome.controller_kind = match &genome.brain {
        BrainGenome::Mlp(_) => ControllerKind::Mlp,
        BrainGenome::Ltc(_) => ControllerKind::Ltc,
    };
}

fn crossover_brain(
    parent_a: &BrainGenome,
    parent_b: &BrainGenome,
    rng: &mut StdRng,
) -> BrainGenome {
    match (parent_a, parent_b) {
        (BrainGenome::Mlp(a), BrainGenome::Mlp(b)) => {
            BrainGenome::Mlp(crossover_flat_mlp(a, b, rng))
        }
        (BrainGenome::Ltc(a), BrainGenome::Ltc(b)) => {
            BrainGenome::Ltc(crossover_flat_ltc(a, b, rng))
        }
        _ => panic!("crossover_brain requires matching controller families"),
    }
}

fn crossover_flat_mlp(parent_a: &MlpGenome, parent_b: &MlpGenome, rng: &mut StdRng) -> MlpGenome {
    let shape = parent_a.shape;
    let a = parent_a.to_flat_parameters();
    let b = parent_b.to_flat_parameters();

    let child = a
        .iter()
        .zip(b.iter())
        .map(|(va, vb)| if rng.gen_bool(0.5) { *va } else { *vb })
        .collect::<Vec<_>>();

    MlpGenome::from_flat_parameters(shape, &child)
        .expect("MLP crossover must preserve valid parameter vector length")
}

fn crossover_flat_ltc(parent_a: &LtcGenome, parent_b: &LtcGenome, rng: &mut StdRng) -> LtcGenome {
    let shape = parent_a.shape;
    let a = parent_a.to_flat_parameters();
    let b = parent_b.to_flat_parameters();

    let child = a
        .iter()
        .zip(b.iter())
        .map(|(va, vb)| if rng.gen_bool(0.5) { *va } else { *vb })
        .collect::<Vec<_>>();

    let mut genome = LtcGenome::from_flat_parameters(shape, &child)
        .expect("LTC crossover must preserve valid parameter vector length");
    genome.clamp_stable_ranges();
    genome
}

fn mutate_brain(brain: &mut BrainGenome, rng: &mut StdRng, mutation_rate: f32) {
    let rate = mutation_rate as f64;

    match brain {
        BrainGenome::Mlp(genome) => {
            let shape = genome.shape;
            let mut params = genome.to_flat_parameters();
            for value in &mut params {
                if rng.gen_bool(rate) {
                    *value += rng.gen_range(-0.25..=0.25);
                }
            }
            *genome = MlpGenome::from_flat_parameters(shape, &params)
                .expect("MLP mutation must preserve vector length");
        }
        BrainGenome::Ltc(genome) => {
            let shape = genome.shape;
            let mut params = genome.to_flat_parameters();
            for value in &mut params {
                if rng.gen_bool(rate) {
                    *value += rng.gen_range(-0.25..=0.25);
                }
            }
            *genome = LtcGenome::from_flat_parameters(shape, &params)
                .expect("LTC mutation must preserve vector length");
            genome.clamp_stable_ranges();
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};

    use crate::brain::{BrainGenome, ControllerKind, MlpGenome, MlpShape};
    use crate::config::SimulationConfig;

    use super::{MAX_SPEED_BOUNDS, METABOLIC_EFFICIENCY_BOUNDS, crossover, mutate};
    use super::{PERCEPTION_CHANNELS_BOUNDS, PERCEPTION_RADIUS_BOUNDS, default_traits};
    use crate::genetics::genome::AgentGenome;

    fn test_mlp_genome(fill: f32) -> BrainGenome {
        BrainGenome::Mlp(
            MlpGenome::from_flat_parameters(
                MlpShape {
                    input_size: 4,
                    hidden_size: 3,
                    output_size: 2,
                },
                &vec![
                    fill;
                    MlpShape {
                        input_size: 4,
                        hidden_size: 3,
                        output_size: 2,
                    }
                    .parameter_count()
                ],
            )
            .expect("test MLP shape should be valid"),
        )
    }

    #[test]
    fn mutation_keeps_traits_within_bounds() {
        let config = SimulationConfig {
            mutation_rate: 1.0,
            ..SimulationConfig::default()
        };
        let mut rng = StdRng::seed_from_u64(99);

        let mut genome = AgentGenome {
            controller_kind: ControllerKind::Mlp,
            brain: test_mlp_genome(0.1),
            traits: default_traits(&config),
        };

        for _ in 0..64 {
            mutate(&mut genome, &mut rng, &config);
            assert!(
                genome.traits.max_speed >= MAX_SPEED_BOUNDS.0
                    && genome.traits.max_speed <= MAX_SPEED_BOUNDS.1
            );
            assert!(
                genome.traits.perception_radius >= PERCEPTION_RADIUS_BOUNDS.0
                    && genome.traits.perception_radius <= PERCEPTION_RADIUS_BOUNDS.1
            );
            assert!(
                genome.traits.perception_channels >= PERCEPTION_CHANNELS_BOUNDS.0
                    && genome.traits.perception_channels <= PERCEPTION_CHANNELS_BOUNDS.1
            );
            assert!(
                genome.traits.metabolic_efficiency >= METABOLIC_EFFICIENCY_BOUNDS.0
                    && genome.traits.metabolic_efficiency <= METABOLIC_EFFICIENCY_BOUNDS.1
            );
        }
    }

    #[test]
    fn crossover_preserves_controller_kind_for_same_family() {
        let mut rng = StdRng::seed_from_u64(5);
        let config = SimulationConfig::default();

        let parent_a = AgentGenome {
            controller_kind: ControllerKind::Mlp,
            brain: test_mlp_genome(0.2),
            traits: default_traits(&config),
        };
        let parent_b = AgentGenome {
            controller_kind: ControllerKind::Mlp,
            brain: test_mlp_genome(-0.2),
            traits: default_traits(&config),
        };

        let child = crossover(&parent_a, &parent_b, &mut rng);
        assert_eq!(child.controller_kind, ControllerKind::Mlp);
    }

    #[test]
    #[should_panic(expected = "crossover requires matching controller kinds")]
    fn crossover_panics_for_mixed_controller_kinds() {
        let mut rng = StdRng::seed_from_u64(11);
        let config = SimulationConfig::default();

        let parent_mlp = AgentGenome {
            controller_kind: ControllerKind::Mlp,
            brain: test_mlp_genome(0.2),
            traits: default_traits(&config),
        };

        let ltc_shape = crate::brain::LtcShape {
            input_size: 4,
            state_size: 3,
            output_size: 2,
        };
        let parent_ltc = AgentGenome {
            controller_kind: ControllerKind::Ltc,
            brain: BrainGenome::Ltc(
                crate::brain::LtcGenome::from_flat_parameters(
                    ltc_shape,
                    &vec![0.0; ltc_shape.parameter_count()],
                )
                .expect("valid LTC test genome"),
            ),
            traits: default_traits(&config),
        };

        let _ = crossover(&parent_mlp, &parent_ltc, &mut rng);
    }
}
