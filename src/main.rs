use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;
use neural_survival::cli::{Cli, Commands};
use neural_survival::config::load_config;

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run(args) => {
            let _config = load_config(&args.config)?;
            println!(
                "run scaffolding ready: seed={}, headless_generations={}, out={}",
                args.seed,
                args.headless_generations,
                args.out.display()
            );
        }
        Commands::Batch(args) => {
            let _config = load_config(&args.config)?;
            println!(
                "batch scaffolding ready: seed={}, generations={}, out={}",
                args.seed,
                args.generations,
                args.out.display()
            );
        }
        Commands::View(args) => {
            println!(
                "view scaffolding ready: snapshot={}",
                args.snapshot.display()
            );
        }
    }

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error:#}");
            ExitCode::FAILURE
        }
    }
}
