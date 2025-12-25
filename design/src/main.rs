//! Design documentation CLI tool

use anyhow::Result;
use clap::Parser;
use design::index::DocumentIndex;

mod cli;
mod commands;

use cli::{Cli, Commands};
use commands::*;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load the document index
    let index = DocumentIndex::new(&cli.docs_dir)?;

    // Execute the command
    match cli.command {
        Commands::List { state, verbose } => {
            list_documents(&index, state, verbose)?;
        }
        Commands::Show { number, metadata_only } => {
            show_document(&index, number, metadata_only)?;
        }
        Commands::New { title, author } => {
            new_document(&index, title, author)?;
        }
        Commands::Validate { fix } => {
            validate_documents(&index, fix)?;
        }
        Commands::Index { format } => {
            eprintln!("Index generation not yet implemented");
            eprintln!("Format: {}", format);
        }
    }

    Ok(())
}
