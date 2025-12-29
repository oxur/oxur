//! AST manipulation and conversion CLI tool

use anyhow::Result;
use clap::Parser;
use colored::*;

mod cli;
mod commands;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Err(e) = execute_command(cli.command) {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }

    Ok(())
}

fn execute_command(command: Commands) -> Result<()> {
    match command {
        Commands::ToAst { input, output, compact } => commands::to_ast(input, output, compact),
        Commands::ToRust { input, output } => commands::to_rust(input, output),
        Commands::Verify { input, verbose } => commands::verify(input, verbose),
    }
}
