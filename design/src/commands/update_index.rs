//! Update index command implementation

use anyhow::{Context, Result};
use colored::*;
use design::doc::DesignDoc;
use design::index::DocumentIndex;
use design::index_sync::*;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Synchronize the index with documents on filesystem
pub fn update_index(index: &DocumentIndex) -> Result<()> {
    println!("{}\n", "Synchronizing index with documents...".bold());

    let docs_dir = PathBuf::from(index.docs_dir());
    let index_path = docs_dir.join("00-index.md");

    // Check if index exists
    if !index_path.exists() {
        println!(
            "{} Index file not found. Use 'oxd index' to generate it first.",
            "Warning:".yellow().bold()
        );
        return Ok(());
    }

    // Read current index
    let current_content = fs::read_to_string(&index_path).context("Failed to read index file")?;

    // Parse current index
    let parsed = ParsedIndex::parse(&current_content).context("Failed to parse index")?;

    // Get all docs from filesystem
    let doc_paths = get_docs_from_filesystem(&docs_dir).context("Failed to scan documents")?;

    // Build document map
    let doc_map = build_doc_map(&doc_paths);

    // Compute changes
    let table_changes = compute_table_changes(&parsed, &doc_map);
    let section_changes = compute_section_changes(&parsed, &doc_map, &docs_dir);

    let mut all_changes = Vec::new();
    all_changes.extend(table_changes);
    all_changes.extend(section_changes);

    // Apply changes to content
    let mut updated_content = current_content.clone();

    for change in &all_changes {
        updated_content = apply_change(&updated_content, change, &doc_map, &docs_dir)?;
    }

    // Store pre-format for comparison
    let pre_format_content = updated_content.clone();

    // Apply formatting cleanup
    updated_content = cleanup_formatting(&updated_content);

    // Check if formatting made changes
    let formatting_changed = pre_format_content != updated_content;

    // Report changes
    if all_changes.is_empty() && !formatting_changed {
        println!("{}\n", "✓ Index is already up to date!".green());
        return Ok(());
    }

    // Report content changes
    if !all_changes.is_empty() {
        println!("{}", "Changes:".bold());
        for change in &all_changes {
            println!("  {} {}", "✓".green(), change.description());
        }
        println!();
    }

    // Report formatting changes
    if formatting_changed && all_changes.is_empty() {
        println!("{}", "Formatting:".bold());
        println!("  {} Applied spacing and formatting cleanup", "✓".green());
        println!();
    }

    // Write updated index
    fs::write(&index_path, &updated_content).context("Failed to write index")?;

    // Summary
    let change_count = all_changes.len();
    if change_count > 0 {
        println!("{} {} change(s) applied to index", "Summary:".bold(), change_count);
    } else {
        println!("{} Formatting cleanup applied", "Summary:".bold());
    }

    Ok(())
}

/// Apply a single change to the index content
fn apply_change(
    content: &str,
    change: &IndexChange,
    _doc_map: &HashMap<String, DesignDoc>,
    _docs_dir: &Path,
) -> Result<String> {
    match change {
        IndexChange::TableAdd { number, title, state, updated } => {
            add_to_table(content, number, title, state, updated)
        }
        IndexChange::TableUpdate { number, field, new, .. } => {
            update_table_field(content, number, field, new)
        }
        IndexChange::TableRemove { number } => remove_from_table(content, number),
        IndexChange::SectionAdd { state, number, title, path } => {
            add_to_section(content, state, number, title, path)
        }
        IndexChange::SectionRemove { state, path } => remove_from_section(content, state, path),
    }
}

/// Add a new row to the table in sorted order
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

    for (idx, line) in lines.iter().enumerate() {
        // Detect table start
        if line.starts_with("| Number | Title") {
            in_table = true;
        }

        // Detect separator
        if in_table && line.contains("---|") {
            passed_separator = true;
            result.push(line.to_string());
            continue;
        }

        // Try to insert in sorted position
        if in_table && passed_separator && line.starts_with("| ") && !inserted {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 2 {
                let row_num_str = parts[1].trim();
                if let Ok(row_num) = row_num_str.parse::<u32>() {
                    if doc_num < row_num {
                        // Insert before this row
                        result
                            .push(format!("| {} | {} | {} | {} |", number, title, state, updated));
                        inserted = true;
                    }
                }
            }
        }

        result.push(line.to_string());

        // If we're leaving the table and didn't insert, add at end
        if in_table && passed_separator && !inserted {
            let next_line = lines.get(idx + 1).unwrap_or(&"");
            if !next_line.starts_with('|') {
                result.pop();
                result.push(format!("| {} | {} | {} | {} |", number, title, state, updated));
                result.push(line.to_string());
                inserted = true;
                in_table = false;
            }
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
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 5 {
                let mut new_line = String::from("|");
                new_line.push_str(&format!(" {} |", parts[1].trim())); // number
                match field {
                    "title" => new_line.push_str(&format!(" {} |", new_value)),
                    _ => new_line.push_str(&format!(" {} |", parts[2].trim())),
                }
                match field {
                    "state" => new_line.push_str(&format!(" {} |", new_value)),
                    _ => new_line.push_str(&format!(" {} |", parts[3].trim())),
                }
                match field {
                    "updated" => new_line.push_str(&format!(" {} |", new_value)),
                    _ => new_line.push_str(&format!(" {} |", parts[4].trim())),
                }
                result.push(new_line);
            } else {
                result.push(line.to_string());
            }
        } else {
            result.push(line.to_string());
        }
    }

    Ok(result.join("\n"))
}

/// Remove a row from the table
fn remove_from_table(content: &str, number: &str) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();
    let result: Vec<String> = lines
        .into_iter()
        .filter(|line| !line.starts_with(&format!("| {} |", number)))
        .map(|s| s.to_string())
        .collect();

    Ok(result.join("\n"))
}

/// Add document to a state section in sorted order
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
    let re = Regex::new(r"^\- \[(\d+)").unwrap();

    for (idx, line) in lines.iter().enumerate() {
        // Check if we're at the state section
        if *line == state_header {
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

        // If at end of section content (blank line or end), insert
        if in_section && !inserted {
            let next_line = lines.get(idx + 1);
            let at_section_end = next_line.is_none()
                || next_line.unwrap().is_empty()
                || next_line.unwrap().starts_with('#');

            if at_section_end && line.starts_with("- [") {
                // We're at the last bullet, insert after
            } else if at_section_end && !line.starts_with("- [") && !line.is_empty() {
                result.push(format!("- [{} - {}]({})", number, title, path));
                inserted = true;
            }
        }
    }

    // If section doesn't exist, we need to create it
    if !section_exists {
        let mut final_result = Vec::new();
        let mut added_section = false;

        for line in &result {
            final_result.push(line.clone());

            // Add section after "## Documents by State" header
            if line == "## Documents by State" && !added_section {
                final_result.push(String::new());
                final_result.push(state_header.clone());
                final_result.push(String::new());
                final_result.push(format!("- [{} - {}]({})", number, title, path));
                added_section = true;
            }
        }

        return Ok(final_result.join("\n"));
    }

    // If in section at end and didn't insert
    if in_section && !inserted {
        result.push(format!("- [{} - {}]({})", number, title, path));
    }

    Ok(result.join("\n"))
}

/// Remove document from a state section
fn remove_from_section(content: &str, state: &str, path: &str) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut result: Vec<String> = Vec::new();

    let state_header = format!("### {}", state);
    let mut in_section = false;
    let mut section_has_other_items = false;

    // First pass: check if section will be empty
    let mut temp_in_section = false;
    for line in &lines {
        if *line == state_header {
            temp_in_section = true;
            continue;
        }
        if temp_in_section && (line.starts_with("### ") || line.starts_with("## ")) {
            break;
        }
        if temp_in_section && line.starts_with("- [") && !line.contains(&format!("]({})", path)) {
            section_has_other_items = true;
            break;
        }
    }

    // Second pass: build result
    let mut skip_header = false;
    for line in &lines {
        // Check if entering section
        if *line == state_header {
            in_section = true;
            if !section_has_other_items {
                skip_header = true;
                // Also skip preceding blank line
                if !result.is_empty() && result.last().unwrap().is_empty() {
                    result.pop();
                }
                continue;
            }
        }

        // Check if leaving section
        if in_section && (line.starts_with("### ") || line.starts_with("## ")) {
            in_section = false;
            skip_header = false;
        }

        // Skip the matching line
        if in_section && line.contains(&format!("]({})", path)) {
            continue;
        }

        // Skip if we're removing entire section
        if skip_header && (line.is_empty() || line.starts_with("- [")) {
            continue;
        }

        if skip_header && !line.is_empty() && !line.starts_with("- [") {
            skip_header = false;
        }

        result.push(line.to_string());
    }

    Ok(result.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_update_index_no_index_file() {
        let temp = TempDir::new().unwrap();
        let index = DocumentIndex::new(temp.path()).unwrap();

        // Should handle missing index gracefully
        let result = update_index(&index);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_index_with_empty_index() {
        let temp = TempDir::new().unwrap();
        let index = DocumentIndex::new(temp.path()).unwrap();

        // Create an empty index file
        let index_path = temp.path().join("00-index.md");
        fs::write(
            &index_path,
            "# Design Document Index\n\n## All Documents by Number\n\n| Number | Title | State | Updated |\n|--------|-------|-------|----------|\n",
        )
        .unwrap();

        let result = update_index(&index);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_index_with_valid_index() {
        let temp = TempDir::new().unwrap();
        let index = DocumentIndex::new(temp.path()).unwrap();

        // Create a valid index file with a document entry
        let index_path = temp.path().join("00-index.md");
        fs::write(
            &index_path,
            r#"# Design Document Index

## All Documents by Number

| Number | Title | State | Updated |
|--------|-------|-------|----------|
| 0001 | Test Doc | Draft | 2024-01-01 |

## Documents by State

### Draft

- [0001 - Test Doc](01-draft/0001-test.md)
"#,
        )
        .unwrap();

        let result = update_index(&index);
        assert!(result.is_ok());
    }
}
