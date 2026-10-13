use neural_survival_core::{AgentId, Config, Showcase};

fn busy_config() -> Config {
    let mut config = Config::default();
    config.world.width = 60.0;
    config.world.height = 60.0;
    config.population.mlp_prey = 8;
    config.food.count = 30;
    config.evaluation.episode_steps = 400;
    config
}

#[test]
fn same_seed_and_config_give_identical_state_after_n_steps() {
    let config = busy_config();
    let mut a = Showcase::new(&config, 1234).unwrap();
    let mut b = Showcase::new(&config, 1234).unwrap();
    for _ in 0..200 {
        a.step();
        b.step();
    }
    assert_eq!(a.arenas(), b.arenas());
    for id in 0..8 {
        assert_eq!(a.telemetry(AgentId(id)), b.telemetry(AgentId(id)));
    }
}

#[test]
fn stepping_one_at_a_time_matches_stepping_in_batches() {
    let config = busy_config();
    let mut single = Showcase::new(&config, 99).unwrap();
    let mut batched = Showcase::new(&config, 99).unwrap();
    for _ in 0..250 {
        single.step();
    }
    batched.step_many(1);
    batched.step_many(100);
    batched.step_many(149);
    assert_eq!(batched.step_count(), 250);
    assert_eq!(single, batched);
}

#[test]
fn step_many_stops_at_episode_end() {
    let config = busy_config();
    let mut showcase = Showcase::new(&config, 99).unwrap();
    showcase.step_many(10_000);
    assert!(showcase.has_ended());
    assert!(showcase.step_count() <= 400);
}

#[test]
fn different_seeds_give_different_worlds() {
    let config = busy_config();
    let a = Showcase::new(&config, 1).unwrap();
    let b = Showcase::new(&config, 2).unwrap();
    assert_ne!(a.arenas(), b.arenas());
}

#[test]
fn the_world_actually_changes_over_time() {
    // Guards against a trivially "deterministic" world that never moves.
    let config = busy_config();
    let mut showcase = Showcase::new(&config, 3).unwrap();
    let start = showcase.arenas().to_vec();
    showcase.step_many(50);
    assert_ne!(showcase.arenas(), &start[..]);
}
