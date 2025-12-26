//! List command implementation

use anyhow::Result;
use colored::*;
use design::doc::DocState;
use design::index::DocumentIndex;
use design::state::StateManager;
use design::theme;

#[allow(dead_code)]
pub fn list_documents(
    index: &DocumentIndex,
    state_filter: Option<String>,
    verbose: bool,
) -> Result<()> {
    list_documents_impl(index, None, state_filter, verbose, false)
}

pub fn list_documents_with_state(
    index: &DocumentIndex,
    state_mgr: Option<&StateManager>,
    state_filter: Option<String>,
    verbose: bool,
    removed: bool,
) -> Result<()> {
    list_documents_impl(index, state_mgr, state_filter, verbose, removed)
}

fn list_documents_impl(
    index: &DocumentIndex,
    state_mgr: Option<&StateManager>,
    state_filter: Option<String>,
    verbose: bool,
    removed: bool,
) -> Result<()> {
    // If showing removed documents, we need special handling
    if removed {
        if let Some(mgr) = state_mgr {
            return list_removed_documents(mgr, verbose);
        } else {
            eprintln!(
                "{} Cannot list removed documents without state manager",
                "ERROR:".red().bold()
            );
            return Ok(());
        }
    }
    let docs = if let Some(state_str) = state_filter {
        match DocState::from_str_flexible(&state_str) {
            Some(state) => index.by_state(state),
            None => {
                eprintln!("{} Unknown state: {}", "ERROR:".red().bold(), state_str);
                eprintln!("Valid states: {}", DocState::all_state_names().join(", "));
                return Ok(());
            }
        }
    } else {
        index.all()
    };

    println!("\n{}", "Design Documents".bold().underline());
    println!();

    for doc in &docs {
        let state = doc.metadata.state.as_str();

        if verbose {
            println!(
                "{} {} [{}]",
                theme::doc_number(doc.metadata.number),
                doc.metadata.title,
                theme::state_badge(state)
            );
            println!("  Author: {}", doc.metadata.author);
            println!("  Created: {} | Updated: {}", doc.metadata.created, doc.metadata.updated);
            if let Some(supersedes) = doc.metadata.supersedes {
                println!("  Supersedes: {:04}", supersedes);
            }
            if let Some(superseded_by) = doc.metadata.superseded_by {
                println!("  Superseded by: {:04}", superseded_by);
            }
            println!();
        } else {
            println!(
                "{} {} [{}]",
                theme::doc_number(doc.metadata.number),
                doc.metadata.title,
                theme::state_badge(state)
            );
        }
    }

    println!("\nTotal: {} documents\n", docs.len());
    Ok(())
}

/// List documents that have been removed to the dustbin
fn list_removed_documents(state_mgr: &StateManager, verbose: bool) -> Result<()> {
    println!();
    println!("{}", "Removed Documents".cyan().bold());
    println!();

    // Filter for removed documents
    let removed_docs: Vec<_> = state_mgr
        .state()
        .all()
        .into_iter()
        .filter(|d| {
            d.metadata.state == DocState::Removed || d.metadata.state == DocState::Overwritten
        })
        .collect();

    if removed_docs.is_empty() {
        println!("  {}", "No removed documents found.".yellow());
        println!();
        return Ok(());
    }

    // Print header
    if verbose {
        println!(
            "{:<8} {:<35} {:<12} {:<8} {}",
            "Number".cyan().bold(),
            "Title".cyan().bold(),
            "Removed".cyan().bold(),
            "Deleted".cyan().bold(),
            "Dustbin Location".cyan().bold()
        );
        println!("{}", "─".repeat(120).cyan());
    } else {
        println!(
            "{:<8} {:<40} {:<12} {}",
            "Number".cyan().bold(),
            "Title".cyan().bold(),
            "Removed".cyan().bold(),
            "Deleted".cyan().bold()
        );
        println!("{}", "─".repeat(80).cyan());
    }

    // Check each document's deletion status
    let mut in_dustbin = 0;
    let mut deleted = 0;

    for doc in &removed_docs {
        let number_str = format!("{:04}", doc.metadata.number);
        let title_truncated = if doc.metadata.title.len() > (if verbose { 33 } else { 38 }) {
            format!("{}...", &doc.metadata.title[..(if verbose { 30 } else { 35 })])
        } else {
            doc.metadata.title.clone()
        };

        // Check if file exists in dustbin
        let file_path = state_mgr.docs_dir().join(&doc.path);
        let file_exists = file_path.exists();
        let deleted_status = if file_exists {
            in_dustbin += 1;
            "false".green()
        } else {
            deleted += 1;
            "true".red()
        };

        if verbose {
            let location =
                if file_exists { doc.path.clone() } else { "(file not found)".to_string() };

            println!(
                "{:<8} {:<35} {:<12} {:<8} {}",
                number_str.yellow(),
                title_truncated,
                doc.metadata.updated.to_string().white(),
                deleted_status,
                location.dimmed()
            );
        } else {
            println!(
                "{:<8} {:<40} {:<12} {}",
                number_str.yellow(),
                title_truncated,
                doc.metadata.updated.to_string().white(),
                deleted_status
            );
        }
    }

    println!();
    println!(
        "Total: {} removed ({} in dustbin, {} deleted)",
        removed_docs.len().to_string().yellow(),
        in_dustbin.to_string().green(),
        deleted.to_string().red()
    );
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use design::doc::DocMetadata;
    use design::index::DocumentIndex;
    use design::state::{DocumentRecord, DocumentState};
    use tempfile::TempDir;

    fn create_test_index() -> DocumentIndex {
        let temp = TempDir::new().unwrap();

        // Create state with test documents
        let mut state = DocumentState::new();

        for (num, title, doc_state) in [
            (1, "First Doc", DocState::Draft),
            (2, "Second Doc", DocState::Final),
            (3, "Third Doc", DocState::Draft),
            (4, "Fourth Doc", DocState::Accepted),
        ] {
            let meta = DocMetadata {
                number: num,
                title: title.to_string(),
                author: "Test Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                state: doc_state,
                supersedes: None,
                superseded_by: None,
            };
            state.upsert(
                num,
                DocumentRecord {
                    metadata: meta,
                    path: format!("{:04}-test.md", num),
                    checksum: "abc123".to_string(),
                    file_size: 100,
                    modified: chrono::Utc::now(),
                },
            );
        }

        DocumentIndex::from_state(&state, temp.path()).unwrap()
    }

    #[test]
    fn test_list_all_documents() {
        let index = create_test_index();

        // Should not panic and should return Ok
        let result = list_documents(&index, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_with_valid_state_filter() {
        let index = create_test_index();

        // Filter by Draft state
        let result = list_documents(&index, Some("Draft".to_string()), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_with_state_filter_case_insensitive() {
        let index = create_test_index();

        // Filter by lowercase "draft"
        let result = list_documents(&index, Some("draft".to_string()), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_with_invalid_state_filter() {
        let index = create_test_index();

        // Invalid state should return Ok but print error
        let result = list_documents(&index, Some("InvalidState".to_string()), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_verbose_mode() {
        let index = create_test_index();

        // Verbose mode should work
        let result = list_documents(&index, None, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_verbose_with_filter() {
        let index = create_test_index();

        // Verbose + filter
        let result = list_documents(&index, Some("Final".to_string()), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_empty_index() {
        let temp = TempDir::new().unwrap();
        let index = DocumentIndex::new(temp.path()).unwrap();

        // Empty index should work
        let result = list_documents(&index, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_all_state_types() {
        let index = create_test_index();

        // Test filtering by each state type
        for state in DocState::all_states() {
            let result = list_documents(&index, Some(state.as_str().to_string()), false);
            assert!(result.is_ok(), "Failed for state: {}", state.as_str());
        }
    }

    // Tests for list_documents_with_state and --removed flag

    fn create_test_state_manager_with_removed() -> (StateManager, TempDir) {
        use std::fs;

        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();

        let mut state_mgr = StateManager::new(&docs_dir).unwrap();

        // Add some normal documents
        for (num, title, doc_state) in
            [(1, "Active Doc", DocState::Active), (2, "Draft Doc", DocState::Draft)]
        {
            let meta = DocMetadata {
                number: num,
                title: title.to_string(),
                author: "Test Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                state: doc_state,
                supersedes: None,
                superseded_by: None,
            };
            state_mgr.state_mut().upsert(
                num,
                DocumentRecord {
                    metadata: meta,
                    path: format!(
                        "{}/{:04}-{}.md",
                        doc_state.directory(),
                        num,
                        title.to_lowercase().replace(' ', "-")
                    ),
                    checksum: "abc123".to_string(),
                    file_size: 100,
                    modified: chrono::Utc::now(),
                },
            );
        }

        // Add removed documents
        for (num, title, doc_state) in
            [(3, "Removed Doc", DocState::Removed), (4, "Overwritten Doc", DocState::Overwritten)]
        {
            let meta = DocMetadata {
                number: num,
                title: title.to_string(),
                author: "Test Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
                state: doc_state,
                supersedes: None,
                superseded_by: None,
            };
            let path = format!(
                "{}/{:04}-{}.md",
                doc_state.directory(),
                num,
                title.to_lowercase().replace(' ', "-")
            );
            state_mgr.state_mut().upsert(
                num,
                DocumentRecord {
                    metadata: meta,
                    path,
                    checksum: "abc123".to_string(),
                    file_size: 100,
                    modified: chrono::Utc::now(),
                },
            );
        }

        (state_mgr, temp)
    }

    #[test]
    fn test_list_documents_with_state_no_removed() {
        let (state_mgr, _temp) = create_test_state_manager_with_removed();
        let index = DocumentIndex::from_state(state_mgr.state(), state_mgr.docs_dir()).unwrap();

        // List without --removed flag should work
        let result = list_documents_with_state(&index, Some(&state_mgr), None, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_documents_with_state_removed_flag() {
        let (state_mgr, _temp) = create_test_state_manager_with_removed();
        let index = DocumentIndex::from_state(state_mgr.state(), state_mgr.docs_dir()).unwrap();

        // List with --removed flag should work
        let result = list_documents_with_state(&index, Some(&state_mgr), None, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_documents_with_state_removed_verbose() {
        let (state_mgr, _temp) = create_test_state_manager_with_removed();
        let index = DocumentIndex::from_state(state_mgr.state(), state_mgr.docs_dir()).unwrap();

        // List with --removed and --verbose should work
        let result = list_documents_with_state(&index, Some(&state_mgr), None, true, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_documents_with_state_removed_no_state_mgr() {
        let index = create_test_index();

        // List with --removed but no state manager should handle gracefully
        let result = list_documents_with_state(&index, None, None, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_removed_documents_empty() {
        use std::fs;

        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();

        let state_mgr = StateManager::new(&docs_dir).unwrap();

        // No removed documents
        let result = list_removed_documents(&state_mgr, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_removed_documents_with_files() {
        use std::fs;

        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();

        let mut state_mgr = StateManager::new(&docs_dir).unwrap();

        // Add removed document with file present
        let meta = DocMetadata {
            number: 1,
            title: "Removed Doc".to_string(),
            author: "Test Author".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            state: DocState::Removed,
            supersedes: None,
            superseded_by: None,
        };

        let dustbin_path = docs_dir.join(".dustbin/0001-removed-doc.md");
        fs::create_dir_all(dustbin_path.parent().unwrap()).unwrap();
        fs::write(&dustbin_path, "test content").unwrap();

        state_mgr.state_mut().upsert(
            1,
            DocumentRecord {
                metadata: meta,
                path: ".dustbin/0001-removed-doc.md".to_string(),
                checksum: "abc123".to_string(),
                file_size: 100,
                modified: chrono::Utc::now(),
            },
        );

        // Should show file exists (deleted=false)
        let result = list_removed_documents(&state_mgr, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_removed_documents_without_files() {
        use std::fs;

        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();

        let mut state_mgr = StateManager::new(&docs_dir).unwrap();

        // Add removed document without file (deleted)
        let meta = DocMetadata {
            number: 1,
            title: "Deleted Doc".to_string(),
            author: "Test Author".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            state: DocState::Removed,
            supersedes: None,
            superseded_by: None,
        };

        state_mgr.state_mut().upsert(
            1,
            DocumentRecord {
                metadata: meta,
                path: ".dustbin/0001-deleted-doc.md".to_string(),
                checksum: "abc123".to_string(),
                file_size: 100,
                modified: chrono::Utc::now(),
            },
        );

        // Should show file doesn't exist (deleted=true)
        let result = list_removed_documents(&state_mgr, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_removed_documents_verbose_with_files() {
        use std::fs;

        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();

        let mut state_mgr = StateManager::new(&docs_dir).unwrap();

        let meta = DocMetadata {
            number: 1,
            title: "Removed Doc".to_string(),
            author: "Test Author".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            state: DocState::Removed,
            supersedes: None,
            superseded_by: None,
        };

        let dustbin_path = docs_dir.join(".dustbin/0001-removed-doc.md");
        fs::create_dir_all(dustbin_path.parent().unwrap()).unwrap();
        fs::write(&dustbin_path, "test content").unwrap();

        state_mgr.state_mut().upsert(
            1,
            DocumentRecord {
                metadata: meta,
                path: ".dustbin/0001-removed-doc.md".to_string(),
                checksum: "abc123".to_string(),
                file_size: 100,
                modified: chrono::Utc::now(),
            },
        );

        // Verbose mode should show location
        let result = list_removed_documents(&state_mgr, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_removed_documents_mixed_overwritten_and_removed() {
        use std::fs;

        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();

        let mut state_mgr = StateManager::new(&docs_dir).unwrap();

        // Add both Removed and Overwritten documents
        for (num, title, doc_state) in
            [(1, "Removed Doc", DocState::Removed), (2, "Overwritten Doc", DocState::Overwritten)]
        {
            let meta = DocMetadata {
                number: num,
                title: title.to_string(),
                author: "Test Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
                state: doc_state,
                supersedes: None,
                superseded_by: None,
            };

            state_mgr.state_mut().upsert(
                num,
                DocumentRecord {
                    metadata: meta,
                    path: format!(
                        "{}/{:04}-{}.md",
                        doc_state.directory(),
                        num,
                        title.to_lowercase().replace(' ', "-")
                    ),
                    checksum: "abc123".to_string(),
                    file_size: 100,
                    modified: chrono::Utc::now(),
                },
            );
        }

        // Should show both types
        let result = list_removed_documents(&state_mgr, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_removed_documents_long_title_truncation() {
        use std::fs;

        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();

        let mut state_mgr = StateManager::new(&docs_dir).unwrap();

        // Add document with very long title (should truncate)
        let long_title = "This is a very long title that should be truncated in the output to fit the column width".to_string();
        let meta = DocMetadata {
            number: 1,
            title: long_title,
            author: "Test Author".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            state: DocState::Removed,
            supersedes: None,
            superseded_by: None,
        };

        state_mgr.state_mut().upsert(
            1,
            DocumentRecord {
                metadata: meta,
                path: ".dustbin/0001-long-title.md".to_string(),
                checksum: "abc123".to_string(),
                file_size: 100,
                modified: chrono::Utc::now(),
            },
        );

        // Should handle truncation without panic
        let result = list_removed_documents(&state_mgr, false);
        assert!(result.is_ok());
    }
}
