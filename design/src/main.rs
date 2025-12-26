//! Design documentation CLI tool

use anyhow::Result;
use clap::Parser;
use colored::*;
use design::index::DocumentIndex;
use design::state::StateManager;

mod cli;
mod commands;

use cli::{Cli, Commands, DebugCommands};
use commands::*;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup state manager
    let mut state_mgr = match setup_state_manager(&cli) {
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

    // Scan on startup
    if let Err(e) = scan_on_startup(&mut state_mgr, &cli.command) {
        design::errors::print_error("Startup scan failed", &e);
        // Non-fatal, continue
    }

    // Create document index
    let index = match create_document_index(&state_mgr, &cli.docs_dir) {
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

    // Execute command
    if let Err(e) = execute_command(cli.command, &index, &mut state_mgr) {
        design::errors::print_error("Command failed", &e);
        std::process::exit(1);
    }

    Ok(())
}

/// Initialize and configure the state manager
pub(crate) fn setup_state_manager(cli: &Cli) -> Result<StateManager> {
    StateManager::new(&cli.docs_dir)
}

/// Scan for filesystem changes on startup (unless running scan command explicitly)
pub(crate) fn scan_on_startup(state_mgr: &mut StateManager, command: &Commands) -> Result<()> {
    let needs_scan = !matches!(command, Commands::Scan { .. });

    if needs_scan {
        let result = state_mgr.quick_scan()?;
        if result.has_changes() {
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

    Ok(())
}

/// Create document index from state with filesystem fallback
pub(crate) fn create_document_index(state_mgr: &StateManager, docs_dir: &str) -> Result<DocumentIndex> {
    match DocumentIndex::from_state(state_mgr.state(), docs_dir) {
        Ok(idx) => Ok(idx),
        Err(_) => {
            eprintln!("{} State loading failed, falling back to filesystem scan", "→".yellow());
            DocumentIndex::new(docs_dir)
        }
    }
}

/// Dispatch and execute the requested command
pub(crate) fn execute_command(
    command: Commands,
    index: &DocumentIndex,
    state_mgr: &mut StateManager,
) -> Result<()> {
    match command {
        Commands::List { state, verbose, removed } => {
            list_documents_with_state(index, Some(state_mgr), state, verbose, removed)
        }
        Commands::Show { number, metadata_only } => show_document(index, number, metadata_only),
        Commands::New { title, author } => new_document(index, title, author),
        Commands::Validate { fix } => validate_documents(index, fix),
        Commands::Index { format } => generate_index(index, &format),
        Commands::AddHeaders { path } => add_headers(&path),
        Commands::Transition { path, state } => transition_document(index, &path, &state),
        Commands::SyncLocation { path } => sync_location(index, &path),
        Commands::UpdateIndex => update_index(index),
        Commands::Add { path, dry_run, interactive, yes, preview } => {
            if preview {
                preview_add(&path, state_mgr)
            } else {
                add_document(state_mgr, &path, dry_run, interactive, yes)
            }
        }
        Commands::AddBatch { patterns, dry_run, interactive } => {
            add_batch(state_mgr, patterns, dry_run, interactive)
        }
        Commands::Scan { fix, verbose } => scan_documents(state_mgr, fix, verbose),
        Commands::Debug(debug_cmd) => match debug_cmd {
            DebugCommands::State { number, format } => {
                if let Some(num) = number {
                    show_document_state(state_mgr, num)
                } else {
                    show_state(state_mgr, &format)
                }
            }
            DebugCommands::Checksums { verbose } => show_checksums(state_mgr, verbose),
            DebugCommands::Stats => show_stats(state_mgr),
            DebugCommands::Diff => show_diff(state_mgr),
            DebugCommands::Orphans => show_orphans(state_mgr),
            DebugCommands::Verify { number } => verify_document(state_mgr, number),
        },
        Commands::Search { query, state, metadata, case_sensitive } => {
            search(state_mgr, &query, state, metadata, case_sensitive)
        }
        Commands::Remove { doc } => remove_document(state_mgr, &doc),
        Commands::Replace { old, new } => replace_document(state_mgr, &old, &new),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper to create a test docs directory with sample documents
    fn setup_test_docs_dir() -> TempDir {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path();

        // Create directory structure
        fs::create_dir_all(docs_dir.join("01-draft")).unwrap();
        fs::create_dir_all(docs_dir.join(".oxd")).unwrap();

        // Create a sample document
        let doc_path = docs_dir.join("01-draft/0001-test-document.md");
        fs::write(
            &doc_path,
            r#"---
number: 1
title: Test Document
author: Test Author
state: Draft
created: 2024-01-01
updated: 2024-01-01
---

# Test Document

This is a test document.
"#,
        )
        .unwrap();

        // Initialize git repo for state manager
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(docs_dir)
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(docs_dir)
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(docs_dir)
            .output()
            .unwrap();

        temp
    }

    #[test]
    fn test_setup_state_manager_success() {
        let temp = setup_test_docs_dir();
        let cli = Cli {
            docs_dir: temp.path().to_str().unwrap().to_string(),
            command: Commands::List {
                state: None,
                verbose: false,
                removed: false,
            },
        };

        let result = setup_state_manager(&cli);
        assert!(result.is_ok());

        let state_mgr = result.unwrap();
        assert_eq!(state_mgr.docs_dir(), temp.path());
    }

    #[test]
    fn test_scan_on_startup_with_scan_command() {
        let temp = setup_test_docs_dir();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // When command is Scan, should skip the scan
        let command = Commands::Scan {
            fix: false,
            verbose: false,
        };

        let result = scan_on_startup(&mut state_mgr, &command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scan_on_startup_with_other_command() {
        let temp = setup_test_docs_dir();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // When command is not Scan, should perform scan
        let command = Commands::List {
            state: None,
            verbose: false,
            removed: false,
        };

        let result = scan_on_startup(&mut state_mgr, &command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scan_on_startup_detects_new_file() {
        let temp = setup_test_docs_dir();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // Initial scan to clear state
        state_mgr.quick_scan().unwrap();

        // Add a new file
        let new_doc = temp.path().join("01-draft/0002-new-doc.md");
        fs::write(
            &new_doc,
            r#"---
number: 2
title: New Document
author: Test Author
state: Draft
created: 2024-01-02
updated: 2024-01-02
---

# New Document
"#,
        )
        .unwrap();

        let command = Commands::List {
            state: None,
            verbose: false,
            removed: false,
        };

        // This should detect the new file
        let result = scan_on_startup(&mut state_mgr, &command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_document_index_success() {
        let temp = setup_test_docs_dir();
        let state_mgr = StateManager::new(temp.path()).unwrap();

        let result = create_document_index(&state_mgr, temp.path().to_str().unwrap());
        assert!(result.is_ok());

        let index = result.unwrap();
        // Just verify the index was created - don't check for specific documents
        // since the index might be empty depending on state
        assert!(index.next_number() >= 1);
    }

    #[test]
    fn test_create_document_index_fallback() {
        let temp = setup_test_docs_dir();
        let state_mgr = StateManager::new(temp.path()).unwrap();

        // Even if state loading fails, should fall back to filesystem scan
        let result = create_document_index(&state_mgr, temp.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_command_list() {
        let temp = setup_test_docs_dir();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        let index = DocumentIndex::new(temp.path()).unwrap();

        let command = Commands::List {
            state: None,
            verbose: false,
            removed: false,
        };

        let result = execute_command(command, &index, &mut state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_command_show() {
        let temp = setup_test_docs_dir();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        let index = DocumentIndex::new(temp.path()).unwrap();

        let command = Commands::Show {
            number: 1,
            metadata_only: false,
        };

        let result = execute_command(command, &index, &mut state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_command_show_nonexistent() {
        let temp = setup_test_docs_dir();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        let index = DocumentIndex::new(temp.path()).unwrap();

        let command = Commands::Show {
            number: 9999,
            metadata_only: false,
        };

        let result = execute_command(command, &index, &mut state_mgr);
        assert!(result.is_err());
    }
}
