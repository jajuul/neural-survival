//! Deterministic fixed-step simulation core.

pub mod types;
mod world;

pub use types::{
    ActionVector, AgentState, EnergyBreakdown, EpisodeSummary, FoodState, SensorObservation,
    SensorVector, StepAgentTelemetry, StepOutcome, Vec2, WorldState,
};
pub use world::{
    CONTROLLER_SENSOR_INPUT_SIZE, CORE_SENSOR_FEATURES, MAX_VISION_CELLS, VISION_CHANNEL_COUNT,
    basic_sensor_vector, energy_breakdown_for_step, initialize_world, run_headless_episode,
    run_steps, sensor_observation, step_world, step_world_with_actions,
    step_world_with_actions_and_telemetry, summarize_world, toroidal_distance, wrap_position,
};
