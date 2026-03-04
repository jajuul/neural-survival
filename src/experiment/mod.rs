//! Experiment orchestration for deterministic headless evolution runs.

mod runner;

pub use runner::{EvolutionRun, GenerationMetrics, run_batch_to_dir, run_generations};
