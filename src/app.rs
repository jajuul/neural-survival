use anyhow::{Context, Result};
use std::io::{self, Write};

use crate::cli::{BatchArgs, Commands, RunArgs, ViewArgs};
use crate::config::load_config;
use crate::experiment::run_batch_to_dir_with_progress;
use crate::io::snapshot::read_snapshot;
use crate::ui::launch_snapshot_viewer;

/// Execute one CLI command.
///
/// Inputs: parsed command enum and arguments.
/// Output: success/error for command execution.
/// Side effects: runs simulations, writes artifacts, reads snapshots, and may launch viewer.
pub fn execute_command(command: Commands) -> Result<()> {
    match command {
        Commands::Run(args) => run_command(args),
        Commands::Batch(args) => batch_command(args),
        Commands::View(args) => view_command(args),
    }
}

fn run_command(args: RunArgs) -> Result<()> {
    let config = load_config(&args.config)
        .with_context(|| format!("failed to load config at {}", args.config.display()))?;

    run_batch_to_dir_with_progress(
        &config,
        args.seed,
        args.headless_generations,
        &args.out,
        |completed, total| {
            print!("\rheadless evolution progress: {completed}/{total} generations");
            let _ = io::stdout().flush();
        },
    )
    .with_context(|| {
        format!(
            "failed to execute headless run for {} generations",
            args.headless_generations
        )
    })?;
    if args.headless_generations > 0 {
        println!();
    }

    let snapshot_path = args.out.join(format!(
        "snapshot_gen_{:04}.json",
        args.headless_generations
    ));
    let snapshot = read_snapshot(&snapshot_path)
        .with_context(|| format!("failed to load snapshot {}", snapshot_path.display()))?;

    launch_snapshot_viewer(&snapshot)
}

fn batch_command(args: BatchArgs) -> Result<()> {
    let config = load_config(&args.config)
        .with_context(|| format!("failed to load config at {}", args.config.display()))?;

    run_batch_to_dir_with_progress(
        &config,
        args.seed,
        args.generations,
        &args.out,
        |completed, total| {
            print!("\rbatch evolution progress: {completed}/{total} generations");
            let _ = io::stdout().flush();
        },
    )
    .with_context(|| {
        format!(
            "failed to execute batch run for {} generations",
            args.generations
        )
    })?;
    if args.generations > 0 {
        println!();
    }

    Ok(())
}

fn view_command(args: ViewArgs) -> Result<()> {
    let snapshot = read_snapshot(&args.snapshot)
        .with_context(|| format!("failed to load snapshot {}", args.snapshot.display()))?;

    launch_snapshot_viewer(&snapshot)
}
