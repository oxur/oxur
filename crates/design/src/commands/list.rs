//! List command implementation

use anyhow::Result;
use chrono::{DateTime, Local};
use colored::*;
use design::doc::DocState;
use design::index::DocumentIndex;
use design::state::StateManager;
use design::theme;
use oxur_table::TableStyleConfig;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tabled::{builder::Builder, Table, Tabled};
use walkdir::WalkDir;

/// Minimal struct for type parameter when building tables with Builder
/// (Not used for actual data - Builder uses plain strings)
#[derive(Tabled)]
struct DocumentRow {
    number: String,
    title: String,
    state: String,
}

/// Minimal struct for type parameter when building removed document tables
/// (Not used for actual data - Builder uses plain strings)
#[derive(Tabled)]
struct RemovedDocRow {
    number: String,
    title: String,
    removed: String,
    deleted: String,
    location: String,
}

/// Minimal struct for type parameter when building dev document tables
/// (Not used for actual data - Builder uses plain strings)
#[derive(Tabled)]
struct DevDocRow {
    filename: String,
    updated: String,
}

/// Apply cell-specific colors to the state column while preserving theme backgrounds
fn apply_state_cell_colors(
    table: &mut Table,
    docs: &[&design::doc::DesignDoc],
    config: &TableStyleConfig,
) {
    let row_bg_colors = oxur_table::helpers::parse_row_bg_colors(config);

    for (i, doc) in docs.iter().enumerate() {
        let row_idx = 2 + i; // Data rows start at index 2 (after title and header)

        if let Some(fg_color) = oxur_table::helpers::state_to_fg_color(doc.metadata.state.as_str())
        {
            let bg_color = oxur_table::helpers::get_data_row_bg_color(i, &row_bg_colors);
            oxur_table::helpers::apply_cell_color(table, row_idx, 2, fg_color, bg_color);
        }
    }
}

#[allow(dead_code)]
pub fn list_documents(
    index: &DocumentIndex,
    state_filter: Option<String>,
    verbose: bool,
) -> Result<()> {
    list_documents_impl(index, None, state_filter, verbose, false, false)
}

pub fn list_documents_with_state(
    index: &DocumentIndex,
    state_mgr: Option<&StateManager>,
    state_filter: Option<String>,
    verbose: bool,
    removed: bool,
    dev: bool,
) -> Result<()> {
    list_documents_impl(index, state_mgr, state_filter, verbose, removed, dev)
}

fn list_documents_impl(
    index: &DocumentIndex,
    state_mgr: Option<&StateManager>,
    state_filter: Option<String>,
    verbose: bool,
    removed: bool,
    dev: bool,
) -> Result<()> {
    // If showing dev documents, use special handling
    if dev {
        return list_dev_documents(verbose);
    }

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

    if verbose {
        // Verbose mode: keep the detailed multi-line format with separate title
        println!("\n{}", "Design Documents".bold().underline());
        println!();

        for doc in &docs {
            let state = doc.metadata.state.as_str();
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
        }

        println!("Total: {} documents\n", docs.len());
    } else {
        // Normal mode: use table format with Builder (like the sample code)
        let mut builder = Builder::default();

        // Row 0: Title - PLAIN TEXT (formatting applied later)
        builder.push_record(["DESIGN", "DOCUMENTS", ""]);

        // Row 1: Header - PLAIN TEXT
        builder.push_record(["Number", "Title", "State"]);

        // Data rows - PLAIN TEXT (no ANSI codes, formatting applied later)
        for doc in &docs {
            builder.push_record([
                &format!(" {:04}", doc.metadata.number),
                &format!(" {:} ", doc.metadata.title.as_str()),
                &format!(" {:<14}", doc.metadata.state.as_str()),
            ]);
        }

        // Last row: Footer with total count - PLAIN TEXT
        let total_text = format!("{} documents", docs.len());
        builder.push_record(["Total:", &total_text, ""]);

        // Build the table structure (width calculation happens here with plain text)
        let mut table = builder.build();

        // Apply the theme (background colors, padding, justification)
        let config = TableStyleConfig::default();
        config.apply_to_table::<DocumentRow>(&mut table);

        // Apply cell-specific colors (state column)
        apply_state_cell_colors(&mut table, &docs, &config);

        println!();
        println!("{}", table);
        println!();
    }
    Ok(())
}

/// List documents that have been removed to the dustbin
fn list_removed_documents(state_mgr: &StateManager, verbose: bool) -> Result<()> {
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
        println!();
        println!("{}", "Removed Documents".cyan().bold());
        println!();
        println!("  {}", "No removed documents found.".yellow());
        println!();
        return Ok(());
    }

    // Build table with Builder (like the sample code)
    let mut builder = Builder::default();

    // Row 0: Title - PLAIN TEXT (formatting applied later)
    builder.push_record(["REMOVED DOCUMENTS", "", "", "", ""]);

    // Row 1: Header - PLAIN TEXT
    if verbose {
        builder.push_record(["Number", "Title", "Removed", "Deleted", "Dustbin Location"]);
    } else {
        builder.push_record(["Number", "Title", "Removed", "Deleted", ""]);
    }

    // Count files in dustbin vs deleted (do this first so we can use in footer)
    let mut in_dustbin = 0;
    let mut deleted = 0;
    for doc in &removed_docs {
        let file_path = state_mgr.docs_dir().join(&doc.path);
        if file_path.exists() {
            in_dustbin += 1;
        } else {
            deleted += 1;
        }
    }

    // Data rows - PLAIN TEXT (no ANSI codes, formatting applied later)
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

        let location = if verbose {
            if file_exists {
                doc.path.clone()
            } else {
                "(file not found)".to_string()
            }
        } else {
            String::new()
        };

        // All plain text - no colors yet
        let deleted_str = if file_exists { "false".to_string() } else { "true".to_string() };
        builder.push_record([
            &number_str, // Plain text, no .yellow()
            &title_truncated,
            &doc.metadata.updated.to_string(), // Plain text, no .white()
            &deleted_str,                      // Plain text, no .green()/.red()
            &location,
        ]);
    }

    // Last row: Footer with total count - PLAIN TEXT
    let total_text = format!(
        "Total: {} removed ({} in dustbin, {} deleted)",
        removed_docs.len(),
        in_dustbin,
        deleted
    );
    builder.push_record([&total_text, "", "", "", ""]);

    // Build the table structure (width calculation happens here with plain text)
    let mut table = builder.build();

    // Apply the theme (background colors, padding, justification)
    let config = TableStyleConfig::default();
    config.apply_to_table::<RemovedDocRow>(&mut table);

    println!();
    println!("{}", table);
    println!();

    Ok(())
}

// ============================================================================
// Dev Documents Listing
// ============================================================================

/// Check if filename has valid NNNN prefix (0000-9999)
fn has_valid_prefix(filename: &str) -> bool {
    if filename.len() < 5 {
        return false;
    }

    // Check first 4 chars are digits, followed by '-'
    let chars: Vec<char> = filename.chars().collect();
    chars[0].is_numeric()
        && chars[1].is_numeric()
        && chars[2].is_numeric()
        && chars[3].is_numeric()
        && chars.get(4) == Some(&'-')
}

/// Extract numerical prefix from filename (e.g., "0019" from "0019-foo.md")
fn extract_prefix_number(path: &Path) -> u32 {
    path.file_name()
        .and_then(|s| s.to_str())
        .and_then(|s| s.get(0..4))
        .and_then(|prefix| prefix.parse::<u32>().ok())
        .unwrap_or(0)
}

/// Format file modification time for display
fn format_mtime(mtime: &SystemTime) -> String {
    let datetime: DateTime<Local> = (*mtime).into();
    datetime.format("%Y-%m-%d %H:%M").to_string()
}

/// Get relative path from invocation directory
fn get_relative_path_from(path: &Path, from: &Path) -> String {
    match path.strip_prefix(from) {
        Ok(rel) => rel.to_string_lossy().to_string(),
        Err(_) => path.to_string_lossy().to_string(),
    }
}

/// Filter and validate a potential dev document file
/// Returns Some((path, metadata)) if valid, None otherwise
fn filter_dev_file(path: &Path) -> Result<Option<(PathBuf, fs::Metadata)>> {
    // Must be .md file
    if path.extension().and_then(|s| s.to_str()) != Some("md") {
        return Ok(None);
    }

    // Must have NNNN prefix
    let filename = match path.file_name().and_then(|s| s.to_str()) {
        Some(f) => f,
        None => return Ok(None),
    };

    if !has_valid_prefix(filename) {
        return Ok(None);
    }

    // Get metadata for mtime
    let metadata = fs::metadata(path)?;

    Ok(Some((path.to_path_buf(), metadata)))
}

/// Collect .md files with NNNN prefixes from a directory
///
/// # Arguments
/// * `dir` - Directory to scan
/// * `recursive` - If true, descend into subdirectories
///
/// Returns sorted list of (PathBuf, file metadata) tuples
fn collect_dev_docs(dir: &Path, recursive: bool) -> Result<Vec<(PathBuf, fs::Metadata)>> {
    let mut docs = Vec::new();

    if recursive {
        // Recursive walk
        for entry in WalkDir::new(dir).min_depth(1).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(doc) = filter_dev_file(entry.path())? {
                    docs.push(doc);
                }
            }
        }
    } else {
        // Top-level only
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(doc) = filter_dev_file(&path)? {
                    docs.push(doc);
                }
            }
        }
    }

    // Sort by NNNN prefix (descending: highest first)
    docs.sort_by(|a, b| {
        let a_num = extract_prefix_number(&a.0);
        let b_num = extract_prefix_number(&b.0);
        b_num.cmp(&a_num) // Reversed for descending
    });

    Ok(docs)
}

/// Get list of subdirectories (excluding hidden dirs like .git)
fn collect_subdirectories(dir: &Path) -> Result<Vec<String>> {
    let mut subdirs = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                // Skip hidden directories
                if !name.starts_with('.') {
                    subdirs.push(name.to_string());
                }
            }
        }
    }

    // Sort alphabetically
    subdirs.sort();

    Ok(subdirs)
}

/// Print a table for dev documents in a specific directory
fn print_dev_table(
    table_name: &str,
    docs: &[(PathBuf, fs::Metadata)],
    invocation_dir: &Path,
    _verbose: bool,
) -> Result<()> {
    let mut builder = Builder::default();

    // Row 0: Title (uppercase directory name)
    let title = table_name.to_uppercase();
    builder.push_record([&title, ""]);

    // Row 1: Header
    builder.push_record(["Filename", "Updated"]);

    // Data rows (sorted, descending by prefix)
    for (path, metadata) in docs {
        let rel_path = get_relative_path_from(path, invocation_dir);

        // Format mtime
        let mtime_str = format_mtime(&metadata.modified()?);

        builder.push_record([&format!(" {} ", rel_path), &format!(" {} ", mtime_str)]);
    }

    // Footer: Total count
    let total_text = if docs.len() == 1 {
        "Total: 1 document".to_string()
    } else {
        format!("Total: {} documents", docs.len())
    };
    builder.push_record([&total_text, ""]);

    // Build and style
    let mut table = builder.build();
    let config = TableStyleConfig::default();
    config.apply_to_table::<DevDocRow>(&mut table);

    println!();
    println!("{}", table);
    println!();

    Ok(())
}

/// List untracked development documents in crates/design/dev
fn list_dev_documents(_verbose: bool) -> Result<()> {
    // Determine base path (from CLI invocation point)
    let invocation_dir = env::current_dir()?;

    // Find dev directory (assume we're in repo)
    let dev_dir = if let Some(repo_root) = design::git::get_repo_root() {
        repo_root.join("crates/design/dev")
    } else {
        // Fallback: try relative path
        invocation_dir.join("crates/design/dev")
    };

    if !dev_dir.exists() {
        eprintln!("{} Dev directory not found: {}", "ERROR:".red().bold(), dev_dir.display());
        return Ok(());
    }

    // Collect documents by directory
    let dev_root_docs = collect_dev_docs(&dev_dir, false)?;
    let subdirs = collect_subdirectories(&dev_dir)?;

    // Print dev/ root table (non-recursive)
    if !dev_root_docs.is_empty() {
        print_dev_table("dev", &dev_root_docs, &invocation_dir, _verbose)?;
    }

    // Print table for each subdirectory (recursive)
    for subdir_name in subdirs {
        let subdir_path = dev_dir.join(&subdir_name);
        let docs = collect_dev_docs(&subdir_path, true)?;

        if !docs.is_empty() {
            print_dev_table(&subdir_name, &docs, &invocation_dir, _verbose)?;
        }
    }

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
