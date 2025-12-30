# Phase 9 Implementation: Document Lifecycle Management

## Overview

This phase implements advanced document lifecycle management:
- `oxd replace` - Replace document while preserving ID
- `oxd remove` - Safe removal to dustbin
- `oxd list --removed` - Track removed documents

**Implementation Order:**
1. Add "overwritten" and "removed" states
2. Implement remove command (simpler, good foundation)
3. Implement replace command (builds on remove)
4. Enhance list command with --removed flag

---

## Task 9.0: Add New States

### Step 1: Update DocState Enum

**File:** `design/src/state.rs`

Add two new states to the `DocState` enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocState {
    Draft,
    UnderReview,
    Revised,
    Accepted,
    Active,
    Final,
    Deferred,
    Rejected,
    Withdrawn,
    Superseded,
    Removed,      // NEW: Document removed to dustbin
    Overwritten,  // NEW: Document replaced via oxd replace
}
```

### Step 2: Update Display Implementation

Add display strings for new states:

```rust
impl fmt::Display for DocState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // ... existing matches ...
            DocState::Removed => write!(f, "removed"),
            DocState::Overwritten => write!(f, "overwritten"),
        }
    }
}
```

### Step 3: Update FromStr Implementation

Add parsing for new states:

```rust
impl FromStr for DocState {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Normalize: lowercase, replace underscores/spaces with hyphens
        let normalized = s.to_lowercase().replace(['_', ' '], "-");
        
        match normalized.as_str() {
            // ... existing matches ...
            "removed" => Ok(DocState::Removed),
            "overwritten" => Ok(DocState::Overwritten),
            _ => Err(anyhow!("Invalid state: {}", s)),
        }
    }
}
```

### Step 4: Update Directory Mapping

The `directory()` method needs special handling for removed/overwritten:

```rust
impl DocState {
    /// Get the directory name for this state
    pub fn directory(&self) -> &'static str {
        match self {
            DocState::Draft => "01-draft",
            DocState::UnderReview => "02-under-review",
            DocState::Revised => "03-revised",
            DocState::Accepted => "04-accepted",
            DocState::Active => "05-active",
            DocState::Final => "06-final",
            DocState::Deferred => "07-deferred",
            DocState::Rejected => "08-rejected",
            DocState::Withdrawn => "09-withdrawn",
            DocState::Superseded => "10-superseded",
            // These states don't have standard directories
            // They're in .dustbin with subdirectories
            DocState::Removed => ".dustbin",
            DocState::Overwritten => ".dustbin/overwritten",
        }
    }
    
    /// Check if this state should be in the dustbin
    pub fn is_in_dustbin(&self) -> bool {
        matches!(self, DocState::Removed | DocState::Overwritten)
    }
}
```

### Step 5: Update State List

Add to the `all_states()` method if it exists:

```rust
impl DocState {
    pub fn all_states() -> Vec<DocState> {
        vec![
            DocState::Draft,
            DocState::UnderReview,
            DocState::Revised,
            DocState::Accepted,
            DocState::Active,
            DocState::Final,
            DocState::Deferred,
            DocState::Rejected,
            DocState::Withdrawn,
            DocState::Superseded,
            DocState::Removed,
            DocState::Overwritten,
        ]
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            // ... existing descriptions ...
            DocState::Removed => "Document has been removed from active use",
            DocState::Overwritten => "Document was replaced via 'oxd replace'",
        }
    }
}
```

### Testing Step 9.0

```bash
# Run existing tests to ensure no breakage
cargo test state

# Test parsing new states
cargo test -- --nocapture test_parse_new_states
```

Expected: All existing tests pass, new states parse correctly.

---

## Task 9.1: Implement Remove Command

### Step 1: Create Remove Command Module

**File:** `design/src/commands/remove.rs`

```rust
use anyhow::{Context, Result, bail};
use colored::Colorize;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use crate::{git, index_sync, state::DocState};

pub fn execute(doc_id_or_path: &str) -> Result<()> {
    println!("{}", "Removing document...".cyan().bold());
    
    // Step 1: Load data sources
    let db_path = PathBuf::from("design/docs/.oxd-db.json");
    let mut db = crate::index_sync::Database::load(&db_path)
        .context("Failed to load database")?;
    
    // Step 2: Find document
    let doc = find_document(&db, doc_id_or_path)?;
    let doc_number = doc.number;
    let doc_title = doc.title.clone();
    let current_state = doc.state;
    
    println!("  Document: {} - {}", 
        format!("{:04}", doc_number).yellow(),
        doc_title.white());
    println!("  Current state: {}", format!("{}", current_state).cyan());
    
    // Check if already removed
    if current_state == DocState::Removed {
        println!("{}", "⚠ Document is already removed".yellow());
        println!("  Location: {}", doc.file_path.display());
        return Ok(());
    }
    
    // Step 3: Prepare dustbin directory
    let dustbin_base = PathBuf::from("design/docs/.dustbin");
    let state_subdir = current_state.directory();
    let dustbin_dir = if current_state.is_in_dustbin() {
        dustbin_base.clone()
    } else {
        dustbin_base.join(state_subdir)
    };
    
    std::fs::create_dir_all(&dustbin_dir)
        .context("Failed to create dustbin directory")?;
    println!("  ✓ Dustbin ready: {}", dustbin_dir.display().to_string().green());
    
    // Step 4: Generate unique filename with UUID
    let current_path = PathBuf::from(&doc.file_path);
    let filename = current_path.file_name()
        .context("Invalid file path")?
        .to_string_lossy();
    
    let uuid = Uuid::new_v4();
    let uuid_short = uuid.to_string().split('-').next().unwrap().to_string();
    
    // Extract base name and number
    let new_filename = if let Some(stem) = current_path.file_stem() {
        let stem_str = stem.to_string_lossy();
        format!("{}-{}.md", stem_str, uuid_short)
    } else {
        format!("{}-{}", filename.trim_end_matches(".md"), uuid_short)
    };
    
    let dustbin_path = dustbin_dir.join(&new_filename);
    println!("  ✓ Generated unique name: {}", new_filename.yellow());
    
    // Step 5: Move file using git
    if current_path.exists() {
        git::git_mv(&current_path, &dustbin_path)
            .context("Failed to move file with git")?;
        println!("  ✓ Moved to dustbin: {}", dustbin_path.display().to_string().green());
    } else {
        println!("  {} File not found on disk: {}", 
            "⚠".yellow(), 
            current_path.display());
    }
    
    // Step 6: Update database
    if let Some(doc_mut) = db.documents.iter_mut()
        .find(|d| d.number == doc_number) {
        doc_mut.state = DocState::Removed;
        doc_mut.file_path = dustbin_path.clone();
        doc_mut.updated = chrono::Local::now().format("%Y-%m-%d").to_string();
    }
    
    db.save(&db_path)
        .context("Failed to save database")?;
    println!("  ✓ Updated database");
    
    // Step 7: Update index
    let index_path = PathBuf::from("design/docs/00-index.md");
    crate::commands::update_index::sync_index(&index_path, &db_path)
        .context("Failed to update index")?;
    println!("  ✓ Updated index");
    
    println!();
    println!("{}", "Document removed successfully!".green().bold());
    println!("  Location: {}", dustbin_path.display().to_string().cyan());
    println!();
    println!("To view removed documents: {}", "oxd list --removed".yellow());
    
    Ok(())
}

fn find_document(db: &crate::index_sync::Database, id_or_path: &str) -> Result<crate::index_sync::DocumentRecord> {
    // Try parsing as number first
    if let Ok(num) = id_or_path.parse::<u32>() {
        if let Some(doc) = db.documents.iter().find(|d| d.number == num) {
            return Ok(doc.clone());
        }
        bail!("Document {} not found", num);
    }
    
    // Try as filename
    let search_path = Path::new(id_or_path);
    if let Some(doc) = db.documents.iter()
        .find(|d| d.file_path.ends_with(search_path)) {
        return Ok(doc.clone());
    }
    
    bail!("Document '{}' not found", id_or_path);
}
```

### Step 2: Add UUID Dependency

**File:** `design/Cargo.toml`

Add to dependencies:

```toml
[dependencies]
uuid = { version = "1.10", features = ["v4"] }
```

### Step 3: Register Remove Command

**File:** `design/src/commands/mod.rs`

Add module declaration:

```rust
pub mod remove;
```

**File:** `design/src/cli.rs`

Add to Commands enum:

```rust
#[derive(Debug, Parser)]
pub enum Commands {
    // ... existing commands ...
    
    /// Remove a document (moves to dustbin)
    Remove {
        /// Document number or filename
        doc: String,
    },
}
```

**File:** `design/src/main.rs`

Add to command dispatch:

```rust
Commands::Remove { doc } => {
    commands::remove::execute(&doc)?;
}
```

### Testing Step 9.1

```bash
# Test basic removal
oxd remove 2

# Test removing already removed document
oxd remove 2

# Test removing by filename
oxd remove 0002-test-document.md

# Verify file in dustbin
ls -la design/docs/.dustbin/01-draft/

# Verify database updated
cat design/docs/.oxd-db.json | jq '.documents[] | select(.number == 2)'

# Verify index updated
grep "0002" design/docs/00-index.md
```

---

## Task 9.2: Implement Replace Command

### Step 1: Create Replace Command Module

**File:** `design/src/commands/replace.rs`

```rust
use anyhow::{Context, Result, bail};
use colored::Colorize;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use crate::{doc::Document, git, index_sync, state::DocState};

pub fn execute(old_id_or_path: &str, new_file_path: &str) -> Result<()> {
    println!("{}", "Replacing document...".cyan().bold());
    
    // Step 1: Load data sources
    let db_path = PathBuf::from("design/docs/.oxd-db.json");
    let mut db = index_sync::Database::load(&db_path)
        .context("Failed to load database")?;
    
    // Step 2: Find old document
    let old_doc = find_document(&db, old_id_or_path)?;
    let old_number = old_doc.number;
    let old_title = old_doc.title.clone();
    let old_state = old_doc.state;
    
    println!();
    println!("{}", "Old Document:".cyan().bold());
    println!("  Number: {}", format!("{:04}", old_number).yellow());
    println!("  Title: {}", old_title.white());
    println!("  State: {}", format!("{}", old_state).cyan());
    
    // Step 3: Load and validate new document
    let new_path = PathBuf::from(new_file_path);
    if !new_path.exists() {
        bail!("New file not found: {}", new_file_path);
    }
    
    let new_content = std::fs::read_to_string(&new_path)
        .context("Failed to read new file")?;
    
    println!();
    println!("{}", "New Document:".cyan().bold());
    println!("  File: {}", new_path.display().to_string().white());
    
    // Parse new document (may or may not have frontmatter)
    let new_doc = Document::parse(&new_content)
        .context("Failed to parse new document")?;
    
    // Step 4: Merge metadata (preserve critical fields from old)
    let merged_metadata = merge_metadata(&old_doc, &new_doc, old_number)?;
    println!();
    println!("{}", "Merged Metadata:".cyan().bold());
    println!("  ✓ Preserved number: {}", format!("{:04}", merged_metadata.number).yellow());
    println!("  ✓ Preserved created: {}", merged_metadata.created.green());
    println!("  ✓ New title: {}", merged_metadata.title.white());
    println!("  ✓ New author: {}", merged_metadata.author.white());
    
    // Step 5: Move old document to dustbin as "overwritten"
    let old_path = PathBuf::from(&old_doc.file_path);
    let dustbin_dir = PathBuf::from("design/docs/.dustbin/overwritten");
    std::fs::create_dir_all(&dustbin_dir)
        .context("Failed to create dustbin directory")?;
    
    let uuid = Uuid::new_v4();
    let uuid_short = uuid.to_string().split('-').next().unwrap().to_string();
    
    let old_filename = old_path.file_name()
        .context("Invalid old file path")?
        .to_string_lossy();
    let new_dustbin_name = format!("{}-{}",
        old_filename.trim_end_matches(".md"),
        uuid_short
    );
    let dustbin_path = dustbin_dir.join(format!("{}.md", new_dustbin_name));
    
    println!();
    println!("{}", "Moving old version to dustbin...".cyan().bold());
    
    if old_path.exists() {
        git::git_mv(&old_path, &dustbin_path)
            .context("Failed to move old file to dustbin")?;
        println!("  ✓ Moved to: {}", dustbin_path.display().to_string().green());
    }
    
    // Update old document in database
    if let Some(old_record) = db.documents.iter_mut()
        .find(|d| d.number == old_number) {
        old_record.state = DocState::Overwritten;
        old_record.file_path = dustbin_path.clone();
        old_record.updated = chrono::Local::now().format("%Y-%m-%d").to_string();
    }
    
    // Step 6: Install new document
    println!();
    println!("{}", "Installing new version...".cyan().bold());
    
    // Generate new filename based on old document's number
    let new_filename = format!("{:04}-{}.md", 
        old_number,
        slugify(&merged_metadata.title)
    );
    
    // Place in draft directory initially
    let new_dir = PathBuf::from("design/docs/01-draft");
    std::fs::create_dir_all(&new_dir)?;
    let new_location = new_dir.join(&new_filename);
    
    // Create content with merged frontmatter
    let new_content_with_frontmatter = format!(
        "---\n{}\n---\n\n{}",
        merged_metadata.to_yaml()?,
        new_doc.body.trim()
    );
    
    std::fs::write(&new_location, new_content_with_frontmatter)
        .context("Failed to write new document")?;
    println!("  ✓ Created: {}", new_location.display().to_string().green());
    
    // Stage with git
    git::git_add(&new_location)
        .context("Failed to stage new file")?;
    println!("  ✓ Staged with git");
    
    // Add new document to database (replaces old entry)
    let new_record = index_sync::DocumentRecord {
        number: merged_metadata.number,
        title: merged_metadata.title.clone(),
        state: DocState::Draft,
        created: merged_metadata.created.clone(),
        updated: chrono::Local::now().format("%Y-%m-%d").to_string(),
        author: merged_metadata.author.clone(),
        file_path: new_location.clone(),
        supersedes: merged_metadata.supersedes,
        superseded_by: merged_metadata.superseded_by,
        tags: merged_metadata.tags.clone(),
    };
    
    // Remove old active record and add new one
    db.documents.retain(|d| d.number != old_number || d.state == DocState::Overwritten);
    db.documents.push(new_record);
    
    db.save(&db_path)
        .context("Failed to save database")?;
    println!("  ✓ Updated database");
    
    // Step 7: Update index
    let index_path = PathBuf::from("design/docs/00-index.md");
    crate::commands::update_index::sync_index(&index_path, &db_path)
        .context("Failed to update index")?;
    println!("  ✓ Updated index");
    
    println!();
    println!("{}", "Document replaced successfully!".green().bold());
    println!("  Old version: {}", dustbin_path.display().to_string().yellow());
    println!("  New version: {}", new_location.display().to_string().green());
    println!();
    println!("To view: {}", format!("oxd show {}", old_number).yellow());
    
    Ok(())
}

struct MergedMetadata {
    number: u32,
    title: String,
    created: String,
    updated: String,
    author: String,
    supersedes: Option<u32>,
    superseded_by: Option<u32>,
    tags: Vec<String>,
}

impl MergedMetadata {
    fn to_yaml(&self) -> Result<String> {
        let mut yaml = String::new();
        yaml.push_str(&format!("number: {}\n", self.number));
        yaml.push_str(&format!("title: \"{}\"\n", self.title));
        yaml.push_str(&format!("state: draft\n"));
        yaml.push_str(&format!("created: {}\n", self.created));
        yaml.push_str(&format!("updated: {}\n", self.updated));
        yaml.push_str(&format!("author: \"{}\"\n", self.author));
        
        if let Some(sup) = self.supersedes {
            yaml.push_str(&format!("supersedes: {}\n", sup));
        }
        if let Some(sup_by) = self.superseded_by {
            yaml.push_str(&format!("superseded-by: {}\n", sup_by));
        }
        if !self.tags.is_empty() {
            yaml.push_str(&format!("tags: [{}]\n", self.tags.join(", ")));
        }
        
        Ok(yaml)
    }
}

fn merge_metadata(
    old_doc: &index_sync::DocumentRecord,
    new_doc: &Document,
    preserve_number: u32,
) -> Result<MergedMetadata> {
    // Always preserve from old document
    let number = preserve_number;
    let created = old_doc.created.clone();
    
    // Use new document's values if present, otherwise fall back to old
    let title = if !new_doc.title.is_empty() {
        new_doc.title.clone()
    } else {
        old_doc.title.clone()
    };
    
    let author = if !new_doc.author.is_empty() {
        new_doc.author.clone()
    } else {
        old_doc.author.clone()
    };
    
    let updated = chrono::Local::now().format("%Y-%m-%d").to_string();
    
    // For optional fields, prefer new but keep old if new is missing
    let supersedes = new_doc.supersedes.or(old_doc.supersedes);
    let superseded_by = new_doc.superseded_by.or(old_doc.superseded_by);
    
    let tags = if !new_doc.tags.is_empty() {
        new_doc.tags.clone()
    } else {
        old_doc.tags.clone()
    };
    
    Ok(MergedMetadata {
        number,
        title,
        created,
        updated,
        author,
        supersedes,
        superseded_by,
        tags,
    })
}

fn find_document(db: &index_sync::Database, id_or_path: &str) -> Result<index_sync::DocumentRecord> {
    // Try parsing as number first
    if let Ok(num) = id_or_path.parse::<u32>() {
        if let Some(doc) = db.documents.iter().find(|d| d.number == num) {
            return Ok(doc.clone());
        }
        bail!("Document {} not found", num);
    }
    
    // Try as filename
    let search_path = Path::new(id_or_path);
    if let Some(doc) = db.documents.iter()
        .find(|d| d.file_path.ends_with(search_path)) {
        return Ok(doc.clone());
    }
    
    bail!("Document '{}' not found", id_or_path);
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "-")
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
```

### Step 2: Register Replace Command

**File:** `design/src/commands/mod.rs`

```rust
pub mod replace;
```

**File:** `design/src/cli.rs`

```rust
#[derive(Debug, Parser)]
pub enum Commands {
    // ... existing commands ...
    
    /// Replace a document while preserving its ID
    Replace {
        /// Document number or filename to replace
        old: String,
        
        /// New document file
        new: String,
    },
}
```

**File:** `design/src/main.rs`

```rust
Commands::Replace { old, new } => {
    commands::replace::execute(&old, &new)?;
}
```

### Step 3: Ensure Document Has Required Fields

**File:** `design/src/doc.rs`

Make sure Document struct has these fields:

```rust
pub struct Document {
    pub number: u32,
    pub title: String,
    pub state: DocState,
    pub created: String,
    pub updated: String,
    pub author: String,
    pub supersedes: Option<u32>,
    pub superseded_by: Option<u32>,
    pub tags: Vec<String>,
    pub body: String,
}
```

### Testing Step 9.2

```bash
# Create a test replacement document
echo "# New Feature Design

This is the new version.
" > /tmp/new-feature.md

# Test replace
oxd replace 2 /tmp/new-feature.md

# Verify old document in dustbin/overwritten
ls -la design/docs/.dustbin/overwritten/

# Verify new document in draft
ls -la design/docs/01-draft/ | grep 0002

# Check metadata preservation
oxd show 2 | grep -A 5 "Metadata"

# Verify database has both records
cat design/docs/.oxd-db.json | jq '.documents[] | select(.number == 2)'
```

---

## Task 9.3: Enhance List Command with --removed Flag

### Step 1: Add Flag to List Command

**File:** `design/src/cli.rs`

Update the List command:

```rust
/// List all documents
List {
    /// Filter by state
    #[arg(short, long)]
    state: Option<String>,
    
    /// Show detailed information
    #[arg(short, long)]
    verbose: bool,
    
    /// Show only removed documents
    #[arg(long)]
    removed: bool,
},
```

### Step 2: Implement Removed Listing

**File:** `design/src/commands/list.rs`

Update the execute function:

```rust
use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;
use crate::{index_sync, state::DocState, theme};

pub fn execute(state_filter: Option<String>, verbose: bool, removed: bool) -> Result<()> {
    let db_path = PathBuf::from("design/docs/.oxd-db.json");
    let db = index_sync::Database::load(&db_path)
        .context("Failed to load database")?;
    
    if removed {
        list_removed_documents(&db, verbose)?;
    } else if let Some(state_str) = state_filter {
        list_by_state(&db, &state_str, verbose)?;
    } else {
        list_all_documents(&db, verbose)?;
    }
    
    Ok(())
}

fn list_removed_documents(db: &index_sync::Database, verbose: bool) -> Result<()> {
    println!();
    println!("{}", "Removed Documents".cyan().bold());
    println!();
    
    // Filter for removed documents
    let mut removed_docs: Vec<_> = db.documents.iter()
        .filter(|d| d.state == DocState::Removed || d.state == DocState::Overwritten)
        .collect();
    
    if removed_docs.is_empty() {
        println!("  {}", "No removed documents found.".yellow());
        println!();
        return Ok(());
    }
    
    removed_docs.sort_by_key(|d| d.number);
    
    // Print header
    if verbose {
        println!("{:<8} {:<35} {:<12} {:<8} {}",
            "Number".cyan().bold(),
            "Title".cyan().bold(),
            "Removed".cyan().bold(),
            "Deleted".cyan().bold(),
            "Dustbin Location".cyan().bold()
        );
        println!("{}", "─".repeat(120).cyan());
    } else {
        println!("{:<8} {:<40} {:<12} {}",
            "Number".cyan().bold(),
            "Title".cyan().bold(),
            "Removed".cyan().bold(),
            "Deleted".cyan().bold()
        );
        println!("{}", "─".repeat(80).cyan());
    }
    
    // Check each document's deletion status
    let mut in_dustbin = 0;
    let mut deleted = 0;
    
    for doc in &removed_docs {
        let number_str = format!("{:04}", doc.number);
        let title_truncated = if doc.title.len() > (if verbose { 33 } else { 38 }) {
            format!("{}...", &doc.title[..(if verbose { 30 } else { 35 })])
        } else {
            doc.title.clone()
        };
        
        // Check if file exists in dustbin
        let file_exists = doc.file_path.exists();
        let deleted_status = if file_exists {
            in_dustbin += 1;
            "false".green()
        } else {
            deleted += 1;
            "true".red()
        };
        
        if verbose {
            let location = if file_exists {
                doc.file_path.display().to_string()
            } else {
                "(file not found)".to_string()
            };
            
            println!("{:<8} {:<35} {:<12} {:<8} {}",
                number_str.yellow(),
                title_truncated,
                doc.updated.white(),
                deleted_status,
                location.dimmed()
            );
        } else {
            println!("{:<8} {:<40} {:<12} {}",
                number_str.yellow(),
                title_truncated,
                doc.updated.white(),
                deleted_status
            );
        }
    }
    
    println!();
    println!("Total: {} removed ({} in dustbin, {} deleted)",
        removed_docs.len().to_string().yellow(),
        in_dustbin.to_string().green(),
        deleted.to_string().red()
    );
    println!();
    
    Ok(())
}

// Keep existing list_all_documents and list_by_state functions
```

### Testing Step 9.3

```bash
# List all removed documents
oxd list --removed

# List with verbose output
oxd list --removed --verbose

# Verify counts are accurate
oxd list --removed | grep "Total:"

# Delete a file from dustbin manually and verify deleted=true
rm design/docs/.dustbin/01-draft/0002-test-document-*.md
oxd list --removed
```

---

## Integration Testing

### Test Complete Workflow

```bash
# Create test document
echo "# Test Feature" > /tmp/test.md

# Add it
oxd add /tmp/test.md

# Get its number (should be next available)
oxd list | tail -n 5

# Replace it
echo "# Better Feature" > /tmp/better.md
oxd replace <number> /tmp/better.md

# Verify old version is overwritten
oxd list --removed --verbose | grep overwritten

# Verify new version is active
oxd show <number>

# Remove the new version
oxd remove <number>

# Verify it's removed
oxd list --removed

# Check both versions in dustbin
ls -la design/docs/.dustbin/overwritten/
ls -la design/docs/.dustbin/01-draft/
```

### Test Edge Cases

```bash
# Try to remove already removed document
oxd remove <removed-number>
# Expected: Warning message, no error

# Try to replace with invalid file
oxd replace 1 /nonexistent/file.md
# Expected: Error message

# Try to replace non-existent document
oxd replace 9999 /tmp/test.md
# Expected: Error message

# Replace same document twice
oxd replace 1 /tmp/v2.md
oxd replace 1 /tmp/v3.md
# Expected: Both v1 and v2 in dustbin with different UUIDs
```

---

## Validation Checklist

- [ ] New states (Removed, Overwritten) added to DocState enum
- [ ] States parse from strings correctly
- [ ] States have descriptions
- [ ] Remove command moves files to dustbin with UUID
- [ ] Remove command updates database correctly
- [ ] Remove command updates index
- [ ] Remove command handles already-removed docs
- [ ] Replace command preserves document number
- [ ] Replace command preserves created date
- [ ] Replace command merges metadata intelligently
- [ ] Replace command moves old to dustbin/overwritten
- [ ] Replace command installs new in draft
- [ ] Replace command handles multiple replacements
- [ ] List --removed shows only removed documents
- [ ] List --removed checks file existence
- [ ] List --removed shows deletion status
- [ ] List --removed verbose shows paths
- [ ] All git operations work correctly
- [ ] All database updates are consistent
- [ ] Index stays synchronized
- [ ] Error messages are clear and helpful
- [ ] Success messages are informative

---

## Common Issues and Solutions

### Issue: UUID crate not found
**Solution:** Make sure `uuid = { version = "1.10", features = ["v4"] }` is in Cargo.toml

### Issue: Dustbin directory creation fails
**Solution:** Check permissions, ensure parent directory exists

### Issue: Git operations fail
**Solution:** Ensure repository is initialized, files are tracked

### Issue: Database deserialization fails with new states
**Solution:** The new states should serialize/deserialize automatically with serde. If issues occur, check that #[serde(rename_all = "lowercase")] is present.

### Issue: Old document not found in database
**Solution:** Ensure document was added properly with `oxd add` or is in the database

---

End of Phase 9 Implementation Instructions
