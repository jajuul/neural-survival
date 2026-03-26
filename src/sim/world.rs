use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::config::SimulationConfig;

use super::types::{
    ActionVector, AgentRole, AgentState, CONTROLLER_SENSOR_INPUT_SIZE, CORE_SENSOR_FEATURES,
    DayNightState, EnergyBreakdown, EpisodeSummary, FoodClusterState, FoodState, MAX_VISION_CELLS,
    SensorObservation, SensorVector, StepAgentTelemetry, StepOutcome, Vec2, WorldState,
};

/// Coarse toroidal spatial index used to prune food/agent sensor queries.
struct SpatialIndex {
    cell_size: f32,
    cols: usize,
    rows: usize,
    max_food_radius: f32,
    alive_prey_count: usize,
    alive_predator_count: usize,
    food_cells: Vec<Vec<usize>>,
    agent_cells: Vec<Vec<usize>>,
}

impl SpatialIndex {
    fn build(world: &WorldState) -> Self {
        let cell_size = world
            .agents
            .iter()
            .map(|agent| agent.perception_radius.max(0.001))
            .fold(1.0_f32, f32::max);
        let cols = ((world.width / cell_size).ceil() as usize).max(1);
        let rows = ((world.height / cell_size).ceil() as usize).max(1);
        let max_food_radius = world
            .food
            .iter()
            .map(|food| food.radius.max(0.0))
            .fold(0.0_f32, f32::max);
        let mut food_cells = vec![Vec::new(); cols * rows];
        let mut agent_cells = vec![Vec::new(); cols * rows];
        let mut alive_prey_count = 0;
        let mut alive_predator_count = 0;

        for (food_idx, food) in world.food.iter().enumerate() {
            let cell_idx = Self::flat_index_for_position(food.position, cell_size, cols, rows);
            food_cells[cell_idx].push(food_idx);
        }

        for (agent_idx, agent) in world.agents.iter().enumerate() {
            if !agent.alive {
                continue;
            }

            match agent.role {
                AgentRole::Prey => alive_prey_count += 1,
                AgentRole::Predator => alive_predator_count += 1,
            }

            let cell_idx = Self::flat_index_for_position(agent.position, cell_size, cols, rows);
            agent_cells[cell_idx].push(agent_idx);
        }

        Self {
            cell_size,
            cols,
            rows,
            max_food_radius,
            alive_prey_count,
            alive_predator_count,
            food_cells,
            agent_cells,
        }
    }

    fn with_candidate_foods<F>(&self, position: Vec2, radius: f32, mut visit: F)
    where
        F: FnMut(usize),
    {
        self.with_candidate_indices(position, radius, &self.food_cells, &mut visit);
    }

    fn with_candidate_agents<F>(&self, position: Vec2, radius: f32, mut visit: F)
    where
        F: FnMut(usize),
    {
        self.with_candidate_indices(position, radius, &self.agent_cells, &mut visit);
    }

    fn has_sensor_targets(&self, observer_role: AgentRole, target_role: AgentRole) -> bool {
        match target_role {
            AgentRole::Prey if observer_role == AgentRole::Prey => self.alive_prey_count > 1,
            AgentRole::Prey => self.alive_prey_count > 0,
            AgentRole::Predator if observer_role == AgentRole::Predator => {
                self.alive_predator_count > 1
            }
            AgentRole::Predator => self.alive_predator_count > 0,
        }
    }

    fn has_live_predator_and_prey(&self) -> bool {
        self.alive_predator_count > 0 && self.alive_prey_count > 0
    }

    fn with_candidate_indices<F>(
        &self,
        position: Vec2,
        radius: f32,
        cells: &[Vec<usize>],
        visit: &mut F,
    ) where
        F: FnMut(usize),
    {
        let (cell_x, cell_y) = self.cell_coords(position);
        let cell_radius = ((radius.max(0.0) / self.cell_size).ceil() as isize).max(0);

        for dy in -cell_radius..=cell_radius {
            let y = wrap_cell_index(cell_y as isize + dy, self.rows);
            for dx in -cell_radius..=cell_radius {
                let x = wrap_cell_index(cell_x as isize + dx, self.cols);
                let flat_idx = self.flat_index(x, y);
                for &entry in &cells[flat_idx] {
                    visit(entry);
                }
            }
        }
    }

    fn cell_coords(&self, position: Vec2) -> (usize, usize) {
        (
            ((position.x / self.cell_size).floor() as usize).min(self.cols - 1),
            ((position.y / self.cell_size).floor() as usize).min(self.rows - 1),
        )
    }

    fn flat_index(&self, x: usize, y: usize) -> usize {
        y * self.cols + x
    }

    fn flat_index_for_position(position: Vec2, cell_size: f32, cols: usize, rows: usize) -> usize {
        let x = ((position.x / cell_size).floor() as usize).min(cols - 1);
        let y = ((position.y / cell_size).floor() as usize).min(rows - 1);
        y * cols + x
    }
}

/// Build a deterministic world state from config defaults and a seed.
///
/// Inputs: static `config` values and a deterministic `seed`.
/// Output: newly initialized world with random-but-repeatable agent/food placement.
/// Side effects: none.
pub fn initialize_world(config: &SimulationConfig, seed: u64) -> WorldState {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let agents = (0..config.population_size)
        .map(|id| AgentState {
            id: id as u32,
            position: Vec2::new(
                rng.gen_range(0.0..config.world_width),
                rng.gen_range(0.0..config.world_height),
            ),
            heading_radians: rng.gen_range(0.0..std::f32::consts::TAU),
            velocity: Vec2::default(),
            energy: config.initial_agent_energy,
            alive: true,
            max_speed: config.default_agent_max_speed,
            perception_radius: config.default_perception_radius,
            perception_channels: config.default_perception_channels,
            eye_cell_angle_radians: config.eye_fov_angle_radians
                / config.default_perception_channels.max(1) as f32,
            network_size: config.default_network_size,
            metabolic_efficiency: config.default_metabolic_efficiency,
            role: if rng.gen_bool(config.predator_fraction as f64) {
                AgentRole::Predator
            } else {
                AgentRole::Prey
            },
            food_consumed: 0,
            prey_captured: 0,
        })
        .collect();

    let food_clusters = initialize_food_clusters(config, &mut rng);
    let mut world = WorldState {
        width: config.world_width,
        height: config.world_height,
        agents,
        food: (0..config.food_count)
            .map(|id| initialize_food_item(id as u32, config, &food_clusters, &mut rng))
            .collect(),
        food_clusters,
        world_seed: seed,
        step_count: 0,
    };
    apply_cluster_food_layout(&mut world, config);
    world
}

fn initialize_food_clusters(
    config: &SimulationConfig,
    rng: &mut ChaCha8Rng,
) -> Vec<FoodClusterState> {
    if !config.food_cluster_enabled {
        return Vec::new();
    }

    (0..config.food_cluster_count)
        .map(|id| FoodClusterState {
            id: id as u32,
            position: Vec2::new(
                rng.gen_range(0.0..config.world_width),
                rng.gen_range(0.0..config.world_height),
            ),
            motion_angle_radians: rng.gen_range(0.0..std::f32::consts::TAU),
            phase_offset: rng.gen_range(0.0..1.0),
        })
        .collect()
}

fn initialize_food_item(
    id: u32,
    config: &SimulationConfig,
    clusters: &[FoodClusterState],
    rng: &mut ChaCha8Rng,
) -> FoodState {
    if let Some(cluster) = clusters.get(id as usize % clusters.len().max(1)) {
        FoodState {
            id,
            position: cluster.position,
            radius: config.food_collision_radius,
            energy_value: config.food_energy_gain,
            motion_angle_radians: rng.gen_range(0.0..std::f32::consts::TAU),
            cluster_id: Some(cluster.id),
            cluster_angle_radians: rng.gen_range(0.0..std::f32::consts::TAU),
            cluster_distance_factor: rng.gen_range(0.0..1.0),
        }
    } else {
        FoodState {
            id,
            position: Vec2::new(
                rng.gen_range(0.0..config.world_width),
                rng.gen_range(0.0..config.world_height),
            ),
            radius: config.food_collision_radius,
            energy_value: config.food_energy_gain,
            motion_angle_radians: rng.gen_range(0.0..std::f32::consts::TAU),
            cluster_id: None,
            cluster_angle_radians: 0.0,
            cluster_distance_factor: 0.0,
        }
    }
}

/// Advance the simulation by one fixed step.
///
/// Inputs: mutable world state and validated simulation config.
/// Output: aggregate changes (`consumed_food`, `deaths`) for this step.
/// Side effects: mutates world state in-place.
pub fn step_world(world: &mut WorldState, config: &SimulationConfig) -> StepOutcome {
    let actions = world
        .agents
        .iter()
        .map(|agent| stub_action(agent.id, world.step_count))
        .collect::<Vec<_>>();

    step_world_with_actions(world, config, &actions)
}

/// Advance the simulation by one fixed step using caller-provided actions.
///
/// Inputs: mutable world state, validated simulation config, and one action per agent.
/// Output: aggregate changes (`consumed_food`, `deaths`) for this step.
/// Side effects: mutates world state in-place.
pub fn step_world_with_actions(
    world: &mut WorldState,
    config: &SimulationConfig,
    actions: &[ActionVector],
) -> StepOutcome {
    step_world_with_actions_and_telemetry(world, config, actions).0
}

/// Advance the simulation by one fixed step using caller-provided actions and capture per-agent
/// telemetry for observability.
///
/// Inputs: mutable world state, validated simulation config, and one action per agent.
/// Output: aggregate changes (`consumed_food`, `deaths`) and one telemetry record per agent index.
/// Side effects: mutates world state in-place.
pub fn step_world_with_actions_and_telemetry(
    world: &mut WorldState,
    config: &SimulationConfig,
    actions: &[ActionVector],
) -> (StepOutcome, Vec<StepAgentTelemetry>) {
    let mut outcome = StepOutcome::default();
    let dt = config.timestep_seconds;
    let mut telemetry = vec![StepAgentTelemetry::default(); world.agents.len()];
    let mut next_food_id = world.food.iter().map(|food| food.id).max().unwrap_or(0) + 1;
    let mut speeds = vec![0.0_f32; world.agents.len()];

    for agent_idx in 0..world.agents.len() {
        if !world.agents[agent_idx].alive {
            telemetry[agent_idx].alive_after = false;
            continue;
        }

        let action = actions
            .get(agent_idx)
            .copied()
            .unwrap_or_else(|| stub_action(world.agents[agent_idx].id, world.step_count));
        speeds[agent_idx] = apply_kinematics(
            &mut world.agents[agent_idx],
            action,
            world.width,
            world.height,
            config,
            dt,
        );
    }

    let spatial = (world.agents.len() > 1).then(|| SpatialIndex::build(world));
    let capture_gains = spatial
        .as_ref()
        .map(|spatial| resolve_predator_captures(world, spatial, config, &mut outcome))
        .unwrap_or_else(|| vec![0.0_f32; world.agents.len()]);
    let mut consumed_food_flags = vec![false; world.food.len()];
    let mut consumed_food_by_agent = vec![None; world.agents.len()];

    for agent_idx in 0..world.agents.len() {
        if !world.agents[agent_idx].alive {
            telemetry[agent_idx].alive_after = false;
            continue;
        }

        let food_gain = if world.agents[agent_idx].role == AgentRole::Prey {
            let food_idx = if let Some(spatial) = &spatial {
                find_collided_food_index(
                    &world.food,
                    spatial,
                    &consumed_food_flags,
                    world.agents[agent_idx].position,
                    world.width,
                    world.height,
                )
            } else {
                find_collided_food_index_direct(
                    &world.food,
                    &consumed_food_flags,
                    world.agents[agent_idx].position,
                    world.width,
                    world.height,
                )
            };
            if let Some(food_idx) = food_idx {
                consumed_food_flags[food_idx] = true;
                let food = world.food[food_idx].clone();
                consumed_food_by_agent[agent_idx] = Some(food.clone());
                world.agents[agent_idx].food_consumed += 1;
                outcome.consumed_food += 1;
                food.energy_value
            } else {
                0.0
            }
        } else {
            0.0
        };

        let capture_gain = capture_gains[agent_idx];
        let energy = energy_breakdown_for_step(
            &world.agents[agent_idx],
            config,
            speeds[agent_idx],
            dt,
            food_gain,
            capture_gain,
        );
        world.agents[agent_idx].energy =
            (world.agents[agent_idx].energy - energy.total_loss + food_gain + capture_gain)
                .max(0.0);

        if world.agents[agent_idx].energy <= 0.0 {
            world.agents[agent_idx].alive = false;
            outcome.deaths += 1;
        }

        telemetry[agent_idx] = StepAgentTelemetry {
            energy,
            consumed_food: food_gain > 0.0,
            captured_prey: capture_gain > 0.0,
            alive_after: world.agents[agent_idx].alive,
        };
    }

    // Remove consumed items in reverse index order so `swap_remove` can avoid O(n) shifting while
    // preserving deterministic "first claimant wins" collision semantics within the step.
    for food_idx in (0..consumed_food_flags.len()).rev() {
        if consumed_food_flags[food_idx] {
            world.food.swap_remove(food_idx);
        }
    }
    for (agent_idx, consumed_food) in consumed_food_by_agent.into_iter().enumerate() {
        if let Some(food) = consumed_food {
            maybe_respawn_food(world, config, agent_idx, &food, &mut next_food_id);
        }
    }
    spawn_food_at_fixed_rate(world, config, &mut next_food_id);

    move_food(world, config, dt);
    move_food_clusters(world, config, dt);
    world.step_count += 1;
    apply_cluster_food_layout(world, config);
    (outcome, telemetry)
}

/// Run one headless episode and return aggregate summary stats.
///
/// Inputs: validated config and deterministic seed.
/// Output: aggregate final episode statistics.
/// Side effects: none.
pub fn run_headless_episode(config: &SimulationConfig, seed: u64) -> EpisodeSummary {
    let mut world = initialize_world(config, seed);
    run_steps(&mut world, config, config.episode_steps);
    summarize_world(&world)
}

/// Advance world by `steps` fixed steps.
///
/// Inputs: mutable world state, config, and number of steps.
/// Output: none.
/// Side effects: mutates world state in-place.
pub fn run_steps(world: &mut WorldState, config: &SimulationConfig, steps: u32) {
    for _ in 0..steps {
        step_world(world, config);
    }
}

/// Build a compact baseline sensor vector for one agent.
///
/// Inputs: the current agent state, world dimensions, and config normalization constants.
/// Output: fixed-width vector
/// `[energy_norm, speed_norm, food_vision(10), prey_vision(10), predator_vision(10)]`, where
/// active bins are bounded by the agent's `perception_channels` trait.
/// Side effects: none.
pub fn basic_sensor_vector(
    agent: &AgentState,
    world: &WorldState,
    config: &SimulationConfig,
) -> SensorVector {
    if world.agents.len() <= 1 {
        return SensorVector {
            values: compute_sensor_values_single_agent(agent, world, config),
        };
    }

    let spatial = SpatialIndex::build(world);
    basic_sensor_vector_with_index(agent, world, &spatial, config)
}

/// Fill one fixed-width controller sensor vector per agent using a shared spatial index for the
/// current world snapshot.
///
/// Inputs: immutable world, validated config, and one output slot per world agent.
/// Output: none.
/// Side effects: overwrites `output` in place.
pub fn fill_population_sensor_vectors(
    world: &WorldState,
    config: &SimulationConfig,
    output: &mut [SensorVector],
) {
    debug_assert_eq!(output.len(), world.agents.len());

    let spatial = SpatialIndex::build(world);
    for (agent_idx, agent) in world.agents.iter().enumerate() {
        output[agent_idx] = if agent.alive {
            basic_sensor_vector_with_index(agent, world, &spatial, config)
        } else {
            SensorVector::default()
        };
    }
}

/// Compute deterministic day/night state from world time and config.
///
/// Inputs: current `step_count` and validated runtime config.
/// Output: normalized phase plus derived daylight and visibility multipliers.
/// Side effects: none.
pub fn current_day_night_state(step_count: u64, config: &SimulationConfig) -> DayNightState {
    if !config.day_night_cycle_enabled || config.day_night_cycle_steps == 0 {
        return DayNightState {
            phase: 0.0,
            daylight: 1.0,
            visibility_multiplier: 1.0,
            is_night: false,
        };
    }

    let cycle_steps = config.day_night_cycle_steps.max(1) as f32;
    let phase = (step_count as f32 / cycle_steps).fract();
    let daylight = (phase * std::f32::consts::TAU).cos() * 0.5 + 0.5;
    let visibility_multiplier =
        config.night_visibility_multiplier + (1.0 - config.night_visibility_multiplier) * daylight;

    DayNightState {
        phase,
        daylight,
        visibility_multiplier,
        is_night: daylight < 0.5,
    }
}

/// Build decomposed controller inputs for one agent so UI code can inspect core scalars and eye
/// bins without reimplementing the sensor model.
///
/// Inputs: current agent state, immutable world state, and validated config.
/// Output: core scalar features, decomposed vision activations, and the final fixed-width
/// controller input.
/// Side effects: none.
pub fn sensor_observation(
    agent: &AgentState,
    world: &WorldState,
    config: &SimulationConfig,
) -> SensorObservation {
    let values = if world.agents.len() <= 1 {
        compute_sensor_values_single_agent(agent, world, config)
    } else {
        let spatial = SpatialIndex::build(world);
        compute_sensor_values(agent, world, &spatial, config)
    };
    let food_start = CORE_SENSOR_FEATURES;
    let prey_start = CORE_SENSOR_FEATURES + MAX_VISION_CELLS;
    let predator_start = CORE_SENSOR_FEATURES + MAX_VISION_CELLS * 2;
    let food_eye_bins = values[food_start..prey_start].to_vec();
    let prey_eye_bins = values[prey_start..predator_start].to_vec();
    let predator_eye_bins = values[predator_start..predator_start + MAX_VISION_CELLS].to_vec();
    let vision_cells = values[CORE_SENSOR_FEATURES..].to_vec();

    SensorObservation {
        core_values: [values[0], values[1]],
        food_eye_bins,
        prey_eye_bins,
        predator_eye_bins,
        eye_bins: vision_cells,
        input: SensorVector { values },
    }
}

fn basic_sensor_vector_with_index(
    agent: &AgentState,
    world: &WorldState,
    spatial: &SpatialIndex,
    config: &SimulationConfig,
) -> SensorVector {
    SensorVector {
        values: compute_sensor_values(agent, world, spatial, config),
    }
}

fn compute_sensor_values(
    agent: &AgentState,
    world: &WorldState,
    spatial: &SpatialIndex,
    config: &SimulationConfig,
) -> [f32; CONTROLLER_SENSOR_INPUT_SIZE] {
    let day_night = current_day_night_state(world.step_count, config);
    let mut values = [0.0; CONTROLLER_SENSOR_INPUT_SIZE];
    values[0] = (agent.energy / config.initial_agent_energy).clamp(0.0, 2.0);
    values[1] = (agent.velocity.length() / agent.max_speed.max(0.001)).clamp(0.0, 1.0);

    let food_start = CORE_SENSOR_FEATURES;
    let prey_start = CORE_SENSOR_FEATURES + MAX_VISION_CELLS;
    let predator_start = CORE_SENSOR_FEATURES + MAX_VISION_CELLS * 2;
    fill_food_eye_cells(
        agent,
        world,
        spatial,
        &mut values[food_start..prey_start],
        day_night.visibility_multiplier,
    );
    if spatial.has_sensor_targets(agent.role, AgentRole::Prey) {
        fill_role_eye_cells(
            agent,
            world,
            spatial,
            &mut values[prey_start..predator_start],
            AgentRole::Prey,
            day_night.visibility_multiplier,
        );
    }
    if spatial.has_sensor_targets(agent.role, AgentRole::Predator) {
        fill_role_eye_cells(
            agent,
            world,
            spatial,
            &mut values[predator_start..predator_start + MAX_VISION_CELLS],
            AgentRole::Predator,
            day_night.visibility_multiplier,
        );
    }
    apply_sensor_corruption_in_place(agent.id, world.step_count, &mut values, config);
    values
}

fn compute_sensor_values_single_agent(
    agent: &AgentState,
    world: &WorldState,
    config: &SimulationConfig,
) -> [f32; CONTROLLER_SENSOR_INPUT_SIZE] {
    let day_night = current_day_night_state(world.step_count, config);
    let mut values = [0.0; CONTROLLER_SENSOR_INPUT_SIZE];
    values[0] = (agent.energy / config.initial_agent_energy).clamp(0.0, 2.0);
    values[1] = (agent.velocity.length() / agent.max_speed.max(0.001)).clamp(0.0, 1.0);

    let food_start = CORE_SENSOR_FEATURES;
    let prey_start = CORE_SENSOR_FEATURES + MAX_VISION_CELLS;
    fill_food_eye_cells_direct(
        agent,
        world,
        &mut values[food_start..prey_start],
        day_night.visibility_multiplier,
    );
    apply_sensor_corruption_in_place(agent.id, world.step_count, &mut values, config);
    values
}

/// Build deterministic aggregate metrics from world state.
///
/// Inputs: immutable world state.
/// Output: episode summary.
/// Side effects: none.
pub fn summarize_world(world: &WorldState) -> EpisodeSummary {
    EpisodeSummary {
        step_count: world.step_count,
        alive_agents: world.agents.iter().filter(|a| a.alive).count(),
        consumed_food: world.agents.iter().map(|a| a.food_consumed).sum(),
        remaining_food: world.food.len(),
        total_energy: world.agents.iter().map(|a| a.energy).sum(),
    }
}

/// Wrap position coordinates into toroidal world bounds.
pub fn wrap_position(position: Vec2, width: f32, height: f32) -> Vec2 {
    Vec2 {
        x: position.x.rem_euclid(width),
        y: position.y.rem_euclid(height),
    }
}

/// Compute shortest Euclidean distance in a toroidal world.
pub fn toroidal_distance(a: Vec2, b: Vec2, width: f32, height: f32) -> f32 {
    let dx = (a.x - b.x).abs();
    let dy = (a.y - b.y).abs();

    let wrapped_dx = dx.min(width - dx);
    let wrapped_dy = dy.min(height - dy);

    (wrapped_dx * wrapped_dx + wrapped_dy * wrapped_dy).sqrt()
}

fn stub_action(agent_id: u32, step_count: u64) -> ActionVector {
    let phase = (step_count as f32 * 0.037) + (agent_id as f32 * 0.173);
    ActionVector {
        steering: phase.sin(),
        throttle: 0.8,
    }
}

fn maybe_respawn_food(
    world: &mut WorldState,
    config: &SimulationConfig,
    agent_idx: usize,
    consumed_food: &FoodState,
    next_food_id: &mut u32,
) {
    if config.food_respawn_probability <= 0.0 {
        return;
    }

    let mut rng = ChaCha8Rng::seed_from_u64(
        world.step_count
            ^ ((agent_idx as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15))
            ^ ((consumed_food.id as u64 + 1).wrapping_mul(0x517C_C1B7_2722_0A95)),
    );

    if !rng.gen_bool(config.food_respawn_probability as f64) {
        return;
    }

    spawn_food_with_rng(world, config, next_food_id, &mut rng);
}

fn spawn_food_at_fixed_rate(
    world: &mut WorldState,
    config: &SimulationConfig,
    next_food_id: &mut u32,
) {
    let spawn_count = fixed_food_spawn_count_for_step(world, config);
    if spawn_count == 0 {
        return;
    }

    for spawn_index in 0..spawn_count {
        let mut rng = ChaCha8Rng::seed_from_u64(
            world.world_seed
                ^ ((world.step_count + 1).wrapping_mul(0xD6E8_FEB8_6659_FD93))
                ^ ((spawn_index as u64 + 1).wrapping_mul(0xA24B_AED4_0FBF_4C3D)),
        );
        spawn_food_with_rng(world, config, next_food_id, &mut rng);
    }
}

fn fixed_food_spawn_count_for_step(world: &WorldState, config: &SimulationConfig) -> u32 {
    if config.food_spawn_rate_per_second <= 0.0 {
        return 0;
    }

    let rate_per_second = config.food_spawn_rate_per_second as f64;
    let dt = config.timestep_seconds as f64;
    let completed_before_step = (world.step_count as f64 * dt * rate_per_second).floor() as u64;
    let completed_after_step =
        ((world.step_count as f64 + 1.0) * dt * rate_per_second).floor() as u64;

    completed_after_step
        .saturating_sub(completed_before_step)
        .min(u32::MAX as u64) as u32
}

fn spawn_food_with_rng(
    world: &mut WorldState,
    config: &SimulationConfig,
    next_food_id: &mut u32,
    rng: &mut ChaCha8Rng,
) {
    let cluster_id = assign_food_cluster(world, config, *next_food_id, rng);
    let mut food = FoodState {
        id: *next_food_id,
        position: Vec2::new(
            rng.gen_range(0.0..world.width),
            rng.gen_range(0.0..world.height),
        ),
        radius: config.food_collision_radius,
        energy_value: config.food_energy_gain,
        motion_angle_radians: rng.gen_range(0.0..std::f32::consts::TAU),
        cluster_id,
        cluster_angle_radians: rng.gen_range(0.0..std::f32::consts::TAU),
        cluster_distance_factor: rng.gen_range(0.0..1.0),
    };
    layout_food_position(
        &mut food,
        &world.food_clusters,
        world.width,
        world.height,
        world.step_count,
        config,
    );
    world.food.push(food);
    *next_food_id = (*next_food_id).saturating_add(1);
}

fn move_food(world: &mut WorldState, config: &SimulationConfig, dt: f32) {
    if config.food_cluster_enabled {
        return;
    }

    if config.food_motion_speed <= 0.0 {
        return;
    }

    for food in &mut world.food {
        let velocity = Vec2::from_angle(food.motion_angle_radians).scale(config.food_motion_speed);
        food.position = wrap_position(
            food.position.add(velocity.scale(dt)),
            world.width,
            world.height,
        );
    }
}

fn move_food_clusters(world: &mut WorldState, config: &SimulationConfig, dt: f32) {
    if !config.food_cluster_enabled || config.food_cluster_motion_speed <= 0.0 {
        return;
    }

    for cluster in &mut world.food_clusters {
        let velocity =
            Vec2::from_angle(cluster.motion_angle_radians).scale(config.food_cluster_motion_speed);
        cluster.position = wrap_position(
            cluster.position.add(velocity.scale(dt)),
            world.width,
            world.height,
        );
    }
}

fn apply_cluster_food_layout(world: &mut WorldState, config: &SimulationConfig) {
    if !config.food_cluster_enabled || world.food_clusters.is_empty() {
        return;
    }

    for food in &mut world.food {
        layout_food_position(
            food,
            &world.food_clusters,
            world.width,
            world.height,
            world.step_count,
            config,
        );
    }
}

fn layout_food_position(
    food: &mut FoodState,
    clusters: &[FoodClusterState],
    width: f32,
    height: f32,
    step_count: u64,
    config: &SimulationConfig,
) {
    let Some(cluster_id) = food.cluster_id else {
        return;
    };
    let Some(cluster) = clusters.iter().find(|cluster| cluster.id == cluster_id) else {
        return;
    };
    let local_radius = config.food_cluster_radius
        * cluster_density_radius_scale(cluster, step_count, config)
        * food.cluster_distance_factor.sqrt();
    let local_offset = Vec2::from_angle(food.cluster_angle_radians).scale(local_radius);
    food.position = wrap_position(cluster.position.add(local_offset), width, height);
}

pub fn cluster_productivity(
    cluster: &FoodClusterState,
    step_count: u64,
    config: &SimulationConfig,
) -> f32 {
    if !config.food_cluster_enabled || world_is_cluster_phase_static(config) {
        return 1.0;
    }

    let day_night = current_day_night_state(step_count, config);
    let shifted_phase = (day_night.phase + cluster.phase_offset).fract();
    let oscillation = ((shifted_phase * std::f32::consts::TAU).cos() * 0.5 + 0.5).clamp(0.0, 1.0);
    (1.0 - config.food_cluster_productivity_bias)
        + config.food_cluster_productivity_bias * oscillation
}

pub fn cluster_density_radius_scale(
    cluster: &FoodClusterState,
    step_count: u64,
    config: &SimulationConfig,
) -> f32 {
    let productivity = cluster_productivity(cluster, step_count, config);
    1.25 - 0.9 * productivity
}

fn assign_food_cluster(
    world: &WorldState,
    config: &SimulationConfig,
    food_id: u32,
    rng: &mut ChaCha8Rng,
) -> Option<u32> {
    if !config.food_cluster_enabled || world.food_clusters.is_empty() {
        return None;
    }

    let total_weight = world
        .food_clusters
        .iter()
        .map(|cluster| cluster_productivity(cluster, world.step_count, config))
        .sum::<f32>()
        .max(1e-6);
    let mut draw = rng.gen_range(0.0..total_weight);
    for cluster in &world.food_clusters {
        draw -= cluster_productivity(cluster, world.step_count, config);
        if draw <= 0.0 {
            return Some(cluster.id);
        }
    }

    Some(world.food_clusters[food_id as usize % world.food_clusters.len()].id)
}

fn world_is_cluster_phase_static(config: &SimulationConfig) -> bool {
    !config.day_night_cycle_enabled || config.food_cluster_productivity_bias <= 0.0
}

fn fill_food_eye_cells(
    agent: &AgentState,
    world: &WorldState,
    spatial: &SpatialIndex,
    cells: &mut [f32],
    visibility_multiplier: f32,
) {
    cells.fill(0.0);
    spatial.with_candidate_foods(agent.position, agent.perception_radius, |food_idx| {
        accumulate_eye_target(
            agent,
            cells,
            world.food[food_idx].position,
            world.width,
            world.height,
            visibility_multiplier,
        );
    });
}

fn fill_food_eye_cells_direct(
    agent: &AgentState,
    world: &WorldState,
    cells: &mut [f32],
    visibility_multiplier: f32,
) {
    cells.fill(0.0);
    for food in &world.food {
        accumulate_eye_target(
            agent,
            cells,
            food.position,
            world.width,
            world.height,
            visibility_multiplier,
        );
    }
}

fn fill_role_eye_cells(
    agent: &AgentState,
    world: &WorldState,
    spatial: &SpatialIndex,
    cells: &mut [f32],
    target_role: AgentRole,
    visibility_multiplier: f32,
) {
    cells.fill(0.0);
    spatial.with_candidate_agents(agent.position, agent.perception_radius, |other_idx| {
        let other = &world.agents[other_idx];
        if other.id == agent.id || !other.alive || other.role != target_role {
            return;
        }

        accumulate_eye_target(
            agent,
            cells,
            other.position,
            world.width,
            world.height,
            visibility_multiplier,
        );
    });
}

fn accumulate_eye_target(
    agent: &AgentState,
    cells: &mut [f32],
    target_position: Vec2,
    world_width: f32,
    world_height: f32,
    visibility_multiplier: f32,
) {
    let active_cells = agent.perception_channels.clamp(1, MAX_VISION_CELLS as u16) as usize;
    let fov = (agent.eye_cell_angle_radians.max(0.001) * active_cells as f32)
        .clamp(0.001, std::f32::consts::TAU);
    let half_fov = fov * 0.5;
    let visibility = visibility_multiplier.clamp(0.0, 1.0);
    let max_range = (agent.perception_radius * visibility).max(0.001);

    let offset = toroidal_offset(agent.position, target_position, world_width, world_height);
    let distance = offset.length();

    if distance > max_range {
        return;
    }

    let heading_to_target = offset.y.atan2(offset.x);
    let relative_angle = wrap_signed_angle(heading_to_target - agent.heading_radians);
    if relative_angle < -half_fov || relative_angle > half_fov {
        return;
    }

    let normalized = (relative_angle + half_fov) / fov;
    let cell = ((normalized * active_cells as f32).floor() as usize).min(active_cells - 1);
    cells[cell] += visibility * (max_range - distance) / max_range;
}

fn apply_sensor_corruption_in_place(
    agent_id: u32,
    step_count: u64,
    values: &mut [f32; CONTROLLER_SENSOR_INPUT_SIZE],
    config: &SimulationConfig,
) {
    if config.sensor_noise_stddev <= 0.0 && config.sensor_dropout_probability <= 0.0 {
        return;
    }

    let mut rng = ChaCha8Rng::seed_from_u64(
        step_count
            ^ ((agent_id as u64 + 1).wrapping_mul(0xD6E8_FEB8_6659_FD93))
            ^ 0xA24B_AED4_0FBF_4C3D,
    );

    for (idx, value) in values.iter_mut().enumerate() {
        if config.sensor_dropout_probability > 0.0
            && rng.gen_bool(config.sensor_dropout_probability as f64)
        {
            *value = 0.0;
            continue;
        }

        if config.sensor_noise_stddev > 0.0 {
            *value = clamp_sensor_value(
                idx,
                *value + gaussian_sample(&mut rng) * config.sensor_noise_stddev,
            );
        } else {
            *value = clamp_sensor_value(idx, *value);
        }
    }
}

fn gaussian_sample(rng: &mut ChaCha8Rng) -> f32 {
    let u1 = (1.0_f32 - rng.gen_range(0.0..1.0)).max(f32::MIN_POSITIVE);
    let u2 = rng.gen_range(0.0..1.0);
    (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos()
}

fn clamp_sensor_value(index: usize, value: f32) -> f32 {
    match index {
        0 => value.clamp(0.0, 2.0),
        1 => value.clamp(0.0, 1.0),
        _ => value.max(0.0),
    }
}

fn toroidal_offset(from: Vec2, to: Vec2, width: f32, height: f32) -> Vec2 {
    let mut dx = to.x - from.x;
    let mut dy = to.y - from.y;

    let half_width = width * 0.5;
    if dx > half_width {
        dx -= width;
    } else if dx < -half_width {
        dx += width;
    }

    let half_height = height * 0.5;
    if dy > half_height {
        dy -= height;
    } else if dy < -half_height {
        dy += height;
    }

    Vec2::new(dx, dy)
}

fn wrap_cell_index(index: isize, max: usize) -> usize {
    index.rem_euclid(max as isize) as usize
}

fn wrap_signed_angle(angle: f32) -> f32 {
    (angle + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

fn apply_kinematics(
    agent: &mut AgentState,
    action: ActionVector,
    width: f32,
    height: f32,
    config: &SimulationConfig,
    dt: f32,
) -> f32 {
    let steering = action.steering.clamp(-1.0, 1.0);
    let throttle = action.throttle.clamp(0.0, 1.0);

    agent.heading_radians = (agent.heading_radians + steering * config.max_turn_rate_radians * dt)
        .rem_euclid(std::f32::consts::TAU);

    let forward_acceleration =
        Vec2::from_angle(agent.heading_radians).scale(throttle * config.max_acceleration);
    agent.velocity = agent.velocity.add(forward_acceleration.scale(dt));

    // Linear drag keeps momentum persistent while still letting speed decay when throttle drops.
    let drag_scale = (1.0 - config.linear_drag * dt).clamp(0.0, 1.0);
    agent.velocity = agent.velocity.scale(drag_scale);

    let speed = agent.velocity.length();
    if speed > agent.max_speed {
        agent.velocity = agent.velocity.scale(agent.max_speed / speed);
    }

    let speed = agent.velocity.length();
    agent.position = wrap_position(agent.position.add(agent.velocity.scale(dt)), width, height);

    speed
}

/// Compute the exact per-term energy accounting for one agent step.
///
/// Inputs: current agent state before the step, config, realized speed for this step, step
/// duration, and food energy gained during the same step.
/// Output: per-term energy losses plus food gain and net delta.
/// Side effects: none.
pub fn energy_breakdown_for_step(
    agent: &AgentState,
    config: &SimulationConfig,
    speed: f32,
    dt: f32,
    food_gain: f32,
    capture_gain: f32,
) -> EnergyBreakdown {
    let metabolic_scale = 1.0 / agent.metabolic_efficiency.max(0.001);

    let base_loss = config.base_energy_cost_per_second * dt * metabolic_scale;
    let speed_loss = if config.rule_toggles.energy_by_speed {
        config.speed_energy_cost_factor * speed * dt
    } else {
        0.0
    };
    let network_loss = if config.rule_toggles.energy_by_network_size {
        config.network_energy_cost_factor * agent.network_size as f32 * dt
    } else {
        0.0
    };
    let perception_loss = if config.rule_toggles.energy_by_perception_size {
        config.perception_energy_cost_factor
            * agent.perception_radius
            * agent.perception_channels as f32
            * dt
    } else {
        0.0
    };
    let total_loss = base_loss + speed_loss + network_loss + perception_loss;

    EnergyBreakdown {
        base_loss,
        speed_loss,
        network_loss,
        perception_loss,
        food_gain,
        capture_gain,
        total_loss,
        net_delta: food_gain + capture_gain - total_loss,
    }
}

fn resolve_predator_captures(
    world: &mut WorldState,
    spatial: &SpatialIndex,
    config: &SimulationConfig,
    outcome: &mut StepOutcome,
) -> Vec<f32> {
    if config.predator_capture_radius <= 0.0 || config.predator_capture_energy_gain <= 0.0 {
        return vec![0.0_f32; world.agents.len()];
    }
    if !spatial.has_live_predator_and_prey() {
        return vec![0.0_f32; world.agents.len()];
    }

    let mut capture_gains = vec![0.0_f32; world.agents.len()];
    let mut captured_prey = vec![false; world.agents.len()];
    for predator_idx in 0..world.agents.len() {
        let predator = &world.agents[predator_idx];
        if !predator.alive || predator.role != AgentRole::Predator {
            continue;
        }

        let mut target: Option<(usize, f32, u32)> = None;
        spatial.with_candidate_agents(
            predator.position,
            config.predator_capture_radius,
            |prey_idx| {
                let prey = &world.agents[prey_idx];
                if prey_idx == predator_idx
                    || !prey.alive
                    || prey.role != AgentRole::Prey
                    || captured_prey[prey_idx]
                {
                    return;
                }

                let distance =
                    toroidal_distance(predator.position, prey.position, world.width, world.height);
                if distance > config.predator_capture_radius {
                    return;
                }

                let should_replace = match target {
                    None => true,
                    Some((_, best_distance, best_id)) => {
                        distance < best_distance || (distance == best_distance && prey.id < best_id)
                    }
                };

                if should_replace {
                    target = Some((prey_idx, distance, prey.id));
                }
            },
        );

        if let Some((prey_idx, _, _)) = target {
            captured_prey[prey_idx] = true;
            capture_gains[predator_idx] += config.predator_capture_energy_gain;
            world.agents[predator_idx].prey_captured += 1;
            outcome.prey_captures += 1;
        }
    }

    for (prey_idx, was_captured) in captured_prey.into_iter().enumerate() {
        if !was_captured {
            continue;
        }

        world.agents[prey_idx].alive = false;
        world.agents[prey_idx].energy = 0.0;
        outcome.deaths += 1;
    }

    capture_gains
}

fn find_collided_food_index(
    food: &[FoodState],
    spatial: &SpatialIndex,
    consumed_food_flags: &[bool],
    position: Vec2,
    width: f32,
    height: f32,
) -> Option<usize> {
    let mut match_idx = None;
    spatial.with_candidate_foods(position, spatial.max_food_radius.max(0.001), |food_idx| {
        if consumed_food_flags.get(food_idx).copied().unwrap_or(false) {
            return;
        }

        let item = &food[food_idx];
        if toroidal_distance(position, item.position, width, height) > item.radius {
            return;
        }

        if match_idx.is_none_or(|best_idx| food_idx < best_idx) {
            match_idx = Some(food_idx);
        }
    });

    match_idx
}

fn find_collided_food_index_direct(
    food: &[FoodState],
    consumed_food_flags: &[bool],
    position: Vec2,
    width: f32,
    height: f32,
) -> Option<usize> {
    food.iter()
        .enumerate()
        .filter(|(food_idx, _)| !consumed_food_flags.get(*food_idx).copied().unwrap_or(false))
        .filter(|(_, item)| {
            toroidal_distance(position, item.position, width, height) <= item.radius
        })
        .map(|(food_idx, _)| food_idx)
        .min()
}

#[cfg(test)]
mod tests {
    use crate::config::{RuleToggles, SimulationConfig};

    use super::{
        ActionVector, AgentRole, AgentState, CORE_SENSOR_FEATURES, FoodClusterState, FoodState,
        MAX_VISION_CELLS, SpatialIndex, Vec2, WorldState, apply_cluster_food_layout,
        basic_sensor_vector, basic_sensor_vector_with_index, cluster_density_radius_scale,
        current_day_night_state, energy_breakdown_for_step, fill_population_sensor_vectors,
        find_collided_food_index, find_collided_food_index_direct, run_steps, sensor_observation,
        step_world, step_world_with_actions, step_world_with_actions_and_telemetry,
        toroidal_distance, wrap_position,
    };

    fn test_agent(energy: f32) -> AgentState {
        AgentState {
            id: 0,
            position: Vec2::new(9.8, 9.8),
            heading_radians: 0.0,
            velocity: Vec2::default(),
            energy,
            alive: true,
            max_speed: 5.0,
            perception_radius: 10.0,
            perception_channels: 4,
            eye_cell_angle_radians: std::f32::consts::FRAC_PI_4,
            network_size: 16,
            metabolic_efficiency: 1.0,
            role: AgentRole::Prey,
            food_consumed: 0,
            prey_captured: 0,
        }
    }

    fn deterministic_sensor_config() -> SimulationConfig {
        SimulationConfig {
            food_motion_speed: 0.0,
            sensor_noise_stddev: 0.0,
            sensor_dropout_probability: 0.0,
            day_night_cycle_enabled: false,
            ..SimulationConfig::default()
        }
    }

    fn test_food(id: u32, position: Vec2) -> FoodState {
        FoodState {
            id,
            position,
            radius: 1.0,
            energy_value: 1.0,
            motion_angle_radians: 0.0,
            cluster_id: None,
            cluster_angle_radians: 0.0,
            cluster_distance_factor: 0.0,
        }
    }

    #[test]
    fn toroidal_wrapping_wraps_coordinates() {
        let wrapped = wrap_position(Vec2::new(10.25, -0.75), 10.0, 10.0);

        assert!((wrapped.x - 0.25).abs() < 1e-6);
        assert!((wrapped.y - 9.25).abs() < 1e-6);
    }

    #[test]
    fn collision_consumes_food_and_adds_energy() {
        let config = SimulationConfig {
            timestep_seconds: 0.1,
            base_energy_cost_per_second: 0.0,
            speed_energy_cost_factor: 0.0,
            network_energy_cost_factor: 0.0,
            perception_energy_cost_factor: 0.0,
            food_respawn_probability: 0.0,
            ..SimulationConfig::default()
        };

        let mut world = WorldState {
            width: 20.0,
            height: 20.0,
            agents: vec![test_agent(5.0)],
            food: vec![FoodState {
                id: 0,
                position: Vec2::new(10.0, 10.0),
                radius: 5.0,
                energy_value: 3.0,
                motion_angle_radians: 0.0,
                cluster_id: None,
                cluster_angle_radians: 0.0,
                cluster_distance_factor: 0.0,
            }],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 0,
        };

        let outcome = step_world(&mut world, &config);

        assert_eq!(outcome.consumed_food, 1);
        assert_eq!(world.food.len(), 0);
        assert_eq!(world.agents[0].food_consumed, 1);
        assert!((world.agents[0].energy - 8.0).abs() < 1e-6);
    }

    #[test]
    fn simultaneous_food_collision_only_rewards_one_agent() {
        let config = SimulationConfig {
            timestep_seconds: 0.1,
            base_energy_cost_per_second: 0.0,
            speed_energy_cost_factor: 0.0,
            network_energy_cost_factor: 0.0,
            perception_energy_cost_factor: 0.0,
            food_respawn_probability: 0.0,
            ..SimulationConfig::default()
        };

        let mut first = test_agent(5.0);
        first.position = Vec2::new(10.0, 10.0);
        let mut second = test_agent(5.0);
        second.id = 1;
        second.position = Vec2::new(10.0, 10.0);

        let mut world = WorldState {
            width: 20.0,
            height: 20.0,
            agents: vec![first, second],
            food: vec![FoodState {
                id: 0,
                position: Vec2::new(10.0, 10.0),
                radius: 5.0,
                energy_value: 3.0,
                motion_angle_radians: 0.0,
                cluster_id: None,
                cluster_angle_radians: 0.0,
                cluster_distance_factor: 0.0,
            }],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 0,
        };

        let outcome = step_world_with_actions(
            &mut world,
            &config,
            &[ActionVector::default(), ActionVector::default()],
        );

        assert_eq!(outcome.consumed_food, 1);
        assert_eq!(world.food.len(), 0);
        assert_eq!(world.agents[0].food_consumed, 1);
        assert_eq!(world.agents[1].food_consumed, 0);
    }

    #[test]
    fn predator_capture_kills_prey_and_grants_energy() {
        let config = SimulationConfig {
            timestep_seconds: 0.1,
            food_count: 1,
            food_respawn_probability: 0.0,
            base_energy_cost_per_second: 0.0,
            speed_energy_cost_factor: 0.0,
            network_energy_cost_factor: 0.0,
            perception_energy_cost_factor: 0.0,
            predator_capture_radius: 2.0,
            predator_capture_energy_gain: 7.5,
            ..SimulationConfig::default()
        };

        let mut predator = test_agent(5.0);
        predator.role = AgentRole::Predator;
        predator.position = Vec2::new(10.0, 10.0);
        let mut prey = test_agent(5.0);
        prey.id = 1;
        prey.position = Vec2::new(10.5, 10.0);

        let mut world = WorldState {
            width: 20.0,
            height: 20.0,
            agents: vec![predator, prey],
            food: vec![FoodState {
                id: 0,
                position: Vec2::new(10.0, 10.0),
                radius: 5.0,
                energy_value: 3.0,
                motion_angle_radians: 0.0,
                cluster_id: None,
                cluster_angle_radians: 0.0,
                cluster_distance_factor: 0.0,
            }],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 0,
        };

        let outcome = step_world_with_actions(
            &mut world,
            &config,
            &[ActionVector::default(), ActionVector::default()],
        );

        assert_eq!(outcome.prey_captures, 1);
        assert_eq!(outcome.deaths, 1);
        assert_eq!(world.agents[0].prey_captured, 1);
        assert!(world.agents[1].energy.abs() < 1e-6);
        assert!(!world.agents[1].alive);
        assert!((world.agents[0].energy - 12.5).abs() < 1e-6);
        assert_eq!(world.agents[0].food_consumed, 0);
        assert_eq!(world.food.len(), 1);
    }

    #[test]
    fn energy_terms_obey_toggle_flags() {
        let base_config = SimulationConfig {
            timestep_seconds: 0.1,
            base_energy_cost_per_second: 1.0,
            speed_energy_cost_factor: 1.0,
            network_energy_cost_factor: 1.0,
            perception_energy_cost_factor: 1.0,
            ..SimulationConfig::default()
        };

        let mut off_world = WorldState {
            width: 20.0,
            height: 20.0,
            agents: vec![test_agent(100.0)],
            food: vec![],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 0,
        };

        let mut speed_on_world = off_world.clone();
        let mut network_on_world = off_world.clone();
        let mut perception_on_world = off_world.clone();

        let off_config = SimulationConfig {
            rule_toggles: RuleToggles {
                energy_by_speed: false,
                energy_by_network_size: false,
                energy_by_perception_size: false,
            },
            ..base_config.clone()
        };

        let speed_config = SimulationConfig {
            rule_toggles: RuleToggles {
                energy_by_speed: true,
                energy_by_network_size: false,
                energy_by_perception_size: false,
            },
            ..base_config.clone()
        };

        let network_config = SimulationConfig {
            rule_toggles: RuleToggles {
                energy_by_speed: false,
                energy_by_network_size: true,
                energy_by_perception_size: false,
            },
            ..base_config.clone()
        };

        let perception_config = SimulationConfig {
            rule_toggles: RuleToggles {
                energy_by_speed: false,
                energy_by_network_size: false,
                energy_by_perception_size: true,
            },
            ..base_config
        };

        step_world(&mut off_world, &off_config);
        step_world(&mut speed_on_world, &speed_config);
        step_world(&mut network_on_world, &network_config);
        step_world(&mut perception_on_world, &perception_config);

        let off_energy = off_world.agents[0].energy;
        assert!(speed_on_world.agents[0].energy < off_energy);
        assert!(network_on_world.agents[0].energy < off_energy);
        assert!(perception_on_world.agents[0].energy < off_energy);
    }

    #[test]
    fn energy_breakdown_reports_each_enabled_term() {
        let config = SimulationConfig {
            timestep_seconds: 0.5,
            base_energy_cost_per_second: 2.0,
            speed_energy_cost_factor: 3.0,
            network_energy_cost_factor: 0.1,
            perception_energy_cost_factor: 0.2,
            rule_toggles: RuleToggles {
                energy_by_speed: true,
                energy_by_network_size: true,
                energy_by_perception_size: true,
            },
            ..SimulationConfig::default()
        };
        let agent = test_agent(10.0);

        let breakdown =
            energy_breakdown_for_step(&agent, &config, 4.0, config.timestep_seconds, 1.5, 0.0);

        assert!((breakdown.base_loss - 1.0).abs() < 1e-6);
        assert!((breakdown.speed_loss - 6.0).abs() < 1e-6);
        assert!((breakdown.network_loss - 0.8).abs() < 1e-6);
        assert!((breakdown.perception_loss - 4.0).abs() < 1e-6);
        assert!((breakdown.food_gain - 1.5).abs() < 1e-6);
        assert!(breakdown.capture_gain.abs() < 1e-6);
        assert!((breakdown.total_loss - 11.8).abs() < 1e-6);
        assert!((breakdown.net_delta + 10.3).abs() < 1e-6);
    }

    #[test]
    fn step_world_with_actions_and_telemetry_reports_food_and_survival() {
        let config = SimulationConfig {
            timestep_seconds: 0.1,
            base_energy_cost_per_second: 0.0,
            speed_energy_cost_factor: 0.0,
            network_energy_cost_factor: 0.0,
            perception_energy_cost_factor: 0.0,
            food_respawn_probability: 0.0,
            ..SimulationConfig::default()
        };
        let mut world = WorldState {
            width: 20.0,
            height: 20.0,
            agents: vec![test_agent(5.0)],
            food: vec![FoodState {
                id: 0,
                position: Vec2::new(10.0, 10.0),
                radius: 5.0,
                energy_value: 3.0,
                motion_angle_radians: 0.0,
                cluster_id: None,
                cluster_angle_radians: 0.0,
                cluster_distance_factor: 0.0,
            }],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 0,
        };

        let (_outcome, telemetry) = step_world_with_actions_and_telemetry(&mut world, &config, &[]);

        assert_eq!(telemetry.len(), 1);
        assert!(telemetry[0].consumed_food);
        assert!(telemetry[0].alive_after);
        assert!((telemetry[0].energy.food_gain - 3.0).abs() < 1e-6);
    }

    #[test]
    fn consumed_food_respawns_when_probability_is_one() {
        let config = SimulationConfig {
            timestep_seconds: 0.1,
            base_energy_cost_per_second: 0.0,
            speed_energy_cost_factor: 0.0,
            network_energy_cost_factor: 0.0,
            perception_energy_cost_factor: 0.0,
            food_respawn_probability: 1.0,
            food_motion_speed: 0.0,
            ..SimulationConfig::default()
        };
        let mut world = WorldState {
            width: 20.0,
            height: 20.0,
            agents: vec![test_agent(5.0)],
            food: vec![FoodState {
                id: 7,
                position: Vec2::new(10.0, 10.0),
                radius: 5.0,
                energy_value: 3.0,
                motion_angle_radians: 0.0,
                cluster_id: None,
                cluster_angle_radians: 0.0,
                cluster_distance_factor: 0.0,
            }],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 0,
        };

        let outcome = step_world(&mut world, &config);

        assert_eq!(outcome.consumed_food, 1);
        assert_eq!(world.food.len(), 1);
        assert_ne!(world.food[0].id, 7);
        assert!(world.food[0].position.x >= 0.0 && world.food[0].position.x < world.width);
        assert!(world.food[0].position.y >= 0.0 && world.food[0].position.y < world.height);
    }

    #[test]
    fn fixed_rate_food_spawning_adds_food_without_consumption() {
        let config = SimulationConfig {
            timestep_seconds: 0.25,
            food_spawn_rate_per_second: 2.0,
            food_respawn_probability: 0.0,
            food_motion_speed: 0.0,
            base_energy_cost_per_second: 0.0,
            speed_energy_cost_factor: 0.0,
            network_energy_cost_factor: 0.0,
            perception_energy_cost_factor: 0.0,
            ..SimulationConfig::default()
        };
        let mut world = WorldState {
            width: 20.0,
            height: 20.0,
            agents: vec![],
            food: vec![],
            food_clusters: vec![],
            world_seed: 17,
            step_count: 0,
        };

        step_world(&mut world, &config);
        assert!(world.food.is_empty());

        step_world(&mut world, &config);
        assert_eq!(world.food.len(), 1);
        assert_eq!(world.food[0].id, 1);

        step_world(&mut world, &config);
        assert_eq!(world.food.len(), 1);

        step_world(&mut world, &config);
        assert_eq!(world.food.len(), 2);
        assert_eq!(world.food[1].id, 2);
    }

    #[test]
    fn fixed_rate_food_spawning_is_deterministic_for_same_seed() {
        let config = SimulationConfig {
            timestep_seconds: 0.2,
            food_spawn_rate_per_second: 3.0,
            food_respawn_probability: 0.0,
            food_motion_speed: 0.0,
            ..SimulationConfig::default()
        };
        let mut world_a = WorldState {
            width: 20.0,
            height: 20.0,
            agents: vec![],
            food: vec![test_food(3, Vec2::new(2.0, 2.0))],
            food_clusters: vec![],
            world_seed: 91,
            step_count: 0,
        };
        let mut world_b = world_a.clone();

        run_steps(&mut world_a, &config, 8);
        run_steps(&mut world_b, &config, 8);

        assert_eq!(world_a.food, world_b.food);
        assert_eq!(world_a.step_count, world_b.step_count);
    }

    #[test]
    fn moving_food_advances_with_configured_speed() {
        let config = SimulationConfig {
            timestep_seconds: 0.5,
            food_motion_speed: 4.0,
            base_energy_cost_per_second: 0.0,
            speed_energy_cost_factor: 0.0,
            network_energy_cost_factor: 0.0,
            perception_energy_cost_factor: 0.0,
            ..SimulationConfig::default()
        };
        let mut world = WorldState {
            width: 20.0,
            height: 20.0,
            agents: vec![],
            food: vec![FoodState {
                id: 0,
                position: Vec2::new(5.0, 5.0),
                radius: 1.0,
                energy_value: 1.0,
                motion_angle_radians: 0.0,
                cluster_id: None,
                cluster_angle_radians: 0.0,
                cluster_distance_factor: 0.0,
            }],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 0,
        };

        step_world(&mut world, &config);

        assert!((world.food[0].position.x - 7.0).abs() < 1e-6);
        assert!((world.food[0].position.y - 5.0).abs() < 1e-6);
    }

    #[test]
    fn throttle_accelerates_and_drag_preserves_momentum() {
        let config = SimulationConfig {
            timestep_seconds: 0.5,
            max_acceleration: 10.0,
            linear_drag: 1.0,
            max_turn_rate_radians: 0.0,
            base_energy_cost_per_second: 0.0,
            speed_energy_cost_factor: 0.0,
            network_energy_cost_factor: 0.0,
            perception_energy_cost_factor: 0.0,
            ..SimulationConfig::default()
        };
        let mut world = WorldState {
            width: 50.0,
            height: 50.0,
            agents: vec![test_agent(10.0)],
            food: vec![],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 0,
        };

        let accelerate = [ActionVector {
            steering: 0.0,
            throttle: 1.0,
        }];
        let coast = [ActionVector::default()];

        step_world_with_actions(&mut world, &config, &accelerate);
        let speed_after_accel = world.agents[0].velocity.length();
        step_world_with_actions(&mut world, &config, &coast);
        let speed_after_coast = world.agents[0].velocity.length();

        assert!((speed_after_accel - 2.5).abs() < 1e-6);
        assert!((speed_after_coast - 1.25).abs() < 1e-6);
        assert!(speed_after_coast > 0.0);
    }

    #[test]
    fn acceleration_limit_prevents_instant_reverse() {
        let config = SimulationConfig {
            timestep_seconds: 1.0,
            max_acceleration: 2.0,
            linear_drag: 0.0,
            max_turn_rate_radians: std::f32::consts::PI,
            base_energy_cost_per_second: 0.0,
            speed_energy_cost_factor: 0.0,
            network_energy_cost_factor: 0.0,
            perception_energy_cost_factor: 0.0,
            ..SimulationConfig::default()
        };
        let mut world = WorldState {
            width: 50.0,
            height: 50.0,
            agents: vec![test_agent(10.0)],
            food: vec![],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 0,
        };

        step_world_with_actions(
            &mut world,
            &config,
            &[ActionVector {
                steering: 0.0,
                throttle: 1.0,
            }],
        );
        let forward_velocity = world.agents[0].velocity.x;

        step_world_with_actions(
            &mut world,
            &config,
            &[ActionVector {
                steering: 1.0,
                throttle: 1.0,
            }],
        );

        assert!(forward_velocity > 0.0);
        assert!(world.agents[0].velocity.x >= 0.0);
    }

    #[test]
    fn food_clusters_move_with_configured_speed() {
        let config = SimulationConfig {
            timestep_seconds: 0.5,
            food_motion_speed: 0.0,
            food_cluster_enabled: true,
            food_cluster_motion_speed: 4.0,
            food_cluster_radius: 10.0,
            ..SimulationConfig::default()
        };
        let mut world = WorldState {
            width: 20.0,
            height: 20.0,
            agents: vec![],
            food: vec![FoodState {
                cluster_id: Some(0),
                cluster_angle_radians: 0.0,
                cluster_distance_factor: 0.0,
                ..test_food(0, Vec2::new(5.0, 5.0))
            }],
            food_clusters: vec![FoodClusterState {
                id: 0,
                position: Vec2::new(5.0, 5.0),
                motion_angle_radians: 0.0,
                phase_offset: 0.0,
            }],
            world_seed: 0,
            step_count: 0,
        };

        step_world(&mut world, &config);

        assert!((world.food_clusters[0].position.x - 7.0).abs() < 1e-6);
        assert!((world.food_clusters[0].position.y - 5.0).abs() < 1e-6);
        assert!((world.food[0].position.x - 7.0).abs() < 1e-6);
    }

    #[test]
    fn cluster_density_changes_with_phase() {
        let config = SimulationConfig {
            food_cluster_enabled: true,
            food_cluster_radius: 20.0,
            food_cluster_productivity_bias: 1.0,
            day_night_cycle_enabled: true,
            day_night_cycle_steps: 240,
            food_motion_speed: 0.0,
            ..SimulationConfig::default()
        };
        let cluster = FoodClusterState {
            id: 0,
            position: Vec2::new(20.0, 20.0),
            motion_angle_radians: 0.0,
            phase_offset: 0.0,
        };
        let mut day_world = WorldState {
            width: 40.0,
            height: 40.0,
            agents: vec![],
            food: vec![
                FoodState {
                    cluster_id: Some(0),
                    cluster_angle_radians: 0.0,
                    cluster_distance_factor: 1.0,
                    ..test_food(0, cluster.position)
                },
                FoodState {
                    cluster_id: Some(0),
                    cluster_angle_radians: std::f32::consts::FRAC_PI_2,
                    cluster_distance_factor: 0.25,
                    ..test_food(1, cluster.position)
                },
            ],
            food_clusters: vec![cluster.clone()],
            world_seed: 0,
            step_count: 0,
        };
        apply_cluster_food_layout(&mut day_world, &config);
        let mut night_world = day_world.clone();
        night_world.step_count = 120;
        apply_cluster_food_layout(&mut night_world, &config);

        let mean_distance = |world: &WorldState| -> f32 {
            world
                .food
                .iter()
                .map(|food| {
                    toroidal_distance(
                        food.position,
                        world.food_clusters[0].position,
                        world.width,
                        world.height,
                    )
                })
                .sum::<f32>()
                / world.food.len() as f32
        };

        assert!(
            cluster_density_radius_scale(&cluster, 0, &config)
                < cluster_density_radius_scale(&cluster, 120, &config)
        );
        assert!(mean_distance(&day_world) < mean_distance(&night_world));
    }

    #[test]
    fn toroidal_distance_uses_shortest_wrapped_path() {
        let a = Vec2::new(9.0, 5.0);
        let b = Vec2::new(1.0, 5.0);

        let distance = toroidal_distance(a, b, 10.0, 10.0);

        assert!((distance - 2.0).abs() < 1e-6);
    }

    #[test]
    fn basic_sensor_vector_places_food_in_forward_center_cell() {
        let config = deterministic_sensor_config();
        let mut agent = test_agent(50.0);
        agent.position = Vec2::new(10.0, 10.0);
        agent.heading_radians = 0.0;
        agent.perception_radius = 20.0;
        agent.perception_channels = 5;
        agent.eye_cell_angle_radians = std::f32::consts::FRAC_PI_2 / 5.0;
        let world = WorldState {
            width: 40.0,
            height: 40.0,
            agents: vec![agent.clone()],
            food: vec![FoodState {
                id: 0,
                position: Vec2::new(14.0, 10.0),
                radius: 1.0,
                energy_value: 1.0,
                motion_angle_radians: 0.0,
                cluster_id: None,
                cluster_angle_radians: 0.0,
                cluster_distance_factor: 0.0,
            }],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 0,
        };

        let sensor = sensor_observation(&agent, &world, &config);
        let food_vision =
            &sensor.input.values[CORE_SENSOR_FEATURES..CORE_SENSOR_FEATURES + MAX_VISION_CELLS];
        let active = &food_vision[..agent.perception_channels as usize];

        assert!(active[2] > 0.0);
        assert!(sensor.food_eye_bins[2] > 0.0);
        assert!(sensor.prey_eye_bins.iter().all(|value| value.abs() < 1e-6));
        assert!(
            sensor
                .predator_eye_bins
                .iter()
                .all(|value| value.abs() < 1e-6)
        );
        assert!(
            sensor.input.values[CORE_SENSOR_FEATURES + MAX_VISION_CELLS..]
                .iter()
                .all(|value| value.abs() < 1e-6)
        );
        assert!(
            active
                .iter()
                .enumerate()
                .all(|(idx, value)| idx == 2 || value.abs() < 1e-6)
        );
    }

    #[test]
    fn basic_sensor_vector_ignores_food_outside_fov() {
        let config = deterministic_sensor_config();
        let mut agent = test_agent(50.0);
        agent.position = Vec2::new(10.0, 10.0);
        agent.heading_radians = 0.0;
        agent.perception_radius = 20.0;
        agent.perception_channels = 5;
        agent.eye_cell_angle_radians = std::f32::consts::FRAC_PI_2 / 5.0;
        let world = WorldState {
            width: 40.0,
            height: 40.0,
            agents: vec![agent.clone()],
            food: vec![FoodState {
                id: 0,
                position: Vec2::new(10.0, 16.0),
                radius: 1.0,
                energy_value: 1.0,
                motion_angle_radians: 0.0,
                cluster_id: None,
                cluster_angle_radians: 0.0,
                cluster_distance_factor: 0.0,
            }],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 0,
        };

        let sensor = basic_sensor_vector(&agent, &world, &config);
        let food_vision =
            &sensor.values[CORE_SENSOR_FEATURES..CORE_SENSOR_FEATURES + MAX_VISION_CELLS];
        let active = &food_vision[..agent.perception_channels as usize];

        assert!(active.iter().all(|value| value.abs() < 1e-6));
    }

    #[test]
    fn basic_sensor_vector_zero_fills_inactive_vision_cells() {
        let config = deterministic_sensor_config();
        let mut agent = test_agent(50.0);
        agent.position = Vec2::new(10.0, 10.0);
        agent.heading_radians = 0.0;
        agent.perception_radius = 20.0;
        agent.perception_channels = 3;
        agent.eye_cell_angle_radians = std::f32::consts::PI / 3.0;
        let world = WorldState {
            width: 40.0,
            height: 40.0,
            agents: vec![agent.clone()],
            food: vec![FoodState {
                id: 0,
                position: Vec2::new(15.0, 10.0),
                radius: 1.0,
                energy_value: 1.0,
                motion_angle_radians: 0.0,
                cluster_id: None,
                cluster_angle_radians: 0.0,
                cluster_distance_factor: 0.0,
            }],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 0,
        };

        let sensor = basic_sensor_vector(&agent, &world, &config);
        let vision = &sensor.values[CORE_SENSOR_FEATURES..CORE_SENSOR_FEATURES + MAX_VISION_CELLS];

        assert!(vision.iter().take(3).any(|value| *value > 0.0));
        assert!(vision.iter().skip(3).all(|value| value.abs() < 1e-6));
        assert!(
            sensor.values[CORE_SENSOR_FEATURES + MAX_VISION_CELLS..]
                .iter()
                .all(|value| value.abs() < 1e-6)
        );
    }

    #[test]
    fn eye_cell_angle_and_channel_count_expand_visible_fov() {
        let config = deterministic_sensor_config();
        let mut narrow = test_agent(50.0);
        narrow.position = Vec2::new(10.0, 10.0);
        narrow.heading_radians = 0.0;
        narrow.perception_radius = 20.0;
        narrow.perception_channels = 2;
        narrow.eye_cell_angle_radians = std::f32::consts::FRAC_PI_8;

        let mut wide = narrow.clone();
        wide.perception_channels = 6;
        wide.eye_cell_angle_radians = std::f32::consts::FRAC_PI_4;

        let world = WorldState {
            width: 40.0,
            height: 40.0,
            agents: vec![narrow.clone(), wide.clone()],
            food: vec![FoodState {
                id: 0,
                position: Vec2::new(10.0, 18.0),
                radius: 1.0,
                energy_value: 1.0,
                motion_angle_radians: 0.0,
                cluster_id: None,
                cluster_angle_radians: 0.0,
                cluster_distance_factor: 0.0,
            }],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 0,
        };

        let narrow_sensor = basic_sensor_vector(&narrow, &world, &config);
        let wide_sensor = basic_sensor_vector(&wide, &world, &config);

        assert!(
            narrow_sensor.values[CORE_SENSOR_FEATURES..CORE_SENSOR_FEATURES + MAX_VISION_CELLS]
                .iter()
                .all(|value| value.abs() < 1e-6)
        );
        assert!(
            wide_sensor.values[CORE_SENSOR_FEATURES..CORE_SENSOR_FEATURES + MAX_VISION_CELLS]
                .iter()
                .any(|value| *value > 0.0)
        );
    }

    #[test]
    fn sensor_observation_places_other_agent_in_forward_center_cell() {
        let config = deterministic_sensor_config();
        let mut agent = test_agent(50.0);
        agent.position = Vec2::new(10.0, 10.0);
        agent.heading_radians = 0.0;
        agent.perception_radius = 20.0;
        agent.perception_channels = 5;
        agent.eye_cell_angle_radians = std::f32::consts::FRAC_PI_2 / 5.0;

        let mut other_agent = test_agent(50.0);
        other_agent.id = 1;
        other_agent.position = Vec2::new(15.0, 10.0);
        other_agent.role = AgentRole::Predator;

        let world = WorldState {
            width: 40.0,
            height: 40.0,
            agents: vec![agent.clone(), other_agent],
            food: vec![],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 0,
        };

        let sensor = sensor_observation(&agent, &world, &config);

        assert!(sensor.predator_eye_bins[2] > 0.0);
        assert!(sensor.food_eye_bins.iter().all(|value| value.abs() < 1e-6));
        assert!(sensor.prey_eye_bins.iter().all(|value| value.abs() < 1e-6));
        assert!(sensor.eye_bins[MAX_VISION_CELLS * 2 + 2] > 0.0);
        assert!(
            sensor.eye_bins[..MAX_VISION_CELLS]
                .iter()
                .all(|value| value.abs() < 1e-6)
        );
    }

    #[test]
    fn sensor_observation_ignores_self_when_building_agent_bins() {
        let config = deterministic_sensor_config();
        let mut agent = test_agent(50.0);
        agent.position = Vec2::new(10.0, 10.0);
        agent.heading_radians = 0.0;
        agent.perception_radius = 20.0;
        agent.perception_channels = 5;
        agent.eye_cell_angle_radians = std::f32::consts::FRAC_PI_2 / 5.0;

        let world = WorldState {
            width: 40.0,
            height: 40.0,
            agents: vec![agent.clone()],
            food: vec![],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 0,
        };

        let sensor = sensor_observation(&agent, &world, &config);

        assert!(sensor.prey_eye_bins.iter().all(|value| value.abs() < 1e-6));
        assert!(
            sensor
                .predator_eye_bins
                .iter()
                .all(|value| value.abs() < 1e-6)
        );
        assert!(sensor.eye_bins.iter().all(|value| value.abs() < 1e-6));
    }

    #[test]
    fn basic_sensor_vector_matches_observation_input() {
        let config = deterministic_sensor_config();
        let mut agent = test_agent(50.0);
        agent.position = Vec2::new(10.0, 10.0);
        agent.heading_radians = 0.0;
        agent.perception_radius = 20.0;
        agent.perception_channels = 5;
        agent.eye_cell_angle_radians = std::f32::consts::FRAC_PI_2 / 5.0;
        let world = WorldState {
            width: 40.0,
            height: 40.0,
            agents: vec![agent.clone()],
            food: vec![FoodState {
                id: 0,
                position: Vec2::new(15.0, 10.0),
                radius: 1.0,
                energy_value: 1.0,
                motion_angle_radians: 0.0,
                cluster_id: None,
                cluster_angle_radians: 0.0,
                cluster_distance_factor: 0.0,
            }],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 0,
        };

        let fast = basic_sensor_vector(&agent, &world, &config);
        let observed = sensor_observation(&agent, &world, &config);

        assert_eq!(fast, observed.input);
    }

    #[test]
    fn single_agent_sensor_fast_path_matches_indexed_path() {
        let config = deterministic_sensor_config();
        let mut agent = test_agent(80.0);
        agent.position = Vec2::new(8.0, 7.0);
        agent.heading_radians = std::f32::consts::FRAC_PI_4;
        agent.perception_radius = 18.0;
        agent.perception_channels = 5;
        agent.eye_cell_angle_radians = std::f32::consts::FRAC_PI_2 / 5.0;
        let world = WorldState {
            width: 40.0,
            height: 40.0,
            agents: vec![agent.clone()],
            food: vec![
                FoodState {
                    id: 0,
                    position: Vec2::new(10.0, 9.0),
                    radius: 1.0,
                    energy_value: 1.0,
                    motion_angle_radians: 0.0,
                    cluster_id: None,
                    cluster_angle_radians: 0.0,
                    cluster_distance_factor: 0.0,
                },
                FoodState {
                    id: 1,
                    position: Vec2::new(15.0, 16.0),
                    radius: 1.0,
                    energy_value: 1.0,
                    motion_angle_radians: 0.0,
                    cluster_id: None,
                    cluster_angle_radians: 0.0,
                    cluster_distance_factor: 0.0,
                },
            ],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 3,
        };

        let fast = basic_sensor_vector(&agent, &world, &config);
        let spatial = SpatialIndex::build(&world);
        let indexed = basic_sensor_vector_with_index(&agent, &world, &spatial, &config);

        assert_eq!(fast, indexed);
    }

    #[test]
    fn single_agent_food_collision_fast_path_matches_spatial_lookup() {
        let food = vec![
            FoodState {
                id: 0,
                position: Vec2::new(25.0, 25.0),
                radius: 1.0,
                energy_value: 5.0,
                motion_angle_radians: 0.0,
                cluster_id: None,
                cluster_angle_radians: 0.0,
                cluster_distance_factor: 0.0,
            },
            FoodState {
                id: 1,
                position: Vec2::new(10.0, 10.0),
                radius: 1.0,
                energy_value: 5.0,
                motion_angle_radians: 0.0,
                cluster_id: None,
                cluster_angle_radians: 0.0,
                cluster_distance_factor: 0.0,
            },
        ];
        let world = WorldState {
            width: 30.0,
            height: 30.0,
            agents: vec![test_agent(50.0)],
            food: food.clone(),
            food_clusters: vec![],
            world_seed: 7,
            step_count: 0,
        };
        let consumed_food_flags = vec![false; food.len()];
        let position = Vec2::new(9.5, 9.7);

        let spatial = SpatialIndex::build(&world);
        let indexed = find_collided_food_index(
            &food,
            &spatial,
            &consumed_food_flags,
            position,
            world.width,
            world.height,
        );
        let direct = find_collided_food_index_direct(
            &food,
            &consumed_food_flags,
            position,
            world.width,
            world.height,
        );

        assert_eq!(direct, indexed);
    }

    #[test]
    fn fill_population_sensor_vectors_matches_individual_queries() {
        let config = deterministic_sensor_config();
        let mut first = test_agent(50.0);
        first.position = Vec2::new(10.0, 10.0);
        first.heading_radians = 0.0;
        let mut second = test_agent(45.0);
        second.id = 1;
        second.position = Vec2::new(20.0, 20.0);
        second.heading_radians = std::f32::consts::PI;
        let world = WorldState {
            width: 40.0,
            height: 40.0,
            agents: vec![first.clone(), second.clone()],
            food: vec![
                FoodState {
                    id: 0,
                    position: Vec2::new(15.0, 10.0),
                    radius: 1.0,
                    energy_value: 1.0,
                    motion_angle_radians: 0.0,
                    cluster_id: None,
                    cluster_angle_radians: 0.0,
                    cluster_distance_factor: 0.0,
                },
                FoodState {
                    id: 1,
                    position: Vec2::new(16.0, 20.0),
                    radius: 1.0,
                    energy_value: 1.0,
                    motion_angle_radians: 0.0,
                    cluster_id: None,
                    cluster_angle_radians: 0.0,
                    cluster_distance_factor: 0.0,
                },
            ],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 0,
        };

        let mut batch = vec![super::SensorVector::default(); world.agents.len()];
        fill_population_sensor_vectors(&world, &config, &mut batch);

        assert_eq!(batch[0], basic_sensor_vector(&first, &world, &config));
        assert_eq!(batch[1], basic_sensor_vector(&second, &world, &config));
    }

    #[test]
    fn current_day_night_state_reports_full_day_and_midnight_levels() {
        let config = SimulationConfig {
            day_night_cycle_enabled: true,
            day_night_cycle_steps: 240,
            night_visibility_multiplier: 0.25,
            ..deterministic_sensor_config()
        };

        let day = current_day_night_state(0, &config);
        let night = current_day_night_state(120, &config);

        assert!((day.visibility_multiplier - 1.0).abs() < 1e-6);
        assert!(!day.is_night);
        assert!((night.visibility_multiplier - 0.25).abs() < 1e-6);
        assert!(night.is_night);
    }

    #[test]
    fn night_visibility_reduces_sensor_strength() {
        let config = SimulationConfig {
            day_night_cycle_enabled: true,
            day_night_cycle_steps: 240,
            night_visibility_multiplier: 0.25,
            ..deterministic_sensor_config()
        };
        let mut agent = test_agent(50.0);
        agent.position = Vec2::new(10.0, 10.0);
        agent.heading_radians = 0.0;
        agent.perception_radius = 20.0;
        agent.perception_channels = 5;
        agent.eye_cell_angle_radians = std::f32::consts::FRAC_PI_2 / 5.0;

        let day_world = WorldState {
            width: 40.0,
            height: 40.0,
            agents: vec![agent.clone()],
            food: vec![FoodState {
                id: 0,
                position: Vec2::new(14.0, 10.0),
                radius: 1.0,
                energy_value: 1.0,
                motion_angle_radians: 0.0,
                cluster_id: None,
                cluster_angle_radians: 0.0,
                cluster_distance_factor: 0.0,
            }],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 0,
        };
        let mut night_world = day_world.clone();
        night_world.step_count = 120;

        let day_sensor = sensor_observation(&agent, &day_world, &config);
        let night_sensor = sensor_observation(&agent, &night_world, &config);
        let day_total = day_sensor.food_eye_bins.iter().sum::<f32>();
        let night_total = night_sensor.food_eye_bins.iter().sum::<f32>();

        assert!(day_total > night_total);
        assert!(night_total > 0.0);
    }

    #[test]
    fn sensor_dropout_can_zero_controller_inputs() {
        let config = SimulationConfig {
            sensor_noise_stddev: 0.0,
            sensor_dropout_probability: 1.0,
            food_motion_speed: 0.0,
            ..SimulationConfig::default()
        };
        let agent = test_agent(50.0);
        let world = WorldState {
            width: 40.0,
            height: 40.0,
            agents: vec![agent.clone()],
            food: vec![],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 0,
        };

        let sensor = sensor_observation(&agent, &world, &config);

        assert!(sensor.input.values.iter().all(|value| value.abs() < 1e-6));
    }

    #[test]
    fn sensor_noise_is_deterministic_for_same_step() {
        let config = SimulationConfig {
            sensor_noise_stddev: 0.2,
            sensor_dropout_probability: 0.0,
            food_motion_speed: 0.0,
            ..SimulationConfig::default()
        };
        let mut agent = test_agent(50.0);
        agent.velocity = Vec2::new(1.5, 0.0);
        let world = WorldState {
            width: 40.0,
            height: 40.0,
            agents: vec![agent.clone()],
            food: vec![],
            food_clusters: vec![],
            world_seed: 0,
            step_count: 11,
        };

        let sensor_a = sensor_observation(&agent, &world, &config);
        let sensor_b = sensor_observation(&agent, &world, &config);

        assert_eq!(sensor_a, sensor_b);
    }
}
