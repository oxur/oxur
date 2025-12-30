# Phase 4: Document Addition Workflow - Detailed Implementation Guide

## Overview
This phase implements the complete document addition workflow - a single command that takes a raw markdown file and fully integrates it into the repository with proper numbering, location, headers, git tracking, and index entries.

**Prerequisites:** Phases 1, 2, and 3 must be complete

---

## Task 4.1: Number Assignment

### Purpose
Automatically assign the next sequential number to documents that don't have a number prefix.

### Implementation Steps

#### Step 1: Add Number Detection Function
File: `design/src/doc.rs`

```rust
use regex::Regex;

/// Check if filename has a number prefix (e.g., 0001-, 0042-)
pub fn has_number_prefix(filename: &str) -> bool {
    let re = Regex::new(r"^\d{4}-").unwrap();
    re.is_match(filename)
}

/// Rename file to include number prefix
pub fn add_number_prefix(path: &Path, number: u32) -> Result<PathBuf, std::io::Error> {
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid filename"
        ))?;
    
    let new_filename = format!("{:04}-{}", number, filename);
    let new_path = path.with_file_name(new_filename);
    
    std::fs::rename(path, &new_path)?;
    
    Ok(new_path)
}
```

### Testing Steps for 4.1
1. Test has_number_prefix with various filenames
2. Test add_number_prefix renames correctly
3. Verify 4-digit padding works
4. Test with files in different directories

---

## Task 4.2: Directory Placement Logic

### Purpose
Determine if a file is in the project directory and in a state directory, moving it if necessary.

### Implementation Steps

#### Step 1: Add Directory Detection Functions
File: `design/src/doc.rs`

```rust
use std::path::{Path, PathBuf};

/// Check if a path is within the project directory
pub fn is_in_project_dir(file_path: &Path, project_dir: &Path) -> Result<bool, std::io::Error> {
    let abs_file = file_path.canonicalize()?;
    let abs_project = project_dir.canonicalize()?;
    
    Ok(abs_file.starts_with(abs_project))
}

/// Check if a path is in one of the state directories
pub fn is_in_state_dir(file_path: &Path) -> bool {
    if let Some(parent) = file_path.parent() {
        if let Some(dir_name) = parent.file_name().and_then(|n| n.to_str()) {
            return DocState::from_directory(dir_name).is_some();
        }
    }
    false
}

/// Get the state from the file's current directory
pub fn state_from_directory(file_path: &Path) -> Option<DocState> {
    file_path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .and_then(DocState::from_directory)
}

/// Move file to project directory
pub fn move_to_project(file_path: &Path, project_dir: &Path) -> Result<PathBuf, std::io::Error> {
    let filename = file_path.file_name()
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid filename"
        ))?;
    
    let new_path = project_dir.join(filename);
    std::fs::rename(file_path, &new_path)?;
    
    Ok(new_path)
}

/// Move file to a state directory
pub fn move_to_state_dir(
    file_path: &Path,
    state: DocState,
    project_dir: &Path,
) -> Result<PathBuf, std::io::Error> {
    let filename = file_path.file_name()
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid filename"
        ))?;
    
    let state_dir = project_dir.join(state.directory());
    std::fs::create_dir_all(&state_dir)?;
    
    let new_path = state_dir.join(filename);
    std::fs::rename(file_path, &new_path)?;
    
    Ok(new_path)
}
```

### Testing Steps for 4.2
1. Test is_in_project_dir with files inside and outside project
2. Test is_in_state_dir with files in various locations
3. Test state_from_directory extraction
4. Test move_to_project
5. Test move_to_state_dir creates directories

---

## Task 4.3: Header Processing Integration

### Purpose
Ensure document has complete, correct headers before integration.

### Implementation Steps

#### Step 1: Add Header Validation
File: `design/src/doc.rs`

```rust
/// Check if content has YAML frontmatter
pub fn has_frontmatter(content: &str) -> bool {
    content.trim_start().starts_with("---\n")
}

/// Check if frontmatter has placeholder values
pub fn has_placeholder_values(content: &str) -> bool {
    content.contains("number: NNNN") || 
    content.contains("author: Unknown") ||
    content.contains("title: \"\"")
}

/// Ensure document has complete, valid headers
pub fn ensure_valid_headers(path: &Path, content: &str) -> Result<String, DocError> {
    if !has_frontmatter(content) || has_placeholder_values(content) {
        let (new_content, _) = add_missing_headers(path, content)?;
        Ok(new_content)
    } else {
        Ok(content.to_string())
    }
}
```

### Testing Steps for 4.3
1. Test has_frontmatter detection
2. Test has_placeholder_values detection
3. Test ensure_valid_headers with various content
4. Verify placeholder values trigger header addition

---

## Task 4.4: State Synchronization

### Purpose
Ensure the document's YAML state field matches its directory location.

### Implementation Steps

#### Step 1: Add State Sync Function
File: `design/src/doc.rs`

```rust
/// Sync document state with its directory location
pub fn sync_state_with_directory(
    path: &Path,
    content: &str,
) -> Result<String, DocError> {
    // Get state from directory
    let dir_state = state_from_directory(path)
        .ok_or_else(|| DocError::InvalidFormat(
            "Document not in a state directory".to_string()
        ))?;
    
    // Parse document to get current state
    let doc = DesignDoc::parse(content, path.to_path_buf())?;
    
    // If states don't match, update the content
    if doc.metadata.state != dir_state {
        DesignDoc::update_state(content, dir_state)
    } else {
        Ok(content.to_string())
    }
}
```

### Testing Steps for 4.4
1. Test with matching state
2. Test with mismatched state
3. Verify YAML is updated correctly
4. Test with document not in state directory (should error)

---

## Task 4.5: Add Command Implementation

### Purpose
Orchestrate the complete document addition workflow.

### Implementation Steps

#### Step 1: Create Add Command
File: `design/src/commands/add.rs`

```rust
//! Add document command implementation

use anyhow::{Context, Result};
use colored::*;
use design::doc::*;
use design::index::DocumentIndex;
use std::fs;
use std::path::PathBuf;

/// Add a new document with full processing
pub fn add_document(index: &DocumentIndex, doc_path: &str) -> Result<()> {
    println!("{} {}\n", "Adding document:".bold(), doc_path);
    
    let mut path = PathBuf::from(doc_path);
    
    // Validate file exists
    if !path.exists() {
        anyhow::bail!("File not found: {}", doc_path);
    }
    
    let project_dir = PathBuf::from(index.docs_dir());
    
    // Step 1: Number Assignment
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?
        .to_string();
    
    if !has_number_prefix(&filename) {
        println!("{}", "Step 1: Assigning number...".cyan());
        
        let next_num = index.next_number();
        println!("  Assigning number: {:04}", next_num);
        
        path = add_number_prefix(&path, next_num)
            .context("Failed to add number prefix")?;
        
        let new_filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        println!("  {} Renamed to: {}\n", "✓".green(), new_filename);
    } else {
        println!("{} File already has number prefix\n", "✓".green());
    }
    
    // Step 2: Move to Project Directory
    if !is_in_project_dir(&path, &project_dir)
        .context("Failed to check project directory")? 
    {
        println!("{}", "Step 2: Moving to project directory...".cyan());
        
        path = move_to_project(&path, &project_dir)
            .context("Failed to move to project")?;
        
        println!("  {} Moved to: {}\n", "✓".green(), path.display());
    } else {
        println!("{} File already in project directory\n", "✓".green());
    }
    
    // Step 3: State Directory Placement
    if !is_in_state_dir(&path) {
        println!("{}", "Step 3: Moving to draft directory...".cyan());
        
        path = move_to_state_dir(&path, DocState::Draft, &project_dir)
            .context("Failed to move to draft directory")?;
        
        println!("  {} Moved to: {}\n", "✓".green(), path.display());
    } else {
        println!("{} File already in state directory\n", "✓".green());
    }
    
    // Step 4: Add/Update YAML Headers
    println!("{}", "Step 4: Processing headers...".cyan());
    
    let content = fs::read_to_string(&path)
        .context("Failed to read file")?;
    
    let updated_content = ensure_valid_headers(&path, &content)
        .context("Failed to ensure valid headers")?;
    
    if content != updated_content {
        fs::write(&path, &updated_content)
            .context("Failed to write headers")?;
        println!("  {} Added/updated headers\n", "✓".green());
    } else {
        println!("  {} Headers already complete\n", "✓".green());
    }
    
    // Step 5: Sync State with Directory
    println!("{}", "Step 5: Syncing state with directory...".cyan());
    
    let content = fs::read_to_string(&path)
        .context("Failed to read file")?;
    
    let synced_content = sync_state_with_directory(&path, &content)
        .context("Failed to sync state")?;
    
    if content != synced_content {
        fs::write(&path, &synced_content)
            .context("Failed to write synced content")?;
        println!("  {} Updated state to match directory\n", "✓".green());
    } else {
        println!("  {} State already matches directory\n", "✓".green());
    }
    
    // Step 6: Git Add
    println!("{}", "Step 6: Adding to git...".cyan());
    
    design::git::git_add(&path)
        .context("Failed to git add")?;
    
    println!("  {} Git staged: {}\n", "✓".green(), path.display());
    
    // Step 7: Update Index
    println!("{}", "Step 7: Updating index...".cyan());
    
    // Reload index to pick up the new file
    let updated_index = DocumentIndex::new(index.docs_dir())
        .context("Failed to reload index")?;
    
    // Run update-index command
    use crate::commands::update_index;
    update_index(&updated_index)
        .context("Failed to update index")?;
    
    // Final summary
    let final_filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    
    println!("\n{} Successfully added document: {}", 
        "✓".green().bold(), 
        final_filename.bold()
    );
    
    Ok(())
}
```

#### Step 2: Update Commands Module
File: `design/src/commands/mod.rs`

```rust
pub mod add;
pub use add::add_document;
```

#### Step 3: Update CLI
File: `design/src/cli.rs`

```rust
/// Add a new document with full processing
Add {
    /// Path to document file
    path: String,
},
```

#### Step 4: Wire Up in Main
File: `design/src/main.rs`

```rust
Commands::Add { path } => {
    add_document(&index, &path)?;
}
```

### Testing Steps for 4.5
1. Test with completely raw file (no number, wrong location, no headers)
2. Test with partially processed file (has number but no headers)
3. Test with file already mostly correct
4. Verify each step executes correctly
5. Verify git staging works
6. Verify index is updated
7. Test error handling at each step

---

## Task 4.6: Progress Reporting Enhancements

### Purpose
Provide clear, informative output at each step of the workflow.

### Implementation Steps

#### Step 1: Add Step Formatting Helper
File: `design/src/commands/add.rs`

Add at the top of the file:

```rust
/// Format a step header
fn step_header(num: u32, title: &str) -> String {
    format!("{} {}", format!("Step {}:", num).cyan().bold(), title.cyan())
}

/// Format a success message
fn success(msg: &str) -> String {
    format!("  {} {}", "✓".green(), msg)
}

/// Format a skip message
fn skip(msg: &str) -> String {
    format!("  {} {}", "→".yellow(), msg)
}
```

#### Step 2: Enhance Output Messages
Update the add_document function to use these helpers:

```rust
// Example for Step 1:
if !has_number_prefix(&filename) {
    println!("{}", step_header(1, "Assigning number"));
    println!("  Assigning number: {:04}", next_num);
    
    path = add_number_prefix(&path, next_num)
        .context("Failed to add number prefix")?;
    
    let new_filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    println!("{}\n", success(&format!("Renamed to: {}", new_filename)));
} else {
    println!("{}\n", skip("File already has number prefix"));
}
```

### Testing Steps for 4.6
1. Verify output is clear and readable
2. Check colors render correctly
3. Verify step numbering is consistent
4. Test with various terminal widths

---

## Task 4.7: Dry Run Support

### Purpose
Allow users to preview what would happen without making changes.

### Implementation Steps

#### Step 1: Add Dry Run Flag
File: `design/src/cli.rs`

```rust
/// Add a new document with full processing
Add {
    /// Path to document file
    path: String,
    
    /// Show what would be done without making changes
    #[arg(long)]
    dry_run: bool,
},
```

#### Step 2: Update Add Command
File: `design/src/commands/add.rs`

```rust
pub fn add_document(index: &DocumentIndex, doc_path: &str, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("{}\n", "DRY RUN MODE - No changes will be made".yellow().bold());
    }
    
    println!("{} {}\n", "Adding document:".bold(), doc_path);
    
    // ... existing code, but wrap all modifications in:
    if !dry_run {
        // actual modification
    } else {
        // just report what would happen
    }
}
```

#### Step 3: Update Main
File: `design/src/main.rs`

```rust
Commands::Add { path, dry_run } => {
    add_document(&index, &path, dry_run)?;
}
```

### Testing Steps for 4.7
1. Run with --dry-run and verify no changes made
2. Verify output explains what would happen
3. Run without --dry-run and verify changes applied
4. Test with various file states

---

## Error Recovery Strategies

### Partial Completion
If the add command fails partway through:

**Number Assignment:**
- File may have been renamed
- User can re-run add command
- Already-numbered files are skipped

**Directory Movement:**
- File may be in intermediate location
- Re-running add will continue from current state
- Each step is idempotent

**Git Operations:**
- If git add fails, user can run manually
- Or re-run add command

**Index Update:**
- Can be run separately with update-index
- Is last step so failures here are less critical

### Error Messages
Provide specific, actionable errors:
```rust
.context("Failed to add number prefix")?
```

Becomes:
```rust
.with_context(|| format!(
    "Failed to add number prefix to {}. \
     Check file permissions and ensure filename is valid.",
    path.display()
))?
```

---

## Integration with Existing Commands

The add command builds on:

**Phase 1 Foundation:**
- Uses git operations (git_add)
- Uses state system
- Uses YAML operations

**Phase 2 Commands:**
- Internally uses add_missing_headers logic
- Implicitly transitions to Draft state
- Updates index via update_index

**Phase 3 Index Sync:**
- Calls update_index as final step
- Ensures document appears in index immediately

---

## Verification Checklist

Before moving to Phase 5, verify:

- [ ] Number assignment works correctly
- [ ] Files moved to project directory when needed
- [ ] Files moved to draft directory when needed
- [ ] Headers added/updated correctly
- [ ] State synced with directory
- [ ] Git add stages file correctly
- [ ] Index updated automatically
- [ ] Each step is idempotent
- [ ] Steps skip when already complete
- [ ] Progress output is clear and helpful
- [ ] Dry run mode works correctly
- [ ] Error messages are actionable
- [ ] Can recover from partial completion
- [ ] Works with various input file states

---

## Usage Examples

Once implemented:

```bash
# Add a completely raw file
oxd add ~/Downloads/new-feature.md

# Add with dry run to preview
oxd add ~/Downloads/new-feature.md --dry-run

# Add file already in project
oxd add docs/some-doc.md

# Add file that already has number
oxd add 0005-existing.md
```

**Typical output:**
```
Adding document: ~/Downloads/new-feature.md

Step 1: Assigning number
  Assigning number: 0012
  ✓ Renamed to: 0012-new-feature.md

Step 2: Moving to project directory
  ✓ Moved to: docs/0012-new-feature.md

Step 3: Moving to draft directory
  ✓ Moved to: docs/01-draft/0012-new-feature.md

Step 4: Processing headers
  ✓ Added/updated headers

Step 5: Syncing state with directory
  ✓ State already matches directory

Step 6: Adding to git
  ✓ Git staged: docs/01-draft/0012-new-feature.md

Step 7: Updating index
  Content Changes:
    ✓ Add to table: 0012 - New Feature
    ✓ Add to Draft: 0012 - New Feature

✓ Successfully added document: 0012-new-feature.md
```

---

## Advanced Features (Optional)

### Batch Add
Support adding multiple files at once:
```bash
oxd add *.md
```

### Custom State
Allow specifying initial state:
```bash
oxd add --state "under review" doc.md
```

### Template Selection
Use different templates:
```bash
oxd add --template rfc doc.md
```

These can be added later if needed.

---

## Notes for Claude Code

- Make each step clearly visible in output
- Use colored output consistently
- Handle edge cases gracefully
- Provide helpful error messages
- Make command idempotent
- Test with files in various states
- Ensure dry-run mode is safe
- Consider adding verbose flag for debugging
- Document the workflow clearly
- Add examples to help text
