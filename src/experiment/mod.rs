//! Experiment orchestration for deterministic headless evolution runs.

mod runner;

pub use runner::{
    EvolutionRun, GenerationMetrics, continue_generations_from_population, run_batch_to_dir,
    run_batch_to_dir_with_progress, run_generations, run_generations_with_progress,
    world_state_for_population,
};
