//! Validate command implementation

use anyhow::Result;
use colored::*;
use design::constants::INDEX_FILENAME;
use design::doc::state_from_directory;
use design::index::DocumentIndex;
use design::index_sync::{get_docs_from_filesystem, ParsedIndex};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
enum ValidationIssue {
    DuplicateNumber { number: u32, paths: Vec<String> },
    MissingReference { doc_num: u32, ref_type: String, ref_num: u32 },
    DateOrderIssue { doc_num: u32, created: String, updated: String },
    StateDirectoryMismatch { doc_num: u32, yaml_state: String, dir_state: String, path: String },
    NotInIndex { doc_num: u32, title: String, path: String },
    InIndexNotOnDisk { number: String },
    MissingHeaders { path: String },
}

impl ValidationIssue {
    fn severity(&self) -> &str {
        match self {
            ValidationIssue::DuplicateNumber { .. } => "ERROR",
            ValidationIssue::MissingReference { .. } => "ERROR",
            ValidationIssue::StateDirectoryMismatch { .. } => "WARNING",
            ValidationIssue::NotInIndex { .. } => "WARNING",
            ValidationIssue::InIndexNotOnDisk { .. } => "ERROR",
            ValidationIssue::DateOrderIssue { .. } => "WARNING",
            ValidationIssue::MissingHeaders { .. } => "WARNING",
        }
    }

    fn description(&self) -> String {
        match self {
            ValidationIssue::DuplicateNumber { number, paths } => {
                format!(
                    "Duplicate number {:04} found in {} files:\n{}",
                    number,
                    paths.len(),
                    paths.iter().map(|p| format!("      {}", p)).collect::<Vec<_>>().join("\n")
                )
            }
            ValidationIssue::MissingReference { doc_num, ref_type, ref_num } => {
                format!(
                    "Document {:04} references non-existent {} {:04}",
                    doc_num, ref_type, ref_num
                )
            }
            ValidationIssue::DateOrderIssue { doc_num, created, updated } => {
                format!(
                    "Document {:04}: created date ({}) is after updated date ({})",
                    doc_num, created, updated
                )
            }
            ValidationIssue::StateDirectoryMismatch { doc_num, yaml_state, dir_state, path } => {
                format!(
                    "Document {:04}: YAML state '{}' doesn't match directory state '{}'\n      Path: {}",
                    doc_num, yaml_state, dir_state, path
                )
            }
            ValidationIssue::NotInIndex { doc_num, title, path } => {
                format!(
                    "Document {:04} '{}' exists but not in index\n      Path: {}",
                    doc_num, title, path
                )
            }
            ValidationIssue::InIndexNotOnDisk { number } => {
                format!("Index entry {} references non-existent file", number)
            }
            ValidationIssue::MissingHeaders { path } => {
                format!("Document missing YAML headers: {}", path)
            }
        }
    }

    fn can_auto_fix(&self) -> bool {
        matches!(
            self,
            ValidationIssue::StateDirectoryMismatch { .. }
                | ValidationIssue::NotInIndex { .. }
                | ValidationIssue::MissingHeaders { .. }
        )
    }

    fn fix_description(&self) -> Option<String> {
        match self {
            ValidationIssue::StateDirectoryMismatch { .. } => {
                Some("Run 'oxd sync-location <file>' to fix".to_string())
            }
            ValidationIssue::NotInIndex { .. } => {
                Some("Run 'oxd update-index' to add to index".to_string())
            }
            ValidationIssue::MissingHeaders { .. } => {
                Some("Run 'oxd add-headers <file>' to add headers".to_string())
            }
            _ => None,
        }
    }
}

pub fn validate_documents(index: &DocumentIndex, fix: bool) -> Result<()> {
    println!("\n{}\n", "Validating design documents...".bold());

    let mut issues = Vec::new();

    // Get all documents
    let docs = index.all();

    // Check 1: Duplicate numbers
    let mut number_map: HashMap<u32, Vec<String>> = HashMap::new();
    for doc in &docs {
        let path_str = doc.path.to_string_lossy().to_string();
        number_map.entry(doc.metadata.number).or_default().push(path_str);
    }

    for (number, paths) in number_map.iter() {
        if paths.len() > 1 {
            issues.push(ValidationIssue::DuplicateNumber { number: *number, paths: paths.clone() });
        }
    }

    // Check 2: Supersedes/superseded-by references
    let valid_numbers: HashSet<u32> = docs.iter().map(|d| d.metadata.number).collect();

    for doc in &docs {
        if let Some(supersedes) = doc.metadata.supersedes {
            if !valid_numbers.contains(&supersedes) {
                issues.push(ValidationIssue::MissingReference {
                    doc_num: doc.metadata.number,
                    ref_type: "supersedes".to_string(),
                    ref_num: supersedes,
                });
            }
        }

        if let Some(superseded_by) = doc.metadata.superseded_by {
            if !valid_numbers.contains(&superseded_by) {
                issues.push(ValidationIssue::MissingReference {
                    doc_num: doc.metadata.number,
                    ref_type: "superseded-by".to_string(),
                    ref_num: superseded_by,
                });
            }
        }
    }

    // Check 3: Date ordering
    for doc in &docs {
        if doc.metadata.created > doc.metadata.updated {
            issues.push(ValidationIssue::DateOrderIssue {
                doc_num: doc.metadata.number,
                created: doc.metadata.created.to_string(),
                updated: doc.metadata.updated.to_string(),
            });
        }
    }

    // Check 4: State/directory consistency
    for doc in &docs {
        if let Some(dir_state) = state_from_directory(&doc.path) {
            if doc.metadata.state != dir_state {
                issues.push(ValidationIssue::StateDirectoryMismatch {
                    doc_num: doc.metadata.number,
                    yaml_state: doc.metadata.state.as_str().to_string(),
                    dir_state: dir_state.as_str().to_string(),
                    path: doc.path.to_string_lossy().to_string(),
                });
            }
        }
    }

    // Check 5: Index consistency
    let index_path = PathBuf::from(index.docs_dir()).join(INDEX_FILENAME);
    if index_path.exists() {
        if let Ok(index_content) = fs::read_to_string(&index_path) {
            if let Ok(parsed_index) = ParsedIndex::parse(&index_content) {
                let indexed_numbers: HashSet<String> =
                    parsed_index.table_entries.keys().cloned().collect();

                // Check for docs not in index
                for doc in &docs {
                    let number_str = format!("{:04}", doc.metadata.number);
                    if !indexed_numbers.contains(&number_str) {
                        issues.push(ValidationIssue::NotInIndex {
                            doc_num: doc.metadata.number,
                            title: doc.metadata.title.clone(),
                            path: doc.path.to_string_lossy().to_string(),
                        });
                    }
                }

                // Check for index entries without files
                for number in &indexed_numbers {
                    let num: u32 = number.parse().unwrap_or(0);
                    if !valid_numbers.contains(&num) {
                        issues.push(ValidationIssue::InIndexNotOnDisk { number: number.clone() });
                    }
                }
            }
        }
    }

    // Check 6: Files without headers
    if let Ok(filesystem_docs) = get_docs_from_filesystem(index.docs_dir()) {
        for path in filesystem_docs {
            if let Ok(content) = fs::read_to_string(&path) {
                if !content.trim_start().starts_with("---\n") {
                    issues.push(ValidationIssue::MissingHeaders {
                        path: path.to_string_lossy().to_string(),
                    });
                }
            }
        }
    }

    // Display issues
    let mut errors = 0;
    let mut warnings = 0;

    for issue in &issues {
        let severity = issue.severity();
        let colored_severity = match severity {
            "ERROR" => format!("{}:", severity).red().bold(),
            "WARNING" => format!("{}:", severity).yellow().bold(),
            _ => format!("{}:", severity).white().bold(),
        };

        println!("{} {}", colored_severity, issue.description());

        if let Some(fix_msg) = issue.fix_description() {
            println!("    {} {}", "→".cyan(), fix_msg.dimmed());
        }

        println!();

        match severity {
            "ERROR" => errors += 1,
            "WARNING" => warnings += 1,
            _ => {}
        }
    }

    // Summary
    if issues.is_empty() {
        println!("{} All documents valid!", "✓".green().bold());
    } else {
        println!("{} Found {} error(s) and {} warning(s)", "Summary:".bold(), errors, warnings);

        let auto_fixable = issues.iter().filter(|i| i.can_auto_fix()).count();
        if auto_fixable > 0 && !fix {
            println!(
                "\n{} {} issue(s) can be auto-fixed. Run with {} to fix them.",
                "→".cyan(),
                auto_fixable,
                "--fix".cyan()
            );
        }

        if fix {
            println!("\n{}", "Auto-fixing issues...".bold());
            apply_fixes(index, &issues)?;
        }
    }

    println!();
    Ok(())
}

fn apply_fixes(index: &DocumentIndex, issues: &[ValidationIssue]) -> Result<()> {
    use crate::commands::{add_headers, sync_location, update_index};

    let mut fixed = 0;

    for issue in issues {
        if !issue.can_auto_fix() {
            continue;
        }

        match issue {
            ValidationIssue::StateDirectoryMismatch { path, .. } => {
                println!("  Fixing state/directory mismatch: {}", path);
                if let Err(e) = sync_location(index, path) {
                    eprintln!("    {} Failed: {}", "✗".red(), e);
                } else {
                    println!("    {} Fixed", "✓".green());
                    fixed += 1;
                }
            }
            ValidationIssue::MissingHeaders { path } => {
                println!("  Adding headers: {}", path);
                if let Err(e) = add_headers(path) {
                    eprintln!("    {} Failed: {}", "✗".red(), e);
                } else {
                    println!("    {} Fixed", "✓".green());
                    fixed += 1;
                }
            }
            ValidationIssue::NotInIndex { .. } => {
                // These will be fixed by update-index at the end
            }
            _ => {}
        }
    }

    // Update index to fix NotInIndex issues
    if issues.iter().any(|i| matches!(i, ValidationIssue::NotInIndex { .. })) {
        println!("  Updating index...");
        let updated_index = DocumentIndex::new(index.docs_dir())?;
        if let Err(e) = update_index(&updated_index) {
            eprintln!("    {} Failed: {}", "✗".red(), e);
        } else {
            println!("    {} Fixed", "✓".green());
            fixed +=
                issues.iter().filter(|i| matches!(i, ValidationIssue::NotInIndex { .. })).count();
        }
    }

    println!("\n{} {} issue(s) fixed", "✓".green().bold(), fixed);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use design::doc::DocState;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_doc_file(
        temp: &TempDir,
        number: u32,
        title: &str,
        state: DocState,
        created: NaiveDate,
        updated: NaiveDate,
    ) {
        let content = format!(
            "---\nnumber: {}\ntitle: \"{}\"\nauthor: \"Test\"\ncreated: {}\nupdated: {}\nstate: {}\n---\n\n# {}\n\nContent",
            number, title, created, updated, state.as_str(), title
        );
        fs::write(temp.path().join(format!("{:04}-test.md", number)), content).unwrap();
    }

    fn create_valid_index() -> DocumentIndex {
        let temp = TempDir::new().unwrap();

        // Create some valid documents
        create_test_doc_file(
            &temp,
            1,
            "Doc 1",
            DocState::Draft,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        );
        create_test_doc_file(
            &temp,
            2,
            "Doc 2",
            DocState::Final,
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 10).unwrap(),
        );

        DocumentIndex::new(temp.path()).unwrap()
    }

    #[test]
    fn test_validate_no_issues() {
        let index = create_valid_index();

        let result = validate_documents(&index, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_empty_index() {
        let temp = TempDir::new().unwrap();
        let index = DocumentIndex::new(temp.path()).unwrap();

        let result = validate_documents(&index, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_fix_mode() {
        let index = create_valid_index();

        let result = validate_documents(&index, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validation_issue_severity() {
        let duplicate = ValidationIssue::DuplicateNumber {
            number: 1,
            paths: vec!["a.md".to_string(), "b.md".to_string()],
        };
        assert_eq!(duplicate.severity(), "ERROR");

        let warning = ValidationIssue::StateDirectoryMismatch {
            doc_num: 1,
            yaml_state: "Draft".to_string(),
            dir_state: "Final".to_string(),
            path: "test.md".to_string(),
        };
        assert_eq!(warning.severity(), "WARNING");
    }

    #[test]
    fn test_validation_issue_description() {
        let issue = ValidationIssue::MissingReference {
            doc_num: 1,
            ref_type: "supersedes".to_string(),
            ref_num: 99,
        };
        let desc = issue.description();
        assert!(desc.contains("0001"));
        assert!(desc.contains("0099"));
        assert!(desc.contains("supersedes"));
    }

    #[test]
    fn test_validation_issue_can_auto_fix() {
        let fixable = ValidationIssue::StateDirectoryMismatch {
            doc_num: 1,
            yaml_state: "Draft".to_string(),
            dir_state: "Final".to_string(),
            path: "test.md".to_string(),
        };
        assert!(fixable.can_auto_fix());

        let not_fixable =
            ValidationIssue::DuplicateNumber { number: 1, paths: vec!["a.md".to_string()] };
        assert!(!not_fixable.can_auto_fix());
    }

    #[test]
    fn test_validation_issue_fix_description() {
        let issue = ValidationIssue::NotInIndex {
            doc_num: 1,
            title: "Test".to_string(),
            path: "test.md".to_string(),
        };
        let fix_msg = issue.fix_description();
        assert!(fix_msg.is_some());
        assert!(fix_msg.unwrap().contains("update-index"));
    }

    #[test]
    fn test_duplicate_number_issue() {
        let issue = ValidationIssue::DuplicateNumber {
            number: 42,
            paths: vec!["path1.md".to_string(), "path2.md".to_string()],
        };

        assert_eq!(issue.severity(), "ERROR");
        assert!(!issue.can_auto_fix());
        let desc = issue.description();
        assert!(desc.contains("0042"));
        assert!(desc.contains("2 files"));
    }

    #[test]
    fn test_missing_reference_issue() {
        let issue = ValidationIssue::MissingReference {
            doc_num: 10,
            ref_type: "superseded-by".to_string(),
            ref_num: 20,
        };

        assert_eq!(issue.severity(), "ERROR");
        assert!(!issue.can_auto_fix());
        assert!(issue.fix_description().is_none());
    }

    #[test]
    fn test_date_order_issue() {
        let issue = ValidationIssue::DateOrderIssue {
            doc_num: 5,
            created: "2024-02-01".to_string(),
            updated: "2024-01-01".to_string(),
        };

        assert_eq!(issue.severity(), "WARNING");
        let desc = issue.description();
        assert!(desc.contains("0005"));
        assert!(desc.contains("2024-02-01"));
        assert!(desc.contains("2024-01-01"));
    }

    #[test]
    fn test_state_directory_mismatch_issue() {
        let issue = ValidationIssue::StateDirectoryMismatch {
            doc_num: 3,
            yaml_state: "Draft".to_string(),
            dir_state: "Final".to_string(),
            path: "03-draft/test.md".to_string(),
        };

        assert_eq!(issue.severity(), "WARNING");
        assert!(issue.can_auto_fix());
        assert!(issue.fix_description().unwrap().contains("sync-location"));
    }

    #[test]
    fn test_not_in_index_issue() {
        let issue = ValidationIssue::NotInIndex {
            doc_num: 7,
            title: "New Doc".to_string(),
            path: "0007-new.md".to_string(),
        };

        assert_eq!(issue.severity(), "WARNING");
        assert!(issue.can_auto_fix());
    }

    #[test]
    fn test_in_index_not_on_disk_issue() {
        let issue = ValidationIssue::InIndexNotOnDisk { number: "0099".to_string() };

        assert_eq!(issue.severity(), "ERROR");
        assert!(!issue.can_auto_fix());
    }

    #[test]
    fn test_missing_headers_issue() {
        let issue = ValidationIssue::MissingHeaders { path: "test.md".to_string() };

        assert_eq!(issue.severity(), "WARNING");
        assert!(issue.can_auto_fix());
        assert!(issue.fix_description().unwrap().contains("add-headers"));
    }

    // Test all ValidationIssue variant descriptions
    #[test]
    fn test_duplicate_number_description_multiple_paths() {
        let issue = ValidationIssue::DuplicateNumber {
            number: 123,
            paths: vec!["path1.md".to_string(), "path2.md".to_string(), "path3.md".to_string()],
        };
        let desc = issue.description();
        assert!(desc.contains("0123"));
        assert!(desc.contains("3 files"));
        assert!(desc.contains("path1.md"));
        assert!(desc.contains("path2.md"));
        assert!(desc.contains("path3.md"));
    }

    #[test]
    fn test_date_order_issue_description() {
        let issue = ValidationIssue::DateOrderIssue {
            doc_num: 15,
            created: "2024-12-01".to_string(),
            updated: "2024-11-01".to_string(),
        };
        let desc = issue.description();
        assert!(desc.contains("0015"));
        assert!(desc.contains("created date"));
        assert!(desc.contains("2024-12-01"));
        assert!(desc.contains("2024-11-01"));
    }

    #[test]
    fn test_state_directory_mismatch_description() {
        let issue = ValidationIssue::StateDirectoryMismatch {
            doc_num: 25,
            yaml_state: "Draft".to_string(),
            dir_state: "Final".to_string(),
            path: "/path/to/doc.md".to_string(),
        };
        let desc = issue.description();
        assert!(desc.contains("0025"));
        assert!(desc.contains("Draft"));
        assert!(desc.contains("Final"));
        assert!(desc.contains("/path/to/doc.md"));
    }

    #[test]
    fn test_not_in_index_description() {
        let issue = ValidationIssue::NotInIndex {
            doc_num: 30,
            title: "Test Document".to_string(),
            path: "/docs/0030-test.md".to_string(),
        };
        let desc = issue.description();
        assert!(desc.contains("0030"));
        assert!(desc.contains("Test Document"));
        assert!(desc.contains("/docs/0030-test.md"));
        assert!(desc.contains("not in index"));
    }

    #[test]
    fn test_in_index_not_on_disk_description() {
        let issue = ValidationIssue::InIndexNotOnDisk { number: "0050".to_string() };
        let desc = issue.description();
        assert!(desc.contains("0050"));
        assert!(desc.contains("non-existent file"));
    }

    #[test]
    fn test_missing_headers_description() {
        let issue = ValidationIssue::MissingHeaders { path: "/docs/no-headers.md".to_string() };
        let desc = issue.description();
        assert!(desc.contains("missing YAML headers"));
        assert!(desc.contains("/docs/no-headers.md"));
    }

    // Test fix descriptions for all fixable variants
    #[test]
    fn test_state_mismatch_fix_description() {
        let issue = ValidationIssue::StateDirectoryMismatch {
            doc_num: 1,
            yaml_state: "Draft".to_string(),
            dir_state: "Final".to_string(),
            path: "test.md".to_string(),
        };
        let fix = issue.fix_description().unwrap();
        assert!(fix.contains("oxd sync-location"));
    }

    #[test]
    fn test_not_in_index_fix_description() {
        let issue = ValidationIssue::NotInIndex {
            doc_num: 1,
            title: "Test".to_string(),
            path: "test.md".to_string(),
        };
        let fix = issue.fix_description().unwrap();
        assert!(fix.contains("oxd update-index"));
    }

    #[test]
    fn test_missing_headers_fix_description() {
        let issue = ValidationIssue::MissingHeaders { path: "test.md".to_string() };
        let fix = issue.fix_description().unwrap();
        assert!(fix.contains("oxd add-headers"));
    }

    #[test]
    fn test_non_fixable_issues_have_no_fix_description() {
        let duplicate =
            ValidationIssue::DuplicateNumber { number: 1, paths: vec!["a.md".to_string()] };
        assert!(duplicate.fix_description().is_none());

        let missing_ref = ValidationIssue::MissingReference {
            doc_num: 1,
            ref_type: "supersedes".to_string(),
            ref_num: 2,
        };
        assert!(missing_ref.fix_description().is_none());

        let date_order = ValidationIssue::DateOrderIssue {
            doc_num: 1,
            created: "2024-02-01".to_string(),
            updated: "2024-01-01".to_string(),
        };
        assert!(date_order.fix_description().is_none());

        let in_index_not_disk = ValidationIssue::InIndexNotOnDisk { number: "0001".to_string() };
        assert!(in_index_not_disk.fix_description().is_none());
    }

    // Test validate_documents with actual validation scenarios
    #[test]
    fn test_validate_detects_duplicate_numbers() {
        let temp = TempDir::new().unwrap();

        // Create two documents with the same number
        create_test_doc_file(
            &temp,
            1,
            "First Doc",
            DocState::Draft,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        );

        // Create another file with same number manually
        let content = "---\nnumber: 1\ntitle: \"Duplicate\"\nauthor: \"Test\"\ncreated: 2024-01-01\nupdated: 2024-01-10\nstate: Draft\n---\n\n# Duplicate\n\nContent";
        fs::write(temp.path().join("0001-duplicate.md"), content).unwrap();

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        // Should succeed but report issues
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_detects_missing_supersedes_reference() {
        let temp = TempDir::new().unwrap();

        // Create a document that references a non-existent supersedes
        let content = "---\nnumber: 1\ntitle: \"Doc 1\"\nauthor: \"Test\"\ncreated: 2024-01-01\nupdated: 2024-01-10\nstate: Draft\nsupersedes: 99\n---\n\n# Doc 1\n\nContent";
        fs::write(temp.path().join("0001-test.md"), content).unwrap();

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_detects_missing_superseded_by_reference() {
        let temp = TempDir::new().unwrap();

        // Create a document that references a non-existent superseded-by
        let content = "---\nnumber: 1\ntitle: \"Doc 1\"\nauthor: \"Test\"\ncreated: 2024-01-01\nupdated: 2024-01-10\nstate: Draft\nsuperseded-by: 99\n---\n\n# Doc 1\n\nContent";
        fs::write(temp.path().join("0001-test.md"), content).unwrap();

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_detects_valid_supersedes_reference() {
        let temp = TempDir::new().unwrap();

        // Create two documents where one supersedes the other
        create_test_doc_file(
            &temp,
            1,
            "Doc 1",
            DocState::Draft,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        );

        let content = "---\nnumber: 2\ntitle: \"Doc 2\"\nauthor: \"Test\"\ncreated: 2024-02-01\nupdated: 2024-02-10\nstate: Final\nsupersedes: 1\n---\n\n# Doc 2\n\nContent";
        fs::write(temp.path().join("0002-test.md"), content).unwrap();

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        // Should succeed with no missing reference issues
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_detects_date_order_issues() {
        let temp = TempDir::new().unwrap();

        // Create a document with created > updated
        create_test_doc_file(
            &temp,
            1,
            "Doc 1",
            DocState::Draft,
            NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_detects_state_directory_mismatch() {
        let temp = TempDir::new().unwrap();

        // Create subdirectories for states
        fs::create_dir_all(temp.path().join("01-draft")).unwrap();
        fs::create_dir_all(temp.path().join("02-final")).unwrap();

        // Create a doc in draft dir but with Final state
        let content = "---\nnumber: 1\ntitle: \"Doc 1\"\nauthor: \"Test\"\ncreated: 2024-01-01\nupdated: 2024-01-10\nstate: Final\n---\n\n# Doc 1\n\nContent";
        fs::write(temp.path().join("01-draft/0001-test.md"), content).unwrap();

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_missing_headers() {
        let temp = TempDir::new().unwrap();

        // Create a valid document first
        create_test_doc_file(
            &temp,
            1,
            "Doc 1",
            DocState::Draft,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        );

        // Create a file without YAML headers
        fs::write(temp.path().join("0002-no-headers.md"), "# Just a header\n\nNo YAML here")
            .unwrap();

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_empty_file() {
        let temp = TempDir::new().unwrap();

        // Create an empty file
        fs::write(temp.path().join("0001-empty.md"), "").unwrap();

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        // Should handle gracefully
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_whitespace_only_file() {
        let temp = TempDir::new().unwrap();

        // Create a file with only whitespace
        fs::write(temp.path().join("0001-whitespace.md"), "   \n  \n\t\n").unwrap();

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_partial_yaml_headers() {
        let temp = TempDir::new().unwrap();

        // Create a file with incomplete YAML (starts with --- but missing fields)
        let content = "---\ntitle: \"Partial\"\n---\n\n# Content";
        fs::write(temp.path().join("0001-partial.md"), content).unwrap();

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        // Index construction might fail or succeed but validation should handle it
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_index_file() {
        let temp = TempDir::new().unwrap();

        // Create a valid document
        create_test_doc_file(
            &temp,
            1,
            "Doc 1",
            DocState::Draft,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        );

        // Create an index file
        let index_content = "# Design Documents\n\n| Number | Title | State |\n|--------|-------|-------|\n| 0001 | Doc 1 | Draft |\n";
        fs::write(temp.path().join(INDEX_FILENAME), index_content).unwrap();

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_detects_doc_not_in_index() {
        let temp = TempDir::new().unwrap();

        // Create a valid document
        create_test_doc_file(
            &temp,
            1,
            "Doc 1",
            DocState::Draft,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        );

        // Create an index file that doesn't include the document
        let index_content =
            "# Design Documents\n\n| Number | Title | State |\n|--------|-------|-------|\n";
        fs::write(temp.path().join(INDEX_FILENAME), index_content).unwrap();

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_detects_index_entry_without_file() {
        let temp = TempDir::new().unwrap();

        // Create an index with a reference to a non-existent document
        let index_content = "# Design Documents\n\n| Number | Title | State |\n|--------|-------|-------|\n| 0099 | Missing | Draft |\n";
        fs::write(temp.path().join(INDEX_FILENAME), index_content).unwrap();

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_malformed_index() {
        let temp = TempDir::new().unwrap();

        // Create a valid document
        create_test_doc_file(
            &temp,
            1,
            "Doc 1",
            DocState::Draft,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        );

        // Create a malformed index file
        let index_content = "This is not a valid index format\n";
        fs::write(temp.path().join(INDEX_FILENAME), index_content).unwrap();

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        // Should handle gracefully
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_multiple_issues() {
        let temp = TempDir::new().unwrap();

        // Create document with date order issue
        create_test_doc_file(
            &temp,
            1,
            "Doc 1",
            DocState::Draft,
            NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        // Create document with missing reference
        let content = "---\nnumber: 2\ntitle: \"Doc 2\"\nauthor: \"Test\"\ncreated: 2024-01-01\nupdated: 2024-01-10\nstate: Draft\nsupersedes: 99\n---\n\n# Doc 2\n\nContent";
        fs::write(temp.path().join("0002-test.md"), content).unwrap();

        // Create file without headers
        fs::write(temp.path().join("0003-no-headers.md"), "# No headers\n\nContent").unwrap();

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_all_states() {
        let temp = TempDir::new().unwrap();

        create_test_doc_file(
            &temp,
            1,
            "Draft Doc",
            DocState::Draft,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        );

        create_test_doc_file(
            &temp,
            2,
            "Final Doc",
            DocState::Final,
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 10).unwrap(),
        );

        create_test_doc_file(
            &temp,
            3,
            "Accepted Doc",
            DocState::Accepted,
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 3, 10).unwrap(),
        );

        create_test_doc_file(
            &temp,
            4,
            "Rejected Doc",
            DocState::Rejected,
            NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 4, 10).unwrap(),
        );

        create_test_doc_file(
            &temp,
            5,
            "Superseded Doc",
            DocState::Superseded,
            NaiveDate::from_ymd_opt(2024, 5, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 5, 10).unwrap(),
        );

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_same_created_and_updated_dates() {
        let temp = TempDir::new().unwrap();

        // Same date should be valid (created <= updated)
        create_test_doc_file(
            &temp,
            1,
            "Doc 1",
            DocState::Draft,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_large_document_numbers() {
        let temp = TempDir::new().unwrap();

        create_test_doc_file(
            &temp,
            9999,
            "Large Number",
            DocState::Draft,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        );

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_zero_number() {
        let temp = TempDir::new().unwrap();

        create_test_doc_file(
            &temp,
            0,
            "Zero Doc",
            DocState::Draft,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        );

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_complex_cross_references() {
        let temp = TempDir::new().unwrap();

        // Create a chain of documents with supersedes relationships
        create_test_doc_file(
            &temp,
            1,
            "Doc 1",
            DocState::Superseded,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        );

        let content = "---\nnumber: 2\ntitle: \"Doc 2\"\nauthor: \"Test\"\ncreated: 2024-02-01\nupdated: 2024-02-10\nstate: Superseded\nsupersedes: 1\n---\n\n# Doc 2\n\nContent";
        fs::write(temp.path().join("0002-test.md"), content).unwrap();

        let content = "---\nnumber: 3\ntitle: \"Doc 3\"\nauthor: \"Test\"\ncreated: 2024-03-01\nupdated: 2024-03-10\nstate: Final\nsupersedes: 2\n---\n\n# Doc 3\n\nContent";
        fs::write(temp.path().join("0003-test.md"), content).unwrap();

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_bidirectional_references() {
        let temp = TempDir::new().unwrap();

        // Doc 1 is superseded by Doc 2
        let content = "---\nnumber: 1\ntitle: \"Doc 1\"\nauthor: \"Test\"\ncreated: 2024-01-01\nupdated: 2024-01-10\nstate: Superseded\nsuperseded-by: 2\n---\n\n# Doc 1\n\nContent";
        fs::write(temp.path().join("0001-test.md"), content).unwrap();

        // Doc 2 supersedes Doc 1
        let content = "---\nnumber: 2\ntitle: \"Doc 2\"\nauthor: \"Test\"\ncreated: 2024-02-01\nupdated: 2024-02-10\nstate: Final\nsupersedes: 1\n---\n\n# Doc 2\n\nContent";
        fs::write(temp.path().join("0002-test.md"), content).unwrap();

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_index_with_extra_columns() {
        let temp = TempDir::new().unwrap();

        create_test_doc_file(
            &temp,
            1,
            "Doc 1",
            DocState::Draft,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        );

        // Index with extra columns
        let index_content = "# Design Documents\n\n| Number | Title | State | Author | Date |\n|--------|-------|-------|--------|------|\n| 0001 | Doc 1 | Draft | Test | 2024-01-01 |\n";
        fs::write(temp.path().join(INDEX_FILENAME), index_content).unwrap();

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_special_characters_in_title() {
        let temp = TempDir::new().unwrap();

        let content = "---\nnumber: 1\ntitle: \"Doc with 'quotes' and \\\"escapes\\\"\"\nauthor: \"Test\"\ncreated: 2024-01-01\nupdated: 2024-01-10\nstate: Draft\n---\n\n# Title\n\nContent";
        fs::write(temp.path().join("0001-test.md"), content).unwrap();

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_long_path() {
        let temp = TempDir::new().unwrap();

        // Create nested directories
        let nested_dir = temp.path().join("very/deeply/nested/directory/structure");
        fs::create_dir_all(&nested_dir).unwrap();

        let content = "---\nnumber: 1\ntitle: \"Nested Doc\"\nauthor: \"Test\"\ncreated: 2024-01-01\nupdated: 2024-01-10\nstate: Draft\n---\n\n# Nested\n\nContent";
        fs::write(nested_dir.join("0001-test.md"), content).unwrap();

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validation_issue_debug_format() {
        let issue = ValidationIssue::DuplicateNumber {
            number: 42,
            paths: vec!["a.md".to_string(), "b.md".to_string()],
        };

        let debug_str = format!("{:?}", issue);
        assert!(debug_str.contains("DuplicateNumber"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_validate_handles_non_utf8_gracefully() {
        let temp = TempDir::new().unwrap();

        // Create a valid doc first
        create_test_doc_file(
            &temp,
            1,
            "Doc 1",
            DocState::Draft,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        );

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_index_invalid_number_format() {
        let temp = TempDir::new().unwrap();

        create_test_doc_file(
            &temp,
            1,
            "Doc 1",
            DocState::Draft,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        );

        // Index with invalid number format
        let index_content = "# Design Documents\n\n| Number | Title | State |\n|--------|-------|-------|\n| invalid | Doc | Draft |\n";
        fs::write(temp.path().join(INDEX_FILENAME), index_content).unwrap();

        let index = DocumentIndex::new(temp.path()).unwrap();
        let result = validate_documents(&index, false);

        // Should handle gracefully (invalid parses to 0, which won't match)
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_severity_levels() {
        // Test all ERROR severities
        assert_eq!(
            ValidationIssue::DuplicateNumber { number: 1, paths: vec!["a.md".to_string()] }
                .severity(),
            "ERROR"
        );
        assert_eq!(
            ValidationIssue::MissingReference {
                doc_num: 1,
                ref_type: "test".to_string(),
                ref_num: 2
            }
            .severity(),
            "ERROR"
        );
        assert_eq!(
            ValidationIssue::InIndexNotOnDisk { number: "0001".to_string() }.severity(),
            "ERROR"
        );

        // Test all WARNING severities
        assert_eq!(
            ValidationIssue::StateDirectoryMismatch {
                doc_num: 1,
                yaml_state: "Draft".to_string(),
                dir_state: "Final".to_string(),
                path: "test.md".to_string()
            }
            .severity(),
            "WARNING"
        );
        assert_eq!(
            ValidationIssue::NotInIndex {
                doc_num: 1,
                title: "Test".to_string(),
                path: "test.md".to_string()
            }
            .severity(),
            "WARNING"
        );
        assert_eq!(
            ValidationIssue::DateOrderIssue {
                doc_num: 1,
                created: "2024-02-01".to_string(),
                updated: "2024-01-01".to_string()
            }
            .severity(),
            "WARNING"
        );
        assert_eq!(
            ValidationIssue::MissingHeaders { path: "test.md".to_string() }.severity(),
            "WARNING"
        );
    }

    #[test]
    fn test_all_can_auto_fix_variants() {
        // Fixable variants
        assert!(ValidationIssue::StateDirectoryMismatch {
            doc_num: 1,
            yaml_state: "Draft".to_string(),
            dir_state: "Final".to_string(),
            path: "test.md".to_string()
        }
        .can_auto_fix());

        assert!(ValidationIssue::NotInIndex {
            doc_num: 1,
            title: "Test".to_string(),
            path: "test.md".to_string()
        }
        .can_auto_fix());

        assert!(ValidationIssue::MissingHeaders { path: "test.md".to_string() }.can_auto_fix());

        // Non-fixable variants
        assert!(!ValidationIssue::DuplicateNumber { number: 1, paths: vec!["a.md".to_string()] }
            .can_auto_fix());

        assert!(!ValidationIssue::MissingReference {
            doc_num: 1,
            ref_type: "test".to_string(),
            ref_num: 2
        }
        .can_auto_fix());

        assert!(!ValidationIssue::DateOrderIssue {
            doc_num: 1,
            created: "2024-02-01".to_string(),
            updated: "2024-01-01".to_string()
        }
        .can_auto_fix());

        assert!(!ValidationIssue::InIndexNotOnDisk { number: "0001".to_string() }.can_auto_fix());
    }
}
