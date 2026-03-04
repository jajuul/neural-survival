//! Experiment orchestration for deterministic headless evolution runs.

mod runner;

pub use runner::{
    EvolutionRun, GenerationMetrics, continue_generations_from_population, run_batch_to_dir,
    run_generations, world_state_for_population,
};
