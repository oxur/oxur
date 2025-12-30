//! State transition command implementation

use anyhow::{Context, Result};
use colored::*;
use design::doc::{DesignDoc, DocState};
use design::index::DocumentIndex;
use design::state::StateManager;
use std::fs;
use std::path::PathBuf;

/// Transition a document to a new state
pub fn transition_document(
    index: &DocumentIndex,
    state_mgr: &StateManager,
    doc_number_or_path: &str,
    new_state_str: &str,
) -> Result<()> {
    // Try to resolve document number or path
    let path = if let Ok(doc_number) = state_mgr.resolve_number_or_path(doc_number_or_path) {
        // Get the document from state
        let doc_record = state_mgr
            .state()
            .get(doc_number)
            .ok_or_else(|| anyhow::anyhow!("Document {} not found", doc_number))?;

        // Build full path to the document
        state_mgr.docs_dir().join(&doc_record.path)
    } else {
        // If resolution fails, treat as direct path (for documents without headers)
        PathBuf::from(doc_number_or_path)
    };

    // Validate file exists
    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
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

    // Parse document to get current state
    let doc = DesignDoc::parse(&content, path.clone()).context("Failed to parse document")?;

    let current_state = doc.metadata.state;

    // Parse new state
    let new_state = DocState::from_str_flexible(new_state_str).ok_or_else(|| {
        let valid_states = DocState::all_state_names().join(", ");
        anyhow::anyhow!("Unsupported state '{}'. Valid states are: {}", new_state_str, valid_states)
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
    let filename = path.file_name().ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;

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

    // Update the index to reflect the state change
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
            component: None,
            tags: Vec::new(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            state: DocState::Draft,
            supersedes: None,
            superseded_by: None,
            version: "1.0".to_string(),
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
    fn test_transition_file_not_found() {
        let temp = TempDir::new().unwrap();
        let index = create_test_index(&temp);
        let state_mgr = StateManager::new(temp.path()).unwrap();

        let result = transition_document(&index, &state_mgr, "/nonexistent/file.md", "Final");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_transition_invalid_state() {
        let temp = TempDir::new().unwrap();
        let index = create_test_index(&temp);

        // Create a document in proper state directory
        let draft_dir = temp.path().join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();
        let doc_path = draft_dir.join("test.md");
        let content = create_test_doc_with_state(DocState::Draft);
        fs::write(&doc_path, content).unwrap();

        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        state_mgr.quick_scan().unwrap();

        let result = transition_document(&index, &state_mgr, "1", "InvalidState");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported state"));
    }

    #[test]
    fn test_transition_already_in_state() {
        let temp = TempDir::new().unwrap();
        let index = create_test_index(&temp);

        // Create a document in Draft state in proper directory
        let draft_dir = temp.path().join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();
        let doc_path = draft_dir.join("test.md");
        let content = create_test_doc_with_state(DocState::Draft);
        fs::write(&doc_path, content).unwrap();

        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        state_mgr.quick_scan().unwrap();

        // Try to transition to same state
        let result = transition_document(&index, &state_mgr, "1", "Draft");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already in state"));
    }

    #[test]
    #[serial]
    fn test_transition_draft_to_final() {
        let temp = TempDir::new().unwrap();
        let repo_path = setup_git_repo(&temp);
        let index = create_test_index(&temp);

        // Create draft directory and document
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

        // Create StateManager after files are set up
        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        state_mgr.quick_scan().unwrap();

        // Transition to Final (must run in repo directory for git mv)
        let result = in_dir(&repo_path, || transition_document(&index, &state_mgr, "1", "Final"));
        assert!(result.is_ok());

        // Verify file was moved
        assert!(!doc_path.exists());

        let final_dir = repo_path.join("06-final");
        let new_path = final_dir.join("test.md");
        assert!(new_path.exists());

        // Verify state was updated in content
        let new_content = fs::read_to_string(&new_path).unwrap();
        assert!(new_content.contains("state: Final"));
    }

    #[test]
    #[serial]
    fn test_transition_updates_yaml_frontmatter() {
        let temp = TempDir::new().unwrap();
        let repo_path = setup_git_repo(&temp);
        let index = create_test_index(&temp);

        // Create document in Active state
        let active_dir = repo_path.join("05-active");
        fs::create_dir_all(&active_dir).unwrap();

        let doc_path = active_dir.join("test.md");
        let content = create_test_doc_with_state(DocState::Active);
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

        // Create StateManager after files are set up
        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        state_mgr.quick_scan().unwrap();

        // Transition to Superseded (must run in repo directory for git mv)
        let result =
            in_dir(&repo_path, || transition_document(&index, &state_mgr, "1", "Superseded"));
        assert!(result.is_ok());

        // Verify new file has updated state
        let superseded_dir = repo_path.join("10-superseded");
        let new_path = superseded_dir.join("test.md");

        let new_content = fs::read_to_string(&new_path).unwrap();
        assert!(new_content.contains("state: Superseded"));
        assert!(!new_content.contains("state: Active"));
    }

    #[test]
    #[serial]
    fn test_transition_document_without_headers() {
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

        // Create StateManager after files are set up
        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        state_mgr.quick_scan().unwrap();

        // Try to transition - should add headers automatically (must run in repo directory)
        // Use file path since document without headers won't be in state with a number
        let result = in_dir(&repo_path, || {
            transition_document(&index, &state_mgr, doc_path.to_str().unwrap(), "UnderReview")
        });
        assert!(result.is_ok());

        // Verify headers were added and file was moved
        let under_review_dir = repo_path.join("02-under-review");
        let new_path = under_review_dir.join("test.md");
        assert!(new_path.exists());

        let new_content = fs::read_to_string(&new_path).unwrap();
        assert!(new_content.contains("---"));
        assert!(new_content.contains("state: Under Review"));
    }

    #[test]
    #[serial]
    fn test_transition_creates_target_directory() {
        let temp = TempDir::new().unwrap();
        let repo_path = setup_git_repo(&temp);
        let index = create_test_index(&temp);

        // Create document in Draft state
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

        // Create StateManager after files are set up
        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        state_mgr.quick_scan().unwrap();

        // Target directory shouldn't exist
        let deferred_dir = repo_path.join("07-deferred");
        assert!(!deferred_dir.exists());

        // Transition to Deferred (must run in repo directory for git mv)
        let result =
            in_dir(&repo_path, || transition_document(&index, &state_mgr, "1", "Deferred"));
        assert!(result.is_ok());

        // Verify directory was created
        assert!(deferred_dir.exists());
        let new_path = deferred_dir.join("test.md");
        assert!(new_path.exists());
    }

    #[test]
    #[serial]
    fn test_transition_multiple_states() {
        let temp = TempDir::new().unwrap();
        let repo_path = setup_git_repo(&temp);
        let index = create_test_index(&temp);

        // Create document
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

        // Create StateManager after files are set up
        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        state_mgr.quick_scan().unwrap();

        // Transition Draft -> UnderReview (must run in repo directory for git mv)
        let result =
            in_dir(&repo_path, || transition_document(&index, &state_mgr, "1", "UnderReview"));
        assert!(result.is_ok());

        let under_review_dir = repo_path.join("02-under-review");
        let path2 = under_review_dir.join("test.md");
        assert!(path2.exists());

        // Rescan to update state after first transition
        state_mgr.quick_scan().unwrap();

        // Transition UnderReview -> Accepted
        let result =
            in_dir(&repo_path, || transition_document(&index, &state_mgr, "1", "Accepted"));
        assert!(result.is_ok());

        let accepted_dir = repo_path.join("04-accepted");
        let path3 = accepted_dir.join("test.md");
        assert!(path3.exists());

        // Verify final state
        let final_content = fs::read_to_string(&path3).unwrap();
        assert!(final_content.contains("state: Accepted"));
    }
}
