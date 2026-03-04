use rand::{SeedableRng, rngs::StdRng};

use neural_survival::brain::{ControllerFactory, MlpControllerFactory, MlpShape, NeuralController};
use neural_survival::config::SimulationConfig;
use neural_survival::sim::{
    ActionVector, basic_sensor_vector, initialize_world, step_world_with_actions, summarize_world,
};

fn run_mlp_episode(config: &SimulationConfig, seed: u64) -> neural_survival::sim::EpisodeSummary {
    let mut world = initialize_world(config, seed);
    let mut rng = StdRng::seed_from_u64(seed ^ 0xA5A5_5A5A);

    let factory = MlpControllerFactory::new(MlpShape {
        input_size: 4,
        hidden_size: 8,
        output_size: 2,
    });

    let mut controllers: Vec<Box<dyn NeuralController>> = (0..world.agents.len())
        .map(|_| {
            let genome = factory.random_genome(&mut rng);
            factory.build(&genome)
        })
        .collect();

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

    summarize_world(&world)
}

#[test]
fn mlp_driven_episode_is_repeatable_for_same_seed() {
    let config = SimulationConfig {
        episode_steps: 80,
        population_size: 20,
        food_count: 80,
        ..SimulationConfig::default()
    };

    let summary_a = run_mlp_episode(&config, 777);
    let summary_b = run_mlp_episode(&config, 777);

    assert_eq!(summary_a, summary_b);
}

#[test]
fn mlp_driven_episode_runs_without_panics() {
    let config = SimulationConfig {
        episode_steps: 20,
        population_size: 8,
        food_count: 20,
        ..SimulationConfig::default()
    };

    let summary = run_mlp_episode(&config, 11);

    assert_eq!(summary.step_count, 20);
    assert!(summary.alive_agents <= config.population_size);
}
