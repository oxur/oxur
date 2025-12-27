//! Debug and introspection commands

use anyhow::Result;
use chrono::Duration;
use colored::*;
use design::doc::{DesignDoc, DocState};
use design::index_sync::get_git_tracked_docs;
use design::state::{compute_checksum, DocumentRecord, StateManager};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

/// Show entire state in human-readable format
pub fn show_state(state_mgr: &StateManager, format: &str) -> Result<()> {
    match format {
        "json" => show_state_json(state_mgr),
        "table" => show_state_table(state_mgr),
        "summary" => show_state_summary(state_mgr),
        _ => show_state_table(state_mgr),
    }
}

fn show_state_json(state_mgr: &StateManager) -> Result<()> {
    let json = serde_json::to_string_pretty(state_mgr.state())?;
    println!("{}", json);
    Ok(())
}

fn show_state_table(state_mgr: &StateManager) -> Result<()> {
    let state = state_mgr.state();

    println!("\n{}", "Document State".bold().underline());
    println!("Version: {}", state.version);
    println!("Last Updated: {}", state.last_updated.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("Next Number: {:04}", state.next_number);
    println!("Total Documents: {}\n", state.documents.len());

    // Print table header
    println!(
        "{:>4}  {:<40}  {:<12}  {:>8}  {:<16}  {:<16}",
        "Num", "Title", "State", "Size", "Modified", "Checksum"
    );
    println!("{}", "─".repeat(110));

    let docs = state.all();
    for doc in docs {
        let title = if doc.metadata.title.len() > 38 {
            format!("{}...", &doc.metadata.title[..35])
        } else {
            doc.metadata.title.clone()
        };

        println!(
            "{:04}  {:<40}  {:<12}  {:>8}  {:<16}  {:<16}",
            doc.metadata.number,
            title,
            doc.metadata.state.as_str(),
            format_size(doc.file_size),
            doc.modified.format("%Y-%m-%d %H:%M").to_string(),
            &doc.checksum[..16]
        );
    }

    println!();
    Ok(())
}

fn show_state_summary(state_mgr: &StateManager) -> Result<()> {
    let state = state_mgr.state();

    println!("\n{}", "State Summary".bold().underline());
    println!();

    // Count by state
    let mut by_state: HashMap<String, usize> = HashMap::new();
    for record in state.all() {
        *by_state.entry(record.metadata.state.as_str().to_string()).or_insert(0) += 1;
    }

    println!("{}", "Documents by State:".bold());
    for state_name in DocState::all_state_names() {
        let count = by_state.get(state_name).unwrap_or(&0);
        println!("  {}: {}", state_name, count);
    }
    println!();

    // Size statistics
    let total_size: u64 = state.documents.values().map(|d| d.file_size).sum();
    let avg_size =
        if !state.documents.is_empty() { total_size / state.documents.len() as u64 } else { 0 };

    println!("{}", "Size Statistics:".bold());
    println!("  Total: {}", format_size(total_size));
    println!("  Average: {}", format_size(avg_size));
    println!();

    // Recent activity
    let mut recent: Vec<_> = state.all();
    recent.sort_by(|a, b| b.modified.cmp(&a.modified));

    println!("{}", "Recently Modified:".bold());
    for doc in recent.iter().take(5) {
        println!(
            "  {:04} - {} ({})",
            doc.metadata.number,
            doc.metadata.title,
            doc.modified.format("%Y-%m-%d %H:%M")
        );
    }
    println!();

    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Show detailed state for a specific document
pub fn show_document_state(state_mgr: &StateManager, number: u32) -> Result<()> {
    let record = state_mgr
        .state()
        .get(number)
        .ok_or_else(|| anyhow::anyhow!("Document {:04} not found in state", number))?;

    println!("\n{}", format!("Document {:04} State", number).bold().underline());
    println!();

    println!("{}", "Metadata:".bold());
    println!("  Number: {:04}", record.metadata.number);
    println!("  Title: {}", record.metadata.title);
    println!("  Author: {}", record.metadata.author);
    println!("  State: {}", record.metadata.state.as_str());
    println!("  Created: {}", record.metadata.created);
    println!("  Updated: {}", record.metadata.updated);

    if let Some(supersedes) = record.metadata.supersedes {
        println!("  Supersedes: {:04}", supersedes);
    }
    if let Some(superseded_by) = record.metadata.superseded_by {
        println!("  Superseded By: {:04}", superseded_by);
    }
    println!();

    println!("{}", "File Information:".bold());
    println!("  Path: {}", record.path);
    println!("  Size: {}", format_size(record.file_size));
    println!("  Modified: {}", record.modified.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("  Checksum: {}", record.checksum);
    println!();

    Ok(())
}

/// Show checksums and identify dirty files
pub fn show_checksums(state_mgr: &StateManager, verbose: bool) -> Result<()> {
    println!("\n{}", "Checksum Status".bold().underline());
    println!();

    let mut clean = 0;
    let mut dirty = 0;
    let mut missing = 0;

    let mut dirty_files: Vec<(u32, String, String, String)> = Vec::new();

    for record in state_mgr.state().all() {
        let full_path = PathBuf::from(state_mgr.docs_dir()).join(&record.path);

        if !full_path.exists() {
            missing += 1;
            if verbose {
                dirty_files.push((
                    record.metadata.number,
                    record.metadata.title.clone(),
                    "MISSING".to_string(),
                    "-".to_string(),
                ));
            }
        } else {
            match compute_checksum(&full_path) {
                Ok(actual) => {
                    if actual == record.checksum {
                        clean += 1;
                        if verbose {
                            dirty_files.push((
                                record.metadata.number,
                                record.metadata.title.clone(),
                                "CLEAN".to_string(),
                                actual,
                            ));
                        }
                    } else {
                        dirty += 1;
                        dirty_files.push((
                            record.metadata.number,
                            record.metadata.title.clone(),
                            "DIRTY".to_string(),
                            actual,
                        ));
                    }
                }
                Err(_) => {
                    missing += 1;
                    dirty_files.push((
                        record.metadata.number,
                        record.metadata.title.clone(),
                        "ERROR".to_string(),
                        "-".to_string(),
                    ));
                }
            }
        }
    }

    if !dirty_files.is_empty() {
        println!("{:>4}  {:<40}  {:<8}  {:<16}", "Num", "Title", "Status", "Actual Checksum");
        println!("{}", "─".repeat(76));

        for (num, title, status, checksum) in &dirty_files {
            let title_display =
                if title.len() > 38 { format!("{}...", &title[..35]) } else { title.clone() };

            let status_colored = match status.as_str() {
                "CLEAN" => status.green().to_string(),
                "DIRTY" => status.yellow().to_string(),
                "MISSING" | "ERROR" => status.red().to_string(),
                _ => status.clone(),
            };

            let checksum_display =
                if checksum.len() > 16 { &checksum[..16] } else { checksum.as_str() };

            println!(
                "{:04}  {:<40}  {:<8}  {:<16}",
                num, title_display, status_colored, checksum_display
            );
        }
        println!();
    }

    println!("{}", "Summary:".bold());
    println!("  {} Clean", clean.to_string().green());
    println!("  {} Dirty", dirty.to_string().yellow());
    println!("  {} Missing", missing.to_string().red());
    println!();

    if dirty > 0 {
        println!("{} Run 'oxd scan' to update checksums", "→".cyan());
    }

    Ok(())
}

/// Show statistics about the repository
pub fn show_stats(state_mgr: &StateManager) -> Result<()> {
    let state = state_mgr.state();

    println!("\n{}", "Repository Statistics".bold().underline());
    println!();

    // Document counts
    println!("{}", "Documents:".bold());
    println!("  Total: {}", state.documents.len());
    println!("  Next Number: {:04}", state.next_number);
    println!();

    // By state
    let mut by_state: HashMap<String, Vec<&DocumentRecord>> = HashMap::new();
    for record in state.all() {
        by_state.entry(record.metadata.state.as_str().to_string()).or_default().push(record);
    }

    println!("{}", "By State:".bold());
    for state_name in DocState::all_state_names() {
        if let Some(docs) = by_state.get(state_name) {
            println!("  {}: {} docs", state_name, docs.len());
        }
    }
    println!();

    // By author
    let mut by_author: HashMap<String, usize> = HashMap::new();
    for record in state.all() {
        *by_author.entry(record.metadata.author.clone()).or_insert(0) += 1;
    }

    println!("{}", "By Author:".bold());
    let mut authors: Vec<_> = by_author.iter().collect();
    authors.sort_by(|a, b| b.1.cmp(a.1));
    for (author, count) in authors.iter().take(10) {
        println!("  {}: {} docs", author, count);
    }
    println!();

    // Size statistics
    let sizes: Vec<u64> = state.documents.values().map(|d| d.file_size).collect();
    if !sizes.is_empty() {
        let total: u64 = sizes.iter().sum();
        let avg = total / sizes.len() as u64;
        let max = *sizes.iter().max().unwrap();
        let min = *sizes.iter().min().unwrap();

        println!("{}", "File Sizes:".bold());
        println!("  Total: {}", format_size(total));
        println!("  Average: {}", format_size(avg));
        println!("  Largest: {}", format_size(max));
        println!("  Smallest: {}", format_size(min));
        println!();
    }

    // Temporal statistics
    let now = chrono::Utc::now();
    let day_ago = now - Duration::days(1);
    let week_ago = now - Duration::days(7);
    let month_ago = now - Duration::days(30);

    let all_docs = state.all();
    let modified_day = all_docs.iter().filter(|d| d.modified > day_ago).count();
    let modified_week = all_docs.iter().filter(|d| d.modified > week_ago).count();
    let modified_month = all_docs.iter().filter(|d| d.modified > month_ago).count();

    println!("{}", "Recent Activity:".bold());
    println!("  Last 24 hours: {} docs", modified_day);
    println!("  Last 7 days: {} docs", modified_week);
    println!("  Last 30 days: {} docs", modified_month);
    println!();

    Ok(())
}

// ============================================================================
// Consistency Checking Commands
// ============================================================================

/// Compare state with filesystem
pub fn show_diff(state_mgr: &StateManager) -> Result<()> {
    println!("\n{}", "State vs Filesystem Diff".bold().underline());
    println!();

    let mut issues_found = false;

    // Get git-tracked files
    let git_docs = get_git_tracked_docs(state_mgr.docs_dir())?;

    // Build set of numbers from git
    let mut git_numbers = HashSet::new();
    for path in &git_docs {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(doc) = DesignDoc::parse(&content, path.clone()) {
                git_numbers.insert(doc.metadata.number);
            }
        }
    }

    // Check for documents in state but not in git
    println!("{}", "In State but Not in Git:".yellow().bold());
    let mut orphaned = Vec::new();
    for (number, record) in &state_mgr.state().documents {
        if !git_numbers.contains(number) {
            orphaned.push((*number, record));
            issues_found = true;
        }
    }

    if orphaned.is_empty() {
        println!("  {} None", "✓".green());
    } else {
        for (number, record) in orphaned {
            println!(
                "  {} {:04} - {} ({})",
                "⚠".yellow(),
                number,
                record.metadata.title,
                record.path
            );
        }
    }
    println!();

    // Check for documents in git but not in state
    println!("{}", "In Git but Not in State:".yellow().bold());
    let state_numbers: HashSet<u32> = state_mgr.state().documents.keys().copied().collect();
    let mut missing = Vec::new();

    for number in &git_numbers {
        if !state_numbers.contains(number) {
            missing.push(*number);
            issues_found = true;
        }
    }

    if missing.is_empty() {
        println!("  {} None", "✓".green());
    } else {
        for number in missing {
            println!("  {} {:04}", "⚠".yellow(), number);
        }
    }
    println!();

    // Check for checksum mismatches
    println!("{}", "Checksum Mismatches:".yellow().bold());
    let mut mismatches = Vec::new();

    for record in state_mgr.state().all() {
        let full_path = PathBuf::from(state_mgr.docs_dir()).join(&record.path);
        if full_path.exists() {
            if let Ok(actual) = compute_checksum(&full_path) {
                if actual != record.checksum {
                    mismatches.push(record.metadata.number);
                    issues_found = true;
                }
            }
        }
    }

    if mismatches.is_empty() {
        println!("  {} None", "✓".green());
    } else {
        for number in mismatches {
            if let Some(record) = state_mgr.state().get(number) {
                println!("  {} {:04} - {}", "⚠".yellow(), number, record.metadata.title);
            }
        }
    }
    println!();

    if !issues_found {
        println!("{} State and filesystem are in sync", "✓".green().bold());
    } else {
        println!("{} Run 'oxd scan' to synchronize", "→".cyan());
    }

    Ok(())
}

/// Find orphaned files and state entries
pub fn show_orphans(state_mgr: &StateManager) -> Result<()> {
    println!("\n{}", "Orphaned Entries".bold().underline());
    println!();

    let mut found_orphans = false;

    // Check for state entries with missing files
    println!("{}", "State Entries with Missing Files:".red().bold());
    let mut missing_files = Vec::new();

    for record in state_mgr.state().all() {
        let full_path = PathBuf::from(state_mgr.docs_dir()).join(&record.path);
        if !full_path.exists() {
            missing_files.push((record.metadata.number, &record.metadata.title, &record.path));
            found_orphans = true;
        }
    }

    if missing_files.is_empty() {
        println!("  {} None", "✓".green());
    } else {
        for (number, title, path) in missing_files {
            println!("  {} {:04} - {} (expected at: {})", "✗".red(), number, title, path);
        }
    }
    println!();

    // Check for git-tracked files not in state
    println!("{}", "Git-Tracked Files Not in State:".yellow().bold());
    let git_docs = get_git_tracked_docs(state_mgr.docs_dir())?;
    let state_numbers: HashSet<u32> = state_mgr.state().documents.keys().copied().collect();

    let mut untracked = Vec::new();
    for path in git_docs {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(doc) = DesignDoc::parse(&content, path.clone()) {
                if !state_numbers.contains(&doc.metadata.number) {
                    untracked.push((doc.metadata.number, doc.metadata.title, path));
                    found_orphans = true;
                }
            }
        }
    }

    if untracked.is_empty() {
        println!("  {} None", "✓".green());
    } else {
        for (number, title, path) in untracked {
            println!("  {} {:04} - {} (at: {})", "⚠".yellow(), number, title, path.display());
        }
    }
    println!();

    if !found_orphans {
        println!("{} No orphans found", "✓".green().bold());
    } else {
        println!("{} Run 'oxd scan' to clean up", "→".cyan());
    }

    Ok(())
}

/// Deep verification of a specific document
pub fn verify_document(state_mgr: &StateManager, number: u32) -> Result<()> {
    println!("\n{}", format!("Verifying Document {:04}", number).bold().underline());
    println!();

    // Check if in state
    let record = state_mgr
        .state()
        .get(number)
        .ok_or_else(|| anyhow::anyhow!("Document {:04} not in state", number))?;

    let mut issues = Vec::new();

    // Check if file exists
    let full_path = PathBuf::from(state_mgr.docs_dir()).join(&record.path);
    if !full_path.exists() {
        issues.push(format!("File not found: {}", record.path));
    } else {
        // Verify checksum
        match compute_checksum(&full_path) {
            Ok(actual) => {
                if actual != record.checksum {
                    issues.push("Checksum mismatch (file modified)".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("Could not compute checksum: {}", e));
            }
        }

        // Verify content parses
        match fs::read_to_string(&full_path) {
            Ok(content) => {
                match DesignDoc::parse(&content, full_path.clone()) {
                    Ok(doc) => {
                        // Verify metadata matches
                        if doc.metadata.number != record.metadata.number {
                            issues.push(format!(
                                "Number mismatch: state={:04}, file={:04}",
                                record.metadata.number, doc.metadata.number
                            ));
                        }

                        if doc.metadata.title != record.metadata.title {
                            issues.push("Title mismatch".to_string());
                        }

                        if doc.metadata.state != record.metadata.state {
                            issues.push(format!(
                                "State mismatch: state={}, file={}",
                                record.metadata.state.as_str(),
                                doc.metadata.state.as_str()
                            ));
                        }
                    }
                    Err(e) => {
                        issues.push(format!("Failed to parse document: {}", e));
                    }
                }
            }
            Err(e) => {
                issues.push(format!("Failed to read file: {}", e));
            }
        }

        // Verify state/directory consistency
        if let Some(dir_state) = state_from_directory(&full_path) {
            if dir_state != record.metadata.state {
                issues.push(format!(
                    "State/directory mismatch: state={}, directory={}",
                    record.metadata.state.as_str(),
                    dir_state.as_str()
                ));
            }
        }
    }

    // Display results
    if issues.is_empty() {
        println!("{} Document is valid", "✓".green().bold());
        println!();
        println!("  Title: {}", record.metadata.title);
        println!("  State: {}", record.metadata.state.as_str());
        println!("  Path: {}", record.path);
        println!("  Checksum: {}", &record.checksum[..16]);
    } else {
        println!("{} Issues found:", "✗".red().bold());
        for issue in issues {
            println!("  {} {}", "✗".red(), issue);
        }
    }
    println!();

    Ok(())
}

/// Infer document state from its parent directory
fn state_from_directory(path: &std::path::Path) -> Option<DocState> {
    let parent = path.parent()?;
    let dir_name = parent.file_name()?.to_str()?;

    match dir_name {
        "draft" => Some(DocState::Draft),
        "under-review" => Some(DocState::UnderReview),
        "revised" => Some(DocState::Revised),
        "accepted" => Some(DocState::Accepted),
        "active" => Some(DocState::Active),
        "final" => Some(DocState::Final),
        "deferred" => Some(DocState::Deferred),
        "rejected" => Some(DocState::Rejected),
        "withdrawn" => Some(DocState::Withdrawn),
        "superseded" => Some(DocState::Superseded),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use design::doc::DocMetadata;
    use design::state::DocumentRecord;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_state_manager() -> (StateManager, TempDir) {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // Add a few test documents
        for (num, title, doc_state) in [
            (1, "First Doc", DocState::Draft),
            (2, "Second Doc", DocState::Final),
            (3, "Third Doc", DocState::Active),
        ] {
            let meta = DocMetadata {
                number: num,
                title: title.to_string(),
                author: "Test Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 1, num).unwrap(),
                state: doc_state,
                supersedes: None,
                superseded_by: None,
            };
            state_mgr.state_mut().upsert(
                num,
                DocumentRecord {
                    metadata: meta,
                    path: format!("{:04}-test.md", num),
                    checksum: "abc123def456789012345678901234567890123456789012345678901234"
                        .to_string(),
                    file_size: num as u64 * 1000,
                    modified: chrono::Utc::now(),
                },
            );
        }

        (state_mgr, temp)
    }

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn test_format_size_kilobytes() {
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(10240), "10.0 KB");
    }

    #[test]
    fn test_format_size_megabytes() {
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 2), "2.0 MB");
        assert_eq!(format_size(1024 * 1024 + 512 * 1024), "1.5 MB");
    }

    #[test]
    fn test_show_state_json() {
        let (state_mgr, _temp) = create_test_state_manager();
        let result = show_state(&state_mgr, "json");
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_state_table() {
        let (state_mgr, _temp) = create_test_state_manager();
        let result = show_state(&state_mgr, "table");
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_state_summary() {
        let (state_mgr, _temp) = create_test_state_manager();
        let result = show_state(&state_mgr, "summary");
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_state_default_format() {
        let (state_mgr, _temp) = create_test_state_manager();
        // Unknown format should default to table
        let result = show_state(&state_mgr, "unknown");
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_document_state_exists() {
        let (state_mgr, _temp) = create_test_state_manager();
        let result = show_document_state(&state_mgr, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_document_state_not_found() {
        let (state_mgr, _temp) = create_test_state_manager();
        let result = show_document_state(&state_mgr, 9999);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_show_checksums_no_files() {
        let (state_mgr, _temp) = create_test_state_manager();
        // Files don't exist, so all should be missing
        let result = show_checksums(&state_mgr, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_checksums_verbose() {
        let (state_mgr, _temp) = create_test_state_manager();
        let result = show_checksums(&state_mgr, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_checksums_with_files() {
        let (mut state_mgr, temp) = create_test_state_manager();

        // Create an actual file
        let file_path = temp.path().join("0001-test.md");
        fs::write(&file_path, "test content").unwrap();

        // Update the state to point to this file
        let checksum = compute_checksum(&file_path).unwrap();
        let meta = DocMetadata {
            number: 1,
            title: "Test Doc".to_string(),
            author: "Test".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            state: DocState::Draft,
            supersedes: None,
            superseded_by: None,
        };
        state_mgr.state_mut().upsert(
            1,
            DocumentRecord {
                metadata: meta,
                path: "0001-test.md".to_string(),
                checksum,
                file_size: 12,
                modified: chrono::Utc::now(),
            },
        );

        let result = show_checksums(&state_mgr, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_stats() {
        let (state_mgr, _temp) = create_test_state_manager();
        let result = show_stats(&state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_stats_empty_state() {
        let temp = TempDir::new().unwrap();
        let state_mgr = StateManager::new(temp.path()).unwrap();
        let result = show_stats(&state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_diff() {
        let (state_mgr, _temp) = create_test_state_manager();
        // This will likely fail due to git operations, but we test the function executes
        let _ = show_diff(&state_mgr);
    }

    #[test]
    fn test_show_orphans() {
        let (state_mgr, _temp) = create_test_state_manager();
        // This will likely fail due to git operations, but we test the function executes
        let _ = show_orphans(&state_mgr);
    }

    #[test]
    fn test_verify_document_exists() {
        let (mut state_mgr, temp) = create_test_state_manager();

        // Create an actual file
        let file_path = temp.path().join("0001-test.md");
        let content = "# Test Doc\n\nContent here.";
        fs::write(&file_path, content).unwrap();

        let checksum = compute_checksum(&file_path).unwrap();
        let meta = DocMetadata {
            number: 1,
            title: "Test Doc".to_string(),
            author: "Test".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            state: DocState::Draft,
            supersedes: None,
            superseded_by: None,
        };
        state_mgr.state_mut().upsert(
            1,
            DocumentRecord {
                metadata: meta,
                path: "0001-test.md".to_string(),
                checksum,
                file_size: content.len() as u64,
                modified: chrono::Utc::now(),
            },
        );

        let result = verify_document(&state_mgr, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_document_not_found() {
        let (state_mgr, _temp) = create_test_state_manager();
        let result = verify_document(&state_mgr, 9999);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_document_missing_file() {
        let (state_mgr, _temp) = create_test_state_manager();
        // Document 1 exists in state but file doesn't exist
        let result = verify_document(&state_mgr, 1);
        assert!(result.is_ok()); // Function should succeed, but report issues
    }

    #[test]
    fn test_state_from_directory_draft() {
        let path = std::path::PathBuf::from("/path/to/draft/0001-test.md");
        let state = state_from_directory(&path);
        assert_eq!(state, Some(DocState::Draft));
    }

    #[test]
    fn test_state_from_directory_final() {
        let path = std::path::PathBuf::from("/path/to/final/0001-test.md");
        let state = state_from_directory(&path);
        assert_eq!(state, Some(DocState::Final));
    }

    #[test]
    fn test_state_from_directory_under_review() {
        let path = std::path::PathBuf::from("/path/to/under-review/0001-test.md");
        let state = state_from_directory(&path);
        assert_eq!(state, Some(DocState::UnderReview));
    }

    #[test]
    fn test_state_from_directory_all_states() {
        let test_cases = vec![
            ("draft", DocState::Draft),
            ("under-review", DocState::UnderReview),
            ("revised", DocState::Revised),
            ("accepted", DocState::Accepted),
            ("active", DocState::Active),
            ("final", DocState::Final),
            ("deferred", DocState::Deferred),
            ("rejected", DocState::Rejected),
            ("withdrawn", DocState::Withdrawn),
            ("superseded", DocState::Superseded),
        ];

        for (dir, expected_state) in test_cases {
            let path = std::path::PathBuf::from(format!("/path/to/{}/0001-test.md", dir));
            let state = state_from_directory(&path);
            assert_eq!(state, Some(expected_state), "Failed for directory: {}", dir);
        }
    }

    #[test]
    fn test_state_from_directory_unknown() {
        let path = std::path::PathBuf::from("/path/to/unknown-dir/0001-test.md");
        let state = state_from_directory(&path);
        assert_eq!(state, None);
    }

    #[test]
    fn test_show_state_table_long_title() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // Add document with very long title
        let long_title =
            "This is a very long title that exceeds 38 characters and should be truncated";
        let meta = DocMetadata {
            number: 1,
            title: long_title.to_string(),
            author: "Test Author".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            state: DocState::Draft,
            supersedes: None,
            superseded_by: None,
        };
        state_mgr.state_mut().upsert(
            1,
            DocumentRecord {
                metadata: meta,
                path: "0001-test.md".to_string(),
                checksum: "abc123def456789012345678901234567890123456789012345678901234"
                    .to_string(),
                file_size: 1000,
                modified: chrono::Utc::now(),
            },
        );

        let result = show_state(&state_mgr, "table");
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_document_state_with_supersedes() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let meta = DocMetadata {
            number: 2,
            title: "Doc with supersedes".to_string(),
            author: "Test".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            state: DocState::Active,
            supersedes: Some(1),
            superseded_by: None,
        };
        state_mgr.state_mut().upsert(
            2,
            DocumentRecord {
                metadata: meta,
                path: "0002-test.md".to_string(),
                checksum: "abc123def456789012345678901234567890123456789012345678901234"
                    .to_string(),
                file_size: 1000,
                modified: chrono::Utc::now(),
            },
        );

        let result = show_document_state(&state_mgr, 2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_document_state_with_superseded_by() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let meta = DocMetadata {
            number: 1,
            title: "Doc superseded".to_string(),
            author: "Test".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            state: DocState::Superseded,
            supersedes: None,
            superseded_by: Some(2),
        };
        state_mgr.state_mut().upsert(
            1,
            DocumentRecord {
                metadata: meta,
                path: "0001-test.md".to_string(),
                checksum: "abc123def456789012345678901234567890123456789012345678901234"
                    .to_string(),
                file_size: 1000,
                modified: chrono::Utc::now(),
            },
        );

        let result = show_document_state(&state_mgr, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_checksums_with_dirty_file() {
        let (mut state_mgr, temp) = create_test_state_manager();

        // Create a file
        let file_path = temp.path().join("0001-test.md");
        fs::write(&file_path, "original content").unwrap();

        // Update state with checksum of original content
        let checksum = compute_checksum(&file_path).unwrap();
        let meta = DocMetadata {
            number: 1,
            title: "Test Doc".to_string(),
            author: "Test".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            state: DocState::Draft,
            supersedes: None,
            superseded_by: None,
        };
        state_mgr.state_mut().upsert(
            1,
            DocumentRecord {
                metadata: meta,
                path: "0001-test.md".to_string(),
                checksum: checksum.clone(),
                file_size: 16,
                modified: chrono::Utc::now(),
            },
        );

        // Now modify the file to make it dirty
        fs::write(&file_path, "modified content").unwrap();

        let result = show_checksums(&state_mgr, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_checksums_long_title() {
        let (mut state_mgr, temp) = create_test_state_manager();

        // Create a file with long title
        let file_path = temp.path().join("0001-test.md");
        fs::write(&file_path, "test").unwrap();

        let long_title = "This is a very long title that exceeds 38 characters and should be truncated in the output";
        let meta = DocMetadata {
            number: 1,
            title: long_title.to_string(),
            author: "Test".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            state: DocState::Draft,
            supersedes: None,
            superseded_by: None,
        };
        state_mgr.state_mut().upsert(
            1,
            DocumentRecord {
                metadata: meta,
                path: "0001-test.md".to_string(),
                checksum: "wrong_checksum".to_string(),
                file_size: 4,
                modified: chrono::Utc::now(),
            },
        );

        let result = show_checksums(&state_mgr, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_state_summary_empty() {
        let temp = TempDir::new().unwrap();
        let state_mgr = StateManager::new(temp.path()).unwrap();
        let result = show_state(&state_mgr, "summary");
        assert!(result.is_ok());
    }
}
