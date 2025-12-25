//! Sync location command implementation

use anyhow::{Context, Result};
use colored::*;
use design::doc::DesignDoc;
use design::index::DocumentIndex;
use std::fs;
use std::path::PathBuf;

/// Move document to match its header state
pub fn sync_location(index: &DocumentIndex, doc_path: &str) -> Result<()> {
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

    // Parse document to get header state
    let doc =
        DesignDoc::parse(&content, path.clone()).context("Failed to parse document")?;

    let header_state = doc.metadata.state;

    // Determine target directory from state
    let target_dir = PathBuf::from(index.docs_dir()).join(header_state.directory());

    // Check current directory
    let current_dir = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine current directory"))?;

    // Canonicalize for comparison (handle . and ..)
    let current_dir_canonical = current_dir.canonicalize().unwrap_or(current_dir.to_path_buf());
    let target_dir_canonical = if target_dir.exists() {
        target_dir.canonicalize().unwrap_or(target_dir.clone())
    } else {
        target_dir.clone()
    };

    if current_dir_canonical == target_dir_canonical {
        println!(
            "{} {}",
            "✓".green().bold(),
            format!(
                "Document is already in the correct directory for state '{}'",
                header_state.as_str()
            )
            .green()
        );
        return Ok(());
    }

    // Move the file
    let filename = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
    let target_path = target_dir.join(filename);

    design::git::git_mv(&path, &target_path).context("Failed to move document")?;

    println!(
        "{} {} {} {} (state: {})",
        "✓".green().bold(),
        "Moved".green(),
        filename.to_string_lossy().bold(),
        "to match header".green(),
        header_state.as_str().cyan()
    );
    println!("  {}: {}", "From".dimmed(), current_dir.display());
    println!("  {}: {}", "To".dimmed(), target_dir.display());

    Ok(())
}
