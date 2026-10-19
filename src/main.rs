//! The `neural-survival` command line.

#![forbid(unsafe_code)]

use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use neural_survival_core::{Config, Evolution, Progress, prepare_output_dir, record_evolution};

#[derive(Parser)]
#[command(version, about = "Evolve neural-network creatures in a 2D world")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Evolve headlessly for a fixed number of Generations and write run files.
    Batch {
        /// TOML config; every field has a default.
        #[arg(long)]
        config: PathBuf,
        /// Run seed; generated and printed when omitted.
        #[arg(long)]
        seed: Option<u64>,
        /// Generations to evolve.
        #[arg(long)]
        generations: u32,
        /// Output directory; must be absent or empty.
        #[arg(long)]
        out: PathBuf,
    },
}

fn main() -> ExitCode {
    let result = match Cli::parse().command {
        Command::Batch {
            config,
            seed,
            generations,
            out,
        } => batch(&config, seed, generations, &out),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn batch(
    config_path: &Path,
    seed: Option<u64>,
    generations: u32,
    out: &Path,
) -> Result<(), Box<dyn Error>> {
    let text = std::fs::read_to_string(config_path)
        .map_err(|error| format!("cannot read config {}: {error}", config_path.display()))?;
    let config = Config::from_toml_str(&text)?;
    let seed = seed.unwrap_or_else(|| {
        let seed = rand::random();
        println!("Generated seed: {seed}");
        seed
    });
    let mut evolution = Evolution::new(config, seed)?;
    prepare_output_dir(out)?;
    record_evolution(&mut evolution, generations, out, print_progress)?;
    println!("Wrote run files to {}", out.display());
    Ok(())
}

fn print_progress(progress: &Progress) {
    let fitness: Vec<String> = progress
        .latest
        .iter()
        .map(|row| {
            format!(
                "{} best {:.3} mean {:.3}",
                row.species, row.best_fitness, row.mean_fitness
            )
        })
        .collect();
    println!(
        "Generation {} ({}/{}): {}",
        progress.generation,
        progress.completed,
        progress.total,
        fitness.join(", ")
    );
}
