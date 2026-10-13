//! The genetic algorithm that breeds one Species' next Genomes.

use rand::seq::index;

use crate::config::GaConfig;
use crate::genome::Genome;
use crate::rng::Rng;

/// The next Generation of a Species, the same size as `parents`: the top
/// `elitism` Genomes unchanged, then offspring of tournament-selected parents
/// by uniform crossover and Gaussian mutation. `fitness[i]` scores `parents[i]`.
pub(crate) fn breed(
    parents: &[Genome],
    fitness: &[f64],
    ga: &GaConfig,
    rng: &mut Rng,
) -> Vec<Genome> {
    debug_assert_eq!(parents.len(), fitness.len());
    let mut ranked: Vec<usize> = (0..parents.len()).collect();
    // Stable, so ties keep the lower index first.
    ranked.sort_by(|&a, &b| fitness[b].total_cmp(&fitness[a]));
    let mut next: Vec<Genome> = ranked
        .iter()
        .take(ga.elitism)
        .map(|&i| parents[i].clone())
        .collect();
    while next.len() < parents.len() {
        let a = tournament(fitness, ga.tournament_size, rng);
        let b = tournament(fitness, ga.tournament_size, rng);
        let mut child = parents[a].crossover(&parents[b], rng);
        child.mutate(ga.mutation_rate, ga.mutation_sigma, rng);
        next.push(child);
    }
    next
}

/// The fittest of `size` distinct Genomes drawn at random (all of them when
/// `size` exceeds the population), ties going to the lower index.
fn tournament(fitness: &[f64], size: usize, rng: &mut Rng) -> usize {
    index::sample(rng, fitness.len(), size.min(fitness.len()))
        .into_iter()
        .max_by(|&a, &b| fitness[a].total_cmp(&fitness[b]).then(b.cmp(&a)))
        .expect("a Species being bred has at least one Genome")
}
