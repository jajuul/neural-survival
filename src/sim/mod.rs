//! Deterministic fixed-step simulation core.

pub mod types;
mod world;

pub use types::{
    ActionVector, AgentRole, AgentState, CONTROLLER_SENSOR_INPUT_SIZE, CORE_SENSOR_FEATURES,
    DayNightState, EnergyBreakdown, EpisodeSummary, FoodClusterState, FoodState, MAX_VISION_CELLS,
    SensorObservation, SensorVector, StepAgentTelemetry, StepOutcome, VISION_CHANNEL_COUNT, Vec2,
    WorldState,
};
pub use world::{
    basic_sensor_vector, cluster_density_radius_scale, cluster_productivity,
    current_day_night_state, energy_breakdown_for_step, fill_population_sensor_vectors,
    initialize_world, run_headless_episode, run_steps, sensor_observation, step_world,
    step_world_with_actions, step_world_with_actions_and_telemetry, summarize_world,
    toroidal_distance, wrap_position,
};
