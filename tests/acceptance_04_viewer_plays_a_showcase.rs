//! Acceptance contract for 04-viewer-plays-a-showcase, written before implementation.
//! Only public core and viewer interfaces are used; no window is opened.

use std::process::{Command, Output};

use neural_survival_core::{
    Config, ControllerFamily, EvaluationStage, Evolution, MLP_PREY, Placement, Showcase, Snapshot,
    Vec2,
};
use neural_survival_viewer::{FOOD_COLOUR, ROLE_COLOURS, Scene, Shape, Viewer};
use proptest::prelude::*;
use tempfile::TempDir;

fn tiny_config(steps: u32) -> Config {
    let mut config = Config::default();
    config.world.width = 50.0;
    config.world.height = 40.0;
    config.population.mlp_prey = 4;
    config.food.count = 5;
    config.food.radius = 0.5;
    config.agent.body_radius = 1.0;
    config.prey.starting_energy = 10_000.0;
    config.mlp.hidden_layers.clear();
    config.evaluation.episodes = 1;
    config.evaluation.episode_steps = steps;
    config.ga.tournament_size = 2;
    config.ga.elitism = 1;
    config
}

fn snapshot(config: Config, generation: u32) -> Snapshot {
    let mut evolution = Evolution::new(config, 42).unwrap();
    evolution.advance(generation, |_| {});
    evolution.snapshot()
}

fn viewer(steps: u32) -> Viewer {
    Viewer::open(snapshot(tiny_config(steps), 0)).unwrap()
}

fn run(dir: &TempDir, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_neural-survival"))
        .current_dir(dir.path())
        .args(args)
        .output()
        .unwrap()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn play_to_end(viewer: &mut Viewer, speed: u32, dt: f64) {
    viewer.set_speed(speed);
    viewer.play();
    // Bounded so a broken playback implementation fails instead of hanging.
    for _ in 0..2000 {
        if viewer.has_ended() {
            return;
        }
        viewer.frame(dt);
    }
    assert!(viewer.has_ended(), "Episode did not end within 2000 frames");
}

#[test]
fn a_saved_snapshot_opens() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("config.toml"),
        tiny_config(20).to_toml_string(),
    )
    .unwrap();
    assert_success(&run(
        &dir,
        &[
            "batch",
            "--config",
            "config.toml",
            "--seed",
            "42",
            "--generations",
            "10",
            "--out",
            "results",
        ],
    ));
    assert!(dir.path().join("results/snapshot_gen_10.json").is_file());
    let output = run(
        &dir,
        &[
            "view",
            "--snapshot",
            "results/snapshot_gen_10.json",
            "--no-window",
        ],
    );
    assert_success(&output);
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("Viewer ready: Generation 10, 4 Agents, step limit 20")
    );
}

#[test]
fn a_missing_snapshot_fails_with_its_path() {
    let dir = TempDir::new().unwrap();
    assert!(!dir.path().join("missing.json").exists());
    let output = run(&dir, &["view", "--snapshot", "missing.json", "--no-window"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing.json"));
}

#[test]
fn an_unsupported_snapshot_version_fails_naming_the_version() {
    let dir = TempDir::new().unwrap();
    let mut saved = snapshot(tiny_config(20), 0);
    saved.format_version = 999;
    saved.save(dir.path().join("future.json")).unwrap();
    let output = run(&dir, &["view", "--snapshot", "future.json", "--no-window"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("999"));
}

#[test]
fn opening_state() {
    let saved = snapshot(tiny_config(20), 3);
    let expected = Showcase::from_snapshot(&saved).unwrap();
    let viewer = Viewer::open(saved).unwrap();
    assert!(viewer.is_paused());
    assert_eq!(viewer.step_count(), 0);
    assert!(!viewer.has_ended());
    assert_eq!(viewer.showcase(), &expected);
}

#[test]
fn frames_advance_nothing_while_paused() {
    let mut viewer = viewer(60);
    viewer.set_speed(32);
    let initial = viewer.showcase().clone();
    for _ in 0..4 {
        viewer.frame(0.125);
    }
    assert_eq!(viewer.step_count(), 0);
    assert_eq!(viewer.showcase(), &initial);
}

#[test]
fn continuing_plays_pausing_stops() {
    let mut viewer = viewer(60);
    viewer.set_speed(32);
    viewer.play();
    for _ in 0..2 {
        viewer.frame(0.125);
    }
    assert_eq!(viewer.step_count(), 8);
    viewer.pause();
    let paused = viewer.showcase().clone();
    for _ in 0..2 {
        viewer.frame(0.125);
    }
    assert_eq!(viewer.step_count(), 8);
    assert_eq!(viewer.showcase(), &paused);
    viewer.play();
    viewer.frame(0.125);
    assert_eq!(viewer.step_count(), 12);
}

#[test]
fn single_steps() {
    let mut viewer = viewer(60);
    for step in 1..=3 {
        viewer.single_step();
        assert_eq!(viewer.step_count(), step);
        assert!(viewer.is_paused());
    }
}

#[test]
fn single_step_is_ignored_while_playing() {
    let mut viewer = viewer(60);
    viewer.set_speed(32);
    viewer.play();
    viewer.frame(0.125);
    assert_eq!(viewer.step_count(), 4);
    let before = viewer.showcase().clone();
    viewer.single_step();
    assert_eq!(viewer.step_count(), 4);
    assert_eq!(viewer.showcase(), &before);
}

#[test]
fn default_speed() {
    assert_eq!(viewer(60).speed(), 30);
}

#[test]
fn fractional_steps_carry_over() {
    let mut viewer = viewer(60);
    viewer.set_speed(10);
    viewer.play();
    viewer.frame(0.125);
    assert_eq!(viewer.step_count(), 1);
    viewer.frame(0.125);
    assert_eq!(viewer.step_count(), 2);
    for _ in 2..8 {
        viewer.frame(0.125);
    }
    assert_eq!(viewer.step_count(), 10);
}

#[test]
fn speed_is_clamped() {
    let mut viewer = viewer(60);
    viewer.set_speed(0);
    assert_eq!(viewer.speed(), 1);
    viewer.set_speed(5000);
    assert_eq!(viewer.speed(), 1000);
}

#[test]
fn a_slow_frame_is_capped() {
    let mut viewer = viewer(60);
    viewer.set_speed(100);
    viewer.play();
    viewer.frame(2.0);
    assert_eq!(viewer.step_count(), 25);
}

#[test]
fn played_and_stepped_showcases_match() {
    let saved = snapshot(tiny_config(60), 0);
    let mut direct = Showcase::from_snapshot(&saved).unwrap();
    let mut played = Viewer::open(saved.clone()).unwrap();
    let mut stepped = Viewer::open(saved).unwrap();
    played.set_speed(8);
    played.play();
    stepped.set_speed(1000);
    for _ in 0..15 {
        played.frame(0.125);
        stepped.single_step();
        direct.step();
    }
    assert_eq!(played.step_count(), 15);
    assert_eq!(stepped.step_count(), 15);
    assert_eq!(direct.step_count(), 15);
    assert_eq!(played.showcase(), &direct);
    assert_eq!(stepped.showcase(), &direct);
}

#[test]
fn different_speeds_reach_the_same_episode_end() {
    let saved = snapshot(tiny_config(60), 0);
    let mut slow = Viewer::open(saved.clone()).unwrap();
    let mut fast = Viewer::open(saved).unwrap();
    play_to_end(&mut slow, 7, 0.1);
    play_to_end(&mut fast, 1000, 0.25);
    assert!(slow.has_ended() && fast.has_ended());
    assert_eq!(slow.step_count(), fast.step_count());
    assert_eq!(slow.showcase(), fast.showcase());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn speed_never_changes_what_happens(
        speed in 1u32..=1000,
        frames in prop::collection::vec((0.0f64..=0.3).prop_filter("positive dt", |dt| *dt > 0.0), 1..24),
    ) {
        let saved = snapshot(tiny_config(400), 0);
        let mut direct = Showcase::from_snapshot(&saved).unwrap();
        let mut played = Viewer::open(saved.clone()).unwrap();
        let mut stepped = Viewer::open(saved).unwrap();
        played.set_speed(speed);
        played.play();
        stepped.set_speed(1000);
        let mut elapsed_steps = 0.0f64;
        for dt in frames {
            elapsed_steps += dt.min(0.25) * f64::from(speed);
            let expected_step = (elapsed_steps.floor() as u32).min(400);
            while direct.step_count() < expected_step {
                direct.step();
                // Paused elapsed time must not change the single-step playback.
                stepped.frame(dt);
                stepped.single_step();
            }
            played.frame(dt);
            prop_assert_eq!(played.step_count(), expected_step);
            prop_assert_eq!(played.showcase(), &direct);
            prop_assert_eq!(stepped.showcase(), &direct);
        }
        // Pausing and resuming must preserve what each later step computes.
        played.pause();
        played.frame(0.3);
        prop_assert_eq!(played.showcase(), &direct);
        played.single_step();
        direct.step();
        prop_assert_eq!(played.showcase(), &direct);
        played.play();
        played.frame(0.25);
        while direct.step_count() < played.step_count() {
            direct.step();
        }
        prop_assert_eq!(played.showcase(), &direct);
    }
}

#[test]
fn playing_past_the_end() {
    let mut viewer = viewer(20);
    viewer.set_speed(1000);
    viewer.play();
    viewer.frame(0.25);
    assert!(viewer.has_ended());
    assert_eq!(viewer.step_count(), 20);
    assert!(
        viewer
            .showcase()
            .arenas()
            .iter()
            .flat_map(|a| a.agents())
            .all(|a| a.alive)
    );
    let ended = viewer.showcase().clone();
    viewer.frame(0.25);
    assert_eq!(viewer.showcase(), &ended);
    viewer.pause();
    viewer.single_step();
    assert_eq!(viewer.step_count(), 20);
    assert!(viewer.has_ended());
    assert_eq!(viewer.showcase(), &ended);
}

#[test]
fn restart_after_the_end() {
    let saved = snapshot(tiny_config(20), 0);
    let fresh = Viewer::open(saved.clone()).unwrap();
    let mut viewer = Viewer::open(saved).unwrap();
    play_to_end(&mut viewer, 1000, 0.25);
    let first_end = viewer.showcase().clone();
    viewer.restart();
    assert_eq!(viewer.step_count(), 0);
    assert!(viewer.is_paused());
    assert!(!viewer.has_ended());
    assert_eq!(viewer.showcase(), fresh.showcase());
    play_to_end(&mut viewer, 1000, 0.25);
    assert_eq!(viewer.showcase(), &first_end);
}

#[test]
fn restart_mid_episode() {
    let mut viewer = viewer(60);
    let initial = viewer.showcase().clone();
    viewer.set_speed(10);
    viewer.play();
    viewer.frame(0.25);
    viewer.frame(0.25);
    viewer.frame(0.25);
    assert_eq!(viewer.step_count(), 7);
    viewer.restart();
    assert_eq!(viewer.step_count(), 0);
    assert!(viewer.is_paused());
    assert!(!viewer.has_ended());
    assert_eq!(viewer.showcase(), &initial);
}

#[test]
fn stats_of_a_fresh_showcase() {
    let viewer = Viewer::open(snapshot(tiny_config(20), 10)).unwrap();
    let stats = viewer.stats();
    assert_eq!(stats.generation, 10);
    assert_eq!(stats.step, 0);
    assert_eq!(stats.alive, 4);
    assert_eq!(stats.food, 5);
    assert_eq!(stats.episodes, 1);
    assert_eq!(stats.episode_steps, 20);
    assert_eq!(stats.species.len(), 1);
    assert_eq!(stats.species[0].species, MLP_PREY);
    assert_eq!((stats.species[0].alive, stats.species[0].total), (4, 4));
    assert_eq!(stats.families.len(), 1);
    assert_eq!(stats.families[0].family, ControllerFamily::Mlp);
    assert_eq!((stats.families[0].alive, stats.families[0].total), (4, 4));
}

#[test]
fn stats_follow_deaths() {
    let mut config = tiny_config(20);
    config.prey.starting_energy = 1.0;
    config.energy.idle_drain = 0.5;
    config.food.count = 0;
    config.agent.max_speed = 0.0;
    let mut viewer = Viewer::open(snapshot(config, 0)).unwrap();
    viewer.set_speed(8);
    viewer.play();
    viewer.frame(0.25);
    let stats = viewer.stats();
    assert_eq!(stats.step, 2);
    assert_eq!(stats.alive, 0);
    let species = stats
        .species
        .iter()
        .find(|s| s.species == MLP_PREY)
        .unwrap();
    assert_eq!((species.alive, species.total), (0, 4));
    let family = stats
        .families
        .iter()
        .find(|s| s.family == ControllerFamily::Mlp)
        .unwrap();
    assert_eq!((family.alive, family.total), (0, 4));
}

#[test]
fn stats_show_the_generations_evaluation_schedule_stage() {
    let mut config = tiny_config(4);
    config.evaluation.stages.push(EvaluationStage {
        from_generation: 2,
        episodes: 3,
        episode_steps: 7,
    });
    let stats = Viewer::open(snapshot(config, 2)).unwrap().stats();
    assert_eq!(stats.episodes, 3);
    assert_eq!(stats.episode_steps, 7);
}

#[test]
fn the_viewers_scene_follows_its_showcase() {
    // Seed 42 was checked through the public core API to move within five steps.
    let mut viewer = viewer(20);
    let initial = viewer.scene();
    viewer.set_speed(20);
    viewer.play();
    viewer.frame(0.25);
    assert_eq!(viewer.step_count(), 5);
    assert_eq!(viewer.scene(), Scene::of(viewer.showcase()));
    assert_ne!(viewer.scene(), initial);
}

fn placed_showcase(config: &Config, positions: &[(f64, f64)], food: &[(f64, f64)]) -> Showcase {
    let genomes = Evolution::new(config.clone(), 42)
        .unwrap()
        .snapshot()
        .genomes;
    Showcase::builder(config, 42)
        .genomes(genomes)
        .placements(
            positions
                .iter()
                .map(|&(x, y)| Placement {
                    position: Vec2::new(x, y),
                    heading: 0.5,
                })
                .collect(),
        )
        .food(food.iter().map(|&(x, y)| Vec2::new(x, y)).collect())
        .build()
        .unwrap()
}

fn scene_config(agents: usize) -> Config {
    let mut config = tiny_config(20);
    config.population.mlp_prey = agents;
    config.food.count = 0;
    config.agent.max_speed = 0.0;
    config
}

#[test]
fn living_prey_and_food() {
    let showcase = placed_showcase(&scene_config(1), &[(10.0, 10.0)], &[(30.0, 20.0)]);
    let scene = Scene::of(&showcase);
    assert_eq!(scene.agents.len(), 1);
    let agent = &scene.agents[0];
    assert_eq!(agent.shape, Shape::Circle);
    assert_eq!(agent.colour, ROLE_COLOURS[0]);
    assert_eq!(agent.position, Vec2::new(10.0, 10.0));
    assert_eq!(agent.radius, 1.0);
    assert_eq!(agent.heading, 0.5);
    assert_eq!(scene.food.len(), 1);
    let food = &scene.food[0];
    assert_eq!(food.position, Vec2::new(30.0, 20.0));
    assert_eq!(food.radius, 0.5);
    assert_eq!(food.colour, FOOD_COLOUR);
    assert_ne!(food.colour, agent.colour);
}

#[test]
fn dead_agents_are_not_drawn() {
    let mut config = scene_config(2);
    config.prey.starting_energy = 1.0;
    config.energy.idle_drain = 0.5;
    // Food keeps only the first stationary Prey alive.
    let mut showcase = placed_showcase(&config, &[(10.0, 10.0), (30.0, 20.0)], &[(10.0, 10.0)]);
    showcase.step_many(2);
    let living: Vec<_> = showcase.arenas()[0]
        .agents()
        .iter()
        .filter(|a| a.alive)
        .collect();
    assert_eq!(living.len(), 1, "fixture must have one living Prey");
    let scene = Scene::of(&showcase);
    assert_eq!(scene.agents.len(), 1);
    assert_eq!(scene.agents[0].position, living[0].position);
    assert_eq!(scene.agents[0].heading, living[0].heading);
}

fn assert_positions(actual: &[Vec2], expected: &[(f64, f64)]) {
    assert_eq!(actual.len(), expected.len());
    for &(x, y) in expected {
        assert_eq!(
            actual
                .iter()
                .filter(|p| (p.x - x).abs() < 1e-10 && (p.y - y).abs() < 1e-10)
                .count(),
            1,
            "expected ({x}, {y}) once in {actual:?}"
        );
    }
}

#[test]
fn shapes_overlapping_an_edge_are_drawn_on_both_sides() {
    let config = scene_config(1);
    for (position, expected) in [
        ((0.5, 20.0), vec![(0.5, 20.0), (50.5, 20.0)]),
        (
            (0.5, 39.5),
            vec![(0.5, 39.5), (50.5, 39.5), (0.5, -0.5), (50.5, -0.5)],
        ),
        ((25.0, 20.0), vec![(25.0, 20.0)]),
    ] {
        let scene = Scene::of(&placed_showcase(&config, &[position], &[]));
        assert_positions(
            &scene.agents.iter().map(|a| a.position).collect::<Vec<_>>(),
            &expected,
        );
        for shape in &scene.agents {
            assert_eq!(shape.radius, 1.0);
            assert_eq!(shape.heading, 0.5);
            assert_eq!(shape.shape, Shape::Circle);
            assert_eq!(shape.colour, ROLE_COLOURS[0]);
        }
    }
}

#[test]
fn food_overlapping_an_edge_is_drawn_on_both_sides() {
    let scene = Scene::of(&placed_showcase(
        &scene_config(1),
        &[(25.0, 20.0)],
        &[(49.8, 10.0)],
    ));
    assert_positions(
        &scene.food.iter().map(|f| f.position).collect::<Vec<_>>(),
        &[(49.8, 10.0), (-0.2, 10.0)],
    );
    for food in &scene.food {
        assert_eq!(food.radius, 0.5);
        assert_eq!(food.colour, FOOD_COLOUR);
    }
}

#[test]
fn one_line_pair_per_species() {
    let mut saved = snapshot(tiny_config(20), 3);
    assert_eq!(saved.metrics.len(), 3);
    for (row, (best, mean)) in saved
        .metrics
        .iter_mut()
        .zip([(4.0, 2.0), (6.0, 3.0), (5.0, 3.5)])
    {
        row.best_fitness = best;
        row.mean_fitness = mean;
    }
    let series = Viewer::open(saved).unwrap().fitness_series();
    assert_eq!(series.len(), 1);
    assert_eq!(series[0].species, MLP_PREY);
    assert_eq!(series[0].best, vec![[0.0, 4.0], [1.0, 6.0], [2.0, 5.0]]);
    assert_eq!(series[0].mean, vec![[0.0, 2.0], [1.0, 3.0], [2.0, 3.5]]);
}

#[test]
fn no_history() {
    let saved = snapshot(tiny_config(20), 0);
    assert!(saved.metrics.is_empty());
    assert!(Viewer::open(saved).unwrap().fitness_series().is_empty());
}
