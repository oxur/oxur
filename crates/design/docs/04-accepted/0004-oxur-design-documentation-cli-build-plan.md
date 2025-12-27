---
number: 4
title: "Oxur Design Documentation CLI - Build Plan"
author: "document number"
created: 2025-12-27
updated: 2025-12-27
state: Accepted
supersedes: null
superseded-by: null
---

# Oxur Design Documentation CLI - Build Plan

## Executive Summary

This document outlines the plan to bring the Rust-based `oxd` CLI tool to feature parity with the existing Go-based `zdp` tool. The Go tool has eight distinct modes of operation and several sophisticated features that need to be ported.

## Current State Analysis

### Rust Tool (oxd) - Current Features
- List documents with optional state filtering and verbose mode
- Show individual documents with metadata
- Create new documents from templates
- Validate documents for consistency
- Stub for index generation (not implemented)
- Uses clap for CLI parsing
- Has library structure with doc/index modules

### Go Tool (zdp) - Complete Feature Set
- Eight operational modes (detailed below)
- Git integration for author/date extraction and file operations
- YAML frontmatter parsing and updating
- Automatic document numbering
- State-based directory organization (10 states vs. 4 in Rust)
- Index synchronization with state sections and tables
- Full document lifecycle management

## Feature Gap Analysis

### Major Missing Features

1. **Git Integration**
   - Extract author from git history
   - Extract creation/modification dates from git log
   - Use `git mv` for file operations to preserve history
   - Stage files with `git add`

2. **Index Management**
   - Generate markdown index with two sections:
     - Table of all documents (by number)
     - Lists organized by state
   - Synchronize index with filesystem
   - Add/remove documents from index automatically
   - Format cleanup for consistent spacing

3. **Document Addition Workflow**
   - Multi-step process for new documents
   - Automatic numbering
   - Directory placement logic
   - Header validation and correction
   - Git staging

4. **YAML Frontmatter Management**
   - Add headers to documents missing them
   - Update specific fields (state, updated date)
   - Parse and validate all metadata fields
   - Handle supersedes/superseded-by relationships

5. **State System Expansion**
   - Go tool has 10 states vs. 4 in Rust
   - States: Draft, Under Review, Revised, Accepted, Active, Final, Deferred, Rejected, Withdrawn, Superseded
   - Bidirectional state-to-directory mapping

6. **Document Movement**
   - Move documents to match their header state
   - Transition documents between states
   - Update index when moving
   - Preserve git history during moves

## Detailed Implementation Plan

### Phase 1: Foundation Enhancements

#### 1.1 Expand State System
- Extend `DocState` enum to include all 10 states from Go tool
- Add state normalization (handle hyphens, spaces, case)
- Create bidirectional mappings between state names and directories
- Update `DocState::directory()` method
- Add title-case conversion for display

#### 1.2 Git Integration Module
Create new `src/git.rs` module with functions:
- `get_author(path)` - extract original author from git log
- `get_created_date(path)` - extract first commit date
- `get_updated_date(path)` - extract last commit date
- `git_mv(src, dst)` - move file preserving history
- `git_add(path)` - stage file
- All functions should handle errors gracefully and provide fallbacks

#### 1.3 Enhanced YAML Operations
Extend `src/doc.rs`:
- Add `update_yaml_field()` function for surgical updates
- Add `add_missing_headers()` function to complete partial frontmatter
- Add validation for all required fields
- Ensure date format consistency (YYYY-MM-DD)
- Handle null/None values properly in supersedes fields

### Phase 2: Core Command Implementation

#### 2.1 Implement Index Generation
Complete the stubbed `Commands::Index`:
- Generate markdown table with columns: Number, Title, State, Updated
- Sort by document number
- Create state sections with document links
- Use relative paths for portability
- Support both markdown and JSON output formats

#### 2.2 Add Document Header Management
New command: `oxd add-headers <doc>`
- Check if document has frontmatter
- Extract metadata from filename and content
- Query git for author/dates
- Add or complete YAML frontmatter
- Report what was added/updated

#### 2.3 Implement State Transitions
Enhance existing functionality:
- New command: `oxd transition <doc> <state>`
- Validate current state
- Update YAML frontmatter (state and updated date)
- Move file to new state directory using git mv
- Update index file
- Provide clear feedback on what changed

#### 2.4 Implement Move-to-Match-Header
New command: `oxd sync-location <doc>`
- Read state from document's YAML
- Check current directory
- Move to correct directory if mismatched
- Don't update YAML (just fix location)
- Use git mv for history preservation

### Phase 3: Index Synchronization

#### 3.1 Index Scanning
New module `src/index_sync.rs`:
- Parse existing index markdown
- Extract table entries
- Extract state section entries
- Track document numbers and locations

#### 3.2 Index Update Logic
Implement comprehensive sync:
- Compare git-tracked files vs index entries
- Add missing documents to table
- Update changed dates/states in table
- Add documents to appropriate state sections
- Remove stale entries
- Maintain sorted order by document number

#### 3.3 Formatting Cleanup
- Ensure consistent blank line spacing around headers
- Remove extra blank lines between bullet items
- Keep exactly one blank line before section headers
- Apply formatting atomically with content updates

#### 3.4 Index Update Command
New command: `oxd update-index`
- Scan all state directories for .md files
- Sync table with actual documents
- Sync all state sections
- Apply formatting cleanup
- Report all changes made
- Show summary of additions/removals/updates

### Phase 4: Document Addition Workflow

#### 4.1 Number Assignment
- Extract highest number from index
- Assign next sequential number
- Rename file with 4-digit prefix
- Handle files that already have numbers

#### 4.2 Directory Placement
- Detect if file is in project directory
- Move to project root if external
- Check if in a state directory
- Move to draft directory if not
- Create directories as needed

#### 4.3 Header Processing
- Check for YAML frontmatter
- Add complete headers if missing
- Validate all required fields
- Sync state field with directory location

#### 4.4 Git Integration
- Stage file after processing
- Provide clear feedback at each step
- Handle errors gracefully

#### 4.5 Add Command
New command: `oxd add <doc>`
Orchestrates full workflow:
1. Validate file exists
2. Assign number if needed
3. Move to project if external
4. Place in state directory
5. Add/update headers
6. Sync state with location
7. Stage with git
8. Update index

### Phase 5: Testing & Polish

#### 5.1 Error Handling
- Provide clear, actionable error messages
- Suggest supported states when invalid state given
- Handle missing files gracefully
- Validate index file exists before operations
- Check git repository status

#### 5.2 Colored Output
- Leverage existing `colored` crate usage
- Consistent color scheme:
  - Errors: red
  - Warnings: yellow
  - Success: green
  - Info: cyan
  - State badges: match existing pattern

#### 5.3 Command Aliases
Add convenient aliases:
- `oxd ls` → `oxd list`
- `oxd new` already exists
- `oxd mv` → `oxd transition`
- `oxd sync` → `oxd update-index`

#### 5.4 Validation Enhancements
Extend `validate` command:
- Check for files not in index
- Check for index entries without files
- Validate all supersedes/superseded-by links
- Check state consistency (header vs directory)
- Optionally fix issues with `--fix` flag

#### 5.5 Documentation
- Update CLI help text with examples
- Add README with common workflows
- Document state transition rules
- Provide troubleshooting guide

## Implementation Order Priority

### High Priority (Core Functionality)
1. Git integration module
2. State system expansion
3. YAML update operations
4. Index generation (markdown)
5. Transition command
6. Update-index command

### Medium Priority (Workflow Enhancement)
1. Add-headers command
2. Add command (full workflow)
3. Sync-location command
4. Enhanced validation
5. Formatting cleanup

### Low Priority (Polish)
1. JSON output format
2. Command aliases
3. Advanced error handling
4. Colored output refinements
5. Documentation

## Technical Considerations

### Dependencies to Add
- `regex` - for pattern matching in YAML and filenames
- `walkdir` - already present, continue using
- Git operations can use `std::process::Command` (no git2 needed)

### Module Structure
```
src/
├── lib.rs          (existing)
├── main.rs         (existing)
├── cli.rs          (existing - extend commands)
├── doc.rs          (existing - extend parsing)
├── index.rs        (existing - extend)
├── git.rs          (new - git operations)
├── index_sync.rs   (new - index synchronization)
└── commands/
    ├── mod.rs      (existing)
    ├── list.rs     (existing)
    ├── new.rs      (existing)
    ├── show.rs     (existing)
    ├── validate.rs (existing - enhance)
    ├── index.rs    (new - implement)
    ├── transition.rs (new)
    ├── add_headers.rs (new)
    ├── sync_location.rs (new)
    ├── add.rs      (new)
    └── update_index.rs (new)
```

### Testing Strategy
- Unit tests for each git operation (mock git commands)
- Integration tests for document operations
- Test fixtures with sample documents
- Index parsing/generation tests
- State transition validation tests

## Migration Path

### For Users
- Existing documents should work unchanged
- State directories may need renaming if switching from 4-state to 10-state model
- Index file will need regeneration
- Consider providing migration script

### Backward Compatibility
- Keep existing 4-state model as option
- Add `--state-model` flag to switch between models
- Or detect from existing directory structure
- Make 10-state the default for new projects

## Success Criteria

The Rust tool will have feature parity when:
1. All 8 modes from Go tool are implemented
2. Git integration works seamlessly
3. Index stays synchronized automatically
4. Documents can be added with single command
5. State transitions preserve git history
6. Validation catches all inconsistencies
7. Error messages are clear and helpful
8. Output is colorful and informative

## Estimated Effort

- Phase 1: 4-6 hours
- Phase 2: 6-8 hours
- Phase 3: 4-6 hours
- Phase 4: 3-4 hours
- Phase 5: 2-3 hours

Total: 19-27 hours of development time

## Future Enhancements (Out of Scope)

- Web interface for browsing documents
- Export to other formats (HTML, PDF)
- Document templates beyond basic
- Approval workflows
- Change notifications
- Integration with issue trackers

---

## Quick Reference: Go Tool Modes

For implementation reference, the 8 modes:

1. **List all documents**: `zdp` - Show documents grouped by state
2. **Move to match header**: `zdp <doc>` - Fix directory/header mismatch
3. **Transition state**: `zdp <doc> <state>` - Change document state
4. **List states**: `zdp states` - Show all valid states
5. **Add to index**: `zdp index <doc>` - Add document entry
6. **Add headers**: `zdp add-headers <doc>` - Add/update YAML
7. **Update index**: `zdp update-index` - Full index synchronization
8. **Add document**: `zdp add <doc>` - Complete addition workflow
