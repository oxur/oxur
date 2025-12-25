//! Design documentation CLI tool

use anyhow::Result;
use clap::Parser;
use colored::*;
use design::index::DocumentIndex;
use design::state::StateManager;

mod cli;
mod commands;

use cli::{Cli, Commands};
use commands::*;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize state manager
    let mut state_mgr = match StateManager::new(&cli.docs_dir) {
        Ok(mgr) => mgr,
        Err(e) => {
            design::errors::print_error_with_suggestion(
                "Failed to initialize state manager",
                &e,
                &format!("Make sure '{}' exists and contains design documents", cli.docs_dir),
            );
            std::process::exit(1);
        }
    };

    // Scan for changes on startup (unless running scan command explicitly)
    let needs_scan = !matches!(cli.command, Commands::Scan { .. });

    if needs_scan {
        if let Ok(result) = state_mgr.quick_scan() {
            if result.has_changes() {
                // Show brief message about detected changes
                let total = result.total_changes();
                if total > 0 {
                    eprintln!(
                        "{} Detected {} change(s) ({} new, {} modified, {} deleted)",
                        "→".cyan(),
                        total,
                        result.new_files.len(),
                        result.changed.len(),
                        result.deleted.len()
                    );
                }
            }
        }
    }

    // Create DocumentIndex from state (for compatibility with existing commands)
    let index = match DocumentIndex::from_state(state_mgr.state(), &cli.docs_dir) {
        Ok(idx) => idx,
        Err(_) => {
            // Fall back to traditional loading if state-based loading fails
            eprintln!("{} State loading failed, falling back to filesystem scan", "→".yellow());
            match DocumentIndex::new(&cli.docs_dir) {
                Ok(idx) => idx,
                Err(e2) => {
                    design::errors::print_error_with_suggestion(
                        "Failed to load document index",
                        &e2,
                        &format!(
                            "Make sure '{}' exists and contains design documents",
                            cli.docs_dir
                        ),
                    );
                    std::process::exit(1);
                }
            }
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
        Commands::Scan { fix, verbose } => scan_documents(&mut state_mgr, fix, verbose),
    };

    if let Err(e) = result {
        design::errors::print_error("Command failed", &e);
        std::process::exit(1);
    }

    Ok(())
}
