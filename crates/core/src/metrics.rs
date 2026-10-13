//! Per-Species results recorded for each evaluated Generation.

use serde::{Deserialize, Serialize};

use crate::species::Species;

/// One Species' results for one Generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricsRow {
    pub generation: u32,
    pub species: Species,
    pub episodes: u32,
    pub episode_steps: u32,
    pub best_fitness: f64,
    pub mean_fitness: f64,
    /// Agents alive at Episode end, averaged over the Episodes.
    pub alive_at_end: f64,
    /// Food eaten per Episode, averaged over the Episodes.
    pub food_eaten: f64,
}
