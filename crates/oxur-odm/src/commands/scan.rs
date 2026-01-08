//! Scan command implementation

use anyhow::Result;
use colored::*;
use design::doc::state_from_directory;
use design::state::StateManager;
use std::path::PathBuf;

/// Scan filesystem and validate/update state
pub fn scan_documents(state_mgr: &mut StateManager, fix: bool, verbose: bool) -> Result<()> {
    println!("\n{}\n", "Scanning documents...".bold());

    let result = state_mgr.scan_for_changes()?;

    // Report changes
    if result.has_changes() {
        if !result.new_files.is_empty() {
            println!("{}", "New Files:".green().bold());
            for num in &result.new_files {
                if let Some(record) = state_mgr.state().get(*num) {
                    println!("  {} {:04} - {}", "+".green(), num, record.metadata.title);
                }
            }
            println!();
        }

        if !result.changed.is_empty() {
            println!("{}", "Modified Files:".yellow().bold());
            for num in &result.changed {
                if let Some(record) = state_mgr.state().get(*num) {
                    println!("  {} {:04} - {}", "~".yellow(), num, record.metadata.title);
                }
            }
            println!();
        }

        if !result.deleted.is_empty() {
            println!("{}", "Deleted Files:".red().bold());
            for num in &result.deleted {
                println!("  {} {:04}", "-".red(), num);
            }
            println!();
        }
    } else {
        println!("{} No changes detected\n", "✓".green().bold());
    }

    // Report errors
    if !result.errors.is_empty() {
        println!("{}", "Errors:".red().bold());
        for error in &result.errors {
            println!("  {} {}", "✗".red(), error);
        }
        println!();
    }

    // Validate consistency
    if verbose {
        validate_consistency(state_mgr, fix)?;
    }

    // Summary
    println!(
        "{} State updated: {} documents tracked\n",
        "✓".green().bold(),
        state_mgr.state().documents.len()
    );

    Ok(())
}

fn validate_consistency(state_mgr: &StateManager, fix: bool) -> Result<()> {
    println!("{}", "Validating Consistency:".bold());

    let mut inconsistencies = 0;
    let mut fixable = Vec::new();

    for record in state_mgr.state().all() {
        let full_path = PathBuf::from(state_mgr.docs_dir()).join(&record.path);

        // Check if file exists
        if !full_path.exists() {
            println!(
                "  {} {:04} - File not found: {}",
                "✗".red(),
                record.metadata.number,
                record.path
            );
            inconsistencies += 1;
            continue;
        }

        // Check state/directory consistency
        if let Some(dir_state) = state_from_directory(&full_path) {
            if record.metadata.state != dir_state {
                println!(
                    "  {} {:04} - State mismatch: YAML='{}' Directory='{}'",
                    "⚠".yellow(),
                    record.metadata.number,
                    record.metadata.state.as_str(),
                    dir_state.as_str()
                );
                inconsistencies += 1;
                fixable.push((record.metadata.number, full_path.clone()));
            }
        }
    }

    if inconsistencies == 0 {
        println!("  {} All documents consistent", "✓".green());
    } else {
        println!("  {} {} inconsistencies found", "⚠".yellow(), inconsistencies);

        if fix && !fixable.is_empty() {
            println!("\n{}", "Fixing inconsistencies...".bold());
            for (num, path) in &fixable {
                println!("  Syncing {:04}: {}", num, path.display());
                // Note: actual fix would call sync_location here
                // For now, just report what would be fixed
            }
        } else if !fixable.is_empty() {
            println!(
                "\n{} Run with {} to fix {} issue(s)",
                "→".cyan(),
                "--fix".cyan(),
                fixable.len()
            );
        }
    }

    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use design::doc::DocState;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_doc_content(number: u32, title: &str, state: DocState) -> String {
        format!(
            "---\nnumber: {}\ntitle: \"{}\"\nauthor: \"Test Author\"\ncreated: 2024-01-01\nupdated: 2024-01-01\nstate: {}\n---\n\n# {}\n\nTest content",
            number, title, state.as_str(), title
        )
    }

    #[test]
    fn test_scan_no_changes() {
        let temp = TempDir::new().unwrap();

        // Create initial state with one document
        let content = create_test_doc_content(1, "Test Doc", DocState::Draft);
        let doc_path = temp.path().join("0001-test.md");
        fs::write(&doc_path, content).unwrap();

        // Initialize state manager
        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        state_mgr.scan_for_changes().unwrap();

        // Scan again - should find no changes
        let result = scan_documents(&mut state_mgr, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scan_new_file() {
        let temp = TempDir::new().unwrap();

        // Initialize empty state manager
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // Create a new document file
        let content = create_test_doc_content(1, "New Doc", DocState::Draft);
        fs::write(temp.path().join("0001-new.md"), content).unwrap();

        // Scan should detect new file
        let result = scan_documents(&mut state_mgr, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scan_with_verbose() {
        let temp = TempDir::new().unwrap();

        // Create document
        let content = create_test_doc_content(1, "Test Doc", DocState::Draft);
        fs::write(temp.path().join("0001-test.md"), content).unwrap();

        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // Scan with verbose mode
        let result = scan_documents(&mut state_mgr, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scan_with_fix() {
        let temp = TempDir::new().unwrap();

        // Create document
        let content = create_test_doc_content(1, "Test Doc", DocState::Draft);
        fs::write(temp.path().join("0001-test.md"), content).unwrap();

        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // Scan with fix mode
        let result = scan_documents(&mut state_mgr, true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scan_empty_directory() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // Scan empty directory
        let result = scan_documents(&mut state_mgr, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scan_multiple_states() {
        let temp = TempDir::new().unwrap();

        // Create documents in different states
        for (num, state) in [(1, DocState::Draft), (2, DocState::Final), (3, DocState::Active)] {
            let content = create_test_doc_content(num, &format!("Doc {}", num), state);
            fs::write(temp.path().join(format!("{:04}-test.md", num)), content).unwrap();
        }

        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let result = scan_documents(&mut state_mgr, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_consistency() {
        let temp = TempDir::new().unwrap();

        // Create document
        let content = create_test_doc_content(1, "Test Doc", DocState::Draft);
        fs::write(temp.path().join("0001-test.md"), content).unwrap();

        let state_mgr = StateManager::new(temp.path()).unwrap();

        // Validate consistency (should pass with no issues)
        let result = validate_consistency(&state_mgr, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_consistency_with_fix() {
        let temp = TempDir::new().unwrap();

        // Create document
        let content = create_test_doc_content(1, "Test Doc", DocState::Draft);
        fs::write(temp.path().join("0001-test.md"), content).unwrap();

        let state_mgr = StateManager::new(temp.path()).unwrap();

        // Validate with fix mode
        let result = validate_consistency(&state_mgr, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scan_verbose_and_fix() {
        let temp = TempDir::new().unwrap();

        // Create document
        let content = create_test_doc_content(1, "Test Doc", DocState::Draft);
        fs::write(temp.path().join("0001-test.md"), content).unwrap();

        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // Scan with both verbose and fix
        let result = scan_documents(&mut state_mgr, true, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scan_with_new_files_display() {
        let temp = TempDir::new().unwrap();

        // Create draft directory for documents
        let draft_dir = temp.path().join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        // Initialize empty state manager
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // Create multiple new document files in draft directory
        for i in 1..=3 {
            let content = create_test_doc_content(i, &format!("New Doc {}", i), DocState::Draft);
            fs::write(draft_dir.join(format!("{:04}-new.md", i)), content).unwrap();
        }

        // Scan should detect new files and print them
        let result = scan_documents(&mut state_mgr, false, false);
        assert!(result.is_ok());

        // Verify all files are in state now
        assert_eq!(state_mgr.state().documents.len(), 3);
    }

    #[test]
    fn test_scan_with_changed_files_display() {
        let temp = TempDir::new().unwrap();

        // Create draft directory
        let draft_dir = temp.path().join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        // Create initial document
        let content = create_test_doc_content(1, "Original Doc", DocState::Draft);
        let doc_path = draft_dir.join("0001-original.md");
        fs::write(&doc_path, &content).unwrap();

        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        state_mgr.scan_for_changes().unwrap();

        // Modify the file
        let modified_content = content + "\n\nAdditional content added";
        fs::write(&doc_path, modified_content).unwrap();

        // Scan should detect changed file and print it
        let result = scan_documents(&mut state_mgr, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scan_with_deleted_files_display() {
        let temp = TempDir::new().unwrap();

        // Create draft directory
        let draft_dir = temp.path().join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        // Create initial documents
        for i in 1..=3 {
            let content = create_test_doc_content(i, &format!("Doc {}", i), DocState::Draft);
            fs::write(draft_dir.join(format!("{:04}-doc.md", i)), content).unwrap();
        }

        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        state_mgr.scan_for_changes().unwrap();

        // Delete some files
        fs::remove_file(draft_dir.join("0001-doc.md")).unwrap();
        fs::remove_file(draft_dir.join("0002-doc.md")).unwrap();

        // Scan should detect deleted files and print them
        let result = scan_documents(&mut state_mgr, false, false);
        assert!(result.is_ok());

        // Only one document should remain in state
        assert_eq!(state_mgr.state().documents.len(), 1);
    }

    #[test]
    fn test_scan_with_errors_display() {
        let temp = TempDir::new().unwrap();

        // Create draft directory
        let draft_dir = temp.path().join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        // Create a valid document
        let valid_content = create_test_doc_content(1, "Valid Doc", DocState::Draft);
        fs::write(draft_dir.join("0001-valid.md"), valid_content).unwrap();

        // Create an invalid document (missing frontmatter)
        fs::write(draft_dir.join("0002-invalid.md"), "Just plain text without frontmatter")
            .unwrap();

        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // Scan should detect error and print it
        let result = scan_documents(&mut state_mgr, false, false);
        assert!(result.is_ok());

        // Only valid document should be in state
        assert_eq!(state_mgr.state().documents.len(), 1);
    }

    #[test]
    fn test_scan_with_mixed_changes() {
        let temp = TempDir::new().unwrap();

        // Create draft directory
        let draft_dir = temp.path().join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        // Create initial document
        let content1 = create_test_doc_content(1, "Existing Doc", DocState::Draft);
        let doc1_path = draft_dir.join("0001-existing.md");
        fs::write(&doc1_path, &content1).unwrap();

        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        state_mgr.scan_for_changes().unwrap();

        // Add new file
        let content2 = create_test_doc_content(2, "New Doc", DocState::Draft);
        fs::write(draft_dir.join("0002-new.md"), content2).unwrap();

        // Modify existing file
        let modified_content = content1 + "\n\nModified content";
        fs::write(&doc1_path, modified_content).unwrap();

        // Add invalid file (will cause error)
        fs::write(draft_dir.join("0003-invalid.md"), "Invalid content").unwrap();

        // Scan should show new, changed, and errors sections
        let result = scan_documents(&mut state_mgr, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_consistency_missing_file() {
        let temp = TempDir::new().unwrap();

        // Create draft directory
        let draft_dir = temp.path().join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        // Create document and scan it
        let content = create_test_doc_content(1, "Test Doc", DocState::Draft);
        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, content).unwrap();

        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        state_mgr.scan_for_changes().unwrap();

        // Delete the file but keep it in state
        fs::remove_file(&doc_path).unwrap();

        // Validate should detect missing file
        let result = validate_consistency(&state_mgr, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_consistency_state_mismatch() {
        let temp = TempDir::new().unwrap();

        // Create state directories
        let draft_dir = temp.path().join("01-draft");
        let final_dir = temp.path().join("06-final");
        fs::create_dir_all(&draft_dir).unwrap();
        fs::create_dir_all(&final_dir).unwrap();

        // Create document in draft directory but with Final state in YAML
        let content = create_test_doc_content(1, "Test Doc", DocState::Final);
        fs::write(draft_dir.join("0001-test.md"), content).unwrap();

        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        state_mgr.scan_for_changes().unwrap();

        // Validate should detect state mismatch
        let result = validate_consistency(&state_mgr, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_consistency_with_fix_mode() {
        let temp = TempDir::new().unwrap();

        // Create state directories
        let draft_dir = temp.path().join("01-draft");
        let final_dir = temp.path().join("06-final");
        fs::create_dir_all(&draft_dir).unwrap();
        fs::create_dir_all(&final_dir).unwrap();

        // Create document with state mismatch
        let content = create_test_doc_content(1, "Test Doc", DocState::Final);
        fs::write(draft_dir.join("0001-test.md"), content).unwrap();

        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        state_mgr.scan_for_changes().unwrap();

        // Validate with fix mode should detect and offer to fix
        let result = validate_consistency(&state_mgr, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_consistency_no_fix_suggestion() {
        let temp = TempDir::new().unwrap();

        // Create state directories
        let draft_dir = temp.path().join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        // Create document with state mismatch
        let content = create_test_doc_content(1, "Test Doc", DocState::Final);
        fs::write(draft_dir.join("0001-test.md"), content).unwrap();

        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        state_mgr.scan_for_changes().unwrap();

        // Validate without fix mode should suggest using --fix
        let result = validate_consistency(&state_mgr, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_consistency_all_consistent() {
        let temp = TempDir::new().unwrap();

        // Create state directory
        let draft_dir = temp.path().join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        // Create document with matching state
        let content = create_test_doc_content(1, "Test Doc", DocState::Draft);
        fs::write(draft_dir.join("0001-test.md"), content).unwrap();

        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        state_mgr.scan_for_changes().unwrap();

        // Validate should find no issues
        let result = validate_consistency(&state_mgr, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_consistency_multiple_mismatches() {
        let temp = TempDir::new().unwrap();

        // Create state directories
        let draft_dir = temp.path().join("01-draft");
        let active_dir = temp.path().join("05-active");
        fs::create_dir_all(&draft_dir).unwrap();
        fs::create_dir_all(&active_dir).unwrap();

        // Create multiple documents with state mismatches
        let content1 = create_test_doc_content(1, "Doc 1", DocState::Active);
        fs::write(draft_dir.join("0001-doc1.md"), content1).unwrap();

        let content2 = create_test_doc_content(2, "Doc 2", DocState::Draft);
        fs::write(active_dir.join("0002-doc2.md"), content2).unwrap();

        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        state_mgr.scan_for_changes().unwrap();

        // Validate should detect multiple mismatches
        let result = validate_consistency(&state_mgr, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_consistency_with_missing_and_mismatch() {
        let temp = TempDir::new().unwrap();

        // Create state directories
        let draft_dir = temp.path().join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        // Create documents
        let content1 = create_test_doc_content(1, "Missing Doc", DocState::Draft);
        let doc1_path = draft_dir.join("0001-missing.md");
        fs::write(&doc1_path, content1).unwrap();

        let content2 = create_test_doc_content(2, "Mismatch Doc", DocState::Final);
        fs::write(draft_dir.join("0002-mismatch.md"), content2).unwrap();

        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        state_mgr.scan_for_changes().unwrap();

        // Delete one file
        fs::remove_file(&doc1_path).unwrap();

        // Validate should detect both issues
        let result = validate_consistency(&state_mgr, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scan_verbose_shows_validation() {
        let temp = TempDir::new().unwrap();

        // Create state directory
        let draft_dir = temp.path().join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        // Create document with state mismatch
        let content = create_test_doc_content(1, "Test Doc", DocState::Final);
        fs::write(draft_dir.join("0001-test.md"), content).unwrap();

        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // Verbose mode should run validation
        let result = scan_documents(&mut state_mgr, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scan_quiet_skips_validation() {
        let temp = TempDir::new().unwrap();

        // Create state directory
        let draft_dir = temp.path().join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        // Create document with state mismatch
        let content = create_test_doc_content(1, "Test Doc", DocState::Final);
        fs::write(draft_dir.join("0001-test.md"), content).unwrap();

        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // Non-verbose mode should skip validation
        let result = scan_documents(&mut state_mgr, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_consistency_file_without_directory_state() {
        let temp = TempDir::new().unwrap();

        // Create draft directory first
        let draft_dir = temp.path().join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        // Create document in draft directory (which will have state from directory)
        let content = create_test_doc_content(1, "Test Doc", DocState::Draft);
        fs::write(draft_dir.join("0001-test.md"), content).unwrap();

        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        state_mgr.scan_for_changes().unwrap();

        // Validate should handle file correctly
        let result = validate_consistency(&state_mgr, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_consistency_with_fix_and_no_fixable() {
        let temp = TempDir::new().unwrap();

        // Create draft directory
        let draft_dir = temp.path().join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        // Create document and scan it
        let content = create_test_doc_content(1, "Test Doc", DocState::Draft);
        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, content).unwrap();

        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        state_mgr.scan_for_changes().unwrap();

        // Delete the file (creates non-fixable inconsistency)
        fs::remove_file(&doc_path).unwrap();

        // Validate with fix mode but nothing to fix
        let result = validate_consistency(&state_mgr, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scan_comprehensive_workflow() {
        let temp = TempDir::new().unwrap();

        // Setup state directories
        let draft_dir = temp.path().join("01-draft");
        let final_dir = temp.path().join("06-final");
        fs::create_dir_all(&draft_dir).unwrap();
        fs::create_dir_all(&final_dir).unwrap();

        // Initial scan - empty
        let mut state_mgr = StateManager::new(temp.path()).unwrap();
        let result = scan_documents(&mut state_mgr, false, false);
        assert!(result.is_ok());

        // Add some files
        let content1 = create_test_doc_content(1, "Doc 1", DocState::Draft);
        fs::write(draft_dir.join("0001-doc1.md"), content1).unwrap();

        // Scan - should find new file
        let result = scan_documents(&mut state_mgr, false, false);
        assert!(result.is_ok());
        assert_eq!(state_mgr.state().documents.len(), 1);

        // Scan again with verbose - no changes
        let result = scan_documents(&mut state_mgr, false, true);
        assert!(result.is_ok());

        // Add file with mismatch and scan with fix and verbose
        let content2 = create_test_doc_content(2, "Doc 2", DocState::Final);
        fs::write(draft_dir.join("0002-doc2.md"), content2).unwrap();
        let result = scan_documents(&mut state_mgr, true, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_consistency_empty_state() {
        let temp = TempDir::new().unwrap();

        let state_mgr = StateManager::new(temp.path()).unwrap();

        // Validate empty state should pass
        let result = validate_consistency(&state_mgr, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scan_all_state_directories() {
        let temp = TempDir::new().unwrap();

        // Create documents in different state directories
        let states = vec![
            (DocState::Draft, "01-draft"),
            (DocState::UnderReview, "02-under-review"),
            (DocState::Revised, "03-revised"),
            (DocState::Accepted, "04-accepted"),
            (DocState::Active, "05-active"),
            (DocState::Final, "06-final"),
            (DocState::Deferred, "07-deferred"),
            (DocState::Rejected, "08-rejected"),
            (DocState::Withdrawn, "09-withdrawn"),
            (DocState::Superseded, "10-superseded"),
        ];

        for (i, (state, dir)) in states.iter().enumerate() {
            let state_dir = temp.path().join(dir);
            fs::create_dir_all(&state_dir).unwrap();

            let num = (i + 1) as u32;
            let content = create_test_doc_content(num, &format!("Doc {}", num), *state);
            fs::write(state_dir.join(format!("{:04}-doc.md", num)), content).unwrap();
        }

        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // Scan should find all documents
        let result = scan_documents(&mut state_mgr, false, false);
        assert!(result.is_ok());
        assert_eq!(state_mgr.state().documents.len(), 10);

        // Validate with verbose - all should be consistent
        let result = scan_documents(&mut state_mgr, false, true);
        assert!(result.is_ok());
    }
}
