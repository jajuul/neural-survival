//! Viewer entrypoints.

#[cfg(not(feature = "gui"))]
use crate::io::snapshot::SnapshotFile;

#[cfg(feature = "gui")]
mod bevy_viewer {
    use bevy::input::keyboard::{Key, KeyboardInput};
    use bevy::math::primitives::{Circle, RegularPolygon};
    use bevy::prelude::*;
    use bevy::prelude::{Mesh2d, MeshMaterial2d};
    use bevy::window::PrimaryWindow;

    use crate::brain::ControllerKind;
    use crate::experiment::{continue_generations_from_population, world_state_for_population};
    use crate::genetics::AgentGenome;
    use crate::io::snapshot::SnapshotFile;
    use crate::sim::{FoodState, Vec2, WorldState, step_world};

    const EVOLVE_GENERATIONS_MIN: u32 = 1;
    const EVOLVE_GENERATIONS_MAX: u32 = 5000;
    const WORLD_SIZE_MIN: f32 = 50.0;
    const WORLD_SIZE_MAX: f32 = 5000.0;
    const FOOD_COUNT_MIN: usize = 1;
    const FOOD_COUNT_MAX: usize = 10000;
    const EPISODE_STEPS_MIN: u32 = 10;
    const EPISODE_STEPS_MAX: u32 = 20000;

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

    #[derive(Component)]
    struct EvolveRunButton;

    #[derive(Component)]
    struct EvolveStatusText;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum NumericFieldKind {
        EvolveGenerations,
        WorldWidth,
        WorldHeight,
        FoodCount,
        EpisodeSteps,
        MutationRate,
    }

    const NUMERIC_FIELDS: [NumericFieldKind; 6] = [
        NumericFieldKind::EvolveGenerations,
        NumericFieldKind::WorldWidth,
        NumericFieldKind::WorldHeight,
        NumericFieldKind::FoodCount,
        NumericFieldKind::EpisodeSteps,
        NumericFieldKind::MutationRate,
    ];

    #[derive(Component)]
    struct NumericField {
        kind: NumericFieldKind,
    }

    #[derive(Component)]
    struct NumericFieldValueText {
        kind: NumericFieldKind,
    }

    #[derive(Clone, Debug)]
    struct NumericFieldBuffers {
        evolve_generations: String,
        world_width: String,
        world_height: String,
        food_count: String,
        episode_steps: String,
        mutation_rate: String,
    }

    impl NumericFieldBuffers {
        fn from_state(evolve_generations: u32, config: &crate::config::SimulationConfig) -> Self {
            Self {
                evolve_generations: evolve_generations.to_string(),
                world_width: format!("{:.1}", config.world_width),
                world_height: format!("{:.1}", config.world_height),
                food_count: config.food_count.to_string(),
                episode_steps: config.episode_steps.to_string(),
                mutation_rate: format!("{:.3}", config.mutation_rate),
            }
        }

        fn get(&self, kind: NumericFieldKind) -> &str {
            match kind {
                NumericFieldKind::EvolveGenerations => &self.evolve_generations,
                NumericFieldKind::WorldWidth => &self.world_width,
                NumericFieldKind::WorldHeight => &self.world_height,
                NumericFieldKind::FoodCount => &self.food_count,
                NumericFieldKind::EpisodeSteps => &self.episode_steps,
                NumericFieldKind::MutationRate => &self.mutation_rate,
            }
        }

        fn get_mut(&mut self, kind: NumericFieldKind) -> &mut String {
            match kind {
                NumericFieldKind::EvolveGenerations => &mut self.evolve_generations,
                NumericFieldKind::WorldWidth => &mut self.world_width,
                NumericFieldKind::WorldHeight => &mut self.world_height,
                NumericFieldKind::FoodCount => &mut self.food_count,
                NumericFieldKind::EpisodeSteps => &mut self.episode_steps,
                NumericFieldKind::MutationRate => &mut self.mutation_rate,
            }
        }
    }

    #[derive(Resource, Clone)]
    struct ViewerRuntime {
        initial_snapshot: SnapshotFile,
        population: Vec<AgentGenome>,
        population_generation_index: u32,
        world: WorldState,
        config: crate::config::SimulationConfig,
        generation: u32,
        paused: bool,
        speed_multiplier: f32,
        evolve_generations: u32,
        rebuild_visuals: bool,
        controller_kinds: Vec<ControllerKind>,
        active_input: Option<NumericFieldKind>,
        field_buffers: NumericFieldBuffers,
        evolution_job: Option<EvolutionJob>,
    }

    #[derive(Clone, Debug)]
    struct EvolutionJob {
        total_generations: u32,
        completed_generations: u32,
    }

    pub fn launch_snapshot_viewer(snapshot: &SnapshotFile) -> anyhow::Result<()> {
        let config = snapshot.resolved_config.clone();
        let field_buffers = NumericFieldBuffers::from_state(1, &config);
        let controller_kinds = snapshot
            .population
            .iter()
            .map(|genome| genome.controller_kind)
            .collect::<Vec<_>>();

        App::new()
            .insert_resource(ViewerRuntime {
                initial_snapshot: snapshot.clone(),
                population: snapshot.population.clone(),
                population_generation_index: snapshot.generation.saturating_sub(1),
                world: snapshot.world_state.clone(),
                config,
                generation: snapshot.generation,
                paused: true,
                speed_multiplier: 1.0,
                evolve_generations: 1,
                rebuild_visuals: false,
                controller_kinds,
                active_input: None,
                field_buffers,
                evolution_job: None,
            })
            .add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "neural-survival viewer".into(),
                    resolution: (1280.0, 860.0).into(),
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
                    numeric_field_focus_interaction,
                    numeric_field_keyboard_input,
                    evolve_run_button_interaction,
                    advance_evolution_job,
                    sync_start_stop_button_state,
                    sync_control_panel_state,
                    rebuild_world_visuals_if_requested,
                    advance_simulation,
                    sync_agent_visuals,
                    sync_food_visuals,
                    update_window_title,
                )
                    .chain(),
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

        spawn_agent_entities(
            &mut commands,
            meshes.as_mut(),
            materials.as_mut(),
            &runtime.world,
            &runtime.controller_kinds,
        );

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
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|parent| {
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(180.0),
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

                parent.spawn((
                    Text::new("Continuation Parameters"),
                    TextFont::from_font_size(16.0),
                    TextColor(Color::srgb(0.93, 0.93, 0.93)),
                ));

                for kind in NUMERIC_FIELDS {
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Text::new(field_label(kind)),
                                TextFont::from_font_size(15.0),
                                TextColor(Color::srgb(0.88, 0.88, 0.88)),
                                Node {
                                    width: Val::Px(150.0),
                                    ..default()
                                },
                            ));

                            row.spawn((
                                Button,
                                Node {
                                    width: Val::Px(140.0),
                                    height: Val::Px(30.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                NumericField { kind },
                            ))
                            .with_children(|button| {
                                button.spawn((
                                    Text::new(runtime.field_buffers.get(kind).to_string()),
                                    TextFont::from_font_size(15.0),
                                    TextColor(Color::srgb(0.95, 0.95, 0.95)),
                                    NumericFieldValueText { kind },
                                ));
                            });
                        });
                }

                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(180.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.35, 0.45, 0.8)),
                        EvolveRunButton,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("Evolve"),
                            TextFont::from_font_size(16.0),
                            TextColor(Color::srgb(0.95, 0.95, 0.95)),
                        ));
                    });

                parent.spawn((
                    Text::new("Status: idle"),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgb(0.86, 0.86, 0.86)),
                    EvolveStatusText,
                ));
            });
    }

    fn keyboard_controls(keys: Res<ButtonInput<KeyCode>>, mut runtime: ResMut<ViewerRuntime>) {
        if runtime.active_input.is_some() || runtime.evolution_job.is_some() {
            return;
        }

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
            reset_runtime_to_snapshot(&mut runtime);
        }
    }

    fn start_stop_button_interaction(
        interactions: Query<
            &Interaction,
            (Changed<Interaction>, With<Button>, With<StartStopButton>),
        >,
        mut runtime: ResMut<ViewerRuntime>,
    ) {
        if runtime.evolution_job.is_some() {
            return;
        }

        for interaction in &interactions {
            if matches!(*interaction, Interaction::Pressed) {
                runtime.active_input = None;
                runtime.paused = !runtime.paused;
            }
        }
    }

    fn numeric_field_focus_interaction(
        interactions: Query<
            (&Interaction, &NumericField),
            (Changed<Interaction>, With<Button>, With<NumericField>),
        >,
        mut runtime: ResMut<ViewerRuntime>,
    ) {
        if runtime.evolution_job.is_some() {
            return;
        }

        for (interaction, field) in &interactions {
            if matches!(*interaction, Interaction::Pressed) {
                runtime.active_input = Some(field.kind);
            }
        }
    }

    fn numeric_field_keyboard_input(
        mut events: EventReader<KeyboardInput>,
        mut runtime: ResMut<ViewerRuntime>,
    ) {
        let Some(active_field) = runtime.active_input else {
            return;
        };

        for event in events.read() {
            if !event.state.is_pressed() {
                continue;
            }

            match &event.logical_key {
                Key::Escape => {
                    runtime.field_buffers = NumericFieldBuffers::from_state(
                        runtime.evolve_generations,
                        &runtime.config,
                    );
                    runtime.active_input = None;
                }
                Key::Enter | Key::Tab => {
                    apply_numeric_field_from_buffer(&mut runtime, active_field);
                    normalize_numeric_field_buffer(&mut runtime, active_field);
                    runtime.active_input = None;
                }
                Key::Backspace => {
                    runtime.field_buffers.get_mut(active_field).pop();
                    apply_numeric_field_from_buffer(&mut runtime, active_field);
                }
                Key::Character(chars) => {
                    let mut changed = false;
                    {
                        let buffer = runtime.field_buffers.get_mut(active_field);
                        for ch in chars.chars() {
                            if accepts_numeric_char(active_field, ch, buffer) {
                                buffer.push(ch);
                                changed = true;
                            }
                        }
                    }

                    if changed {
                        apply_numeric_field_from_buffer(&mut runtime, active_field);
                    }
                }
                _ => {}
            }
        }
    }

    fn evolve_run_button_interaction(
        interactions: Query<
            &Interaction,
            (Changed<Interaction>, With<Button>, With<EvolveRunButton>),
        >,
        mut runtime: ResMut<ViewerRuntime>,
    ) {
        if runtime.evolution_job.is_some() {
            return;
        }

        let run_pressed = interactions
            .iter()
            .any(|interaction| matches!(*interaction, Interaction::Pressed));
        if !run_pressed {
            return;
        }

        runtime.active_input = None;
        apply_all_numeric_buffers(&mut runtime);
        for field in NUMERIC_FIELDS {
            normalize_numeric_field_buffer(&mut runtime, field);
        }

        let generations_to_advance = runtime.evolve_generations;
        if generations_to_advance == 0 {
            return;
        }

        runtime.paused = true;
        runtime.evolution_job = Some(EvolutionJob {
            total_generations: generations_to_advance,
            completed_generations: 0,
        });
    }

    fn advance_evolution_job(mut runtime: ResMut<ViewerRuntime>) {
        let Some(mut job) = runtime.evolution_job.take() else {
            return;
        };

        let run = continue_generations_from_population(
            &runtime.config,
            runtime.initial_snapshot.seed,
            runtime.population_generation_index,
            &runtime.population,
            1,
        );

        runtime.population = run.final_population;
        runtime.population_generation_index = runtime.population_generation_index.saturating_add(1);
        runtime.generation = runtime.generation.saturating_add(1);
        job.completed_generations = job.completed_generations.saturating_add(1);

        if job.completed_generations >= job.total_generations {
            runtime.controller_kinds = runtime
                .population
                .iter()
                .map(|genome| genome.controller_kind)
                .collect::<Vec<_>>();
            runtime.world = world_state_for_population(
                &runtime.config,
                runtime.initial_snapshot.seed,
                runtime.generation,
                &runtime.population,
            );
            runtime.rebuild_visuals = true;
            runtime.paused = true;
        } else {
            runtime.evolution_job = Some(job);
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
            *color = start_stop_button_color(runtime.paused, *interaction);
        }

        for mut label in &mut label_query {
            **label = if runtime.paused {
                "Start".to_string()
            } else {
                "Stop".to_string()
            };
        }
    }

    fn sync_control_panel_state(
        runtime: Res<ViewerRuntime>,
        mut button_query: Query<
            (
                &Interaction,
                &mut BackgroundColor,
                Option<&NumericField>,
                Option<&EvolveRunButton>,
            ),
            With<Button>,
        >,
        mut text_query: Query<(
            &mut Text,
            Option<&NumericFieldValueText>,
            Option<&EvolveStatusText>,
        )>,
    ) {
        let is_evolving = runtime.evolution_job.is_some();
        for (interaction, mut color, maybe_field, maybe_evolve) in &mut button_query {
            if let Some(field) = maybe_field {
                *color = numeric_field_color(
                    *interaction,
                    runtime
                        .active_input
                        .is_some_and(|active| active == field.kind),
                );
            } else if maybe_evolve.is_some() {
                *color = evolve_run_button_color(*interaction, is_evolving);
            }
        }

        let status = if let Some(job) = &runtime.evolution_job {
            format!(
                "Status: evolving {}/{} generations",
                job.completed_generations, job.total_generations
            )
        } else {
            format!(
                "Status: ready (next evolve: {} generations)",
                runtime.evolve_generations
            )
        };

        for (mut text, value_marker, status_marker) in &mut text_query {
            if let Some(marker) = value_marker {
                **text = runtime.field_buffers.get(marker.kind).to_string();
            } else if status_marker.is_some() {
                **text = status.clone();
            }
        }
    }

    fn start_stop_button_color(paused: bool, interaction: Interaction) -> BackgroundColor {
        if matches!(interaction, Interaction::Hovered) {
            return BackgroundColor(Color::srgb(0.35, 0.55, 0.85));
        }

        if paused {
            BackgroundColor(Color::srgb(0.2, 0.7, 0.3))
        } else {
            BackgroundColor(Color::srgb(0.8, 0.35, 0.3))
        }
    }

    fn numeric_field_color(interaction: Interaction, active: bool) -> BackgroundColor {
        if active {
            return BackgroundColor(Color::srgb(0.2, 0.45, 0.75));
        }

        match interaction {
            Interaction::Pressed => BackgroundColor(Color::srgb(0.42, 0.42, 0.42)),
            Interaction::Hovered => BackgroundColor(Color::srgb(0.32, 0.32, 0.32)),
            Interaction::None => BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
        }
    }

    fn evolve_run_button_color(interaction: Interaction, is_evolving: bool) -> BackgroundColor {
        if is_evolving {
            return BackgroundColor(Color::srgb(0.55, 0.45, 0.25));
        }

        match interaction {
            Interaction::Pressed => BackgroundColor(Color::srgb(0.2, 0.65, 0.45)),
            Interaction::Hovered => BackgroundColor(Color::srgb(0.4, 0.5, 0.85)),
            Interaction::None => BackgroundColor(Color::srgb(0.35, 0.45, 0.8)),
        }
    }

    fn rebuild_world_visuals_if_requested(
        mut commands: Commands,
        mut runtime: ResMut<ViewerRuntime>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        agent_entities: Query<Entity, With<AgentVisual>>,
        food_entities: Query<Entity, With<FoodVisual>>,
    ) {
        if !runtime.rebuild_visuals {
            return;
        }

        runtime.rebuild_visuals = false;
        let world = runtime.world.clone();
        let controller_kinds = runtime.controller_kinds.clone();

        for entity in &agent_entities {
            commands.entity(entity).despawn_recursive();
        }

        for entity in &food_entities {
            commands.entity(entity).despawn_recursive();
        }

        spawn_agent_entities(
            &mut commands,
            meshes.as_mut(),
            materials.as_mut(),
            &world,
            &controller_kinds,
        );
        spawn_food_entities(&mut commands, &world.food, world.width, world.height);
    }

    fn advance_simulation(mut runtime: ResMut<ViewerRuntime>) {
        if runtime.paused || runtime.evolution_job.is_some() {
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
        let evolution_progress = if let Some(job) = &runtime.evolution_job {
            format!(
                " evolving={}/{}",
                job.completed_generations, job.total_generations
            )
        } else {
            String::new()
        };

        window.title = format!(
            "neural-survival | gen={} alive={} food={} bestE={:.1} meanE={:.1} MLP={} LTC={} speed={}x world={:.0}x{:.0} rules[s:{} n:{} p:{}]{}",
            runtime.generation,
            alive,
            food,
            best_energy,
            mean_energy,
            mlp,
            ltc,
            runtime.speed_multiplier,
            runtime.config.world_width,
            runtime.config.world_height,
            runtime.config.rule_toggles.energy_by_speed,
            runtime.config.rule_toggles.energy_by_network_size,
            runtime.config.rule_toggles.energy_by_perception_size,
            evolution_progress
        );
    }

    fn spawn_food_entities(commands: &mut Commands, food: &[FoodState], width: f32, height: f32) {
        for item in food {
            spawn_one_food(commands, item, width, height);
        }
    }

    fn spawn_agent_entities(
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
        world: &WorldState,
        controller_kinds: &[ControllerKind],
    ) {
        for (idx, agent) in world.agents.iter().enumerate() {
            let kind = controller_kinds
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
                    world.width,
                    world.height,
                    1.0,
                )),
                AgentVisual { index: idx },
            ));
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

    fn field_label(kind: NumericFieldKind) -> &'static str {
        match kind {
            NumericFieldKind::EvolveGenerations => "Generations",
            NumericFieldKind::WorldWidth => "World Width",
            NumericFieldKind::WorldHeight => "World Height",
            NumericFieldKind::FoodCount => "Food Count",
            NumericFieldKind::EpisodeSteps => "Episode Steps",
            NumericFieldKind::MutationRate => "Mutation Rate",
        }
    }

    fn accepts_numeric_char(kind: NumericFieldKind, ch: char, current: &str) -> bool {
        if ch.is_ascii_digit() {
            return true;
        }

        matches!(
            kind,
            NumericFieldKind::WorldWidth
                | NumericFieldKind::WorldHeight
                | NumericFieldKind::MutationRate
        ) && ch == '.'
            && !current.contains('.')
    }

    fn apply_all_numeric_buffers(runtime: &mut ViewerRuntime) {
        for field in NUMERIC_FIELDS {
            apply_numeric_field_from_buffer(runtime, field);
        }
    }

    fn apply_numeric_field_from_buffer(runtime: &mut ViewerRuntime, field: NumericFieldKind) {
        let raw = runtime.field_buffers.get(field).trim().to_string();
        if raw.is_empty() {
            return;
        }

        match field {
            NumericFieldKind::EvolveGenerations => {
                if let Ok(value) = raw.parse::<u32>() {
                    runtime.evolve_generations =
                        value.clamp(EVOLVE_GENERATIONS_MIN, EVOLVE_GENERATIONS_MAX);
                }
            }
            NumericFieldKind::WorldWidth => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.world_width = value.clamp(WORLD_SIZE_MIN, WORLD_SIZE_MAX);
                }
            }
            NumericFieldKind::WorldHeight => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.world_height = value.clamp(WORLD_SIZE_MIN, WORLD_SIZE_MAX);
                }
            }
            NumericFieldKind::FoodCount => {
                if let Ok(value) = raw.parse::<usize>() {
                    runtime.config.food_count = value.clamp(FOOD_COUNT_MIN, FOOD_COUNT_MAX);
                }
            }
            NumericFieldKind::EpisodeSteps => {
                if let Ok(value) = raw.parse::<u32>() {
                    runtime.config.episode_steps =
                        value.clamp(EPISODE_STEPS_MIN, EPISODE_STEPS_MAX);
                }
            }
            NumericFieldKind::MutationRate => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.mutation_rate = value.clamp(0.0, 1.0);
                }
            }
        }
    }

    fn normalize_numeric_field_buffer(runtime: &mut ViewerRuntime, field: NumericFieldKind) {
        match field {
            NumericFieldKind::EvolveGenerations => {
                runtime.field_buffers.evolve_generations = runtime.evolve_generations.to_string();
            }
            NumericFieldKind::WorldWidth => {
                runtime.field_buffers.world_width = format!("{:.1}", runtime.config.world_width);
            }
            NumericFieldKind::WorldHeight => {
                runtime.field_buffers.world_height = format!("{:.1}", runtime.config.world_height);
            }
            NumericFieldKind::FoodCount => {
                runtime.field_buffers.food_count = runtime.config.food_count.to_string();
            }
            NumericFieldKind::EpisodeSteps => {
                runtime.field_buffers.episode_steps = runtime.config.episode_steps.to_string();
            }
            NumericFieldKind::MutationRate => {
                runtime.field_buffers.mutation_rate =
                    format!("{:.3}", runtime.config.mutation_rate);
            }
        }
    }

    fn reset_runtime_to_snapshot(runtime: &mut ViewerRuntime) {
        runtime.world = runtime.initial_snapshot.world_state.clone();
        runtime.config = runtime.initial_snapshot.resolved_config.clone();
        runtime.population = runtime.initial_snapshot.population.clone();
        runtime.generation = runtime.initial_snapshot.generation;
        runtime.population_generation_index = runtime.initial_snapshot.generation.saturating_sub(1);
        runtime.paused = true;
        runtime.speed_multiplier = 1.0;
        runtime.evolve_generations = 1;
        runtime.controller_kinds = runtime
            .population
            .iter()
            .map(|genome| genome.controller_kind)
            .collect::<Vec<_>>();
        runtime.active_input = None;
        runtime.field_buffers =
            NumericFieldBuffers::from_state(runtime.evolve_generations, &runtime.config);
        runtime.evolution_job = None;
        runtime.rebuild_visuals = true;
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
