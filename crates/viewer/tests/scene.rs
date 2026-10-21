//! Edge copies of the drawn scene on every side and corner of the torus.

use std::f64::consts::PI;

use neural_survival_core::{Config, Evolution, Placement, Showcase, Vec2};
use neural_survival_viewer::{HEADING_MARKER_LENGTH, Scene};

/// One stationary Prey of body radius 1 and food of radius 0.5 in a 50 × 40
/// world.
fn scene(agent: (f64, f64), heading: f64, food: &[(f64, f64)]) -> Scene {
    let mut config = Config::default();
    config.world.width = 50.0;
    config.world.height = 40.0;
    config.population.mlp_prey = 1;
    config.agent.body_radius = 1.0;
    config.agent.max_speed = 0.0;
    config.food.radius = 0.5;
    let genomes = Evolution::new(config.clone(), 1)
        .unwrap()
        .snapshot()
        .genomes;
    let showcase = Showcase::builder(&config, 1)
        .genomes(genomes)
        .placements(vec![Placement {
            position: Vec2::new(agent.0, agent.1),
            heading,
        }])
        .food(food.iter().map(|&(x, y)| Vec2::new(x, y)).collect())
        .build()
        .unwrap();
    Scene::of(&showcase)
}

fn sorted(mut positions: Vec<Vec2>) -> Vec<(f64, f64)> {
    positions.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    positions
        .into_iter()
        .map(|p| ((p.x * 1e9).round() / 1e9, (p.y * 1e9).round() / 1e9))
        .collect()
}

fn agent_positions_heading(at: (f64, f64), heading: f64) -> Vec<(f64, f64)> {
    sorted(
        scene(at, heading, &[])
            .agents
            .iter()
            .map(|a| a.position)
            .collect(),
    )
}

fn agent_positions(at: (f64, f64)) -> Vec<(f64, f64)> {
    agent_positions_heading(at, 0.0)
}

#[test]
fn right_edge_is_copied_to_the_left() {
    assert_eq!(agent_positions((49.5, 20.0)), [(-0.5, 20.0), (49.5, 20.0)]);
}

#[test]
fn bottom_edge_is_copied_to_the_top() {
    assert_eq!(agent_positions((25.0, 0.5)), [(25.0, 0.5), (25.0, 40.5)]);
}

#[test]
fn top_edge_is_copied_to_the_bottom() {
    assert_eq!(agent_positions((25.0, 39.5)), [(25.0, -0.5), (25.0, 39.5)]);
}

#[test]
fn a_corner_is_copied_to_the_other_three() {
    assert_eq!(
        agent_positions((49.5, 0.5)),
        [(-0.5, 0.5), (-0.5, 40.5), (49.5, 0.5), (49.5, 40.5)]
    );
}

#[test]
fn a_heading_marker_crossing_an_edge_is_copied() {
    // The body (radius 1) stops short of the left edge; the marker pointing
    // left does not.
    assert_eq!(
        agent_positions_heading((1.2, 20.0), PI),
        [(1.2, 20.0), (51.2, 20.0)]
    );
}

#[test]
fn touching_an_edge_is_not_overlapping_it() {
    // An Agent's reach is its heading marker, which is longer than its body.
    assert_eq!(HEADING_MARKER_LENGTH, 1.5);
    for at in [(1.5, 20.0), (48.5, 20.0), (25.0, 1.5), (25.0, 38.5)] {
        assert_eq!(agent_positions(at), [at], "at {at:?}");
    }
}

#[test]
fn food_in_a_corner_is_copied_to_the_other_three() {
    let scene = scene((25.0, 20.0), 0.0, &[(0.25, 39.75)]);
    assert_eq!(
        sorted(scene.food.iter().map(|f| f.position).collect()),
        [(0.25, -0.25), (0.25, 39.75), (50.25, -0.25), (50.25, 39.75)]
    );
}
