# Implementation Guide: Add Component, Tags, and Version Fields to Design Doc Metadata

## Overview

This guide details the implementation steps needed to extend the design document metadata/frontmatter with three new fields:

- `component` (String) - The system component this document relates to
- `tags` (Vec<String>) - Keywords/topics for categorization
- `version` (String) - Document version number

## Current State

### Current Metadata Structure

```yaml
---
number: 17
title: "Document Title"
author: "Author Name"
created: 2025-12-28
updated: 2025-12-28
state: Under Review
supersedes: null
superseded-by: null
---
```

### Target Metadata Structure

```yaml
---
number: 17
title: "Document Title"
author: "Author Name"
component: REPL
tags: tcp, ipc, serde
created: 2025-12-28
updated: 2025-12-28
state: Under Review
supersedes: null
superseded-by: null
version: 1.0
---
```

## Files to Modify

### Primary Changes

1. `crates/design/src/doc.rs` - Core metadata type definitions
2. `crates/design/src/commands/add_headers.rs` - Header addition logic
3. Test files in `crates/design/tests/` and `crates/design/src/` test modules

### Secondary Changes (Validation)

- Any code that serializes/deserializes `DocMetadata`
- Documentation in `crates/design/README.md`
- Template file at `crates/design/docs/templates/design-doc-template.md`

## Implementation Steps

### Step 1: Update DocMetadata Struct

**File:** `crates/design/src/doc.rs`

**Location:** Around line 74-86 (the `DocMetadata` struct definition)

**Current Code:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocMetadata {
    pub number: u32,
    pub title: String,
    pub author: String,
    pub created: NaiveDate,
    pub updated: NaiveDate,
    pub state: DocState,
    pub supersedes: Option<u32>,
    #[serde(rename = "superseded-by")]
    pub superseded_by: Option<u32>,
}
```

**New Code:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocMetadata {
    pub number: u32,
    pub title: String,
    pub author: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub created: NaiveDate,
    pub updated: NaiveDate,
    pub state: DocState,
    pub supersedes: Option<u32>,
    #[serde(rename = "superseded-by")]
    pub superseded_by: Option<u32>,
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_version() -> String {
    "1.0".to_string()
}
```

**Rationale:**

- `component` is optional and won't be serialized if None (backward compatibility)
- `tags` defaults to empty vector and won't be serialized if empty
- `version` has a default of "1.0" for backward compatibility
- Fields are positioned logically: component/tags after author (metadata about what), then dates, then lifecycle fields

### Step 2: Update build_yaml_frontmatter Function

**File:** `crates/design/src/doc.rs`

**Location:** Around line 246-271 (the `build_yaml_frontmatter` function)

**Current Code:**

```rust
pub fn build_yaml_frontmatter(metadata: &DocMetadata) -> String {
    let mut yaml = String::from("---\n");
    yaml.push_str(&format!("number: {}\n", metadata.number));
    yaml.push_str(&format!("title: \"{}\"\n", escape_yaml_string(&metadata.title)));
    yaml.push_str(&format!("author: \"{}\"\n", escape_yaml_string(&metadata.author)));
    yaml.push_str(&format!("created: {}\n", metadata.created));
    yaml.push_str(&format!("updated: {}\n", metadata.updated));
    yaml.push_str(&format!("state: {}\n", metadata.state.as_str()));

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

    yaml.push_str("---\n\n");
    yaml
}
```

**New Code:**

```rust
pub fn build_yaml_frontmatter(metadata: &DocMetadata) -> String {
    let mut yaml = String::from("---\n");
    yaml.push_str(&format!("number: {}\n", metadata.number));
    yaml.push_str(&format!("title: \"{}\"\n", escape_yaml_string(&metadata.title)));
    yaml.push_str(&format!("author: \"{}\"\n", escape_yaml_string(&metadata.author)));

    // Add component if present
    if let Some(component) = &metadata.component {
        yaml.push_str(&format!("component: {}\n", component));
    }

    // Add tags if present
    if !metadata.tags.is_empty() {
        let tags_str = metadata.tags.join(", ");
        yaml.push_str(&format!("tags: {}\n", tags_str));
    }

    yaml.push_str(&format!("created: {}\n", metadata.created));
    yaml.push_str(&format!("updated: {}\n", metadata.updated));
    yaml.push_str(&format!("state: {}\n", metadata.state.as_str()));

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

    yaml.push_str(&format!("version: {}\n", metadata.version));

    yaml.push_str("---\n\n");
    yaml
}
```

### Step 3: Update add_missing_headers Function

**File:** `crates/design/src/doc.rs`

**Location:** Around line 302-387 (the `add_missing_headers` function)

**Changes Needed:**

1. Initialize new fields in metadata creation
2. Add new fields to the `added_fields` list when creating from scratch

**In the section where metadata is created from scratch (around line 360-369):**

**Current Code:**

```rust
let metadata = DocMetadata {
    number,
    title,
    author,
    created,
    updated,
    state: DocState::Draft,
    supersedes: None,
    superseded_by: None,
};
```

**New Code:**

```rust
let metadata = DocMetadata {
    number,
    title,
    author,
    component: None,  // Optional, user can add later
    tags: Vec::new(), // Empty by default, user can add later
    created,
    updated,
    state: DocState::Draft,
    supersedes: None,
    superseded_by: None,
    version: "1.0".to_string(), // Default version
};
```

**And update the added_fields list (around line 370-382):**

**Current Code:**

```rust
added_fields = [
    "number",
    "title",
    "author",
    "created",
    "updated",
    "state",
    "supersedes",
    "superseded-by",
]
.iter()
.map(|s| s.to_string())
.collect();
```

**New Code:**

```rust
added_fields = [
    "number",
    "title",
    "author",
    "created",
    "updated",
    "state",
    "supersedes",
    "superseded-by",
    "version",
]
.iter()
.map(|s| s.to_string())
.collect();
```

**Note:** Don't add `component` and `tags` to the added_fields list when they're None/empty, as they're truly optional and their absence is intentional.

### Step 4: Update All Test Fixtures

**Files to Update:**

- All test functions in `crates/design/src/doc.rs` (docstate_tests, parsing_tests, frontmatter_tests, file_operations_tests, property_tests)
- `crates/design/src/commands/add_headers.rs` test module
- Any integration tests in `crates/design/tests/`

**Pattern for updating test metadata creation:**

**Old:**

```rust
let metadata = DocMetadata {
    number: 42,
    title: "Test Document".to_string(),
    author: "Test Author".to_string(),
    created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    updated: NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
    state: DocState::Draft,
    supersedes: None,
    superseded_by: None,
};
```

**New:**

```rust
let metadata = DocMetadata {
    number: 42,
    title: "Test Document".to_string(),
    author: "Test Author".to_string(),
    component: None,
    tags: Vec::new(),
    created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    updated: NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
    state: DocState::Draft,
    supersedes: None,
    superseded_by: None,
    version: "1.0".to_string(),
};
```

**For test YAML strings, add the new fields in the appropriate location:**

**Old:**

```rust
let content = "---\nnumber: 42\ntitle: \"Test\"\nauthor: \"Author\"\ncreated: 2024-01-01\nupdated: 2024-01-02\nstate: Draft\nsupersedes: null\nsuperseded-by: null\n---\n\nContent";
```

**New (minimal - no component/tags):**

```rust
let content = "---\nnumber: 42\ntitle: \"Test\"\nauthor: \"Author\"\ncreated: 2024-01-01\nupdated: 2024-01-02\nstate: Draft\nsupersedes: null\nsuperseded-by: null\nversion: 1.0\n---\n\nContent";
```

**New (with component and tags):**

```rust
let content = "---\nnumber: 42\ntitle: \"Test\"\nauthor: \"Author\"\ncomponent: REPL\ntags: tcp, ipc, serde\ncreated: 2024-01-01\nupdated: 2024-01-02\nstate: Draft\nsupersedes: null\nsuperseded-by: null\nversion: 1.0\n---\n\nContent";
```

### Step 5: Add New Tests for New Fields

**File:** `crates/design/src/doc.rs`

**Add to the `frontmatter_tests` module:**

```rust
#[test]
fn test_build_yaml_frontmatter_with_component() {
    let metadata = DocMetadata {
        number: 1,
        title: "Test".to_string(),
        author: "Author".to_string(),
        component: Some("REPL".to_string()),
        tags: Vec::new(),
        created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        state: DocState::Draft,
        supersedes: None,
        superseded_by: None,
        version: "1.0".to_string(),
    };

    let yaml = build_yaml_frontmatter(&metadata);
    assert!(yaml.contains("component: REPL\n"));
}

#[test]
fn test_build_yaml_frontmatter_with_tags() {
    let metadata = DocMetadata {
        number: 1,
        title: "Test".to_string(),
        author: "Author".to_string(),
        component: None,
        tags: vec!["tcp".to_string(), "ipc".to_string(), "serde".to_string()],
        created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        state: DocState::Draft,
        supersedes: None,
        superseded_by: None,
        version: "1.0".to_string(),
    };

    let yaml = build_yaml_frontmatter(&metadata);
    assert!(yaml.contains("tags: tcp, ipc, serde\n"));
}

#[test]
fn test_build_yaml_frontmatter_without_optional_fields() {
    let metadata = DocMetadata {
        number: 1,
        title: "Test".to_string(),
        author: "Author".to_string(),
        component: None,
        tags: Vec::new(),
        created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        state: DocState::Draft,
        supersedes: None,
        superseded_by: None,
        version: "1.0".to_string(),
    };

    let yaml = build_yaml_frontmatter(&metadata);
    assert!(!yaml.contains("component:"));
    assert!(!yaml.contains("tags:"));
    assert!(yaml.contains("version: 1.0\n"));
}

#[test]
fn test_build_yaml_frontmatter_with_version() {
    let metadata = DocMetadata {
        number: 1,
        title: "Test".to_string(),
        author: "Author".to_string(),
        component: None,
        tags: Vec::new(),
        created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        state: DocState::Draft,
        supersedes: None,
        superseded_by: None,
        version: "2.1".to_string(),
    };

    let yaml = build_yaml_frontmatter(&metadata);
    assert!(yaml.contains("version: 2.1\n"));
}

#[test]
fn test_parse_document_with_new_fields() {
    let content = "---\nnumber: 42\ntitle: \"Test Document\"\nauthor: \"Test Author\"\ncomponent: REPL\ntags: tcp, ipc, serde\ncreated: 2024-01-01\nupdated: 2024-01-02\nstate: Draft\nsupersedes: null\nsuperseded-by: null\nversion: 1.0\n---\n\n# Test Document\n\nThis is the content.";

    let result = DesignDoc::parse(content, PathBuf::from("test.md"));

    assert!(result.is_ok());
    let doc = result.unwrap();
    assert_eq!(doc.metadata.component, Some("REPL".to_string()));
    assert_eq!(doc.metadata.tags.len(), 3);
    assert!(doc.metadata.tags.contains(&"tcp".to_string()));
    assert!(doc.metadata.tags.contains(&"ipc".to_string()));
    assert!(doc.metadata.tags.contains(&"serde".to_string()));
    assert_eq!(doc.metadata.version, "1.0");
}

#[test]
fn test_parse_document_backward_compatibility() {
    // Old format without new fields should still parse
    let content = "---\nnumber: 42\ntitle: \"Test Document\"\nauthor: \"Test Author\"\ncreated: 2024-01-01\nupdated: 2024-01-02\nstate: Draft\nsupersedes: null\nsuperseded-by: null\n---\n\n# Test Document\n\nThis is the content.";

    let result = DesignDoc::parse(content, PathBuf::from("test.md"));

    assert!(result.is_ok());
    let doc = result.unwrap();
    assert_eq!(doc.metadata.component, None);
    assert!(doc.metadata.tags.is_empty());
    assert_eq!(doc.metadata.version, "1.0"); // Default version
}
```

### Step 6: Update Template File

**File:** `crates/design/docs/templates/design-doc-template.md`

Update the frontmatter section to include the new fields:

```markdown
---
number: NNNN
title: "Document Title"
author: "Your Name"
component: ComponentName  # Optional: e.g., REPL, AST, Compiler
tags: tag1, tag2, tag3     # Optional: keywords for categorization
created: YYYY-MM-DD
updated: YYYY-MM-DD
state: Draft
supersedes: null
superseded-by: null
version: 1.0
---
```

### Step 7: Update Documentation

**File:** `crates/design/README.md`

Add documentation about the new fields. Look for the section describing document structure and add:

```markdown
### Metadata Fields

Design documents use YAML frontmatter with the following fields:

- `number`: Sequential document number (auto-assigned)
- `title`: Document title
- `author`: Author name(s)
- `component` (optional): System component (e.g., REPL, AST, Compiler)
- `tags` (optional): Comma-separated keywords for categorization
- `created`: Creation date (YYYY-MM-DD)
- `updated`: Last update date (YYYY-MM-DD)
- `state`: Document lifecycle state
- `supersedes`: Number of document this replaces (or null)
- `superseded-by`: Number of document that replaces this (or null)
- `version`: Document version (default: 1.0)
```

## Testing Strategy

### Unit Tests

1. Run existing tests to ensure backward compatibility:

   ```bash
   cd crates/design
   cargo test
   ```

2. Add and run new tests for the new fields:
   - Test parsing documents with new fields
   - Test parsing documents without new fields (backward compatibility)
   - Test YAML generation with/without optional fields
   - Test metadata serialization/deserialization

### Integration Tests

1. Create a test document with all new fields
2. Run `oxd add` to verify it processes correctly
3. Run `oxd list` to verify display works
4. Run `oxd show` to verify metadata display
5. Run `oxd validate` to ensure validation passes

### Manual Tests

1. Create a new document with the new fields manually
2. Run `oxd add-headers` on a document without the new fields
3. Verify existing documents still work without the new fields
4. Test the `oxd new` command creates documents with proper defaults

## Migration Considerations

### Backward Compatibility

- **Critical:** Existing documents without the new fields MUST continue to work
- The `component` and `tags` fields are optional (Option/Vec)
- The `version` field has a default value
- Serde attributes handle deserialization of old documents

### Forward Compatibility

- New documents should include `version: 1.0` by default
- `component` and `tags` can be omitted if not applicable
- Consider adding a migration command if bulk updates are needed

### Recommended Migration Path

1. Deploy changes without requiring updates to existing documents
2. Update template and documentation
3. Optionally create a migration tool: `oxd migrate-metadata`
4. Update existing documents gradually or on-demand

## Potential Issues and Solutions

### Issue 1: Serde Deserialization Order

**Problem:** YAML field order might matter for readability
**Solution:** The implementation controls serialization order in `build_yaml_frontmatter`, so output will always be consistent

### Issue 2: Tags Parsing

**Problem:** Tags are comma-separated in YAML but Vec in Rust
**Solution:** Consider implementing custom deserializer if the simple string approach doesn't work:

```rust
#[serde(
    default,
    skip_serializing_if = "Vec::is_empty",
    deserialize_with = "deserialize_tags"
)]
pub tags: Vec<String>,

fn deserialize_tags<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(Vec::new())
    } else {
        Ok(s.split(',').map(|t| t.trim().to_string()).collect())
    }
}
```

### Issue 3: Existing State Files

**Problem:** State files might have serialized metadata without new fields
**Solution:** The serde `default` attributes handle this automatically

## Checklist

- [ ] Update `DocMetadata` struct with new fields
- [ ] Update `build_yaml_frontmatter` function
- [ ] Update `add_missing_headers` function
- [ ] Update all test fixtures to include new fields
- [ ] Add tests for new fields
- [ ] Add backward compatibility tests
- [ ] Update template file
- [ ] Update README documentation
- [ ] Run full test suite
- [ ] Manual integration testing
- [ ] Update any CLI help text if needed
- [ ] Consider adding filtering/search by component/tags (future enhancement)

## Future Enhancements

Once the basic implementation is complete, consider:

1. **Filtering by component:**

   ```bash
   oxd list --component REPL
   ```

2. **Searching by tags:**

   ```bash
   oxd search --tag tcp
   ```

3. **Tag validation:**
   - Maintain a list of valid tags
   - Warn on unknown tags

4. **Component validation:**
   - Validate against known components
   - Auto-suggest components

5. **Version management:**
   - Implement version comparison
   - Track version history
   - Auto-increment version on updates

## Summary

This implementation adds three new metadata fields while maintaining full backward compatibility. The key principles are:

1. **Optional fields** (component, tags) use Option/Vec with serde defaults
2. **Version field** has a sensible default
3. **Serialization order** is controlled for readability
4. **Tests** ensure both forward and backward compatibility
5. **Migration** is gradual and non-breaking

The changes are primarily in `doc.rs` with updates to tests and documentation. The implementation should take 2-4 hours for an experienced Rust developer.
