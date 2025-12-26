//! Search command implementation

use anyhow::Result;
use colored::*;
use design::doc::DocState;
use design::state::StateManager;
use regex::Regex;
use std::process::Command;

/// Search documents using git grep
pub fn search(
    state_mgr: &StateManager,
    query: &str,
    state_filter: Option<String>,
    metadata_only: bool,
    case_sensitive: bool,
) -> Result<()> {
    println!("{} Searching for: {}\n", "→".cyan(), query.bold());

    // Build git grep command
    let mut cmd = Command::new("git");
    cmd.arg("grep");

    // Options
    cmd.arg("-n"); // Show line numbers
    cmd.arg("--color=never"); // We'll colorize ourselves

    if !case_sensitive {
        cmd.arg("-i"); // Case insensitive
    }

    // Pattern
    cmd.arg(query);

    // Paths to search
    if let Some(state_str) = &state_filter {
        // Filter by state directory
        if let Some(state) = DocState::from_str_flexible(state_str) {
            let state_dir = format!("{}/{}", state_mgr.docs_dir().display(), state.directory());
            cmd.arg("--");
            cmd.arg(format!("{}/*.md", state_dir));
        } else {
            anyhow::bail!("Invalid state: {}", state_str);
        }
    } else {
        // Search all docs
        cmd.arg("--");
        cmd.arg(format!("{}/**/*.md", state_mgr.docs_dir().display()));
    }

    // Execute search
    let output = cmd.output()?;

    if output.status.success() || !output.stdout.is_empty() {
        let results = String::from_utf8_lossy(&output.stdout);

        // Parse and enhance results
        let match_count = display_results(&results, state_mgr, metadata_only, query)?;

        if match_count > 0 {
            println!("\n{} {} matches found", "✓".green(), match_count);
        } else {
            println!("{} No matches found", "→".cyan());
        }
    } else {
        println!("{} No matches found", "→".cyan());
    }

    Ok(())
}

fn display_results(
    results: &str,
    state_mgr: &StateManager,
    metadata_only: bool,
    query: &str,
) -> Result<usize> {
    // Pattern to extract: path:line:content
    let re = Regex::new(r"^([^:]+):(\d+):(.*)$").unwrap();

    let mut current_file = String::new();
    let mut match_count = 0;

    for line in results.lines() {
        if let Some(caps) = re.captures(line) {
            let path = caps.get(1).unwrap().as_str();
            let line_num = caps.get(2).unwrap().as_str();
            let content = caps.get(3).unwrap().as_str();

            // Extract document number from path
            let doc_number = extract_number_from_path(path);

            // Check if in YAML frontmatter (lines < ~15 usually)
            let line_num_val = line_num.parse::<usize>().unwrap_or(999);
            let is_metadata = line_num_val < 15;

            // Skip if metadata_only and not in metadata
            if metadata_only && !is_metadata {
                continue;
            }

            match_count += 1;

            // Print file header on change
            if path != current_file {
                println!();

                // Try to get document title from state
                if let Some(num) = doc_number {
                    if let Some(record) = state_mgr.state().get(num) {
                        println!(
                            "{} {:04} - {} ({})",
                            "→".cyan(),
                            num,
                            record.metadata.title.bold(),
                            record.metadata.state.as_str().dimmed()
                        );
                    } else {
                        println!("{} {}", "→".cyan(), path.bold());
                    }
                } else {
                    println!("{} {}", "→".cyan(), path.bold());
                }

                current_file = path.to_string();
            }

            // Highlight the matched text in content
            let highlighted = highlight_match(content, query);

            // Print the match
            println!("  {}:{}", line_num.dimmed(), highlighted);
        }
    }

    Ok(match_count)
}

fn extract_number_from_path(path: &str) -> Option<u32> {
    let re = Regex::new(r"(\d{4})-").unwrap();
    re.captures(path).and_then(|caps| caps.get(1)).and_then(|m| m.as_str().parse().ok())
}

fn highlight_match(content: &str, query: &str) -> String {
    // Simple case-insensitive highlighting
    let lower_content = content.to_lowercase();
    let lower_query = query.to_lowercase();

    if let Some(pos) = lower_content.find(&lower_query) {
        let before = &content[..pos];
        let matched = &content[pos..pos + query.len()];
        let after = &content[pos + query.len()..];

        format!("{}{}{}", before, matched.red().bold(), after)
    } else {
        content.to_string()
    }
}

/// Search options for advanced searches
#[derive(Default)]
#[allow(dead_code)]
pub struct SearchOptions {
    pub state: Option<String>,
    pub metadata_only: bool,
    pub case_sensitive: bool,
    #[allow(dead_code)]
    pub context_lines: usize,
    #[allow(dead_code)]
    pub regex: bool,
}

/// Search with more advanced options
#[allow(dead_code)]
pub fn search_advanced(
    state_mgr: &StateManager,
    query: &str,
    options: SearchOptions,
) -> Result<()> {
    search(state_mgr, query, options.state, options.metadata_only, options.case_sensitive)
}

#[cfg(test)]
mod tests {
    use super::*;
    use design::doc::{DocMetadata, DocState};
    use design::state::{DocumentRecord, DocumentState};
    use chrono::NaiveDate;
    use std::fs;
    use tempfile::TempDir;
    use serial_test::serial;

    fn create_test_doc_with_content(number: u32, title: &str, state: DocState, extra_content: &str) -> String {
        format!(
            r#"---
number: {}
title: "{}"
author: "Test Author"
created: 2024-01-01
updated: 2024-01-01
state: {}
---

# {}

{}
"#,
            number, title, state.as_str(), title, extra_content
        )
    }

    fn setup_git_repo_with_docs(temp: &TempDir) -> StateManager {
        let repo_path = temp.path();

        // Initialize git repo
        std::process::Command::new("git")
            .arg("init")
            .current_dir(&repo_path)
            .output()
            .unwrap();

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

        // Create state manager
        let mut state_mgr = StateManager::new(repo_path).unwrap();

        // Create a few test documents
        let draft_dir = repo_path.join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        let doc1_path = draft_dir.join("0001-first-doc.md");
        let content1 = create_test_doc_with_content(
            1,
            "First Document",
            DocState::Draft,
            "This document contains information about testing.\n\nKeyword: important",
        );
        fs::write(&doc1_path, &content1).unwrap();

        let doc2_path = draft_dir.join("0002-second-doc.md");
        let content2 = create_test_doc_with_content(
            2,
            "Second Document",
            DocState::Draft,
            "Another document with different content.\n\nKeyword: testing",
        );
        fs::write(&doc2_path, &content2).unwrap();

        // Create a Final document
        let final_dir = repo_path.join("06-final");
        fs::create_dir_all(&final_dir).unwrap();

        let doc3_path = final_dir.join("0003-final-doc.md");
        let content3 = create_test_doc_with_content(
            3,
            "Final Document",
            DocState::Final,
            "This is a final document.\n\nKeyword: important",
        );
        fs::write(&doc3_path, &content3).unwrap();

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

        // Update state manager
        state_mgr.scan_for_changes().unwrap();

        state_mgr
    }

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
    #[serial]
    fn test_search_basic() {
        let temp = TempDir::new().unwrap();
        let state_mgr = setup_git_repo_with_docs(&temp);

        let result = in_dir(temp.path(), || {
            search(&state_mgr, "important", None, false, false)
        });

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_search_case_sensitive() {
        let temp = TempDir::new().unwrap();
        let state_mgr = setup_git_repo_with_docs(&temp);

        let result = in_dir(temp.path(), || {
            search(&state_mgr, "Important", None, false, true)
        });

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_search_case_insensitive() {
        let temp = TempDir::new().unwrap();
        let state_mgr = setup_git_repo_with_docs(&temp);

        let result = in_dir(temp.path(), || {
            search(&state_mgr, "IMPORTANT", None, false, false)
        });

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_search_with_state_filter() {
        let temp = TempDir::new().unwrap();
        let state_mgr = setup_git_repo_with_docs(&temp);

        let result = in_dir(temp.path(), || {
            search(&state_mgr, "important", Some("Draft".to_string()), false, false)
        });

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_search_invalid_state() {
        let temp = TempDir::new().unwrap();
        let state_mgr = setup_git_repo_with_docs(&temp);

        let result = in_dir(temp.path(), || {
            search(&state_mgr, "test", Some("InvalidState".to_string()), false, false)
        });

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid state"));
    }

    #[test]
    #[serial]
    fn test_search_metadata_only() {
        let temp = TempDir::new().unwrap();
        let state_mgr = setup_git_repo_with_docs(&temp);

        let result = in_dir(temp.path(), || {
            search(&state_mgr, "author", None, true, false)
        });

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_search_no_matches() {
        let temp = TempDir::new().unwrap();
        let state_mgr = setup_git_repo_with_docs(&temp);

        let result = in_dir(temp.path(), || {
            search(&state_mgr, "nonexistent_keyword_12345", None, false, false)
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_number_from_path() {
        assert_eq!(extract_number_from_path("01-draft/0001-test.md"), Some(1));
        assert_eq!(extract_number_from_path("06-final/0042-document.md"), Some(42));
        assert_eq!(extract_number_from_path("docs/0123-file.md"), Some(123));
        assert_eq!(extract_number_from_path("no-number-here.md"), None);
        assert_eq!(extract_number_from_path("999-invalid.md"), None); // Not 4 digits
    }

    #[test]
    fn test_highlight_match() {
        let result = highlight_match("This is a test string", "test");
        assert!(result.contains("test"));

        let result = highlight_match("UPPERCASE TEST", "test");
        assert!(result.contains("TEST"));

        let result = highlight_match("no match here", "xyz");
        assert_eq!(result, "no match here");
    }

    #[test]
    fn test_search_options_default() {
        let options = SearchOptions::default();
        assert!(options.state.is_none());
        assert!(!options.metadata_only);
        assert!(!options.case_sensitive);
        assert_eq!(options.context_lines, 0);
    }

    #[test]
    #[serial]
    fn test_search_advanced() {
        let temp = TempDir::new().unwrap();
        let state_mgr = setup_git_repo_with_docs(&temp);

        let options = SearchOptions {
            state: Some("Draft".to_string()),
            metadata_only: false,
            case_sensitive: false,
            context_lines: 0,
            regex: false,
        };

        let result = in_dir(temp.path(), || {
            search_advanced(&state_mgr, "testing", options)
        });

        assert!(result.is_ok());
    }
}
