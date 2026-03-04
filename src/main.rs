use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;
use neural_survival::app::execute_command;
use neural_survival::cli::Cli;

fn run() -> Result<()> {
    let cli = Cli::parse();
    execute_command(cli.command)
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
