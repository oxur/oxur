---
number: 10
title: "Oxur Design Documentation CLI - Phases 9-10 Build Plan"
author: "Duncan McGreggor & Claude"
component: Design
tags: [cli, tooling]
created: 2025-12-27
updated: 2025-12-27
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# Oxur Design Documentation CLI - Phases 9-10 Build Plan

## Executive Summary

This document outlines the next phase of development for the `oxd` CLI tool, focusing on:

- **Phase 9**: Advanced document lifecycle management (replace, remove, list removed)
- **Phase 10**: Tool introspection and discovery features (info command suite)

These phases build upon the solid foundation established in Phases 1-8, adding sophisticated document replacement workflows and comprehensive self-documentation capabilities.

## Background

After implementing core functionality (Phases 1-5), establishing canonical data sources (Phase 6), building advanced onboarding (Phase 7), and adding debug/search capabilities (Phase 8), we identified two key areas for enhancement:

1. **Document Lifecycle Gaps**: Need better handling for document replacement and removal
2. **Discoverability**: Users need easy ways to discover valid states, supported fields, and configuration

---

## Phase 9: Advanced Document Lifecycle Management

### Problem Statement

Current limitations:

- No way to replace an existing document while preserving its ID and history
- Document "removal" means deletion, with no recovery option
- Can't easily track what's been removed vs permanently deleted
- No distinction between "superseded" (both versions kept) and "replaced/overwritten" (old version archived)

### Goals

1. Enable document replacement with ID/number preservation
2. Implement safe removal with dustbin recovery mechanism
3. Add new "overwritten" state for replaced documents
4. Provide visibility into removed documents and deletion status

---

## Task 9.1: Implement Replace Command

### Purpose

Allow users to replace an existing document with new content while preserving the document ID and selected metadata.

### Command Signature

```bash
oxd replace <old-id-or-filename> <new-filename>
```

### Examples

```bash
# Replace by ID
oxd replace 42 new-feature-design.md

# Replace by filename
oxd replace 0042-old-feature.md new-feature-design.md
```

### Behavior Specification

#### Step 1: Validate Inputs

- Verify old document exists in data sources
- Verify new file exists on filesystem
- Verify new file is not already in the project
- Parse and validate new file content

#### Step 2: Preserve Critical Metadata

From old document, preserve:

- `number` - Document ID (most important!)
- `created` - Original creation date (required)
- Any other metadata fields that exist in old but not in new

From new document, use:

- `title` - New title
- `author` - New author (or old if missing)
- `state` - Set to "draft" initially (user can transition later)
- `updated` - Set to current date
- All other present metadata fields

#### Step 3: Handle Old Document

- Change old document's state to "overwritten"
- Add UUIDv4 suffix to filename: `0042-old-feature-{uuid}.md`
- Move to dustbin directory (preserve state subdirectory structure)
- Update data sources to reflect new state and location

#### Step 4: Install New Document

- Rename new file with old document's number: `0042-old-feature.md`
- Update frontmatter with merged metadata
- Move to appropriate state directory (likely draft)
- Update data sources with new document info
- Create superseded-by link from old → new (optional, configurable)

#### Step 5: Git Operations

```bash
git mv old-location .dustbin/XX-state/0042-old-feature-{uuid}.md
git add new-location/0042-old-feature.md
```

### New State: "overwritten"

Add to `DocState` enum:

```rust
pub enum DocState {
    // ... existing states ...
    Overwritten,  // Document has been replaced by a new version
}
```

Directory mapping: `.dustbin/overwritten/` or integrated into state-specific dustbin subdirs

### Implementation Files

- `design/src/commands/replace.rs` - New command implementation
- `design/src/state.rs` - Add Overwritten state
- `design/src/doc.rs` - Metadata merging logic
- Update CLI enum in `design/src/cli.rs`

### Error Handling

- Old document not found
- New file not found
- New file already has frontmatter with different number
- Validation failures on new document
- Git operation failures
- Dustbin directory creation failures

### Output Example

```
Replacing document 0042: "Old Feature Design"

✓ Validated new document: new-feature-design.md
✓ Preserved metadata:
  - number: 42
  - created: 2024-03-15
  - tags: [backend, api]
✓ Moved old version to dustbin:
  .dustbin/04-accepted/0042-old-feature-a3f5c9e1.md
✓ Installed new version:
  01-draft/0042-old-feature.md
✓ Updated data sources
✓ Staged changes with git

Replacement complete! Use 'oxd show 42' to view the new document.
```

---

## Task 9.2: Implement Remove Command

### Purpose

Safely remove documents from active use while preserving them in a dustbin directory for potential recovery.

### Command Signature

```bash
oxd remove <id-or-filename>
```

### Examples

```bash
# Remove by ID
oxd remove 42

# Remove by filename
oxd remove 0042-obsolete-feature.md
```

### Configuration

Add to `Cargo.toml`:

```toml
[package.metadata.oxd]
dustbin_directory = "./design/docs/.dustbin"
```

Or create dedicated config file `.oxd/config.toml`:

```toml
[dustbin]
directory = "./design/docs/.dustbin"
preserve_structure = true  # Keep state subdirectories
```

### Behavior Specification

#### Step 1: Validate Input

- Verify document exists in data sources
- Check current state (warn if already removed)
- Confirm document file exists on filesystem

#### Step 2: Prepare Dustbin

- Read dustbin_directory from config
- Create dustbin directory if it doesn't exist
- Create state-specific subdirectory if preserve_structure=true

#### Step 3: Generate Unique Filename

- Extract base filename and number
- Generate UUIDv4
- Create new name: `{number}-{slug}-{uuid}.md`
- Example: `0042-feature-design-7f3e9a12.md`

#### Step 4: Move to Dustbin

```bash
# From: 04-accepted/0042-feature-design.md
# To: .dustbin/04-accepted/0042-feature-design-7f3e9a12.md
```

#### Step 5: Update Data Sources

- Change document state to "removed"
- Update file path to dustbin location
- Set `updated` to current date
- Preserve all other metadata

#### Step 6: Git Operations

```bash
git mv 04-accepted/0042-feature-design.md .dustbin/04-accepted/0042-feature-design-7f3e9a12.md
```

### Dustbin Directory Structure

```
.dustbin/
├── 01-draft/
│   ├── 0003-early-idea-a1b2c3d4.md
│   └── 0007-abandoned-concept-e5f6g7h8.md
├── 04-accepted/
│   ├── 0042-obsolete-feature-7f3e9a12.md
│   └── 0042-obsolete-feature-9a8b7c6d.md  # Same doc removed twice
└── overwritten/
    └── 0042-old-feature-a3f5c9e1.md
```

### New State: "removed"

Add to `DocState` enum (if not already present):

```rust
pub enum DocState {
    // ... existing states ...
    Removed,  // Document has been removed from active use
}
```

### Implementation Files

- `design/src/commands/remove.rs` - New command implementation
- `design/src/state.rs` - Ensure Removed state exists
- `design/src/config.rs` - Configuration loading
- Update CLI enum in `design/src/cli.rs`

### Output Example

```
Removing document 0042: "Obsolete Feature"

✓ Current state: accepted
✓ Created dustbin directory: .dustbin/04-accepted/
✓ Generated unique name: 0042-obsolete-feature-7f3e9a12.md
✓ Moved to dustbin:
  .dustbin/04-accepted/0042-obsolete-feature-7f3e9a12.md
✓ Updated state to: removed
✓ Updated data sources
✓ Staged with git

Document removed! To recover: oxd restore 42
(Note: restore command not yet implemented)
```

---

## Task 9.3: Enhance List Command for Removed Documents

### Purpose

Provide visibility into removed documents and track whether they've been permanently deleted from the dustbin.

### Command Enhancement

```bash
# List only removed documents
oxd list --removed

# Verbose mode shows full paths
oxd list --removed --verbose
```

### Output Format

#### Basic Output

```
Removed Documents:

Number | Title                  | Removed    | Deleted
-------|------------------------|------------|--------
0023   | Old API Design         | 2024-11-15 | false
0042   | Obsolete Feature       | 2025-01-10 | false
0099   | Deprecated Component   | 2024-08-30 | true
0101   | Failed Experiment      | 2024-09-12 | true

Total: 4 removed (2 in dustbin, 2 deleted)
```

#### Verbose Output

```
Removed Documents:

Number | Title                  | Removed    | Deleted | Dustbin Location
-------|------------------------|------------|---------|------------------
0023   | Old API Design         | 2024-11-15 | false   | .dustbin/04-accepted/0023-old-api-7f3e9a12.md
0042   | Obsolete Feature       | 2025-01-10 | false   | .dustbin/04-accepted/0042-obsolete-feature-7f3e9a12.md
0099   | Deprecated Component   | 2024-08-30 | true    | (file not found)
0101   | Failed Experiment      | 2024-09-12 | true    | (file not found)
```

### Behavior Specification

#### Step 1: Query Data Sources

- Filter documents with state="removed"
- Extract metadata: number, title, updated date (as "removed" date)
- Get dustbin file path from data sources

#### Step 2: Check Filesystem

For each removed document:

- Check if file exists at dustbin location
- Set `deleted` = true if file not found
- Set `deleted` = false if file exists

#### Step 3: Format and Display

- Sort by number (ascending)
- Display table with columns: Number, Title, Removed, Deleted
- Optionally show dustbin path if --verbose
- Show summary count

### Color Coding

- `deleted: true` → Red text
- `deleted: false` → Green text (recoverable)
- Header row → Cyan bold

### Implementation Files

- `design/src/commands/list.rs` - Enhance existing list command
- Add `--removed` flag to CLI args
- Add deleted status checking logic

### Edge Cases

- No removed documents: Display "No removed documents found."
- Dustbin directory doesn't exist: All show as deleted=true
- Multiple versions in dustbin: Show most recent only, or all with verbose

---

## Phase 10: Tool Introspection & Discovery

### Problem Statement

Current challenges:

- Users don't know what states are valid without reading docs/code
- Frontmatter field requirements are unclear
- Configuration values are hidden in code
- No easy way to see project statistics
- Tool capabilities aren't discoverable

### Goals

1. Make all valid states discoverable via CLI
2. Document all supported frontmatter fields in-tool
3. Expose configuration values for inspection
4. Provide project statistics and health metrics
5. Create comprehensive help system beyond `--help`

---

## Task 10.1: Create Info Command Framework

### Purpose

Establish a command structure for tool introspection and self-documentation.

### Command Structure

```bash
oxd info [SUBCOMMAND]

Subcommands:
  states     List all valid document states
  fields     Show supported frontmatter fields
  config     Display current configuration
  stats      Show project statistics
  dirs       Show directory structure
  (default)  Show general tool information
```

### Implementation Files

- `design/src/commands/info.rs` - New command module
- Add Info enum to CLI in `design/src/cli.rs`
- Create subcommand routing logic

---

## Task 10.2: Implement `oxd info` (Default)

### Purpose

Provide high-level overview of the tool and quick links to other info.

### Output Example

```
Oxur Design Documentation Tool (oxd) v0.3.0

Project: /home/user/project/design
Documents: 42 total
  - 3 draft
  - 5 under-review
  - 30 accepted
  - 2 removed
  - 2 superseded

Quick Help:
  oxd help              Full command reference
  oxd info states       Valid document states
  oxd info fields       Frontmatter fields
  oxd info config       Configuration values
  oxd info stats        Project statistics

Documentation: https://github.com/yourusername/oxur
```

### Implementation

- Count documents by state from data sources
- Display version from Cargo.toml
- Show project root path
- Provide navigation to other info commands

---

## Task 10.3: Implement `oxd info states`

### Purpose

List all valid document states with descriptions.

### Output Example

```
Valid Document States:

  draft          Initial state for new documents
                 Directory: 01-draft/

  under-review   Document is being reviewed
                 Directory: 02-under-review/

  revised        Document has been revised after review
                 Directory: 03-revised/

  accepted       Document has been accepted
                 Directory: 04-accepted/

  active         Document is actively being implemented
                 Directory: 05-active/

  final          Document is complete and final
                 Directory: 06-final/

  deferred       Document is deferred for future consideration
                 Directory: 07-deferred/

  rejected       Document has been rejected
                 Directory: 08-rejected/

  withdrawn      Document has been withdrawn by author
                 Directory: 09-withdrawn/

  superseded     Document has been replaced by a newer version
                 Directory: 10-superseded/

  removed        Document has been removed from active use
                 Directory: .dustbin/ (various subdirs)

  overwritten    Document was replaced via 'oxd replace'
                 Directory: .dustbin/overwritten/

Transition a document: oxd transition <doc> <state>
```

### Implementation

- Iterate through DocState enum variants
- Display state name in lowercase with hyphens
- Show description for each state
- Show directory mapping
- Include usage hint at bottom

---

## Task 10.4: Implement `oxd info fields`

### Purpose

Document all supported frontmatter fields with examples.

### Output Example

```
Supported Frontmatter Fields:

Required Fields:
  number         Document number (4-digit integer)
                 Example: 42

  title          Document title
                 Example: "Feature Design: Advanced Caching"

  state          Current document state
                 Example: draft
                 Valid states: oxd info states

  created        Creation date (YYYY-MM-DD)
                 Example: 2025-01-15
                 Note: Auto-extracted from git if missing

  updated        Last update date (YYYY-MM-DD)
                 Example: 2025-01-20
                 Note: Auto-updated on transitions

  author         Document author name
                 Example: "Jane Developer"
                 Note: Auto-extracted from git if missing

Optional Fields:
  supersedes     Number of document this supersedes
                 Example: 41
                 Note: Used when document replaces another

  superseded-by  Number of document that supersedes this
                 Example: 43
                 Note: Auto-set when document is superseded

  tags           List of tags for categorization
                 Example: [backend, performance, api]

Example Document Header:
  ---
  number: 42
  title: "Feature Design: Advanced Caching"
  state: draft
  created: 2025-01-15
  updated: 2025-01-20
  author: "Jane Developer"
  tags: [backend, performance]
  ---

Commands:
  oxd add-headers <doc>     Add missing headers to a document
  oxd validate              Check all documents for valid headers
```

### Implementation

- Group fields by required/optional
- Show field name, type, and description
- Provide examples for each field
- Show complete example header
- Link to related commands

---

## Task 10.5: Implement `oxd info config`

### Purpose

Display current configuration values and locations.

### Output Example

```
Configuration:

Project:
  Root:          /home/user/project
  Docs Directory: ./design/docs

Data Sources:
  Index File:    ./design/docs/00-index.md
  Database File: ./design/docs/.oxd-db.json

Templates:
  Template Dir:  ./design/docs/templates
  Default:       design-doc-template.md

Dustbin:
  Directory:     ./design/docs/.dustbin
  Structure:     preserve_state_dirs

State Directories:
  draft          → 01-draft
  under-review   → 02-under-review
  revised        → 03-revised
  accepted       → 04-accepted
  active         → 05-active
  final          → 06-final
  deferred       → 07-deferred
  rejected       → 08-rejected
  withdrawn      → 09-withdrawn
  superseded     → 10-superseded

Configuration Sources:
  1. Cargo.toml [package.metadata.oxd]
  2. .oxd/config.toml (if exists)
  3. Built-in defaults

Modify Configuration:
  Edit: Cargo.toml or create .oxd/config.toml
  Reload: Configuration is read on each command
```

### Implementation

- Read from all configuration sources
- Display merged/resolved values
- Show which config takes precedence
- Include directory mappings
- Provide guidance on modifying config

---

## Task 10.6: Implement `oxd info stats`

### Purpose

Provide statistics and health metrics for the project.

### Output Example

```
Project Statistics:

Document Counts:
  Total Documents:        42

  By State:
    Draft:                3 docs
    Under Review:         5 docs
    Revised:              1 doc
    Accepted:            30 docs
    Active:               0 docs
    Final:                0 docs
    Removed:              2 docs
    Superseded:           1 doc

Activity:
  Created Today:          1
  Updated This Week:      7
  Updated This Month:    15

Timeline:
  Oldest Document:        0001 (2024-01-15)
  Newest Document:        0042 (2025-01-20)
  Average Age:            147 days

Data Sources:
  Index Entries:         42
  Database Records:      42
  Files on Disk:         40 (2 in dustbin)

Health:
  ✓ Index synchronized
  ✓ Database synchronized
  ✓ All files have valid headers
  ⚠ 2 documents in dustbin (consider permanent deletion)
```

### Implementation

- Query data sources for all documents
- Count by state
- Calculate activity metrics (today, week, month)
- Find oldest/newest documents
- Compare data source consistency
- Provide health checks and warnings

---

## Task 10.7: Implement `oxd info dirs` (Optional)

### Purpose

Show the project directory structure.

### Output Example

```
Directory Structure:

design/
├── docs/
│   ├── 00-index.md                (project index)
│   ├── .oxd-db.json              (database)
│   ├── .dustbin/                 (removed documents)
│   │   ├── 04-accepted/
│   │   └── overwritten/
│   ├── 01-draft/                 (3 docs)
│   ├── 02-under-review/          (5 docs)
│   ├── 03-revised/               (1 doc)
│   ├── 04-accepted/              (30 docs)
│   ├── 05-active/                (0 docs)
│   ├── 06-final/                 (0 docs)
│   ├── 07-deferred/              (0 docs)
│   ├── 08-rejected/              (0 docs)
│   ├── 09-withdrawn/             (0 docs)
│   ├── 10-superseded/            (1 doc)
│   └── templates/
│       └── design-doc-template.md
└── Cargo.toml

Document Distribution:
  ████████████████████████████████████ 30  accepted
  ███████                               5  under-review
  ███                                   3  draft
  █                                     1  revised
  █                                     1  superseded
  █                                     2  removed
```

### Implementation

- Scan project directory structure
- Count documents in each state directory
- Display as tree with counts
- Optional: Show visual distribution chart
- Handle missing directories gracefully

---

## Configuration Management Enhancement

### Task 10.8: Formalize Configuration System

### Current Approach

Configuration is currently hardcoded or partially implemented.

### New Approach: Layered Configuration

#### Layer 1: Built-in Defaults

```rust
// src/config.rs
pub struct Config {
    pub docs_directory: PathBuf,
    pub index_file: PathBuf,
    pub database_file: PathBuf,
    pub dustbin_directory: PathBuf,
    pub template_directory: PathBuf,
    pub preserve_dustbin_structure: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            docs_directory: PathBuf::from("./design/docs"),
            index_file: PathBuf::from("./design/docs/00-index.md"),
            database_file: PathBuf::from("./design/docs/.oxd-db.json"),
            dustbin_directory: PathBuf::from("./design/docs/.dustbin"),
            template_directory: PathBuf::from("./design/docs/templates"),
            preserve_dustbin_structure: true,
        }
    }
}
```

#### Layer 2: Cargo.toml Metadata

```toml
[package.metadata.oxd]
docs_directory = "./design/docs"
dustbin_directory = "./design/docs/.dustbin"
preserve_dustbin_structure = true
```

#### Layer 3: Dedicated Config File (Optional)

```toml
# .oxd/config.toml
[paths]
docs = "./design/docs"
index = "./design/docs/00-index.md"
database = "./design/docs/.oxd-db.json"
dustbin = "./design/docs/.dustbin"
templates = "./design/docs/templates"

[dustbin]
preserve_structure = true

[behavior]
auto_stage_git = true
require_confirmation = false
```

### Configuration Loading Logic

```rust
impl Config {
    pub fn load() -> Result<Self> {
        // Start with defaults
        let mut config = Config::default();

        // Override with Cargo.toml if present
        if let Some(cargo_config) = load_cargo_metadata()? {
            config.merge(cargo_config);
        }

        // Override with .oxd/config.toml if present
        if let Some(file_config) = load_config_file()? {
            config.merge(file_config);
        }

        Ok(config)
    }
}
```

### Implementation Files

- `design/src/config.rs` - New configuration module
- Update all commands to use Config instead of hardcoded paths

---

## Testing Strategy

### Phase 9 Tests

#### Replace Command Tests

- Replace with valid new document
- Replace preserves critical metadata
- Replace moves old to dustbin with UUID
- Replace updates data sources
- Replace handles missing metadata fields
- Replace rejects invalid new document
- Replace handles git operation failures

#### Remove Command Tests

- Remove moves to dustbin
- Remove adds UUID suffix
- Remove preserves state subdirectory
- Remove updates data sources
- Remove handles missing dustbin directory
- Remove handles git operation failures
- Remove handles duplicate removals (same doc twice)

#### List Removed Tests

- List shows only removed documents
- List accurately detects deleted status
- List handles missing dustbin directory
- List handles multiple versions in dustbin
- List verbose shows full paths

### Phase 10 Tests

#### Info Command Tests

- Info (default) displays correctly
- Info states lists all states
- Info fields shows all required/optional fields
- Info config displays merged configuration
- Info stats calculates counts correctly
- Info stats detects health issues

#### Configuration Tests

- Default config loads correctly
- Cargo.toml overrides defaults
- File config overrides Cargo.toml
- Invalid config values are rejected
- Missing config files handled gracefully

---

## Implementation Order Priority

### High Priority (Core Lifecycle)

1. Configuration system (Task 10.8)
2. Add "overwritten" state to DocState enum
3. Remove command (Task 9.2)
4. Replace command (Task 9.1)
5. List --removed enhancement (Task 9.3)

### Medium Priority (Discovery)

1. Info command framework (Task 10.1)
2. Info states (Task 10.3)
3. Info fields (Task 10.4)
4. Info config (Task 10.5)
5. Info default (Task 10.2)

### Low Priority (Polish)

1. Info stats (Task 10.6)
2. Info dirs (Task 10.7)

---

## Module Structure Updates

```
design/src/
├── cli.rs              (add Replace, Remove, Info commands)
├── config.rs           (NEW - configuration management)
├── commands/
│   ├── mod.rs
│   ├── replace.rs      (NEW - Task 9.1)
│   ├── remove.rs       (NEW - Task 9.2)
│   ├── list.rs         (UPDATE - add --removed flag)
│   └── info.rs         (NEW - Task 10.1-10.7)
└── state.rs            (UPDATE - add Overwritten state)
```

---

## Dependencies to Add

Current dependencies should be sufficient, but verify:

- `uuid` crate (for UUIDv4 generation)
- `toml` crate (for config file parsing, if not already present)
- `serde` with derive features (likely already present)

Add to `Cargo.toml`:

```toml
[dependencies]
uuid = { version = "1.0", features = ["v4"] }
toml = "0.8"
```

---

## Success Criteria

Phase 9 is complete when:

1. ✓ Documents can be replaced with `oxd replace`
2. ✓ Old document metadata is intelligently preserved
3. ✓ Replaced documents move to dustbin with UUID suffix
4. ✓ Documents can be removed with `oxd remove`
5. ✓ Removed documents are safely stored in dustbin
6. ✓ `oxd list --removed` shows deletion status
7. ✓ "overwritten" state is fully implemented
8. ✓ All operations update both data sources
9. ✓ Git operations preserve history

Phase 10 is complete when:

1. ✓ `oxd info` provides tool overview
2. ✓ `oxd info states` lists all valid states
3. ✓ `oxd info fields` documents all frontmatter fields
4. ✓ `oxd info config` displays configuration
5. ✓ `oxd info stats` provides project statistics
6. ✓ Configuration system supports multiple layers
7. ✓ All info commands have helpful, formatted output
8. ✓ Tool is self-documenting and discoverable

---

## Estimated Effort

- Task 10.8 (Config System): 2-3 hours
- Task 9.2 (Remove Command): 2-3 hours
- Task 9.1 (Replace Command): 3-4 hours
- Task 9.3 (List Removed): 1-2 hours
- Task 10.1 (Info Framework): 1 hour
- Task 10.2-10.5 (Info Subcommands): 3-4 hours
- Task 10.6-10.7 (Stats/Dirs): 2-3 hours
- Testing & Polish: 2-3 hours

**Total: 16-23 hours of development time**

---

## Future Enhancements (Out of Scope for Phases 9-10)

- `oxd restore <id>` - Restore document from dustbin
- `oxd clean-dustbin --older-than 90d` - Permanent deletion
- `oxd history <id>` - Show document history including replacements
- `oxd diff <id1> <id2>` - Compare two documents
- Configuration validation command
- Configuration wizard for first-time setup
- Export configuration to file
- Interactive document replacement workflow
- Dustbin size limits and auto-cleanup

---

## Migration Notes

### For Existing Projects

When upgrading to include Phases 9-10:

1. **Dustbin Directory**: Will be created automatically on first remove/replace
2. **New States**: "overwritten" and "removed" states are backward compatible
3. **Configuration**: Existing projects work with defaults; no config changes required
4. **Data Sources**: Existing index and database files work without changes

### Breaking Changes

None expected. All new features are additive.

---

## Quick Reference: New Commands

```bash
# Replace document (preserve ID)
oxd replace <old-id> <new-file>

# Remove document (move to dustbin)
oxd remove <id>

# List removed documents
oxd list --removed
oxd list --removed --verbose

# Tool information
oxd info                  # Overview
oxd info states           # Valid states
oxd info fields           # Frontmatter fields
oxd info config           # Configuration
oxd info stats            # Project statistics
oxd info dirs             # Directory structure
```

---

## Appendix: State Machine Diagram

```
                    ┌──────────────┐
                    │    draft     │
                    └──────┬───────┘
                           │
                           v
                    ┌──────────────┐
                ┌──>│ under-review │
                │   └──────┬───────┘
                │          │
                │          v
                │   ┌──────────────┐
                └───│   revised    │
                    └──────┬───────┘
                           │
                           v
        ┌──────────────────┴──────────────────┐
        │                                      │
        v                                      v
┌──────────────┐                      ┌──────────────┐
│   accepted   │                      │   rejected   │
└──────┬───────┘                      └──────────────┘
       │
       ├────────> active ────────> final
       │
       ├────────> deferred
       │
       └────────> withdrawn

Any state ──> removed (via oxd remove)
Any state ──> overwritten (via oxd replace)

superseded: Set via metadata links, not direct transition
```

---

End of Build Plan - Phases 9-10
