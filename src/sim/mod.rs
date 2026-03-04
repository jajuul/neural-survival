//! Deterministic fixed-step simulation core.

pub mod types;
mod world;

pub use types::{
    ActionVector, AgentState, EpisodeSummary, FoodState, SensorVector, StepOutcome, Vec2,
    WorldState,
};
pub use world::{
    CONTROLLER_SENSOR_INPUT_SIZE, CORE_SENSOR_FEATURES, MAX_VISION_CELLS, basic_sensor_vector,
    initialize_world, run_headless_episode, run_steps, step_world, step_world_with_actions,
    summarize_world, toroidal_distance, wrap_position,
};
