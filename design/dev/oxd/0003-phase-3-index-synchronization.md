# Phase 3: Index Synchronization - Detailed Implementation Guide

## Overview
This phase implements the sophisticated index synchronization system that automatically keeps the 00-index.md file in sync with the actual documents on the filesystem. This is the most complex phase, involving parsing, comparison, and intelligent updates.

**Prerequisites:** Phases 1 and 2 must be complete

---

## Task 3.1: Index Scanning and Parsing

### Purpose
Parse the existing 00-index.md file to extract:
1. Table entries (all documents by number)
2. State section entries (documents grouped by state)

### Implementation Steps

#### Step 1: Create Index Sync Module
File: `design/src/index_sync.rs`

```rust
//! Index synchronization module

use crate::doc::{DesignDoc, DocState};
use anyhow::{Context, Result};
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
        
        Ok(ParsedIndex {
            table_entries,
            state_sections,
        })
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
        if in_table && !line.starts_with("|") {
            break;
        }
        
        // Parse data rows
        if in_table && passed_separator && line.starts_with("|") {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 5 {
                let number = parts[1].trim();
                let title = parts[2].trim();
                let state = parts[3].trim();
                let updated = parts[4].trim();
                
                if !number.is_empty() && number != "Number" {
                    entries.insert(number.to_string(), IndexEntry {
                        number: number.to_string(),
                        title: title.to_string(),
                        state: state.to_string(),
                        updated: updated.to_string(),
                    });
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
        if line.starts_with("### ") {
            current_state = Some(line[4..].trim().to_string());
            sections.insert(current_state.clone().unwrap(), Vec::new());
            continue;
        }
        
        // Detect end of state sections (## header or lower)
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
```

#### Step 2: Add Helper Functions
Continue in `design/src/index_sync.rs`:

```rust
/// Get all git-tracked markdown files in state directories
pub fn get_git_tracked_docs(docs_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    use std::process::Command;
    
    let docs_dir = docs_dir.as_ref();
    let mut all_docs = Vec::new();
    
    // Get all state directories
    for state in DocState::all_states() {
        let state_dir = docs_dir.join(state.directory());
        let pattern = format!("{}/*.md", state_dir.display());
        
        let output = Command::new("git")
            .args(["ls-files", &pattern])
            .output()
            .context("Failed to execute git ls-files")?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if !line.is_empty() {
                    all_docs.push(PathBuf::from(line));
                }
            }
        }
    }
    
    Ok(all_docs)
}

/// Extract document metadata for all git-tracked docs
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
```

### Testing Steps for 3.1
1. Create test index file with table and state sections
2. Parse and verify table entries extracted correctly
3. Verify state sections parsed correctly
4. Test with missing sections
5. Test with malformed entries (graceful degradation)

---

## Task 3.2: Index Update Logic

### Purpose
Compare the parsed index against the filesystem and determine what changes need to be made.

### Implementation Steps

#### Step 1: Add Change Tracking
Continue in `design/src/index_sync.rs`:

```rust
/// Types of changes that can occur
#[derive(Debug, Clone)]
pub enum IndexChange {
    TableAdd { number: String, title: String, state: String, updated: String },
    TableUpdate { number: String, field: String, old: String, new: String },
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
            IndexChange::SectionAdd { state, number, title, .. } => {
                format!("Add to {}: {} - {}", state, number, title)
            }
            IndexChange::SectionRemove { state, path } => {
                format!("Remove from {}: {}", state, path)
            }
        }
    }
}

/// Compare index with filesystem and determine changes
pub fn compute_changes(
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
        let current_paths = parsed.state_sections
            .get(&state_name)
            .cloned()
            .unwrap_or_default();
        
        // Build set of current paths for quick lookup
        let current_set: HashSet<String> = current_paths.iter().cloned().collect();
        
        // Check for documents that should be in section but aren't
        for doc in &expected_docs {
            let rel_path = doc.path.strip_prefix(docs_dir)
                .unwrap_or(&doc.path);
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
        let expected_set: HashSet<String> = expected_docs.iter()
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
```

### Testing Steps for 3.2
1. Test with document missing from table
2. Test with outdated state in table
3. Test with outdated updated date
4. Test with document missing from state section
5. Test with stale document in state section
6. Verify change descriptions are clear

---

## Task 3.3: Formatting Cleanup

### Purpose
Ensure consistent spacing around headers and within bullet lists.

### Implementation Steps

#### Step 1: Add Formatting Functions
Continue in `design/src/index_sync.rs`:

```rust
/// Clean up section formatting for consistent spacing
pub fn cleanup_section_formatting(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let cleaned = cleanup_lines(&lines);
    cleaned.join("\n")
}

fn cleanup_lines(lines: &[&str]) -> Vec<String> {
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i];
        
        // Check if this is a section header (### or ##)
        let is_header = line.starts_with("### ") || line.starts_with("## ");
        
        if is_header {
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
            
            // Add exactly one blank line after header (unless next is another header)
            if j < lines.len() && !lines[j].starts_with("##") {
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
        
        // For other lines, just add them
        result.push(line.to_string());
        i += 1;
    }
    
    result
}
```

### Testing Steps for 3.3
1. Test with extra blank lines between bullets
2. Test with missing blank lines before headers
3. Test with multiple blank lines before headers
4. Test with blank lines after headers
5. Verify idempotency (running twice produces same result)

---

## Task 3.4: Index Update Command

### Purpose
Orchestrate the full synchronization: scan, compare, update, format.

### Implementation Steps

#### Step 1: Create Update Index Command
File: `design/src/commands/update_index.rs`

```rust
//! Update index command implementation

use anyhow::{Context, Result};
use colored::*;
use design::index::DocumentIndex;
use design::index_sync::*;
use std::fs;
use std::path::PathBuf;

/// Synchronize the index with git-tracked documents
pub fn update_index(index: &DocumentIndex) -> Result<()> {
    println!("{}\n", "Synchronizing index with git-tracked documents...".bold());
    
    let index_path = PathBuf::from(index.docs_dir()).join("00-index.md");
    
    // Read current index
    let current_content = fs::read_to_string(&index_path)
        .context("Failed to read index file")?;
    
    // Parse current index
    let parsed = ParsedIndex::parse(&current_content)
        .context("Failed to parse index")?;
    
    // Get all git-tracked docs
    let git_docs = get_git_tracked_docs(index.docs_dir())
        .context("Failed to get git-tracked documents")?;
    
    // Build document map
    let doc_map = build_doc_map(&git_docs);
    
    // Compute changes
    let table_changes = compute_changes(&parsed, &doc_map);
    let section_changes = compute_section_changes(&parsed, &doc_map, index.docs_dir());
    
    let mut all_changes = Vec::new();
    all_changes.extend(table_changes);
    all_changes.extend(section_changes);
    
    // Apply changes to content
    let mut updated_content = current_content.clone();
    
    if !all_changes.is_empty() {
        // Apply table changes
        for change in &all_changes {
            updated_content = apply_change(&updated_content, change, &doc_map, index.docs_dir())?;
        }
    }
    
    // Store original for comparison
    let pre_format_content = updated_content.clone();
    
    // Always apply formatting cleanup
    updated_content = cleanup_section_formatting(&updated_content);
    
    // Check if formatting made changes
    let formatting_changed = pre_format_content != updated_content;
    
    // Report changes
    if all_changes.is_empty() && !formatting_changed {
        println!("{}\n", "✓ Index is already up to date!".green());
        return Ok(());
    }
    
    // Report content changes
    if !all_changes.is_empty() {
        println!("{}", "Content Changes:".bold());
        for change in &all_changes {
            println!("  {} {}", "✓".green(), change.description());
        }
        println!();
    }
    
    // Report formatting changes
    if formatting_changed {
        if all_changes.is_empty() {
            println!("{}", "Formatting Cleanup:".bold());
        }
        println!("  {} Fixed section heading spacing and bullet list formatting", "✓".green());
        println!();
    }
    
    // Write updated index
    fs::write(&index_path, updated_content)
        .context("Failed to write index")?;
    
    // Summary
    if !all_changes.is_empty() {
        println!("{} {} content changes made to index", "Summary:".bold(), all_changes.len());
    } else if formatting_changed {
        println!("{} Formatting cleanup applied to index", "Summary:".bold());
    }
    
    Ok(())
}

/// Apply a single change to the index content
fn apply_change(
    content: &str,
    change: &IndexChange,
    doc_map: &HashMap<String, DesignDoc>,
    docs_dir: &std::path::Path,
) -> Result<String> {
    match change {
        IndexChange::TableAdd { number, title, state, updated } => {
            add_to_table(content, number, title, state, updated)
        }
        IndexChange::TableUpdate { number, field, new, .. } => {
            update_table_field(content, number, field, new)
        }
        IndexChange::SectionAdd { state, number, title, path } => {
            add_to_section(content, state, number, title, path)
        }
        IndexChange::SectionRemove { state, path } => {
            remove_from_section(content, state, path)
        }
    }
}
```

#### Step 2: Implement Table Manipulation
Continue in `design/src/commands/update_index.rs`:

```rust
use std::collections::HashMap;
use design::doc::DesignDoc;

/// Add a new row to the table
fn add_to_table(
    content: &str,
    number: &str,
    title: &str,
    state: &str,
    updated: &str,
) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    
    let doc_num: u32 = number.parse().unwrap_or(0);
    let mut inserted = false;
    let mut in_table = false;
    let mut passed_separator = false;
    
    for line in lines {
        // Detect table start
        if line.starts_with("| Number | Title") {
            in_table = true;
        }
        
        // Detect separator
        if in_table && line.contains("---|") {
            passed_separator = true;
        }
        
        // Try to insert in sorted position
        if in_table && passed_separator && line.starts_with("| ") && !inserted {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 2 {
                let row_num_str = parts[1].trim();
                if let Ok(row_num) = row_num_str.parse::<u32>() {
                    if doc_num < row_num {
                        // Insert before this row
                        result.push(format!("| {} | {} | {} | {} |", number, title, state, updated));
                        inserted = true;
                    }
                }
            }
        }
        
        result.push(line.to_string());
        
        // If we left the table and didn't insert, we need to handle it
        if in_table && !line.starts_with("|") && !inserted {
            // Insert before leaving table
            result.pop(); // Remove the line we just added
            result.push(format!("| {} | {} | {} | {} |", number, title, state, updated));
            result.push(line.to_string());
            inserted = true;
            in_table = false;
        }
    }
    
    Ok(result.join("\n"))
}

/// Update a field in the table
fn update_table_field(content: &str, number: &str, field: &str, new_value: &str) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    
    for line in lines {
        if line.starts_with(&format!("| {} |", number)) {
            // Parse and update this row
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 5 {
                let mut new_parts = parts.clone();
                match field {
                    "state" => new_parts[3] = &format!(" {} ", new_value),
                    "updated" => new_parts[4] = &format!(" {} ", new_value),
                    _ => {}
                }
                result.push(new_parts.join("|"));
            } else {
                result.push(line.to_string());
            }
        } else {
            result.push(line.to_string());
        }
    }
    
    Ok(result.join("\n"))
}
```

#### Step 3: Implement Section Manipulation
Continue in `design/src/commands/update_index.rs`:

```rust
/// Add document to a state section
fn add_to_section(
    content: &str,
    state: &str,
    number: &str,
    title: &str,
    path: &str,
) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    
    let state_header = format!("### {}", state);
    let doc_num: u32 = number.parse().unwrap_or(0);
    let mut in_section = false;
    let mut section_exists = false;
    let mut inserted = false;
    
    for line in lines {
        // Check if we're at the state section
        if line == state_header {
            section_exists = true;
            in_section = true;
            result.push(line.to_string());
            continue;
        }
        
        // Check if we're leaving the section
        if in_section && (line.starts_with("### ") || line.starts_with("## ")) {
            // Insert before leaving if not yet inserted
            if !inserted {
                result.push(format!("- [{} - {}]({})", number, title, path));
                inserted = true;
            }
            in_section = false;
        }
        
        // Try to insert in sorted position within section
        if in_section && line.starts_with("- [") && !inserted {
            // Extract number from existing line
            let re = regex::Regex::new(r"^\- \[(\d+)").unwrap();
            if let Some(caps) = re.captures(line) {
                if let Some(num_match) = caps.get(1) {
                    if let Ok(existing_num) = num_match.as_str().parse::<u32>() {
                        if doc_num < existing_num {
                            result.push(format!("- [{} - {}]({})", number, title, path));
                            inserted = true;
                        }
                    }
                }
            }
        }
        
        result.push(line.to_string());
    }
    
    // If section doesn't exist, create it after "## Documents by State"
    if !section_exists {
        let mut final_result = Vec::new();
        for line in result {
            final_result.push(line.clone());
            if line == "## Documents by State" {
                final_result.push(String::new());
                final_result.push(state_header.clone());
                final_result.push(format!("- [{} - {}]({})", number, title, path));
                inserted = true;
            }
        }
        return Ok(final_result.join("\n"));
    }
    
    // If in section and didn't insert, add at end
    if in_section && !inserted {
        result.push(format!("- [{} - {}]({})", number, title, path));
    }
    
    Ok(result.join("\n"))
}

/// Remove document from a state section
fn remove_from_section(content: &str, state: &str, path: &str) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    
    let state_header = format!("### {}", state);
    let mut in_section = false;
    let mut skip_next_blank = false;
    
    for (idx, line) in lines.iter().enumerate() {
        // Check if entering section
        if *line == state_header {
            in_section = true;
            result.push(line.to_string());
            continue;
        }
        
        // Check if leaving section
        if in_section && (line.starts_with("### ") || line.starts_with("## ")) {
            in_section = false;
        }
        
        // Skip the matching line
        if in_section && line.contains(&format!("]({})", path)) {
            // Check if section will be empty
            let section_start = result.len();
            let mut has_other_content = false;
            for i in (idx + 1)..lines.len() {
                if lines[i].starts_with("###") || lines[i].starts_with("##") {
                    break;
                }
                if !lines[i].is_empty() && lines[i].starts_with("- [") {
                    has_other_content = true;
                    break;
                }
            }
            
            // If section will be empty, remove the header too
            if !has_other_content {
                // Remove the section header we just added
                while !result.is_empty() && result.last().unwrap() != &state_header {
                    result.pop();
                }
                if !result.is_empty() && result.last().unwrap() == &state_header {
                    result.pop();
                }
                // Also remove blank line before header if present
                while !result.is_empty() && result.last().unwrap().is_empty() {
                    result.pop();
                }
            }
            continue;
        }
        
        result.push(line.to_string());
    }
    
    Ok(result.join("\n"))
}
```

#### Step 4: Update Commands Module
File: `design/src/commands/mod.rs`

```rust
pub mod update_index;
pub use update_index::update_index;
```

#### Step 5: Update CLI
File: `design/src/cli.rs`

```rust
/// Synchronize index with git-tracked documents
UpdateIndex,
```

#### Step 6: Wire Up in Main
File: `design/src/main.rs`

```rust
Commands::UpdateIndex => {
    update_index(&index)?;
}
```

#### Step 7: Export Module in lib.rs
File: `design/src/lib.rs`

```rust
pub mod index_sync;
```

### Testing Steps for 3.4
1. Add new document and run update-index
2. Modify document state and run update-index
3. Delete document and run update-index
4. Run update-index twice (should be idempotent)
5. Test with empty sections (should remove header)
6. Test with multiple changes at once
7. Verify formatting cleanup applied

---

## Edge Cases to Handle

### Empty Sections
When a state section has no documents:
- Remove the section header
- Remove associated blank lines
- Don't leave orphaned headers

### Missing Index File
If 00-index.md doesn't exist:
- Consider creating it fresh
- Or provide clear error message

### Malformed Index
If index can't be parsed:
- Provide helpful error
- Consider --force flag to regenerate

### Concurrent Changes
If index modified during scan:
- Current approach: last write wins
- Could add file locking in future

---

## Performance Considerations

### Large Repositories
For repositories with 100+ documents:
- Git operations may be slow
- Consider caching git metadata
- Could parallelize document parsing

### Optimization Opportunities
- Cache parsed documents in DocumentIndex
- Only re-parse modified files
- Batch git operations

---

## Verification Checklist

Before moving to Phase 4, verify:

- [ ] Index table parses correctly
- [ ] State sections parse correctly
- [ ] Missing documents detected
- [ ] Outdated fields detected
- [ ] Table updates work correctly
- [ ] Documents added to sections in sorted order
- [ ] Documents removed from sections correctly
- [ ] Empty sections removed entirely
- [ ] Formatting cleanup works correctly
- [ ] Idempotent (running twice produces same result)
- [ ] Clear, colorful output
- [ ] All changes reported accurately

---

## Integration Notes

The update-index command is the "sync everything" command:
- Run after bulk operations
- Run to fix inconsistencies
- Can be run anytime (idempotent)
- Should be fast enough for frequent use

It integrates with:
- Phase 1: Uses git operations and state system
- Phase 2: Complements transition and add-headers
- Phase 4: Called by the add command
- Phase 5: Enhanced validation can suggest running it

---

## Usage Examples

Once implemented:

```bash
# Sync index after making changes
oxd update-index

# Typical workflow
oxd add-headers some-doc.md
oxd transition some-doc.md "under review"
oxd update-index

# After bulk git operations
git mv 01-draft/*.md 02-under-review/
oxd update-index
```

---

## Notes for Claude Code

- Regex patterns must handle various markdown formatting
- Table parsing should be resilient to spacing variations
- Section removal should clean up completely
- Formatting cleanup should be conservative
- Test with both Unix and Windows line endings
- Git operations should handle relative paths correctly
- Error messages should suggest fixes
- Consider adding --dry-run flag to preview changes
- Add progress indicators for large repositories
