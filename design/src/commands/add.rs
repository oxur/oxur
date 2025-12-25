//! Add document command implementation

use anyhow::{Context, Result};
use colored::*;
use design::doc::*;
use design::index::DocumentIndex;
use std::fs;
use std::path::PathBuf;

/// Format a step header
fn step_header(num: u32, title: &str) -> String {
    format!("{} {}", format!("Step {}:", num).cyan().bold(), title.cyan())
}

/// Format a success message
fn success(msg: &str) -> String {
    format!("  {} {}", "✓".green(), msg)
}

/// Format a skip message
fn skip(msg: &str) -> String {
    format!("  {} {}", "→".yellow(), msg)
}

/// Add a new document with full processing
pub fn add_document(index: &DocumentIndex, doc_path: &str, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("{}\n", "DRY RUN MODE - No changes will be made".yellow().bold());
    }

    println!("{} {}\n", "Adding document:".bold(), doc_path);

    let mut path = PathBuf::from(doc_path);

    // Validate file exists
    if !path.exists() {
        anyhow::bail!("File not found: {}", doc_path);
    }

    let project_dir = PathBuf::from(index.docs_dir());

    // Step 1: Number Assignment
    let mut filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?
        .to_string();

    // Track simulated path for dry-run mode
    let mut simulated_path = path.clone();

    if !has_number_prefix(&filename) {
        println!("{}", step_header(1, "Assigning number"));

        let next_num = index.next_number();
        println!("  Assigning number: {:04}", next_num);

        if !dry_run {
            path = add_number_prefix(&path, next_num)
                .with_context(|| format!("Failed to add number prefix to {}", path.display()))?;

            let new_filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
            println!("{}\n", success(&format!("Renamed to: {}", new_filename)));
        } else {
            filename = format!("{:04}-{}", next_num, filename);
            simulated_path = path.with_file_name(&filename);
            println!("{}\n", success(&format!("Would rename to: {}", filename)));
        }
    } else {
        println!("{}\n", skip("File already has number prefix"));
    }

    // Step 2: Move to Project Directory
    let in_project = is_in_project_dir(&path, &project_dir).unwrap_or(false);

    if !in_project {
        println!("{}", step_header(2, "Moving to project directory"));

        if !dry_run {
            path = move_to_project(&path, &project_dir)
                .with_context(|| format!("Failed to move {} to project", path.display()))?;

            println!("{}\n", success(&format!("Moved to: {}", path.display())));
        } else {
            simulated_path = project_dir.join(&filename);
            println!("{}\n", success(&format!("Would move to: {}", simulated_path.display())));
        }
    } else {
        println!("{}\n", skip("File already in project directory"));
    }

    // Step 3: State Directory Placement
    let check_path = if dry_run { &simulated_path } else { &path };
    if !is_in_state_dir(check_path) {
        println!("{}", step_header(3, "Moving to draft directory"));

        if !dry_run {
            path = move_to_state_dir(&path, DocState::Draft, &project_dir)
                .with_context(|| format!("Failed to move {} to draft directory", path.display()))?;

            println!("{}\n", success(&format!("Moved to: {}", path.display())));
        } else {
            simulated_path = project_dir.join(DocState::Draft.directory()).join(&filename);
            println!("{}\n", success(&format!("Would move to: {}", simulated_path.display())));
        }
    } else {
        println!("{}\n", skip("File already in state directory"));
    }

    // Step 4: Add/Update YAML Headers
    println!("{}", step_header(4, "Processing headers"));

    let content = fs::read_to_string(&path).context("Failed to read file")?;

    // Use simulated path for header processing in dry-run mode
    let header_path = if dry_run { &simulated_path } else { &path };
    let updated_content =
        ensure_valid_headers(header_path, &content).context("Failed to ensure valid headers")?;

    if content != updated_content {
        if !dry_run {
            fs::write(&path, &updated_content).context("Failed to write headers")?;
            println!("{}\n", success("Added/updated headers"));
        } else {
            println!("{}\n", success("Would add/update headers"));
        }
    } else {
        println!("{}\n", skip("Headers already complete"));
    }

    // Step 5: Sync State with Directory
    println!("{}", step_header(5, "Syncing state with directory"));

    // In dry-run mode, state will already match since we'd be in draft dir
    if dry_run {
        // The simulated path is in the draft directory, so state will match
        println!("{}\n", skip("State would match directory"));
    } else {
        // Re-read content if we might have updated it
        let content = if content != updated_content {
            fs::read_to_string(&path).context("Failed to read file")?
        } else {
            content
        };

        let synced_content =
            sync_state_with_directory(&path, &content).context("Failed to sync state")?;

        if content != synced_content {
            fs::write(&path, &synced_content).context("Failed to write synced content")?;
            println!("{}\n", success("Updated state to match directory"));
        } else {
            println!("{}\n", skip("State already matches directory"));
        }
    }

    // Step 6: Git Add
    println!("{}", step_header(6, "Adding to git"));

    if !dry_run {
        design::git::git_add(&path).context("Failed to git add")?;

        println!("{}\n", success(&format!("Git staged: {}", path.display())));
    } else {
        println!("{}\n", success(&format!("Would git stage: {}", simulated_path.display())));
    }

    // Step 7: Update Index
    println!("{}", step_header(7, "Updating index"));

    if !dry_run {
        // Reload index to pick up the new file
        let updated_index =
            DocumentIndex::new(index.docs_dir()).context("Failed to reload index")?;

        // Run update-index command
        crate::commands::update_index::update_index(&updated_index)
            .context("Failed to update index")?;
    } else {
        println!("{}\n", success("Would update index"));
    }

    // Final summary
    let final_path = if dry_run { &simulated_path } else { &path };
    let final_filename = final_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");

    if !dry_run {
        println!("\n{} Successfully added document: {}", "✓".green().bold(), final_filename.bold());
    } else {
        println!("\n{} Would add document: {}", "→".yellow().bold(), final_filename.bold());
    }

    Ok(())
}
