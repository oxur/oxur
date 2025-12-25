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
