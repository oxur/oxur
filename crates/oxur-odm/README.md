# ODD Document Manager

A command-line tool for managing design documents with YAML frontmatter, git integration, and automatic indexing.

## Installation

```bash
cd design
cargo build --release
```

The binary will be at `target/release/odm`.

## Quick Start

```bash
# List all documents
odm list

# Create a new document
odm new "My Feature Design"

# Add an existing document
odm add path/to/document.md

# Transition a document to review
odm transition docs/01-draft/0001-my-feature.md "under review"

# Validate all documents
odm validate

# Update the index
odm update-index
```

## Commands

### `odm list` (alias: `ls`)
List all design documents, optionally filtered by state.

```bash
# List all documents
odm list

# List only drafts
odm list --state draft

# Show full details
odm list --verbose
```

### `odm show <number>`
Display a specific document by number.

```bash
# Show document with full content
odm show 42

# Show only metadata
odm show 42 --metadata-only
```

### `odm new <title>`
Create a new design document from template.

```bash
# Create with auto-detected author
odm new "Feature Name"

# Specify author
odm new "Feature Name" --author "Alice"
```

### `odm add <path>`
Add a document with full processing (numbering, headers, git staging).

```bash
# Add a document
odm add ~/Downloads/new-design.md

# Preview what would happen
odm add ~/Downloads/new-design.md --dry-run
```

### `odm add-headers <path>` (alias: `headers`)
Add or update YAML frontmatter headers.

```bash
odm add-headers docs/01-draft/0001-feature.md
```

### `odm transition <path> <state>` (alias: `mv`)
Transition a document to a new state.

```bash
odm transition docs/01-draft/0001-feature.md "under review"
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

### `odm sync-location <path>` (alias: `sync`)
Move document to match its YAML state header.

```bash
odm sync-location docs/wrong-dir/0001-feature.md
```

### `odm validate` (alias: `check`)
Validate all documents for consistency.

```bash
# Check for issues
odm validate

# Auto-fix issues where possible
odm validate --fix
```

### `odm update-index` (alias: `sync-index`)
Synchronize the index with documents on filesystem.

```bash
odm update-index
```

### `odm index`
Generate the index file.

```bash
# Generate markdown index
odm index

# Generate JSON index
odm index --format json
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
odm new "Authentication System"

# 2. Edit the document
vim docs/01-draft/0001-authentication-system.md

# 3. When ready for review
odm transition docs/01-draft/0001-authentication-system.md "under review"

# 4. Update index
odm update-index
```

### Adding an Existing Document

```bash
# Add document with full processing
odm add ~/Documents/my-design.md

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
    odm sync-location "$file"
done

# Update index
odm update-index
```

## Troubleshooting

### "Failed to load document index"
Make sure you're in a directory with design docs or specify the docs directory:
```bash
odm --docs-dir path/to/docs list
```

### State/Directory Mismatch
Run `odm validate --fix` to automatically correct mismatches.

### Document Not in Index
Run `odm update-index` to sync the index.

### Git Errors
Ensure you're in a git repository and have committed the docs directory.

## Tips

- Use tab completion for file paths
- Run `odm validate` before committing
- Use `--dry-run` with `add` to preview changes
- Aliases make common commands faster (`ls`, `mv`, `sync`)
