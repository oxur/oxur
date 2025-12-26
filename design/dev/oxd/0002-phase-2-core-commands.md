# Phase 2: Core Command Implementation - Detailed Implementation Guide

## Overview
This phase implements the core commands that users will interact with daily. We'll complete the index generation, add header management, implement state transitions, and add location synchronization.

**Prerequisites:** Phase 1 must be complete (expanded states, git integration, YAML operations)

---

## Task 2.1: Implement Index Generation

### Purpose
Generate a markdown index file (00-index.md) with two sections:
1. A table of all documents sorted by number
2. Lists organized by state

### Implementation Steps

#### Step 1: Create Index Generation Module
File: `design/src/commands/index.rs`

```rust
//! Index generation command implementation

use anyhow::Result;
use design::index::DocumentIndex;
use design::doc::DocState;
use std::fs;
use std::path::PathBuf;

/// Generate the index markdown
pub fn generate_index(index: &DocumentIndex, format: &str) -> Result<()> {
    match format {
        "markdown" => generate_markdown_index(index),
        "json" => generate_json_index(index),
        _ => {
            eprintln!("Unknown format: {}. Using markdown.", format);
            generate_markdown_index(index)
        }
    }
}

fn generate_markdown_index(index: &DocumentIndex) -> Result<()> {
    let mut content = String::new();
    
    // Header
    content.push_str("# Design Document Index\n\n");
    content.push_str("This index is automatically generated. Do not edit manually.\n\n");
    
    // Table section
    content.push_str("## All Documents by Number\n\n");
    content.push_str("| Number | Title | State | Updated |\n");
    content.push_str("|--------|-------|-------|----------|\n");
    
    let mut docs = index.all();
    docs.sort_by_key(|d| d.metadata.number);
    
    for doc in &docs {
        content.push_str(&format!(
            "| {:04} | {} | {} | {} |\n",
            doc.metadata.number,
            doc.metadata.title,
            doc.metadata.state.as_str(),
            doc.metadata.updated
        ));
    }
    
    content.push_str("\n");
    
    // State sections
    content.push_str("## Documents by State\n");
    
    for state in DocState::all_states() {
        let state_docs = index.by_state(state);
        
        if !state_docs.is_empty() {
            content.push_str(&format!("\n### {}\n\n", state.as_str()));
            
            for doc in state_docs {
                let rel_path = doc.path.strip_prefix(index.docs_dir())
                    .unwrap_or(&doc.path);
                let path_str = rel_path.to_string_lossy();
                
                content.push_str(&format!(
                    "- [{:04} - {}]({})\n",
                    doc.metadata.number,
                    doc.metadata.title,
                    path_str
                ));
            }
        }
    }
    
    // Write to file
    let index_path = PathBuf::from(index.docs_dir()).join("00-index.md");
    fs::write(&index_path, content)?;
    
    println!("Generated index at: {}", index_path.display());
    Ok(())
}

fn generate_json_index(index: &DocumentIndex) -> Result<()> {
    use serde_json;
    
    #[derive(serde::Serialize)]
    struct JsonDoc {
        number: u32,
        title: String,
        author: String,
        state: String,
        created: String,
        updated: String,
        path: String,
    }
    
    let docs: Vec<JsonDoc> = index.all()
        .iter()
        .map(|doc| {
            let rel_path = doc.path.strip_prefix(index.docs_dir())
                .unwrap_or(&doc.path);
            
            JsonDoc {
                number: doc.metadata.number,
                title: doc.metadata.title.clone(),
                author: doc.metadata.author.clone(),
                state: doc.metadata.state.as_str().to_string(),
                created: doc.metadata.created.to_string(),
                updated: doc.metadata.updated.to_string(),
                path: rel_path.to_string_lossy().to_string(),
            }
        })
        .collect();
    
    let json = serde_json::to_string_pretty(&docs)?;
    
    let index_path = PathBuf::from(index.docs_dir()).join("00-index.json");
    fs::write(&index_path, json)?;
    
    println!("Generated JSON index at: {}", index_path.display());
    Ok(())
}
```

#### Step 2: Update Commands Module
File: `design/src/commands/mod.rs`

Add the index module:
```rust
pub mod index;
pub use index::generate_index;
```

#### Step 3: Wire Up in Main
File: `design/src/main.rs`

Update the Index command handler:
```rust
Commands::Index { format } => {
    generate_index(&index, &format)?;
}
```

### Testing Steps for 2.1
1. Run `oxd index` in a docs directory
2. Verify 00-index.md is created
3. Check table has all documents sorted by number
4. Verify state sections are correct
5. Test JSON format with `oxd index --format json`
6. Verify relative paths work correctly

---

## Task 2.2: Add Document Header Management

### Purpose
Add or complete YAML frontmatter headers for documents that are missing them or have incomplete headers.

### Implementation Steps

#### Step 1: Create Add Headers Command
File: `design/src/commands/add_headers.rs`

```rust
//! Add headers command implementation

use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::PathBuf;

/// Add or update YAML frontmatter headers
pub fn add_headers(doc_path: &str) -> Result<()> {
    let path = PathBuf::from(doc_path);
    
    // Validate file exists
    if !path.exists() {
        anyhow::bail!("File not found: {}", doc_path);
    }
    
    println!("Adding/updating headers for: {}\n", path.display());
    
    // Read current content
    let content = fs::read_to_string(&path)
        .context("Failed to read file")?;
    
    // Add missing headers
    let (new_content, added_fields) = design::doc::add_missing_headers(&path, &content)?;
    
    // Write updated content
    fs::write(&path, new_content)
        .context("Failed to write file")?;
    
    // Report what was done
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    
    if added_fields.is_empty() {
        println!("{}", format!("✓ All headers already present in {}", filename).green());
    } else {
        println!("{}", format!("✓ Added/updated headers in {}", filename).green());
        for field in added_fields {
            println!("  {}: {}", "Added".cyan(), field);
        }
    }
    
    println!();
    Ok(())
}
```

#### Step 2: Update CLI
File: `design/src/cli.rs`

Add new command:
```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands ...
    
    /// Add or update YAML frontmatter headers
    AddHeaders {
        /// Path to document
        path: String,
    },
}
```

#### Step 3: Update Commands Module
File: `design/src/commands/mod.rs`

```rust
pub mod add_headers;
pub use add_headers::add_headers;
```

#### Step 4: Wire Up in Main
File: `design/src/main.rs`

```rust
Commands::AddHeaders { path } => {
    add_headers(&path)?;
}
```

### Testing Steps for 2.2
1. Test with document missing frontmatter entirely
2. Test with partial frontmatter
3. Test with complete frontmatter (no changes)
4. Verify git metadata is extracted correctly
5. Test with numbered and unnumbered files

---

## Task 2.3: Implement State Transitions

### Purpose
Transition a document from one state to another, updating the YAML and moving the file to the appropriate directory.

### Implementation Steps

#### Step 1: Create Transition Command
File: `design/src/commands/transition.rs`

```rust
//! State transition command implementation

use anyhow::{Context, Result};
use colored::*;
use design::doc::{DesignDoc, DocState};
use design::index::DocumentIndex;
use std::fs;
use std::path::{Path, PathBuf};

/// Transition a document to a new state
pub fn transition_document(
    index: &DocumentIndex,
    doc_path: &str,
    new_state_str: &str,
) -> Result<()> {
    let path = PathBuf::from(doc_path);
    
    // Validate file exists
    if !path.exists() {
        anyhow::bail!("File not found: {}", doc_path);
    }
    
    // Check if document has headers, add them if missing
    let content = fs::read_to_string(&path)
        .context("Failed to read file")?;
    
    if !content.trim_start().starts_with("---\n") {
        println!("{}", "Document missing headers, adding them automatically...".yellow());
        let (new_content, _) = design::doc::add_missing_headers(&path, &content)?;
        fs::write(&path, new_content)
            .context("Failed to write headers")?;
    }
    
    // Parse document to get current state
    let content = fs::read_to_string(&path)?;
    let doc = DesignDoc::parse(&content, path.clone())
        .context("Failed to parse document")?;
    
    let current_state = doc.metadata.state;
    
    // Parse new state
    let new_state = DocState::from_str_flexible(new_state_str)
        .ok_or_else(|| {
            let valid_states = DocState::all_state_names().join(", ");
            anyhow::anyhow!(
                "Unsupported state '{}'. Valid states are: {}",
                new_state_str,
                valid_states
            )
        })?;
    
    // Check if already in that state
    if current_state == new_state {
        anyhow::bail!(
            "Document is already in state '{}'",
            current_state.as_str()
        );
    }
    
    // Update YAML frontmatter
    let updated_content = DesignDoc::update_state(&content, new_state)
        .context("Failed to update YAML")?;
    
    // Write updated content back to same file first
    fs::write(&path, updated_content)
        .context("Failed to write updated content")?;
    
    // Move to new state directory
    let filename = path.file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
    
    let new_dir = PathBuf::from(index.docs_dir()).join(new_state.directory());
    let new_path = new_dir.join(filename);
    
    design::git::git_mv(&path, &new_path)
        .context("Failed to move document")?;
    
    println!(
        "{} {} {} {} {}",
        "✓".green().bold(),
        "Moved".green(),
        filename.to_string_lossy(),
        "from".green(),
        current_state.as_str().cyan()
    );
    println!(
        "  {} {}",
        "to".green(),
        new_state.as_str().cyan()
    );
    
    Ok(())
}
```

#### Step 2: Update CLI
File: `design/src/cli.rs`

```rust
/// Transition document to a new state
Transition {
    /// Path to document
    path: String,
    
    /// New state (draft, under-review, revised, accepted, active, final, deferred, rejected, withdrawn, superseded)
    state: String,
},
```

#### Step 3: Update Commands Module
File: `design/src/commands/mod.rs`

```rust
pub mod transition;
pub use transition::transition_document;
```

#### Step 4: Wire Up in Main
File: `design/src/main.rs`

```rust
Commands::Transition { path, state } => {
    transition_document(&index, &path, &state)?;
}
```

### Testing Steps for 2.3
1. Transition document through various states
2. Verify YAML is updated correctly
3. Verify file is moved to correct directory
4. Test with various state name formats (hyphens, spaces, case)
5. Test error handling for invalid states
6. Verify git history is preserved

---

## Task 2.4: Implement Move-to-Match-Header

### Purpose
Move a document to the directory that matches its YAML state header, without updating the state itself.

### Implementation Steps

#### Step 1: Create Sync Location Command
File: `design/src/commands/sync_location.rs`

```rust
//! Sync location command implementation

use anyhow::{Context, Result};
use colored::*;
use design::doc::{DesignDoc, DocState};
use design::index::DocumentIndex;
use std::fs;
use std::path::{Path, PathBuf};

/// Move document to match its header state
pub fn sync_location(index: &DocumentIndex, doc_path: &str) -> Result<()> {
    let path = PathBuf::from(doc_path);
    
    // Validate file exists
    if !path.exists() {
        anyhow::bail!("File not found: {}", doc_path);
    }
    
    // Check if document has headers, add them if missing
    let content = fs::read_to_string(&path)
        .context("Failed to read file")?;
    
    if !content.trim_start().starts_with("---\n") {
        println!("{}", "Document missing headers, adding them automatically...".yellow());
        let (new_content, _) = design::doc::add_missing_headers(&path, &content)?;
        fs::write(&path, new_content)
            .context("Failed to write headers")?;
    }
    
    // Parse document to get header state
    let content = fs::read_to_string(&path)?;
    let doc = DesignDoc::parse(&content, path.clone())
        .context("Failed to parse document")?;
    
    let header_state = doc.metadata.state;
    
    // Determine target directory from state
    let target_dir = PathBuf::from(index.docs_dir()).join(header_state.directory());
    
    // Check current directory
    let current_dir = path.parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine current directory"))?;
    
    if current_dir == target_dir {
        println!(
            "{} {}",
            "✓".green().bold(),
            format!(
                "Document is already in the correct directory for state '{}'",
                header_state.as_str()
            ).green()
        );
        return Ok(());
    }
    
    // Move the file
    let filename = path.file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
    let target_path = target_dir.join(filename);
    
    design::git::git_mv(&path, &target_path)
        .context("Failed to move document")?;
    
    println!(
        "{} {} {} {} (state: {})",
        "✓".green().bold(),
        "Moved".green(),
        filename.to_string_lossy(),
        "to".green(),
        header_state.as_str().cyan()
    );
    println!("  From: {}", current_dir.display());
    println!("  To:   {}", target_dir.display());
    
    Ok(())
}
```

#### Step 2: Update CLI
File: `design/src/cli.rs`

```rust
/// Move document to directory matching its state header
SyncLocation {
    /// Path to document
    path: String,
},
```

#### Step 3: Update Commands Module
File: `design/src/commands/mod.rs`

```rust
pub mod sync_location;
pub use sync_location::sync_location;
```

#### Step 4: Wire Up in Main
File: `design/src/main.rs`

```rust
Commands::SyncLocation { path } => {
    sync_location(&index, &path)?;
}
```

### Testing Steps for 2.4
1. Create document with Draft state in wrong directory
2. Run sync-location and verify it moves to correct directory
3. Verify YAML state is not changed
4. Test when document is already in correct location
5. Test with documents in various mismatched states

---

## Additional Updates Needed

### Update lib.rs
File: `design/src/lib.rs`

Ensure new functions are exported:
```rust
pub mod doc;
pub mod index;
pub mod git;

pub use doc::{DesignDoc, DocMetadata, DocState};
pub use index::DocumentIndex;

// Re-export commonly used items
pub use anyhow::{Error, Result};
```

### Add serde_json Dependency
File: `design/Cargo.toml`

```toml
[dependencies]
serde_json = "1"
```

---

## Error Handling Enhancements

All commands should:
1. Provide clear error messages with context
2. Use colored output for errors (red)
3. Suggest valid options when appropriate
4. Handle missing files gracefully
5. Check git repository status where needed

Example error handling pattern:
```rust
use colored::*;

// When state is invalid
if let None = DocState::from_str_flexible(state_str) {
    eprintln!("{}", format!("Error: Invalid state '{}'", state_str).red().bold());
    eprintln!("Valid states are:");
    for state in DocState::all_state_names() {
        eprintln!("  - {}", state);
    }
    return Ok(());
}
```

---

## Verification Checklist

Before moving to Phase 3, verify:

- [ ] Index generation creates proper markdown structure
- [ ] Index table is sorted by document number
- [ ] State sections only show documents in that state
- [ ] JSON format works correctly
- [ ] Add-headers works on documents without frontmatter
- [ ] Add-headers completes partial frontmatter
- [ ] Add-headers reports what was added
- [ ] Transition updates YAML correctly
- [ ] Transition moves files to correct directory
- [ ] Transition preserves git history
- [ ] Transition handles invalid states gracefully
- [ ] Sync-location moves files to match header state
- [ ] Sync-location doesn't change YAML
- [ ] Sync-location detects when already in correct location
- [ ] All commands provide colored, helpful output
- [ ] Error messages are clear and actionable

---

## Integration Notes

These commands form the core user-facing functionality:

**Index Generation:**
- Used after bulk changes to regenerate index
- Will be automated in Phase 3 (update-index)
- Provides both human (markdown) and machine (json) formats

**Add Headers:**
- Prerequisite for other operations
- Safe to run multiple times
- Automatically called by other commands when needed

**Transition:**
- Primary way to move documents through workflow
- Combines YAML update + file movement
- Preserves git history

**Sync Location:**
- Fixes directory/header mismatches
- Useful after manual YAML edits
- Complements transition command

---

## Usage Examples

Once implemented, users can:

```bash
# Generate/regenerate index
oxd index

# Add headers to a new document
oxd add-headers docs/01-draft/0005-new-feature.md

# Transition document to review
oxd transition docs/01-draft/0005-new-feature.md "under review"

# Fix location after manual state edit
oxd sync-location docs/01-draft/0006-wrong-location.md

# Get JSON version of index
oxd index --format json
```

---

## Notes for Claude Code

- All commands should work with relative or absolute paths
- Use existing DocumentIndex for consistency
- Follow colored output patterns from existing commands
- Git operations should always preserve history
- Add comprehensive error handling
- Test edge cases (missing files, invalid states, etc.)
- Ensure commands are idempotent where possible
- Use the Phase 1 foundation (git module, YAML functions)
