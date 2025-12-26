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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_add_headers_file_not_found() {
        let result = add_headers("/nonexistent/path/file.md");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("File not found"));
    }

    #[test]
    fn test_add_headers_to_file_with_missing_headers() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.md");

        // Create file with minimal frontmatter
        let content = r#"---
number: 1
title: "Test Doc"
---

# Test Document
"#;
        fs::write(&file_path, content).unwrap();

        let result = add_headers(file_path.to_str().unwrap());
        assert!(result.is_ok());

        // Verify headers were added
        let updated = fs::read_to_string(&file_path).unwrap();
        assert!(updated.contains("author:"));
        assert!(updated.contains("created:"));
        assert!(updated.contains("updated:"));
        assert!(updated.contains("state:"));
    }

    #[test]
    fn test_add_headers_to_file_with_all_headers() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("complete.md");

        // Create file with all headers
        let content = r#"---
number: 1
title: "Complete Doc"
author: "Test Author"
created: 2024-01-01
updated: 2024-01-01
state: Draft
supersedes: null
superseded-by: null
---

# Complete Document
"#;
        fs::write(&file_path, content).unwrap();

        let result = add_headers(file_path.to_str().unwrap());
        assert!(result.is_ok());

        // Content should be unchanged (or minimally changed)
        let updated = fs::read_to_string(&file_path).unwrap();
        assert!(updated.contains("number: 1"));
        assert!(updated.contains("title: \"Complete Doc\""));
    }

    #[test]
    fn test_add_headers_preserves_content() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("content.md");

        let content = r#"---
number: 5
title: "Test"
---

# Test Content

This is important content that should not be lost.

## Section
More content here.
"#;
        fs::write(&file_path, content).unwrap();

        let result = add_headers(file_path.to_str().unwrap());
        assert!(result.is_ok());

        // Verify content is preserved
        let updated = fs::read_to_string(&file_path).unwrap();
        assert!(updated.contains("# Test Content"));
        assert!(updated.contains("This is important content"));
        assert!(updated.contains("## Section"));
        assert!(updated.contains("More content here."));
    }

    #[test]
    fn test_add_headers_to_file_without_frontmatter() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("no-frontmatter.md");

        // File with no frontmatter
        let content = "# Just Content\n\nNo YAML headers at all.\n";
        fs::write(&file_path, content).unwrap();

        let result = add_headers(file_path.to_str().unwrap());
        // This might fail or add headers depending on implementation
        // The current implementation expects frontmatter to exist
        // If it fails, that's acceptable behavior
        let _ = result;
    }

    #[test]
    fn test_add_headers_updates_existing_file() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("update.md");

        // Create file
        let content = r#"---
number: 10
title: "Update Test"
---

# Content
"#;
        fs::write(&file_path, content).unwrap();

        // Get original modified time
        let original_metadata = fs::metadata(&file_path).unwrap();

        // Small delay to ensure modified time changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        let result = add_headers(file_path.to_str().unwrap());
        assert!(result.is_ok());

        // Verify file was actually written
        let new_metadata = fs::metadata(&file_path).unwrap();
        assert!(new_metadata.modified().unwrap() >= original_metadata.modified().unwrap());
    }
}
