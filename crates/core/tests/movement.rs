mod common;

use common::{assert_close, constant_prey, tiny_config};
use neural_survival_core::{Placement, Showcase, Vec2};

fn one_prey(steering: f64, throttle: f64, position: Vec2, heading: f64) -> Showcase {
    let config = tiny_config();
    Showcase::builder(&config, 3)
        .genomes(vec![constant_prey(&config, steering, throttle)])
        .placements(vec![Placement { position, heading }])
        .build()
        .unwrap()
}

#[test]
fn steering_turns_heading_by_turn_rate() {
    let mut showcase = one_prey(0.5, -1.0 + 1e-12, Vec2::new(10.0, 10.0), 1.0);
    showcase.step();
    assert_close(showcase.arenas()[0].agents()[0].heading, 1.0 + 0.5 * 0.3);
    showcase.step();
    assert_close(
        showcase.arenas()[0].agents()[0].heading,
        1.0 + 2.0 * 0.5 * 0.3,
    );
}

#[test]
fn throttle_maps_to_forward_thrust_with_drag() {
    // throttle 0 → thrust 0.5 → +0.25 speed per step, minus 10% drag.
    let mut showcase = one_prey(0.0, 0.0, Vec2::new(10.0, 10.0), 0.0);
    showcase.step();
    let agent = &showcase.arenas()[0].agents()[0];
    assert_close(agent.speed, 0.25);
    assert_close(agent.position.x, 10.25);
    assert_close(agent.position.y, 10.0);

    showcase.step();
    let agent = &showcase.arenas()[0].agents()[0];
    assert_close(agent.speed, 0.475);
    assert_close(agent.position.x, 10.725);
}

#[test]
fn full_reverse_throttle_gives_no_thrust() {
    let mut showcase = one_prey(0.0, -1.0 + 1e-12, Vec2::new(10.0, 10.0), 0.0);
    for _ in 0..5 {
        showcase.step();
    }
    let agent = &showcase.arenas()[0].agents()[0];
    assert!(agent.speed < 1e-9, "{}", agent.speed);
    assert!((agent.position.x - 10.0).abs() < 1e-9);
}

#[test]
fn speed_is_capped_at_max_speed() {
    let mut showcase = one_prey(0.0, 1.0 - 1e-12, Vec2::new(10.0, 10.0), 0.0);
    for _ in 0..50 {
        showcase.step();
        assert!(showcase.arenas()[0].agents()[0].speed <= 2.0);
    }
    assert_close(showcase.arenas()[0].agents()[0].speed, 2.0);
}

#[test]
fn agents_wrap_around_torus_edges() {
    // Full thrust east from x = 49 in a 50-wide world: 49.5, then 50.45 → 0.45.
    let mut showcase = one_prey(0.0, 1.0 - 1e-12, Vec2::new(49.0, 20.0), 0.0);
    showcase.step();
    assert_close(showcase.arenas()[0].agents()[0].position.x, 49.5);
    showcase.step();
    assert_close(showcase.arenas()[0].agents()[0].position.x, 0.45);

    // Heading south (-y) from y = 0.2 in a 40-high world wraps to the top.
    let mut showcase = one_prey(
        0.0,
        1.0 - 1e-12,
        Vec2::new(5.0, 0.2),
        -std::f64::consts::FRAC_PI_2,
    );
    showcase.step();
    assert_close(showcase.arenas()[0].agents()[0].position.y, 39.7);
}

#[test]
fn positions_stay_inside_the_world_over_a_long_run() {
    let mut showcase = one_prey(0.37, 1.0 - 1e-12, Vec2::new(25.0, 20.0), 0.0);
    for _ in 0..500 {
        showcase.step();
        let p = showcase.arenas()[0].agents()[0].position;
        assert!(
            (0.0..50.0).contains(&p.x) && (0.0..40.0).contains(&p.y),
            "{p:?}"
        );
    }
}
