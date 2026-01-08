# Phase 8: Debug Tools & Search - Detailed Implementation Guide

## Overview
Add comprehensive debugging and introspection tools to help developers understand and troubleshoot the state management system, plus a powerful search command that wraps git grep with intelligent filtering.

**Prerequisites:** Phases 1-7 should be complete (especially Phase 6 for state)

---

## Task 8.1: State Inspection Commands

### Purpose
Allow users to inspect the internal state in human-readable formats.

### Implementation Steps

#### Step 1: Create Debug Module
File: `design/src/commands/debug.rs`

```rust
//! Debug and introspection commands

use anyhow::Result;
use colored::*;
use design::state::{StateManager, DocumentRecord};
use design::doc::DocState;
use prettytable::{Table, Row, Cell, format};
use serde_json;
use std::collections::HashMap;

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
    println!("Last Updated: {}", state.last_updated);
    println!("Next Number: {:04}", state.next_number);
    println!("Total Documents: {}\n", state.documents.len());
    
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BOX_CHARS);
    
    table.add_row(Row::new(vec![
        Cell::new("Num"),
        Cell::new("Title"),
        Cell::new("State"),
        Cell::new("Size"),
        Cell::new("Modified"),
        Cell::new("Checksum"),
    ]));
    
    let mut docs: Vec<_> = state.all();
    for doc in docs {
        table.add_row(Row::new(vec![
            Cell::new(&format!("{:04}", doc.metadata.number)),
            Cell::new(&doc.metadata.title),
            Cell::new(doc.metadata.state.as_str()),
            Cell::new(&format_size(doc.file_size)),
            Cell::new(&doc.modified.format("%Y-%m-%d %H:%M").to_string()),
            Cell::new(&doc.checksum[..8]), // First 8 chars of checksum
        ]));
    }
    
    table.printstd();
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
        *by_state.entry(record.metadata.state.as_str().to_string())
            .or_insert(0) += 1;
    }
    
    println!("{}", "Documents by State:".bold());
    for state_name in DocState::all_state_names() {
        let count = by_state.get(state_name).unwrap_or(&0);
        println!("  {}: {}", state_name, count);
    }
    println!();
    
    // Size statistics
    let total_size: u64 = state.documents.values().map(|d| d.file_size).sum();
    let avg_size = if !state.documents.is_empty() {
        total_size / state.documents.len() as u64
    } else {
        0
    };
    
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
    let record = state_mgr.state().get(number)
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
    println!("  Modified: {}", record.modified);
    println!("  Checksum: {}", record.checksum);
    println!();
    
    Ok(())
}
```

#### Step 2: Add Checksum Inspection
Continue in `design/src/commands/debug.rs`:

```rust
use std::path::PathBuf;
use std::fs;
use design::state::compute_checksum;

/// Show checksums and identify dirty files
pub fn show_checksums(state_mgr: &StateManager, verbose: bool) -> Result<()> {
    println!("\n{}", "Checksum Status".bold().underline());
    println!();
    
    let mut clean = 0;
    let mut dirty = 0;
    let mut missing = 0;
    
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BOX_CHARS);
    
    table.add_row(Row::new(vec![
        Cell::new("Num"),
        Cell::new("Title"),
        Cell::new("Status"),
        Cell::new("Stored Checksum"),
        Cell::new("Actual Checksum"),
    ]));
    
    for record in state_mgr.state().all() {
        let full_path = PathBuf::from(state_mgr.docs_dir()).join(&record.path);
        
        let (status, actual_checksum) = if !full_path.exists() {
            missing += 1;
            ("MISSING".red().to_string(), "-".to_string())
        } else {
            match compute_checksum(&full_path) {
                Ok(actual) => {
                    if actual == record.checksum {
                        clean += 1;
                        ("CLEAN".green().to_string(), actual)
                    } else {
                        dirty += 1;
                        ("DIRTY".yellow().to_string(), actual)
                    }
                }
                Err(_) => {
                    missing += 1;
                    ("ERROR".red().to_string(), "-".to_string())
                }
            }
        };
        
        if verbose || status != "CLEAN".green().to_string() {
            table.add_row(Row::new(vec![
                Cell::new(&format!("{:04}", record.metadata.number)),
                Cell::new(&record.metadata.title),
                Cell::new(&status),
                Cell::new(&record.checksum[..16]),
                Cell::new(&actual_checksum[..16]),
            ]));
        }
    }
    
    if verbose || dirty > 0 || missing > 0 {
        table.printstd();
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
        by_state.entry(record.metadata.state.as_str().to_string())
            .or_insert_with(Vec::new)
            .push(record);
    }
    
    println!("{}", "By State:".bold());
    let mut states: Vec<_> = by_state.keys().collect();
    states.sort();
    for state_name in states {
        let docs = &by_state[state_name];
        println!("  {}: {} docs", state_name, docs.len());
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
    let day_ago = now - chrono::Duration::days(1);
    let week_ago = now - chrono::Duration::days(7);
    let month_ago = now - chrono::Duration::days(30);
    
    let modified_day = state.all().iter().filter(|d| d.modified > day_ago).count();
    let modified_week = state.all().iter().filter(|d| d.modified > week_ago).count();
    let modified_month = state.all().iter().filter(|d| d.modified > month_ago).count();
    
    println!("{}", "Recent Activity:".bold());
    println!("  Last 24 hours: {} docs", modified_day);
    println!("  Last 7 days: {} docs", modified_week);
    println!("  Last 30 days: {} docs", modified_month);
    println!();
    
    Ok(())
}
```

### Testing Steps for 8.1
1. Test state display in different formats
2. Test document-specific state view
3. Test checksum verification
4. Test statistics generation
5. Verify table formatting

---

## Task 8.2: Consistency Checking Commands

### Purpose
Find and report inconsistencies between state, filesystem, and index.

### Implementation Steps

#### Step 1: Add Diff Command
File: `design/src/commands/debug.rs`

```rust
use design::index_sync::get_git_tracked_docs;
use design::doc::DesignDoc;
use std::collections::HashSet;

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
                println!(
                    "  {} {:04} - {}",
                    "⚠".yellow(),
                    number,
                    record.metadata.title
                );
            }
        }
    }
    println!();
    
    if !issues_found {
        println!("{} State and filesystem are in sync", "✓".green().bold());
    } else {
        println!(
            "{} Run 'oxd scan' to synchronize",
            "→".cyan()
        );
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
            println!(
                "  {} {:04} - {} (expected at: {})",
                "✗".red(),
                number,
                title,
                path
            );
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
            println!(
                "  {} {:04} - {} (at: {})",
                "⚠".yellow(),
                number,
                title,
                path.display()
            );
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
    let record = state_mgr.state().get(number)
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
                                record.metadata.number,
                                doc.metadata.number
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
        use design::doc::state_from_directory;
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
```

### Testing Steps for 8.2
1. Test diff with in-sync state
2. Test diff with orphaned entries
3. Test orphans detection
4. Test document verification
5. Create inconsistencies and verify detection

---

## Task 8.3: Search Command

### Purpose
Powerful search that wraps git grep with intelligent filtering.

### Implementation Steps

#### Step 1: Create Search Command
File: `design/src/commands/search.rs`

```rust
//! Search command implementation

use anyhow::Result;
use colored::*;
use design::state::StateManager;
use design::doc::DocState;
use std::process::Command;
use regex::Regex;

pub fn search(
    state_mgr: &StateManager,
    query: &str,
    state_filter: Option<String>,
    metadata_only: bool,
    case_sensitive: bool,
) -> Result<()> {
    println!(
        "{} Searching for: {}\n",
        "→".cyan(),
        query.bold()
    );
    
    // Build git grep command
    let mut cmd = Command::new("git");
    cmd.arg("grep");
    
    // Options
    cmd.arg("-n"); // Show line numbers
    cmd.arg("--color=always"); // Colorize output
    
    if !case_sensitive {
        cmd.arg("-i"); // Case insensitive
    }
    
    // Pattern
    cmd.arg(query);
    
    // Paths to search
    if let Some(state_str) = state_filter {
        // Filter by state directory
        if let Some(state) = DocState::from_str_flexible(&state_str) {
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
    
    if output.status.success() {
        let results = String::from_utf8_lossy(&output.stdout);
        
        // Parse and enhance results
        display_results(&results, state_mgr, metadata_only)?;
        
        // Count results
        let line_count = results.lines().count();
        println!("\n{} {} matches found", "✓".green(), line_count);
    } else {
        println!("{} No matches found", "→".cyan());
    }
    
    Ok(())
}

fn display_results(
    results: &str,
    state_mgr: &StateManager,
    metadata_only: bool,
) -> Result<()> {
    // Pattern to extract: path:line:content
    let re = Regex::new(r"^([^:]+):(\d+):(.*)$").unwrap();
    
    let mut current_file = String::new();
    
    for line in results.lines() {
        if let Some(caps) = re.captures(line) {
            let path = caps.get(1).unwrap().as_str();
            let line_num = caps.get(2).unwrap().as_str();
            let content = caps.get(3).unwrap().as_str();
            
            // Extract document number from path
            let doc_number = extract_number_from_path(path);
            
            // Check if in YAML frontmatter (lines < ~15 usually)
            let is_metadata = line_num.parse::<usize>().unwrap_or(999) < 15;
            
            // Skip if metadata_only and not in metadata
            if metadata_only && !is_metadata {
                continue;
            }
            
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
            
            // Print the match
            println!("  {}:{}", line_num.dimmed(), content);
        }
    }
    
    Ok(())
}

fn extract_number_from_path(path: &str) -> Option<u32> {
    let re = Regex::new(r"(\d{4})-").unwrap();
    re.captures(path)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

/// Search with more advanced options
pub fn search_advanced(
    state_mgr: &StateManager,
    query: &str,
    options: SearchOptions,
) -> Result<()> {
    // Extended search with regex support, context lines, etc.
    // TODO: Implement advanced features
    search(
        state_mgr,
        query,
        options.state,
        options.metadata_only,
        options.case_sensitive,
    )
}

pub struct SearchOptions {
    pub state: Option<String>,
    pub metadata_only: bool,
    pub case_sensitive: bool,
    pub context_lines: usize,
    pub regex: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        SearchOptions {
            state: None,
            metadata_only: false,
            case_sensitive: false,
            context_lines: 0,
            regex: false,
        }
    }
}
```

#### Step 2: Export Modules
File: `design/src/commands/mod.rs`

```rust
pub mod debug;
pub mod search;

pub use debug::*;
pub use search::search;
```

#### Step 3: Update CLI
File: `design/src/cli.rs`

```rust
/// Debug and introspection commands
#[command(subcommand)]
Debug(DebugCommands),

/// Search documents
Search {
    /// Search query
    query: String,
    
    /// Filter by state
    #[arg(short, long)]
    state: Option<String>,
    
    /// Search only in metadata (YAML frontmatter)
    #[arg(short, long)]
    metadata: bool,
    
    /// Case-sensitive search
    #[arg(short = 'I', long)]
    case_sensitive: bool,
},

#[derive(Subcommand)]
pub enum DebugCommands {
    /// Show state information
    State {
        /// Document number (optional, shows all if omitted)
        number: Option<u32>,
        
        /// Output format (json, table, summary)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Show checksums and dirty files
    Checksums {
        /// Show all files, not just dirty ones
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Show repository statistics
    Stats,
    
    /// Show diff between state and filesystem
    Diff,
    
    /// Show orphaned entries
    Orphans,
    
    /// Verify a specific document
    Verify {
        /// Document number
        number: u32,
    },
}
```

#### Step 4: Wire Up in Main
File: `design/src/main.rs`

```rust
Commands::Debug(debug_cmd) => {
    match debug_cmd {
        DebugCommands::State { number, format } => {
            if let Some(num) = number {
                show_document_state(&state_mgr, *num)?;
            } else {
                show_state(&state_mgr, format)?;
            }
        }
        DebugCommands::Checksums { verbose } => {
            show_checksums(&state_mgr, *verbose)?;
        }
        DebugCommands::Stats => {
            show_stats(&state_mgr)?;
        }
        DebugCommands::Diff => {
            show_diff(&state_mgr)?;
        }
        DebugCommands::Orphans => {
            show_orphans(&state_mgr)?;
        }
        DebugCommands::Verify { number } => {
            verify_document(&state_mgr, *number)?;
        }
    }
}

Commands::Search { query, state, metadata, case_sensitive } => {
    search(&state_mgr, query, state.clone(), *metadata, *case_sensitive)?;
}
```

### Testing Steps for 8.3
1. Test basic search
2. Test with state filter
3. Test metadata-only search
4. Test case sensitivity
5. Test with no results
6. Test result formatting

---

## Verification Checklist

- [ ] State inspection shows readable output
- [ ] Document-specific state view works
- [ ] Checksum status displays correctly
- [ ] Statistics are accurate
- [ ] Diff detects inconsistencies
- [ ] Orphan detection works
- [ ] Document verification is thorough
- [ ] Search wraps git grep correctly
- [ ] Search filters by state
- [ ] Search results are well-formatted

---

## Usage Examples

```bash
# Inspect state
oxd debug state                    # All documents
oxd debug state 42                 # Specific document
oxd debug state --format json      # JSON output
oxd debug state --format summary   # Summary only

# Check consistency
oxd debug checksums                # Show dirty files
oxd debug checksums --verbose      # Show all
oxd debug diff                     # State vs filesystem
oxd debug orphans                  # Find orphaned entries
oxd debug verify 42                # Deep check one doc

# Statistics
oxd debug stats                    # Full stats

# Search
oxd search "authentication"        # Basic search
oxd search "API" --state draft     # In drafts only
oxd search "author" --metadata     # In YAML only
oxd search "TODO" -I               # Case sensitive
```

---

## Notes for Claude Code

- **Wrap git grep** - leverage existing tool, add intelligence
- **Make output scannable** - use colors, tables, clear sections
- **Show context** - document titles, states alongside results
- **Performance** - checksums can be slow, consider caching
- **Error handling** - git commands can fail, handle gracefully
- **Add prettytable** dependency for nice tables
