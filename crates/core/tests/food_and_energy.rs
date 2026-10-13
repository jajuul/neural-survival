mod common;

use common::{assert_close, constant_prey, tiny_config};
use neural_survival_core::{Config, Placement, Showcase, Vec2};

const STILL: f64 = -1.0 + 1e-12;

fn still_prey_with_food(config: &Config, at: Vec2, food: Vec<Vec2>) -> Showcase {
    Showcase::builder(config, 11)
        .genomes(vec![constant_prey(config, 0.0, STILL)])
        .placements(vec![Placement {
            position: at,
            heading: 0.0,
        }])
        .food(food)
        .build()
        .unwrap()
}

#[test]
fn prey_eat_touching_food_and_gain_its_energy() {
    let mut config = tiny_config();
    config.agent.body_radius = 2.0;
    config.food.radius = 1.0;
    config.food.energy = 30.0;
    config.prey.starting_energy = 50.0;
    config.energy.idle_drain = 0.5;
    let mut showcase =
        still_prey_with_food(&config, Vec2::new(10.0, 10.0), vec![Vec2::new(12.5, 10.0)]);

    showcase.step();
    assert_close(showcase.arenas()[0].agents()[0].energy, 50.0 + 30.0 - 0.5);
}

#[test]
fn eaten_food_respawns_immediately_elsewhere() {
    let config = tiny_config();
    let eaten = Vec2::new(10.5, 10.0);
    let far = Vec2::new(30.0, 30.0);
    let mut showcase = still_prey_with_food(&config, Vec2::new(10.0, 10.0), vec![eaten, far]);

    showcase.step();
    let food = showcase.arenas()[0].food();
    assert_eq!(food.len(), 2);
    assert_ne!(food[0].position, eaten);
    assert_eq!(food[1].position, far);
    for item in food {
        assert!((0.0..50.0).contains(&item.position.x) && (0.0..40.0).contains(&item.position.y));
    }
}

#[test]
fn food_out_of_contact_is_not_eaten() {
    let mut config = tiny_config();
    config.agent.body_radius = 2.0;
    config.food.radius = 1.0;
    config.prey.starting_energy = 50.0;
    config.energy.idle_drain = 0.5;
    let food = Vec2::new(13.1, 10.0);
    let mut showcase = still_prey_with_food(&config, Vec2::new(10.0, 10.0), vec![food]);

    showcase.step();
    assert_close(showcase.arenas()[0].agents()[0].energy, 49.5);
    assert_eq!(showcase.arenas()[0].food()[0].position, food);
}

#[test]
fn prey_eat_food_across_the_torus_edge() {
    let mut config = tiny_config();
    config.prey.starting_energy = 50.0;
    config.food.energy = 30.0;
    config.energy.idle_drain = 0.5;
    let mut showcase =
        still_prey_with_food(&config, Vec2::new(0.5, 20.0), vec![Vec2::new(49.5, 20.0)]);

    showcase.step();
    assert_close(showcase.arenas()[0].agents()[0].energy, 79.5);
}

#[test]
fn idle_drain_reduces_energy_each_step() {
    let mut config = tiny_config();
    config.prey.starting_energy = 10.0;
    config.energy.idle_drain = 0.25;
    let mut showcase = still_prey_with_food(&config, Vec2::new(10.0, 10.0), vec![]);

    for _ in 0..4 {
        showcase.step();
    }
    assert_close(showcase.arenas()[0].agents()[0].energy, 9.0);
}

#[test]
fn agent_at_zero_energy_dies_and_stays_dead() {
    let mut config = tiny_config();
    config.prey.starting_energy = 1.0;
    config.energy.idle_drain = 0.4;
    let genomes = vec![
        constant_prey(&config, 0.2, 0.5),
        constant_prey(&config, 0.0, STILL),
    ];
    let mut showcase = Showcase::builder(&config, 5)
        .genomes(genomes)
        .placements(vec![
            Placement {
                position: Vec2::new(10.0, 10.0),
                heading: 0.0,
            },
            Placement {
                position: Vec2::new(30.0, 30.0),
                heading: 0.0,
            },
        ])
        // Food right under the second Prey keeps it fed.
        .food(vec![Vec2::new(30.0, 30.0)])
        .build()
        .unwrap();

    showcase.step();
    showcase.step();
    assert!(showcase.arenas()[0].agents()[0].alive);
    showcase.step();
    let dead = showcase.arenas()[0].agents()[0].clone();
    assert!(!dead.alive);
    assert_eq!(dead.energy, 0.0);

    for _ in 0..5 {
        showcase.step();
    }
    assert_eq!(showcase.arenas()[0].agents()[0], dead);
    assert!(!showcase.has_ended());
}

#[test]
fn showcase_ends_when_every_agent_is_dead() {
    let mut config = tiny_config();
    config.prey.starting_energy = 1.0;
    config.energy.idle_drain = 0.5;
    let mut showcase = still_prey_with_food(&config, Vec2::new(10.0, 10.0), vec![]);

    showcase.step();
    assert!(!showcase.has_ended());
    showcase.step();
    assert!(showcase.has_ended());
    assert_eq!(showcase.step_count(), 2);
}
