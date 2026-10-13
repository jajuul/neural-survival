mod common;

use common::{assert_close, constant_prey, probe_prey, tiny_config};
use neural_survival_core::{
    AgentId, Config, EnergyRule, EyeKind, Genome, Placement, Showcase, Vec2,
};
use std::f64::consts::{FRAC_PI_2, PI};

const STILL: f64 = -1.0 + 1e-12;

fn showcase(config: &Config, genome: Genome, at: Placement, food: Vec<Vec2>) -> Showcase {
    Showcase::builder(config, 9)
        .genomes(vec![genome])
        .placements(vec![at])
        .food(food)
        .build()
        .unwrap()
}

fn at_polar(origin: Vec2, degrees: f64, distance: f64) -> Vec2 {
    let a = degrees.to_radians();
    Vec2::new(origin.x + distance * a.cos(), origin.y + distance * a.sin())
}

fn eye_config(bins: usize) -> Config {
    let mut config = tiny_config();
    config.sensors.eye_bins = bins;
    config.agent.field_of_view = FRAC_PI_2;
    config.agent.vision_range = 20.0;
    config.agent.body_radius = 1.0;
    config.food.radius = 0.5;
    config
}

#[test]
fn telemetry_is_absent_before_the_first_step_and_for_unknown_agents() {
    let config = tiny_config();
    let at = Placement {
        position: Vec2::new(5.0, 5.0),
        heading: 0.0,
    };
    let mut showcase = showcase(&config, constant_prey(&config, 0.0, 0.0), at, vec![]);
    assert!(showcase.telemetry(AgentId(0)).is_none());
    showcase.step();
    assert!(showcase.telemetry(AgentId(0)).is_some());
    assert!(showcase.telemetry(AgentId(1)).is_none());
}

#[test]
fn food_eye_bins_activate_for_the_nearest_food_in_each_sector() {
    // 90° field of view split into four 22.5° sectors, bin 0 clockwise-most.
    let config = eye_config(4);
    let me = Vec2::new(20.0, 20.0);
    let food = vec![
        at_polar(me, 30.0, 10.0),  // bin 3, 1 - 10/20
        at_polar(me, -10.0, 5.0),  // bin 1, 1 - 5/20
        at_polar(me, -20.0, 15.0), // bin 1 but farther: ignored
        at_polar(me, -40.0, 21.0), // bin 0 but out of range
        at_polar(me, 180.0, 5.0),  // behind: outside the field of view
    ];
    let at = Placement {
        position: me,
        heading: 0.0,
    };
    let mut showcase = showcase(&config, constant_prey(&config, 0.0, STILL), at, food);
    showcase.step();

    let telemetry = showcase.telemetry(AgentId(0)).unwrap();
    assert_eq!(telemetry.eyes.len(), 1);
    assert_eq!(telemetry.eyes[0].kind, EyeKind::Food);
    let bins = &telemetry.eyes[0].bins;
    assert_eq!(bins.len(), 4);
    let expected = [0.0, 0.75, 0.0, 0.5];
    for (bin, want) in bins.iter().zip(expected) {
        assert_close(*bin, want);
    }
}

#[test]
fn eye_bins_follow_the_heading_and_see_across_the_torus_edge() {
    // Facing west from x = 1; food at x = 45 is 6 units ahead across the edge.
    let config = eye_config(3);
    let at = Placement {
        position: Vec2::new(1.0, 20.0),
        heading: PI,
    };
    let mut showcase = showcase(
        &config,
        constant_prey(&config, 0.0, STILL),
        at,
        vec![Vec2::new(45.0, 20.0)],
    );
    showcase.step();

    let bins = &showcase.telemetry(AgentId(0)).unwrap().eyes[0].bins;
    assert_close(bins[0], 0.0);
    assert_close(bins[1], 0.7);
    assert_close(bins[2], 0.0);
}

#[test]
fn core_sensors_report_normalised_energy_and_speed() {
    let mut config = tiny_config();
    config.prey.starting_energy = 50.0;
    config.sensors.energy_scale = 200.0;
    config.energy.idle_drain = 1.0;
    let at = Placement {
        position: Vec2::new(5.0, 5.0),
        heading: 0.0,
    };
    let mut showcase = showcase(&config, constant_prey(&config, 0.0, 0.0), at, vec![]);

    showcase.step();
    let core = showcase.telemetry(AgentId(0)).unwrap().core;
    assert_close(core.energy, 0.25);
    assert_close(core.speed, 0.0);

    // After one step at thrust 0.5: speed 0.25 of max 2, energy 49 of 200.
    showcase.step();
    let core = showcase.telemetry(AgentId(0)).unwrap().core;
    assert_close(core.energy, 49.0 / 200.0);
    assert_close(core.speed, 0.125);
}

#[test]
fn normalised_energy_saturates_at_one() {
    let mut config = tiny_config();
    config.prey.starting_energy = 500.0;
    config.sensors.energy_scale = 200.0;
    let at = Placement {
        position: Vec2::new(5.0, 5.0),
        heading: 0.0,
    };
    let mut showcase = showcase(&config, constant_prey(&config, 0.0, 0.0), at, vec![]);
    showcase.step();
    assert_close(showcase.telemetry(AgentId(0)).unwrap().core.energy, 1.0);
}

#[test]
fn telemetry_reports_raw_and_applied_outputs() {
    let config = tiny_config();
    let at = Placement {
        position: Vec2::new(5.0, 5.0),
        heading: 0.0,
    };
    let mut showcase = showcase(&config, constant_prey(&config, -0.5, -0.6), at, vec![]);
    showcase.step();

    let telemetry = showcase.telemetry(AgentId(0)).unwrap();
    assert_close(telemetry.raw_outputs.steering, -0.5);
    assert_close(telemetry.raw_outputs.throttle, -0.6);
    assert_close(telemetry.applied.steering, -0.5);
    assert_close(telemetry.applied.thrust, 0.2);
}

#[test]
fn telemetry_lists_energy_cost_per_energy_rule() {
    let mut config = tiny_config();
    config.energy.idle_drain = 0.3;
    let at = Placement {
        position: Vec2::new(5.0, 5.0),
        heading: 0.0,
    };
    let mut showcase = showcase(&config, constant_prey(&config, 0.0, 0.0), at, vec![]);
    showcase.step();

    let costs = &showcase.telemetry(AgentId(0)).unwrap().energy_costs;
    assert_eq!(costs.len(), 1);
    assert_eq!(costs[0].rule, EnergyRule::IdleDrain);
    assert_close(costs[0].amount, 0.3);
}

#[test]
fn controller_input_is_energy_then_speed_then_food_bins() {
    let mut config = eye_config(3);
    config.prey.starting_energy = 50.0;
    config.sensors.energy_scale = 100.0;
    let me = Vec2::new(20.0, 20.0);
    // Dead ahead at distance 8: middle bin reads 1 - 8/20 = 0.6.
    let food = vec![at_polar(me, 0.0, 8.0)];
    let at = Placement {
        position: me,
        heading: 0.0,
    };

    let expected_inputs: [f64; 5] = [0.5, 0.0, 0.0, 0.6, 0.0];
    for (index, input) in expected_inputs.into_iter().enumerate() {
        let mut showcase = showcase(&config, probe_prey(&config, index, STILL), at, food.clone());
        showcase.step();
        let steering = showcase.telemetry(AgentId(0)).unwrap().raw_outputs.steering;
        assert_close(steering, input.tanh());
    }
}

#[test]
fn last_telemetry_is_kept_after_the_agent_dies() {
    let mut config = tiny_config();
    config.prey.starting_energy = 1.0;
    config.energy.idle_drain = 0.6;
    let at = Placement {
        position: Vec2::new(5.0, 5.0),
        heading: 0.0,
    };
    let mut showcase = showcase(&config, constant_prey(&config, 0.3, 0.0), at, vec![]);
    showcase.step();
    showcase.step();
    assert!(!showcase.arenas()[0].agents()[0].alive);
    let last = showcase.telemetry(AgentId(0)).unwrap().clone();
    assert_close(last.core.energy, 0.4 / config.sensors.energy_scale);

    // The Showcase has ended, but telemetry stays readable.
    assert!(showcase.has_ended());
    assert_eq!(showcase.telemetry(AgentId(0)), Some(&last));
}

#[test]
fn dead_agents_telemetry_is_not_overwritten_while_others_live() {
    let mut config = tiny_config();
    config.prey.starting_energy = 1.0;
    config.energy.idle_drain = 0.6;
    config.food.energy = 100.0;
    let genomes = vec![
        constant_prey(&config, 0.3, 0.0),
        constant_prey(&config, 0.0, STILL),
    ];
    let mut showcase = Showcase::builder(&config, 9)
        .genomes(genomes)
        .placements(vec![
            Placement {
                position: Vec2::new(5.0, 5.0),
                heading: 0.0,
            },
            Placement {
                position: Vec2::new(30.0, 30.0),
                heading: 0.0,
            },
        ])
        .food(vec![Vec2::new(30.0, 30.0)])
        .build()
        .unwrap();
    showcase.step();
    showcase.step();
    let last = showcase.telemetry(AgentId(0)).unwrap().clone();
    for _ in 0..3 {
        showcase.step();
    }
    assert!(!showcase.has_ended());
    assert_eq!(showcase.telemetry(AgentId(0)), Some(&last));
}
