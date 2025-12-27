# Phase 1: Foundation Enhancements - Detailed Implementation Guide

## Overview

This phase establishes the foundational capabilities needed for all subsequent features. We'll expand the state system, add git integration, and enhance YAML operations.

## Task 1.1: Expand State System

### Current State (Rust)

```rust
// In design/src/doc.rs
pub enum DocState {
    Draft,
    UnderReview,
    Final,
    Superseded,
}
```

### Target State (from Go tool)

10 states with directory mappings:

- Draft → 01-draft
- Under Review → 02-under-review
- Revised → 03-revised
- Accepted → 04-accepted
- Active → 05-active
- Final → 06-final
- Deferred → 07-deferred
- Rejected → 08-rejected
- Withdrawn → 09-withdrawn
- Superseded → 10-superseded

### Implementation Steps

#### Step 1: Update DocState Enum

File: `design/src/doc.rs`

Expand the enum to include all 10 states:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
}
```

#### Step 2: Update as_str() Method

Provide proper display names for each state:

```rust
pub fn as_str(&self) -> &'static str {
    match self {
        DocState::Draft => "Draft",
        DocState::UnderReview => "Under Review",
        DocState::Revised => "Revised",
        DocState::Accepted => "Accepted",
        DocState::Active => "Active",
        DocState::Final => "Final",
        DocState::Deferred => "Deferred",
        DocState::Rejected => "Rejected",
        DocState::Withdrawn => "Withdrawn",
        DocState::Superseded => "Superseded",
    }
}
```

#### Step 3: Update directory() Method

Update directory mappings:

```rust
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
    }
}
```

#### Step 4: Add State Parsing Methods

Add helper methods for flexible state parsing:

```rust
impl DocState {
    /// Parse from string, handling various formats
    pub fn from_str_flexible(s: &str) -> Option<Self> {
        let normalized = s.to_lowercase().replace('-', " ").trim().to_string();
        match normalized.as_str() {
            "draft" => Some(DocState::Draft),
            "under review" | "review" => Some(DocState::UnderReview),
            "revised" => Some(DocState::Revised),
            "accepted" => Some(DocState::Accepted),
            "active" => Some(DocState::Active),
            "final" => Some(DocState::Final),
            "deferred" => Some(DocState::Deferred),
            "rejected" => Some(DocState::Rejected),
            "withdrawn" => Some(DocState::Withdrawn),
            "superseded" => Some(DocState::Superseded),
            _ => None,
        }
    }

    /// Get DocState from directory name
    pub fn from_directory(dir: &str) -> Option<Self> {
        match dir {
            "01-draft" => Some(DocState::Draft),
            "02-under-review" => Some(DocState::UnderReview),
            "03-revised" => Some(DocState::Revised),
            "04-accepted" => Some(DocState::Accepted),
            "05-active" => Some(DocState::Active),
            "06-final" => Some(DocState::Final),
            "07-deferred" => Some(DocState::Deferred),
            "08-rejected" => Some(DocState::Rejected),
            "09-withdrawn" => Some(DocState::Withdrawn),
            "10-superseded" => Some(DocState::Superseded),
            _ => None,
        }
    }

    /// Get all valid states
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
        ]
    }

    /// Get all valid state names for display
    pub fn all_state_names() -> Vec<&'static str> {
        Self::all_states().iter().map(|s| s.as_str()).collect()
    }
}
```

#### Step 5: Update Deserialization

Ensure serde can handle the YAML variations:

```rust
impl<'de> Deserialize<'de> for DocState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DocState::from_str_flexible(&s)
            .ok_or_else(|| serde::de::Error::custom(format!("Invalid state: {}", s)))
    }
}
```

### Testing Steps for 1.1

1. Create test documents with each state in YAML
2. Verify parsing works for all states
3. Test flexible parsing (hyphens, spaces, case)
4. Verify directory mappings are correct
5. Test serialization/deserialization round-trip

---

## Task 1.2: Git Integration Module

### Purpose

Extract metadata from git history and perform git operations while preserving history.

### Implementation Steps

#### Step 1: Create git.rs Module

File: `design/src/git.rs`

Add module declaration in `design/src/lib.rs`:

```rust
pub mod git;
```

#### Step 2: Implement Author Extraction

```rust
use std::path::Path;
use std::process::Command;

/// Extract the original author from git history
/// Falls back to "Unknown Author" if git fails
pub fn get_author(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();

    let output = Command::new("git")
        .args(["log", "--format=%an", "--reverse"])
        .arg(path)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines()
                .next()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "Unknown Author".to_string())
        }
        _ => "Unknown Author".to_string(),
    }
}
```

#### Step 3: Implement Date Extraction

```rust
use chrono::NaiveDate;

/// Extract creation date from git history (first commit)
/// Falls back to today's date if git fails
pub fn get_created_date(path: impl AsRef<Path>) -> NaiveDate {
    let path = path.as_ref();

    let output = Command::new("git")
        .args(["log", "--format=%ai", "--reverse"])
        .arg(path)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines()
                .next()
                .and_then(|line| {
                    // Extract just the date portion (YYYY-MM-DD)
                    line.split_whitespace()
                        .next()
                        .and_then(|date_str| NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok())
                })
                .unwrap_or_else(|| chrono::Local::now().naive_local().date())
        }
        _ => chrono::Local::now().naive_local().date(),
    }
}

/// Extract last modified date from git history
/// Falls back to today's date if git fails
pub fn get_updated_date(path: impl AsRef<Path>) -> NaiveDate {
    let path = path.as_ref();

    let output = Command::new("git")
        .args(["log", "--format=%ai", "-1"])
        .arg(path)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.split_whitespace()
                .next()
                .and_then(|date_str| NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok())
                .unwrap_or_else(|| chrono::Local::now().naive_local().date())
        }
        _ => chrono::Local::now().naive_local().date(),
    }
}
```

#### Step 4: Implement Git Move

```rust
use anyhow::{Context, Result};
use std::fs;

/// Move a file using git mv to preserve history
/// Creates destination directory if needed
pub fn git_mv(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    // Ensure destination directory exists
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)
            .context("Failed to create destination directory")?;
    }

    // Execute git mv
    let output = Command::new("git")
        .args(["mv"])
        .arg(src)
        .arg(dst)
        .output()
        .context("Failed to execute git mv")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git mv failed: {}", stderr);
    }

    Ok(())
}
```

#### Step 5: Implement Git Add

```rust
/// Stage a file with git add
pub fn git_add(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();

    let output = Command::new("git")
        .args(["add"])
        .arg(path)
        .output()
        .context("Failed to execute git add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git add failed: {}", stderr);
    }

    Ok(())
}
```

#### Step 6: Add Helper Functions

```rust
/// Check if a path is in a git repository
pub fn is_git_repo(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();

    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(path.parent().unwrap_or(path))
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Check if a file is tracked by git
pub fn is_tracked(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();

    Command::new("git")
        .args(["ls-files", "--error-unmatch"])
        .arg(path)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
```

### Testing Steps for 1.2

1. Test with files in git repository
2. Test fallback behavior (non-git files)
3. Verify git mv preserves history
4. Test git add staging
5. Test helper functions (is_git_repo, is_tracked)

---

## Task 1.3: Enhanced YAML Operations

### Purpose

Add functions to surgically update YAML frontmatter and add missing fields.

### Implementation Steps

#### Step 1: Add YAML Update Function

File: `design/src/doc.rs`

```rust
use regex::Regex;

impl DesignDoc {
    /// Update a specific field in the YAML frontmatter
    pub fn update_yaml_field(content: &str, field: &str, value: &str) -> Result<String, DocError> {
        // Pattern to match the field line
        let pattern = format!(r"(?m)^{}: .*$", regex::escape(field));
        let re = Regex::new(&pattern)
            .map_err(|e| DocError::InvalidFormat(format!("Regex error: {}", e)))?;

        let replacement = format!("{}: {}", field, value);
        Ok(re.replace(content, replacement.as_str()).to_string())
    }

    /// Update the state and updated date in one operation
    pub fn update_state(content: &str, new_state: DocState) -> Result<String, DocError> {
        let today = chrono::Local::now().naive_local().date();

        let mut updated = Self::update_yaml_field(content, "state", new_state.as_str())?;
        updated = Self::update_yaml_field(&updated, "updated", &today.to_string())?;

        Ok(updated)
    }
}
```

#### Step 2: Add Missing Headers Function

```rust
use std::path::Path;

/// Build complete YAML frontmatter from metadata
pub fn build_yaml_frontmatter(metadata: &DocMetadata) -> String {
    let mut yaml = String::from("---\n");
    yaml.push_str(&format!("number: {}\n", metadata.number));
    yaml.push_str(&format!("title: \"{}\"\n", metadata.title));
    yaml.push_str(&format!("author: {}\n", metadata.author));
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

/// Extract title from content (first # heading) or filename
pub fn extract_title_from_content(content: &str, filename: &str) -> String {
    // Look for first # heading
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            return trimmed[2..].trim().to_string();
        }
    }

    // Infer from filename: 0001-my-document.md -> "My Document"
    let re = Regex::new(r"^\d+-(.+)\.md$").unwrap();
    if let Some(caps) = re.captures(filename) {
        if let Some(slug) = caps.get(1) {
            return slug.as_str()
                .split('-')
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
        }
    }

    "Untitled Document".to_string()
}

/// Extract number from filename (with 4-digit padding)
pub fn extract_number_from_filename(filename: &str) -> String {
    let re = Regex::new(r"^(\d+)-").unwrap();
    if let Some(caps) = re.captures(filename) {
        if let Some(num) = caps.get(1) {
            // Pad to 4 digits
            let num_str = num.as_str();
            return format!("{:0>4}", num_str);
        }
    }
    "0000".to_string()
}

/// Add or complete YAML frontmatter headers
pub fn add_missing_headers(
    path: impl AsRef<Path>,
    content: &str,
) -> Result<(String, Vec<String>), DocError> {
    use crate::git;

    let path = path.as_ref();
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| DocError::InvalidFormat("Invalid filename".to_string()))?;

    // Extract metadata
    let number = extract_number_from_filename(filename);
    let title = extract_title_from_content(content, filename);
    let author = git::get_author(path);
    let created = git::get_created_date(path);
    let updated = git::get_updated_date(path);

    let mut added_fields = Vec::new();

    // Check if document has frontmatter
    if content.trim_start().starts_with("---\n") {
        // Parse existing frontmatter
        let existing = match DesignDoc::parse(content, path.to_path_buf()) {
            Ok(doc) => doc.metadata,
            Err(_) => {
                // Partial frontmatter, build from scratch
                let metadata = DocMetadata {
                    number: number.parse().unwrap_or(0),
                    title: title.clone(),
                    author: author.clone(),
                    created,
                    updated,
                    state: DocState::Draft,
                    supersedes: None,
                    superseded_by: None,
                };
                added_fields = vec!["number", "title", "author", "created", "updated", "state", "supersedes", "superseded-by"]
                    .iter().map(|s| s.to_string()).collect();

                // Strip old frontmatter and add new
                let re = Regex::new(r"(?s)^---\n.*?\n---\n\n?").unwrap();
                let body = re.replace(content, "");
                let new_content = build_yaml_frontmatter(&metadata) + &body;
                return Ok((new_content, added_fields));
            }
        };

        // Merge with discovered metadata
        let mut metadata = existing;

        // Only update if empty/default
        if metadata.number == 0 {
            metadata.number = number.parse().unwrap_or(0);
            added_fields.push("number".to_string());
        }
        if metadata.title.is_empty() || metadata.title == "Untitled Document" {
            metadata.title = title;
            added_fields.push("title".to_string());
        }
        if metadata.author.is_empty() || metadata.author == "Unknown Author" {
            metadata.author = author;
            added_fields.push("author".to_string());
        }

        // Strip old frontmatter and add complete new one
        let re = Regex::new(r"(?s)^---\n.*?\n---\n\n?").unwrap();
        let body = re.replace(content, "");
        let new_content = build_yaml_frontmatter(&metadata) + &body;

        Ok((new_content, added_fields))
    } else {
        // No frontmatter, add it
        let metadata = DocMetadata {
            number: number.parse().unwrap_or(0),
            title,
            author,
            created,
            updated,
            state: DocState::Draft,
            supersedes: None,
            superseded_by: None,
        };

        added_fields = vec!["number", "title", "author", "created", "updated", "state", "supersedes", "superseded-by"]
            .iter().map(|s| s.to_string()).collect();

        let new_content = build_yaml_frontmatter(&metadata) + content;
        Ok((new_content, added_fields))
    }
}
```

### Testing Steps for 1.3

1. Test updating single YAML fields
2. Test updating state and date together
3. Test adding headers to file without frontmatter
4. Test completing partial frontmatter
5. Test title extraction from content vs filename
6. Test number extraction and padding

---

## Dependencies to Add

Add to `design/Cargo.toml`:

```toml
[dependencies]
regex = "1"
```

(Other dependencies like `chrono`, `serde`, `serde_yaml`, `colored`, `anyhow`, `thiserror`, `clap`, `walkdir` are already present)

---

## Verification Checklist

Before moving to Phase 2, verify:

- [ ] All 10 states defined and working
- [ ] State parsing handles various formats (hyphens, spaces, case)
- [ ] Directory mappings correct for all states
- [ ] Git author extraction working
- [ ] Git date extraction working (created and updated)
- [ ] Git mv preserves history
- [ ] Git add stages files
- [ ] YAML field updates work correctly
- [ ] State transitions update both state and date
- [ ] Missing headers can be added automatically
- [ ] Title extraction works from content and filename
- [ ] Number extraction and padding works

---

## Integration Points

These foundation pieces will be used by:

- Phase 2: Commands will use git functions and YAML operations
- Phase 3: Index sync will use state system
- Phase 4: Add workflow will use all foundation pieces
- Phase 5: Validation will check state consistency

---

## Notes for Claude Code

- Use existing error types (`DocError`) where possible
- Follow existing patterns for module structure
- Maintain consistency with existing colored output
- All git operations should have sensible fallbacks
- Test with both git-tracked and non-git files
- Ensure YAML updates preserve formatting where reasonable
