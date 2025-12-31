//! AST manipulation and conversion CLI tool

use anyhow::Result;
use clap::Parser;
use oxur_ast::commands;

mod cli;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Err(e) = execute_command(cli.command) {
        oxur_cli::common::output::error(&e.to_string());
        std::process::exit(1);
    }

    Ok(())
}

fn execute_command(command: Commands) -> Result<()> {
    match command {
        Commands::ToAst { input, output, compact, continue_after_error } => {
            commands::to_ast(input, output, compact, continue_after_error)
        }
        Commands::ToRust { input, output } => commands::to_rust(input, output),
        Commands::Verify { input, verbose } => commands::verify(input, verbose),
    }
}
