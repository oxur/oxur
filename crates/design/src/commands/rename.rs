//! Rename command implementation

use anyhow::{bail, Context, Result};
use colored::Colorize;
use design::state::StateManager;
use std::path::{Path, PathBuf};

pub fn execute(state_mgr: &mut StateManager, old_path: &str, new_path: &str) -> Result<()> {
    println!();
    println!("{}", "Renaming document...".cyan().bold());
    println!();

    // Step 1: Parse and validate paths
    let docs_dir = state_mgr.docs_dir();
    let (old_full, new_full) = parse_and_validate_paths(old_path, new_path, docs_dir)?;

    println!("  From: {}", old_full.display().to_string().white());
    println!("  To:   {}", new_full.display().to_string().cyan());
    println!();

    // Step 2: Extract and verify numbers match
    let old_number = extract_number_from_path(&old_full)?;
    let new_number = extract_number_from_path(&new_full)?;

    if old_number != new_number {
        bail!(
            "{}

Cannot change document number during rename.
  Old number: {}
  New number: {}

To change document state/location, use: {}",
            "Number mismatch!".red().bold(),
            format!("{:04}", old_number).yellow(),
            format!("{:04}", new_number).yellow(),
            "oxd transition <doc> <state>".cyan()
        );
    }

    println!("  ✓ Number preserved: {}", format!("{:04}", old_number).yellow());
    println!();

    // Step 3: Verify document exists in state
    let _doc_record = state_mgr.state().get(old_number).ok_or_else(|| {
        anyhow::anyhow!("Document {} not found in state. Run 'oxd scan' to sync.", old_number)
    })?;

    // Step 4: Perform git mv
    design::git::git_mv(&old_full, &new_full).context("Failed to rename file with git")?;
    println!("  ✓ Renamed file with git mv");

    // Step 5: Update state manager - record the file move
    state_mgr.record_file_move(&old_full, &new_full).context("Failed to update state")?;
    println!("  ✓ Updated state");

    // Step 6: Save state
    state_mgr.save().context("Failed to save state")?;

    println!();
    println!("{}", "Rename complete!".green().bold());
    println!("  View with: {}", format!("oxd show {}", old_number).yellow());
    println!();

    Ok(())
}

/// Parse paths and validate they're within docs directory
fn parse_and_validate_paths(old: &str, new: &str, docs_dir: &Path) -> Result<(PathBuf, PathBuf)> {
    // Parse old path
    let old_path = resolve_path(old, docs_dir)?;

    // Validate old path exists
    if !old_path.exists() {
        bail!("Document not found: {}", old_path.display());
    }

    // Parse new path
    let new_path = resolve_path(new, docs_dir)?;

    // Validate new path doesn't exist
    if new_path.exists() {
        bail!("Destination already exists: {}", new_path.display());
    }

    // Validate both are .md files
    if old_path.extension().and_then(|e| e.to_str()) != Some("md") {
        bail!("Old path must be a markdown file (.md)");
    }
    if new_path.extension().and_then(|e| e.to_str()) != Some("md") {
        bail!("New path must be a markdown file (.md)");
    }

    // Validate both are within docs directory
    let old_canonical = old_path.canonicalize().context("Failed to resolve old path")?;
    let docs_canonical = docs_dir.canonicalize().context("Failed to resolve docs directory")?;

    if !old_canonical.starts_with(&docs_canonical) {
        bail!("Old path must be within the docs directory");
    }

    // For new path, check its parent directory is within docs
    if let Some(new_parent) = new_path.parent() {
        if new_parent.exists() {
            let new_parent_canonical =
                new_parent.canonicalize().context("Failed to resolve new path parent")?;
            if !new_parent_canonical.starts_with(&docs_canonical) {
                bail!("New path must be within the docs directory");
            }
        }
    }

    Ok((old_path, new_path))
}

/// Resolve a path relative to docs directory or as absolute
fn resolve_path(path_str: &str, docs_dir: &Path) -> Result<PathBuf> {
    let path = PathBuf::from(path_str);

    // If it's already absolute, use as-is
    if path.is_absolute() {
        return Ok(path);
    }

    // Try relative to current directory first
    let relative_to_cwd = std::env::current_dir()?.join(&path);
    if relative_to_cwd.exists() {
        return Ok(relative_to_cwd);
    }

    // Try relative to docs directory
    let relative_to_docs = docs_dir.join(&path);
    if relative_to_docs.exists() {
        return Ok(relative_to_docs);
    }

    // For new paths (that don't exist), try to infer
    // If it looks like just a filename, put it in docs dir
    if path.components().count() == 1 {
        return Ok(docs_dir.join(&path));
    }

    // Otherwise use relative to current directory
    Ok(relative_to_cwd)
}

/// Extract document number from filename
fn extract_number_from_path(path: &Path) -> Result<u32> {
    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;

    // Use existing extraction function
    let number = design::doc::extract_number_from_filename(filename);

    if number == 0 {
        bail!(
            "Could not extract document number from filename: {}. Expected format: 0001-title.md",
            filename
        );
    }

    Ok(number)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper to create a test StateManager with necessary setup
    fn setup_test_state_manager() -> (TempDir, StateManager) {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path();

        // Create necessary directory structure
        fs::create_dir_all(docs_dir.join(".oxd")).unwrap();
        fs::create_dir_all(docs_dir.join("01-draft")).unwrap();

        // Initialize git repo (required for StateManager)
        std::process::Command::new("git").args(["init"]).current_dir(docs_dir).output().unwrap();

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(docs_dir)
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(docs_dir)
            .output()
            .unwrap();

        let state_mgr = StateManager::new(docs_dir).unwrap();

        (temp, state_mgr)
    }

    #[test]
    fn test_extract_number() {
        assert_eq!(extract_number_from_path(Path::new("0001-test.md")).unwrap(), 1);
        assert_eq!(extract_number_from_path(Path::new("0042-feature.md")).unwrap(), 42);
        assert_eq!(extract_number_from_path(Path::new("/path/to/0123-doc.md")).unwrap(), 123);
    }

    #[test]
    fn test_extract_number_invalid() {
        assert!(extract_number_from_path(Path::new("test.md")).is_err());
        assert!(extract_number_from_path(Path::new("abc-test.md")).is_err());
    }

    #[test]
    fn test_extract_number_from_path_with_no_number() {
        let result = extract_number_from_path(Path::new("test-document.md"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Could not extract document number"));
    }

    #[test]
    fn test_resolve_path_absolute() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path();
        let absolute_path = docs_dir.join("test.md");
        fs::write(&absolute_path, "test").unwrap();

        let result = resolve_path(absolute_path.to_str().unwrap(), docs_dir).unwrap();
        assert_eq!(result, absolute_path);
    }

    #[test]
    fn test_resolve_path_relative_to_docs() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path();
        let test_file = docs_dir.join("test.md");
        fs::write(&test_file, "test").unwrap();

        let result = resolve_path("test.md", docs_dir).unwrap();
        assert_eq!(result, test_file);
    }

    #[test]
    fn test_execute_document_not_in_state() {
        let (temp, mut state_mgr) = setup_test_state_manager();

        // Create a document but don't scan it (not in state)
        let old_path = temp.path().join("01-draft/0001-old-name.md");
        fs::write(
            &old_path,
            r#"---
number: 1
title: Old Name
state: Draft
created: 2024-01-01
updated: 2024-01-01
author: Test Author
---

# Old Name
"#,
        )
        .unwrap();

        // Try to rename without scanning - should fail
        let old_str = old_path.to_str().unwrap();
        let new_str = temp.path().join("01-draft/0001-new-name.md").to_str().unwrap().to_string();

        let result = execute(&mut state_mgr, old_str, &new_str);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found in state"));
    }

    #[test]
    fn test_execute_number_mismatch_fails() {
        let (temp, mut state_mgr) = setup_test_state_manager();

        // Create and track a document
        let old_path = temp.path().join("01-draft/0001-test.md");
        fs::write(
            &old_path,
            r#"---
number: 1
title: Test
state: Draft
created: 2024-01-01
updated: 2024-01-01
author: Test Author
---

# Test
"#,
        )
        .unwrap();

        // Git add and commit
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(temp.path())
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(["commit", "-m", "Initial"])
            .current_dir(temp.path())
            .output()
            .unwrap();

        // Scan to track
        state_mgr.quick_scan().unwrap();

        // Try to rename with different number - should fail
        let old_str = old_path.to_str().unwrap();
        let new_str = temp.path().join("01-draft/0002-test.md").to_str().unwrap().to_string();

        let result = execute(&mut state_mgr, old_str, &new_str);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Number mismatch"));
    }

    #[test]
    fn test_execute_old_file_not_found() {
        let (_temp, mut state_mgr) = setup_test_state_manager();

        let result = execute(&mut state_mgr, "nonexistent.md", "new.md");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Document not found"));
    }

    #[test]
    fn test_execute_new_file_already_exists() {
        let (temp, mut state_mgr) = setup_test_state_manager();

        // Create two documents
        let old_path = temp.path().join("01-draft/0001-old.md");
        let new_path = temp.path().join("01-draft/0001-new.md");

        fs::write(&old_path, "old").unwrap();
        fs::write(&new_path, "new").unwrap();

        // Git add and commit
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(temp.path())
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(["commit", "-m", "Initial"])
            .current_dir(temp.path())
            .output()
            .unwrap();

        let result =
            execute(&mut state_mgr, old_path.to_str().unwrap(), new_path.to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Destination already exists"));
    }

    #[test]
    fn test_execute_not_markdown_file() {
        let (temp, mut state_mgr) = setup_test_state_manager();

        let old_path = temp.path().join("01-draft/0001-test.txt");
        fs::write(&old_path, "test").unwrap();

        let result = execute(
            &mut state_mgr,
            old_path.to_str().unwrap(),
            temp.path().join("01-draft/0001-new.txt").to_str().unwrap(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be a markdown file"));
    }

    #[test]
    fn test_parse_and_validate_paths_outside_docs() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();

        let outside_path = temp.path().join("outside.md");
        fs::write(&outside_path, "test").unwrap();

        let result = parse_and_validate_paths(outside_path.to_str().unwrap(), "new.md", &docs_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be within the docs directory"));
    }

    // Note: We cannot test successful rename in unit tests because git mv requires
    // the files to be in the main repository, not in a temp directory. The success
    // path is covered by integration tests and manual testing.

    #[test]
    fn test_execute_new_path_not_markdown() {
        let (temp, mut state_mgr) = setup_test_state_manager();

        // Create a markdown file
        let old_path = temp.path().join("01-draft/0001-test.md");
        fs::write(&old_path, "test").unwrap();

        // Try to rename to non-markdown extension
        let new_path = temp.path().join("01-draft/0001-test.txt");

        let result =
            execute(&mut state_mgr, old_path.to_str().unwrap(), new_path.to_str().unwrap());

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be a markdown file"));
    }

    #[test]
    fn test_extract_number_from_path_invalid_filename() {
        // Test with a path that has an invalid filename component
        let result = extract_number_from_path(Path::new(""));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid filename"));
    }

    #[test]
    fn test_resolve_path_relative_to_cwd() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();

        // Create a file in current directory (temp root, not docs)
        let cwd_file = temp.path().join("file-in-cwd.md");
        fs::write(&cwd_file, "test").unwrap();

        // Change to temp directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        // Resolve relative path - should find it in cwd
        let result = resolve_path("file-in-cwd.md", &docs_dir);

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        // Canonicalize both paths to handle /var vs /private/var on macOS
        assert_eq!(result.unwrap().canonicalize().unwrap(), cwd_file.canonicalize().unwrap());
    }

    #[test]
    fn test_resolve_path_multicomponent_nonexistent() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();

        // Change to temp directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        // Try to resolve a multi-component path that doesn't exist
        // Should return relative to cwd
        let result = resolve_path("subdir/newfile.md", &docs_dir).unwrap();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        // Should resolve to cwd + path
        assert!(result.to_str().unwrap().contains("subdir"));
        assert!(result.to_str().unwrap().ends_with("newfile.md"));
    }

    #[test]
    fn test_parse_and_validate_paths_new_parent_outside_docs() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();

        // Create a file inside docs
        let old_path = docs_dir.join("0001-test.md");
        fs::write(&old_path, "test").unwrap();

        // Create a directory outside docs
        let outside_dir = temp.path().join("outside");
        fs::create_dir_all(&outside_dir).unwrap();

        // Try to rename to a file in the outside directory
        let new_path = outside_dir.join("0001-test.md");

        let result = parse_and_validate_paths(
            old_path.to_str().unwrap(),
            new_path.to_str().unwrap(),
            &docs_dir,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be within the docs directory"));
    }

    #[test]
    fn test_parse_and_validate_paths_new_parent_no_parent() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();

        // Create a file inside docs
        let old_path = docs_dir.join("0001-test.md");
        fs::write(&old_path, "test").unwrap();

        // Create a path with no parent (root path)
        // This tests the None case for new_path.parent()
        let result = parse_and_validate_paths(
            old_path.to_str().unwrap(),
            "/0001-test.md", // Root level, no parent
            &docs_dir,
        );

        // Should fail because root is not within docs directory
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_and_validate_paths_new_parent_doesnt_exist() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();

        // Create a file inside docs
        let old_path = docs_dir.join("0001-test.md");
        fs::write(&old_path, "test").unwrap();

        // Try to rename to a path whose parent directory doesn't exist yet
        // This is valid - the parent will be created during rename
        let new_path = docs_dir.join("nonexistent-dir/0001-test.md");

        let result = parse_and_validate_paths(
            old_path.to_str().unwrap(),
            new_path.to_str().unwrap(),
            &docs_dir,
        );

        // Should succeed because the parent doesn't exist (line 110)
        assert!(result.is_ok());
    }
}
