//! Deterministic fixed-step simulation core.

pub mod types;
mod world;

pub use types::{
    ActionVector, AgentState, EpisodeSummary, FoodState, SensorVector, StepOutcome, Vec2,
    WorldState,
};
pub use world::{
    initialize_world, run_headless_episode, run_steps, step_world, summarize_world,
    toroidal_distance, wrap_position,
};
