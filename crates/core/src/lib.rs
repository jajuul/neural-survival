//! Headless Neural Survival library: config and simulation, with no GUI
//! dependency (ADR-0003).

mod config;
mod genome;
mod geometry;
mod mlp;
mod rng;
mod sensing;
mod showcase;
mod telemetry;
mod world;

pub use config::*;
pub use genome::{CONTROLLER_OUTPUTS, ControllerFamily, Genome, Network, Role};
pub use geometry::Vec2;
pub use mlp::{Mlp, NetworkError};
pub use showcase::{Showcase, ShowcaseBuilder, ShowcaseError};
pub use telemetry::{
    AgentTelemetry, AppliedOutputs, CoreSensors, EnergyCost, EnergyRule, EyeKind, EyeReading,
    RawOutputs,
};
pub use world::{Agent, AgentId, Arena, Food, Placement};
