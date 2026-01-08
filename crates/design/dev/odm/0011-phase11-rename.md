# Phase 11 Implementation: Rename Command & Alphabetical Help

## Overview

This phase implements two quality-of-life improvements:
1. **Rename Command**: Rename document files while preserving their ID numbers
2. **Alphabetical Help**: Sort commands alphabetically in help output

**Implementation Order:**
1. Implement alphabetical help (quick win)
2. Implement rename command (main feature)
3. Test both features

---

## Task 11.1: Alphabetize Help Output

This is a quick change that significantly improves UX.

### Step 1: Update CLI Command Enum

**File:** `crates/design/src/cli.rs`

Find the `Commands` enum and add the `next_display_order` attribute:

```rust
#[derive(Debug, Parser)]
#[command(next_display_order = None)]  // Add this line - enables alphabetical sorting
pub enum Commands {
    // All existing commands...
    // They will now appear alphabetically in help output
}
```

### Alternative: Manual Ordering

If `next_display_order = None` doesn't work with your version of clap, use manual ordering:

```rust
#[derive(Debug, Parser)]
pub enum Commands {
    /// Add a document to the project
    #[command(display_order = 1)]
    Add {
        // ...
    },

    /// Add or update document headers
    #[command(display_order = 2)]
    AddHeaders {
        // ...
    },

    /// Debug command for development
    #[command(display_order = 3)]
    Debug {
        // ...
    },

    // Continue for all commands in alphabetical order
    // ...
}
```

### Step 2: Test Alphabetical Output

```bash
# Build the project
cargo build

# Check help output
oxd -h

# Verify commands are in alphabetical order
oxd -h | grep -A 30 "Commands:"

# Should see:
# Commands:
#   add
#   add-headers
#   debug
#   index
#   info
#   list
#   new
#   remove
#   rename
#   replace
#   scan
#   search
#   show
#   sync-location
#   transition
#   update-index
#   validate
```

### Testing Step 11.1

```bash
# Verify alphabetical ordering
oxd -h | grep "Commands:" -A 50

# Verify all commands still work
oxd info states
oxd list
oxd scan

# Check long help too
oxd --help
```

---

## Task 11.2: Implement Rename Command

### Step 1: Create Rename Command Module

**File:** `crates/design/src/commands/rename.rs`

```rust
use anyhow::{Context, Result, bail};
use colored::Colorize;
use std::path::{Path, PathBuf};

pub fn execute(old_path: &str, new_path: &str) -> Result<()> {
    println!();
    println!("{}", "Renaming document...".cyan().bold());
    println!();
    
    // Step 1: Load configuration and database
    let config = crate::config::Config::load()?;
    let db_path = config.database_file.clone();
    let mut db = crate::index_sync::Database::load(&db_path)
        .context("Failed to load database")?;
    
    // Step 2: Parse and validate paths
    let (old_full, new_full) = parse_and_validate_paths(old_path, new_path, &config)?;
    
    println!("  From: {}", old_full.display().to_string().white());
    println!("  To:   {}", new_full.display().to_string().cyan());
    println!();
    
    // Step 3: Extract and verify numbers match
    let old_number = extract_number_from_path(&old_full)?;
    let new_number = extract_number_from_path(&new_full)?;
    
    if old_number != new_number {
        bail!(
            "{}

Cannot change document number during rename.
  Old number: {}
  New number: {}

To change document state/location, use: {}",
            "Number mismatch!".red().bold(),
            format!("{:04}", old_number).yellow(),
            format!("{:04}", new_number).yellow(),
            "oxd transition <doc> <state>".cyan()
        );
    }
    
    println!("  ✓ Number preserved: {}", format!("{:04}", old_number).yellow());
    println!();
    
    // Step 4: Find document in database
    let doc_record = db.documents.iter()
        .find(|d| d.number == old_number)
        .ok_or_else(|| anyhow::anyhow!(
            "Document {} not found in database. Run 'oxd scan' to sync.",
            old_number
        ))?
        .clone();
    
    // Step 5: Perform git mv
    crate::git::git_mv(&old_full, &new_full)
        .context("Failed to rename file with git")?;
    println!("  ✓ Renamed file with git mv");
    
    // Step 6: Update database
    if let Some(record) = db.documents.iter_mut().find(|d| d.number == old_number) {
        record.file_path = new_full.clone();
        record.updated = chrono::Local::now().format("%Y-%m-%d").to_string();
    }
    
    db.save(&db_path)
        .context("Failed to save database")?;
    println!("  ✓ Updated database");
    
    // Step 7: Update index
    let index_path = config.index_file.clone();
    crate::commands::update_index::sync_index(&index_path, &db_path)
        .context("Failed to update index")?;
    println!("  ✓ Updated index");
    
    println!();
    println!("{}", "Rename complete!".green().bold());
    println!("  View with: {}", format!("oxd show {}", old_number).yellow());
    println!();
    
    Ok(())
}

/// Parse paths and validate they're within docs directory
fn parse_and_validate_paths(
    old: &str,
    new: &str,
    config: &crate::config::Config,
) -> Result<(PathBuf, PathBuf)> {
    let docs_dir = &config.docs_directory;
    
    // Parse old path
    let old_path = resolve_path(old, docs_dir)?;
    
    // Validate old path exists
    if !old_path.exists() {
        bail!("Document not found: {}", old_path.display());
    }
    
    // Parse new path
    let new_path = resolve_path(new, docs_dir)?;
    
    // Validate new path doesn't exist
    if new_path.exists() {
        bail!("Destination already exists: {}", new_path.display());
    }
    
    // Validate both are .md files
    if old_path.extension().and_then(|e| e.to_str()) != Some("md") {
        bail!("Old path must be a markdown file (.md)");
    }
    if new_path.extension().and_then(|e| e.to_str()) != Some("md") {
        bail!("New path must be a markdown file (.md)");
    }
    
    // Validate both are within docs directory
    let old_canonical = old_path.canonicalize()
        .context("Failed to resolve old path")?;
    let docs_canonical = docs_dir.canonicalize()
        .context("Failed to resolve docs directory")?;
    
    if !old_canonical.starts_with(&docs_canonical) {
        bail!("Old path must be within the docs directory");
    }
    
    // For new path, check its parent directory is within docs
    if let Some(new_parent) = new_path.parent() {
        if new_parent.exists() {
            let new_parent_canonical = new_parent.canonicalize()
                .context("Failed to resolve new path parent")?;
            if !new_parent_canonical.starts_with(&docs_canonical) {
                bail!("New path must be within the docs directory");
            }
        }
    }
    
    Ok((old_path, new_path))
}

/// Resolve a path relative to docs directory or as absolute
fn resolve_path(path_str: &str, docs_dir: &Path) -> Result<PathBuf> {
    let path = PathBuf::from(path_str);
    
    // If it's already absolute or starts with docs dir, use as-is
    if path.is_absolute() {
        return Ok(path);
    }
    
    // Try relative to current directory first
    let relative_to_cwd = std::env::current_dir()?.join(&path);
    if relative_to_cwd.exists() {
        return Ok(relative_to_cwd);
    }
    
    // Try relative to docs directory
    let relative_to_docs = docs_dir.join(&path);
    if relative_to_docs.exists() {
        return Ok(relative_to_docs);
    }
    
    // For new paths (that don't exist), try to infer
    // If it looks like just a filename, put it in docs dir
    if path.components().count() == 1 {
        return Ok(docs_dir.join(&path));
    }
    
    // Otherwise use relative to current directory
    Ok(relative_to_cwd)
}

/// Extract document number from filename
fn extract_number_from_path(path: &Path) -> Result<u32> {
    let filename = path.file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
    
    // Look for 4-digit number at start
    if filename.len() < 4 {
        bail!("Filename too short to contain document number: {}", filename);
    }
    
    let number_str = &filename[0..4];
    
    number_str.parse::<u32>()
        .with_context(|| format!(
            "Could not extract document number from filename: {}. Expected format: 0001-title.md",
            filename
        ))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_number() {
        assert_eq!(extract_number_from_path(Path::new("0001-test.md")).unwrap(), 1);
        assert_eq!(extract_number_from_path(Path::new("0042-feature.md")).unwrap(), 42);
        assert_eq!(extract_number_from_path(Path::new("/path/to/0123-doc.md")).unwrap(), 123);
    }
    
    #[test]
    fn test_extract_number_invalid() {
        assert!(extract_number_from_path(Path::new("test.md")).is_err());
        assert!(extract_number_from_path(Path::new("abc-test.md")).is_err());
    }
}
```

### Step 2: Register Rename Command

**File:** `crates/design/src/commands/mod.rs`

Add module declaration:

```rust
pub mod rename;
```

**File:** `crates/design/src/cli.rs`

Add to Commands enum (in alphabetical order if manually ordering):

```rust
#[derive(Debug, Parser)]
#[command(next_display_order = None)]
pub enum Commands {
    // ... other commands ...
    
    /// Rename a document file (preserves number)
    Rename {
        /// Old file path
        old: String,
        
        /// New file path
        new: String,
    },
    
    // ... other commands ...
}
```

**File:** `crates/design/src/main.rs`

Add to command dispatch:

```rust
Commands::Rename { old, new } => {
    commands::rename::execute(&old, &new)?;
}
```

### Step 3: Ensure Required Dependencies

**File:** `crates/design/Cargo.toml`

Verify these dependencies exist:

```toml
[dependencies]
anyhow = "1.0"
colored = "2.0"
chrono = "0.4"
```

### Testing Step 11.2

```bash
# Test basic rename
cd crates/design
cargo build

# Create a test document first
echo "# Test Doc" > /tmp/test.md
oxd add /tmp/test.md

# Get the number (let's say it's 0003)
oxd list | tail -n 5

# Test valid rename (same number)
oxd rename 0003-test.md 0003-renamed-doc.md

# Verify file renamed
ls -la docs/01-draft/ | grep 0003

# Verify database updated
cat docs/.oxd-db.json | jq '.documents[] | select(.number == 3)'

# Verify index updated
grep "0003" docs/index.md

# Test invalid rename (different number) - should fail
oxd rename 0003-renamed-doc.md 0004-renamed-doc.md

# Expected error message with explanation
```

### Advanced Testing

```bash
# Test with full paths
oxd rename crates/design/docs/01-draft/0003-old.md \
           crates/design/docs/01-draft/0003-new.md

# Test with relative paths
cd crates/design/docs/01-draft
oxd rename 0003-old.md 0003-new.md

# Test error: file not found
oxd rename 9999-nonexistent.md 9999-new.md
# Expected: "Document not found"

# Test error: destination exists
oxd rename 0001-old.md 0002-test-document.md
# Expected: "Destination already exists"

# Test error: not .md file
oxd rename 0001-old.md 0001-new.txt
# Expected: "New path must be a markdown file"
```

### Edge Case Testing

```bash
# Test rename to same name (should be no-op or error)
oxd rename 0003-test.md 0003-test.md

# Test with spaces in filename (if supported)
oxd rename 0003-test.md "0003-test with spaces.md"

# Test path traversal attempt
oxd rename 0003-test.md ../../0003-escape.md
# Expected: "New path must be within the docs directory"
```

---

## Integration Testing

### Complete Workflow Test

```bash
# 1. Create new document
echo "# Feature Design" > /tmp/feature.md
oxd add /tmp/feature.md
# Note the assigned number (e.g., 0004)

# 2. Rename it
oxd rename 0004-feature.md 0004-awesome-feature.md

# 3. Verify all data sources updated
oxd show 4
# Should show new filename

oxd list
# Should show new filename

cat docs/index.md | grep 0004
# Should show new filename in links

cat docs/.oxd-db.json | jq '.documents[] | select(.number == 4)'
# Should show new file_path

# 4. Verify git history preserved
git log --follow -- "docs/01-draft/0004-awesome-feature.md"
# Should show history including rename

# 5. Try to change number (should fail)
oxd rename 0004-awesome-feature.md 0005-awesome-feature.md
# Should see error message

# 6. Verify nothing changed after failed rename
oxd show 4
# Should still show 0004-awesome-feature.md
```

---

## Error Message Testing

Verify each error message is clear and helpful:

### Number Mismatch Error

```bash
oxd rename 0001-old.md 0002-new.md
```

Expected output:
```
Number mismatch!

Cannot change document number during rename.
  Old number: 0001
  New number: 0002

To change document state/location, use: oxd transition <doc> <state>
```

### File Not Found Error

```bash
oxd rename 9999-nonexistent.md 9999-new.md
```

Expected output:
```
Error: Document not found: crates/design/docs/9999-nonexistent.md
```

### Destination Exists Error

```bash
oxd rename 0001-old.md 0002-existing.md
```

Expected output:
```
Error: Destination already exists: crates/design/docs/01-draft/0002-existing.md
```

---

## Path Handling Test Matrix

| Input Old | Input New | Expected Behavior |
|-----------|-----------|-------------------|
| `0001-a.md` | `0001-b.md` | ✓ Rename in current/docs dir |
| `docs/01-draft/0001-a.md` | `docs/01-draft/0001-b.md` | ✓ Rename with relative paths |
| `/full/path/0001-a.md` | `/full/path/0001-b.md` | ✓ Rename with absolute paths |
| `0001-a.md` | `0002-a.md` | ✗ Number mismatch error |
| `0001-a.md` | `/tmp/0001-a.md` | ✗ Outside docs dir error |
| `9999-a.md` | `9999-b.md` | ✗ File not found error |
| `0001-a.md` | `0002-existing.md` | ✗ Destination exists error |

---

## Validation Checklist

- [ ] Alphabetical help implemented
- [ ] `oxd -h` shows commands in alphabetical order
- [ ] All commands still work after help change
- [ ] Rename command created in commands/rename.rs
- [ ] Rename registered in mod.rs
- [ ] Rename added to CLI enum
- [ ] Rename dispatched in main.rs
- [ ] Number extraction works correctly
- [ ] Number mismatch detected and rejected
- [ ] Path validation works for all cases
- [ ] Absolute paths work
- [ ] Relative paths work
- [ ] Git mv performed successfully
- [ ] Database updated correctly
- [ ] Index updated correctly
- [ ] Error messages are clear and helpful
- [ ] Success messages are informative
- [ ] Tests pass (if unit tests added)
- [ ] Integration tests pass
- [ ] Edge cases handled properly
- [ ] Documentation updated

---

## Common Issues and Solutions

### Issue: clap doesn't recognize next_display_order
**Solution:** Update clap to version 4.x or use manual display_order attributes

### Issue: Path resolution fails
**Solution:** Ensure you're handling both absolute and relative paths, check current_dir() vs docs_dir resolution

### Issue: Git mv fails
**Solution:** Verify git is initialized, file is tracked, check git::git_mv implementation

### Issue: Number extraction fails on valid filenames
**Solution:** Check the filename format, ensure 4-digit padding (0001 not 1)

### Issue: Database doesn't update
**Solution:** Verify the document exists in database before renaming, check save() is called

---

## Documentation Updates

After implementation, update these files:

### README.md

Add to commands section:
```markdown
### Rename Document

Rename a document file while preserving its ID number:

```bash
oxd rename <old-path> <new-path>
```

Example:
```bash
oxd rename 0001-old-name.md 0001-better-name.md
```

**Note:** The document number must remain the same. To change a document's state or location, use `oxd transition`.
```

### Help Text

The rename command help should be clear:
```rust
/// Rename a document file (preserves number)
///
/// Renames a document file while enforcing that the document number
/// remains unchanged. This updates the file path in all data sources
/// and preserves git history with 'git mv'.
///
/// Example:
///   oxd rename 0001-old.md 0001-new.md
///
/// To change document state/location, use 'oxd transition' instead.
Rename {
    /// Old file path (relative or absolute)
    old: String,
    
    /// New file path (relative or absolute)
    new: String,
},
```

---

## Performance Considerations

The rename operation is lightweight:
- Path validation: O(1)
- Number extraction: O(1)
- Database lookup: O(n) where n = number of documents
- Database update: O(1)
- Index update: O(n) for regeneration

For projects with thousands of documents, index regeneration might take a few seconds. Consider adding a progress indicator if this becomes an issue.

---

## Future Enhancements

Ideas for future iterations:

1. **Interactive Mode**
   ```bash
   oxd rename --interactive 0001-old.md
   # Prompts for new name
   ```

2. **Bulk Rename**
   ```bash
   oxd rename --pattern "old-*" --replace "new-*"
   ```

3. **Dry Run**
   ```bash
   oxd rename --dry-run 0001-old.md 0001-new.md
   # Shows what would happen without doing it
   ```

4. **Smart Suggestions**
   ```bash
   oxd rename --suggest 0001-old.md
   # Suggests name based on frontmatter title
   ```

---

## Success Criteria

Phase 11 is complete when:

1. ✓ `oxd -h` shows commands alphabetically
2. ✓ `oxd rename <old> <new>` command works
3. ✓ Rename preserves document numbers
4. ✓ Rename rejects number changes with clear error
5. ✓ Rename updates database correctly
6. ✓ Rename updates index correctly
7. ✓ Rename uses git mv
8. ✓ All path formats work (absolute, relative)
9. ✓ Error messages are helpful
10. ✓ All tests pass

---

End of Phase 11 Implementation Instructions
