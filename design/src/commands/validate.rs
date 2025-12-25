//! Validate command implementation

use anyhow::Result;
use colored::*;
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
    let index_path = PathBuf::from(index.docs_dir()).join("00-index.md");
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
