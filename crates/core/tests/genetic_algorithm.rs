mod common;

use common::tiny_config;
use neural_survival_core::{Config, Evolution, Genome, MLP_PREY, Network};

fn params(genome: &Genome) -> &[f64] {
    let Network::Mlp(mlp) = &genome.network;
    mlp.params()
}

fn ga_config(population: usize) -> Config {
    let mut config = tiny_config();
    config.population.mlp_prey = population;
    config.food.count = 10;
    config.prey.starting_energy = 3.0;
    config.energy.idle_drain = 0.1;
    config.mlp.hidden_layers = vec![4];
    config.evaluation.episode_steps = 20;
    config
}

/// Genomes before and after one Generation of breeding.
fn breed_once(config: Config) -> (Vec<Genome>, Vec<Genome>) {
    let mut evolution = Evolution::new(config, 21).unwrap();
    let parents = evolution.genomes(MLP_PREY).to_vec();
    evolution.advance(1, |_| {});
    (parents, evolution.genomes(MLP_PREY).to_vec())
}

#[test]
fn elitism_copies_exactly_the_top_k_unchanged() {
    for elitism in [0, 1, 3] {
        let mut config = ga_config(8);
        config.ga.elitism = elitism;
        config.ga.mutation_rate = 1.0;
        config.ga.mutation_sigma = 1.0;
        let (parents, children) = breed_once(config);

        let unchanged = children.iter().filter(|c| parents.contains(c)).count();
        assert_eq!(unchanged, elitism, "elitism {elitism}");
        // Elites lead the next Generation.
        assert!(children[..elitism].iter().all(|c| parents.contains(c)));
    }
}

#[test]
fn elites_are_the_fittest_genomes() {
    // With a single elite and a tournament over the whole Species, the elite
    // and both parents of every offspring are the same best Genome, so with
    // no mutation the whole next Generation is that Genome.
    let mut config = ga_config(6);
    config.ga.elitism = 1;
    config.ga.tournament_size = 6;
    config.ga.mutation_rate = 0.0;
    let (parents, children) = breed_once(config);

    assert!(parents.contains(&children[0]));
    assert!(children.iter().all(|c| *c == children[0]));
}

#[test]
fn small_tournaments_let_weaker_genomes_breed() {
    let mut config = ga_config(6);
    config.ga.elitism = 0;
    config.ga.tournament_size = 1;
    config.ga.mutation_rate = 0.0;
    let (_, children) = breed_once(config);

    assert!(children.iter().any(|c| *c != children[0]));
}

#[test]
fn uniform_crossover_takes_each_parameter_from_a_parent() {
    let mut config = ga_config(6);
    config.ga.elitism = 0;
    config.ga.tournament_size = 1;
    config.ga.mutation_rate = 0.0;
    let (parents, children) = breed_once(config);

    for child in &children {
        for (j, value) in params(child).iter().enumerate() {
            assert!(
                parents.iter().any(|p| params(p)[j] == *value),
                "parameter {j} came from no parent"
            );
        }
    }
    // Offspring mix their parents rather than copying one of them.
    assert!(children.iter().any(|c| !parents.contains(c)));
}

/// Per-parameter changes from a lone parent to its offspring: with one
/// Genome, every offspring is a mutated copy of it.
fn mutation_deltas(rate: f64, sigma: f64) -> Vec<f64> {
    let mut config = ga_config(1);
    config.mlp.hidden_layers = vec![60];
    config.ga.elitism = 0;
    config.ga.mutation_rate = rate;
    config.ga.mutation_sigma = sigma;
    let (parents, children) = breed_once(config);
    params(&children[0])
        .iter()
        .zip(params(&parents[0]))
        .map(|(c, p)| c - p)
        .collect()
}

#[test]
fn gaussian_mutation_has_configured_sigma() {
    let deltas = mutation_deltas(1.0, 0.5);
    let n = deltas.len() as f64;
    let mean = deltas.iter().sum::<f64>() / n;
    let sd = (deltas.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / n).sqrt();
    assert!(deltas.iter().all(|&d| d != 0.0));
    assert!(mean.abs() < 0.1, "mean {mean}");
    assert!((0.4..0.6).contains(&sd), "sd {sd}");
}

#[test]
fn mutation_rate_sets_the_share_of_mutated_parameters() {
    let deltas = mutation_deltas(0.25, 0.5);
    let share = deltas.iter().filter(|&&d| d != 0.0).count() as f64 / deltas.len() as f64;
    assert!((0.18..0.32).contains(&share), "share {share}");
}
