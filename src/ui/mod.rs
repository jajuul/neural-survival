//! Viewer entrypoints.

#[cfg(not(feature = "gui"))]
use crate::io::snapshot::SnapshotFile;

#[cfg(feature = "gui")]
mod bevy_viewer {
    use bevy::input::keyboard::{Key, KeyboardInput};
    use bevy::input::mouse::MouseButton;
    use bevy::math::primitives::{Circle, RegularPolygon};
    use bevy::prelude::*;
    use bevy::prelude::{Mesh2d, MeshMaterial2d};
    use bevy::window::PrimaryWindow;
    use std::collections::BTreeMap;

    use crate::brain::{
        ControllerDiagnostics, ControllerFactory, ControllerKind, ControllerNetworkSnapshot,
        ControllerStepOutput, NeuralController,
    };
    use crate::config::EvaluationMode;
    use crate::experiment::{
        GenerationMetrics, continue_generations_from_population, world_state_for_population,
    };
    use crate::genetics::{AgentGenome, default_mixed_factory};
    use crate::io::snapshot::SnapshotFile;
    use crate::sim::{
        EnergyBreakdown, FoodState, SensorObservation, Vec2, WorldState,
        cluster_density_radius_scale, cluster_productivity, current_day_night_state,
        sensor_observation, step_world_with_actions_and_telemetry, wrap_position,
    };

    const EVOLVE_GENERATIONS_MIN: u32 = 1;
    const EVOLVE_GENERATIONS_MAX: u32 = 5000;
    const WORLD_SIZE_MIN: f32 = 50.0;
    const WORLD_SIZE_MAX: f32 = 5000.0;
    const FOOD_COUNT_MIN: usize = 1;
    const FOOD_COUNT_MAX: usize = 10000;
    const FOOD_MOTION_SPEED_MAX: f32 = 50.0;
    const MAX_ACCELERATION_MAX: f32 = 100.0;
    const LINEAR_DRAG_MAX: f32 = 20.0;
    const EPISODE_STEPS_MIN: u32 = 10;
    const EPISODE_STEPS_MAX: u32 = 20000;
    const SENSOR_NOISE_STDDEV_MAX: f32 = 2.0;
    const FITNESS_PANEL_WIDTH: f32 = 460.0;
    const FITNESS_PANEL_HEIGHT: f32 = 240.0;
    const FITNESS_PANEL_MARGIN: f32 = 12.0;
    const FITNESS_PLOT_INSET_LEFT: f32 = 18.0;
    const FITNESS_PLOT_INSET_RIGHT: f32 = 16.0;
    const FITNESS_PLOT_INSET_TOP: f32 = 64.0;
    const FITNESS_PLOT_INSET_BOTTOM: f32 = 22.0;
    const FITNESS_PLOT_MAX_POINTS: usize = 240;
    const OBSERVABILITY_PANEL_WIDTH: f32 = 460.0;
    const OBSERVABILITY_PANEL_HEIGHT: f32 = 330.0;
    const OBSERVABILITY_PANEL_GAP: f32 = 12.0;
    const OBSERVABILITY_EYE_PLOT_INSET_LEFT: f32 = 14.0;
    const OBSERVABILITY_EYE_PLOT_INSET_RIGHT: f32 = 14.0;
    const OBSERVABILITY_EYE_PLOT_INSET_TOP: f32 = 240.0;
    const OBSERVABILITY_EYE_PLOT_INSET_BOTTOM: f32 = 18.0;
    const NETWORK_PANEL_WIDTH: f32 = 620.0;
    const NETWORK_PANEL_HEIGHT: f32 = 320.0;
    const NETWORK_PANEL_GAP: f32 = 12.0;
    const NETWORK_PLOT_INSET_LEFT: f32 = 28.0;
    const NETWORK_PLOT_INSET_RIGHT: f32 = 28.0;
    const NETWORK_PLOT_INSET_TOP: f32 = 86.0;
    const NETWORK_PLOT_INSET_BOTTOM: f32 = 28.0;
    const SELECTION_PICK_RADIUS: f32 = 12.0;
    const CONTROL_LABEL_WIDTH: f32 = 120.0;
    const CONTROL_VALUE_WIDTH: f32 = 110.0;
    const CONTROL_COLUMN_WIDTH: f32 = 252.0;
    const CONTROL_PANEL_GAP: f32 = 16.0;
    const CONTROL_PANEL_TOP: f32 = 12.0;
    const CONTROL_PANEL_LEFT: f32 = 12.0;
    const TIMESTEP_SECONDS_MAX: f32 = 10.0;
    const POPULATION_SIZE_MAX: usize = 4096;
    const FOOD_CLUSTER_COUNT_MAX: usize = 256;
    const FOOD_CLUSTER_RADIUS_MAX: f32 = 500.0;
    const FOOD_CLUSTER_MOTION_SPEED_MAX: f32 = 100.0;
    const TOURNAMENT_SIZE_MAX: usize = 4096;
    const INITIAL_AGENT_ENERGY_MAX: f32 = 10_000.0;
    const ENERGY_COST_MAX: f32 = 100.0;
    const FOOD_ENERGY_GAIN_MAX: f32 = 10_000.0;
    const FOOD_COLLISION_RADIUS_MAX: f32 = 100.0;
    const EYE_FOV_MAX: f32 = std::f32::consts::TAU;
    const DEFAULT_SPEED_MAX: f32 = 100.0;
    const TURN_RATE_MAX: f32 = std::f32::consts::TAU * 10.0;
    const PERCEPTION_RADIUS_MAX: f32 = 500.0;
    const NETWORK_SIZE_MAX: usize = 100_000;
    const HIDDEN_LAYER_SIZE_MAX: usize = 1024;
    const METABOLIC_EFFICIENCY_MAX: f32 = 10.0;
    const DAY_NIGHT_CYCLE_STEPS_MAX: u32 = 1_000_000;
    const FITNESS_WEIGHT_MAX: f32 = 1_000.0;

    #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
    enum ViewerUpdateSet {
        ControlsPrimary,
        ControlsSecondary,
        StatePrimary,
        StateSecondary,
        Draw,
    }

    #[derive(Component)]
    struct AgentVisual {
        index: usize,
    }

    #[derive(Component)]
    struct AgentHeadingMarker;

    #[derive(Component)]
    struct FoodVisual {
        id: u32,
    }

    #[derive(Component)]
    struct PauseButton;

    #[derive(Component)]
    struct ContinueButton;

    #[derive(Component)]
    struct StepButton;

    #[derive(Component)]
    struct EvolveRunButton;

    #[derive(Component)]
    struct EvolveStatusText;

    #[derive(Component)]
    struct EvaluationModeButton;

    #[derive(Component)]
    struct EvaluationModeButtonLabel;

    #[derive(Component)]
    struct EvaluationModeAfterSwitchButton;

    #[derive(Component)]
    struct EvaluationModeAfterSwitchButtonLabel;

    #[derive(Component)]
    struct ToggleField {
        kind: ToggleFieldKind,
    }

    #[derive(Component)]
    struct ToggleFieldValueText {
        kind: ToggleFieldKind,
    }

    #[derive(Component)]
    struct FitnessPlotSummaryText;

    #[derive(Component)]
    struct SelectedAgentSummaryText;

    #[derive(Component)]
    struct SelectedAgentNetworkSummaryText;

    #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
    enum NumericFieldKind {
        EvolveGenerations,
        EvaluationSwitchAfterGeneration,
        WorldWidth,
        WorldHeight,
        TimestepSeconds,
        PopulationSize,
        FoodCount,
        FoodMotionSpeed,
        FoodRespawnProbability,
        FoodClusterCount,
        FoodClusterRadius,
        FoodClusterMotionSpeed,
        FoodClusterProductivityBias,
        ElitismFraction,
        TournamentSize,
        InitialAgentEnergy,
        BaseEnergyCostPerSecond,
        SpeedEnergyCostFactor,
        NetworkEnergyCostFactor,
        PerceptionEnergyCostFactor,
        FoodEnergyGain,
        FoodCollisionRadius,
        EyeFovAngleRadians,
        DefaultAgentMaxSpeed,
        MaxAcceleration,
        LinearDrag,
        MaxTurnRateRadians,
        DefaultPerceptionRadius,
        DefaultPerceptionChannels,
        DefaultNetworkSize,
        MlpHiddenSize,
        LtcStateSize,
        DefaultMetabolicEfficiency,
        ControllerLtcProbability,
        PredatorFraction,
        PredatorCaptureRadius,
        PredatorCaptureEnergyGain,
        SensorNoiseStddev,
        SensorDropoutProbability,
        DayNightCycleSteps,
        NightVisibilityMultiplier,
        EpisodeSteps,
        MutationRate,
        FitnessWeightSurvival,
        FitnessWeightFood,
        FitnessWeightPredation,
        FitnessWeightEnergy,
    }

    const NUMERIC_FIELDS: [NumericFieldKind; 47] = [
        NumericFieldKind::EvolveGenerations,
        NumericFieldKind::EvaluationSwitchAfterGeneration,
        NumericFieldKind::WorldWidth,
        NumericFieldKind::WorldHeight,
        NumericFieldKind::TimestepSeconds,
        NumericFieldKind::PopulationSize,
        NumericFieldKind::FoodCount,
        NumericFieldKind::FoodMotionSpeed,
        NumericFieldKind::FoodRespawnProbability,
        NumericFieldKind::FoodClusterCount,
        NumericFieldKind::FoodClusterRadius,
        NumericFieldKind::FoodClusterMotionSpeed,
        NumericFieldKind::FoodClusterProductivityBias,
        NumericFieldKind::ElitismFraction,
        NumericFieldKind::TournamentSize,
        NumericFieldKind::InitialAgentEnergy,
        NumericFieldKind::BaseEnergyCostPerSecond,
        NumericFieldKind::SpeedEnergyCostFactor,
        NumericFieldKind::NetworkEnergyCostFactor,
        NumericFieldKind::PerceptionEnergyCostFactor,
        NumericFieldKind::FoodEnergyGain,
        NumericFieldKind::FoodCollisionRadius,
        NumericFieldKind::EyeFovAngleRadians,
        NumericFieldKind::DefaultAgentMaxSpeed,
        NumericFieldKind::MaxAcceleration,
        NumericFieldKind::LinearDrag,
        NumericFieldKind::MaxTurnRateRadians,
        NumericFieldKind::DefaultPerceptionRadius,
        NumericFieldKind::DefaultPerceptionChannels,
        NumericFieldKind::DefaultNetworkSize,
        NumericFieldKind::MlpHiddenSize,
        NumericFieldKind::LtcStateSize,
        NumericFieldKind::DefaultMetabolicEfficiency,
        NumericFieldKind::ControllerLtcProbability,
        NumericFieldKind::PredatorFraction,
        NumericFieldKind::PredatorCaptureRadius,
        NumericFieldKind::PredatorCaptureEnergyGain,
        NumericFieldKind::SensorNoiseStddev,
        NumericFieldKind::SensorDropoutProbability,
        NumericFieldKind::DayNightCycleSteps,
        NumericFieldKind::NightVisibilityMultiplier,
        NumericFieldKind::EpisodeSteps,
        NumericFieldKind::MutationRate,
        NumericFieldKind::FitnessWeightSurvival,
        NumericFieldKind::FitnessWeightFood,
        NumericFieldKind::FitnessWeightPredation,
        NumericFieldKind::FitnessWeightEnergy,
    ];

    const CONTROL_COLUMN_1_FIELDS: [NumericFieldKind; 15] = [
        NumericFieldKind::EvolveGenerations,
        NumericFieldKind::EvaluationSwitchAfterGeneration,
        NumericFieldKind::WorldWidth,
        NumericFieldKind::WorldHeight,
        NumericFieldKind::TimestepSeconds,
        NumericFieldKind::EpisodeSteps,
        NumericFieldKind::PopulationSize,
        NumericFieldKind::FoodCount,
        NumericFieldKind::FoodMotionSpeed,
        NumericFieldKind::FoodRespawnProbability,
        NumericFieldKind::FoodClusterCount,
        NumericFieldKind::FoodClusterRadius,
        NumericFieldKind::FoodClusterMotionSpeed,
        NumericFieldKind::FoodClusterProductivityBias,
        NumericFieldKind::InitialAgentEnergy,
    ];

    const CONTROL_COLUMN_2_FIELDS: [NumericFieldKind; 17] = [
        NumericFieldKind::ElitismFraction,
        NumericFieldKind::TournamentSize,
        NumericFieldKind::MutationRate,
        NumericFieldKind::BaseEnergyCostPerSecond,
        NumericFieldKind::SpeedEnergyCostFactor,
        NumericFieldKind::NetworkEnergyCostFactor,
        NumericFieldKind::PerceptionEnergyCostFactor,
        NumericFieldKind::FoodEnergyGain,
        NumericFieldKind::FoodCollisionRadius,
        NumericFieldKind::FitnessWeightSurvival,
        NumericFieldKind::FitnessWeightFood,
        NumericFieldKind::FitnessWeightPredation,
        NumericFieldKind::FitnessWeightEnergy,
        NumericFieldKind::PredatorFraction,
        NumericFieldKind::PredatorCaptureRadius,
        NumericFieldKind::PredatorCaptureEnergyGain,
        NumericFieldKind::ControllerLtcProbability,
    ];

    const CONTROL_COLUMN_3_FIELDS: [NumericFieldKind; 15] = [
        NumericFieldKind::EyeFovAngleRadians,
        NumericFieldKind::DefaultAgentMaxSpeed,
        NumericFieldKind::MaxAcceleration,
        NumericFieldKind::LinearDrag,
        NumericFieldKind::MaxTurnRateRadians,
        NumericFieldKind::DefaultPerceptionRadius,
        NumericFieldKind::DefaultPerceptionChannels,
        NumericFieldKind::DefaultNetworkSize,
        NumericFieldKind::MlpHiddenSize,
        NumericFieldKind::LtcStateSize,
        NumericFieldKind::DefaultMetabolicEfficiency,
        NumericFieldKind::SensorNoiseStddev,
        NumericFieldKind::SensorDropoutProbability,
        NumericFieldKind::DayNightCycleSteps,
        NumericFieldKind::NightVisibilityMultiplier,
    ];

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum ToggleFieldKind {
        FoodClusterEnabled,
        DayNightCycleEnabled,
        EnergyBySpeed,
        EnergyByNetworkSize,
        EnergyByPerceptionSize,
    }

    #[derive(Component)]
    struct NumericField {
        kind: NumericFieldKind,
    }

    #[derive(Component)]
    struct NumericFieldValueText {
        kind: NumericFieldKind,
    }

    #[derive(Clone, Debug)]
    struct NumericFieldBuffers(BTreeMap<NumericFieldKind, String>);

    impl NumericFieldBuffers {
        fn from_state(evolve_generations: u32, config: &crate::config::SimulationConfig) -> Self {
            let mut values = BTreeMap::new();
            for kind in NUMERIC_FIELDS {
                values.insert(kind, numeric_field_string(kind, evolve_generations, config));
            }
            Self(values)
        }

        fn get(&self, kind: NumericFieldKind) -> &str {
            self.0
                .get(&kind)
                .map(String::as_str)
                .expect("all numeric fields must have buffer entries")
        }

        fn get_mut(&mut self, kind: NumericFieldKind) -> &mut String {
            self.0
                .get_mut(&kind)
                .expect("all numeric fields must have buffer entries")
        }
    }

    fn visual_heading_rotation(heading_radians: f32) -> Quat {
        // The sim uses 0 rad = +X, while Bevy's regular polygon mesh points along +Y.
        Quat::from_rotation_z(heading_radians - std::f32::consts::FRAC_PI_2)
    }

    #[derive(Clone, Debug)]
    struct SelectedAgentSnapshot {
        agent_id: u32,
        controller_kind: ControllerKind,
        role: crate::sim::AgentRole,
        alive: bool,
        world_step: u64,
        perception_radius: f32,
        perception_channels: u16,
        eye_cell_angle_radians: f32,
        sensor: SensorObservation,
        controller_output: ControllerStepOutput,
        network: ControllerNetworkSnapshot,
        energy: EnergyBreakdown,
        consumed_food: bool,
        has_live_step: bool,
    }

    #[derive(Resource)]
    struct ViewerRuntime {
        initial_snapshot: SnapshotFile,
        population: Vec<AgentGenome>,
        controllers: Vec<Box<dyn NeuralController>>,
        population_generation_index: u32,
        world: WorldState,
        config: crate::config::SimulationConfig,
        generation: u32,
        paused: bool,
        speed_multiplier: f32,
        evolve_generations: u32,
        rebuild_visuals: bool,
        controller_kinds: Vec<ControllerKind>,
        metrics_history: Vec<GenerationMetrics>,
        active_input: Option<NumericFieldKind>,
        field_buffers: NumericFieldBuffers,
        selected_agent_index: Option<usize>,
        selected_agent_snapshot: Option<SelectedAgentSnapshot>,
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
        let controllers = build_runtime_controllers(&config, &snapshot.population);

        App::new()
            .insert_resource(ViewerRuntime {
                initial_snapshot: snapshot.clone(),
                population: snapshot.population.clone(),
                controllers,
                population_generation_index: snapshot.generation.saturating_sub(1),
                world: snapshot.world_state.clone(),
                config,
                generation: snapshot.generation,
                paused: true,
                speed_multiplier: 1.0,
                evolve_generations: 1,
                rebuild_visuals: false,
                controller_kinds,
                metrics_history: snapshot.metrics_history.clone(),
                active_input: None,
                field_buffers,
                selected_agent_index: None,
                selected_agent_snapshot: None,
                evolution_job: None,
            })
            .add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "neural-survival viewer".into(),
                    resolution: (1860.0, 1120.0).into(),
                    ..default()
                }),
                ..default()
            }))
            .configure_sets(
                Update,
                (
                    ViewerUpdateSet::ControlsPrimary,
                    ViewerUpdateSet::ControlsSecondary,
                    ViewerUpdateSet::StatePrimary,
                    ViewerUpdateSet::StateSecondary,
                    ViewerUpdateSet::Draw,
                )
                    .chain(),
            )
            .add_systems(Startup, setup_scene)
            .add_systems(
                Update,
                ((
                    keyboard_controls,
                    pause_button_interaction,
                    continue_button_interaction,
                    step_button_interaction,
                    toggle_field_interaction,
                    evaluation_mode_button_interaction,
                    evaluation_mode_after_switch_button_interaction,
                )
                    .chain()
                    .in_set(ViewerUpdateSet::ControlsPrimary),),
            )
            .add_systems(
                Update,
                (
                    numeric_field_focus_interaction,
                    numeric_field_keyboard_input,
                    evolve_run_button_interaction,
                    agent_selection_input,
                    advance_evolution_job,
                )
                    .chain()
                    .in_set(ViewerUpdateSet::ControlsSecondary),
            )
            .add_systems(
                Update,
                ((
                    sync_simulation_control_state,
                    sync_control_panel_state,
                    sync_fitness_plot_summary,
                    sync_selected_agent_network_summary,
                    rebuild_world_visuals_if_requested,
                )
                    .chain()
                    .in_set(ViewerUpdateSet::StatePrimary),),
            )
            .add_systems(
                Update,
                ((
                    advance_simulation,
                    sync_selected_agent_summary,
                    sync_agent_visuals,
                    sync_food_visuals,
                    update_window_title,
                )
                    .chain()
                    .in_set(ViewerUpdateSet::StateSecondary),),
            )
            .add_systems(
                Update,
                ((
                    draw_food_clusters,
                    draw_fitness_plot,
                    draw_selected_agent_eye_bins,
                    draw_selected_agent_network,
                )
                    .in_set(ViewerUpdateSet::Draw),),
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
                left: Val::Px(CONTROL_PANEL_LEFT),
                top: Val::Px(CONTROL_PANEL_TOP),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|parent| {
                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Text::new("Evaluation"),
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
                            BackgroundColor(Color::srgb(0.35, 0.35, 0.42)),
                            EvaluationModeButton,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new(format_evaluation_mode(runtime.config.evaluation_mode)),
                                TextFont::from_font_size(15.0),
                                TextColor(Color::srgb(0.95, 0.95, 0.95)),
                                EvaluationModeButtonLabel,
                            ));
                        });
                    });

                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Text::new("After Switch"),
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
                            BackgroundColor(Color::srgb(0.35, 0.35, 0.42)),
                            EvaluationModeAfterSwitchButton,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new(format_evaluation_mode(
                                    runtime.config.evaluation_mode_after_switch,
                                )),
                                TextFont::from_font_size(15.0),
                                TextColor(Color::srgb(0.95, 0.95, 0.95)),
                                EvaluationModeAfterSwitchButtonLabel,
                            ));
                        });
                    });

                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Button,
                            Node {
                                width: Val::Px(56.0),
                                height: Val::Px(36.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.7, 0.3, 0.25)),
                            PauseButton,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Pause"),
                                TextFont::from_font_size(14.0),
                                TextColor(Color::srgb(0.95, 0.95, 0.95)),
                            ));
                        });

                        row.spawn((
                            Button,
                            Node {
                                width: Val::Px(56.0),
                                height: Val::Px(36.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.22, 0.65, 0.35)),
                            StepButton,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Step"),
                                TextFont::from_font_size(14.0),
                                TextColor(Color::srgb(0.95, 0.95, 0.95)),
                            ));
                        });

                        row.spawn((
                            Button,
                            Node {
                                width: Val::Px(68.0),
                                height: Val::Px(36.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.45, 0.78)),
                            ContinueButton,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Continue"),
                                TextFont::from_font_size(14.0),
                                TextColor(Color::srgb(0.95, 0.95, 0.95)),
                            ));
                        });
                    });

                parent.spawn((
                    Text::new("Continuation Parameters"),
                    TextFont::from_font_size(16.0),
                    TextColor(Color::srgb(0.93, 0.93, 0.93)),
                ));
                parent.spawn((
                    Text::new(
                        "Edits apply to the next GUI evolution run. Default/architecture fields affect newly initialized agents, not already-evolved genomes.",
                    ),
                    TextFont::from_font_size(11.0),
                    TextColor(Color::srgb(0.75, 0.8, 0.86)),
                    Node {
                        width: Val::Px(CONTROL_COLUMN_WIDTH * 3.0 + CONTROL_PANEL_GAP * 2.0),
                        ..default()
                    },
                ));

                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(CONTROL_PANEL_GAP),
                        align_items: AlignItems::FlexStart,
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_control_column(
                            row,
                            "Environment",
                            &CONTROL_COLUMN_1_FIELDS,
                            &runtime.field_buffers,
                            &[
                                ToggleFieldKind::FoodClusterEnabled,
                                ToggleFieldKind::DayNightCycleEnabled,
                            ],
                            &runtime.config,
                        );
                        spawn_control_column(
                            row,
                            "Evolution + Fitness",
                            &CONTROL_COLUMN_2_FIELDS,
                            &runtime.field_buffers,
                            &[
                                ToggleFieldKind::EnergyBySpeed,
                                ToggleFieldKind::EnergyByNetworkSize,
                                ToggleFieldKind::EnergyByPerceptionSize,
                            ],
                            &runtime.config,
                        );
                        spawn_control_column(
                            row,
                            "Agents + Sensors",
                            &CONTROL_COLUMN_3_FIELDS,
                            &runtime.field_buffers,
                            &[],
                            &runtime.config,
                        );
                    });

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

        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(FITNESS_PANEL_MARGIN),
                    top: Val::Px(FITNESS_PANEL_MARGIN),
                    width: Val::Px(FITNESS_PANEL_WIDTH),
                    height: Val::Px(FITNESS_PANEL_HEIGHT),
                    padding: UiRect::all(Val::Px(12.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.07, 0.1, 0.14, 0.92)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Fitness History"),
                    TextFont::from_font_size(17.0),
                    TextColor(Color::srgb(0.95, 0.95, 0.95)),
                ));
                parent.spawn((
                    Text::new("No generation metrics loaded"),
                    TextFont::from_font_size(13.0),
                    TextColor(Color::srgb(0.82, 0.86, 0.9)),
                    FitnessPlotSummaryText,
                ));
            });

        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(FITNESS_PANEL_MARGIN),
                    top: Val::Px(
                        FITNESS_PANEL_MARGIN * 2.0 + FITNESS_PANEL_HEIGHT + OBSERVABILITY_PANEL_GAP,
                    ),
                    width: Val::Px(OBSERVABILITY_PANEL_WIDTH),
                    height: Val::Px(OBSERVABILITY_PANEL_HEIGHT),
                    padding: UiRect::all(Val::Px(12.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.09, 0.11, 0.94)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Selected Agent"),
                    TextFont::from_font_size(17.0),
                    TextColor(Color::srgb(0.95, 0.95, 0.95)),
                ));
                parent.spawn((
                    Text::new(
                        "Click an agent in the world to inspect sensors and controller output.",
                    ),
                    TextFont::from_font_size(12.0),
                    TextColor(Color::srgb(0.82, 0.86, 0.9)),
                    SelectedAgentSummaryText,
                ));
            });

        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(FITNESS_PANEL_MARGIN),
                    top: Val::Px(
                        FITNESS_PANEL_MARGIN * 3.0
                            + FITNESS_PANEL_HEIGHT
                            + OBSERVABILITY_PANEL_HEIGHT
                            + OBSERVABILITY_PANEL_GAP
                            + NETWORK_PANEL_GAP,
                    ),
                    width: Val::Px(NETWORK_PANEL_WIDTH),
                    height: Val::Px(NETWORK_PANEL_HEIGHT),
                    padding: UiRect::all(Val::Px(12.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.13, 0.95)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Network Inspector"),
                    TextFont::from_font_size(17.0),
                    TextColor(Color::srgb(0.95, 0.95, 0.95)),
                ));
                parent.spawn((
                    Text::new("Select an agent to view topology and real-time activations."),
                    TextFont::from_font_size(12.0),
                    TextColor(Color::srgb(0.82, 0.86, 0.9)),
                    SelectedAgentNetworkSummaryText,
                ));
            });
    }

    fn spawn_control_column(
        parent: &mut ChildBuilder,
        title: &str,
        fields: &[NumericFieldKind],
        field_buffers: &NumericFieldBuffers,
        toggles: &[ToggleFieldKind],
        config: &crate::config::SimulationConfig,
    ) {
        parent
            .spawn(Node {
                width: Val::Px(CONTROL_COLUMN_WIDTH),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            })
            .with_children(|column: &mut ChildBuilder| {
                column.spawn((
                    Text::new(title),
                    TextFont::from_font_size(15.0),
                    TextColor(Color::srgb(0.94, 0.94, 0.94)),
                ));

                for &kind in fields {
                    spawn_numeric_field_row(column, kind, field_buffers);
                }

                if !toggles.is_empty() {
                    column.spawn((
                        Text::new("Toggles"),
                        TextFont::from_font_size(13.0),
                        TextColor(Color::srgb(0.82, 0.88, 0.94)),
                        Node {
                            margin: UiRect::top(Val::Px(4.0)),
                            ..default()
                        },
                    ));

                    for &kind in toggles {
                        spawn_toggle_field_row(column, kind, config);
                    }
                }
            });
    }

    fn spawn_numeric_field_row(
        parent: &mut ChildBuilder,
        kind: NumericFieldKind,
        field_buffers: &NumericFieldBuffers,
    ) {
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|row: &mut ChildBuilder| {
                row.spawn((
                    Text::new(field_label(kind)),
                    TextFont::from_font_size(13.0),
                    TextColor(Color::srgb(0.88, 0.88, 0.88)),
                    Node {
                        width: Val::Px(CONTROL_LABEL_WIDTH),
                        ..default()
                    },
                ));

                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(CONTROL_VALUE_WIDTH),
                        height: Val::Px(28.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    NumericField { kind },
                ))
                .with_children(|button: &mut ChildBuilder| {
                    button.spawn((
                        Text::new(field_buffers.get(kind).to_string()),
                        TextFont::from_font_size(13.0),
                        TextColor(Color::srgb(0.95, 0.95, 0.95)),
                        NumericFieldValueText { kind },
                    ));
                });
            });
    }

    fn spawn_toggle_field_row(
        parent: &mut ChildBuilder,
        kind: ToggleFieldKind,
        config: &crate::config::SimulationConfig,
    ) {
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|row: &mut ChildBuilder| {
                row.spawn((
                    Text::new(toggle_field_label(kind)),
                    TextFont::from_font_size(13.0),
                    TextColor(Color::srgb(0.88, 0.88, 0.88)),
                    Node {
                        width: Val::Px(CONTROL_LABEL_WIDTH),
                        ..default()
                    },
                ));

                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(CONTROL_VALUE_WIDTH),
                        height: Val::Px(28.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    toggle_button_color(Interaction::None, toggle_field_value(kind, config)),
                    ToggleField { kind },
                ))
                .with_children(|button: &mut ChildBuilder| {
                    button.spawn((
                        Text::new(format_toggle_value(toggle_field_value(kind, config))),
                        TextFont::from_font_size(13.0),
                        TextColor(Color::srgb(0.95, 0.95, 0.95)),
                        ToggleFieldValueText { kind },
                    ));
                });
            });
    }

    fn keyboard_controls(keys: Res<ButtonInput<KeyCode>>, mut runtime: ResMut<ViewerRuntime>) {
        if runtime.active_input.is_some() || runtime.evolution_job.is_some() {
            return;
        }

        if keys.just_pressed(KeyCode::Space) {
            runtime.paused = !runtime.paused;
        }

        if keys.just_pressed(KeyCode::Period) {
            run_simulation_steps(&mut runtime, 1);
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

    fn pause_button_interaction(
        interactions: Query<&Interaction, (Changed<Interaction>, With<Button>, With<PauseButton>)>,
        mut runtime: ResMut<ViewerRuntime>,
    ) {
        if runtime.evolution_job.is_some() {
            return;
        }

        for interaction in &interactions {
            if matches!(*interaction, Interaction::Pressed) {
                runtime.active_input = None;
                runtime.paused = true;
            }
        }
    }

    fn continue_button_interaction(
        interactions: Query<
            &Interaction,
            (Changed<Interaction>, With<Button>, With<ContinueButton>),
        >,
        mut runtime: ResMut<ViewerRuntime>,
    ) {
        if runtime.evolution_job.is_some() {
            return;
        }

        for interaction in &interactions {
            if matches!(*interaction, Interaction::Pressed) {
                runtime.active_input = None;
                runtime.paused = false;
            }
        }
    }

    fn step_button_interaction(
        interactions: Query<&Interaction, (Changed<Interaction>, With<Button>, With<StepButton>)>,
        mut runtime: ResMut<ViewerRuntime>,
    ) {
        if runtime.evolution_job.is_some() {
            return;
        }

        for interaction in &interactions {
            if matches!(*interaction, Interaction::Pressed) {
                runtime.active_input = None;
                runtime.paused = true;
                run_simulation_steps(&mut runtime, 1);
            }
        }
    }

    fn toggle_field_interaction(
        interactions: Query<
            (&Interaction, &ToggleField),
            (Changed<Interaction>, With<Button>, With<ToggleField>),
        >,
        mut runtime: ResMut<ViewerRuntime>,
    ) {
        if runtime.evolution_job.is_some() {
            return;
        }

        for (interaction, toggle) in &interactions {
            if !matches!(*interaction, Interaction::Pressed) {
                continue;
            }

            runtime.active_input = None;
            match toggle.kind {
                ToggleFieldKind::FoodClusterEnabled => {
                    runtime.config.food_cluster_enabled = !runtime.config.food_cluster_enabled;
                }
                ToggleFieldKind::DayNightCycleEnabled => {
                    runtime.config.day_night_cycle_enabled =
                        !runtime.config.day_night_cycle_enabled;
                }
                ToggleFieldKind::EnergyBySpeed => {
                    runtime.config.rule_toggles.energy_by_speed =
                        !runtime.config.rule_toggles.energy_by_speed;
                }
                ToggleFieldKind::EnergyByNetworkSize => {
                    runtime.config.rule_toggles.energy_by_network_size =
                        !runtime.config.rule_toggles.energy_by_network_size;
                }
                ToggleFieldKind::EnergyByPerceptionSize => {
                    runtime.config.rule_toggles.energy_by_perception_size =
                        !runtime.config.rule_toggles.energy_by_perception_size;
                }
            }
        }
    }

    fn evaluation_mode_button_interaction(
        interactions: Query<
            &Interaction,
            (
                Changed<Interaction>,
                With<Button>,
                With<EvaluationModeButton>,
            ),
        >,
        mut runtime: ResMut<ViewerRuntime>,
    ) {
        if runtime.evolution_job.is_some() {
            return;
        }

        for interaction in &interactions {
            if matches!(*interaction, Interaction::Pressed) {
                runtime.active_input = None;
                runtime.config.evaluation_mode =
                    toggle_evaluation_mode(runtime.config.evaluation_mode);
            }
        }
    }

    fn evaluation_mode_after_switch_button_interaction(
        interactions: Query<
            &Interaction,
            (
                Changed<Interaction>,
                With<Button>,
                With<EvaluationModeAfterSwitchButton>,
            ),
        >,
        mut runtime: ResMut<ViewerRuntime>,
    ) {
        if runtime.evolution_job.is_some() {
            return;
        }

        for interaction in &interactions {
            if matches!(*interaction, Interaction::Pressed) {
                runtime.active_input = None;
                runtime.config.evaluation_mode_after_switch =
                    toggle_evaluation_mode(runtime.config.evaluation_mode_after_switch);
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

    fn agent_selection_input(
        mouse_buttons: Res<ButtonInput<MouseButton>>,
        window_query: Query<&Window, With<PrimaryWindow>>,
        mut runtime: ResMut<ViewerRuntime>,
    ) {
        if !mouse_buttons.just_pressed(MouseButton::Left)
            || runtime.evolution_job.is_some()
            || runtime.active_input.is_some()
        {
            return;
        }

        let Ok(window) = window_query.get_single() else {
            return;
        };
        let Some(cursor) = window.cursor_position() else {
            return;
        };
        if cursor_over_ui_panel(cursor, window) {
            return;
        }

        let selected = runtime
            .world
            .agents
            .iter()
            .enumerate()
            .filter(|(_, agent)| agent.alive)
            .filter_map(|(idx, agent)| {
                let agent_cursor = world_to_cursor(
                    agent.position,
                    runtime.world.width,
                    runtime.world.height,
                    window,
                );
                let distance = cursor.distance(agent_cursor);
                (distance <= SELECTION_PICK_RADIUS).then_some((idx, distance))
            })
            .min_by(|(_, distance_a), (_, distance_b)| distance_a.total_cmp(distance_b))
            .map(|(idx, _)| idx);

        runtime.selected_agent_index = selected;
        refresh_selected_agent_preview(&mut runtime);
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

        merge_generation_metrics(&mut runtime.metrics_history, &run.generations);
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
            runtime.controllers = build_runtime_controllers(&runtime.config, &runtime.population);
            runtime.rebuild_visuals = true;
            runtime.paused = true;
            refresh_selected_agent_preview(&mut runtime);
        } else {
            runtime.evolution_job = Some(job);
        }
    }

    fn sync_simulation_control_state(
        runtime: Res<ViewerRuntime>,
        mut button_query: Query<
            (
                &Interaction,
                &mut BackgroundColor,
                Option<&PauseButton>,
                Option<&StepButton>,
                Option<&ContinueButton>,
            ),
            With<Button>,
        >,
    ) {
        for (interaction, mut color, pause, step, cont) in &mut button_query {
            let base = if pause.is_some() {
                Color::srgb(0.70, 0.30, 0.25)
            } else if step.is_some() {
                Color::srgb(0.22, 0.65, 0.35)
            } else if cont.is_some() {
                Color::srgb(0.20, 0.45, 0.78)
            } else {
                continue;
            };

            *color = match *interaction {
                Interaction::Pressed => BackgroundColor(Color::srgb(0.84, 0.84, 0.84)),
                Interaction::Hovered => BackgroundColor(lighten_color(base, 0.12)),
                Interaction::None => BackgroundColor(if pause.is_some() && runtime.paused {
                    lighten_color(base, 0.08)
                } else if cont.is_some() && !runtime.paused {
                    lighten_color(base, 0.08)
                } else {
                    base
                }),
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
                Option<&ToggleField>,
                Option<&EvolveRunButton>,
                Option<&EvaluationModeButton>,
                Option<&EvaluationModeAfterSwitchButton>,
            ),
            With<Button>,
        >,
        mut text_query: Query<(
            &mut Text,
            Option<&NumericFieldValueText>,
            Option<&ToggleFieldValueText>,
            Option<&EvolveStatusText>,
            Option<&EvaluationModeButtonLabel>,
            Option<&EvaluationModeAfterSwitchButtonLabel>,
        )>,
    ) {
        let is_evolving = runtime.evolution_job.is_some();
        for (
            interaction,
            mut color,
            maybe_field,
            maybe_toggle,
            maybe_evolve,
            maybe_eval_mode,
            maybe_eval_mode_after,
        ) in &mut button_query
        {
            if let Some(field) = maybe_field {
                *color = numeric_field_color(
                    *interaction,
                    runtime
                        .active_input
                        .is_some_and(|active| active == field.kind),
                );
            } else if let Some(toggle) = maybe_toggle {
                *color = toggle_button_color(
                    *interaction,
                    toggle_field_value(toggle.kind, &runtime.config),
                );
            } else if maybe_evolve.is_some() {
                *color = evolve_run_button_color(*interaction, is_evolving);
            } else if maybe_eval_mode.is_some() {
                *color = evaluation_mode_button_color(*interaction, runtime.config.evaluation_mode);
            } else if maybe_eval_mode_after.is_some() {
                *color = evaluation_mode_button_color(
                    *interaction,
                    runtime.config.evaluation_mode_after_switch,
                );
            }
        }

        let status = if let Some(job) = &runtime.evolution_job {
            format!(
                "Status: evolving {}/{} generations",
                visible_progress_completed(job),
                job.total_generations
            )
        } else {
            format!(
                "Status: ready (next evolve: {} generations)",
                runtime.evolve_generations
            )
        };

        for (
            mut text,
            value_marker,
            toggle_marker,
            status_marker,
            eval_mode_marker,
            eval_mode_after_marker,
        ) in &mut text_query
        {
            if let Some(marker) = value_marker {
                **text = runtime.field_buffers.get(marker.kind).to_string();
            } else if let Some(marker) = toggle_marker {
                **text = format_toggle_value(toggle_field_value(marker.kind, &runtime.config))
                    .to_string();
            } else if status_marker.is_some() {
                **text = status.clone();
            } else if eval_mode_marker.is_some() {
                **text = format_evaluation_mode(runtime.config.evaluation_mode).to_string();
            } else if eval_mode_after_marker.is_some() {
                **text =
                    format_evaluation_mode(runtime.config.evaluation_mode_after_switch).to_string();
            }
        }
    }

    fn sync_fitness_plot_summary(
        runtime: Res<ViewerRuntime>,
        mut query: Query<&mut Text, With<FitnessPlotSummaryText>>,
    ) {
        let summary = if let Some(metric) = runtime.metrics_history.last() {
            format!(
                "gens: {}\nMLP best/mean: {} / {}\nLTC best/mean: {} / {}\nlegend: blue=MLP, orange=LTC, solid=best, dashed=mean",
                runtime.metrics_history.len(),
                format_optional(metric.mlp_best_fitness),
                format_optional(metric.mlp_mean_fitness),
                format_optional(metric.ltc_best_fitness),
                format_optional(metric.ltc_mean_fitness),
            )
        } else {
            "No embedded history in this snapshot yet.\nLegend: blue=MLP, orange=LTC, solid=best, dashed=mean".to_string()
        };

        for mut text in &mut query {
            **text = summary.clone();
        }
    }

    fn sync_selected_agent_summary(
        runtime: Res<ViewerRuntime>,
        mut query: Query<&mut Text, With<SelectedAgentSummaryText>>,
    ) {
        let summary = selected_agent_summary_text(&runtime);

        for mut text in &mut query {
            **text = summary.clone();
        }
    }

    fn sync_selected_agent_network_summary(
        runtime: Res<ViewerRuntime>,
        mut query: Query<&mut Text, With<SelectedAgentNetworkSummaryText>>,
    ) {
        let summary = selected_agent_network_summary_text(&runtime);

        for mut text in &mut query {
            **text = summary.clone();
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

    fn toggle_button_color(interaction: Interaction, enabled: bool) -> BackgroundColor {
        if matches!(interaction, Interaction::Hovered) {
            return BackgroundColor(Color::srgb(0.42, 0.42, 0.48));
        }

        if enabled {
            BackgroundColor(Color::srgb(0.23, 0.56, 0.34))
        } else {
            BackgroundColor(Color::srgb(0.46, 0.22, 0.22))
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

    fn evaluation_mode_button_color(
        interaction: Interaction,
        mode: EvaluationMode,
    ) -> BackgroundColor {
        if matches!(interaction, Interaction::Hovered) {
            return BackgroundColor(Color::srgb(0.48, 0.48, 0.58));
        }

        match mode {
            EvaluationMode::Solo => BackgroundColor(Color::srgb(0.24, 0.42, 0.62)),
            EvaluationMode::Shared => BackgroundColor(Color::srgb(0.62, 0.38, 0.24)),
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
        run_simulation_steps(&mut runtime, steps);
    }

    fn run_simulation_steps(runtime: &mut ViewerRuntime, steps: u32) {
        let config = runtime.config.clone();
        for _ in 0..steps {
            let world_snapshot = runtime.world.clone();
            let mut actions = Vec::with_capacity(world_snapshot.agents.len());
            let mut selected_sensor = None;
            let mut selected_diagnostics: Option<ControllerDiagnostics> = None;

            for (idx, agent) in world_snapshot.agents.iter().enumerate() {
                if !agent.alive {
                    actions.push(crate::sim::ActionVector::default());
                    continue;
                }

                let sensor = sensor_observation(agent, &world_snapshot, &config);
                if runtime.selected_agent_index == Some(idx) {
                    let diagnostics = runtime
                        .controllers
                        .get_mut(idx)
                        .map(|controller| {
                            controller.step_with_diagnostics(&sensor.input, config.timestep_seconds)
                        })
                        .unwrap_or_default();
                    actions.push(diagnostics.output.action);
                    selected_sensor = Some(sensor);
                    selected_diagnostics = Some(diagnostics);
                } else {
                    let output = runtime
                        .controllers
                        .get_mut(idx)
                        .map(|controller| {
                            controller
                                .step_with_observability(&sensor.input, config.timestep_seconds)
                        })
                        .unwrap_or_default();
                    actions.push(output.action);
                }
            }

            let (_outcome, step_telemetry) =
                step_world_with_actions_and_telemetry(&mut runtime.world, &config, &actions);

            if let Some(selected_idx) = runtime.selected_agent_index {
                if let (Some(agent), Some(sensor), Some(diagnostics), Some(step_data)) = (
                    runtime.world.agents.get(selected_idx),
                    selected_sensor,
                    selected_diagnostics,
                    step_telemetry.get(selected_idx).copied(),
                ) {
                    runtime.selected_agent_snapshot = Some(SelectedAgentSnapshot {
                        agent_id: agent.id,
                        controller_kind: runtime
                            .controller_kinds
                            .get(selected_idx)
                            .copied()
                            .unwrap_or(ControllerKind::Mlp),
                        role: agent.role,
                        alive: agent.alive,
                        world_step: runtime.world.step_count,
                        perception_radius: agent.perception_radius,
                        perception_channels: agent.perception_channels,
                        eye_cell_angle_radians: agent.eye_cell_angle_radians,
                        sensor,
                        controller_output: diagnostics.output,
                        network: diagnostics.network,
                        energy: step_data.energy,
                        consumed_food: step_data.consumed_food,
                        has_live_step: true,
                    });
                }
            }
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
                    if runtime.selected_agent_index == Some(visual.index) {
                        2.0
                    } else {
                        1.0
                    },
                );
                transform.rotation = visual_heading_rotation(agent.heading_radians);
                transform.scale = if runtime.selected_agent_index == Some(visual.index) {
                    bevy::math::Vec3::splat(1.8)
                } else {
                    bevy::math::Vec3::ONE
                };
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
        mut existing_food: Query<(Entity, &FoodVisual, &mut Transform, &mut Sprite)>,
    ) {
        let world_food_ids = runtime
            .world
            .food
            .iter()
            .map(|food| food.id)
            .collect::<Vec<_>>();

        for (entity, food_visual, _, _) in &existing_food {
            if !world_food_ids.contains(&food_visual.id) {
                commands.entity(entity).despawn_recursive();
            }
        }

        for food in &runtime.world.food {
            let mut found = false;
            for (_, marker, mut transform, mut sprite) in &mut existing_food {
                if marker.id == food.id {
                    transform.translation = world_to_screen(
                        food.position,
                        runtime.world.width,
                        runtime.world.height,
                        0.0,
                    );
                    sprite.color = food_visual_color(food);
                    found = true;
                    break;
                }
            }

            if !found {
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
                visible_progress_completed(job),
                job.total_generations
            )
        } else {
            String::new()
        };
        let day_night = current_day_night_state(runtime.world.step_count, &runtime.config);
        let light_label = if runtime.config.day_night_cycle_enabled {
            if day_night.is_night { "night" } else { "day" }
        } else {
            "fixed"
        };

        window.title = format!(
            "neural-survival | gen={} alive={} food={} bestE={:.1} meanE={:.1} MLP={} LTC={} speed={}x eval={} light={} vis={:.2} world={:.0}x{:.0} rules[s:{} n:{} p:{}]{}",
            runtime.generation,
            alive,
            food,
            best_energy,
            mean_energy,
            mlp,
            ltc,
            runtime.speed_multiplier,
            format_evaluation_schedule(&runtime.config),
            light_label,
            day_night.visibility_multiplier,
            runtime.config.world_width,
            runtime.config.world_height,
            runtime.config.rule_toggles.energy_by_speed,
            runtime.config.rule_toggles.energy_by_network_size,
            runtime.config.rule_toggles.energy_by_perception_size,
            evolution_progress
        );
    }

    fn draw_food_clusters(runtime: Res<ViewerRuntime>, mut gizmos: Gizmos) {
        if !runtime.config.food_cluster_enabled {
            return;
        }

        for cluster in &runtime.world.food_clusters {
            let radius = runtime.config.food_cluster_radius
                * cluster_density_radius_scale(cluster, runtime.world.step_count, &runtime.config);
            let productivity =
                cluster_productivity(cluster, runtime.world.step_count, &runtime.config)
                    .clamp(0.0, 1.0);
            let color = cluster_visual_color(cluster.id, productivity);
            draw_world_circle(
                &mut gizmos,
                cluster.position,
                radius.max(4.0),
                runtime.world.width,
                runtime.world.height,
                color,
            );
        }
    }

    fn draw_fitness_plot(
        runtime: Res<ViewerRuntime>,
        window_query: Query<&Window, With<PrimaryWindow>>,
        mut gizmos: Gizmos,
    ) {
        let Ok(window) = window_query.get_single() else {
            return;
        };

        let panel_left = window.width() - FITNESS_PANEL_MARGIN - FITNESS_PANEL_WIDTH;
        let panel_top = FITNESS_PANEL_MARGIN;
        let plot_left = panel_left + FITNESS_PLOT_INSET_LEFT;
        let plot_right = panel_left + FITNESS_PANEL_WIDTH - FITNESS_PLOT_INSET_RIGHT;
        let plot_top = panel_top + FITNESS_PLOT_INSET_TOP;
        let plot_bottom = panel_top + FITNESS_PANEL_HEIGHT - FITNESS_PLOT_INSET_BOTTOM;

        let axis_color = Color::srgba(0.72, 0.78, 0.86, 0.45);
        let grid_color = Color::srgba(0.38, 0.46, 0.58, 0.28);

        draw_screen_line(
            &mut gizmos,
            window,
            plot_left,
            plot_top,
            plot_right,
            plot_top,
            axis_color,
        );
        draw_screen_line(
            &mut gizmos,
            window,
            plot_left,
            plot_bottom,
            plot_right,
            plot_bottom,
            axis_color,
        );
        draw_screen_line(
            &mut gizmos,
            window,
            plot_left,
            plot_top,
            plot_left,
            plot_bottom,
            axis_color,
        );
        draw_screen_line(
            &mut gizmos,
            window,
            plot_left,
            (plot_top + plot_bottom) * 0.5,
            plot_right,
            (plot_top + plot_bottom) * 0.5,
            grid_color,
        );

        let Some(sampled) = sampled_metrics(&runtime.metrics_history, FITNESS_PLOT_MAX_POINTS)
        else {
            return;
        };

        let min_generation = sampled.first().map(|m| m.generation_index).unwrap_or(0);
        let max_generation = sampled
            .last()
            .map(|m| m.generation_index)
            .unwrap_or(min_generation);
        let max_fitness = sampled
            .iter()
            .flat_map(|metric| {
                [
                    metric.mlp_best_fitness,
                    metric.mlp_mean_fitness,
                    metric.ltc_best_fitness,
                    metric.ltc_mean_fitness,
                ]
            })
            .flatten()
            .fold(0.0_f32, f32::max)
            .max(1.0);
        let generation_span = (max_generation.saturating_sub(min_generation)).max(1) as f32;

        let x_for_generation = |generation: u32| -> f32 {
            let progress = (generation.saturating_sub(min_generation)) as f32 / generation_span;
            plot_left + progress * (plot_right - plot_left)
        };
        let y_for_fitness = |fitness: f32| -> f32 {
            let progress = (fitness / max_fitness).clamp(0.0, 1.0);
            plot_bottom - progress * (plot_bottom - plot_top)
        };

        draw_species_series(
            &mut gizmos,
            window,
            &sampled,
            |metric| metric.mlp_best_fitness,
            &x_for_generation,
            &y_for_fitness,
            Color::srgb(0.19, 0.45, 0.92),
            false,
        );
        draw_species_series(
            &mut gizmos,
            window,
            &sampled,
            |metric| metric.mlp_mean_fitness,
            &x_for_generation,
            &y_for_fitness,
            Color::srgb(0.50, 0.72, 1.0),
            true,
        );
        draw_species_series(
            &mut gizmos,
            window,
            &sampled,
            |metric| metric.ltc_best_fitness,
            &x_for_generation,
            &y_for_fitness,
            Color::srgb(0.93, 0.53, 0.10),
            false,
        );
        draw_species_series(
            &mut gizmos,
            window,
            &sampled,
            |metric| metric.ltc_mean_fitness,
            &x_for_generation,
            &y_for_fitness,
            Color::srgb(1.0, 0.75, 0.45),
            true,
        );
    }

    fn draw_selected_agent_eye_bins(
        runtime: Res<ViewerRuntime>,
        window_query: Query<&Window, With<PrimaryWindow>>,
        mut gizmos: Gizmos,
    ) {
        let Some(snapshot) = &runtime.selected_agent_snapshot else {
            return;
        };
        let Ok(window) = window_query.get_single() else {
            return;
        };

        let panel_left = window.width() - FITNESS_PANEL_MARGIN - OBSERVABILITY_PANEL_WIDTH;
        let panel_top = FITNESS_PANEL_MARGIN * 2.0 + FITNESS_PANEL_HEIGHT + OBSERVABILITY_PANEL_GAP;
        let plot_left = panel_left + OBSERVABILITY_EYE_PLOT_INSET_LEFT;
        let plot_right =
            panel_left + OBSERVABILITY_PANEL_WIDTH - OBSERVABILITY_EYE_PLOT_INSET_RIGHT;
        let plot_top = panel_top + OBSERVABILITY_EYE_PLOT_INSET_TOP;
        let plot_bottom =
            panel_top + OBSERVABILITY_PANEL_HEIGHT - OBSERVABILITY_EYE_PLOT_INSET_BOTTOM;

        draw_screen_line(
            &mut gizmos,
            window,
            plot_left,
            plot_top,
            plot_left,
            plot_bottom,
            Color::srgba(0.72, 0.78, 0.86, 0.45),
        );
        draw_screen_line(
            &mut gizmos,
            window,
            plot_left,
            plot_bottom,
            plot_right,
            plot_bottom,
            Color::srgba(0.72, 0.78, 0.86, 0.45),
        );

        let active_bins = snapshot.perception_channels.max(1) as usize;
        let bins_len = active_bins
            .min(snapshot.sensor.food_eye_bins.len())
            .min(snapshot.sensor.prey_eye_bins.len())
            .min(snapshot.sensor.predator_eye_bins.len());
        let food_bins = &snapshot.sensor.food_eye_bins[..bins_len];
        let prey_bins = &snapshot.sensor.prey_eye_bins[..bins_len];
        let predator_bins = &snapshot.sensor.predator_eye_bins[..bins_len];
        if food_bins.is_empty() {
            return;
        }

        let max_activation = food_bins
            .iter()
            .chain(prey_bins.iter())
            .chain(predator_bins.iter())
            .copied()
            .fold(0.0_f32, f32::max)
            .max(1.0);
        let bar_width = ((plot_right - plot_left) / food_bins.len() as f32).max(4.0);

        for idx in 0..food_bins.len() {
            let x0 = plot_left + idx as f32 * bar_width + 1.0;
            let x1 = (x0 + bar_width - 2.0).min(plot_right);
            let food_top = plot_bottom
                - ((food_bins[idx] / max_activation).clamp(0.0, 1.0) * (plot_bottom - plot_top));
            let prey_top = plot_bottom
                - ((prey_bins[idx] / max_activation).clamp(0.0, 1.0) * (plot_bottom - plot_top));
            let predator_top = plot_bottom
                - ((predator_bins[idx] / max_activation).clamp(0.0, 1.0)
                    * (plot_bottom - plot_top));

            let mut x = x0;
            while x <= x1 {
                draw_screen_line(
                    &mut gizmos,
                    window,
                    x,
                    plot_bottom,
                    x,
                    food_top,
                    Color::srgb(0.22, 0.86, 0.55),
                );
                draw_screen_line(
                    &mut gizmos,
                    window,
                    x + 1.0,
                    plot_bottom,
                    x + 1.0,
                    prey_top,
                    Color::srgb(0.34, 0.70, 1.0),
                );
                draw_screen_line(
                    &mut gizmos,
                    window,
                    x + 2.0,
                    plot_bottom,
                    x + 2.0,
                    predator_top,
                    Color::srgb(0.98, 0.62, 0.20),
                );
                x += 4.0;
            }
        }
    }

    fn draw_selected_agent_network(
        runtime: Res<ViewerRuntime>,
        window_query: Query<&Window, With<PrimaryWindow>>,
        mut gizmos: Gizmos,
    ) {
        let Some(snapshot) = &runtime.selected_agent_snapshot else {
            return;
        };
        let Ok(window) = window_query.get_single() else {
            return;
        };

        let panel_left = window.width() - FITNESS_PANEL_MARGIN - NETWORK_PANEL_WIDTH;
        let panel_top = FITNESS_PANEL_MARGIN * 3.0
            + FITNESS_PANEL_HEIGHT
            + OBSERVABILITY_PANEL_HEIGHT
            + OBSERVABILITY_PANEL_GAP
            + NETWORK_PANEL_GAP;
        let plot_left = panel_left + NETWORK_PLOT_INSET_LEFT;
        let plot_right = panel_left + NETWORK_PANEL_WIDTH - NETWORK_PLOT_INSET_RIGHT;
        let plot_top = panel_top + NETWORK_PLOT_INSET_TOP;
        let plot_bottom = panel_top + NETWORK_PANEL_HEIGHT - NETWORK_PLOT_INSET_BOTTOM;

        draw_screen_line(
            &mut gizmos,
            window,
            plot_left,
            plot_top,
            plot_right,
            plot_top,
            Color::srgba(0.72, 0.78, 0.86, 0.45),
        );
        draw_screen_line(
            &mut gizmos,
            window,
            plot_left,
            plot_bottom,
            plot_right,
            plot_bottom,
            Color::srgba(0.72, 0.78, 0.86, 0.45),
        );

        match &snapshot.network {
            ControllerNetworkSnapshot::Mlp(mlp) => draw_layered_network(
                &mut gizmos,
                window,
                plot_left,
                plot_right,
                plot_top,
                plot_bottom,
                &mlp.input_values,
                &mlp.hidden_values,
                &mlp.output_values,
                &mlp.weights_input_hidden,
                &mlp.weights_hidden_output,
                None,
            ),
            ControllerNetworkSnapshot::Ltc(ltc) => draw_layered_network(
                &mut gizmos,
                window,
                plot_left,
                plot_right,
                plot_top,
                plot_bottom,
                &ltc.input_values,
                &ltc.state_values
                    .iter()
                    .map(|value| value.tanh())
                    .collect::<Vec<_>>(),
                &ltc.output_values,
                &ltc.sensory_weights,
                &ltc.readout_weights,
                Some(&ltc.recurrent_weights),
            ),
            ControllerNetworkSnapshot::Unavailable => {}
        }
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

            let entity = commands
                .spawn((
                    Mesh2d(mesh),
                    MeshMaterial2d(material),
                    Transform {
                        translation: world_to_screen(
                            agent.position,
                            world.width,
                            world.height,
                            1.0,
                        ),
                        rotation: visual_heading_rotation(agent.heading_radians),
                        ..default()
                    },
                    AgentVisual { index: idx },
                ))
                .id();

            if kind == ControllerKind::Mlp {
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        Sprite {
                            color: Color::srgb(0.95, 0.97, 1.0),
                            custom_size: Some(bevy::math::Vec2::new(2.0, 6.0)),
                            ..default()
                        },
                        Transform::from_translation(bevy::math::Vec3::new(0.0, 6.5, 0.1)),
                        AgentHeadingMarker,
                    ));
                });
            }
        }
    }

    fn spawn_one_food(commands: &mut Commands, food: &FoodState, width: f32, height: f32) {
        commands.spawn((
            Sprite {
                color: food_visual_color(food),
                custom_size: Some(bevy::math::Vec2::new(4.0, 4.0)),
                ..default()
            },
            Transform::from_translation(world_to_screen(food.position, width, height, 0.0)),
            FoodVisual { id: food.id },
        ));
    }

    fn build_runtime_controllers(
        config: &crate::config::SimulationConfig,
        population: &[AgentGenome],
    ) -> Vec<Box<dyn NeuralController>> {
        let factory = default_mixed_factory(config);
        population
            .iter()
            .map(|genome| {
                let mut controller = factory.build(&genome.brain);
                controller.reset();
                controller
            })
            .collect()
    }

    fn refresh_selected_agent_preview(runtime: &mut ViewerRuntime) {
        let Some(selected_idx) = runtime.selected_agent_index else {
            runtime.selected_agent_snapshot = None;
            return;
        };
        let Some(agent) = runtime.world.agents.get(selected_idx) else {
            runtime.selected_agent_index = None;
            runtime.selected_agent_snapshot = None;
            return;
        };

        let sensor = sensor_observation(agent, &runtime.world, &runtime.config);
        let network = runtime
            .controllers
            .get(selected_idx)
            .map(|controller| controller.inspect_network(&sensor.input))
            .unwrap_or_default();
        runtime.selected_agent_snapshot = Some(SelectedAgentSnapshot {
            agent_id: agent.id,
            controller_kind: runtime
                .controller_kinds
                .get(selected_idx)
                .copied()
                .unwrap_or(ControllerKind::Mlp),
            role: agent.role,
            alive: agent.alive,
            world_step: runtime.world.step_count,
            perception_radius: agent.perception_radius,
            perception_channels: agent.perception_channels,
            eye_cell_angle_radians: agent.eye_cell_angle_radians,
            sensor,
            controller_output: ControllerStepOutput::default(),
            network,
            energy: EnergyBreakdown::default(),
            consumed_food: false,
            has_live_step: false,
        });
    }

    fn world_to_screen(position: Vec2, width: f32, height: f32, z: f32) -> bevy::math::Vec3 {
        bevy::math::Vec3::new(position.x - width / 2.0, position.y - height / 2.0, z)
    }

    fn world_to_cursor(
        position: Vec2,
        width: f32,
        height: f32,
        window: &Window,
    ) -> bevy::math::Vec2 {
        let screen = world_to_screen(position, width, height, 0.0);
        bevy::math::Vec2::new(
            screen.x + window.width() * 0.5,
            window.height() * 0.5 - screen.y,
        )
    }

    fn cursor_over_ui_panel(cursor: bevy::math::Vec2, window: &Window) -> bool {
        let left_panel_right =
            CONTROL_PANEL_LEFT + CONTROL_COLUMN_WIDTH * 3.0 + CONTROL_PANEL_GAP * 2.0 + 40.0;
        if cursor.x <= left_panel_right && cursor.y <= window.height() {
            return true;
        }

        let right_panel_left = window.width()
            - FITNESS_PANEL_MARGIN
            - FITNESS_PANEL_WIDTH
                .max(OBSERVABILITY_PANEL_WIDTH)
                .max(NETWORK_PANEL_WIDTH);
        cursor.x >= right_panel_left
            && cursor.y
                <= FITNESS_PANEL_MARGIN * 4.0
                    + FITNESS_PANEL_HEIGHT
                    + OBSERVABILITY_PANEL_GAP
                    + OBSERVABILITY_PANEL_HEIGHT
                    + NETWORK_PANEL_GAP
                    + NETWORK_PANEL_HEIGHT
    }

    fn selected_agent_summary_text(runtime: &ViewerRuntime) -> String {
        let Some(snapshot) = &runtime.selected_agent_snapshot else {
            return "Click an agent in the world to inspect sensors and controller output."
                .to_string();
        };
        let day_night = current_day_night_state(runtime.world.step_count, &runtime.config);

        let [energy_norm, speed_norm] = snapshot.sensor.core_values;
        let output_text = if snapshot.has_live_step {
            format!(
                "last action: steer={:.3} throttle_raw={:.3} throttle={:.3}",
                snapshot.controller_output.steering_raw,
                snapshot.controller_output.throttle_raw,
                snapshot.controller_output.throttle_applied
            )
        } else {
            "last action: waiting for first live simulation step".to_string()
        };
        let energy_text = if snapshot.has_live_step {
            format!(
                "energy: base={:.3} speed={:.3} network={:.3} perceive={:.3} food={:+.3} capture={:+.3} net={:+.3}",
                snapshot.energy.base_loss,
                snapshot.energy.speed_loss,
                snapshot.energy.network_loss,
                snapshot.energy.perception_loss,
                snapshot.energy.food_gain,
                snapshot.energy.capture_gain,
                snapshot.energy.net_delta
            )
        } else {
            "energy: waiting for first live simulation step".to_string()
        };
        let food_eye_max = snapshot
            .sensor
            .food_eye_bins
            .iter()
            .take(snapshot.perception_channels as usize)
            .copied()
            .fold(0.0_f32, f32::max);
        let prey_eye_max = snapshot
            .sensor
            .prey_eye_bins
            .iter()
            .take(snapshot.perception_channels as usize)
            .copied()
            .fold(0.0_f32, f32::max);
        let predator_eye_max = snapshot
            .sensor
            .predator_eye_bins
            .iter()
            .take(snapshot.perception_channels as usize)
            .copied()
            .fold(0.0_f32, f32::max);

        format!(
            "agent #{} ({:?}, {:?}) {}\nworld step: {}\nlight: {} phase={:.2} visibility={:.2}\nperception: radius={:.1} bins={} eye_angle={:.3} total_fov={:.3}\ncore sensors: energy={:.3} speed={:.3}\n{}\n{}\nfood this step: {}\npeak eyes: food={:.3} prey={:.3} predator={:.3}\nchart below: green=food, blue=prey, orange=predator, controller input channels are [food bins | prey bins | predator bins]",
            snapshot.agent_id,
            snapshot.controller_kind,
            snapshot.role,
            if snapshot.alive { "alive" } else { "dead" },
            snapshot.world_step,
            if runtime.config.day_night_cycle_enabled {
                if day_night.is_night { "night" } else { "day" }
            } else {
                "fixed"
            },
            day_night.phase,
            day_night.visibility_multiplier,
            snapshot.perception_radius,
            snapshot.perception_channels,
            snapshot.eye_cell_angle_radians,
            (snapshot.eye_cell_angle_radians * snapshot.perception_channels.max(1) as f32)
                .min(std::f32::consts::TAU),
            energy_norm,
            speed_norm,
            output_text,
            energy_text,
            snapshot.consumed_food,
            food_eye_max,
            prey_eye_max,
            predator_eye_max
        )
    }

    fn selected_agent_network_summary_text(runtime: &ViewerRuntime) -> String {
        let Some(snapshot) = &runtime.selected_agent_snapshot else {
            return "Select an agent to view topology and real-time activations.".to_string();
        };

        match &snapshot.network {
            ControllerNetworkSnapshot::Mlp(mlp) => format!(
                "MLP topology: {} -> {} -> {}\ncolumns: left=input [{}], middle=hidden tanh, right=output [steer throttle]\ncurrent output: steer={:.3} throttle={:.3}\ncolors: teal=positive activation/weight, orange-red=negative, brighter=stronger magnitude",
                mlp.input_values.len(),
                mlp.hidden_values.len(),
                mlp.output_values.len(),
                controller_input_legend(),
                mlp.output_values.first().copied().unwrap_or_default(),
                mlp.output_values.get(1).copied().unwrap_or_default(),
            ),
            ControllerNetworkSnapshot::Ltc(ltc) => format!(
                "LTC topology: {} input, {} recurrent state, {} output\ncolumns: left=input [{}], middle=state (recurrent), right=output [steer throttle]\ncurrent output: steer={:.3} throttle={:.3}\ncolors: teal=positive activation/weight, orange-red=negative, brighter=stronger magnitude",
                ltc.input_values.len(),
                ltc.state_values.len(),
                ltc.output_values.len(),
                controller_input_legend(),
                ltc.output_values.first().copied().unwrap_or_default(),
                ltc.output_values.get(1).copied().unwrap_or_default(),
            ),
            ControllerNetworkSnapshot::Unavailable => {
                "Network topology is not available for this controller.".to_string()
            }
        }
    }

    fn controller_input_legend() -> &'static str {
        "energy speed food0..9 prey0..9 predator0..9"
    }

    fn field_label(kind: NumericFieldKind) -> &'static str {
        match kind {
            NumericFieldKind::EvolveGenerations => "Generations",
            NumericFieldKind::EvaluationSwitchAfterGeneration => "Eval Switch After",
            NumericFieldKind::WorldWidth => "World Width",
            NumericFieldKind::WorldHeight => "World Height",
            NumericFieldKind::TimestepSeconds => "Timestep",
            NumericFieldKind::PopulationSize => "Population",
            NumericFieldKind::FoodCount => "Food Count",
            NumericFieldKind::FoodMotionSpeed => "Food Motion",
            NumericFieldKind::FoodRespawnProbability => "Food Respawn",
            NumericFieldKind::FoodClusterCount => "Cluster Count",
            NumericFieldKind::FoodClusterRadius => "Cluster Radius",
            NumericFieldKind::FoodClusterMotionSpeed => "Cluster Motion",
            NumericFieldKind::FoodClusterProductivityBias => "Cluster Bias",
            NumericFieldKind::ElitismFraction => "Elitism",
            NumericFieldKind::TournamentSize => "Tournament",
            NumericFieldKind::InitialAgentEnergy => "Start Energy",
            NumericFieldKind::BaseEnergyCostPerSecond => "Base Cost/s",
            NumericFieldKind::SpeedEnergyCostFactor => "Speed Cost",
            NumericFieldKind::NetworkEnergyCostFactor => "Network Cost",
            NumericFieldKind::PerceptionEnergyCostFactor => "Vision Cost",
            NumericFieldKind::FoodEnergyGain => "Food Gain",
            NumericFieldKind::FoodCollisionRadius => "Food Radius",
            NumericFieldKind::EyeFovAngleRadians => "Eye FOV Rad",
            NumericFieldKind::DefaultAgentMaxSpeed => "Default Speed",
            NumericFieldKind::MaxAcceleration => "Acceleration",
            NumericFieldKind::LinearDrag => "Linear Drag",
            NumericFieldKind::MaxTurnRateRadians => "Turn Rate Rad",
            NumericFieldKind::DefaultPerceptionRadius => "Default Range",
            NumericFieldKind::DefaultPerceptionChannels => "Default Eyes",
            NumericFieldKind::DefaultNetworkSize => "Default Net Size",
            NumericFieldKind::MlpHiddenSize => "MLP Hidden",
            NumericFieldKind::LtcStateSize => "LTC State",
            NumericFieldKind::DefaultMetabolicEfficiency => "Default Metab",
            NumericFieldKind::ControllerLtcProbability => "LTC Probability",
            NumericFieldKind::PredatorFraction => "Predator Frac",
            NumericFieldKind::PredatorCaptureRadius => "Capture Radius",
            NumericFieldKind::PredatorCaptureEnergyGain => "Capture Gain",
            NumericFieldKind::SensorNoiseStddev => "Sensor Noise",
            NumericFieldKind::SensorDropoutProbability => "Sensor Dropout",
            NumericFieldKind::DayNightCycleSteps => "Day/Night Steps",
            NumericFieldKind::NightVisibilityMultiplier => "Night Vision",
            NumericFieldKind::EpisodeSteps => "Episode Steps",
            NumericFieldKind::MutationRate => "Mutation Rate",
            NumericFieldKind::FitnessWeightSurvival => "Fit Survival",
            NumericFieldKind::FitnessWeightFood => "Fit Food",
            NumericFieldKind::FitnessWeightPredation => "Fit Predation",
            NumericFieldKind::FitnessWeightEnergy => "Fit Energy",
        }
    }

    fn toggle_field_label(kind: ToggleFieldKind) -> &'static str {
        match kind {
            ToggleFieldKind::FoodClusterEnabled => "Food Clusters",
            ToggleFieldKind::DayNightCycleEnabled => "Day/Night",
            ToggleFieldKind::EnergyBySpeed => "Energy by Speed",
            ToggleFieldKind::EnergyByNetworkSize => "Energy by Net",
            ToggleFieldKind::EnergyByPerceptionSize => "Energy by Vision",
        }
    }

    fn toggle_field_value(kind: ToggleFieldKind, config: &crate::config::SimulationConfig) -> bool {
        match kind {
            ToggleFieldKind::FoodClusterEnabled => config.food_cluster_enabled,
            ToggleFieldKind::DayNightCycleEnabled => config.day_night_cycle_enabled,
            ToggleFieldKind::EnergyBySpeed => config.rule_toggles.energy_by_speed,
            ToggleFieldKind::EnergyByNetworkSize => config.rule_toggles.energy_by_network_size,
            ToggleFieldKind::EnergyByPerceptionSize => {
                config.rule_toggles.energy_by_perception_size
            }
        }
    }

    fn format_toggle_value(value: bool) -> &'static str {
        if value { "On" } else { "Off" }
    }

    fn numeric_field_string(
        kind: NumericFieldKind,
        evolve_generations: u32,
        config: &crate::config::SimulationConfig,
    ) -> String {
        match kind {
            NumericFieldKind::EvolveGenerations => evolve_generations.to_string(),
            NumericFieldKind::EvaluationSwitchAfterGeneration => {
                config.evaluation_mode_switch_after_generation.to_string()
            }
            NumericFieldKind::WorldWidth => format!("{:.1}", config.world_width),
            NumericFieldKind::WorldHeight => format!("{:.1}", config.world_height),
            NumericFieldKind::TimestepSeconds => format!("{:.3}", config.timestep_seconds),
            NumericFieldKind::PopulationSize => config.population_size.to_string(),
            NumericFieldKind::FoodCount => config.food_count.to_string(),
            NumericFieldKind::FoodMotionSpeed => format!("{:.2}", config.food_motion_speed),
            NumericFieldKind::FoodRespawnProbability => {
                format!("{:.3}", config.food_respawn_probability)
            }
            NumericFieldKind::FoodClusterCount => config.food_cluster_count.to_string(),
            NumericFieldKind::FoodClusterRadius => format!("{:.2}", config.food_cluster_radius),
            NumericFieldKind::FoodClusterMotionSpeed => {
                format!("{:.2}", config.food_cluster_motion_speed)
            }
            NumericFieldKind::FoodClusterProductivityBias => {
                format!("{:.3}", config.food_cluster_productivity_bias)
            }
            NumericFieldKind::ElitismFraction => format!("{:.3}", config.elitism_fraction),
            NumericFieldKind::TournamentSize => config.tournament_size.to_string(),
            NumericFieldKind::MutationRate => format!("{:.3}", config.mutation_rate),
            NumericFieldKind::InitialAgentEnergy => format!("{:.2}", config.initial_agent_energy),
            NumericFieldKind::BaseEnergyCostPerSecond => {
                format!("{:.3}", config.base_energy_cost_per_second)
            }
            NumericFieldKind::SpeedEnergyCostFactor => {
                format!("{:.3}", config.speed_energy_cost_factor)
            }
            NumericFieldKind::NetworkEnergyCostFactor => {
                format!("{:.4}", config.network_energy_cost_factor)
            }
            NumericFieldKind::PerceptionEnergyCostFactor => {
                format!("{:.4}", config.perception_energy_cost_factor)
            }
            NumericFieldKind::FoodEnergyGain => format!("{:.2}", config.food_energy_gain),
            NumericFieldKind::FoodCollisionRadius => format!("{:.2}", config.food_collision_radius),
            NumericFieldKind::EyeFovAngleRadians => format!("{:.3}", config.eye_fov_angle_radians),
            NumericFieldKind::DefaultAgentMaxSpeed => {
                format!("{:.2}", config.default_agent_max_speed)
            }
            NumericFieldKind::MaxAcceleration => format!("{:.2}", config.max_acceleration),
            NumericFieldKind::LinearDrag => format!("{:.2}", config.linear_drag),
            NumericFieldKind::MaxTurnRateRadians => {
                format!("{:.3}", config.max_turn_rate_radians)
            }
            NumericFieldKind::DefaultPerceptionRadius => {
                format!("{:.2}", config.default_perception_radius)
            }
            NumericFieldKind::DefaultPerceptionChannels => {
                config.default_perception_channels.to_string()
            }
            NumericFieldKind::DefaultNetworkSize => config.default_network_size.to_string(),
            NumericFieldKind::MlpHiddenSize => config.mlp_hidden_size.to_string(),
            NumericFieldKind::LtcStateSize => config.ltc_state_size.to_string(),
            NumericFieldKind::DefaultMetabolicEfficiency => {
                format!("{:.3}", config.default_metabolic_efficiency)
            }
            NumericFieldKind::ControllerLtcProbability => {
                format!("{:.3}", config.controller_ltc_probability)
            }
            NumericFieldKind::PredatorFraction => format!("{:.3}", config.predator_fraction),
            NumericFieldKind::PredatorCaptureRadius => {
                format!("{:.2}", config.predator_capture_radius)
            }
            NumericFieldKind::PredatorCaptureEnergyGain => {
                format!("{:.2}", config.predator_capture_energy_gain)
            }
            NumericFieldKind::SensorNoiseStddev => format!("{:.3}", config.sensor_noise_stddev),
            NumericFieldKind::SensorDropoutProbability => {
                format!("{:.3}", config.sensor_dropout_probability)
            }
            NumericFieldKind::DayNightCycleSteps => config.day_night_cycle_steps.to_string(),
            NumericFieldKind::NightVisibilityMultiplier => {
                format!("{:.3}", config.night_visibility_multiplier)
            }
            NumericFieldKind::EpisodeSteps => config.episode_steps.to_string(),
            NumericFieldKind::FitnessWeightSurvival => {
                format!("{:.3}", config.fitness_weight_survival)
            }
            NumericFieldKind::FitnessWeightFood => format!("{:.3}", config.fitness_weight_food),
            NumericFieldKind::FitnessWeightPredation => {
                format!("{:.3}", config.fitness_weight_predation)
            }
            NumericFieldKind::FitnessWeightEnergy => {
                format!("{:.3}", config.fitness_weight_energy)
            }
        }
    }

    fn format_evaluation_mode(mode: EvaluationMode) -> &'static str {
        match mode {
            EvaluationMode::Solo => "Solo",
            EvaluationMode::Shared => "Shared",
        }
    }

    fn format_evaluation_schedule(config: &crate::config::SimulationConfig) -> String {
        if config.evaluation_mode_switch_after_generation == 0 {
            format_evaluation_mode(config.evaluation_mode).to_string()
        } else {
            format!(
                "{}->{}@{}",
                format_evaluation_mode(config.evaluation_mode),
                format_evaluation_mode(config.evaluation_mode_after_switch),
                config.evaluation_mode_switch_after_generation
            )
        }
    }

    fn toggle_evaluation_mode(mode: EvaluationMode) -> EvaluationMode {
        match mode {
            EvaluationMode::Solo => EvaluationMode::Shared,
            EvaluationMode::Shared => EvaluationMode::Solo,
        }
    }

    fn visible_progress_completed(job: &EvolutionJob) -> u32 {
        if job.total_generations < 10 || job.completed_generations >= job.total_generations {
            return job.completed_generations.min(job.total_generations);
        }

        (job.completed_generations / 10) * 10
    }

    fn accepts_numeric_char(kind: NumericFieldKind, ch: char, current: &str) -> bool {
        if ch.is_ascii_digit() {
            return true;
        }

        matches!(
            kind,
            NumericFieldKind::WorldWidth
                | NumericFieldKind::WorldHeight
                | NumericFieldKind::TimestepSeconds
                | NumericFieldKind::FoodMotionSpeed
                | NumericFieldKind::FoodRespawnProbability
                | NumericFieldKind::FoodClusterRadius
                | NumericFieldKind::FoodClusterMotionSpeed
                | NumericFieldKind::FoodClusterProductivityBias
                | NumericFieldKind::ElitismFraction
                | NumericFieldKind::InitialAgentEnergy
                | NumericFieldKind::BaseEnergyCostPerSecond
                | NumericFieldKind::SpeedEnergyCostFactor
                | NumericFieldKind::NetworkEnergyCostFactor
                | NumericFieldKind::PerceptionEnergyCostFactor
                | NumericFieldKind::FoodEnergyGain
                | NumericFieldKind::FoodCollisionRadius
                | NumericFieldKind::EyeFovAngleRadians
                | NumericFieldKind::DefaultAgentMaxSpeed
                | NumericFieldKind::MaxAcceleration
                | NumericFieldKind::LinearDrag
                | NumericFieldKind::MaxTurnRateRadians
                | NumericFieldKind::DefaultPerceptionRadius
                | NumericFieldKind::DefaultMetabolicEfficiency
                | NumericFieldKind::ControllerLtcProbability
                | NumericFieldKind::PredatorFraction
                | NumericFieldKind::PredatorCaptureRadius
                | NumericFieldKind::PredatorCaptureEnergyGain
                | NumericFieldKind::SensorNoiseStddev
                | NumericFieldKind::SensorDropoutProbability
                | NumericFieldKind::NightVisibilityMultiplier
                | NumericFieldKind::MutationRate
                | NumericFieldKind::FitnessWeightSurvival
                | NumericFieldKind::FitnessWeightFood
                | NumericFieldKind::FitnessWeightPredation
                | NumericFieldKind::FitnessWeightEnergy
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
            NumericFieldKind::EvaluationSwitchAfterGeneration => {
                if let Ok(value) = raw.parse::<u32>() {
                    runtime.config.evaluation_mode_switch_after_generation = value;
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
            NumericFieldKind::TimestepSeconds => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.timestep_seconds = value.clamp(0.001, TIMESTEP_SECONDS_MAX);
                }
            }
            NumericFieldKind::PopulationSize => {
                if let Ok(value) = raw.parse::<usize>() {
                    runtime.config.population_size = value.clamp(1, POPULATION_SIZE_MAX);
                }
            }
            NumericFieldKind::FoodCount => {
                if let Ok(value) = raw.parse::<usize>() {
                    runtime.config.food_count = value.clamp(FOOD_COUNT_MIN, FOOD_COUNT_MAX);
                }
            }
            NumericFieldKind::FoodMotionSpeed => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.food_motion_speed = value.clamp(0.0, FOOD_MOTION_SPEED_MAX);
                }
            }
            NumericFieldKind::FoodRespawnProbability => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.food_respawn_probability = value.clamp(0.0, 1.0);
                }
            }
            NumericFieldKind::FoodClusterCount => {
                if let Ok(value) = raw.parse::<usize>() {
                    runtime.config.food_cluster_count = value.clamp(1, FOOD_CLUSTER_COUNT_MAX);
                }
            }
            NumericFieldKind::FoodClusterRadius => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.food_cluster_radius = value.clamp(0.01, FOOD_CLUSTER_RADIUS_MAX);
                }
            }
            NumericFieldKind::FoodClusterMotionSpeed => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.food_cluster_motion_speed =
                        value.clamp(0.0, FOOD_CLUSTER_MOTION_SPEED_MAX);
                }
            }
            NumericFieldKind::FoodClusterProductivityBias => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.food_cluster_productivity_bias = value.clamp(0.0, 1.0);
                }
            }
            NumericFieldKind::ElitismFraction => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.elitism_fraction = value.clamp(0.001, 1.0);
                }
            }
            NumericFieldKind::TournamentSize => {
                if let Ok(value) = raw.parse::<usize>() {
                    runtime.config.tournament_size = value.clamp(1, TOURNAMENT_SIZE_MAX);
                }
            }
            NumericFieldKind::InitialAgentEnergy => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.initial_agent_energy =
                        value.clamp(0.001, INITIAL_AGENT_ENERGY_MAX);
                }
            }
            NumericFieldKind::BaseEnergyCostPerSecond => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.base_energy_cost_per_second = value.clamp(0.0, ENERGY_COST_MAX);
                }
            }
            NumericFieldKind::SpeedEnergyCostFactor => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.speed_energy_cost_factor = value.clamp(0.0, ENERGY_COST_MAX);
                }
            }
            NumericFieldKind::NetworkEnergyCostFactor => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.network_energy_cost_factor = value.clamp(0.0, ENERGY_COST_MAX);
                }
            }
            NumericFieldKind::PerceptionEnergyCostFactor => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.perception_energy_cost_factor =
                        value.clamp(0.0, ENERGY_COST_MAX);
                }
            }
            NumericFieldKind::FoodEnergyGain => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.food_energy_gain = value.clamp(0.001, FOOD_ENERGY_GAIN_MAX);
                }
            }
            NumericFieldKind::FoodCollisionRadius => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.food_collision_radius =
                        value.clamp(0.001, FOOD_COLLISION_RADIUS_MAX);
                }
            }
            NumericFieldKind::EyeFovAngleRadians => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.eye_fov_angle_radians = value.clamp(0.001, EYE_FOV_MAX);
                }
            }
            NumericFieldKind::DefaultAgentMaxSpeed => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.default_agent_max_speed = value.clamp(0.001, DEFAULT_SPEED_MAX);
                }
            }
            NumericFieldKind::MaxAcceleration => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.max_acceleration = value.clamp(0.01, MAX_ACCELERATION_MAX);
                }
            }
            NumericFieldKind::LinearDrag => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.linear_drag = value.clamp(0.0, LINEAR_DRAG_MAX);
                }
            }
            NumericFieldKind::MaxTurnRateRadians => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.max_turn_rate_radians = value.clamp(0.001, TURN_RATE_MAX);
                }
            }
            NumericFieldKind::DefaultPerceptionRadius => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.default_perception_radius =
                        value.clamp(0.001, PERCEPTION_RADIUS_MAX);
                }
            }
            NumericFieldKind::DefaultPerceptionChannels => {
                if let Ok(value) = raw.parse::<u16>() {
                    runtime.config.default_perception_channels =
                        value.clamp(1, crate::config::MAX_PERCEPTION_CHANNELS);
                }
            }
            NumericFieldKind::DefaultNetworkSize => {
                if let Ok(value) = raw.parse::<usize>() {
                    runtime.config.default_network_size = value.clamp(1, NETWORK_SIZE_MAX);
                }
            }
            NumericFieldKind::MlpHiddenSize => {
                if let Ok(value) = raw.parse::<usize>() {
                    runtime.config.mlp_hidden_size = value.clamp(1, HIDDEN_LAYER_SIZE_MAX);
                }
            }
            NumericFieldKind::LtcStateSize => {
                if let Ok(value) = raw.parse::<usize>() {
                    runtime.config.ltc_state_size = value.clamp(1, HIDDEN_LAYER_SIZE_MAX);
                }
            }
            NumericFieldKind::DefaultMetabolicEfficiency => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.default_metabolic_efficiency =
                        value.clamp(0.001, METABOLIC_EFFICIENCY_MAX);
                }
            }
            NumericFieldKind::ControllerLtcProbability => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.controller_ltc_probability = value.clamp(0.0, 1.0);
                }
            }
            NumericFieldKind::PredatorFraction => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.predator_fraction = value.clamp(0.0, 1.0);
                }
            }
            NumericFieldKind::PredatorCaptureRadius => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.predator_capture_radius =
                        value.clamp(0.001, FOOD_COLLISION_RADIUS_MAX);
                }
            }
            NumericFieldKind::PredatorCaptureEnergyGain => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.predator_capture_energy_gain =
                        value.clamp(0.001, FOOD_ENERGY_GAIN_MAX);
                }
            }
            NumericFieldKind::SensorNoiseStddev => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.sensor_noise_stddev = value.clamp(0.0, SENSOR_NOISE_STDDEV_MAX);
                }
            }
            NumericFieldKind::SensorDropoutProbability => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.sensor_dropout_probability = value.clamp(0.0, 1.0);
                }
            }
            NumericFieldKind::DayNightCycleSteps => {
                if let Ok(value) = raw.parse::<u32>() {
                    runtime.config.day_night_cycle_steps =
                        value.clamp(1, DAY_NIGHT_CYCLE_STEPS_MAX);
                }
            }
            NumericFieldKind::NightVisibilityMultiplier => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.night_visibility_multiplier = value.clamp(0.0, 1.0);
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
            NumericFieldKind::FitnessWeightSurvival => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.fitness_weight_survival = value.clamp(0.0, FITNESS_WEIGHT_MAX);
                }
            }
            NumericFieldKind::FitnessWeightFood => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.fitness_weight_food = value.clamp(0.0, FITNESS_WEIGHT_MAX);
                }
            }
            NumericFieldKind::FitnessWeightPredation => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.fitness_weight_predation = value.clamp(0.0, FITNESS_WEIGHT_MAX);
                }
            }
            NumericFieldKind::FitnessWeightEnergy => {
                if let Ok(value) = raw.parse::<f32>() {
                    runtime.config.fitness_weight_energy = value.clamp(0.0, FITNESS_WEIGHT_MAX);
                }
            }
        }
    }

    fn normalize_numeric_field_buffer(runtime: &mut ViewerRuntime, field: NumericFieldKind) {
        runtime.field_buffers.0.insert(
            field,
            numeric_field_string(field, runtime.evolve_generations, &runtime.config),
        );
    }

    fn reset_runtime_to_snapshot(runtime: &mut ViewerRuntime) {
        runtime.world = runtime.initial_snapshot.world_state.clone();
        runtime.config = runtime.initial_snapshot.resolved_config.clone();
        runtime.population = runtime.initial_snapshot.population.clone();
        runtime.controllers = build_runtime_controllers(&runtime.config, &runtime.population);
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
        runtime.metrics_history = runtime.initial_snapshot.metrics_history.clone();
        runtime.active_input = None;
        runtime.field_buffers =
            NumericFieldBuffers::from_state(runtime.evolve_generations, &runtime.config);
        if runtime
            .selected_agent_index
            .is_some_and(|idx| idx >= runtime.world.agents.len())
        {
            runtime.selected_agent_index = None;
        }
        refresh_selected_agent_preview(runtime);
        runtime.evolution_job = None;
        runtime.rebuild_visuals = true;
    }

    fn sampled_metrics(
        metrics: &[GenerationMetrics],
        max_points: usize,
    ) -> Option<Vec<&GenerationMetrics>> {
        if metrics.is_empty() {
            return None;
        }

        if metrics.len() <= max_points {
            return Some(metrics.iter().collect());
        }

        let stride = metrics.len().div_ceil(max_points);
        let mut sampled = metrics.iter().step_by(stride).collect::<Vec<_>>();
        if let Some(last) = metrics.last() {
            if sampled
                .last()
                .map(|entry| entry.generation_index != last.generation_index)
                .unwrap_or(true)
            {
                sampled.push(last);
            }
        }
        Some(sampled)
    }

    fn merge_generation_metrics(
        history: &mut Vec<GenerationMetrics>,
        new_metrics: &[GenerationMetrics],
    ) {
        for metric in new_metrics {
            if let Some(existing) = history
                .iter_mut()
                .find(|existing| existing.generation_index == metric.generation_index)
            {
                *existing = metric.clone();
            } else {
                history.push(metric.clone());
            }
        }
        history.sort_by_key(|metric| metric.generation_index);
    }

    fn draw_screen_line(
        gizmos: &mut Gizmos,
        window: &Window,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        color: Color,
    ) {
        gizmos.line_2d(
            screen_to_world(window, x0, y0),
            screen_to_world(window, x1, y1),
            color,
        );
    }

    fn draw_world_circle(
        gizmos: &mut Gizmos,
        center: Vec2,
        radius: f32,
        width: f32,
        height: f32,
        color: Color,
    ) {
        const SEGMENTS: usize = 40;
        let mut previous = None;

        for segment in 0..=SEGMENTS {
            let theta = segment as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
            let point = wrap_position(
                center.add(Vec2::from_angle(theta).scale(radius)),
                width,
                height,
            );
            let point = world_to_screen(point, width, height, -0.1).truncate();
            if let Some(prev) = previous {
                gizmos.line_2d(prev, point, color);
            }
            previous = Some(point);
        }
    }

    fn food_visual_color(food: &FoodState) -> Color {
        match food.cluster_id {
            Some(cluster_id) => {
                let (r, g, b) = cluster_palette(cluster_id);
                Color::srgb(r, g, b)
            }
            None => Color::srgb(0.10, 0.90, 0.30),
        }
    }

    fn cluster_visual_color(cluster_id: u32, productivity: f32) -> Color {
        let (r, g, b) = cluster_palette(cluster_id);
        Color::srgba(r, g, b, 0.16 + 0.18 * productivity)
    }

    fn cluster_palette(cluster_id: u32) -> (f32, f32, f32) {
        match cluster_id % 5 {
            0 => (0.20, 0.88, 0.48),
            1 => (0.34, 0.72, 1.00),
            2 => (1.00, 0.73, 0.25),
            3 => (0.98, 0.48, 0.40),
            _ => (0.78, 0.56, 1.00),
        }
    }

    fn draw_layered_network(
        gizmos: &mut Gizmos,
        window: &Window,
        left: f32,
        right: f32,
        top: f32,
        bottom: f32,
        input_values: &[f32],
        middle_values: &[f32],
        output_values: &[f32],
        input_middle_weights: &[f32],
        middle_output_weights: &[f32],
        recurrent_weights: Option<&[f32]>,
    ) {
        let columns = [
            left + (right - left) * 0.10,
            left + (right - left) * 0.50,
            left + (right - left) * 0.90,
        ];
        let input_positions = layer_node_positions(columns[0], top, bottom, input_values.len());
        let middle_positions = layer_node_positions(columns[1], top, bottom, middle_values.len());
        let output_positions = layer_node_positions(columns[2], top, bottom, output_values.len());

        draw_weight_matrix(
            gizmos,
            window,
            &input_positions,
            &middle_positions,
            input_middle_weights,
            input_values.len(),
        );
        draw_weight_matrix(
            gizmos,
            window,
            &middle_positions,
            &output_positions,
            middle_output_weights,
            middle_values.len(),
        );

        if let Some(recurrent) = recurrent_weights {
            for (from_idx, from) in middle_positions.iter().enumerate() {
                for (to_idx, to) in middle_positions.iter().enumerate() {
                    let weight = recurrent[from_idx * middle_positions.len() + to_idx];
                    if weight.abs() < 0.2 {
                        continue;
                    }

                    let x_mid = from.x + (to.x - from.x) * 0.5 + 12.0;
                    let y_mid = from.y + (to.y - from.y) * 0.5;
                    let color = weight_color(weight, 0.08);
                    draw_screen_line(gizmos, window, from.x, from.y, x_mid, y_mid, color);
                    draw_screen_line(gizmos, window, x_mid, y_mid, to.x, to.y, color);
                }
            }
        }

        draw_node_layer(gizmos, window, &input_positions, input_values);
        draw_node_layer(gizmos, window, &middle_positions, middle_values);
        draw_node_layer(gizmos, window, &output_positions, output_values);
    }

    fn layer_node_positions(x: f32, top: f32, bottom: f32, count: usize) -> Vec<bevy::math::Vec2> {
        if count == 0 {
            return Vec::new();
        }

        let span = (bottom - top).max(1.0);
        let step = if count == 1 {
            0.0
        } else {
            span / (count - 1) as f32
        };

        (0..count)
            .map(|idx| bevy::math::Vec2::new(x, top + idx as f32 * step))
            .collect()
    }

    fn draw_weight_matrix(
        gizmos: &mut Gizmos,
        window: &Window,
        from_positions: &[bevy::math::Vec2],
        to_positions: &[bevy::math::Vec2],
        weights: &[f32],
        row_width: usize,
    ) {
        if row_width == 0 || from_positions.is_empty() || to_positions.is_empty() {
            return;
        }

        for (to_idx, to) in to_positions.iter().enumerate() {
            for (from_idx, from) in from_positions.iter().enumerate() {
                let weight = weights[to_idx * row_width + from_idx];
                if weight.abs() < 0.15 {
                    continue;
                }

                draw_screen_line(
                    gizmos,
                    window,
                    from.x,
                    from.y - 0.7,
                    to.x,
                    to.y - 0.7,
                    weight_color(weight, 0.16),
                );
                draw_screen_line(
                    gizmos,
                    window,
                    from.x,
                    from.y + 0.7,
                    to.x,
                    to.y + 0.7,
                    weight_color(weight, 0.06),
                );
            }
        }
    }

    fn draw_node_layer(
        gizmos: &mut Gizmos,
        window: &Window,
        positions: &[bevy::math::Vec2],
        values: &[f32],
    ) {
        for (position, value) in positions.iter().zip(values.iter()) {
            let color = activation_color(*value);
            draw_screen_line(
                gizmos,
                window,
                position.x - 5.0,
                position.y,
                position.x + 5.0,
                position.y,
                Color::srgba(0.96, 0.98, 1.0, 0.28),
            );
            draw_screen_line(
                gizmos,
                window,
                position.x - 4.0,
                position.y,
                position.x + 4.0,
                position.y,
                color,
            );
            draw_screen_line(
                gizmos,
                window,
                position.x,
                position.y - 5.0,
                position.x,
                position.y + 5.0,
                Color::srgba(0.96, 0.98, 1.0, 0.28),
            );
            draw_screen_line(
                gizmos,
                window,
                position.x,
                position.y - 4.0,
                position.x,
                position.y + 4.0,
                color,
            );
        }
    }

    fn activation_color(value: f32) -> Color {
        let magnitude = value.abs().clamp(0.0, 1.0);
        if value >= 0.0 {
            Color::srgba(0.42, 1.0, 0.84, 0.55 + magnitude * 0.45)
        } else {
            Color::srgba(1.0, 0.54, 0.34, 0.55 + magnitude * 0.45)
        }
    }

    fn weight_color(weight: f32, min_alpha: f32) -> Color {
        let magnitude = weight.abs().clamp(0.0, 1.0);
        if weight >= 0.0 {
            Color::srgba(0.52, 0.86, 1.0, min_alpha + magnitude * 0.44)
        } else {
            Color::srgba(1.0, 0.60, 0.46, min_alpha + magnitude * 0.44)
        }
    }

    fn lighten_color(color: Color, amount: f32) -> Color {
        let srgba = color.to_srgba();
        Color::srgb(
            (srgba.red + amount).clamp(0.0, 1.0),
            (srgba.green + amount).clamp(0.0, 1.0),
            (srgba.blue + amount).clamp(0.0, 1.0),
        )
    }

    fn draw_species_series<Fv, Fx, Fy>(
        gizmos: &mut Gizmos,
        window: &Window,
        metrics: &[&GenerationMetrics],
        value_for_metric: Fv,
        x_for_generation: &Fx,
        y_for_fitness: &Fy,
        color: Color,
        dashed: bool,
    ) where
        Fv: Fn(&GenerationMetrics) -> Option<f32>,
        Fx: Fn(u32) -> f32,
        Fy: Fn(f32) -> f32,
    {
        for pair in metrics.windows(2) {
            let previous = pair[0];
            let current = pair[1];
            let (Some(previous_value), Some(current_value)) =
                (value_for_metric(previous), value_for_metric(current))
            else {
                continue;
            };

            if dashed {
                draw_dashed_screen_line(
                    gizmos,
                    window,
                    x_for_generation(previous.generation_index),
                    y_for_fitness(previous_value),
                    x_for_generation(current.generation_index),
                    y_for_fitness(current_value),
                    color,
                );
            } else {
                draw_screen_line(
                    gizmos,
                    window,
                    x_for_generation(previous.generation_index),
                    y_for_fitness(previous_value),
                    x_for_generation(current.generation_index),
                    y_for_fitness(current_value),
                    color,
                );
            }
        }
    }

    fn draw_dashed_screen_line(
        gizmos: &mut Gizmos,
        window: &Window,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        color: Color,
    ) {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let distance = (dx * dx + dy * dy).sqrt();
        if distance <= f32::EPSILON {
            return;
        }

        let dash = 8.0;
        let gap = 6.0;
        let step = dash + gap;
        let ux = dx / distance;
        let uy = dy / distance;
        let mut cursor = 0.0;
        while cursor < distance {
            let segment_end = (cursor + dash).min(distance);
            draw_screen_line(
                gizmos,
                window,
                x0 + ux * cursor,
                y0 + uy * cursor,
                x0 + ux * segment_end,
                y0 + uy * segment_end,
                color,
            );
            cursor += step;
        }
    }

    fn format_optional(value: Option<f32>) -> String {
        value
            .map(|value| format!("{value:.2}"))
            .unwrap_or_else(|| "-".to_string())
    }

    fn screen_to_world(window: &Window, x: f32, y: f32) -> bevy::math::Vec2 {
        bevy::math::Vec2::new(x - window.width() * 0.5, window.height() * 0.5 - y)
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
