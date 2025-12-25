//! Add headers command implementation

use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::PathBuf;

/// Add or update YAML frontmatter headers
pub fn add_headers(doc_path: &str) -> Result<()> {
    let path = PathBuf::from(doc_path);

    // Validate file exists
    if !path.exists() {
        anyhow::bail!("File not found: {}", doc_path);
    }

    println!("Adding/updating headers for: {}\n", path.display());

    // Read current content
    let content = fs::read_to_string(&path).context("Failed to read file")?;

    // Add missing headers
    let (new_content, added_fields) = design::doc::add_missing_headers(&path, &content)?;

    // Write updated content
    fs::write(&path, new_content).context("Failed to write file")?;

    // Report what was done
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");

    if added_fields.is_empty() {
        println!("{}", format!("✓ All headers already present in {}", filename).green());
    } else {
        println!("{}", format!("✓ Added/updated headers in {}", filename).green());
        for field in added_fields {
            println!("  {}: {}", "Added".cyan(), field);
        }
    }

    println!();
    Ok(())
}
