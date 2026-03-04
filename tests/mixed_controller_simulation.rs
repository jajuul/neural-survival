use rand::{SeedableRng, rngs::StdRng};

use neural_survival::brain::{
    ControllerFactory, ControllerKind, LtcControllerFactory, LtcShape, MixedControllerFactory,
    MlpControllerFactory, MlpShape, NeuralController,
};
use neural_survival::config::SimulationConfig;
use neural_survival::sim::{
    ActionVector, basic_sensor_vector, initialize_world, step_world_with_actions, summarize_world,
};

#[derive(Debug, PartialEq)]
struct RunSnapshot {
    summary: neural_survival::sim::EpisodeSummary,
    mlp_count: usize,
    ltc_count: usize,
}

fn run_mixed_episode(config: &SimulationConfig, seed: u64) -> RunSnapshot {
    let mut world = initialize_world(config, seed);
    let mut rng = StdRng::seed_from_u64(seed ^ 0x55AA_F00D);

    let factory = MixedControllerFactory::new(
        MlpControllerFactory::new(MlpShape {
            input_size: 4,
            hidden_size: 8,
            output_size: 2,
        }),
        LtcControllerFactory::new(LtcShape {
            input_size: 4,
            state_size: 8,
            output_size: 2,
        }),
        0.5,
    );

    let genomes = (0..world.agents.len())
        .map(|_| factory.random_genome(&mut rng))
        .collect::<Vec<_>>();

    let mut controllers: Vec<Box<dyn NeuralController>> =
        genomes.iter().map(|genome| factory.build(genome)).collect();

    let (mlp_count, ltc_count) = controllers.iter().fold((0, 0), |(mlp, ltc), controller| {
        match controller.controller_kind() {
            ControllerKind::Mlp => (mlp + 1, ltc),
            ControllerKind::Ltc => (mlp, ltc + 1),
        }
    });

    for _ in 0..config.episode_steps {
        let actions = world
            .agents
            .iter()
            .enumerate()
            .map(|(idx, agent)| {
                if !agent.alive {
                    return ActionVector::default();
                }

                let sensor = basic_sensor_vector(agent, &world, config);
                controllers[idx].step(&sensor, config.timestep_seconds)
            })
            .collect::<Vec<_>>();

        step_world_with_actions(&mut world, config, &actions);
    }

    RunSnapshot {
        summary: summarize_world(&world),
        mlp_count,
        ltc_count,
    }
}

#[test]
fn mixed_mlp_ltc_episode_is_repeatable_for_same_seed() {
    let config = SimulationConfig {
        episode_steps: 60,
        population_size: 24,
        food_count: 90,
        ..SimulationConfig::default()
    };

    let run_a = run_mixed_episode(&config, 2024);
    let run_b = run_mixed_episode(&config, 2024);

    assert_eq!(run_a, run_b);
    assert!(run_a.mlp_count > 0);
    assert!(run_a.ltc_count > 0);
}
