# Phase 6: State Management & Source of Truth - Detailed Implementation Guide

## Overview
This phase establishes a **canonical source of truth** for document state, implements change detection, and ensures consistency between files, metadata, and the index. This solves critical architectural issues around data consistency and performance.

**Prerequisites:** Phases 1-5 should be complete (though this can be implemented alongside them)

---

## Problem Statement

### Current Architecture Issues

**Three "Sources of Truth" Can Diverge:**
1. **YAML frontmatter** in each .md file
2. **00-index.md** markdown file
3. **Physical directory structure** (state directories)

**Consequences:**
- External git operations can cause drift
- Manual file edits aren't tracked
- No way to detect what changed since last run
- Full re-parse on every command (slow)
- Validation only catches issues after they occur
- No atomic operations across multiple files

**Real-World Scenarios That Break:**
```bash
# Scenario 1: External git operation
git mv 01-draft/0001-feature.md 02-under-review/
# → YAML still says "Draft", directory says "Under Review"

# Scenario 2: Manual YAML edit
vim 01-draft/0001-feature.md  # Change state to "Final"
# → File says "Final", directory says "Draft", index says "Draft"

# Scenario 3: Concurrent operations
# Two terminal windows both running oxd commands
# → Race condition, last write wins, no consistency guarantees
```

---

## Solution Architecture

### Single Source of Truth

**Design Decision:** Create a **canonical state file** that is:
1. The authoritative source for all document metadata
2. Serialized to disk in a simple, fast format
3. Loaded into memory on startup
4. Updated atomically with all operations
5. Used to detect changes in files/filesystem

**State File Location:** `.oxd/state.json` (or `.oxd/state.toml`)

### Data Flow

```
Startup:
  1. Load state.json → Memory
  2. Detect changed files (checksums/mtimes)
  3. Re-scan changed files
  4. Update state.json
  5. Ready for commands

Operation:
  1. Read from memory state
  2. Perform operation
  3. Update files
  4. Update state.json
  5. Commit changes

Scan Command:
  1. Re-scan all files
  2. Compare with state.json
  3. Report inconsistencies
  4. Optionally fix
  5. Update state.json
```

---

## Task 6.1: Choose Serialization Format

### Purpose
Select a simple, reliable serialization format for the state file.

### Options Analysis

#### Option 1: JSON (serde_json)
**Pros:**
- Already in dependencies (Phase 2)
- Human-readable
- Wide tool support
- Fast serialization

**Cons:**
- No comments
- Verbose for large datasets

#### Option 2: TOML (toml)
**Pros:**
- Human-readable
- Supports comments
- Nice for config files

**Cons:**
- Slower than JSON
- Less suitable for large data

#### Option 3: Bincode (bincode)
**Pros:**
- Very fast
- Compact
- Efficient

**Cons:**
- Binary format (not human-readable)
- Versioning challenges

#### Option 4: RON (ron)
**Pros:**
- Rust-native
- Human-readable
- Supports Rust types well

**Cons:**
- Less common
- Smaller ecosystem

### Recommendation

**Use JSON** for the state file:
- Already have serde_json dependency
- Fast enough for our use case
- Human-readable (can inspect/debug)
- Wide tooling support

**Use separate JSON file for cache:**
- `.oxd/state.json` - canonical state
- `.oxd/cache.json` - file checksums/mtimes

### Implementation

File: `design/Cargo.toml`

```toml
[dependencies]
# Already present from Phase 2
serde_json = "1"
```

---

## Task 6.2: Formalize Schemas

### Purpose
Define proper data structures for all state.

### Implementation Steps

#### Step 1: Create State Module
File: `design/src/state.rs`

```rust
//! State management and persistence

use crate::doc::{DocMetadata, DocState};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

/// The canonical state of all documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentState {
    /// Schema version for migrations
    pub version: u32,
    
    /// When this state was last updated
    pub last_updated: chrono::DateTime<chrono::Utc>,
    
    /// All documents keyed by number
    pub documents: HashMap<u32, DocumentRecord>,
    
    /// Next available document number
    pub next_number: u32,
}

/// A single document's canonical record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRecord {
    /// Document metadata
    pub metadata: DocMetadata,
    
    /// Relative path from docs_dir
    pub path: String,
    
    /// File checksum (for change detection)
    pub checksum: String,
    
    /// File size in bytes
    pub file_size: u64,
    
    /// Last modified time (from filesystem)
    pub modified: chrono::DateTime<chrono::Utc>,
}

impl DocumentState {
    /// Create a new empty state
    pub fn new() -> Self {
        DocumentState {
            version: 1,
            last_updated: chrono::Utc::now(),
            documents: HashMap::new(),
            next_number: 1,
        }
    }
    
    /// Load state from disk
    pub fn load(state_dir: impl AsRef<Path>) -> Result<Self> {
        let state_file = state_dir.as_ref().join("state.json");
        
        if !state_file.exists() {
            return Ok(Self::new());
        }
        
        let content = fs::read_to_string(&state_file)
            .context("Failed to read state file")?;
        
        let state: DocumentState = serde_json::from_str(&content)
            .context("Failed to parse state file")?;
        
        Ok(state)
    }
    
    /// Save state to disk
    pub fn save(&self, state_dir: impl AsRef<Path>) -> Result<()> {
        let state_dir = state_dir.as_ref();
        fs::create_dir_all(state_dir)
            .context("Failed to create state directory")?;
        
        let state_file = state_dir.join("state.json");
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize state")?;
        
        // Atomic write: write to temp file, then rename
        let temp_file = state_dir.join("state.json.tmp");
        fs::write(&temp_file, content)
            .context("Failed to write temp state file")?;
        
        fs::rename(&temp_file, &state_file)
            .context("Failed to rename state file")?;
        
        Ok(())
    }
    
    /// Add or update a document record
    pub fn upsert(&mut self, number: u32, record: DocumentRecord) {
        self.documents.insert(number, record);
        self.last_updated = chrono::Utc::now();
        
        // Update next_number if needed
        if number >= self.next_number {
            self.next_number = number + 1;
        }
    }
    
    /// Remove a document record
    pub fn remove(&mut self, number: u32) -> Option<DocumentRecord> {
        self.last_updated = chrono::Utc::now();
        self.documents.remove(&number)
    }
    
    /// Get a document record
    pub fn get(&self, number: u32) -> Option<&DocumentRecord> {
        self.documents.get(&number)
    }
    
    /// Get all documents
    pub fn all(&self) -> Vec<&DocumentRecord> {
        let mut docs: Vec<_> = self.documents.values().collect();
        docs.sort_by_key(|d| d.metadata.number);
        docs
    }
    
    /// Get documents by state
    pub fn by_state(&self, state: DocState) -> Vec<&DocumentRecord> {
        let mut docs: Vec<_> = self.documents.values()
            .filter(|d| d.metadata.state == state)
            .collect();
        docs.sort_by_key(|d| d.metadata.number);
        docs
    }
}
```

#### Step 2: Add Checksum Utilities
Continue in `design/src/state.rs`:

```rust
use sha2::{Sha256, Digest};
use std::io::Read;

/// Compute SHA-256 checksum of a file
pub fn compute_checksum(path: impl AsRef<Path>) -> Result<String> {
    let mut file = fs::File::open(path.as_ref())
        .context("Failed to open file for checksum")?;
    
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];
    
    loop {
        let n = file.read(&mut buffer)
            .context("Failed to read file for checksum")?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    
    Ok(format!("{:x}", hasher.finalize()))
}

/// Check if a file has changed based on checksum
pub fn file_changed(path: impl AsRef<Path>, expected_checksum: &str) -> Result<bool> {
    let actual = compute_checksum(path)?;
    Ok(actual != expected_checksum)
}

/// Get file metadata (size, mtime)
pub fn file_metadata(path: impl AsRef<Path>) -> Result<(u64, chrono::DateTime<chrono::Utc>)> {
    let metadata = fs::metadata(path.as_ref())
        .context("Failed to read file metadata")?;
    
    let size = metadata.len();
    let modified = metadata.modified()
        .context("Failed to get modification time")?;
    
    let datetime = chrono::DateTime::<chrono::Utc>::from(modified);
    
    Ok((size, datetime))
}
```

#### Step 3: Export Module
File: `design/src/lib.rs`

```rust
pub mod state;
```

#### Step 4: Add Dependencies
File: `design/Cargo.toml`

```toml
[dependencies]
sha2 = "0.10"
```

### Testing Steps for 6.2
1. Create state, add records, save/load
2. Verify JSON format is readable
3. Test checksum computation
4. Test file change detection
5. Verify atomic writes (temp file rename)

---

## Task 6.3: State Initialization on Startup

### Purpose
Load state at startup, detect changes, and ensure consistency before commands run.

### Implementation Steps

#### Step 1: Create State Manager
File: `design/src/state.rs`

Add to the file:

```rust
use crate::doc::DesignDoc;
use crate::index_sync::get_git_tracked_docs;

/// State manager handles loading, updating, and persisting state
pub struct StateManager {
    state: DocumentState,
    docs_dir: PathBuf,
    state_dir: PathBuf,
}

impl StateManager {
    /// Initialize state manager
    pub fn new(docs_dir: impl AsRef<Path>) -> Result<Self> {
        let docs_dir = docs_dir.as_ref().to_path_buf();
        let state_dir = docs_dir.join(".oxd");
        
        // Load existing state or create new
        let state = DocumentState::load(&state_dir)?;
        
        Ok(StateManager {
            state,
            docs_dir,
            state_dir,
        })
    }
    
    /// Initialize and scan for changes
    pub fn init_with_scan(&mut self) -> Result<ScanResult> {
        self.scan_for_changes()
    }
    
    /// Scan filesystem for changes
    pub fn scan_for_changes(&mut self) -> Result<ScanResult> {
        let mut result = ScanResult::new();
        
        // Get all git-tracked docs
        let git_docs = get_git_tracked_docs(&self.docs_dir)?;
        
        // Check each tracked file
        for path in &git_docs {
            let content = fs::read_to_string(path)
                .context(format!("Failed to read {}", path.display()))?;
            
            // Parse document
            let doc = match DesignDoc::parse(&content, path.clone()) {
                Ok(d) => d,
                Err(e) => {
                    result.errors.push(format!(
                        "Failed to parse {}: {}",
                        path.display(),
                        e
                    ));
                    continue;
                }
            };
            
            let number = doc.metadata.number;
            
            // Check if we have a record
            if let Some(record) = self.state.get(number) {
                // Check if file changed
                if file_changed(path, &record.checksum)? {
                    result.changed.push(number);
                    self.update_record_from_file(&doc, path)?;
                }
            } else {
                // New file not in state
                result.new_files.push(number);
                self.update_record_from_file(&doc, path)?;
            }
        }
        
        // Check for deleted files
        let git_numbers: std::collections::HashSet<u32> = git_docs
            .iter()
            .filter_map(|p| {
                fs::read_to_string(p)
                    .ok()
                    .and_then(|c| DesignDoc::parse(&c, p.clone()).ok())
                    .map(|d| d.metadata.number)
            })
            .collect();
        
        for number in self.state.documents.keys() {
            if !git_numbers.contains(number) {
                result.deleted.push(*number);
            }
        }
        
        // Remove deleted files from state
        for number in &result.deleted {
            self.state.remove(*number);
        }
        
        // Save updated state
        self.save()?;
        
        Ok(result)
    }
    
    /// Update a record from a file
    fn update_record_from_file(&mut self, doc: &DesignDoc, path: &Path) -> Result<()> {
        let checksum = compute_checksum(path)?;
        let (file_size, modified) = file_metadata(path)?;
        
        let rel_path = path.strip_prefix(&self.docs_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        
        let record = DocumentRecord {
            metadata: doc.metadata.clone(),
            path: rel_path,
            checksum,
            file_size,
            modified,
        };
        
        self.state.upsert(doc.metadata.number, record);
        Ok(())
    }
    
    /// Get state reference
    pub fn state(&self) -> &DocumentState {
        &self.state
    }
    
    /// Get mutable state reference
    pub fn state_mut(&mut self) -> &mut DocumentState {
        &mut self.state
    }
    
    /// Save state to disk
    pub fn save(&self) -> Result<()> {
        self.state.save(&self.state_dir)
    }
    
    /// Get docs directory
    pub fn docs_dir(&self) -> &Path {
        &self.docs_dir
    }
}

/// Results of a filesystem scan
#[derive(Debug)]
pub struct ScanResult {
    pub new_files: Vec<u32>,
    pub changed: Vec<u32>,
    pub deleted: Vec<u32>,
    pub errors: Vec<String>,
}

impl ScanResult {
    fn new() -> Self {
        ScanResult {
            new_files: Vec::new(),
            changed: Vec::new(),
            deleted: Vec::new(),
            errors: Vec::new(),
        }
    }
    
    pub fn has_changes(&self) -> bool {
        !self.new_files.is_empty() 
            || !self.changed.is_empty() 
            || !self.deleted.is_empty()
    }
}
```

#### Step 2: Update Main to Use State Manager
File: `design/src/main.rs`

```rust
use design::state::StateManager;
use colored::*;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize state manager
    let mut state_mgr = match StateManager::new(&cli.docs_dir) {
        Ok(mgr) => mgr,
        Err(e) => {
            design::errors::print_error_with_suggestion(
                "Failed to initialize state manager",
                &e,
                "Make sure you're in a valid design docs directory"
            );
            std::process::exit(1);
        }
    };
    
    // Scan for changes (unless running scan command explicitly)
    let needs_scan = !matches!(cli.command, Commands::Scan { .. });
    
    if needs_scan {
        if let Ok(result) = state_mgr.init_with_scan() {
            if result.has_changes() {
                // Silently update, or show brief message
                if !result.new_files.is_empty() || !result.changed.is_empty() {
                    eprintln!(
                        "{} Detected changes ({} new, {} modified)",
                        "→".cyan(),
                        result.new_files.len(),
                        result.changed.len()
                    );
                }
            }
        }
    }
    
    // Create DocumentIndex from state (for compatibility with existing commands)
    let index = DocumentIndex::from_state(state_mgr.state())?;

    // Execute the command
    let result = match cli.command {
        // ... existing commands, but pass both index and state_mgr where needed
        Commands::Scan { fix, verbose } => {
            scan_documents(&mut state_mgr, fix, verbose)
        }
        // ... other commands
    };

    if let Err(e) = result {
        design::errors::print_error("Command failed", &e);
        std::process::exit(1);
    }

    Ok(())
}
```

#### Step 3: Update DocumentIndex to Work with State
File: `design/src/index.rs`

```rust
use crate::state::DocumentState;

impl DocumentIndex {
    /// Create index from state
    pub fn from_state(state: &DocumentState) -> Result<Self> {
        let mut docs = HashMap::new();
        
        for record in state.documents.values() {
            let doc = DesignDoc {
                metadata: record.metadata.clone(),
                content: String::new(), // Don't load content unless needed
                path: PathBuf::from(&record.path),
            };
            docs.insert(record.metadata.number, doc);
        }
        
        Ok(DocumentIndex {
            docs,
            docs_dir: PathBuf::new(), // Will be set by caller
        })
    }
}
```

### Testing Steps for 6.3
1. Test state initialization with empty directory
2. Test state load with existing state file
3. Test change detection (modify file, re-run)
4. Test new file detection
5. Test deleted file detection
6. Verify startup is fast with cached state

---

## Task 6.4: Implement Scan Command

### Purpose
Explicit command to validate and sync state with filesystem.

### Implementation Steps

#### Step 1: Create Scan Command
File: `design/src/commands/scan.rs`

```rust
//! Scan command implementation

use anyhow::Result;
use colored::*;
use design::state::StateManager;

/// Scan filesystem and validate/update state
pub fn scan_documents(
    state_mgr: &mut StateManager,
    fix: bool,
    verbose: bool,
) -> Result<()> {
    println!("\n{}\n", "Scanning documents...".bold());
    
    let result = state_mgr.scan_for_changes()?;
    
    // Report changes
    if result.has_changes() {
        if !result.new_files.is_empty() {
            println!("{}", "New Files:".green().bold());
            for num in &result.new_files {
                let record = state_mgr.state().get(*num).unwrap();
                println!("  {} {:04} - {}", 
                    "✓".green(), 
                    num, 
                    record.metadata.title
                );
            }
            println!();
        }
        
        if !result.changed.is_empty() {
            println!("{}", "Modified Files:".yellow().bold());
            for num in &result.changed {
                let record = state_mgr.state().get(*num).unwrap();
                println!("  {} {:04} - {}", 
                    "⟳".yellow(), 
                    num, 
                    record.metadata.title
                );
            }
            println!();
        }
        
        if !result.deleted.is_empty() {
            println!("{}", "Deleted Files:".red().bold());
            for num in &result.deleted {
                println!("  {} {:04}", "✗".red(), num);
            }
            println!();
        }
    } else {
        println!("{} No changes detected\n", "✓".green().bold());
    }
    
    // Report errors
    if !result.errors.is_empty() {
        println!("{}", "Errors:".red().bold());
        for error in &result.errors {
            println!("  {} {}", "✗".red(), error);
        }
        println!();
    }
    
    // Validate consistency
    if verbose {
        validate_consistency(state_mgr)?;
    }
    
    // Summary
    println!(
        "{} State updated: {} documents tracked",
        "✓".green().bold(),
        state_mgr.state().documents.len()
    );
    
    Ok(())
}

fn validate_consistency(state_mgr: &StateManager) -> Result<()> {
    use crate::doc::{state_from_directory, is_in_state_dir};
    use std::path::PathBuf;
    
    println!("{}", "Validating Consistency:".bold());
    
    let mut inconsistencies = 0;
    
    for record in state_mgr.state().all() {
        let full_path = PathBuf::from(state_mgr.docs_dir()).join(&record.path);
        
        // Check if file exists
        if !full_path.exists() {
            println!(
                "  {} {:04} - File not found: {}",
                "✗".red(),
                record.metadata.number,
                record.path
            );
            inconsistencies += 1;
            continue;
        }
        
        // Check state/directory consistency
        if let Some(dir_state) = state_from_directory(&full_path) {
            if record.metadata.state != dir_state {
                println!(
                    "  {} {:04} - State mismatch: YAML='{}' Directory='{}'",
                    "⚠".yellow(),
                    record.metadata.number,
                    record.metadata.state.as_str(),
                    dir_state.as_str()
                );
                inconsistencies += 1;
            }
        }
    }
    
    if inconsistencies == 0 {
        println!("  {} All documents consistent", "✓".green());
    } else {
        println!("  {} {} inconsistencies found", "⚠".yellow(), inconsistencies);
    }
    
    println!();
    Ok(())
}
```

#### Step 2: Update Commands Module
File: `design/src/commands/mod.rs`

```rust
pub mod scan;
pub use scan::scan_documents;
```

#### Step 3: Update CLI
File: `design/src/cli.rs`

```rust
/// Scan filesystem and validate document state
#[command(visible_alias = "rescan")]
Scan {
    /// Fix issues automatically where possible
    #[arg(short, long)]
    fix: bool,
    
    /// Show detailed validation output
    #[arg(short, long)]
    verbose: bool,
},
```

### Testing Steps for 6.4
1. Run scan with no changes
2. Modify a file, run scan
3. Add a new file, run scan
4. Delete a file, run scan
5. Test --verbose flag
6. Test --fix flag (once implemented)

---

## Task 6.5: Update File Write Operations

### Purpose
Ensure all operations that modify files also update the state.

### Implementation Steps

#### Step 1: Create State Update Helpers
File: `design/src/state.rs`

```rust
impl StateManager {
    /// Update state after modifying a file
    pub fn record_file_change(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        
        // Re-parse the file
        let content = std::fs::read_to_string(path)
            .context("Failed to read modified file")?;
        
        let doc = DesignDoc::parse(&content, path.to_path_buf())
            .context("Failed to parse modified file")?;
        
        // Update record
        self.update_record_from_file(&doc, path)?;
        
        // Save state
        self.save()?;
        
        Ok(())
    }
    
    /// Update state after moving a file
    pub fn record_file_move(
        &mut self,
        old_path: impl AsRef<Path>,
        new_path: impl AsRef<Path>,
    ) -> Result<()> {
        // Just re-record the file at new location
        self.record_file_change(new_path)
    }
    
    /// Remove file from state
    pub fn record_file_deletion(&mut self, number: u32) -> Result<()> {
        self.state.remove(number);
        self.save()?;
        Ok(())
    }
}
```

#### Step 2: Update Add Headers Command
File: `design/src/commands/add_headers.rs`

```rust
pub fn add_headers(state_mgr: &mut StateManager, doc_path: &str) -> Result<()> {
    // ... existing implementation ...
    
    // Write updated content
    fs::write(&path, new_content)
        .context("Failed to write file")?;
    
    // Update state
    state_mgr.record_file_change(&path)?;
    
    // ... rest of implementation ...
}
```

#### Step 3: Update Transition Command
File: `design/src/commands/transition.rs`

```rust
pub fn transition_document(
    state_mgr: &mut StateManager,
    doc_path: &str,
    new_state_str: &str,
) -> Result<()> {
    // ... existing implementation up to git mv ...
    
    design::git::git_mv(&path, &new_path)
        .context("Failed to move document")?;
    
    // Update state with new location
    state_mgr.record_file_move(&path, &new_path)?;
    
    // ... rest of implementation ...
}
```

#### Step 4: Update Add Command
File: `design/src/commands/add.rs`

```rust
pub fn add_document(
    state_mgr: &mut StateManager,
    doc_path: &str,
    dry_run: bool,
) -> Result<()> {
    // ... existing implementation ...
    
    // After each file modification:
    if !dry_run {
        state_mgr.record_file_change(&path)?;
    }
    
    // ... rest of implementation ...
}
```

### Testing Steps for 6.5
1. Add headers to file, verify state updated
2. Transition file, verify state updated
3. Add new file, verify state updated
4. Check state.json after each operation
5. Verify checksums updated

---

## Task 6.6: Optimized File Reading

### Purpose
Read from state when possible, only re-parse files when needed.

### Implementation Steps

#### Step 1: Add Lazy Loading to DocumentIndex
File: `design/src/index.rs`

```rust
impl DocumentIndex {
    /// Get document with lazy content loading
    pub fn get_with_content(&self, number: u32) -> Option<DesignDoc> {
        let doc = self.docs.get(&number)?;
        
        // If content is empty, load it
        if doc.content.is_empty() {
            if let Ok(content) = std::fs::read_to_string(&doc.path) {
                let mut loaded_doc = doc.clone();
                loaded_doc.content = content;
                return Some(loaded_doc);
            }
        }
        
        Some(doc.clone())
    }
}
```

#### Step 2: Update Show Command
File: `design/src/commands/show.rs`

```rust
pub fn show_document(index: &DocumentIndex, number: u32, metadata_only: bool) -> Result<()> {
    let doc = index.get_with_content(number)
        .ok_or_else(|| anyhow::anyhow!("Document {:04} not found", number))?;
    
    // ... rest of implementation uses doc.content ...
}
```

### Testing Steps for 6.6
1. Run list (should be fast, no content loading)
2. Run show (should load content on demand)
3. Verify performance improvement for large repos
4. Test with missing files (graceful degradation)

---

## Task 6.7: Change Detection Optimization

### Purpose
Make startup fast by only re-scanning changed files.

### Implementation Steps

#### Step 1: Add Quick Change Detection
File: `design/src/state.rs`

```rust
impl StateManager {
    /// Quick check if file might have changed (without full checksum)
    fn quick_check_changed(
        &self,
        path: &Path,
        record: &DocumentRecord,
    ) -> Result<bool> {
        let (size, modified) = file_metadata(path)?;
        
        // Quick checks first
        if size != record.file_size {
            return Ok(true);
        }
        
        if modified > record.modified {
            return Ok(true);
        }
        
        // Size and mtime match, probably unchanged
        Ok(false)
    }
    
    /// Fast scan using quick checks
    pub fn quick_scan(&mut self) -> Result<ScanResult> {
        let mut result = ScanResult::new();
        let git_docs = get_git_tracked_docs(&self.docs_dir)?;
        
        for path in &git_docs {
            let content = fs::read_to_string(path)?;
            let doc = DesignDoc::parse(&content, path.clone())?;
            let number = doc.metadata.number;
            
            if let Some(record) = self.state.get(number) {
                // Quick check first
                if self.quick_check_changed(path, record)? {
                    // Verify with full checksum
                    if file_changed(path, &record.checksum)? {
                        result.changed.push(number);
                        self.update_record_from_file(&doc, path)?;
                    }
                }
            } else {
                result.new_files.push(number);
                self.update_record_from_file(&doc, path)?;
            }
        }
        
        // Check for deletions
        // ... same as before ...
        
        if result.has_changes() {
            self.save()?;
        }
        
        Ok(result)
    }
}
```

#### Step 2: Use Quick Scan on Startup
File: `design/src/main.rs`

```rust
// In main(), replace init_with_scan with:
if needs_scan {
    if let Ok(result) = state_mgr.quick_scan() {
        // ... handle result ...
    }
}
```

### Testing Steps for 6.7
1. Measure startup time with large repo (100+ docs)
2. Modify one file, verify only that file rescanned
3. Touch file (mtime change), verify detected
4. Verify accuracy vs full scan

---

## Verification Checklist

Before considering Phase 6 complete:

- [ ] State file created and loaded on startup
- [ ] State persists across runs
- [ ] File changes detected correctly
- [ ] New files added to state
- [ ] Deleted files removed from state
- [ ] All write operations update state
- [ ] Scan command works correctly
- [ ] Quick scan is fast
- [ ] State is atomic (temp file + rename)
- [ ] Errors don't corrupt state
- [ ] State migration plan exists (version field)
- [ ] Documentation updated

---

## Migration Guide

### For Existing Repositories

When users first run the updated tool:

```bash
# First run will build state from scratch
oxd list
# → Scanning 50 documents...
# → State initialized

# Subsequent runs are fast
oxd list
# → (instant)

# Explicit scan when needed
oxd scan --verbose
```

### State File Location

```
docs/
├── .oxd/
│   ├── state.json          # Canonical state
│   └── .gitignore          # Don't commit state file
├── 00-index.md
└── 01-draft/
    └── ...
```

### Gitignore

Add to `docs/.gitignore`:
```
.oxd/
```

---

## Performance Metrics

### Expected Performance

**Cold Start (no state file):**
- 10 docs: < 100ms
- 50 docs: < 500ms
- 100 docs: < 1s

**Warm Start (with state):**
- Any size: < 50ms (just load JSON)

**Quick Scan (no changes):**
- 100 docs: < 200ms (stat calls only)

**Full Scan:**
- 100 docs: < 2s (full checksum)

---

## Future Enhancements

### Phase 6.1: Watch Mode
```bash
oxd watch
# → Watches filesystem, auto-updates state
```

### Phase 6.2: State Inspection
```bash
oxd state info
# → Show state statistics

oxd state verify
# → Deep verification pass
```

### Phase 6.3: State Repair
```bash
oxd state rebuild
# → Rebuild state from scratch

oxd state clean
# → Remove orphaned entries
```

---

## Notes for Claude Code

- **Atomic writes are critical** - always write to temp file + rename
- **Handle missing state gracefully** - first run should just work
- **Quick checks before expensive ops** - mtime/size before checksum
- **State is single source of truth** - files are just serialization
- **Migration path** - version field enables future schema changes
- **Error handling** - corrupt state should regenerate, not crash
- **Testing** - test state persistence, change detection, corruption recovery
- **Performance** - measure before/after, aim for <50ms startup
