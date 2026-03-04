use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::config::SimulationConfig;

use super::types::{
    ActionVector, AgentState, EpisodeSummary, FoodState, StepOutcome, Vec2, WorldState,
};

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
    let mut outcome = StepOutcome::default();
    let dt = config.timestep_seconds;

    for agent_idx in 0..world.agents.len() {
        if !world.agents[agent_idx].alive {
            continue;
        }

        let action = stub_action(world.agents[agent_idx].id, world.step_count);
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
            food.energy_value
        } else {
            0.0
        };

        let energy_loss = energy_loss_for_step(&world.agents[agent_idx], config, speed, dt);
        world.agents[agent_idx].energy =
            (world.agents[agent_idx].energy - energy_loss + food_gain).max(0.0);

        if world.agents[agent_idx].energy <= 0.0 {
            world.agents[agent_idx].alive = false;
            outcome.deaths += 1;
        }
    }

    world.step_count += 1;
    outcome
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

fn energy_loss_for_step(agent: &AgentState, config: &SimulationConfig, speed: f32, dt: f32) -> f32 {
    let metabolic_scale = 1.0 / agent.metabolic_efficiency.max(0.001);

    let mut total_loss = config.base_energy_cost_per_second * dt * metabolic_scale;

    if config.rule_toggles.energy_by_speed {
        total_loss += config.speed_energy_cost_factor * speed * dt;
    }

    if config.rule_toggles.energy_by_network_size {
        total_loss += config.network_energy_cost_factor * agent.network_size as f32 * dt;
    }

    if config.rule_toggles.energy_by_perception_size {
        total_loss += config.perception_energy_cost_factor
            * agent.perception_radius
            * agent.perception_channels as f32
            * dt;
    }

    total_loss
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
        AgentState, FoodState, Vec2, WorldState, step_world, toroidal_distance, wrap_position,
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
    fn toroidal_distance_uses_shortest_wrapped_path() {
        let a = Vec2::new(9.0, 5.0);
        let b = Vec2::new(1.0, 5.0);

        let distance = toroidal_distance(a, b, 10.0, 10.0);

        assert!((distance - 2.0).abs() < 1e-6);
    }
}
