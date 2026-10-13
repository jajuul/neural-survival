//! Headless Neural Survival library: config, simulation, and evolution, with no GUI
//! dependency (ADR-0003).

mod config;
mod episode;
mod evolution;
mod ga;
mod genome;
mod geometry;
mod metrics;
mod mlp;
mod rng;
mod run_files;
mod sensing;
mod showcase;
mod snapshot;
mod species;
mod telemetry;
mod world;

pub use config::*;
pub use evolution::{Evolution, EvolutionError, PROGRESS_INTERVAL, Progress};
pub use genome::{CONTROLLER_OUTPUTS, ControllerFamily, Genome, Network, NetworkShapeError, Role};
pub use geometry::Vec2;
pub use metrics::MetricsRow;
pub use mlp::{Mlp, NetworkError};
pub use run_files::{
    RunFilesError, prepare_output_dir, record_evolution, snapshot_file_name, write_run_files,
};
pub use showcase::{Showcase, ShowcaseBuilder, ShowcaseError};
pub use snapshot::{SNAPSHOT_FORMAT_VERSION, Snapshot, SnapshotError};
pub use species::{MLP_PREY, Species};
pub use telemetry::{
    AgentTelemetry, AppliedOutputs, CoreSensors, EnergyCost, EnergyRule, EyeKind, EyeReading,
    RawOutputs,
};
pub use world::{Agent, AgentId, Arena, Food, Placement};
