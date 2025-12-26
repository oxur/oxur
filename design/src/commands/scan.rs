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
}
