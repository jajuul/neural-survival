//! The step budget's edges: the frame cap, the carried fraction, and what
//! clears it.

use neural_survival_core::{Config, Evolution};
use neural_survival_viewer::Viewer;

/// A viewer at speed 10 on a 100-step Showcase.
fn viewer() -> Viewer {
    let mut config = Config::default();
    config.population.mlp_prey = 2;
    config.food.count = 2;
    config.prey.starting_energy = 10_000.0;
    config.evaluation.episode_steps = 100;
    let mut viewer = Viewer::open(Evolution::new(config, 7).unwrap().snapshot()).unwrap();
    viewer.set_speed(10);
    viewer
}

#[test]
fn a_capped_frame_still_carries_its_fraction() {
    let mut viewer = viewer();
    viewer.play();
    viewer.frame(2.0); // counts as 0.25 s: 2.5 steps
    assert_eq!(viewer.step_count(), 2);
    viewer.frame(0.05); // 0.5 more
    assert_eq!(viewer.step_count(), 3);
}

#[test]
fn pausing_drops_the_carried_fraction() {
    let mut viewer = viewer();
    viewer.play();
    viewer.frame(0.15); // 1.5 steps
    assert_eq!(viewer.step_count(), 1);
    viewer.pause();
    viewer.play();
    viewer.frame(0.05); // 0.5 steps, not 1.0
    assert_eq!(viewer.step_count(), 1);
    viewer.frame(0.05);
    assert_eq!(viewer.step_count(), 2);
}

#[test]
fn restart_drops_the_carried_fraction() {
    let mut viewer = viewer();
    viewer.play();
    viewer.frame(0.15);
    viewer.restart();
    viewer.play();
    viewer.frame(0.05);
    assert_eq!(viewer.step_count(), 0);
}

#[test]
fn a_negative_frame_counts_as_no_time() {
    let mut viewer = viewer();
    viewer.play();
    viewer.frame(0.15);
    viewer.frame(-1.0);
    viewer.frame(0.05);
    assert_eq!(viewer.step_count(), 2);
}

#[test]
fn changing_speed_keeps_the_carried_fraction() {
    let mut viewer = viewer();
    viewer.play();
    viewer.frame(0.15); // 1.5 steps at speed 10
    viewer.set_speed(20);
    viewer.frame(0.025); // 0.5 steps at speed 20
    assert_eq!(viewer.step_count(), 2);
}

#[test]
fn playing_again_while_playing_keeps_the_carried_fraction() {
    let mut viewer = viewer();
    viewer.play();
    viewer.frame(0.15);
    viewer.play();
    viewer.frame(0.05);
    assert_eq!(viewer.step_count(), 2);
}

#[test]
fn speed_bounds_are_kept() {
    let mut viewer = viewer();
    viewer.set_speed(1);
    assert_eq!(viewer.speed(), 1);
    viewer.set_speed(1000);
    assert_eq!(viewer.speed(), 1000);
}

#[test]
fn playing_unpauses_and_pausing_pauses() {
    let mut viewer = viewer();
    viewer.play();
    assert!(!viewer.is_paused());
    viewer.pause();
    assert!(viewer.is_paused());
}
