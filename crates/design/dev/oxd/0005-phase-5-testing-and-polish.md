# Phase 5: Testing & Polish - Detailed Implementation Guide

## Overview
This final phase adds the finishing touches that transform the tool from functional to production-ready. We'll enhance validation, add command aliases, improve error handling, refine colored output, and create comprehensive documentation.

**Prerequisites:** Phases 1-4 must be complete

---

## Task 5.1: Enhanced Validation

### Purpose
Extend the validate command to catch all possible inconsistencies and optionally fix them.

### Implementation Steps

#### Step 1: Expand Validation Checks
File: `design/src/commands/validate.rs`

Replace the existing implementation with comprehensive checks:

```rust
//! Validate command implementation

use anyhow::Result;
use colored::*;
use design::doc::{is_in_state_dir, state_from_directory};
use design::index::DocumentIndex;
use design::index_sync::{get_git_tracked_docs, ParsedIndex};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
enum ValidationIssue {
    DuplicateNumber { number: u32, paths: Vec<String> },
    MissingReference { doc_num: u32, ref_type: String, ref_num: u32 },
    DateOrderIssue { doc_num: u32, created: String, updated: String },
    StateDirectoryMismatch { doc_num: u32, yaml_state: String, dir_state: String, path: String },
    NotInIndex { doc_num: u32, title: String, path: String },
    InIndexNotInGit { number: String, path: String },
    MissingHeaders { path: String },
    InvalidState { doc_num: u32, state: String },
}

impl ValidationIssue {
    fn severity(&self) -> &str {
        match self {
            ValidationIssue::DuplicateNumber { .. } => "ERROR",
            ValidationIssue::MissingReference { .. } => "ERROR",
            ValidationIssue::StateDirectoryMismatch { .. } => "WARNING",
            ValidationIssue::NotInIndex { .. } => "WARNING",
            ValidationIssue::InIndexNotInGit { .. } => "ERROR",
            ValidationIssue::DateOrderIssue { .. } => "WARNING",
            ValidationIssue::MissingHeaders { .. } => "WARNING",
            ValidationIssue::InvalidState { .. } => "ERROR",
        }
    }

    fn description(&self) -> String {
        match self {
            ValidationIssue::DuplicateNumber { number, paths } => {
                format!(
                    "Duplicate number {:04} found in {} files:\n{}",
                    number,
                    paths.len(),
                    paths.iter().map(|p| format!("      {}", p)).collect::<Vec<_>>().join("\n")
                )
            }
            ValidationIssue::MissingReference { doc_num, ref_type, ref_num } => {
                format!(
                    "Document {:04} references non-existent {} {:04}",
                    doc_num, ref_type, ref_num
                )
            }
            ValidationIssue::DateOrderIssue { doc_num, created, updated } => {
                format!(
                    "Document {:04}: created date ({}) is after updated date ({})",
                    doc_num, created, updated
                )
            }
            ValidationIssue::StateDirectoryMismatch { doc_num, yaml_state, dir_state, path } => {
                format!(
                    "Document {:04}: YAML state '{}' doesn't match directory state '{}'\n      Path: {}",
                    doc_num, yaml_state, dir_state, path
                )
            }
            ValidationIssue::NotInIndex { doc_num, title, path } => {
                format!(
                    "Document {:04} '{}' exists but not in index\n      Path: {}",
                    doc_num, title, path
                )
            }
            ValidationIssue::InIndexNotInGit { number, path } => {
                format!(
                    "Index entry {} references non-existent file: {}",
                    number, path
                )
            }
            ValidationIssue::MissingHeaders { path } => {
                format!("Document missing YAML headers: {}", path)
            }
            ValidationIssue::InvalidState { doc_num, state } => {
                format!(
                    "Document {:04} has invalid state: '{}'",
                    doc_num, state
                )
            }
        }
    }

    fn can_auto_fix(&self) -> bool {
        matches!(
            self,
            ValidationIssue::StateDirectoryMismatch { .. }
                | ValidationIssue::NotInIndex { .. }
                | ValidationIssue::MissingHeaders { .. }
        )
    }

    fn fix_description(&self) -> Option<String> {
        match self {
            ValidationIssue::StateDirectoryMismatch { .. } => {
                Some("Run 'oxd sync-location <file>' to fix".to_string())
            }
            ValidationIssue::NotInIndex { .. } => {
                Some("Run 'oxd update-index' to add to index".to_string())
            }
            ValidationIssue::MissingHeaders { .. } => {
                Some("Run 'oxd add-headers <file>' to add headers".to_string())
            }
            _ => None,
        }
    }
}

pub fn validate_documents(index: &DocumentIndex, fix: bool) -> Result<()> {
    println!("\n{}\n", "Validating design documents...".bold());

    let mut issues = Vec::new();

    // Get all documents
    let docs = index.all();

    // Check 1: Duplicate numbers
    let mut number_map: HashMap<u32, Vec<String>> = HashMap::new();
    for doc in &docs {
        let path_str = doc.path.to_string_lossy().to_string();
        number_map
            .entry(doc.metadata.number)
            .or_insert_with(Vec::new)
            .push(path_str);
    }

    for (number, paths) in number_map.iter() {
        if paths.len() > 1 {
            issues.push(ValidationIssue::DuplicateNumber {
                number: *number,
                paths: paths.clone(),
            });
        }
    }

    // Check 2: Supersedes/superseded-by references
    let valid_numbers: HashSet<u32> = docs.iter().map(|d| d.metadata.number).collect();

    for doc in &docs {
        if let Some(supersedes) = doc.metadata.supersedes {
            if !valid_numbers.contains(&supersedes) {
                issues.push(ValidationIssue::MissingReference {
                    doc_num: doc.metadata.number,
                    ref_type: "supersedes".to_string(),
                    ref_num: supersedes,
                });
            }
        }

        if let Some(superseded_by) = doc.metadata.superseded_by {
            if !valid_numbers.contains(&superseded_by) {
                issues.push(ValidationIssue::MissingReference {
                    doc_num: doc.metadata.number,
                    ref_type: "superseded-by".to_string(),
                    ref_num: superseded_by,
                });
            }
        }
    }

    // Check 3: Date ordering
    for doc in &docs {
        if doc.metadata.created > doc.metadata.updated {
            issues.push(ValidationIssue::DateOrderIssue {
                doc_num: doc.metadata.number,
                created: doc.metadata.created.to_string(),
                updated: doc.metadata.updated.to_string(),
            });
        }
    }

    // Check 4: State/directory consistency
    for doc in &docs {
        if let Some(dir_state) = state_from_directory(&doc.path) {
            if doc.metadata.state != dir_state {
                issues.push(ValidationIssue::StateDirectoryMismatch {
                    doc_num: doc.metadata.number,
                    yaml_state: doc.metadata.state.as_str().to_string(),
                    dir_state: dir_state.as_str().to_string(),
                    path: doc.path.to_string_lossy().to_string(),
                });
            }
        }
    }

    // Check 5: Index consistency
    let index_path = PathBuf::from(index.docs_dir()).join("00-index.md");
    if index_path.exists() {
        let index_content = fs::read_to_string(&index_path)?;
        let parsed_index = ParsedIndex::parse(&index_content)?;

        // Check for git files not in index
        let git_docs = get_git_tracked_docs(index.docs_dir())?;
        let indexed_numbers: HashSet<String> =
            parsed_index.table_entries.keys().cloned().collect();

        for doc in &docs {
            let number_str = format!("{:04}", doc.metadata.number);
            if !indexed_numbers.contains(&number_str) {
                issues.push(ValidationIssue::NotInIndex {
                    doc_num: doc.metadata.number,
                    title: doc.metadata.title.clone(),
                    path: doc.path.to_string_lossy().to_string(),
                });
            }
        }

        // Check for index entries without files
        for number in indexed_numbers {
            let num: u32 = number.parse().unwrap_or(0);
            if !valid_numbers.contains(&num) {
                // Find the path from index
                if let Some(entry) = parsed_index.table_entries.get(&number) {
                    issues.push(ValidationIssue::InIndexNotInGit {
                        number: number.clone(),
                        path: format!("(see index for {:04})", num),
                    });
                }
            }
        }
    }

    // Check 6: Files without headers
    let git_docs = get_git_tracked_docs(index.docs_dir())?;
    for path in git_docs {
        if let Ok(content) = fs::read_to_string(&path) {
            if !content.trim_start().starts_with("---\n") {
                issues.push(ValidationIssue::MissingHeaders {
                    path: path.to_string_lossy().to_string(),
                });
            }
        }
    }

    // Display issues
    let mut errors = 0;
    let mut warnings = 0;

    for issue in &issues {
        let severity = issue.severity();
        let color = match severity {
            "ERROR" => "red",
            "WARNING" => "yellow",
            _ => "white",
        };

        println!(
            "{} {}",
            format!("{}:", severity).color(color).bold(),
            issue.description()
        );

        if let Some(fix_msg) = issue.fix_description() {
            println!("    {} {}", "→".cyan(), fix_msg.dimmed());
        }

        println!();

        match severity {
            "ERROR" => errors += 1,
            "WARNING" => warnings += 1,
            _ => {}
        }
    }

    // Summary
    if issues.is_empty() {
        println!("{} All documents valid!", "✓".green().bold());
    } else {
        println!(
            "{} Found {} errors and {} warnings",
            "Summary:".bold(),
            errors,
            warnings
        );

        let auto_fixable = issues.iter().filter(|i| i.can_auto_fix()).count();
        if auto_fixable > 0 && !fix {
            println!(
                "\n{} {} issues can be auto-fixed. Run with {} to fix them.",
                "→".cyan(),
                auto_fixable,
                "--fix".cyan()
            );
        }

        if fix {
            println!("\n{}", "Auto-fixing issues...".bold());
            apply_fixes(index, &issues)?;
        }
    }

    println!();
    Ok(())
}

fn apply_fixes(index: &DocumentIndex, issues: &[ValidationIssue]) -> Result<()> {
    use crate::commands::{add_headers, sync_location, update_index};

    let mut fixed = 0;

    for issue in issues {
        if !issue.can_auto_fix() {
            continue;
        }

        match issue {
            ValidationIssue::StateDirectoryMismatch { path, .. } => {
                println!("  Fixing state/directory mismatch: {}", path);
                if let Err(e) = sync_location(index, path) {
                    eprintln!("    {} Failed: {}", "✗".red(), e);
                } else {
                    println!("    {} Fixed", "✓".green());
                    fixed += 1;
                }
            }
            ValidationIssue::MissingHeaders { path } => {
                println!("  Adding headers: {}", path);
                if let Err(e) = add_headers(path) {
                    eprintln!("    {} Failed: {}", "✗".red(), e);
                } else {
                    println!("    {} Fixed", "✓".green());
                    fixed += 1;
                }
            }
            ValidationIssue::NotInIndex { .. } => {
                // These will be fixed by update-index at the end
            }
            _ => {}
        }
    }

    // Update index to fix NotInIndex issues
    if issues.iter().any(|i| matches!(i, ValidationIssue::NotInIndex { .. })) {
        println!("  Updating index...");
        let updated_index = DocumentIndex::new(index.docs_dir())?;
        if let Err(e) = update_index(&updated_index) {
            eprintln!("    {} Failed: {}", "✗".red(), e);
        } else {
            println!("    {} Fixed", "✓".green());
            fixed += issues.iter().filter(|i| matches!(i, ValidationIssue::NotInIndex { .. })).count();
        }
    }

    println!("\n{} {} issues fixed", "✓".green().bold(), fixed);
    Ok(())
}
```

### Testing Steps for 5.1
1. Create documents with various issues
2. Run validate without --fix
3. Verify all issues detected
4. Run validate with --fix
5. Verify auto-fixable issues corrected
6. Test with duplicate numbers
7. Test with missing references
8. Test with state/directory mismatches

---

## Task 5.2: Command Aliases

### Purpose
Add convenient short aliases for common commands.

### Implementation Steps

#### Step 1: Add Aliases to CLI
File: `design/src/cli.rs`

```rust
#[derive(Subcommand)]
pub enum Commands {
    /// List all design documents
    #[command(visible_alias = "ls")]
    List {
        /// Filter by state (draft, under-review, final, superseded)
        #[arg(short, long)]
        state: Option<String>,

        /// Show full details
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show a specific document
    Show {
        /// Document number
        number: u32,

        /// Show only metadata
        #[arg(short, long)]
        metadata_only: bool,
    },

    /// Create a new design document
    New {
        /// Document title
        title: String,

        /// Author name (defaults to git config user.name)
        #[arg(short, long)]
        author: Option<String>,
    },

    /// Validate all documents
    #[command(visible_alias = "check")]
    Validate {
        /// Fix issues automatically where possible
        #[arg(short, long)]
        fix: bool,
    },

    /// Generate the index file (00-index.md)
    #[command(visible_alias = "gen-index")]
    Index {
        /// Output format (markdown or json)
        #[arg(short, long, default_value = "markdown")]
        format: String,
    },

    /// Add or update YAML frontmatter headers
    #[command(visible_alias = "headers")]
    AddHeaders {
        /// Path to document
        path: String,
    },

    /// Transition document to a new state
    #[command(visible_alias = "mv")]
    Transition {
        /// Path to document
        path: String,

        /// New state (draft, under-review, revised, accepted, active, final, deferred, rejected, withdrawn, superseded)
        state: String,
    },

    /// Move document to directory matching its state header
    #[command(visible_alias = "sync")]
    SyncLocation {
        /// Path to document
        path: String,
    },

    /// Synchronize index with git-tracked documents
    #[command(visible_alias = "sync-index")]
    UpdateIndex,

    /// Add a new document with full processing
    Add {
        /// Path to document file
        path: String,

        /// Show what would be done without making changes
        #[arg(long)]
        dry_run: bool,
    },
}
```

### Testing Steps for 5.2
1. Test `oxd ls` (alias for list)
2. Test `oxd check` (alias for validate)
3. Test `oxd mv` (alias for transition)
4. Test `oxd sync` (alias for sync-location)
5. Verify help text shows aliases
6. Verify original commands still work

---

## Task 5.3: Improved Error Handling

### Purpose
Provide clear, actionable error messages with helpful suggestions.

### Implementation Steps

#### Step 1: Create Error Helper Module
File: `design/src/errors.rs`

```rust
//! Error handling utilities

use colored::*;

/// Print a formatted error message
pub fn print_error(context: &str, error: &anyhow::Error) {
    eprintln!("{} {}", "Error:".red().bold(), context);
    eprintln!("  {}", error.to_string().red());

    // Show chain of causes
    let mut current = error.source();
    while let Some(cause) = current {
        eprintln!("  {} {}", "Caused by:".dimmed(), cause.to_string().dimmed());
        current = cause.source();
    }
}

/// Print an error with a suggestion
pub fn print_error_with_suggestion(context: &str, error: &anyhow::Error, suggestion: &str) {
    print_error(context, error);
    eprintln!("\n{} {}", "Suggestion:".cyan().bold(), suggestion);
}

/// Print a warning message
pub fn print_warning(message: &str) {
    eprintln!("{} {}", "Warning:".yellow().bold(), message);
}
```

#### Step 2: Update lib.rs
File: `design/src/lib.rs`

```rust
pub mod errors;
```

#### Step 3: Use Error Helpers in Main
File: `design/src/main.rs`

Update the main function:

```rust
fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load the document index
    let index = match DocumentIndex::new(&cli.docs_dir) {
        Ok(idx) => idx,
        Err(e) => {
            design::errors::print_error_with_suggestion(
                "Failed to load document index",
                &e,
                "Make sure you're in a valid design docs directory"
            );
            std::process::exit(1);
        }
    };

    // Execute the command
    let result = match cli.command {
        Commands::List { state, verbose } => {
            list_documents(&index, state, verbose)
        }
        Commands::Show { number, metadata_only } => {
            show_document(&index, number, metadata_only)
        }
        Commands::New { title, author } => {
            new_document(&index, title, author)
        }
        Commands::Validate { fix } => {
            validate_documents(&index, fix)
        }
        Commands::Index { format } => {
            generate_index(&index, &format)
        }
        Commands::AddHeaders { path } => {
            add_headers(&path)
        }
        Commands::Transition { path, state } => {
            transition_document(&index, &path, &state)
        }
        Commands::SyncLocation { path } => {
            sync_location(&index, &path)
        }
        Commands::UpdateIndex => {
            update_index(&index)
        }
        Commands::Add { path, dry_run } => {
            add_document(&index, &path, dry_run)
        }
    };

    if let Err(e) = result {
        design::errors::print_error("Command failed", &e);
        std::process::exit(1);
    }

    Ok(())
}
```

### Testing Steps for 5.3
1. Test with missing docs directory
2. Test with invalid file paths
3. Test with invalid states
4. Verify error messages are helpful
5. Verify suggestions appear when appropriate

---

## Task 5.4: Colored Output Refinements

### Purpose
Ensure consistent, attractive colored output throughout the tool.

### Implementation Steps

#### Step 1: Create Color Theme Module
File: `design/src/theme.rs`

```rust
//! Color theme for consistent output

use colored::*;

/// Color for success messages
pub fn success(msg: &str) -> ColoredString {
    msg.green()
}

/// Color for error messages
pub fn error(msg: &str) -> ColoredString {
    msg.red()
}

/// Color for warning messages
pub fn warning(msg: &str) -> ColoredString {
    msg.yellow()
}

/// Color for info messages
pub fn info(msg: &str) -> ColoredString {
    msg.cyan()
}

/// Color for document numbers
pub fn doc_number(num: u32) -> ColoredString {
    format!("{:04}", num).bold()
}

/// Color for state badges
pub fn state_badge(state: &str) -> ColoredString {
    match state.to_lowercase().as_str() {
        "draft" => state.yellow(),
        "under review" => state.cyan(),
        "revised" => state.blue(),
        "accepted" => state.green(),
        "active" => state.green().bold(),
        "final" => state.green().bold(),
        "deferred" => state.magenta(),
        "rejected" => state.red(),
        "withdrawn" => state.red(),
        "superseded" => state.red(),
        _ => state.white(),
    }
}

/// Symbol for success
pub fn success_symbol() -> &'static str {
    "✓"
}

/// Symbol for error
pub fn error_symbol() -> &'static str {
    "✗"
}

/// Symbol for warning
pub fn warning_symbol() -> &'static str {
    "⚠"
}

/// Symbol for info
pub fn info_symbol() -> &'static str {
    "→"
}
```

#### Step 2: Update lib.rs
File: `design/src/lib.rs`

```rust
pub mod theme;
```

#### Step 3: Update List Command to Use Theme
File: `design/src/commands/list.rs`

```rust
use design::theme;

// In list_documents function, replace color usage:
println!("{} {} [{}]",
    theme::doc_number(doc.metadata.number),
    doc.metadata.title,
    theme::state_badge(doc.metadata.state.as_str())
);
```

### Testing Steps for 5.4
1. Run all commands and verify colors are consistent
2. Test with different terminal backgrounds
3. Verify state badges use appropriate colors
4. Test with colored output disabled (NO_COLOR env var)

---

## Task 5.5: Documentation

### Purpose
Provide comprehensive documentation for users.

### Implementation Steps

#### Step 1: Create README
File: `design/README.md`

```markdown
# Oxur Design Documentation Manager

A command-line tool for managing design documents with YAML frontmatter, git integration, and automatic indexing.

## Installation

```bash
cd design
cargo build --release
```

The binary will be at `target/release/oxd`.

## Quick Start

```bash
# List all documents
oxd list

# Create a new document
oxd new "My Feature Design"

# Add an existing document
oxd add path/to/document.md

# Transition a document to review
oxd transition docs/01-draft/0001-my-feature.md "under review"

# Validate all documents
oxd validate

# Update the index
oxd update-index
```

## Commands

### `oxd list` (alias: `ls`)
List all design documents, optionally filtered by state.

```bash
# List all documents
oxd list

# List only drafts
oxd list --state draft

# Show full details
oxd list --verbose
```

### `oxd show <number>`
Display a specific document by number.

```bash
# Show document with full content
oxd show 42

# Show only metadata
oxd show 42 --metadata-only
```

### `oxd new <title>`
Create a new design document from template.

```bash
# Create with auto-detected author
oxd new "Feature Name"

# Specify author
oxd new "Feature Name" --author "Alice"
```

### `oxd add <path>`
Add a document with full processing (numbering, headers, git staging).

```bash
# Add a document
oxd add ~/Downloads/new-design.md

# Preview what would happen
oxd add ~/Downloads/new-design.md --dry-run
```

### `oxd add-headers <path>` (alias: `headers`)
Add or update YAML frontmatter headers.

```bash
oxd add-headers docs/01-draft/0001-feature.md
```

### `oxd transition <path> <state>` (alias: `mv`)
Transition a document to a new state.

```bash
oxd transition docs/01-draft/0001-feature.md "under review"
```

Valid states:
- draft
- under-review (or "under review")
- revised
- accepted
- active
- final
- deferred
- rejected
- withdrawn
- superseded

### `oxd sync-location <path>` (alias: `sync`)
Move document to match its YAML state header.

```bash
oxd sync-location docs/wrong-dir/0001-feature.md
```

### `oxd validate` (alias: `check`)
Validate all documents for consistency.

```bash
# Check for issues
oxd validate

# Auto-fix issues where possible
oxd validate --fix
```

### `oxd update-index` (alias: `sync-index`)
Synchronize the index with git-tracked documents.

```bash
oxd update-index
```

### `oxd index`
Generate the index file.

```bash
# Generate markdown index
oxd index

# Generate JSON index
oxd index --format json
```

## Document States

Documents progress through these states:

1. **Draft** - Initial work in progress
2. **Under Review** - Ready for team review
3. **Revised** - Revisions made after review
4. **Accepted** - Approved by team
5. **Active** - Currently being implemented
6. **Final** - Implementation complete
7. **Deferred** - Postponed for later
8. **Rejected** - Not approved
9. **Withdrawn** - Author withdrew proposal
10. **Superseded** - Replaced by newer document

## Document Structure

Each document should have YAML frontmatter:

```yaml
---
number: 1
title: "Feature Name"
author: Alice Smith
created: 2024-01-15
updated: 2024-01-20
state: Draft
supersedes: null
superseded-by: null
---

# Feature Name

## Overview
...
```

## Directory Structure

```
docs/
├── 00-index.md                    # Auto-generated index
├── 01-draft/                      # Draft documents
├── 02-under-review/               # Documents under review
├── 03-revised/                    # Revised documents
├── 04-accepted/                   # Accepted documents
├── 05-active/                     # Active implementation
├── 06-final/                      # Final documents
├── 07-deferred/                   # Deferred documents
├── 08-rejected/                   # Rejected documents
├── 09-withdrawn/                  # Withdrawn documents
└── 10-superseded/                 # Superseded documents
```

## Workflow Examples

### Creating a New Design

```bash
# 1. Create from template
oxd new "Authentication System"

# 2. Edit the document
vim docs/01-draft/0001-authentication-system.md

# 3. When ready for review
oxd transition docs/01-draft/0001-authentication-system.md "under review"

# 4. Update index
oxd update-index
```

### Adding an Existing Document

```bash
# Add document with full processing
oxd add ~/Documents/my-design.md

# The tool will:
# - Assign number (e.g., 0042)
# - Move to project
# - Place in draft directory
# - Add YAML headers
# - Stage with git
# - Update index
```

### Bulk Operations

```bash
# After manually moving files
git mv 01-draft/*.md 02-under-review/

# Fix YAML states to match new location
for file in 02-under-review/*.md; do
    oxd sync-location "$file"
done

# Update index
oxd update-index
```

## Troubleshooting

### "Failed to load document index"
Make sure you're in a directory with design docs or specify the docs directory:
```bash
oxd --docs-dir path/to/docs list
```

### State/Directory Mismatch
Run `oxd validate --fix` to automatically correct mismatches.

### Document Not in Index
Run `oxd update-index` to sync the index.

### Git Errors
Ensure you're in a git repository and have committed the docs directory.

## Tips

- Use tab completion for file paths
- Run `oxd validate` before committing
- Use `--dry-run` with `add` to preview changes
- Aliases make common commands faster (`ls`, `mv`, `sync`)
```

#### Step 2: Create CHANGELOG
File: `design/CHANGELOG.md`

```markdown
# Changelog

All notable changes to this project will be documented in this file.

## [1.0.0] - 2024-XX-XX

### Added
- Initial release
- Document listing with state filtering
- Document creation from template
- Full document addition workflow with `add` command
- State transitions with git history preservation
- YAML frontmatter management
- Automatic index generation and synchronization
- Comprehensive validation with auto-fix
- Git integration for metadata extraction
- Command aliases for common operations
- Colored output with consistent theme
- Dry-run support for previewing changes

### Features
- 10 document states (draft through superseded)
- Automatic document numbering
- Git-based author and date extraction
- Index table and state sections
- Supersedes/superseded-by tracking
- State/directory consistency checking
- Flexible state name parsing (hyphens, spaces, case-insensitive)

### Commands
- `list` - List all documents
- `show` - Show specific document
- `new` - Create new document
- `add` - Add document with full processing
- `add-headers` - Add/update YAML headers
- `transition` - Change document state
- `sync-location` - Fix directory/header mismatch
- `validate` - Validate all documents
- `update-index` - Sync index with filesystem
- `index` - Generate index file
```

#### Step 3: Add Examples Directory
Create: `design/examples/`

```bash
mkdir -p design/examples
```

File: `design/examples/sample-document.md`

```markdown
---
number: 1
title: "Sample Design Document"
author: Alice Smith
created: 2024-01-15
updated: 2024-01-20
state: Draft
supersedes: null
superseded-by: null
---

# Sample Design Document

## Overview

This is a sample design document showing the expected structure.

## Background

Provide context and motivation for this design.

## Proposal

Detailed description of the proposed design:

- Key feature 1
- Key feature 2
- Key feature 3

## Alternatives Considered

What other approaches were considered and why were they rejected?

## Implementation Plan

1. Phase 1: Foundation
2. Phase 2: Core features
3. Phase 3: Polish

## Open Questions

- Question 1?
- Question 2?

## Success Criteria

How will we know this design is successful?

- Metric 1
- Metric 2
```

#### Step 4: Update Help Text
File: `design/src/cli.rs`

Add long_about to commands:

```rust
#[derive(Parser)]
#[command(name = "oxd")]
#[command(about = "Oxur Design Documentation Manager", long_about = None)]
#[command(after_help = "Use 'oxd <command> --help' for more information about a command.")]
pub struct Cli {
    // ... existing fields
}
```

### Testing Steps for 5.5
1. Read through README for clarity
2. Test all examples in README
3. Verify help text is helpful
4. Check that sample document parses correctly

---

## Task 5.6: Final Quality Checks

### Purpose
Ensure the tool is polished and production-ready.

### Implementation Steps

#### Step 1: Add Version Information
File: `design/Cargo.toml`

```toml
[package]
name = "oxd"
version = "1.0.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "Design documentation manager with git integration"
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/oxur"
```

#### Step 2: Add Build Script (Optional)
File: `design/build.rs`

```rust
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Could add build-time validation, asset generation, etc.
}
```

#### Step 3: Create Install Script
File: `design/install.sh`

```bash
#!/bin/bash
set -e

echo "Building oxd..."
cargo build --release

echo "Installing to /usr/local/bin..."
sudo cp target/release/oxd /usr/local/bin/

echo "✓ oxd installed successfully!"
echo "Run 'oxd --help' to get started"
```

#### Step 4: Add Bash Completion (Optional)
File: `design/completions/oxd.bash`

```bash
# Bash completion for oxd
# Source this file or install to /etc/bash_completion.d/

_oxd() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    # Top-level commands
    if [ $COMP_CWORD -eq 1 ]; then
        opts="list show new add add-headers transition sync-location validate update-index index help"
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
        return 0
    fi
    
    # Command-specific completions
    case "${prev}" in
        --state|-s)
            opts="draft under-review revised accepted active final deferred rejected withdrawn superseded"
            COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
            return 0
            ;;
        --format|-f)
            opts="markdown json"
            COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
            return 0
            ;;
        transition|mv)
            # First arg is file, second is state
            if [ $COMP_CWORD -eq 3 ]; then
                opts="draft under-review revised accepted active final deferred rejected withdrawn superseded"
                COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
            else
                COMPREPLY=( $(compgen -f -- ${cur}) )
            fi
            return 0
            ;;
        *)
            # File completion by default
            COMPREPLY=( $(compgen -f -- ${cur}) )
            return 0
            ;;
    esac
}

complete -F _oxd oxd
```

### Testing Steps for 5.6
1. Verify version displays correctly
2. Test install script
3. Test bash completion if added
4. Run `cargo clippy` for linting
5. Run `cargo test` for tests
6. Test on clean system

---

## Verification Checklist

Final verification before release:

- [ ] All commands work correctly
- [ ] Help text is clear and helpful
- [ ] Error messages are actionable
- [ ] Colors are consistent and appropriate
- [ ] Validation catches all issues
- [ ] Auto-fix works correctly
- [ ] Command aliases work
- [ ] README is comprehensive
- [ ] Examples are accurate
- [ ] Installation works
- [ ] No compiler warnings
- [ ] Clippy passes
- [ ] All tests pass
- [ ] Documentation is up to date
- [ ] CHANGELOG is complete

---

## Performance Testing

Test with repositories of various sizes:

### Small (< 10 docs)
- All operations should be instant
- No noticeable delay

### Medium (10-50 docs)
- List: < 100ms
- Validate: < 500ms
- Update-index: < 1s

### Large (50-100+ docs)
- List: < 500ms
- Validate: < 2s
- Update-index: < 3s

If performance is slow, consider:
- Caching parsed documents
- Parallel processing
- Lazy loading

---

## Final Polish Items

### Code Quality
```bash
# Run clippy
cargo clippy --all-targets --all-features

# Run tests
cargo test

# Check formatting
cargo fmt -- --check

# Build docs
cargo doc --no-deps
```

### User Experience
- [ ] Consistent terminology throughout
- [ ] No jargon without explanation
- [ ] Commands feel intuitive
- [ ] Output is scannable
- [ ] Progress is visible for long operations

### Maintainability
- [ ] Code is well-commented
- [ ] Functions are focused
- [ ] Error handling is comprehensive
- [ ] Tests cover key functionality
- [ ] Documentation matches implementation

---

## Release Checklist

1. [ ] Update version in Cargo.toml
2. [ ] Update CHANGELOG.md
3. [ ] Update README.md with any new features
4. [ ] Run full test suite
5. [ ] Run clippy and fix warnings
6. [ ] Build release binary
7. [ ] Test release binary on clean system
8. [ ] Create git tag
9. [ ] Push to repository
10. [ ] Celebrate! 🎉

---

## Future Enhancements (Post-Release)

Ideas for future versions:

### v1.1
- Web UI for browsing documents
- Export to PDF/HTML
- Document templates beyond basic
- Search within documents

### v1.2
- Integration with GitHub/GitLab
- Review workflow automation
- Email notifications
- Approval tracking

### v2.0
- Multiple projects support
- Custom state workflows
- Plugins/extensions
- REST API

---

## Notes for Claude Code

- This is the final phase - make it shine!
- Focus on user experience
- Test thoroughly
- Document everything
- Make error messages helpful
- Polish the rough edges
- Ensure consistency throughout
- Add those final touches that show care
