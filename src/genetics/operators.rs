use rand::{Rng, rngs::StdRng};

use crate::brain::{BrainGenome, ControllerKind, LtcGenome, MlpGenome};
use crate::config::{MAX_PERCEPTION_CHANNELS, SimulationConfig};
use crate::sim::AgentRole;

use super::genome::{AgentGenome, TraitGenome};

const MAX_SPEED_BOUNDS: (f32, f32) = (1.0, 12.0);
const PERCEPTION_RADIUS_BOUNDS: (f32, f32) = (2.0, 40.0);
const PERCEPTION_CHANNELS_BOUNDS: (u16, u16) = (1, MAX_PERCEPTION_CHANNELS);
const EYE_CELL_ANGLE_BOUNDS: (f32, f32) = (0.05, std::f32::consts::PI);
const METABOLIC_EFFICIENCY_BOUNDS: (f32, f32) = (0.5, 2.0);

/// Build trait defaults from the simulation config.
pub fn default_traits(config: &SimulationConfig) -> TraitGenome {
    TraitGenome {
        max_speed: config.default_agent_max_speed,
        perception_radius: config.default_perception_radius,
        perception_channels: config.default_perception_channels,
        eye_cell_angle_radians: (config.eye_fov_angle_radians
            / config.default_perception_channels.max(1) as f32)
            .clamp(EYE_CELL_ANGLE_BOUNDS.0, EYE_CELL_ANGLE_BOUNDS.1),
        metabolic_efficiency: config.default_metabolic_efficiency,
        role: AgentRole::Prey,
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
    assert_eq!(
        parent_a.traits.role, parent_b.traits.role,
        "crossover requires matching ecological roles"
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
        eye_cell_angle_radians: if rng.gen_bool(0.5) {
            parent_a.traits.eye_cell_angle_radians
        } else {
            parent_b.traits.eye_cell_angle_radians
        },
        metabolic_efficiency: if rng.gen_bool(0.5) {
            parent_a.traits.metabolic_efficiency
        } else {
            parent_b.traits.metabolic_efficiency
        },
        role: parent_a.traits.role,
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
        genome.traits.eye_cell_angle_radians = (genome.traits.eye_cell_angle_radians
            + rng.gen_range(-0.2..=0.2))
        .clamp(EYE_CELL_ANGLE_BOUNDS.0, EYE_CELL_ANGLE_BOUNDS.1);
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
            mutate_ltc_genome(genome, rng, rate);
        }
    }
}

fn mutate_ltc_genome(genome: &mut LtcGenome, rng: &mut StdRng, rate: f64) {
    mutate_values(&mut genome.sensory_weight, rng, rate, 0.12);
    mutate_values(&mut genome.sensory_mu, rng, rate, 0.2);
    mutate_values(&mut genome.sensory_sigma, rng, rate, 0.8);
    mutate_values(&mut genome.sensory_erev, rng, rate, 0.18);
    mutate_values(&mut genome.recurrent_weight, rng, rate, 0.12);
    mutate_values(&mut genome.recurrent_mu, rng, rate, 0.2);
    mutate_values(&mut genome.recurrent_sigma, rng, rate, 0.8);
    mutate_values(&mut genome.recurrent_erev, rng, rate, 0.18);
    mutate_values(&mut genome.vleak_state, rng, rate, 0.12);
    mutate_values(&mut genome.cm_state, rng, rate, 0.1);
    mutate_values(&mut genome.gleak_state, rng, rate, 0.1);
    mutate_values(&mut genome.weights_state_output, rng, rate, 0.25);
    mutate_values(&mut genome.bias_output, rng, rate, 0.2);
    genome.clamp_stable_ranges();
}

fn mutate_values(values: &mut [f32], rng: &mut StdRng, rate: f64, amplitude: f32) {
    for value in values {
        if rng.gen_bool(rate) {
            *value += rng.gen_range(-amplitude..=amplitude);
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};

    use crate::brain::{BrainGenome, ControllerKind, LtcGenome, LtcShape, MlpGenome, MlpShape};
    use crate::config::SimulationConfig;
    use crate::sim::AgentRole;

    use super::{
        EYE_CELL_ANGLE_BOUNDS, MAX_SPEED_BOUNDS, METABOLIC_EFFICIENCY_BOUNDS,
        PERCEPTION_CHANNELS_BOUNDS, PERCEPTION_RADIUS_BOUNDS, crossover, default_traits, mutate,
    };
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

    fn test_ltc_genome() -> BrainGenome {
        let mut rng = StdRng::seed_from_u64(17);
        BrainGenome::Ltc(LtcGenome::random(
            LtcShape {
                input_size: 4,
                state_size: 3,
                output_size: 2,
            },
            &mut rng,
        ))
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
        genome.traits.role = AgentRole::Predator;

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
                genome.traits.eye_cell_angle_radians >= EYE_CELL_ANGLE_BOUNDS.0
                    && genome.traits.eye_cell_angle_radians <= EYE_CELL_ANGLE_BOUNDS.1
            );
            assert!(
                genome.traits.metabolic_efficiency >= METABOLIC_EFFICIENCY_BOUNDS.0
                    && genome.traits.metabolic_efficiency <= METABOLIC_EFFICIENCY_BOUNDS.1
            );
        }
    }

    #[test]
    fn mutation_keeps_ltc_parameters_within_semantic_ranges() {
        let config = SimulationConfig {
            mutation_rate: 1.0,
            ..SimulationConfig::default()
        };
        let mut rng = StdRng::seed_from_u64(101);

        let mut genome = AgentGenome {
            controller_kind: ControllerKind::Ltc,
            brain: test_ltc_genome(),
            traits: default_traits(&config),
        };

        for _ in 0..64 {
            mutate(&mut genome, &mut rng, &config);

            let BrainGenome::Ltc(ltc) = &genome.brain else {
                panic!("expected LTC genome after mutation");
            };

            assert!(ltc.sensory_weight.iter().all(|value| *value >= 0.001));
            assert!(ltc.recurrent_weight.iter().all(|value| *value >= 0.001));
            assert!(ltc.sensory_sigma.iter().all(|value| *value >= 0.1));
            assert!(ltc.recurrent_sigma.iter().all(|value| *value >= 0.1));
            assert!(
                ltc.sensory_erev
                    .iter()
                    .all(|value| (-1.0..=1.0).contains(value))
            );
            assert!(
                ltc.recurrent_erev
                    .iter()
                    .all(|value| (-1.0..=1.0).contains(value))
            );
            assert!(
                ltc.vleak_state
                    .iter()
                    .all(|value| (-1.0..=1.0).contains(value))
            );
            assert!(ltc.cm_state.iter().all(|value| *value >= 0.05));
            assert!(ltc.gleak_state.iter().all(|value| *value >= 0.001));
        }
    }

    #[test]
    fn crossover_preserves_controller_kind_for_same_family() {
        let mut rng = StdRng::seed_from_u64(5);
        let config = SimulationConfig::default();

        let mut parent_a = AgentGenome {
            controller_kind: ControllerKind::Mlp,
            brain: test_mlp_genome(0.2),
            traits: default_traits(&config),
        };
        parent_a.traits.role = AgentRole::Predator;
        let mut parent_b = AgentGenome {
            controller_kind: ControllerKind::Mlp,
            brain: test_mlp_genome(-0.2),
            traits: default_traits(&config),
        };
        parent_b.traits.role = AgentRole::Predator;

        let child = crossover(&parent_a, &parent_b, &mut rng);
        assert_eq!(child.controller_kind, ControllerKind::Mlp);
        assert_eq!(child.traits.role, AgentRole::Predator);
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
