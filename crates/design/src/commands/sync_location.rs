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
        println!("{}", "Document missing headers, adding them automatically...".yellow());
        let (new_content, _) = design::doc::add_missing_headers(&path, &content)?;
        fs::write(&path, &new_content).context("Failed to write headers")?;
        new_content
    } else {
        content
    };

    // Parse document to get header state
    let doc = DesignDoc::parse(&content, path.clone()).context("Failed to parse document")?;

    let header_state = doc.metadata.state;

    // Determine target directory from state
    let target_dir = PathBuf::from(index.docs_dir()).join(header_state.directory());

    // Check current directory
    let current_dir =
        path.parent().ok_or_else(|| anyhow::anyhow!("Cannot determine current directory"))?;

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
    let filename = path.file_name().ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
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

    // Update the index to reflect the location sync
    println!();
    if let Err(e) = crate::commands::update_index::update_index(index) {
        println!("{} Failed to update index", "Warning:".yellow());
        println!("  {}", e);
        println!("  Run 'oxd update-index' manually to sync the index");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use design::doc::{DocMetadata, DocState};
    use design::index::DocumentIndex;
    use design::state::{DocumentRecord, DocumentState};
    use serial_test::serial;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_doc_with_state(state: DocState) -> String {
        format!(
            r#"---
number: 1
title: "Test Document"
author: "Test Author"
created: 2024-01-01
updated: 2024-01-01
state: {}
---

# Test Document

Test content.
"#,
            state.as_str()
        )
    }

    fn setup_git_repo(temp: &TempDir) -> PathBuf {
        let repo_path = temp.path().to_path_buf();

        // Initialize git repo
        std::process::Command::new("git").arg("init").current_dir(&repo_path).output().unwrap();

        std::process::Command::new("git")
            .args(&["config", "user.name", "Test User"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(&["config", "user.email", "test@example.com"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        repo_path
    }

    fn create_test_index(temp: &TempDir) -> DocumentIndex {
        let mut state = DocumentState::new();

        let meta = DocMetadata {
            number: 1,
            title: "Test Document".to_string(),
            author: "Test Author".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            state: DocState::Draft,
            supersedes: None,
            superseded_by: None,
        };
        state.upsert(
            1,
            DocumentRecord {
                metadata: meta,
                path: "0001-test.md".to_string(),
                checksum: "abc123".to_string(),
                file_size: 100,
                modified: chrono::Utc::now(),
            },
        );

        DocumentIndex::from_state(&state, temp.path()).unwrap()
    }

    /// Helper to run code in a directory and restore the original directory afterward
    fn in_dir<F, R>(dir: &std::path::Path, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let original_dir = std::env::current_dir().ok();
        std::env::set_current_dir(dir).unwrap();
        let result = f();

        if let Some(orig) = original_dir {
            let _ = std::env::set_current_dir(orig);
        }

        result
    }

    #[test]
    fn test_sync_location_file_not_found() {
        let temp = TempDir::new().unwrap();
        let index = create_test_index(&temp);

        let result = sync_location(&index, "/nonexistent/file.md");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("File not found"));
    }

    #[test]
    #[serial]
    fn test_sync_location_already_in_correct_location() {
        let temp = TempDir::new().unwrap();
        let repo_path = setup_git_repo(&temp);
        let index = create_test_index(&temp);

        // Create document in Draft directory with Draft state
        let draft_dir = repo_path.join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        let doc_path = draft_dir.join("test.md");
        let content = create_test_doc_with_state(DocState::Draft);
        fs::write(&doc_path, &content).unwrap();

        // Add to git
        std::process::Command::new("git")
            .args(&["add", "."])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(&["commit", "-m", "Initial commit"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        // Sync location - should report already in correct place (must run in repo directory)
        let result = in_dir(&repo_path, || sync_location(&index, doc_path.to_str().unwrap()));
        assert!(result.is_ok());

        // File should still be in same place
        assert!(doc_path.exists());
    }

    #[test]
    #[serial]
    fn test_sync_location_moves_to_match_header() {
        let temp = TempDir::new().unwrap();
        let repo_path = setup_git_repo(&temp);
        let index = create_test_index(&temp);

        // Create document in Draft directory but with Final state in header
        let draft_dir = repo_path.join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        let doc_path = draft_dir.join("test.md");
        let content = create_test_doc_with_state(DocState::Final);
        fs::write(&doc_path, &content).unwrap();

        // Add to git
        std::process::Command::new("git")
            .args(&["add", "."])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(&["commit", "-m", "Initial commit"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        // Sync location - should move to Final directory (must run in repo directory)
        let result = in_dir(&repo_path, || sync_location(&index, doc_path.to_str().unwrap()));
        assert!(result.is_ok());

        // File should be moved
        assert!(!doc_path.exists());

        let final_dir = repo_path.join("06-final");
        let new_path = final_dir.join("test.md");
        assert!(new_path.exists());
    }

    #[test]
    #[serial]
    fn test_sync_location_document_without_headers() {
        let temp = TempDir::new().unwrap();
        let repo_path = setup_git_repo(&temp);
        let index = create_test_index(&temp);

        // Create document without headers
        let draft_dir = repo_path.join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        let doc_path = draft_dir.join("test.md");
        let content = "# Test Document\n\nNo headers here.\n";
        fs::write(&doc_path, content).unwrap();

        // Add to git
        std::process::Command::new("git")
            .args(&["add", "."])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(&["commit", "-m", "Initial commit"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        // Sync location - should add headers automatically (must run in repo directory)
        let result = in_dir(&repo_path, || sync_location(&index, doc_path.to_str().unwrap()));
        assert!(result.is_ok());

        // Verify headers were added (document should remain in draft after adding headers)
        let updated_content = fs::read_to_string(&doc_path).unwrap();
        assert!(updated_content.contains("---"));
        assert!(updated_content.contains("state:"));
    }

    #[test]
    #[serial]
    fn test_sync_location_creates_target_directory() {
        let temp = TempDir::new().unwrap();
        let repo_path = setup_git_repo(&temp);
        let index = create_test_index(&temp);

        // Create document in Draft directory with Rejected state
        let draft_dir = repo_path.join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        let doc_path = draft_dir.join("test.md");
        let content = create_test_doc_with_state(DocState::Rejected);
        fs::write(&doc_path, &content).unwrap();

        // Add to git
        std::process::Command::new("git")
            .args(&["add", "."])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(&["commit", "-m", "Initial commit"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        // Target directory shouldn't exist yet
        let rejected_dir = repo_path.join("08-rejected");
        assert!(!rejected_dir.exists());

        // Sync location - should create target directory (must run in repo directory)
        let result = in_dir(&repo_path, || sync_location(&index, doc_path.to_str().unwrap()));
        assert!(result.is_ok());

        // Verify directory was created and file was moved
        assert!(rejected_dir.exists());
        let new_path = rejected_dir.join("test.md");
        assert!(new_path.exists());
    }

    #[test]
    #[serial]
    fn test_sync_location_preserves_content() {
        let temp = TempDir::new().unwrap();
        let repo_path = setup_git_repo(&temp);
        let index = create_test_index(&temp);

        // Create document with specific content
        let draft_dir = repo_path.join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        let doc_path = draft_dir.join("test.md");
        let mut content = create_test_doc_with_state(DocState::Active);
        content.push_str("\n## Additional Section\n\nImportant content here.\n");
        fs::write(&doc_path, &content).unwrap();

        // Add to git
        std::process::Command::new("git")
            .args(&["add", "."])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(&["commit", "-m", "Initial commit"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        // Sync location (must run in repo directory for git mv)
        let result = in_dir(&repo_path, || sync_location(&index, doc_path.to_str().unwrap()));
        assert!(result.is_ok());

        // Verify content is preserved
        let active_dir = repo_path.join("05-active");
        let new_path = active_dir.join("test.md");

        let new_content = fs::read_to_string(&new_path).unwrap();
        assert!(new_content.contains("## Additional Section"));
        assert!(new_content.contains("Important content here."));
    }

    #[test]
    #[serial]
    fn test_sync_location_different_states() {
        let temp = TempDir::new().unwrap();
        let repo_path = setup_git_repo(&temp);
        let index = create_test_index(&temp);

        // Test multiple state mismatches
        for (dir_state, header_state) in [
            (DocState::Draft, DocState::UnderReview),
            (DocState::Active, DocState::Superseded),
            (DocState::Accepted, DocState::Final),
        ] {
            // Create directory for current location
            let current_dir = repo_path.join(dir_state.directory());
            fs::create_dir_all(&current_dir).unwrap();

            let doc_path = current_dir.join(format!("test-{}.md", header_state.as_str()));
            let content = create_test_doc_with_state(header_state);
            fs::write(&doc_path, &content).unwrap();

            // Add to git
            std::process::Command::new("git")
                .args(&["add", "."])
                .current_dir(&repo_path)
                .output()
                .unwrap();

            std::process::Command::new("git")
                .args(&["commit", "-m", "Add document"])
                .current_dir(&repo_path)
                .output()
                .unwrap();

            // Sync location (must run in repo directory for git mv)
            let result = in_dir(&repo_path, || sync_location(&index, doc_path.to_str().unwrap()));
            assert!(
                result.is_ok(),
                "Failed to sync {} to {}",
                dir_state.as_str(),
                header_state.as_str()
            );

            // Verify moved to correct directory
            let target_dir = repo_path.join(header_state.directory());
            let new_path = target_dir.join(format!("test-{}.md", header_state.as_str()));
            assert!(new_path.exists(), "Document not found at {}", new_path.display());
        }
    }
}
