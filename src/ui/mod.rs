//! Viewer entrypoints.

#[cfg(not(feature = "gui"))]
use crate::io::snapshot::SnapshotFile;

#[cfg(feature = "gui")]
mod bevy_viewer {
    use bevy::math::primitives::{Circle, RegularPolygon};
    use bevy::prelude::*;
    use bevy::prelude::{Mesh2d, MeshMaterial2d};
    use bevy::window::PrimaryWindow;

    use crate::brain::ControllerKind;
    use crate::io::snapshot::SnapshotFile;
    use crate::sim::{FoodState, Vec2, WorldState, step_world};

    #[derive(Component)]
    struct AgentVisual {
        index: usize,
    }

    #[derive(Component)]
    struct FoodVisual {
        id: u32,
    }

    #[derive(Component)]
    struct StartStopButton;

    #[derive(Component)]
    struct StartStopButtonLabel;

    #[derive(Resource, Clone)]
    struct ViewerRuntime {
        initial_snapshot: SnapshotFile,
        world: WorldState,
        config: crate::config::SimulationConfig,
        generation: u32,
        paused: bool,
        speed_multiplier: f32,
        controller_kinds: Vec<ControllerKind>,
    }

    pub fn launch_snapshot_viewer(snapshot: &SnapshotFile) -> anyhow::Result<()> {
        let controller_kinds = snapshot
            .population
            .iter()
            .map(|genome| genome.controller_kind)
            .collect::<Vec<_>>();

        App::new()
            .insert_resource(ViewerRuntime {
                initial_snapshot: snapshot.clone(),
                world: snapshot.world_state.clone(),
                config: snapshot.resolved_config.clone(),
                generation: snapshot.generation,
                paused: true,
                speed_multiplier: 1.0,
                controller_kinds,
            })
            .add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "neural-survival viewer".into(),
                    resolution: (1200.0, 800.0).into(),
                    ..default()
                }),
                ..default()
            }))
            .add_systems(Startup, setup_scene)
            .add_systems(
                Update,
                (
                    keyboard_controls,
                    start_stop_button_interaction,
                    sync_start_stop_button_state,
                    advance_simulation,
                    sync_agent_visuals,
                    sync_food_visuals,
                    update_window_title,
                ),
            )
            .run();

        Ok(())
    }

    fn setup_scene(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        runtime: Res<ViewerRuntime>,
    ) {
        commands.spawn(Camera2d);

        for (idx, agent) in runtime.world.agents.iter().enumerate() {
            let kind = runtime
                .controller_kinds
                .get(idx)
                .copied()
                .unwrap_or(ControllerKind::Mlp);
            let color = match kind {
                ControllerKind::Mlp => Color::srgb(0.2, 0.5, 1.0),
                ControllerKind::Ltc => Color::srgb(1.0, 0.45, 0.1),
            };
            let mesh = match kind {
                ControllerKind::Mlp => meshes.add(Circle::new(4.5)),
                ControllerKind::Ltc => meshes.add(RegularPolygon::new(6.0, 3)),
            };
            let material = materials.add(ColorMaterial::from(color));

            commands.spawn((
                Mesh2d(mesh),
                MeshMaterial2d(material),
                Transform::from_translation(world_to_screen(
                    agent.position,
                    runtime.world.width,
                    runtime.world.height,
                    1.0,
                )),
                AgentVisual { index: idx },
            ));
        }

        spawn_food_entities(
            &mut commands,
            &runtime.world.food,
            runtime.world.width,
            runtime.world.height,
        );

        commands
            .spawn(Node {
                position_type: PositionType::Absolute,
                left: Val::Px(12.0),
                top: Val::Px(12.0),
                ..default()
            })
            .with_children(|parent| {
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(150.0),
                            height: Val::Px(42.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.7, 0.3)),
                        StartStopButton,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("Start"),
                            TextFont::from_font_size(18.0),
                            TextColor(Color::srgb(0.06, 0.06, 0.06)),
                            StartStopButtonLabel,
                        ));
                    });
            });
    }

    fn keyboard_controls(keys: Res<ButtonInput<KeyCode>>, mut runtime: ResMut<ViewerRuntime>) {
        if keys.just_pressed(KeyCode::Space) {
            runtime.paused = !runtime.paused;
        }

        if keys.just_pressed(KeyCode::Equal) {
            runtime.speed_multiplier = (runtime.speed_multiplier * 2.0).min(16.0);
        }

        if keys.just_pressed(KeyCode::Minus) {
            runtime.speed_multiplier = (runtime.speed_multiplier * 0.5).max(0.25);
        }

        if keys.just_pressed(KeyCode::Digit1) {
            runtime.config.rule_toggles.energy_by_speed =
                !runtime.config.rule_toggles.energy_by_speed;
        }

        if keys.just_pressed(KeyCode::Digit2) {
            runtime.config.rule_toggles.energy_by_network_size =
                !runtime.config.rule_toggles.energy_by_network_size;
        }

        if keys.just_pressed(KeyCode::Digit3) {
            runtime.config.rule_toggles.energy_by_perception_size =
                !runtime.config.rule_toggles.energy_by_perception_size;
        }

        if keys.just_pressed(KeyCode::KeyR) {
            runtime.world = runtime.initial_snapshot.world_state.clone();
            runtime.config = runtime.initial_snapshot.resolved_config.clone();
            runtime.generation = runtime.initial_snapshot.generation;
            runtime.paused = true;
            runtime.speed_multiplier = 1.0;
        }
    }

    fn start_stop_button_interaction(
        interactions: Query<
            &Interaction,
            (Changed<Interaction>, With<Button>, With<StartStopButton>),
        >,
        mut runtime: ResMut<ViewerRuntime>,
    ) {
        for interaction in &interactions {
            if matches!(*interaction, Interaction::Pressed) {
                runtime.paused = !runtime.paused;
            }
        }
    }

    fn sync_start_stop_button_state(
        runtime: Res<ViewerRuntime>,
        mut button_query: Query<
            (&Interaction, &mut BackgroundColor),
            (With<Button>, With<StartStopButton>),
        >,
        mut label_query: Query<&mut Text, With<StartStopButtonLabel>>,
    ) {
        for (interaction, mut color) in &mut button_query {
            *color = button_color(runtime.paused, *interaction);
        }

        for mut label in &mut label_query {
            **label = if runtime.paused {
                "Start".to_string()
            } else {
                "Stop".to_string()
            };
        }
    }

    fn button_color(paused: bool, interaction: Interaction) -> BackgroundColor {
        if matches!(interaction, Interaction::Hovered) {
            return BackgroundColor(Color::srgb(0.35, 0.55, 0.85));
        }

        if paused {
            BackgroundColor(Color::srgb(0.2, 0.7, 0.3))
        } else {
            BackgroundColor(Color::srgb(0.8, 0.35, 0.3))
        }
    }

    fn advance_simulation(mut runtime: ResMut<ViewerRuntime>) {
        if runtime.paused {
            return;
        }

        let steps = runtime.speed_multiplier.round().max(1.0) as u32;
        let config = runtime.config.clone();
        for _ in 0..steps {
            step_world(&mut runtime.world, &config);
        }
    }

    fn sync_agent_visuals(
        runtime: Res<ViewerRuntime>,
        mut query: Query<(&AgentVisual, &mut Transform, &mut Visibility)>,
    ) {
        for (visual, mut transform, mut visibility) in &mut query {
            if let Some(agent) = runtime.world.agents.get(visual.index) {
                transform.translation = world_to_screen(
                    agent.position,
                    runtime.world.width,
                    runtime.world.height,
                    1.0,
                );
                *visibility = if agent.alive {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }

    fn sync_food_visuals(
        mut commands: Commands,
        runtime: Res<ViewerRuntime>,
        existing_food: Query<(Entity, &FoodVisual)>,
    ) {
        let world_food_ids = runtime
            .world
            .food
            .iter()
            .map(|food| food.id)
            .collect::<Vec<_>>();

        for (entity, food_visual) in &existing_food {
            if !world_food_ids.contains(&food_visual.id) {
                commands.entity(entity).despawn_recursive();
            }
        }

        for food in &runtime.world.food {
            let exists = existing_food.iter().any(|(_, marker)| marker.id == food.id);
            if !exists {
                spawn_one_food(
                    &mut commands,
                    food,
                    runtime.world.width,
                    runtime.world.height,
                );
            }
        }
    }

    fn update_window_title(
        runtime: Res<ViewerRuntime>,
        mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    ) {
        let Ok(mut window) = window_query.get_single_mut() else {
            return;
        };

        let alive = runtime.world.agents.iter().filter(|a| a.alive).count();
        let food = runtime.world.food.len();
        let total_energy = runtime.world.agents.iter().map(|a| a.energy).sum::<f32>();
        let mean_energy = if runtime.world.agents.is_empty() {
            0.0
        } else {
            total_energy / runtime.world.agents.len() as f32
        };
        let best_energy = runtime
            .world
            .agents
            .iter()
            .map(|a| a.energy)
            .fold(0.0_f32, f32::max);
        let mlp = runtime
            .controller_kinds
            .iter()
            .filter(|kind| **kind == ControllerKind::Mlp)
            .count();
        let ltc = runtime
            .controller_kinds
            .iter()
            .filter(|kind| **kind == ControllerKind::Ltc)
            .count();

        window.title = format!(
            "neural-survival | gen={} alive={} food={} bestE={:.1} meanE={:.1} MLP={} LTC={} speed={}x rules[s:{} n:{} p:{}]",
            runtime.generation,
            alive,
            food,
            best_energy,
            mean_energy,
            mlp,
            ltc,
            runtime.speed_multiplier,
            runtime.config.rule_toggles.energy_by_speed,
            runtime.config.rule_toggles.energy_by_network_size,
            runtime.config.rule_toggles.energy_by_perception_size
        );
    }

    fn spawn_food_entities(commands: &mut Commands, food: &[FoodState], width: f32, height: f32) {
        for item in food {
            spawn_one_food(commands, item, width, height);
        }
    }

    fn spawn_one_food(commands: &mut Commands, food: &FoodState, width: f32, height: f32) {
        commands.spawn((
            Sprite {
                color: Color::srgb(0.1, 0.9, 0.3),
                custom_size: Some(bevy::math::Vec2::new(4.0, 4.0)),
                ..default()
            },
            Transform::from_translation(world_to_screen(food.position, width, height, 0.0)),
            FoodVisual { id: food.id },
        ));
    }

    fn world_to_screen(position: Vec2, width: f32, height: f32, z: f32) -> bevy::math::Vec3 {
        bevy::math::Vec3::new(position.x - width / 2.0, position.y - height / 2.0, z)
    }
}

#[cfg(feature = "gui")]
pub use bevy_viewer::launch_snapshot_viewer;

#[cfg(not(feature = "gui"))]
/// Launch viewer from a snapshot payload.
///
/// Inputs: snapshot state loaded from disk.
/// Output: `Ok(())` after viewer exits.
/// Side effects: in this build profile, prints summary and returns.
pub fn launch_snapshot_viewer(snapshot: &SnapshotFile) -> anyhow::Result<()> {
    println!(
        "viewer stub: generation={}, population={}, world={}x{}",
        snapshot.generation,
        snapshot.population.len(),
        snapshot.world_state.width,
        snapshot.world_state.height
    );
    Ok(())
}
