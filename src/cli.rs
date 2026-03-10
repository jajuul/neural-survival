use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// Top-level command line interface for neural-survival.
#[derive(Debug, Parser)]
#[command(
    name = "neural-survival",
    version,
    about = "Evolve and inspect neural survival experiments"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Supported user workflows.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Evolve headlessly for N generations, then open the viewer.
    Run(RunArgs),
    /// Evolve headlessly and emit output artifacts.
    Batch(BatchArgs),
    /// Open the viewer from an existing snapshot.
    View(ViewArgs),
}

/// Arguments for `run` mode.
#[derive(Debug, Args)]
pub struct RunArgs {
    /// Path to TOML simulation configuration.
    #[arg(long)]
    pub config: PathBuf,
    /// Deterministic run seed. If omitted, a random seed is generated.
    #[arg(long)]
    pub seed: Option<u64>,
    /// Number of generations to evolve before opening the GUI.
    #[arg(long = "headless-generations")]
    pub headless_generations: u32,
    /// Output directory for artifacts.
    #[arg(long)]
    pub out: PathBuf,
}

/// Arguments for `batch` mode.
#[derive(Debug, Args)]
pub struct BatchArgs {
    /// Path to TOML simulation configuration.
    #[arg(long)]
    pub config: PathBuf,
    /// Deterministic run seed. If omitted, a random seed is generated.
    #[arg(long)]
    pub seed: Option<u64>,
    /// Number of generations to evolve.
    #[arg(long)]
    pub generations: u32,
    /// Output directory for artifacts.
    #[arg(long)]
    pub out: PathBuf,
}

/// Arguments for `view` mode.
#[derive(Debug, Args)]
pub struct ViewArgs {
    /// Path to snapshot JSON.
    #[arg(long)]
    pub snapshot: PathBuf,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::Cli;

    #[test]
    fn run_parses_without_seed() {
        let parsed = Cli::try_parse_from([
            "neural-survival",
            "run",
            "--config",
            "configs/default.toml",
            "--headless-generations",
            "10",
            "--out",
            "/tmp/ns",
        ]);

        assert!(parsed.is_ok());
    }

    #[test]
    fn run_parses_with_required_flags() {
        let parsed = Cli::try_parse_from([
            "neural-survival",
            "run",
            "--config",
            "configs/default.toml",
            "--seed",
            "42",
            "--headless-generations",
            "10",
            "--out",
            "/tmp/ns",
        ]);

        assert!(parsed.is_ok());
    }

    #[test]
    fn view_parses_snapshot_flag() {
        let parsed = Cli::try_parse_from([
            "neural-survival",
            "view",
            "--snapshot",
            "/tmp/snapshot_gen_0010.json",
        ]);

        assert!(parsed.is_ok());
    }
}
