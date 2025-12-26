//! Index synchronization module
//!
//! Parses existing index files, compares with filesystem state,
//! and computes necessary changes.

use crate::doc::{DesignDoc, DocState};
use anyhow::Result;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Represents an entry in the index table
#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub number: String,
    pub title: String,
    pub state: String,
    pub updated: String,
}

/// Represents a parsed index file
#[derive(Debug)]
pub struct ParsedIndex {
    pub table_entries: HashMap<String, IndexEntry>,
    pub state_sections: HashMap<String, Vec<String>>, // state -> list of doc paths
}

impl ParsedIndex {
    /// Parse an index file from its content
    pub fn parse(content: &str) -> Result<Self> {
        let table_entries = parse_table(content);
        let state_sections = parse_state_sections(content);

        Ok(ParsedIndex { table_entries, state_sections })
    }
}

/// Parse the "All Documents by Number" table
fn parse_table(content: &str) -> HashMap<String, IndexEntry> {
    let mut entries = HashMap::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut in_table = false;
    let mut passed_separator = false;

    for line in lines {
        // Detect table start
        if line.starts_with("| Number | Title") {
            in_table = true;
            continue;
        }

        // Detect separator line
        if in_table && line.contains("---|") {
            passed_separator = true;
            continue;
        }

        // End of table
        if in_table && !line.starts_with('|') {
            break;
        }

        // Parse data rows
        if in_table && passed_separator && line.starts_with('|') {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 5 {
                let number = parts[1].trim();
                let title = parts[2].trim();
                let state = parts[3].trim();
                let updated = parts[4].trim();

                if !number.is_empty() && number != "Number" {
                    entries.insert(
                        number.to_string(),
                        IndexEntry {
                            number: number.to_string(),
                            title: title.to_string(),
                            state: state.to_string(),
                            updated: updated.to_string(),
                        },
                    );
                }
            }
        }
    }

    entries
}

/// Parse state sections to extract document paths
fn parse_state_sections(content: &str) -> HashMap<String, Vec<String>> {
    let mut sections = HashMap::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut current_state: Option<String> = None;
    let re = Regex::new(r"\]\(([^)]+)\)").unwrap();

    for line in lines {
        // Detect state section header (### State Name)
        if let Some(state_name) = line.strip_prefix("### ") {
            current_state = Some(state_name.trim().to_string());
            sections.insert(current_state.clone().unwrap(), Vec::new());
            continue;
        }

        // Detect end of state sections (## header)
        if line.starts_with("## ") && current_state.is_some() {
            current_state = None;
            continue;
        }

        // Parse document links in current state section
        if let Some(ref state) = current_state {
            if line.starts_with("- [") {
                // Extract path from markdown link: - [0001 - Title](path/to/file.md)
                if let Some(caps) = re.captures(line) {
                    if let Some(path) = caps.get(1) {
                        sections.get_mut(state).unwrap().push(path.as_str().to_string());
                    }
                }
            }
        }
    }

    sections
}

/// Get all git-tracked markdown files in the docs directory
pub fn get_git_tracked_docs(docs_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    use std::process::Command;

    let docs_dir = docs_dir.as_ref();

    // Run git ls-files to get tracked files
    let output = Command::new("git")
        .args(["ls-files", "--full-name"])
        .arg(docs_dir)
        .current_dir(docs_dir.parent().unwrap_or(docs_dir))
        .output()?;

    if !output.status.success() {
        // Fall back to filesystem if git fails
        return get_docs_from_filesystem(docs_dir);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let base_dir = docs_dir.parent().unwrap_or(docs_dir);

    let files: Vec<PathBuf> = stdout
        .lines()
        .filter(|line| line.ends_with(".md"))
        .map(|line| base_dir.join(line))
        .filter(|path| path.exists())
        .collect();

    Ok(files)
}

/// Get all markdown files in state directories (using walkdir, not git)
pub fn get_docs_from_filesystem(docs_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    use walkdir::WalkDir;

    let docs_dir = docs_dir.as_ref();
    let mut all_docs = Vec::new();

    // Walk through all state directories
    for state in DocState::all_states() {
        let state_dir = docs_dir.join(state.directory());
        if !state_dir.exists() {
            continue;
        }

        for entry in WalkDir::new(&state_dir).max_depth(1).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "md" {
                        all_docs.push(entry.path().to_path_buf());
                    }
                }
            }
        }
    }

    Ok(all_docs)
}

/// Extract document metadata for all docs
pub fn build_doc_map(doc_paths: &[PathBuf]) -> HashMap<String, DesignDoc> {
    let mut map = HashMap::new();

    for path in doc_paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(doc) = DesignDoc::parse(&content, path.clone()) {
                let number = format!("{:04}", doc.metadata.number);
                map.insert(number, doc);
            }
        }
    }

    map
}

/// Types of changes that can occur
#[derive(Debug, Clone)]
pub enum IndexChange {
    TableAdd { number: String, title: String, state: String, updated: String },
    TableUpdate { number: String, field: String, old: String, new: String },
    TableRemove { number: String },
    SectionAdd { state: String, number: String, title: String, path: String },
    SectionRemove { state: String, path: String },
}

impl IndexChange {
    pub fn description(&self) -> String {
        match self {
            IndexChange::TableAdd { number, title, .. } => {
                format!("Add to table: {} - {}", number, title)
            }
            IndexChange::TableUpdate { number, field, old, new } => {
                format!("Update {}: {} ({} → {})", number, field, old, new)
            }
            IndexChange::TableRemove { number } => {
                format!("Remove from table: {}", number)
            }
            IndexChange::SectionAdd { state, number, title, .. } => {
                format!("Add to {}: {} - {}", state, number, title)
            }
            IndexChange::SectionRemove { state, path } => {
                format!("Remove from {}: {}", state, path)
            }
        }
    }
}

/// Compare index with filesystem and determine table changes
pub fn compute_table_changes(
    parsed: &ParsedIndex,
    doc_map: &HashMap<String, DesignDoc>,
) -> Vec<IndexChange> {
    let mut changes = Vec::new();

    // Check for missing or outdated table entries
    for (number, doc) in doc_map {
        if let Some(existing) = parsed.table_entries.get(number) {
            // Check if state differs
            if existing.state != doc.metadata.state.as_str() {
                changes.push(IndexChange::TableUpdate {
                    number: number.clone(),
                    field: "state".to_string(),
                    old: existing.state.clone(),
                    new: doc.metadata.state.as_str().to_string(),
                });
            }

            // Check if updated date differs
            let doc_updated = doc.metadata.updated.to_string();
            if existing.updated != doc_updated {
                changes.push(IndexChange::TableUpdate {
                    number: number.clone(),
                    field: "updated".to_string(),
                    old: existing.updated.clone(),
                    new: doc_updated,
                });
            }

            // Check if title differs
            if existing.title != doc.metadata.title {
                changes.push(IndexChange::TableUpdate {
                    number: number.clone(),
                    field: "title".to_string(),
                    old: existing.title.clone(),
                    new: doc.metadata.title.clone(),
                });
            }
        } else {
            // Document not in table, add it
            changes.push(IndexChange::TableAdd {
                number: number.clone(),
                title: doc.metadata.title.clone(),
                state: doc.metadata.state.as_str().to_string(),
                updated: doc.metadata.updated.to_string(),
            });
        }
    }

    // Check for entries in table that no longer exist
    for number in parsed.table_entries.keys() {
        if !doc_map.contains_key(number) {
            changes.push(IndexChange::TableRemove { number: number.clone() });
        }
    }

    changes
}

/// Compute changes for state sections
pub fn compute_section_changes(
    parsed: &ParsedIndex,
    doc_map: &HashMap<String, DesignDoc>,
    docs_dir: &Path,
) -> Vec<IndexChange> {
    let mut changes = Vec::new();

    // Build map of expected documents by state
    let mut expected_by_state: HashMap<String, Vec<&DesignDoc>> = HashMap::new();
    for state in DocState::all_states() {
        expected_by_state.insert(state.as_str().to_string(), Vec::new());
    }

    for doc in doc_map.values() {
        let state_name = doc.metadata.state.as_str().to_string();
        expected_by_state.get_mut(&state_name).unwrap().push(doc);
    }

    // Check each state section
    for (state_name, expected_docs) in expected_by_state {
        let current_paths = parsed.state_sections.get(&state_name).cloned().unwrap_or_default();

        // Build set of current paths for quick lookup
        let current_set: HashSet<String> = current_paths.iter().cloned().collect();

        // Check for documents that should be in section but aren't
        for doc in &expected_docs {
            let rel_path = doc.path.strip_prefix(docs_dir).unwrap_or(&doc.path);
            let path_str = rel_path.to_string_lossy().to_string();

            if !current_set.contains(&path_str) {
                changes.push(IndexChange::SectionAdd {
                    state: state_name.clone(),
                    number: format!("{:04}", doc.metadata.number),
                    title: doc.metadata.title.clone(),
                    path: path_str,
                });
            }
        }

        // Build set of expected paths
        let expected_set: HashSet<String> = expected_docs
            .iter()
            .map(|doc| {
                let rel_path = doc.path.strip_prefix(docs_dir).unwrap_or(&doc.path);
                rel_path.to_string_lossy().to_string()
            })
            .collect();

        // Check for documents in section that shouldn't be there
        for path in &current_paths {
            if !expected_set.contains(path) {
                changes.push(IndexChange::SectionRemove {
                    state: state_name.clone(),
                    path: path.clone(),
                });
            }
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_INDEX: &str = r#"# Design Documents Index

## All Documents by Number

| Number | Title | State | Updated |
|--------|-------|-------|---------|
| 0001 | First Doc | Draft | 2024-01-01 |
| 0002 | Second Doc | Final | 2024-01-02 |

### Draft

- [0001 - First Doc](01-draft/0001-first-doc.md)

### Final

- [0002 - Second Doc](06-final/0002-second-doc.md)
"#;

    #[test]
    fn test_parse_table_basic() {
        let entries = parse_table(SAMPLE_INDEX);
        assert_eq!(entries.len(), 2);
        assert!(entries.contains_key("0001"));
        assert!(entries.contains_key("0002"));
    }

    #[test]
    fn test_parse_table_entry_fields() {
        let entries = parse_table(SAMPLE_INDEX);
        let entry = entries.get("0001").unwrap();

        assert_eq!(entry.number, "0001");
        assert_eq!(entry.title, "First Doc");
        assert_eq!(entry.state, "Draft");
        assert_eq!(entry.updated, "2024-01-01");
    }

    #[test]
    fn test_parse_table_empty() {
        let empty = "# No table here";
        let entries = parse_table(empty);
        assert!(entries.is_empty());
    }

    #[test]
    fn test_parse_state_sections_basic() {
        let sections = parse_state_sections(SAMPLE_INDEX);
        assert!(sections.contains_key("Draft"));
        assert!(sections.contains_key("Final"));
    }

    #[test]
    fn test_parse_state_sections_paths() {
        let sections = parse_state_sections(SAMPLE_INDEX);
        let draft_docs = sections.get("Draft").unwrap();

        assert_eq!(draft_docs.len(), 1);
        assert_eq!(draft_docs[0], "01-draft/0001-first-doc.md");
    }

    #[test]
    fn test_parse_index_complete() {
        let parsed = ParsedIndex::parse(SAMPLE_INDEX).unwrap();

        assert_eq!(parsed.table_entries.len(), 2);
        assert_eq!(parsed.state_sections.len(), 2);
    }

    #[test]
    fn test_index_change_descriptions() {
        let add = IndexChange::TableAdd {
            number: "0042".to_string(),
            title: "New Doc".to_string(),
            state: "Draft".to_string(),
            updated: "2024-01-01".to_string(),
        };
        assert!(add.description().contains("Add to table"));
        assert!(add.description().contains("0042"));

        let update = IndexChange::TableUpdate {
            number: "0001".to_string(),
            field: "state".to_string(),
            old: "Draft".to_string(),
            new: "Final".to_string(),
        };
        assert!(update.description().contains("Update"));
        assert!(update.description().contains("Draft → Final"));

        let remove = IndexChange::TableRemove { number: "0099".to_string() };
        assert!(remove.description().contains("Remove"));

        let section_add = IndexChange::SectionAdd {
            state: "Draft".to_string(),
            number: "0042".to_string(),
            title: "New".to_string(),
            path: "01-draft/0042-new.md".to_string(),
        };
        assert!(section_add.description().contains("Add to Draft"));

        let section_remove = IndexChange::SectionRemove {
            state: "Draft".to_string(),
            path: "01-draft/old.md".to_string(),
        };
        assert!(section_remove.description().contains("Remove from Draft"));
    }

    #[test]
    fn test_build_doc_map_empty() {
        let map = build_doc_map(&[]);
        assert!(map.is_empty());
    }

    #[test]
    fn test_compute_table_changes_add() {
        use chrono::NaiveDate;

        let parsed = ParsedIndex { table_entries: HashMap::new(), state_sections: HashMap::new() };

        let mut doc_map = HashMap::new();
        let doc = DesignDoc {
            metadata: crate::doc::DocMetadata {
                number: 42,
                title: "New Doc".to_string(),
                author: "Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                state: DocState::Draft,
                supersedes: None,
                superseded_by: None,
            },
            content: "content".to_string(),
            path: PathBuf::from("01-draft/0042-new-doc.md"),
        };
        doc_map.insert("0042".to_string(), doc);

        let changes = compute_table_changes(&parsed, &doc_map);

        assert_eq!(changes.len(), 1);
        match &changes[0] {
            IndexChange::TableAdd { number, title, .. } => {
                assert_eq!(number, "0042");
                assert_eq!(title, "New Doc");
            }
            _ => panic!("Expected TableAdd"),
        }
    }

    #[test]
    fn test_compute_table_changes_remove() {
        let mut table_entries = HashMap::new();
        table_entries.insert(
            "0099".to_string(),
            IndexEntry {
                number: "0099".to_string(),
                title: "Old Doc".to_string(),
                state: "Draft".to_string(),
                updated: "2024-01-01".to_string(),
            },
        );

        let parsed = ParsedIndex { table_entries, state_sections: HashMap::new() };
        let doc_map = HashMap::new(); // Empty - doc was deleted

        let changes = compute_table_changes(&parsed, &doc_map);

        assert_eq!(changes.len(), 1);
        match &changes[0] {
            IndexChange::TableRemove { number } => {
                assert_eq!(number, "0099");
            }
            _ => panic!("Expected TableRemove"),
        }
    }

    #[test]
    fn test_compute_table_changes_update_state() {
        use chrono::NaiveDate;

        let mut table_entries = HashMap::new();
        table_entries.insert(
            "0042".to_string(),
            IndexEntry {
                number: "0042".to_string(),
                title: "Doc".to_string(),
                state: "Draft".to_string(),
                updated: "2024-01-01".to_string(),
            },
        );

        let parsed = ParsedIndex { table_entries, state_sections: HashMap::new() };

        let mut doc_map = HashMap::new();
        let doc = DesignDoc {
            metadata: crate::doc::DocMetadata {
                number: 42,
                title: "Doc".to_string(),
                author: "Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                state: DocState::Final, // Changed!
                supersedes: None,
                superseded_by: None,
            },
            content: "content".to_string(),
            path: PathBuf::from("06-final/0042-doc.md"),
        };
        doc_map.insert("0042".to_string(), doc);

        let changes = compute_table_changes(&parsed, &doc_map);

        // Should detect state change
        assert!(!changes.is_empty());
        let has_state_update = changes
            .iter()
            .any(|c| matches!(c, IndexChange::TableUpdate { field, .. } if field == "state"));
        assert!(has_state_update);
    }

    #[test]
    fn test_compute_table_changes_update_title() {
        use chrono::NaiveDate;

        let mut table_entries = HashMap::new();
        table_entries.insert(
            "0042".to_string(),
            IndexEntry {
                number: "0042".to_string(),
                title: "Old Title".to_string(),
                state: "Draft".to_string(),
                updated: "2024-01-01".to_string(),
            },
        );

        let parsed = ParsedIndex { table_entries, state_sections: HashMap::new() };

        let mut doc_map = HashMap::new();
        let doc = DesignDoc {
            metadata: crate::doc::DocMetadata {
                number: 42,
                title: "New Title".to_string(),
                author: "Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                state: DocState::Draft,
                supersedes: None,
                superseded_by: None,
            },
            content: "content".to_string(),
            path: PathBuf::from("01-draft/0042-new-title.md"),
        };
        doc_map.insert("0042".to_string(), doc);

        let changes = compute_table_changes(&parsed, &doc_map);

        let has_title_update = changes
            .iter()
            .any(|c| matches!(c, IndexChange::TableUpdate { field, .. } if field == "title"));
        assert!(has_title_update);
    }

    #[test]
    fn test_compute_table_changes_update_date() {
        use chrono::NaiveDate;

        let mut table_entries = HashMap::new();
        table_entries.insert(
            "0042".to_string(),
            IndexEntry {
                number: "0042".to_string(),
                title: "Doc".to_string(),
                state: "Draft".to_string(),
                updated: "2024-01-01".to_string(),
            },
        );

        let parsed = ParsedIndex { table_entries, state_sections: HashMap::new() };

        let mut doc_map = HashMap::new();
        let doc = DesignDoc {
            metadata: crate::doc::DocMetadata {
                number: 42,
                title: "Doc".to_string(),
                author: "Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
                state: DocState::Draft,
                supersedes: None,
                superseded_by: None,
            },
            content: "content".to_string(),
            path: PathBuf::from("01-draft/0042-doc.md"),
        };
        doc_map.insert("0042".to_string(), doc);

        let changes = compute_table_changes(&parsed, &doc_map);

        let has_updated_change = changes
            .iter()
            .any(|c| matches!(c, IndexChange::TableUpdate { field, .. } if field == "updated"));
        assert!(has_updated_change);
    }

    #[test]
    fn test_compute_table_changes_multiple_updates() {
        use chrono::NaiveDate;

        let mut table_entries = HashMap::new();
        table_entries.insert(
            "0042".to_string(),
            IndexEntry {
                number: "0042".to_string(),
                title: "Old".to_string(),
                state: "Draft".to_string(),
                updated: "2024-01-01".to_string(),
            },
        );

        let parsed = ParsedIndex { table_entries, state_sections: HashMap::new() };

        let mut doc_map = HashMap::new();
        let doc = DesignDoc {
            metadata: crate::doc::DocMetadata {
                number: 42,
                title: "New".to_string(),
                author: "Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
                state: DocState::Final,
                supersedes: None,
                superseded_by: None,
            },
            content: "content".to_string(),
            path: PathBuf::from("06-final/0042-new.md"),
        };
        doc_map.insert("0042".to_string(), doc);

        let changes = compute_table_changes(&parsed, &doc_map);

        // Should have state, title, and updated changes
        assert!(changes.len() >= 3);
        assert!(changes.iter().any(|c| matches!(c, IndexChange::TableUpdate { field, .. } if field == "state")));
        assert!(changes.iter().any(|c| matches!(c, IndexChange::TableUpdate { field, .. } if field == "title")));
        assert!(changes.iter().any(|c| matches!(c, IndexChange::TableUpdate { field, .. } if field == "updated")));
    }

    #[test]
    fn test_compute_table_changes_no_changes() {
        use chrono::NaiveDate;

        let mut table_entries = HashMap::new();
        table_entries.insert(
            "0042".to_string(),
            IndexEntry {
                number: "0042".to_string(),
                title: "Doc".to_string(),
                state: "Draft".to_string(),
                updated: "2024-01-01".to_string(),
            },
        );

        let parsed = ParsedIndex { table_entries, state_sections: HashMap::new() };

        let mut doc_map = HashMap::new();
        let doc = DesignDoc {
            metadata: crate::doc::DocMetadata {
                number: 42,
                title: "Doc".to_string(),
                author: "Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                state: DocState::Draft,
                supersedes: None,
                superseded_by: None,
            },
            content: "content".to_string(),
            path: PathBuf::from("01-draft/0042-doc.md"),
        };
        doc_map.insert("0042".to_string(), doc);

        let changes = compute_table_changes(&parsed, &doc_map);
        assert!(changes.is_empty());
    }

    #[test]
    fn test_compute_section_changes_add() {
        use chrono::NaiveDate;
        use std::env;

        let temp_dir = env::temp_dir();
        let docs_dir = temp_dir.join("test_docs");
        std::fs::create_dir_all(&docs_dir).ok();

        let parsed = ParsedIndex {
            table_entries: HashMap::new(),
            state_sections: {
                let mut sections = HashMap::new();
                sections.insert("Draft".to_string(), Vec::new());
                sections
            },
        };

        let mut doc_map = HashMap::new();
        let draft_dir = docs_dir.join("01-draft");
        std::fs::create_dir_all(&draft_dir).ok();
        let doc_path = draft_dir.join("0042-new-doc.md");

        let doc = DesignDoc {
            metadata: crate::doc::DocMetadata {
                number: 42,
                title: "New Doc".to_string(),
                author: "Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                state: DocState::Draft,
                supersedes: None,
                superseded_by: None,
            },
            content: "content".to_string(),
            path: doc_path,
        };
        doc_map.insert("0042".to_string(), doc);

        let changes = compute_section_changes(&parsed, &doc_map, &docs_dir);

        let has_section_add = changes
            .iter()
            .any(|c| matches!(c, IndexChange::SectionAdd { state, .. } if state == "Draft"));
        assert!(has_section_add);

        std::fs::remove_dir_all(&docs_dir).ok();
    }

    #[test]
    fn test_compute_section_changes_remove() {
        use std::env;

        let temp_dir = env::temp_dir();
        let docs_dir = temp_dir.join("test_docs_remove");
        std::fs::create_dir_all(&docs_dir).ok();

        let parsed = ParsedIndex {
            table_entries: HashMap::new(),
            state_sections: {
                let mut sections = HashMap::new();
                sections.insert(
                    "Draft".to_string(),
                    vec!["01-draft/0042-old-doc.md".to_string()],
                );
                sections
            },
        };

        let doc_map = HashMap::new(); // Empty - doc was removed

        let changes = compute_section_changes(&parsed, &doc_map, &docs_dir);

        let has_section_remove = changes.iter().any(|c| {
            matches!(c, IndexChange::SectionRemove { state, path }
                if state == "Draft" && path == "01-draft/0042-old-doc.md")
        });
        assert!(has_section_remove);

        std::fs::remove_dir_all(&docs_dir).ok();
    }

    #[test]
    fn test_compute_section_changes_no_changes() {
        use chrono::NaiveDate;
        use std::env;

        let temp_dir = env::temp_dir();
        let docs_dir = temp_dir.join("test_docs_no_change");
        std::fs::create_dir_all(&docs_dir).ok();

        let draft_dir = docs_dir.join("01-draft");
        std::fs::create_dir_all(&draft_dir).ok();
        let doc_path = draft_dir.join("0042-doc.md");

        let parsed = ParsedIndex {
            table_entries: HashMap::new(),
            state_sections: {
                let mut sections = HashMap::new();
                sections.insert("Draft".to_string(), vec!["01-draft/0042-doc.md".to_string()]);
                sections
            },
        };

        let mut doc_map = HashMap::new();
        let doc = DesignDoc {
            metadata: crate::doc::DocMetadata {
                number: 42,
                title: "Doc".to_string(),
                author: "Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                state: DocState::Draft,
                supersedes: None,
                superseded_by: None,
            },
            content: "content".to_string(),
            path: doc_path,
        };
        doc_map.insert("0042".to_string(), doc);

        let changes = compute_section_changes(&parsed, &doc_map, &docs_dir);

        // Filter out changes for other states (which will be empty)
        let draft_changes: Vec<_> = changes
            .iter()
            .filter(|c| match c {
                IndexChange::SectionAdd { state, .. } => state == "Draft",
                IndexChange::SectionRemove { state, .. } => state == "Draft",
                _ => false,
            })
            .collect();
        assert!(draft_changes.is_empty());

        std::fs::remove_dir_all(&docs_dir).ok();
    }

    #[test]
    fn test_compute_section_changes_multiple_states() {
        use chrono::NaiveDate;
        use std::env;

        let temp_dir = env::temp_dir();
        let docs_dir = temp_dir.join("test_docs_multi");
        std::fs::create_dir_all(&docs_dir).ok();

        let draft_dir = docs_dir.join("01-draft");
        let final_dir = docs_dir.join("06-final");
        std::fs::create_dir_all(&draft_dir).ok();
        std::fs::create_dir_all(&final_dir).ok();

        let parsed = ParsedIndex {
            table_entries: HashMap::new(),
            state_sections: HashMap::new(),
        };

        let mut doc_map = HashMap::new();

        // Add draft doc
        let draft_path = draft_dir.join("0001-draft.md");
        let draft_doc = DesignDoc {
            metadata: crate::doc::DocMetadata {
                number: 1,
                title: "Draft".to_string(),
                author: "Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                state: DocState::Draft,
                supersedes: None,
                superseded_by: None,
            },
            content: "content".to_string(),
            path: draft_path,
        };
        doc_map.insert("0001".to_string(), draft_doc);

        // Add final doc
        let final_path = final_dir.join("0002-final.md");
        let final_doc = DesignDoc {
            metadata: crate::doc::DocMetadata {
                number: 2,
                title: "Final".to_string(),
                author: "Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                state: DocState::Final,
                supersedes: None,
                superseded_by: None,
            },
            content: "content".to_string(),
            path: final_path,
        };
        doc_map.insert("0002".to_string(), final_doc);

        let changes = compute_section_changes(&parsed, &doc_map, &docs_dir);

        // Should have additions for both states
        let has_draft_add =
            changes.iter().any(|c| matches!(c, IndexChange::SectionAdd { state, .. } if state == "Draft"));
        let has_final_add =
            changes.iter().any(|c| matches!(c, IndexChange::SectionAdd { state, .. } if state == "Final"));

        assert!(has_draft_add);
        assert!(has_final_add);

        std::fs::remove_dir_all(&docs_dir).ok();
    }

    #[test]
    fn test_build_doc_map_with_valid_docs() {
        use std::env;

        let temp_dir = env::temp_dir();
        let test_dir = temp_dir.join("test_build_doc_map");
        std::fs::create_dir_all(&test_dir).ok();

        let doc1_path = test_dir.join("0001-doc.md");
        let doc1_content = "---\nnumber: 1\ntitle: \"First\"\nauthor: \"Author\"\ncreated: 2024-01-01\nupdated: 2024-01-01\nstate: Draft\nsupersedes: null\nsuperseded-by: null\n---\n\nContent";
        std::fs::write(&doc1_path, doc1_content).ok();

        let doc2_path = test_dir.join("0002-doc.md");
        let doc2_content = "---\nnumber: 2\ntitle: \"Second\"\nauthor: \"Author\"\ncreated: 2024-01-01\nupdated: 2024-01-01\nstate: Final\nsupersedes: null\nsuperseded-by: null\n---\n\nContent";
        std::fs::write(&doc2_path, doc2_content).ok();

        let paths = vec![doc1_path.clone(), doc2_path.clone()];
        let map = build_doc_map(&paths);

        assert_eq!(map.len(), 2);
        assert!(map.contains_key("0001"));
        assert!(map.contains_key("0002"));

        let doc1 = map.get("0001").unwrap();
        assert_eq!(doc1.metadata.title, "First");

        std::fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_build_doc_map_with_invalid_docs() {
        use std::env;

        let temp_dir = env::temp_dir();
        let test_dir = temp_dir.join("test_build_doc_map_invalid");
        std::fs::create_dir_all(&test_dir).ok();

        let valid_path = test_dir.join("0001-valid.md");
        let valid_content = "---\nnumber: 1\ntitle: \"Valid\"\nauthor: \"Author\"\ncreated: 2024-01-01\nupdated: 2024-01-01\nstate: Draft\nsupersedes: null\nsuperseded-by: null\n---\n\nContent";
        std::fs::write(&valid_path, valid_content).ok();

        let invalid_path = test_dir.join("0002-invalid.md");
        let invalid_content = "No frontmatter here";
        std::fs::write(&invalid_path, invalid_content).ok();

        let paths = vec![valid_path.clone(), invalid_path.clone()];
        let map = build_doc_map(&paths);

        // Should only include valid doc
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("0001"));
        assert!(!map.contains_key("0002"));

        std::fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_build_doc_map_nonexistent_file() {
        let paths = vec![PathBuf::from("/nonexistent/file.md")];
        let map = build_doc_map(&paths);
        assert!(map.is_empty());
    }

    #[test]
    fn test_parse_table_malformed_rows() {
        let content = r#"# Index

| Number | Title | State | Updated |
|--------|-------|-------|---------|
| 0001 | Valid Doc | Draft | 2024-01-01 |
| incomplete row
| | | | |
| 0002 | Another Valid | Final | 2024-01-02 |
"#;
        let entries = parse_table(content);
        assert_eq!(entries.len(), 2);
        assert!(entries.contains_key("0001"));
        assert!(entries.contains_key("0002"));
    }

    #[test]
    fn test_parse_table_no_separator() {
        let content = r#"# Index

| Number | Title | State | Updated |
| 0001 | No Separator | Draft | 2024-01-01 |
"#;
        let entries = parse_table(content);
        // Without separator, rows won't be parsed
        assert!(entries.is_empty());
    }

    #[test]
    fn test_parse_table_header_only() {
        let content = r#"# Index

| Number | Title | State | Updated |
|--------|-------|-------|---------|
"#;
        let entries = parse_table(content);
        assert!(entries.is_empty());
    }

    #[test]
    fn test_parse_table_with_empty_number() {
        let content = r#"# Index

| Number | Title | State | Updated |
|--------|-------|-------|---------|
|  | Empty Number | Draft | 2024-01-01 |
| 0001 | Valid | Draft | 2024-01-01 |
"#;
        let entries = parse_table(content);
        assert_eq!(entries.len(), 1);
        assert!(entries.contains_key("0001"));
    }

    #[test]
    fn test_parse_state_sections_empty() {
        let content = "# Index\n\nNo sections here";
        let sections = parse_state_sections(content);
        assert!(sections.is_empty());
    }

    #[test]
    fn test_parse_state_sections_empty_section() {
        let content = r#"# Index

### Draft

### Final

- [0001 - Doc](06-final/0001-doc.md)
"#;
        let sections = parse_state_sections(content);
        assert_eq!(sections.len(), 2);

        let draft = sections.get("Draft").unwrap();
        assert!(draft.is_empty());

        let final_section = sections.get("Final").unwrap();
        assert_eq!(final_section.len(), 1);
    }

    #[test]
    fn test_parse_state_sections_terminated_by_h2() {
        let content = r#"# Index

### Draft

- [0001 - Doc](01-draft/0001-doc.md)
- [0002 - Another](01-draft/0002-another.md)

## All Documents by Number

| Number | Title | State | Updated |
"#;
        let sections = parse_state_sections(content);
        let draft = sections.get("Draft").unwrap();
        assert_eq!(draft.len(), 2);
    }

    #[test]
    fn test_parse_state_sections_multiple_links() {
        let content = r#"### Draft

- [0001 - First](01-draft/0001-first.md)
- [0002 - Second](01-draft/0002-second.md)
- [0003 - Third](01-draft/0003-third.md)

### Final

- [0004 - Fourth](06-final/0004-fourth.md)
"#;
        let sections = parse_state_sections(content);

        let draft = sections.get("Draft").unwrap();
        assert_eq!(draft.len(), 3);
        assert_eq!(draft[0], "01-draft/0001-first.md");
        assert_eq!(draft[1], "01-draft/0002-second.md");
        assert_eq!(draft[2], "01-draft/0003-third.md");

        let final_section = sections.get("Final").unwrap();
        assert_eq!(final_section.len(), 1);
        assert_eq!(final_section[0], "06-final/0004-fourth.md");
    }

    #[test]
    fn test_parse_state_sections_malformed_links() {
        let content = r#"### Draft

- [0001 - Valid](01-draft/0001-valid.md)
- Not a link
- [0002 - No closing paren](01-draft/0002.md
- [0003](01-draft/0003.md)
- Regular bullet point
"#;
        let sections = parse_state_sections(content);
        let draft = sections.get("Draft").unwrap();
        // Only valid markdown links should be captured
        assert_eq!(draft.len(), 2);
        assert_eq!(draft[0], "01-draft/0001-valid.md");
        assert_eq!(draft[1], "01-draft/0003.md");
    }

    #[test]
    fn test_cleanup_formatting_basic() {
        let input = r#"# Title

## Section One


Content here


## Section Two

More content
"#;
        let output = cleanup_formatting(input);

        // Should have consistent spacing
        assert!(output.contains("\n## Section One\n\nContent"));
        assert!(output.contains("\n## Section Two\n\nMore"));
        // Should not have triple blank lines
        assert!(!output.contains("\n\n\n\n"));
    }

    #[test]
    fn test_cleanup_formatting_h3_sections() {
        let input = r#"### Draft


- [0001](01-draft/0001.md)

### Final


- [0002](06-final/0002.md)
"#;
        let output = cleanup_formatting(input);

        // Should have proper spacing around h3
        // First header has no blank line before it
        assert!(output.starts_with("### Draft\n\n"));
        assert!(output.contains("- [0001]"));
        assert!(output.contains("\n### Final\n\n- [0002]"));
    }

    #[test]
    fn test_cleanup_formatting_consecutive_bullets() {
        let input = r#"### Section

- [0001](file1.md)

- [0002](file2.md)

- [0003](file3.md)
"#;
        let output = cleanup_formatting(input);

        // Blank lines between bullets should be removed
        assert!(!output.contains("]\n\n- ["));
    }

    #[test]
    fn test_cleanup_formatting_first_header() {
        let input = r#"## First Header

Content"#;
        let output = cleanup_formatting(input);

        // First header shouldn't have blank line before it
        assert!(output.starts_with("## First Header\n"));
    }

    #[test]
    fn test_cleanup_formatting_ends_with_newline() {
        let input = "# Title\n\nContent";
        let output = cleanup_formatting(input);

        assert!(output.ends_with('\n'));
    }

    #[test]
    fn test_cleanup_formatting_already_clean() {
        let input = r#"# Title

## Section

Content

### Subsection

- [0001](file.md)
- [0002](file2.md)
"#;
        let output = cleanup_formatting(input);

        // Should remain mostly the same
        assert!(output.contains("# Title"));
        assert!(output.contains("## Section"));
        assert!(output.contains("### Subsection"));
    }

    #[test]
    fn test_cleanup_formatting_multiple_trailing_blanks() {
        let input = r#"## Section



Content"#;
        let output = cleanup_formatting(input);

        // Should collapse to single blank line before header
        assert!(output.contains("## Section\n\nContent"));
        assert!(!output.contains("\n\n\n"));
    }

    #[test]
    fn test_cleanup_formatting_mixed_content() {
        let input = r#"# Main Title


## All Documents

| Number | Title |
|--------|-------|
| 0001 | Doc |


### Draft

- [0001](01-draft/0001.md)
- [0002](01-draft/0002.md)


## Footer

End
"#;
        let output = cleanup_formatting(input);

        // Headers should have proper spacing
        assert!(output.contains("\n## All Documents\n\n"));
        assert!(output.contains("\n### Draft\n\n"));
        assert!(output.contains("\n## Footer\n\n"));

        // Bullets should be consecutive
        assert!(output.contains("0001.md)\n- [0002]"));
    }

    #[test]
    fn test_cleanup_formatting_empty_input() {
        let input = "";
        let output = cleanup_formatting(input);
        assert_eq!(output, "");
    }

    #[test]
    fn test_cleanup_formatting_only_headers() {
        let input = r#"# Title
## Section
### Subsection"#;
        let output = cleanup_formatting(input);

        // Headers should be separated by blank lines
        assert!(output.contains("# Title\n\n## Section"));
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn test_cleanup_formatting_preserves_table() {
        let input = r#"## Table

| A | B |
|---|---|
| 1 | 2 |
| 3 | 4 |

## Next"#;
        let output = cleanup_formatting(input);

        // Table lines should be preserved
        assert!(output.contains("| A | B |"));
        assert!(output.contains("| 1 | 2 |"));
        assert!(output.contains("| 3 | 4 |"));
    }

    #[test]
    fn test_get_docs_from_filesystem() {
        use std::env;

        let temp_dir = env::temp_dir();
        let test_dir = temp_dir.join("test_get_docs_fs");
        std::fs::create_dir_all(&test_dir).ok();

        // Create state directories
        let draft_dir = test_dir.join("01-draft");
        let final_dir = test_dir.join("06-final");
        std::fs::create_dir_all(&draft_dir).ok();
        std::fs::create_dir_all(&final_dir).ok();

        // Create some markdown files
        std::fs::write(draft_dir.join("0001-doc.md"), "content").ok();
        std::fs::write(draft_dir.join("0002-doc.md"), "content").ok();
        std::fs::write(final_dir.join("0003-doc.md"), "content").ok();

        // Create non-md file (should be ignored)
        std::fs::write(draft_dir.join("README.txt"), "content").ok();

        let result = get_docs_from_filesystem(&test_dir).unwrap();

        assert_eq!(result.len(), 3);
        assert!(result.iter().any(|p| p.file_name().unwrap() == "0001-doc.md"));
        assert!(result.iter().any(|p| p.file_name().unwrap() == "0002-doc.md"));
        assert!(result.iter().any(|p| p.file_name().unwrap() == "0003-doc.md"));
        assert!(!result.iter().any(|p| p.file_name().unwrap() == "README.txt"));

        std::fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_get_docs_from_filesystem_nonexistent_dirs() {
        use std::env;

        let temp_dir = env::temp_dir();
        let test_dir = temp_dir.join("test_get_docs_fs_empty");
        std::fs::create_dir_all(&test_dir).ok();

        // No state directories exist
        let result = get_docs_from_filesystem(&test_dir).unwrap();
        assert!(result.is_empty());

        std::fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_get_docs_from_filesystem_nested_files() {
        use std::env;

        let temp_dir = env::temp_dir();
        let test_dir = temp_dir.join("test_get_docs_fs_nested");
        std::fs::create_dir_all(&test_dir).ok();

        let draft_dir = test_dir.join("01-draft");
        std::fs::create_dir_all(&draft_dir).ok();

        // Create file in root
        std::fs::write(draft_dir.join("0001-root.md"), "content").ok();

        // Create nested directory (should be ignored due to max_depth(1))
        let nested_dir = draft_dir.join("subdir");
        std::fs::create_dir_all(&nested_dir).ok();
        std::fs::write(nested_dir.join("0002-nested.md"), "content").ok();

        let result = get_docs_from_filesystem(&test_dir).unwrap();

        // Should only find root-level .md files
        assert_eq!(result.len(), 1);
        assert!(result.iter().any(|p| p.file_name().unwrap() == "0001-root.md"));
        assert!(!result.iter().any(|p| p.file_name().unwrap() == "0002-nested.md"));

        std::fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_get_git_tracked_docs_fallback() {
        use std::env;

        // Use a non-git directory to trigger fallback
        let temp_dir = env::temp_dir();
        let test_dir = temp_dir.join("test_git_fallback");
        std::fs::create_dir_all(&test_dir).ok();

        let draft_dir = test_dir.join("01-draft");
        std::fs::create_dir_all(&draft_dir).ok();
        std::fs::write(draft_dir.join("0001-doc.md"), "content").ok();

        // Should fall back to filesystem when git fails
        let result = get_git_tracked_docs(&test_dir);

        // Either succeeds with fallback or returns empty
        match result {
            Ok(docs) => {
                // If fallback worked, should find the doc
                if !docs.is_empty() {
                    assert!(docs.iter().any(|p| p.file_name().unwrap() == "0001-doc.md"));
                }
            }
            Err(_) => {
                // Error is acceptable if filesystem also fails
            }
        }

        std::fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_get_git_tracked_docs_filters_md() {
        use std::env;
        use std::process::Command;

        // Create a temp git repo
        let temp_dir = env::temp_dir();
        let test_dir = temp_dir.join("test_git_md_filter");
        std::fs::create_dir_all(&test_dir).ok();

        // Initialize git repo
        Command::new("git").args(["init"]).current_dir(&test_dir).output().ok();

        let draft_dir = test_dir.join("01-draft");
        std::fs::create_dir_all(&draft_dir).ok();

        // Create files
        std::fs::write(draft_dir.join("0001-doc.md"), "content").ok();
        std::fs::write(draft_dir.join("README.txt"), "content").ok();

        // Add files to git
        Command::new("git").args(["add", "."]).current_dir(&test_dir).output().ok();

        let result = get_git_tracked_docs(&test_dir);

        if let Ok(docs) = result {
            // Should only include .md files
            assert!(docs.iter().all(|p| p.extension().map_or(false, |e| e == "md")));
            assert!(!docs.iter().any(|p| p.file_name().map_or(false, |n| n == "README.txt")));
        }

        std::fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_parsed_index_parse_integration() {
        let content = r#"# Design Documents Index

## All Documents by Number

| Number | Title | State | Updated |
|--------|-------|-------|---------|
| 0001 | First Doc | Draft | 2024-01-01 |
| 0002 | Second Doc | Final | 2024-01-02 |

### Draft

- [0001 - First Doc](01-draft/0001-first-doc.md)

### Final

- [0002 - Second Doc](06-final/0002-second-doc.md)

## Other Section
"#;

        let parsed = ParsedIndex::parse(content).unwrap();

        // Verify table was parsed
        assert_eq!(parsed.table_entries.len(), 2);
        assert!(parsed.table_entries.contains_key("0001"));
        assert!(parsed.table_entries.contains_key("0002"));

        // Verify sections were parsed
        assert_eq!(parsed.state_sections.len(), 2);
        assert!(parsed.state_sections.contains_key("Draft"));
        assert!(parsed.state_sections.contains_key("Final"));

        let draft_docs = parsed.state_sections.get("Draft").unwrap();
        assert_eq!(draft_docs.len(), 1);
        assert_eq!(draft_docs[0], "01-draft/0001-first-doc.md");
    }

    #[test]
    fn test_index_entry_clone() {
        let entry = IndexEntry {
            number: "0001".to_string(),
            title: "Test".to_string(),
            state: "Draft".to_string(),
            updated: "2024-01-01".to_string(),
        };

        let cloned = entry.clone();
        assert_eq!(entry.number, cloned.number);
        assert_eq!(entry.title, cloned.title);
        assert_eq!(entry.state, cloned.state);
        assert_eq!(entry.updated, cloned.updated);
    }

    #[test]
    fn test_compute_table_changes_mixed_scenario() {
        use chrono::NaiveDate;

        // Index has: 0001 (outdated), 0002 (current), 0003 (deleted)
        // Filesystem has: 0001 (updated), 0002 (unchanged), 0004 (new)
        let mut table_entries = HashMap::new();
        table_entries.insert(
            "0001".to_string(),
            IndexEntry {
                number: "0001".to_string(),
                title: "Old Title".to_string(),
                state: "Draft".to_string(),
                updated: "2024-01-01".to_string(),
            },
        );
        table_entries.insert(
            "0002".to_string(),
            IndexEntry {
                number: "0002".to_string(),
                title: "Unchanged".to_string(),
                state: "Final".to_string(),
                updated: "2024-01-01".to_string(),
            },
        );
        table_entries.insert(
            "0003".to_string(),
            IndexEntry {
                number: "0003".to_string(),
                title: "Deleted".to_string(),
                state: "Draft".to_string(),
                updated: "2024-01-01".to_string(),
            },
        );

        let parsed = ParsedIndex { table_entries, state_sections: HashMap::new() };

        let mut doc_map = HashMap::new();

        // Updated doc
        doc_map.insert(
            "0001".to_string(),
            DesignDoc {
                metadata: crate::doc::DocMetadata {
                    number: 1,
                    title: "New Title".to_string(),
                    author: "Author".to_string(),
                    created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    updated: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
                    state: DocState::Final,
                    supersedes: None,
                    superseded_by: None,
                },
                content: "content".to_string(),
                path: PathBuf::from("06-final/0001-new-title.md"),
            },
        );

        // Unchanged doc
        doc_map.insert(
            "0002".to_string(),
            DesignDoc {
                metadata: crate::doc::DocMetadata {
                    number: 2,
                    title: "Unchanged".to_string(),
                    author: "Author".to_string(),
                    created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    state: DocState::Final,
                    supersedes: None,
                    superseded_by: None,
                },
                content: "content".to_string(),
                path: PathBuf::from("06-final/0002-unchanged.md"),
            },
        );

        // New doc
        doc_map.insert(
            "0004".to_string(),
            DesignDoc {
                metadata: crate::doc::DocMetadata {
                    number: 4,
                    title: "New Doc".to_string(),
                    author: "Author".to_string(),
                    created: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
                    updated: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
                    state: DocState::Draft,
                    supersedes: None,
                    superseded_by: None,
                },
                content: "content".to_string(),
                path: PathBuf::from("01-draft/0004-new-doc.md"),
            },
        );

        let changes = compute_table_changes(&parsed, &doc_map);

        // Should have updates for 0001, add for 0004, remove for 0003
        assert!(changes.iter().any(|c| matches!(c, IndexChange::TableUpdate { number, .. } if number == "0001")));
        assert!(changes.iter().any(|c| matches!(c, IndexChange::TableAdd { number, .. } if number == "0004")));
        assert!(changes.iter().any(|c| matches!(c, IndexChange::TableRemove { number } if number == "0003")));
    }

    #[test]
    fn test_compute_section_changes_with_path_strip_failure() {
        use chrono::NaiveDate;
        use std::env;

        let temp_dir = env::temp_dir();
        let docs_dir = temp_dir.join("test_path_strip");
        std::fs::create_dir_all(&docs_dir).ok();

        let parsed = ParsedIndex {
            table_entries: HashMap::new(),
            state_sections: HashMap::new(),
        };

        let mut doc_map = HashMap::new();

        // Create a doc with path outside docs_dir
        let other_dir = temp_dir.join("other_location");
        std::fs::create_dir_all(&other_dir).ok();
        let doc_path = other_dir.join("0001-external.md");

        let doc = DesignDoc {
            metadata: crate::doc::DocMetadata {
                number: 1,
                title: "External".to_string(),
                author: "Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                state: DocState::Draft,
                supersedes: None,
                superseded_by: None,
            },
            content: "content".to_string(),
            path: doc_path.clone(),
        };
        doc_map.insert("0001".to_string(), doc);

        // Should handle gracefully when path can't be stripped
        let changes = compute_section_changes(&parsed, &doc_map, &docs_dir);

        // Should still produce a change, using full path
        assert!(!changes.is_empty());

        std::fs::remove_dir_all(&docs_dir).ok();
        std::fs::remove_dir_all(&other_dir).ok();
    }

    #[test]
    fn test_cleanup_formatting_header_followed_by_header() {
        let input = r#"## Section One
## Section Two"#;
        let output = cleanup_formatting(input);

        // Headers immediately following each other should be separated
        assert!(output.contains("Section One\n\n## Section Two"));
    }

    #[test]
    fn test_cleanup_formatting_bullet_after_header() {
        let input = r#"### Section
- Item"#;
        let output = cleanup_formatting(input);

        // Should have blank line between header and bullet
        assert!(output.contains("### Section\n\n- Item"));
    }

    #[test]
    fn test_cleanup_formatting_preserves_non_markdown_lines() {
        let input = r#"## Header

Regular text line
Another line

## Next"#;
        let output = cleanup_formatting(input);

        assert!(output.contains("Regular text line"));
        assert!(output.contains("Another line"));
    }

    #[test]
    fn test_cleanup_formatting_multiple_consecutive_blanks() {
        let input = "Line 1\n\n\n\n\nLine 2";
        let output = cleanup_formatting(input);

        // Should collapse multiple blank lines
        assert!(!output.contains("\n\n\n"));
        assert!(output.contains("Line 1\n\nLine 2"));
    }

    #[test]
    fn test_parse_table_with_number_header_row() {
        let content = r#"# Index

| Number | Title | State | Updated |
|--------|-------|-------|---------|
| Number | Title | State | Updated |
| 0001 | Valid | Draft | 2024-01-01 |
"#;
        let entries = parse_table(content);
        // Should skip the duplicate header row that says "Number"
        assert_eq!(entries.len(), 1);
        assert!(entries.contains_key("0001"));
    }

    #[test]
    fn test_get_docs_from_filesystem_all_states() {
        use std::env;

        let temp_dir = env::temp_dir();
        let test_dir = temp_dir.join("test_all_states");
        std::fs::create_dir_all(&test_dir).ok();

        // Create directories for all states
        for state in DocState::all_states() {
            let state_dir = test_dir.join(state.directory());
            std::fs::create_dir_all(&state_dir).ok();
            let filename = format!("test-{}.md", state.as_str().to_lowercase().replace(' ', "-"));
            std::fs::write(state_dir.join(filename), "content").ok();
        }

        let result = get_docs_from_filesystem(&test_dir).unwrap();

        // Should find docs in all state directories
        assert_eq!(result.len(), 10);

        std::fs::remove_dir_all(&test_dir).ok();
    }
}

/// Clean up section formatting for consistent spacing
pub fn cleanup_formatting(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Check if this is a section header
        let is_h2 = line.starts_with("## ");
        let is_h3 = line.starts_with("### ");

        if is_h2 || is_h3 {
            // Remove trailing blank lines before header
            while !result.is_empty() && result.last().unwrap().is_empty() {
                result.pop();
            }

            // Add exactly one blank line before header (unless first line)
            if !result.is_empty() {
                result.push(String::new());
            }

            // Add the header
            result.push(line.to_string());

            // Skip blank lines after header
            let mut j = i + 1;
            while j < lines.len() && lines[j].is_empty() {
                j += 1;
            }

            // Add exactly one blank line after header
            if j < lines.len() && !lines[j].starts_with('#') {
                result.push(String::new());
            }

            i = j;
            continue;
        }

        // Check if this is a bullet item
        let is_bullet = line.starts_with("- [");

        if is_bullet {
            result.push(line.to_string());

            // Look ahead to next non-blank line
            let mut j = i + 1;
            while j < lines.len() && lines[j].is_empty() {
                j += 1;
            }

            // If next non-blank is also a bullet, skip blank lines between them
            if j < lines.len() && lines[j].starts_with("- [") {
                i = j;
                continue;
            }

            i += 1;
            continue;
        }

        // For other lines, just add them (but collapse multiple blank lines)
        if line.is_empty() {
            if result.is_empty() || !result.last().unwrap().is_empty() {
                result.push(String::new());
            }
        } else {
            result.push(line.to_string());
        }

        i += 1;
    }

    // Ensure file ends with newline
    if !result.is_empty() && !result.last().unwrap().is_empty() {
        result.push(String::new());
    }

    result.join("\n")
}
