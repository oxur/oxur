# Implementation Plan: Add Component, Tags, and Version Metadata Fields

## Overview

Add three new metadata fields to design documents with full CLI support:
- `component` (optional String) - System component categorization
- `tags` (Vec<String>) - Keywords for filtering/searching
- `version` (String) - Document version with validation (format: major.minor)

## User Requirements

**YAML Field Order (EXACT):**
```yaml
number: 17
title: "Document Title"
author: "Author Name"
component: REPL          # NEW - optional, after author
tags: [tcp, ipc, serde]  # NEW - YAML list format, after component
created: 2025-12-28
updated: 2025-12-28
state: Under Review
supersedes: null
superseded-by: null
version: 1.0             # NEW - required with default, after superseded-by
```

**CLI Features:**
- `oxd new --component <NAME> --tags <A>,<B>,<C>`
- `oxd list --component <NAME> --tags <A>,<B>,<C>` (OR filtering)
- `oxd show <NNNN>` (display new metadata)

**Validation:**
- Version format: `major.minor` (e.g., "1.0", "2.3")
- Reject: "1.0.0" (has micro), "1" (missing minor), "1.a" (non-numeric)

**Document Updates:**
- Review and update all 18 existing design docs with appropriate component/tags

---

## Critical Files to Modify

1. `/Users/oubiwann/lab/oxur/oxur/crates/design/src/doc.rs` (lines 178-286)
2. `/Users/oubiwann/lab/oxur/oxur/crates/design/src/cli.rs` (lines 56-94)
3. `/Users/oubiwann/lab/oxur/oxur/crates/design/src/commands/new.rs` (lines 12-95)
4. `/Users/oubiwann/lab/oxur/oxur/crates/design/src/commands/list.rs` (lines 66-186)
5. `/Users/oubiwann/lab/oxur/oxur/crates/design/src/commands/show.rs` (lines 27-92)
6. `/Users/oubiwann/lab/oxur/oxur/crates/design/src/main.rs` (dispatch updates)

---

## Implementation Phases

### Phase 1: Core Metadata Structure

**File: `src/doc.rs`**

**Step 1.1: Add fields to DocMetadata struct (lines 178-190)**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocMetadata {
    pub number: u32,
    pub title: String,
    pub author: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created: NaiveDate,
    pub updated: NaiveDate,
    pub state: DocState,
    pub supersedes: Option<u32>,
    #[serde(rename = "superseded-by")]
    pub superseded_by: Option<u32>,
    #[serde(default = "default_version", deserialize_with = "deserialize_version")]
    pub version: String,
}

fn default_version() -> String {
    "1.0".to_string()
}
```

**Step 1.2: Add version validation deserializer (after line 176)**

```rust
fn deserialize_version<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Deserialize;
    let s = String::deserialize(deserializer)?;

    // Validate format: major.minor (e.g., "1.0", "2.3")
    let parts: Vec<&str> = s.split('.').collect();

    if parts.len() != 2 {
        return Err(serde::de::Error::custom(
            format!("Version must be in major.minor format (e.g., '1.0'), got: '{}'", s)
        ));
    }

    // Validate both parts are numeric
    for (idx, part) in parts.iter().enumerate() {
        if part.parse::<u32>().is_err() {
            let label = if idx == 0 { "major" } else { "minor" };
            return Err(serde::de::Error::custom(
                format!("Invalid {} version number: '{}' in '{}'", label, part, s)
            ));
        }
    }

    Ok(s)
}
```

**Step 1.3: Update build_yaml_frontmatter (lines 263-286)**

Replace entire function to maintain exact field order:

```rust
pub fn build_yaml_frontmatter(metadata: &DocMetadata) -> String {
    let mut yaml = String::from("---\n");

    // 1-3: number, title, author
    yaml.push_str(&format!("number: {}\n", metadata.number));
    yaml.push_str(&format!("title: \"{}\"\n", escape_yaml_string(&metadata.title)));
    yaml.push_str(&format!("author: \"{}\"\n", escape_yaml_string(&metadata.author)));

    // 4: component (only if Some)
    if let Some(component) = &metadata.component {
        yaml.push_str(&format!("component: {}\n", component));
    }

    // 5: tags (only if non-empty) - YAML flow sequence
    if !metadata.tags.is_empty() {
        yaml.push_str(&format!("tags: [{}]\n", metadata.tags.join(", ")));
    }

    // 6-8: created, updated, state
    yaml.push_str(&format!("created: {}\n", metadata.created));
    yaml.push_str(&format!("updated: {}\n", metadata.updated));
    yaml.push_str(&format!("state: {}\n", metadata.state.as_str()));

    // 9-10: supersedes, superseded-by
    if let Some(supersedes) = metadata.supersedes {
        yaml.push_str(&format!("supersedes: {}\n", supersedes));
    } else {
        yaml.push_str("supersedes: null\n");
    }

    if let Some(superseded_by) = metadata.superseded_by {
        yaml.push_str(&format!("superseded-by: {}\n", superseded_by));
    } else {
        yaml.push_str("superseded-by: null\n");
    }

    // 11: version (always present)
    yaml.push_str(&format!("version: {}\n", metadata.version));

    yaml.push_str("---\n\n");
    yaml
}
```

**Step 1.4: Update all test fixtures in doc.rs**

Pattern for updating DocMetadata construction:
```rust
let metadata = DocMetadata {
    number: 42,
    title: "Test".to_string(),
    author: "Author".to_string(),
    component: None,              // ADD
    tags: Vec::new(),             // ADD
    created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    state: DocState::Draft,
    supersedes: None,
    superseded_by: None,
    version: "1.0".to_string(),   // ADD
};
```

---

### Phase 2: CLI Argument Definitions

**File: `src/cli.rs`**

**Step 2.1: Update New command (lines 86-94)**

```rust
New {
    /// Document title
    title: String,

    /// Author name (defaults to git config user.name)
    #[arg(short, long)]
    author: Option<String>,

    /// System component (e.g., Compiler, AST, REPL, Tooling)
    #[arg(short, long)]
    component: Option<String>,

    /// Tags (comma-separated, e.g., "Phase-0,Research,Protocol")
    #[arg(short, long, value_delimiter = ',')]
    tags: Vec<String>,
},
```

**Step 2.2: Update List command (lines 58-74)**

```rust
List {
    #[arg(short, long)]
    state: Option<String>,

    #[arg(short, long)]
    verbose: bool,

    #[arg(long)]
    removed: bool,

    #[arg(long)]
    dev: bool,

    /// Filter by component
    #[arg(short, long)]
    component: Option<String>,

    /// Filter by tags (comma-separated, matches ANY tag)
    #[arg(short, long, value_delimiter = ',')]
    tags: Vec<String>,
},
```

---

### Phase 3: Command Implementations

**File: `src/commands/new.rs`**

**Step 3.1: Update function signature (line 12)**

```rust
pub fn new_document(
    index: &DocumentIndex,
    title: String,
    author: Option<String>,
    component: Option<String>,
    tags: Vec<String>,
) -> Result<()>
```

**Step 3.2: Update template generation (lines 18-61)**

```rust
// After line 16 (today = ...)
let component_yaml = component.as_ref()
    .map(|c| format!("component: {}\n", c))
    .unwrap_or_default();

let tags_yaml = if tags.is_empty() {
    String::new()
} else {
    format!("tags: [{}]\n", tags.join(", "))
};

let template = format!(
    r#"---
number: {}
title: "{}"
author: "{}"
{}{}created: {}
updated: {}
state: Draft
supersedes: null
superseded-by: null
version: 1.0
---

# {}

[... rest of template unchanged ...]
"#,
    number, title, author, component_yaml, tags_yaml, today, today, title
);
```

---

**File: `src/commands/list.rs`**

**Step 3.3: Update function signatures**

```rust
// Line 66
pub fn list_documents_with_state(
    index: &DocumentIndex,
    state_mgr: Option<&StateManager>,
    state_filter: Option<String>,
    verbose: bool,
    removed: bool,
    dev: bool,
    component_filter: Option<String>,
    tags_filter: Vec<String>,
) -> Result<()>

// Line 85
fn list_documents_impl(
    index: &DocumentIndex,
    state_mgr: Option<&StateManager>,
    state_filter: Option<String>,
    verbose: bool,
    removed: bool,
    dev: bool,
    component_filter: Option<String>,
    tags_filter: Vec<String>,
) -> Result<()>
```

**Step 3.4: Add filtering logic (after state filtering, before display)**

```rust
// After getting initial docs by state...

// Apply component filter
if let Some(component) = &component_filter {
    docs.retain(|doc| {
        doc.metadata.component.as_ref()
            .map(|c| c == component)
            .unwrap_or(false)
    });
}

// Apply tags filter (OR logic - match ANY tag)
if !tags_filter.is_empty() {
    docs.retain(|doc| {
        tags_filter.iter().any(|filter_tag| {
            doc.metadata.tags.iter().any(|doc_tag| doc_tag == filter_tag)
        })
    });
}
```

---

**File: `src/commands/show.rs`**

**Step 3.5: Add metadata display (after "Updated" row, around line 60)**

```rust
// After: builder.push_record([" Updated ", &format!(" {}", doc.metadata.updated)]);

// Component (only if present)
if let Some(component) = &doc.metadata.component {
    builder.push_record([" Component", &format!(" {}", component)]);
}

// Tags (only if non-empty)
if !doc.metadata.tags.is_empty() {
    let tags_str = doc.metadata.tags.join(", ");
    builder.push_record([" Tags", &format!(" {}", tags_str)]);
}

// Version
builder.push_record([" Version", &format!(" {}", doc.metadata.version)]);
```

---

**File: `src/main.rs`**

**Step 3.6: Update command dispatch**

```rust
Commands::New { title, author, component, tags } => {
    commands::new::new_document(&index, title, author, component, tags)?;
}

Commands::List { state, verbose, removed, dev, component, tags } => {
    commands::list::list_documents_with_state(
        &index,
        Some(&state_mgr),
        state,
        verbose,
        removed,
        dev,
        component,
        tags,
    )?;
}
```

---

### Phase 4: Testing

**Add to doc.rs test module:**

```rust
#[test]
fn test_version_validation_valid() {
    // Test "1.0", "2.5", etc.
}

#[test]
fn test_version_validation_invalid_micro() {
    // Reject "1.0.0"
}

#[test]
fn test_version_validation_invalid_single() {
    // Reject "1"
}

#[test]
fn test_component_and_tags_parsing() {
    // Test parsing docs with component and tags
}

#[test]
fn test_backward_compatibility() {
    // Old docs without new fields should parse
}

#[test]
fn test_yaml_field_order() {
    // Verify exact field ordering in output
}
```

**Add to list.rs test module:**

```rust
#[test]
fn test_filter_by_component() {
    // Test --component filtering
}

#[test]
fn test_filter_by_tags_or_logic() {
    // Test --tags with multiple values (OR)
}

#[test]
fn test_combined_filters() {
    // Test --state + --component + --tags
}
```

---

### Phase 5: Update Existing Documents

**18 Documents to Update:**

| Doc | Title | Component | Tags |
|-----|-------|-----------|------|
| 0001 | Oxur: A Letter of Intent | Core | Vision, Architecture |
| 0002 | Design Docs CLI - Build Plan | Tooling | CLI, Infrastructure |
| 0003 | oxur-ast: Canonical S-Expression | AST | S-Expression, Phase-0 |
| 0004 | oxur-ast Phase 0 | AST | Phase-0, Infrastructure |
| 0005 | oxur-ast Phase 1 | AST | Phase-1, Builder |
| 0006 | oxur-ast Phase 2 | AST | Phase-2, Generator |
| 0007 | oxur-ast Phase 3 | AST | Phase-3, Testing, CLI |
| 0008 | oxur-ast Phase 4 | AST | Phase-4, Coverage |
| 0009 | Design Docs CLI - Phases 6-8 | Tooling | CLI, Phase-6, Phase-7, Phase-8 |
| 0010 | Design Docs CLI - Phases 9-10 | Tooling | CLI, Phase-9, Phase-10 |
| 0011 | Dead Code Remediation | AST | Maintenance, Refactoring |
| 0012 | File-Based S-Expression Test Data | Testing | Infrastructure, S-Expression |
| 0013 | Compilation Chain Architecture | Compiler | Architecture, Multi-Stage |
| 0014 | Rust AST Inventory | AST | Reference, Documentation |
| 0015 | oxur-table API Design | UI | API-Design, Table |
| 0016 | Transport-agnostic REPL | REPL | Research, Protocol, Multi-Transport |
| 0017 | Future-proofing REPL Protocols | REPL | Protocol, Design |
| 0018 | Remote REPL Protocol Design | REPL | Protocol, Multi-Transport, API-Design |

**Update Process:**
1. For each document, add after the `author:` line:
   - `component: <ComponentName>` (if applicable)
   - `tags: [tag1, tag2, ...]` (if applicable)
2. Add before final `---`:
   - `version: 1.0`
3. Verify YAML parses correctly
4. Run `oxd update-index` to refresh index

---

## Implementation Order

1. **Phase 1** - Core metadata (doc.rs)
   - Add fields to DocMetadata
   - Add version deserializer
   - Update build_yaml_frontmatter
   - Update test fixtures

2. **Phase 2** - CLI definitions (cli.rs)
   - Add flags to New command
   - Add flags to List command

3. **Phase 3** - Commands (new.rs, list.rs, show.rs, main.rs)
   - Update new_document
   - Update list filtering
   - Update show display
   - Update main dispatch

4. **Phase 4** - Testing
   - Add unit tests
   - Run test suite
   - Manual CLI testing

5. **Phase 5** - Document updates
   - Update all 18 documents
   - Verify index sync

---

## Validation & Testing

**Manual Tests:**
```bash
# Create with all fields
oxd new "Test Doc" --component REPL --tags Phase-0,Research

# List filtering
oxd list --component REPL
oxd list --tags Phase-0,Research
oxd list --state draft --component REPL

# Show metadata
oxd show 0001

# Verify old docs still work
oxd list  # Should show all docs without errors
```

**Success Criteria:**
- All existing tests pass
- New tests cover version validation
- New tests cover filtering logic
- All 18 documents updated successfully
- Backward compatibility maintained
- YAML field order matches specification exactly

---

## Notes

- **Tags format**: YAML list `[tcp, ipc, serde]` handled natively by serde
- **No custom tags deserializer needed**: serde handles `Vec<String>` automatically
- **OR filtering**: `--component REPL --tags tcp,ipc` matches docs with REPL OR any tag
- **Tag matching**: `--tags tcp,ipc` matches docs with tcp OR ipc
- **Version default**: Documents without version field default to "1.0"
- **Backward compat**: Old documents parse correctly (component=None, tags=[], version="1.0")
