//! State transition command implementation

use anyhow::{Context, Result};
use colored::*;
use design::doc::{DesignDoc, DocState};
use design::index::DocumentIndex;
use std::fs;
use std::path::PathBuf;

/// Transition a document to a new state
pub fn transition_document(
    index: &DocumentIndex,
    doc_path: &str,
    new_state_str: &str,
) -> Result<()> {
    let path = PathBuf::from(doc_path);

    // Validate file exists
    if !path.exists() {
        anyhow::bail!("File not found: {}", doc_path);
    }

    // Check if document has headers, add them if missing
    let content = fs::read_to_string(&path).context("Failed to read file")?;

    let content = if !content.trim_start().starts_with("---") {
        println!(
            "{}",
            "Document missing headers, adding them automatically...".yellow()
        );
        let (new_content, _) = design::doc::add_missing_headers(&path, &content)?;
        fs::write(&path, &new_content).context("Failed to write headers")?;
        new_content
    } else {
        content
    };

    // Parse document to get current state
    let doc =
        DesignDoc::parse(&content, path.clone()).context("Failed to parse document")?;

    let current_state = doc.metadata.state;

    // Parse new state
    let new_state = DocState::from_str_flexible(new_state_str).ok_or_else(|| {
        let valid_states = DocState::all_state_names().join(", ");
        anyhow::anyhow!(
            "Unsupported state '{}'. Valid states are: {}",
            new_state_str,
            valid_states
        )
    })?;

    // Check if already in that state
    if current_state == new_state {
        anyhow::bail!("Document is already in state '{}'", current_state.as_str());
    }

    // Update YAML frontmatter
    let updated_content =
        DesignDoc::update_state(&content, new_state).context("Failed to update YAML")?;

    // Write updated content back to same file first
    fs::write(&path, updated_content).context("Failed to write updated content")?;

    // Move to new state directory
    let filename = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;

    let new_dir = PathBuf::from(index.docs_dir()).join(new_state.directory());
    let new_path = new_dir.join(filename);

    design::git::git_mv(&path, &new_path).context("Failed to move document")?;

    println!(
        "{} {} {} {} {}",
        "✓".green().bold(),
        "Transitioned".green(),
        filename.to_string_lossy().bold(),
        "from".green(),
        current_state.as_str().cyan()
    );
    println!("  {} {}", "to".green(), new_state.as_str().cyan());
    println!("  {} {}", "File:".dimmed(), new_path.display());

    Ok(())
}
