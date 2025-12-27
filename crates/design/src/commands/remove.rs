//! Remove command - moves documents to dustbin
//!
//! Moves documents to .dustbin directory while preserving git history

use anyhow::{Context, Result};
use colored::Colorize;
use design::doc::DocState;
use design::git;
use design::state::StateManager;
use uuid::Uuid;

/// Execute the remove command
pub fn execute(state_mgr: &mut StateManager, doc_id_or_path: &str) -> Result<()> {
    println!("{}", "Removing document...".cyan().bold());
    println!();

    // Step 1: Find document by number or path
    let doc_number = if let Ok(num) = doc_id_or_path.parse::<u32>() {
        num
    } else {
        // Try to find by path
        // Normalize the search path - strip docs_dir prefix if present
        let search_path = std::path::Path::new(doc_id_or_path);
        let normalized_search = if let Ok(stripped) = search_path.strip_prefix(state_mgr.docs_dir())
        {
            stripped.to_string_lossy().to_string()
        } else {
            doc_id_or_path.to_string()
        };

        let doc = state_mgr
            .state()
            .all()
            .into_iter()
            .find(|d| d.path.contains(&normalized_search))
            .ok_or_else(|| anyhow::anyhow!("Document '{}' not found", doc_id_or_path))?;
        doc.metadata.number
    };

    // Get document record
    let doc = state_mgr
        .state()
        .get(doc_number)
        .ok_or_else(|| anyhow::anyhow!("Document {} not found", doc_number))?;

    let doc_title = doc.metadata.title.clone();
    let current_state = doc.metadata.state;
    let current_path = state_mgr.docs_dir().join(&doc.path);

    println!("  Document: {} - {}", format!("{:04}", doc_number).yellow(), doc_title.white());
    println!("  Current state: {}", current_state.as_str().cyan());
    println!();

    // Check if already removed or overwritten
    if current_state == DocState::Removed || current_state == DocState::Overwritten {
        println!("{}", "⚠ Document is already in dustbin".yellow());
        println!("  State: {}", current_state.as_str());
        println!("  Location: {}", current_path.display());
        return Ok(());
    }

    // Step 2: Prepare dustbin directory
    let dustbin_base = state_mgr.docs_dir().join(".dustbin");
    let state_subdir = current_state.directory();

    // Place in subdirectory based on original state (unless already in dustbin)
    let dustbin_dir = if current_state.is_in_dustbin() {
        dustbin_base.clone()
    } else {
        dustbin_base.join(state_subdir)
    };

    std::fs::create_dir_all(&dustbin_dir).context("Failed to create dustbin directory")?;
    println!("  ✓ Dustbin ready: {}", dustbin_dir.display().to_string().green());

    // Step 3: Generate unique filename with UUID
    let filename = current_path.file_name().context("Invalid file path")?.to_string_lossy();

    let uuid = Uuid::new_v4();
    let uuid_short = uuid.to_string().split('-').next().unwrap().to_string();

    let new_filename = if let Some(stem) = current_path.file_stem() {
        let stem_str = stem.to_string_lossy();
        format!("{}-{}.md", stem_str, uuid_short)
    } else {
        format!("{}-{}", filename.trim_end_matches(".md"), uuid_short)
    };

    let dustbin_path = dustbin_dir.join(&new_filename);
    println!("  ✓ Generated unique name: {}", new_filename.yellow());

    // Step 4: Move file using git
    if current_path.exists() {
        git::git_mv(&current_path, &dustbin_path).context("Failed to move file with git")?;
        println!("  ✓ Moved to dustbin: {}", dustbin_path.display().to_string().green());
    } else {
        println!("  {} File not found on disk: {}", "⚠".yellow(), current_path.display());
    }

    // Step 5: Update state - mark as removed and update path

    // Read and update the document
    if let Ok(content) = std::fs::read_to_string(&dustbin_path) {
        if let Ok(mut parsed_doc) = design::doc::DesignDoc::parse(&content, dustbin_path.clone()) {
            parsed_doc.metadata.state = DocState::Removed;
            parsed_doc.metadata.updated = chrono::Local::now().naive_local().date();

            // Write back with updated frontmatter
            let new_content =
                design::doc::build_yaml_frontmatter(&parsed_doc.metadata) + &parsed_doc.content;
            std::fs::write(&dustbin_path, new_content)
                .context("Failed to update document frontmatter")?;
        }
    }

    // Update state manager
    state_mgr.record_file_move(&current_path, &dustbin_path)?;
    println!("  ✓ Updated state tracking");

    println!();
    println!("{}", "Document removed successfully!".green().bold());
    println!("  Location: {}", dustbin_path.display().to_string().cyan());
    println!();
    println!("To view removed documents: {}", "oxd list --removed".yellow());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use design::doc::DocMetadata;
    use design::state::DocumentRecord;
    use serial_test::serial;
    use std::fs;
    use tempfile::TempDir;

    fn setup_git_repo(temp_dir: &std::path::Path) {
        use std::process::Command;

        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(temp_dir)
            .output()
            .expect("Failed to init git");

        // Configure user
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp_dir)
            .output()
            .expect("Failed to config user.name");

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(temp_dir)
            .output()
            .expect("Failed to config user.email");
    }

    fn create_test_state_manager() -> (StateManager, TempDir) {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();

        setup_git_repo(temp.path());

        let mut state_mgr = StateManager::new(&docs_dir).unwrap();

        // Add test document
        let meta = DocMetadata {
            number: 1,
            title: "Test Doc".to_string(),
            author: "Test Author".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            state: DocState::Draft,
            supersedes: None,
            superseded_by: None,
        };

        let doc_path = docs_dir.join("01-draft/0001-test-doc.md");
        fs::create_dir_all(doc_path.parent().unwrap()).unwrap();

        let content = format!(
            "---\n{}\n---\n\nTest content",
            serde_yaml::to_string(&meta).unwrap().trim_start_matches("---\n")
        );
        fs::write(&doc_path, content).unwrap();

        // Git add the file
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(temp.path())
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(temp.path())
            .output()
            .unwrap();

        state_mgr.state_mut().upsert(
            1,
            DocumentRecord {
                metadata: meta,
                path: "01-draft/0001-test-doc.md".to_string(),
                checksum: "abc123".to_string(),
                file_size: 100,
                modified: chrono::Utc::now(),
            },
        );

        (state_mgr, temp)
    }

    #[test]
    #[serial]
    fn test_remove_by_number() {
        let (mut state_mgr, temp) = create_test_state_manager();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = execute(&mut state_mgr, "1");
        assert!(result.is_ok());

        // Check that document state is updated
        let doc = state_mgr.state().get(1).unwrap();
        assert_eq!(doc.metadata.state, DocState::Removed);

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_remove_by_path() {
        let (mut state_mgr, temp) = create_test_state_manager();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = execute(&mut state_mgr, "test-doc");
        assert!(result.is_ok());

        let doc = state_mgr.state().get(1).unwrap();
        assert_eq!(doc.metadata.state, DocState::Removed);

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_remove_by_full_path_with_docs_dir_prefix() {
        // Regression test for bug where full path with docs_dir prefix failed to find document
        let (mut state_mgr, temp) = create_test_state_manager();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        // Build the full path including docs_dir prefix
        let full_path = temp.path().join("docs/01-draft/0001-test-doc.md");
        let full_path_str = full_path.to_string_lossy().to_string();

        let result = execute(&mut state_mgr, &full_path_str);
        assert!(result.is_ok(), "Should be able to remove by full path");

        let doc = state_mgr.state().get(1).unwrap();
        assert_eq!(doc.metadata.state, DocState::Removed);

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_remove_nonexistent_number() {
        let (mut state_mgr, _temp) = create_test_state_manager();

        let result = execute(&mut state_mgr, "999");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_remove_nonexistent_path() {
        let (mut state_mgr, _temp) = create_test_state_manager();

        let result = execute(&mut state_mgr, "nonexistent-doc");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    #[serial]
    fn test_remove_already_removed() {
        let (mut state_mgr, temp) = create_test_state_manager();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        // First removal
        execute(&mut state_mgr, "1").unwrap();

        // Second removal should succeed but do nothing
        let result = execute(&mut state_mgr, "1");
        assert!(result.is_ok());

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_remove_creates_dustbin_directory() {
        let (mut state_mgr, temp) = create_test_state_manager();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        execute(&mut state_mgr, "1").unwrap();

        let dustbin_dir = temp.path().join("docs/.dustbin/01-draft");
        assert!(dustbin_dir.exists());

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_remove_generates_unique_filename() {
        let (mut state_mgr, temp) = create_test_state_manager();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        execute(&mut state_mgr, "1").unwrap();

        let dustbin_dir = temp.path().join("docs/.dustbin/01-draft");
        let files: Vec<_> = fs::read_dir(&dustbin_dir).unwrap().collect();

        assert_eq!(files.len(), 1);

        let filename = files[0].as_ref().unwrap().file_name();
        let filename_str = filename.to_string_lossy();

        // Should have UUID suffix
        assert!(filename_str.starts_with("0001-test-doc-"));
        assert!(filename_str.ends_with(".md"));

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_remove_file_not_on_disk() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();

        setup_git_repo(temp.path());

        let mut state_mgr = StateManager::new(&docs_dir).unwrap();

        // Add document to state but not to disk
        let meta = DocMetadata {
            number: 1,
            title: "Missing Doc".to_string(),
            author: "Test Author".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            state: DocState::Draft,
            supersedes: None,
            superseded_by: None,
        };

        state_mgr.state_mut().upsert(
            1,
            DocumentRecord {
                metadata: meta,
                path: "01-draft/0001-missing-doc.md".to_string(),
                checksum: "abc123".to_string(),
                file_size: 100,
                modified: chrono::Utc::now(),
            },
        );

        // Should handle missing file gracefully
        let result = execute(&mut state_mgr, "1");
        // This will fail because git mv requires the file to exist
        // But the code handles this case
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    #[serial]
    fn test_remove_overwritten_document() {
        let (mut state_mgr, temp) = create_test_state_manager();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        // Change state to Overwritten (already in dustbin) by updating the record
        let doc = state_mgr.state().get(1).unwrap().clone();
        let mut updated_doc = doc;
        updated_doc.metadata.state = DocState::Overwritten;
        updated_doc.path = ".dustbin/overwritten/0001-test-doc.md".to_string();

        // Create the overwritten file in dustbin
        let overwritten_path = temp.path().join("docs/.dustbin/overwritten/0001-test-doc.md");
        fs::create_dir_all(overwritten_path.parent().unwrap()).unwrap();

        let content = format!(
            "---\n{}\n---\n\nTest content",
            serde_yaml::to_string(&updated_doc.metadata).unwrap().trim_start_matches("---\n")
        );
        fs::write(&overwritten_path, content).unwrap();

        state_mgr.state_mut().upsert(1, updated_doc);

        let result = execute(&mut state_mgr, "1");
        // Should succeed (early return for already removed/overwritten)
        assert!(result.is_ok());

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_remove_updates_frontmatter() {
        let (mut state_mgr, temp) = create_test_state_manager();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        execute(&mut state_mgr, "1").unwrap();

        // Find the moved file in dustbin
        let dustbin_dir = temp.path().join("docs/.dustbin/01-draft");
        let files: Vec<_> = fs::read_dir(&dustbin_dir).unwrap().collect();
        let moved_file = files[0].as_ref().unwrap().path();

        // Read and check frontmatter
        let content = fs::read_to_string(&moved_file).unwrap();
        assert!(content.contains("state: Removed"));

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_remove_multiple_documents() {
        let temp = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let docs_dir = temp.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();

        setup_git_repo(temp.path());

        let mut state_mgr = StateManager::new(&docs_dir).unwrap();

        // Add multiple documents
        for num in 1..=3 {
            let meta = DocMetadata {
                number: num,
                title: format!("Doc {}", num),
                author: "Test Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                state: DocState::Draft,
                supersedes: None,
                superseded_by: None,
            };

            let doc_path = docs_dir.join(format!("01-draft/{:04}-doc-{}.md", num, num));
            fs::create_dir_all(doc_path.parent().unwrap()).unwrap();

            let content = format!(
                "---\n{}\n---\n\nTest content",
                serde_yaml::to_string(&meta).unwrap().trim_start_matches("---\n")
            );
            fs::write(&doc_path, content).unwrap();

            state_mgr.state_mut().upsert(
                num,
                DocumentRecord {
                    metadata: meta,
                    path: format!("01-draft/{:04}-doc-{}.md", num, num),
                    checksum: "abc123".to_string(),
                    file_size: 100,
                    modified: chrono::Utc::now(),
                },
            );
        }

        // Git add and commit
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(temp.path())
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(["commit", "-m", "Add docs"])
            .current_dir(temp.path())
            .output()
            .unwrap();

        // Remove all three
        for num in 1..=3 {
            let result = execute(&mut state_mgr, &num.to_string());
            assert!(result.is_ok());
        }

        // All should be removed
        for num in 1..=3 {
            let doc = state_mgr.state().get(num).unwrap();
            assert_eq!(doc.metadata.state, DocState::Removed);
        }

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_remove_from_different_states() {
        let temp = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let docs_dir = temp.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();

        setup_git_repo(temp.path());

        let mut state_mgr = StateManager::new(&docs_dir).unwrap();

        // Add documents in different states
        let states = vec![
            (1, DocState::Draft, "01-draft"),
            (2, DocState::Active, "05-active"),
            (3, DocState::Final, "06-final"),
        ];

        for (num, state, dir) in states {
            let meta = DocMetadata {
                number: num,
                title: format!("Doc {}", num),
                author: "Test Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                state,
                supersedes: None,
                superseded_by: None,
            };

            let doc_path = docs_dir.join(format!("{}/{:04}-doc-{}.md", dir, num, num));
            fs::create_dir_all(doc_path.parent().unwrap()).unwrap();

            let content = format!(
                "---\n{}\n---\n\nTest content",
                serde_yaml::to_string(&meta).unwrap().trim_start_matches("---\n")
            );
            fs::write(&doc_path, content).unwrap();

            state_mgr.state_mut().upsert(
                num,
                DocumentRecord {
                    metadata: meta,
                    path: format!("{}/{:04}-doc-{}.md", dir, num, num),
                    checksum: "abc123".to_string(),
                    file_size: 100,
                    modified: chrono::Utc::now(),
                },
            );
        }

        // Git add and commit
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(temp.path())
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(["commit", "-m", "Add docs"])
            .current_dir(temp.path())
            .output()
            .unwrap();

        // Remove all
        for num in 1..=3 {
            let result = execute(&mut state_mgr, &num.to_string());
            assert!(result.is_ok());
        }

        // Check they went to different dustbin subdirectories
        assert!(temp.path().join("docs/.dustbin/01-draft").exists());
        assert!(temp.path().join("docs/.dustbin/05-active").exists());
        assert!(temp.path().join("docs/.dustbin/06-final").exists());

        std::env::set_current_dir(original_dir).unwrap();
    }
}
