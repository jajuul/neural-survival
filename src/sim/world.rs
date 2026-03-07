use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::config::SimulationConfig;

use super::types::{
    ActionVector, AgentState, EnergyBreakdown, EpisodeSummary, FoodState, SensorObservation,
    SensorVector, StepAgentTelemetry, StepOutcome, Vec2, WorldState,
};

/// Number of non-vision scalar features included in controller input.
pub const CORE_SENSOR_FEATURES: usize = 4;
/// Maximum number of bins in one vision channel.
pub const MAX_VISION_CELLS: usize = crate::config::MAX_PERCEPTION_CHANNELS as usize;
/// Number of distinct vision channels currently exposed to controllers.
///
/// Keeping this as a constant makes it straightforward to add future channels such as
/// `edible_agents` or `predators` without reworking the flattening logic again.
pub const VISION_CHANNEL_COUNT: usize = 2;
/// Fixed controller input width used by MLP/LTC genomes.
pub const CONTROLLER_SENSOR_INPUT_SIZE: usize =
    CORE_SENSOR_FEATURES + MAX_VISION_CELLS * VISION_CHANNEL_COUNT;

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
            food_consumed: 0,
        })
        .collect();

    let food = (0..config.food_count)
        .map(|id| FoodState {
            id: id as u32,
            position: Vec2::new(
                rng.gen_range(0.0..config.world_width),
                rng.gen_range(0.0..config.world_height),
            ),
            radius: config.food_collision_radius,
            energy_value: config.food_energy_gain,
        })
        .collect();

    WorldState {
        width: config.world_width,
        height: config.world_height,
        agents,
        food,
        step_count: 0,
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

    for agent_idx in 0..world.agents.len() {
        if !world.agents[agent_idx].alive {
            telemetry[agent_idx].alive_after = false;
            continue;
        }

        let action = actions
            .get(agent_idx)
            .copied()
            .unwrap_or_else(|| stub_action(world.agents[agent_idx].id, world.step_count));
        let speed = apply_kinematics(
            &mut world.agents[agent_idx],
            action,
            world.width,
            world.height,
            config,
            dt,
        );

        let food_gain = if let Some(food_idx) = find_collided_food_index(
            &world.food,
            world.agents[agent_idx].position,
            world.width,
            world.height,
        ) {
            let food = world.food.remove(food_idx);
            world.agents[agent_idx].food_consumed += 1;
            outcome.consumed_food += 1;
            maybe_respawn_food(world, config, agent_idx, &food, &mut next_food_id);
            food.energy_value
        } else {
            0.0
        };

        let energy =
            energy_breakdown_for_step(&world.agents[agent_idx], config, speed, dt, food_gain);
        world.agents[agent_idx].energy =
            (world.agents[agent_idx].energy - energy.total_loss + food_gain).max(0.0);

        if world.agents[agent_idx].energy <= 0.0 {
            world.agents[agent_idx].alive = false;
            outcome.deaths += 1;
        }

        telemetry[agent_idx] = StepAgentTelemetry {
            energy,
            consumed_food: food_gain > 0.0,
            alive_after: world.agents[agent_idx].alive,
        };
    }

    world.step_count += 1;
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
/// `[x_norm, y_norm, energy_norm, speed_norm, food_vision(10), agent_vision(10)]`, where active
/// bins are bounded by the agent's `perception_channels` trait.
/// Side effects: none.
pub fn basic_sensor_vector(
    agent: &AgentState,
    world: &WorldState,
    config: &SimulationConfig,
) -> SensorVector {
    sensor_observation(agent, world, config).input
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
    let x_norm = (agent.position.x / world.width).clamp(0.0, 1.0);
    let y_norm = (agent.position.y / world.height).clamp(0.0, 1.0);
    let energy_norm = (agent.energy / config.initial_agent_energy).clamp(0.0, 2.0);
    let speed_norm = (agent.velocity.length() / agent.max_speed.max(0.001)).clamp(0.0, 1.0);
    let food_eye_bins = food_eye_cells(agent, world, config);
    let agent_eye_bins = agent_eye_cells(agent, world, config);
    let vision_cells = flatten_vision_channels([&food_eye_bins, &agent_eye_bins]);

    let mut values = Vec::with_capacity(CONTROLLER_SENSOR_INPUT_SIZE);
    values.extend_from_slice(&[x_norm, y_norm, energy_norm, speed_norm]);
    values.extend_from_slice(&vision_cells);

    SensorObservation {
        core_values: [x_norm, y_norm, energy_norm, speed_norm],
        food_eye_bins,
        agent_eye_bins,
        eye_bins: vision_cells,
        input: SensorVector { values },
    }
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

    world.food.push(FoodState {
        id: *next_food_id,
        position: Vec2::new(
            rng.gen_range(0.0..world.width),
            rng.gen_range(0.0..world.height),
        ),
        radius: config.food_collision_radius,
        energy_value: config.food_energy_gain,
    });
    *next_food_id = (*next_food_id).saturating_add(1);
}

fn food_eye_cells(agent: &AgentState, world: &WorldState, _config: &SimulationConfig) -> Vec<f32> {
    let mut cells = empty_eye_cells(agent);
    accumulate_eye_cells(
        agent,
        &mut cells,
        world.food.iter().map(|food| food.position),
        world.width,
        world.height,
    );
    cells
}

fn agent_eye_cells(agent: &AgentState, world: &WorldState, _config: &SimulationConfig) -> Vec<f32> {
    let mut cells = empty_eye_cells(agent);
    accumulate_eye_cells(
        agent,
        &mut cells,
        world
            .agents
            .iter()
            .filter(|other| other.alive && other.id != agent.id)
            .map(|other| other.position),
        world.width,
        world.height,
    );
    cells
}

fn empty_eye_cells(_agent: &AgentState) -> Vec<f32> {
    vec![0.0; MAX_VISION_CELLS]
}

fn accumulate_eye_cells<I>(
    agent: &AgentState,
    cells: &mut [f32],
    targets: I,
    world_width: f32,
    world_height: f32,
) where
    I: IntoIterator<Item = Vec2>,
{
    let active_cells = agent.perception_channels.clamp(1, MAX_VISION_CELLS as u16) as usize;
    let fov = (agent.eye_cell_angle_radians.max(0.001) * active_cells as f32)
        .clamp(0.001, std::f32::consts::TAU);
    let half_fov = fov * 0.5;
    let max_range = agent.perception_radius.max(0.001);

    for target_position in targets {
        let offset = toroidal_offset(agent.position, target_position, world_width, world_height);
        let distance = offset.length();

        if distance > max_range {
            continue;
        }

        let heading_to_food = offset.y.atan2(offset.x);
        let relative_angle = wrap_signed_angle(heading_to_food - agent.heading_radians);
        if relative_angle < -half_fov || relative_angle > half_fov {
            continue;
        }

        let normalized = (relative_angle + half_fov) / fov;
        let cell = ((normalized * active_cells as f32).floor() as usize).min(active_cells - 1);
        cells[cell] += (max_range - distance) / max_range;
    }
}

fn flatten_vision_channels<'a, I>(channels: I) -> Vec<f32>
where
    I: IntoIterator<Item = &'a Vec<f32>>,
{
    let mut flat = Vec::with_capacity(MAX_VISION_CELLS * VISION_CHANNEL_COUNT);
    for channel in channels {
        flat.extend_from_slice(channel);
    }
    flat
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

    let speed = throttle * agent.max_speed;
    agent.velocity = Vec2::from_angle(agent.heading_radians).scale(speed);
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
        total_loss,
        net_delta: food_gain - total_loss,
    }
}

fn find_collided_food_index(
    food: &[FoodState],
    position: Vec2,
    width: f32,
    height: f32,
) -> Option<usize> {
    food.iter()
        .position(|item| toroidal_distance(position, item.position, width, height) <= item.radius)
}

#[cfg(test)]
mod tests {
    use crate::config::{RuleToggles, SimulationConfig};

    use super::{
        AgentState, CORE_SENSOR_FEATURES, FoodState, MAX_VISION_CELLS, Vec2, WorldState,
        basic_sensor_vector, energy_breakdown_for_step, sensor_observation, step_world,
        step_world_with_actions_and_telemetry, toroidal_distance, wrap_position,
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
            food_consumed: 0,
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
            }],
            step_count: 0,
        };

        let outcome = step_world(&mut world, &config);

        assert_eq!(outcome.consumed_food, 1);
        assert_eq!(world.food.len(), 0);
        assert_eq!(world.agents[0].food_consumed, 1);
        assert!((world.agents[0].energy - 8.0).abs() < 1e-6);
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
            energy_breakdown_for_step(&agent, &config, 4.0, config.timestep_seconds, 1.5);

        assert!((breakdown.base_loss - 1.0).abs() < 1e-6);
        assert!((breakdown.speed_loss - 6.0).abs() < 1e-6);
        assert!((breakdown.network_loss - 0.8).abs() < 1e-6);
        assert!((breakdown.perception_loss - 4.0).abs() < 1e-6);
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
            }],
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
            }],
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
    fn toroidal_distance_uses_shortest_wrapped_path() {
        let a = Vec2::new(9.0, 5.0);
        let b = Vec2::new(1.0, 5.0);

        let distance = toroidal_distance(a, b, 10.0, 10.0);

        assert!((distance - 2.0).abs() < 1e-6);
    }

    #[test]
    fn basic_sensor_vector_places_food_in_forward_center_cell() {
        let config = SimulationConfig::default();
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
            }],
            step_count: 0,
        };

        let sensor = sensor_observation(&agent, &world, &config);
        let food_vision =
            &sensor.input.values[CORE_SENSOR_FEATURES..CORE_SENSOR_FEATURES + MAX_VISION_CELLS];
        let active = &food_vision[..agent.perception_channels as usize];

        assert!(active[2] > 0.0);
        assert!(sensor.food_eye_bins[2] > 0.0);
        assert!(sensor.agent_eye_bins.iter().all(|value| value.abs() < 1e-6));
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
        let config = SimulationConfig::default();
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
            }],
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
        let config = SimulationConfig::default();
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
            }],
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
        let config = SimulationConfig::default();
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
            }],
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
        let config = SimulationConfig::default();
        let mut agent = test_agent(50.0);
        agent.position = Vec2::new(10.0, 10.0);
        agent.heading_radians = 0.0;
        agent.perception_radius = 20.0;
        agent.perception_channels = 5;
        agent.eye_cell_angle_radians = std::f32::consts::FRAC_PI_2 / 5.0;

        let mut other_agent = test_agent(50.0);
        other_agent.id = 1;
        other_agent.position = Vec2::new(15.0, 10.0);

        let world = WorldState {
            width: 40.0,
            height: 40.0,
            agents: vec![agent.clone(), other_agent],
            food: vec![],
            step_count: 0,
        };

        let sensor = sensor_observation(&agent, &world, &config);

        assert!(sensor.agent_eye_bins[2] > 0.0);
        assert!(sensor.food_eye_bins.iter().all(|value| value.abs() < 1e-6));
        assert!(sensor.eye_bins[MAX_VISION_CELLS + 2] > 0.0);
        assert!(
            sensor.eye_bins[..MAX_VISION_CELLS]
                .iter()
                .all(|value| value.abs() < 1e-6)
        );
    }

    #[test]
    fn sensor_observation_ignores_self_when_building_agent_bins() {
        let config = SimulationConfig::default();
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
            step_count: 0,
        };

        let sensor = sensor_observation(&agent, &world, &config);

        assert!(sensor.agent_eye_bins.iter().all(|value| value.abs() < 1e-6));
        assert!(sensor.eye_bins.iter().all(|value| value.abs() < 1e-6));
    }
}
