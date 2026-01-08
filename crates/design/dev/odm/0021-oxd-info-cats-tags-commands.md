# Implementation Plan: Add `oxd info tags` and `oxd info components` Commands

## Overview

Add two new subcommands to `oxd info` for listing tags and components with occurrence counts.

**New Commands:**
- `oxd info tags` - List all tags with how many times each is used
- `oxd info components` - List all components with how many times each is used

**Output Format:**
Both commands display results in CLI tables similar to `oxd list --dev`, with:
- Table title
- Column headers ("Tag"/"Component" and "Occurrences")
- Data rows sorted by occurrence count (descending)
- Footer showing total count

---

## User Requirements

**Command 1: `oxd info tags`**
```bash
$ oxd info tags

TAGS

Tag               Occurrences
Phase-0           5
Research          3
Protocol          2
Compiler          1

Total Tags: 4
```

**Command 2: `oxd info components`**
```bash
$ oxd info components

COMPONENTS

Component         Occurrences
REPL              8
AST               5
Compiler          3
Tooling           2

Total Components: 4
```

---

## Critical Files to Modify

1. **`/Users/oubiwann/lab/oxur/oxur/crates/design/src/commands/info.rs`**
   - Lines 5-14: `InfoCommand` enum - add `Tags` and `Components` variants
   - Lines 16-27: `from_str()` function - add parsing for "tags" and "components"
   - Lines 29-42: `execute()` function - add match arms for new commands
   - End of file: Add `show_tags()` and `show_components()` functions
   - Test module: Add tests for new commands

---

## Implementation Steps

### Step 1: Update InfoCommand Enum

**File:** `src/commands/info.rs`
**Location:** Lines 5-14

Add two new variants to the enum:

```rust
#[derive(Debug, Clone)]
pub enum InfoCommand {
    Overview,
    States,
    Fields,
    Config,
    Stats,
    Dirs,
    Tags,        // NEW
    Components,  // NEW
}
```

### Step 2: Update from_str() Parser

**File:** `src/commands/info.rs`
**Location:** Lines 16-27

Add parsing for the new commands:

```rust
impl InfoCommand {
    pub fn from_str(s: Option<&str>) -> Self {
        match s {
            Some("states") => InfoCommand::States,
            Some("fields") | Some("metadata") => InfoCommand::Fields,
            Some("config") => InfoCommand::Config,
            Some("stats") => InfoCommand::Stats,
            Some("dirs") | Some("structure") => InfoCommand::Dirs,
            Some("tags") => InfoCommand::Tags,              // NEW
            Some("components") => InfoCommand::Components,  // NEW
            _ => InfoCommand::Overview,
        }
    }
}
```

### Step 3: Update execute() Function

**File:** `src/commands/info.rs`
**Location:** Lines 29-42

Add match arms to dispatch the new commands:

```rust
pub fn execute(subcommand: Option<String>, state_mgr: &StateManager) -> Result<()> {
    let cmd = InfoCommand::from_str(subcommand.as_deref());

    match cmd {
        InfoCommand::Overview => show_overview(state_mgr)?,
        InfoCommand::States => show_states()?,
        InfoCommand::Fields => show_fields()?,
        InfoCommand::Config => show_config(state_mgr)?,
        InfoCommand::Stats => show_stats(state_mgr)?,
        InfoCommand::Dirs => show_dirs(state_mgr)?,
        InfoCommand::Tags => show_tags(state_mgr)?,          // NEW
        InfoCommand::Components => show_components(state_mgr)?,  // NEW
    }

    Ok(())
}
```

### Step 4: Add show_tags() Function

**File:** `src/commands/info.rs`
**Location:** After `show_dirs()` function (after line 490)

Add the complete implementation:

```rust
fn show_tags(state_mgr: &StateManager) -> Result<()> {
    use std::collections::HashMap;
    use tabled::builder::Builder;
    use tabled::{Table, Tabled};

    // Marker struct for table type parameter
    #[derive(Tabled)]
    struct TagRow {
        tag: String,
        occurrences: String,
    }

    // Collect all tags with occurrence counts
    let all_docs = state_mgr.state().all();
    let mut tag_counts: HashMap<String, usize> = HashMap::new();

    for doc in &all_docs {
        for tag in &doc.metadata.tags {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }

    // Convert to sorted vector (by occurrence count, descending)
    let mut tag_vec: Vec<_> = tag_counts.into_iter().collect();
    tag_vec.sort_by(|a, b| b.1.cmp(&a.1));

    // Build table
    let mut builder = Builder::default();

    // Row 0: Title
    builder.push_record(["TAGS", ""]);

    // Row 1: Header
    builder.push_record(["Tag", "Occurrences"]);

    // Rows 2+: Data rows
    for (tag, count) in &tag_vec {
        builder.push_record([
            &format!(" {}", tag),
            &format!(" {}", count),
        ]);
    }

    // Last row: Footer
    let total_text = format!("Total Tags: {}", tag_vec.len());
    builder.push_record([&total_text, ""]);

    // Build and style
    let mut table = builder.build();
    let config = oxur_table::TableStyleConfig::default();
    config.apply_to_table::<TagRow>(&mut table);

    println!();
    println!("{}", table);
    println!();

    Ok(())
}
```

### Step 5: Add show_components() Function

**File:** `src/commands/info.rs`
**Location:** After `show_tags()` function

Add the complete implementation:

```rust
fn show_components(state_mgr: &StateManager) -> Result<()> {
    use std::collections::HashMap;
    use tabled::builder::Builder;
    use tabled::{Table, Tabled};

    // Marker struct for table type parameter
    #[derive(Tabled)]
    struct ComponentRow {
        component: String,
        occurrences: String,
    }

    // Collect all components with occurrence counts
    let all_docs = state_mgr.state().all();
    let mut component_counts: HashMap<String, usize> = HashMap::new();

    for doc in &all_docs {
        if let Some(component) = &doc.metadata.component {
            *component_counts.entry(component.clone()).or_insert(0) += 1;
        }
    }

    // Convert to sorted vector (by occurrence count, descending)
    let mut component_vec: Vec<_> = component_counts.into_iter().collect();
    component_vec.sort_by(|a, b| b.1.cmp(&a.1));

    // Build table
    let mut builder = Builder::default();

    // Row 0: Title
    builder.push_record(["COMPONENTS", ""]);

    // Row 1: Header
    builder.push_record(["Component", "Occurrences"]);

    // Rows 2+: Data rows
    for (component, count) in &component_vec {
        builder.push_record([
            &format!(" {}", component),
            &format!(" {}", count),
        ]);
    }

    // Last row: Footer
    let total_text = format!("Total Components: {}", component_vec.len());
    builder.push_record([&total_text, ""]);

    // Build and style
    let mut table = builder.build();
    let config = oxur_table::TableStyleConfig::default();
    config.apply_to_table::<ComponentRow>(&mut table);

    println!();
    println!("{}", table);
    println!();

    Ok(())
}
```

### Step 6: Add Required Imports

**File:** `src/commands/info.rs`
**Location:** Top of file (after line 3)

Ensure these imports are present (they may already exist):

```rust
use tabled::builder::Builder;
use tabled::{Table, Tabled};
```

Note: `oxur_table` is already imported via the `design` crate, and `HashMap` is imported in multiple places already.

### Step 7: Add Unit Tests

**File:** `src/commands/info.rs`
**Location:** In test module (after line 784)

Add tests for the new commands:

```rust
#[test]
fn test_info_command_from_str_tags() {
    let cmd = InfoCommand::from_str(Some("tags"));
    assert!(matches!(cmd, InfoCommand::Tags));
}

#[test]
fn test_info_command_from_str_components() {
    let cmd = InfoCommand::from_str(Some("components"));
    assert!(matches!(cmd, InfoCommand::Components));
}

#[test]
fn test_execute_tags() {
    let (_temp, state_mgr) = setup_test_state_manager();
    let result = execute(Some("tags".to_string()), &state_mgr);
    assert!(result.is_ok());
}

#[test]
fn test_execute_components() {
    let (_temp, state_mgr) = setup_test_state_manager();
    let result = execute(Some("components".to_string()), &state_mgr);
    assert!(result.is_ok());
}

#[test]
fn test_show_tags_executes() {
    let (_temp, state_mgr) = setup_test_state_manager();
    let result = show_tags(&state_mgr);
    assert!(result.is_ok());
}

#[test]
fn test_show_components_executes() {
    let (_temp, state_mgr) = setup_test_state_manager();
    let result = show_components(&state_mgr);
    assert!(result.is_ok());
}

#[test]
fn test_show_tags_with_documents() {
    let (temp, mut state_mgr) = setup_test_state_manager();

    // Create documents with tags
    let doc_path = temp.path().join("01-draft/0001-test.md");
    fs::write(
        &doc_path,
        r#"---
number: 1
title: Test Document
state: Draft
created: 2024-01-01
updated: 2024-01-01
author: Test Author
tags: [Phase-0, Research]
---

# Test Document
"#,
    )
    .unwrap();

    state_mgr.quick_scan().unwrap();

    let result = show_tags(&state_mgr);
    assert!(result.is_ok());
}

#[test]
fn test_show_components_with_documents() {
    let (temp, mut state_mgr) = setup_test_state_manager();

    // Create documents with components
    let doc_path = temp.path().join("01-draft/0001-test.md");
    fs::write(
        &doc_path,
        r#"---
number: 1
title: Test Document
state: Draft
created: 2024-01-01
updated: 2024-01-01
author: Test Author
component: REPL
---

# Test Document
"#,
    )
    .unwrap();

    state_mgr.quick_scan().unwrap();

    let result = show_components(&state_mgr);
    assert!(result.is_ok());
}
```

### Step 8: Update Help Text in show_overview()

**File:** `src/commands/info.rs`
**Location:** Lines 92-99 (Quick Help section)

Add the new commands to the help text:

```rust
// Quick help
println!("{}", "Quick Help:".cyan().bold());
println!("  {}  Full command reference", "oxd help".yellow());
println!("  {}  Valid document states", "oxd info states".yellow());
println!("  {}  Frontmatter fields", "oxd info fields".yellow());
println!("  {}  Configuration values", "oxd info config".yellow());
println!("  {}  Project statistics", "oxd info stats".yellow());
println!("  {}  List all tags", "oxd info tags".yellow());          // NEW
println!("  {}  List all components", "oxd info components".yellow()); // NEW
println!();
```

---

## Implementation Order

1. **Step 1-3**: Update InfoCommand enum, parser, and execute function
2. **Step 4**: Implement show_tags() function
3. **Step 5**: Implement show_components() function
4. **Step 6**: Verify/add required imports
5. **Step 7**: Add unit tests
6. **Step 8**: Update help text
7. **Manual Testing**: Run commands with real documents

---

## Example Usage

**With Tags:**
```bash
$ oxd info tags

TAGS

Tag               Occurrences
Phase-0           12
Research          8
Protocol          5
Architecture      3
Performance       2
Security          1

Total Tags: 6
```

**With Components:**
```bash
$ oxd info components

COMPONENTS

Component         Occurrences
REPL              15
AST               12
Compiler          10
Tooling           5
Runtime           3

Total Components: 5
```

**Empty Results:**
```bash
$ oxd info tags

TAGS

Tag               Occurrences

Total Tags: 0
```

---

## Success Criteria

- ✅ `oxd info tags` lists all tags with occurrence counts
- ✅ `oxd info components` lists all components with occurrence counts
- ✅ Both commands use CLI tables with proper formatting
- ✅ Results sorted by occurrence count (descending)
- ✅ Footer shows total count
- ✅ Commands handle empty results gracefully
- ✅ All new tests pass
- ✅ All existing tests continue to pass
- ✅ Help text updated in overview

---

## Notes

- Tags can appear multiple times across documents (Vec<String>)
- Components are singular per document (Option<String>)
- Both use HashMap for efficient counting
- Table styling is consistent with existing `oxd list` commands
- No changes needed to CLI argument parsing (already supports arbitrary info subcommands)
