# Spec Delta

## Purpose

The viewer plays a Showcase of a saved Generation so the experimenter can watch it: opening a Snapshot, playback controls, overview stats, the drawn world, and the Fitness history chart. These scenarios cover the viewer's behaviour without a window; what appears on screen is checked by hand.

## ADDED Requirements

### Requirement: The view command loads a Snapshot

`neural-survival view --snapshot <path>` SHALL load the Snapshot and open the viewer on a Showcase of its Generation. With the hidden `--no-window` flag it SHALL do everything except open the window, print `Viewer ready: Generation <g>, <n> Agents, step limit <s>`, and exit successfully. A Snapshot that cannot be loaded SHALL fail the command with the load error on stderr, before any window opens.

#### Scenario: A saved Snapshot opens

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::a_saved_snapshot_opens -->

- **WHEN** a `batch` run with seed 42, 4 Prey and a 20-step Episode writes `results/snapshot_gen_10.json`
- **AND** the user runs `neural-survival view --snapshot results/snapshot_gen_10.json --no-window`
- **THEN** the command succeeds and stdout contains `Viewer ready: Generation 10, 4 Agents, step limit 20`

#### Scenario: A missing Snapshot fails with its path

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::a_missing_snapshot_fails_with_its_path -->

- **WHEN** the user runs `neural-survival view --snapshot missing.json --no-window` and `missing.json` does not exist
- **THEN** the command fails and stderr contains `missing.json`

#### Scenario: An unsupported Snapshot version fails naming the version

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::an_unsupported_snapshot_version_fails_naming_the_version -->

- **WHEN** a Snapshot file's format version is changed to 999 and the user runs `view --snapshot` on it with `--no-window`
- **THEN** the command fails and stderr contains `999`

### Requirement: The viewer opens paused on the Snapshot's Showcase

A viewer opened on a Snapshot SHALL start paused at step 0, holding exactly the Showcase built from that Snapshot.

#### Scenario: Opening state

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::opening_state -->

- **WHEN** a viewer is opened on a Generation-3 Snapshot
- **THEN** it is paused, at step 0, not ended
- **AND** its Showcase equals a Showcase built directly from the same Snapshot

### Requirement: Pause and continue

While playing, each frame SHALL advance the Showcase by the steps its elapsed time allows at the current speed. While paused, frames SHALL advance nothing, and time elapsed while paused SHALL not carry over into later frames.

#### Scenario: Frames advance nothing while paused

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::frames_advance_nothing_while_paused -->

- **WHEN** a paused viewer at speed 32 steps per second receives 4 frames of 0.125 s
- **THEN** it is still at step 0

#### Scenario: Continuing plays, pausing stops

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::continuing_plays_pausing_stops -->
<!-- verified-by: crates/viewer/tests/playback.rs::pausing_drops_the_carried_fraction -->
<!-- verified-by: crates/viewer/tests/playback.rs::playing_again_while_playing_keeps_the_carried_fraction -->
<!-- verified-by: crates/viewer/tests/playback.rs::playing_unpauses_and_pausing_pauses -->

- **WHEN** a paused viewer at speed 32 continues and receives 2 frames of 0.125 s
- **THEN** it is at step 8
- **WHEN** it is paused and receives 2 more frames of 0.125 s
- **THEN** it is still at step 8
- **WHEN** it continues and receives 1 frame of 0.125 s
- **THEN** it is at step 12

### Requirement: Single-step while paused

A single-step command SHALL advance a paused viewer by exactly one step, and SHALL do nothing while playing.

#### Scenario: Single steps

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::single_steps -->

- **WHEN** a paused viewer at step 0 is single-stepped 3 times
- **THEN** it is at step 3 and still paused

#### Scenario: Single-step is ignored while playing

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::single_step_is_ignored_while_playing -->

- **WHEN** a playing viewer at step 4 is single-stepped
- **THEN** it is still at step 4

### Requirement: Simulation speed

Speed SHALL be set in steps per second, defaulting to 30 and clamped to [1, 1000]. Each frame SHALL run the whole steps its elapsed time allows, carrying the fractional remainder into the next frame. A single frame's elapsed time SHALL count for at most 0.25 s, so one slow frame cannot run a burst of steps.

#### Scenario: Default speed

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::default_speed -->

- **WHEN** a viewer is opened
- **THEN** its speed is 30 steps per second

#### Scenario: Fractional steps carry over

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::fractional_steps_carry_over -->
<!-- verified-by: crates/viewer/tests/playback.rs::changing_speed_keeps_the_carried_fraction -->
<!-- verified-by: crates/viewer/tests/playback.rs::a_negative_frame_counts_as_no_time -->

- **WHEN** a playing viewer at speed 10 receives 8 frames of 0.125 s
- **THEN** after the first frame it is at step 1, after the second at step 2, and after all 8 at step 10

#### Scenario: Speed is clamped

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::speed_is_clamped -->
<!-- verified-by: crates/viewer/tests/playback.rs::speed_bounds_are_kept -->

- **WHEN** the speed is set to 0
- **THEN** the speed is 1
- **WHEN** the speed is set to 5000
- **THEN** the speed is 1000

#### Scenario: A slow frame is capped

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::a_slow_frame_is_capped -->
<!-- verified-by: crates/viewer/tests/playback.rs::a_capped_frame_still_carries_its_fraction -->

- **WHEN** a playing viewer at speed 100 receives one frame of 2 s
- **THEN** it is at step 25

### Requirement: Speed never changes what happens

Playback speed, frame timing, pausing and single-stepping SHALL change only when steps run, never what a step computes (ADR-0001).

#### Scenario: Played and stepped Showcases match

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::played_and_stepped_showcases_match -->

- **WHEN** one viewer plays at speed 8 for 15 frames of 0.125 s, another at speed 1000 is single-stepped 15 times, and a Showcase built from the same Snapshot is stepped 15 times
- **THEN** all three are at step 15 with equal Showcases

#### Scenario: Different speeds reach the same Episode end

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::different_speeds_reach_the_same_episode_end -->

- **WHEN** two viewers on the same Snapshot with a 60-step Episode play to the end, one at speed 7 with 0.1 s frames and one at speed 1000 with 0.25 s frames
- **THEN** both have ended at the same step with equal Showcases

### Requirement: The Showcase stops at Episode end

When the Showcase's Episode ends, the viewer SHALL report it as ended, and further frames and single steps SHALL change nothing.

#### Scenario: Playing past the end

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::playing_past_the_end -->

- **WHEN** a viewer on a Snapshot with a 20-step Episode, in which no Agent dies, plays at speed 1000 for 1 frame of 0.25 s
- **THEN** it has ended at step 20
- **WHEN** it receives another frame and is paused and single-stepped
- **THEN** it is still at step 20 with an unchanged Showcase

### Requirement: Restart replays the Showcase exactly

Restart SHALL return the viewer to step 0, paused, with a Showcase equal to the one it opened with, so playing it again replays the same Episode.

#### Scenario: Restart after the end

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::restart_after_the_end -->

- **WHEN** a viewer plays to the end of its Episode, is restarted, and plays to the end again
- **THEN** right after restart it is at step 0, paused, not ended, and its Showcase equals a freshly opened viewer's
- **AND** its final Showcase equals the one from the first playthrough

#### Scenario: Restart mid-Episode

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::restart_mid_episode -->
<!-- verified-by: crates/viewer/tests/playback.rs::restart_drops_the_carried_fraction -->

- **WHEN** a viewer at step 7 is restarted
- **THEN** it is at step 0, paused, with the Showcase it opened with

### Requirement: Stats

The viewer SHALL report the Snapshot's Generation, the current step, the number of living Agents, the number of food items, living and total Agents per Species and per Controller Family, and the Episode count and step limit the Evaluation Schedule gives the Generation.

#### Scenario: Stats of a fresh Showcase

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::stats_of_a_fresh_showcase -->

- **WHEN** a viewer is opened on a Generation-10 Snapshot of 4 MLP Prey with 5 food, 1 Episode of 20 steps
- **THEN** its stats report Generation 10, step 0, 4 alive, 5 food, MLP-Prey 4 alive of 4, MLP 4 alive of 4, and 1 Episode of 20 steps

#### Scenario: Stats follow deaths

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::stats_follow_deaths -->

- **WHEN** a viewer on a Snapshot of 4 Prey with starting energy 1.0, idle drain 0.5 and no food plays 2 steps
- **THEN** its stats report step 2, 0 alive, MLP-Prey 0 alive of 4, and MLP 0 alive of 4

#### Scenario: Stats show the Generation's Evaluation Schedule stage

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::stats_show_the_generations_evaluation_schedule_stage -->

- **WHEN** the Snapshot is of Generation 2 and its schedule is 1 Episode of 4 steps with a stage from Generation 2 of 3 Episodes of 7 steps
- **THEN** its stats report 3 Episodes of 7 steps

### Requirement: The viewer draws its current Showcase

The world view SHALL draw the scene of the viewer's current Showcase, as the scene requirement below defines for any Showcase.

#### Scenario: The viewer's scene follows its Showcase

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::the_viewers_scene_follows_its_showcase -->

- **WHEN** a viewer is opened on a Snapshot and plays 5 steps
- **THEN** its scene equals the scene of its current Showcase
- **AND** that scene differs from the scene at step 0

### Requirement: The drawn scene

The scene of a Showcase SHALL draw each living Agent as its Controller Family's shape (MLP: circle) in its Role's colour, at its position with its body radius and a heading marker, and each food item as a dot of the food radius in a colour distinct from every Role. Dead Agents SHALL not be drawn. Anything overlapping a torus edge SHALL also be drawn shifted by the world's width, height, or both, so it shows on both sides. Scenarios build the Showcase with explicit placements and food.

#### Scenario: Living Prey and food

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::living_prey_and_food -->

- **WHEN** a Showcase in a 50 × 40 world has a living MLP Prey of body radius 1 at (10, 10) heading 0.5, and food of radius 0.5 at (30, 20)
- **THEN** the scene has one Agent shape: a circle in the Prey colour at (10, 10), radius 1, heading 0.5
- **AND** one food dot at (30, 20), radius 0.5, in the food colour, which differs from the Prey colour

#### Scenario: Dead Agents are not drawn

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::dead_agents_are_not_drawn -->

- **WHEN** one of two Prey has died
- **THEN** the scene has exactly one Agent shape, for the living Prey

#### Scenario: Shapes overlapping an edge are drawn on both sides

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::shapes_overlapping_an_edge_are_drawn_on_both_sides -->
<!-- verified-by: crates/viewer/tests/scene.rs::right_edge_is_copied_to_the_left -->
<!-- verified-by: crates/viewer/tests/scene.rs::bottom_edge_is_copied_to_the_top -->
<!-- verified-by: crates/viewer/tests/scene.rs::top_edge_is_copied_to_the_bottom -->
<!-- verified-by: crates/viewer/tests/scene.rs::a_corner_is_copied_to_the_other_three -->
<!-- verified-by: crates/viewer/tests/scene.rs::touching_an_edge_is_not_overlapping_it -->

- **WHEN** a living Prey of body radius 1 is at (0.5, 20) in a 50 × 40 world
- **THEN** the scene draws it at (0.5, 20) and at (50.5, 20)
- **WHEN** it is at (0.5, 39.5)
- **THEN** the scene draws it at (0.5, 39.5), (50.5, 39.5), (0.5, −0.5) and (50.5, −0.5)
- **WHEN** it is at (25, 20)
- **THEN** the scene draws it once

#### Scenario: Food overlapping an edge is drawn on both sides

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::food_overlapping_an_edge_is_drawn_on_both_sides -->
<!-- verified-by: crates/viewer/tests/scene.rs::food_in_a_corner_is_copied_to_the_other_three -->

- **WHEN** food of radius 0.5 is at (49.8, 10) in a 50 × 40 world
- **THEN** the scene draws it at (49.8, 10) and at (−0.2, 10)

### Requirement: Fitness chart data

The viewer SHALL chart, for each Species in the Snapshot's metrics history, a best Fitness line and a mean Fitness line with one point per evaluated Generation, at x = Generation and y = that Generation's value. A Snapshot with no history SHALL give no lines.

#### Scenario: One line pair per Species

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::one_line_pair_per_species -->

- **WHEN** the viewer is opened on a Snapshot whose history has MLP-Prey rows for Generations 0, 1, 2 with best Fitness 4, 6, 5 and mean Fitness 2, 3, 3.5
- **THEN** the chart has exactly one Species, MLP-Prey, with best points (0, 4), (1, 6), (2, 5) and mean points (0, 2), (1, 3), (2, 3.5)

#### Scenario: No history

<!-- verified-by: tests/acceptance_04_viewer_plays_a_showcase.rs::no_history -->

- **WHEN** the viewer is opened on a Generation-0 Snapshot
- **THEN** the chart has no lines
