# Implementation Plan: Refactor `oxd show` Command with Themed Table Output

## Overview
Refactor the `oxd show <doc-id>` command to remove unused content display, add path information, and use the themed ANSI-colored table format consistent with the `oxd list` command.

## Current State Analysis

### File: `crates/design/src/commands/show.rs`

**Current behavior:**
- Displays document metadata in plain text format
- Has unused "Content:" section (lines 28-32) with decorative separators
- Missing relative path information
- Uses simple `println!` statements

**Current output format:**
```
Document 0006

Title: oxur-ast Phase 2: Generator (Rust AST → S-expression)
Author: hand let
State: Active
Created: 2025-12-27
Updated: 2025-12-27

Content:
────────────────────────────────────────────────────────────────────────────────

────────────────────────────────────────────────────────────────────────────────
```

## Target State

### Desired output format:
A themed table with:
- **Title**: "DOCUMENT" | "INFORMATION" | "" (3-column title row)
- **Headers**: "Field" | "Content" | "" (with consistent spacing)
- **Data rows**: Each metadata field as a row
- **No footer row** (unlike list command)

### Field Order (in data rows):
1. Number (0-padded to 4 digits)
2. Title
3. Author
4. State
5. Created
6. Updated
7. Path (relative from user's working directory)
8. Supersedes (if not null)
9. Superseded By (if not null)

## Implementation Steps

### Step 1: Add Required Dependencies and Imports

**File: `crates/design/src/commands/show.rs`**

Add these imports at the top (after the existing imports):

```rust
use design::theme;
use oxur_table::TableStyleConfig;
use tabled::{builder::Builder, Table, Tabled};
use std::env;
```

### Step 2: Create the DocumentInfoRow Struct

**File: `crates/design/src/commands/show.rs`**

Add this struct after the existing imports, before the `show_document` function:

```rust
/// Minimal struct for type parameter when building tables with Builder
/// (Not used for actual data - Builder uses plain strings)
#[derive(Tabled)]
struct DocumentInfoRow {
    field: String,
    content: String,
}
```

**Note**: This struct is required for the type parameter in `config.apply_to_table::<DocumentInfoRow>()` but isn't used for actual data since we use Builder with plain strings.

### Step 3: Create Helper Function for Relative Path

**File: `crates/design/src/commands/show.rs`**

Add this helper function before `show_document`:

```rust
/// Get the relative path from the current working directory to the document
fn get_relative_path(doc_path: &std::path::Path) -> String {
    let current_dir = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    
    match doc_path.strip_prefix(&current_dir) {
        Ok(rel_path) => rel_path.to_string_lossy().to_string(),
        Err(_) => doc_path.to_string_lossy().to_string(),
    }
}
```

**Rationale**: Users want to see paths relative to where they issued the command, not absolute paths or paths relative to docs_dir.

### Step 4: Refactor show_document Function

**File: `crates/design/src/commands/show.rs`**

**Current signature:**
```rust
pub fn show_document(index: &DocumentIndex, number: u32, metadata_only: bool) -> Result<()>
```

**Keep the signature unchanged** (the `metadata_only` parameter is now effectively unused but we'll leave it for backward compatibility).

**Replace the entire function body** with the following implementation:

```rust
pub fn show_document(index: &DocumentIndex, number: u32, _metadata_only: bool) -> Result<()> {
    let doc = match index.get(number) {
        Some(d) => d,
        None => bail!("Document {:04} not found", number),
    };

    // Build table with Builder (plain text only)
    let mut builder = Builder::default();

    // Row 0: Title - PLAIN TEXT (formatting applied later)
    builder.push_record(["DOCUMENT", "INFORMATION", ""]);

    // Row 1: Header - PLAIN TEXT
    builder.push_record(["Field", "Content", ""]);

    // Data rows - PLAIN TEXT (no ANSI codes, formatting applied later)
    
    // Number (0-padded to 4 digits)
    builder.push_record([
        "Number",
        &format!("{:04}", doc.metadata.number),
        "",
    ]);

    // Title
    builder.push_record([
        "Title",
        &doc.metadata.title,
        "",
    ]);

    // Author
    builder.push_record([
        "Author",
        &doc.metadata.author,
        "",
    ]);

    // State
    builder.push_record([
        "State",
        doc.metadata.state.as_str(),
        "",
    ]);

    // Created
    builder.push_record([
        "Created",
        &doc.metadata.created.to_string(),
        "",
    ]);

    // Updated
    builder.push_record([
        "Updated",
        &doc.metadata.updated.to_string(),
        "",
    ]);

    // Path (relative to current working directory)
    let rel_path = get_relative_path(&doc.path);
    builder.push_record([
        "Path",
        &rel_path,
        "",
    ]);

    // Supersedes (only if not null)
    if let Some(supersedes) = doc.metadata.supersedes {
        builder.push_record([
            "Supersedes",
            &format!("{:04}", supersedes),
            "",
        ]);
    }

    // Superseded By (only if not null)
    if let Some(superseded_by) = doc.metadata.superseded_by {
        builder.push_record([
            "Superseded By",
            &format!("{:04}", superseded_by),
            "",
        ]);
    }

    // Build the table structure (width calculation happens here with plain text)
    let mut table = builder.build();

    // Apply the theme (background colors, padding, justification)
    let config = TableStyleConfig::default();
    config.apply_to_table::<DocumentInfoRow>(&mut table);

    // Print with blank lines for spacing
    println!();
    println!("{}", table);
    println!();

    Ok(())
}
```

### Step 5: Update Tests

**File: `crates/design/src/commands/show.rs`**

The existing tests should still pass with minimal changes:

1. **Tests that check for specific output strings**: These will need updates since we're changing from plain text to table format
2. **Tests that check return values**: These should continue to work without modification

**Update these specific tests:**

#### Test: `test_show_existing_document`
```rust
#[test]
fn test_show_existing_document() {
    let index = create_test_index_with_docs();
    
    // Should not panic and should return Ok
    let result = show_document(&index, 1, false);
    assert!(result.is_ok());
}
```
**Action**: No changes needed - this just checks for Ok result.

#### Test: `test_show_nonexistent_document`
```rust
#[test]
fn test_show_nonexistent_document() {
    let index = create_test_index_with_docs();
    
    let result = show_document(&index, 9999, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}
```
**Action**: No changes needed - error handling is unchanged.

#### Test: `test_show_metadata_only`
```rust
#[test]
fn test_show_metadata_only() {
    let index = create_test_index_with_docs();
    
    // The metadata_only parameter is now unused but the function should still work
    let result = show_document(&index, 1, true);
    assert!(result.is_ok());
}
```
**Action**: Update comment to reflect that `metadata_only` is now unused.

**All other tests** should continue to work without modification since they only check return values, not output format.

### Step 6: Verify Integration

After implementation, manually test:

1. **Basic show command:**
   ```bash
   ./bin/oxd show 1
   ```
   Expected: Themed table with all fields, properly formatted

2. **Show with supersedes:**
   ```bash
   ./bin/oxd show 2
   ```
   (Assuming doc 2 has supersedes field set)
   Expected: Supersedes row appears

3. **Show with superseded-by:**
   ```bash
   ./bin/oxd show 3
   ```
   (Assuming doc 3 has superseded-by field set)
   Expected: Superseded By row appears

4. **Show nonexistent document:**
   ```bash
   ./bin/oxd show 9999
   ```
   Expected: Error message "Document 9999 not found"

5. **Path relative to different working directories:**
   ```bash
   cd /tmp
   /path/to/oxd show 1
   ```
   Expected: Path shown relative to /tmp

## Implementation Checklist

- [ ] Step 1: Add required imports
- [ ] Step 2: Create `DocumentInfoRow` struct
- [ ] Step 3: Create `get_relative_path` helper function
- [ ] Step 4: Refactor `show_document` function body
- [ ] Step 5: Update test comments
- [ ] Step 6: Manual testing verification
- [ ] Run full test suite: `cargo test`
- [ ] Run clippy: `cargo clippy`
- [ ] Build release: `cargo build --release`

## Key Design Decisions

### Why use Builder instead of struct-based Tabled?
Following the pattern from `list.rs`, we use `Builder` because:
1. Variable number of rows (supersedes/superseded-by are conditional)
2. Need plain text during build for accurate width calculation
3. Theme application happens after table structure is built

### Why keep the metadata_only parameter?
- Backward compatibility with existing code
- CLI signature remains unchanged
- Marked as unused with `_metadata_only` prefix
- Could be removed in future major version

### Why calculate relative path?
- Users expect to see paths relative to where they invoked the command
- More intuitive than absolute paths or docs_dir-relative paths
- Matches common CLI tool behavior (git, ls, etc.)

### Why no footer row?
- `show` displays single document, not a collection
- No "total count" to display
- Cleaner output for single-item display

## Success Criteria

✅ Output is a properly themed ANSI-colored table
✅ All metadata fields are displayed in correct order
✅ Number is 0-padded to 4 digits
✅ Path shows location relative to user's working directory
✅ Supersedes/Superseded By only show when not null
✅ Content display section is removed
✅ Existing tests pass (or have updated comments)
✅ New table matches visual style of `list` command

## References

- **Source file**: `crates/design/src/commands/show.rs`
- **Reference implementation**: `crates/design/src/commands/list.rs` (lines 32-103 for table building)
- **Table config**: `oxur_table::TableStyleConfig`
- **CLI definition**: `crates/design/src/cli.rs` (Show command, line 85)
