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
    let index = match DocumentIndex::new(&cli.docs_dir) {
        Ok(idx) => idx,
        Err(e) => {
            design::errors::print_error_with_suggestion(
                "Failed to load document index",
                &e,
                &format!("Make sure '{}' exists and contains design documents", cli.docs_dir),
            );
            std::process::exit(1);
        }
    };

    // Execute the command
    let result = match cli.command {
        Commands::List { state, verbose } => list_documents(&index, state, verbose),
        Commands::Show { number, metadata_only } => show_document(&index, number, metadata_only),
        Commands::New { title, author } => new_document(&index, title, author),
        Commands::Validate { fix } => validate_documents(&index, fix),
        Commands::Index { format } => generate_index(&index, &format),
        Commands::AddHeaders { path } => add_headers(&path),
        Commands::Transition { path, state } => transition_document(&index, &path, &state),
        Commands::SyncLocation { path } => sync_location(&index, &path),
        Commands::UpdateIndex => update_index(&index),
        Commands::Add { path, dry_run } => add_document(&index, &path, dry_run),
    };

    if let Err(e) = result {
        design::errors::print_error("Command failed", &e);
        std::process::exit(1);
    }

    Ok(())
}
