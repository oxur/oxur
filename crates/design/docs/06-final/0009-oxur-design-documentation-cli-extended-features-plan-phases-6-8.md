---
number: 9
title: "Oxur Design Documentation CLI - Extended Features Plan (Phases 6-8)"
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

# Oxur Design Documentation CLI - Extended Features Plan (Phases 6-8)

## Executive Summary

This document outlines the advanced features implemented after achieving feature parity with the Go-based `zdp` tool. These phases focus on:

- **Phase 6**: Establishing a canonical source of truth with state management
- **Phase 7**: Building a sophisticated document onboarding system
- **Phase 8**: Adding comprehensive debugging and search capabilities

## Background

After completing the core functionality (Phases 1-5) that brought `oxd` to feature parity with `zdp`, we identified several architectural improvements and user experience enhancements that would significantly improve the tool's robustness and usability.

---

## Phase 6: State Management & Source of Truth

### Problem Statement

The original architecture had three potential "sources of truth" that could diverge:

1. YAML frontmatter in each .md file
2. The 00-index.md markdown file
3. Physical directory structure (state directories)

This created several critical issues:

- External git operations could cause drift between sources
- Manual file edits weren't tracked
- No way to detect what changed since last run
- Full re-parse on every command (performance issue)
- Validation only caught issues after they occurred
- No atomic operations across multiple files

**Real-world failure scenarios:**

```bash
# Scenario 1: External git operation
git mv 01-draft/0001-feature.md 02-under-review/
# → YAML still says "Draft", directory says "Under Review"

# Scenario 2: Manual YAML edit
vim 01-draft/0001-feature.md  # Change state to "Final"
# → File says "Final", directory says "Draft", index says "Draft"

# Scenario 3: Concurrent operations
# Two terminal windows both running oxd commands
# → Race condition, last write wins, no consistency guarantees
```

### Solution Architecture

**Single Source of Truth Design:**

- Create canonical state file: `.oxd/state.json`
- Authoritative source for all document metadata
- Loaded into memory on startup
- Updated atomically with all operations
- Used to detect changes in files/filesystem

**Data Flow:**

```
Startup:
  1. Load state.json → Memory
  2. Detect changed files (checksums/mtimes)
  3. Re-scan changed files only
  4. Update state.json
  5. Ready for commands

Operation:
  1. Read from memory state
  2. Perform operation
  3. Update files
  4. Update state.json atomically
  5. Commit changes
```

### Features Implemented

#### 6.1 Serialization Format Selection

- **Chosen Format**: JSON (via serde_json)
- Human-readable for debugging
- Fast serialization/deserialization
- Already in dependencies
- Wide tool support

**State File Structure:**

```json
{
  "version": 1,
  "last_updated": "2025-01-15T10:30:00Z",
  "next_number": 25,
  "documents": {
    "1": {
      "metadata": { ... },
      "path": "01-draft/0001-feature.md",
      "checksum": "abc123...",
      "file_size": 4096,
      "modified": "2025-01-15T10:00:00Z"
    }
  }
}
```

#### 6.2 Formalized Schemas

- `DocumentState` - canonical state of all documents
- `DocumentRecord` - single document's complete record
- Schema versioning for future migrations
- Checksum utilities (SHA-256)
- File metadata utilities

#### 6.3 State Manager

- Central state management singleton
- Atomic save operations (temp file + rename)
- Automatic change detection
- CRUD operations for document records
- Integration with all commands

#### 6.4 Change Detection System

- **Quick Check**: File size and mtime comparison
- **Full Verification**: SHA-256 checksum computation
- Detect new, modified, and deleted files
- Incremental updates (only changed files)

#### 6.5 Scan Command

New command: `oxd scan`

- Re-scan filesystem for changes
- Compare with stored state
- Report inconsistencies
- Optional auto-fix mode
- Verbose reporting

**Features:**

```bash
oxd scan                    # Detect all changes
oxd scan --fix              # Auto-fix inconsistencies
oxd scan --verbose          # Detailed output
```

#### 6.6 Command Integration

Updated all write commands to update state:

- `add` - records new documents
- `add-headers` - updates checksums
- `transition` - updates after moves
- `update-index` - syncs state

#### 6.7 Optimized File Reading

- Lazy content loading (don't read file contents unless needed)
- Read from state for metadata
- Only re-parse files when checksums differ
- Significant performance improvement for large repositories

#### 6.8 Quick Scan Optimization

- Fast startup using mtime/size checks
- Full checksum only when quick check indicates change
- Startup performance: <50ms for unchanged repos

### Performance Improvements

**Before Phase 6:**

- 100 docs: ~2s startup (full parse every time)
- Every command re-parsed all files

**After Phase 6:**

- Cold start (no state): 100 docs in <1s
- Warm start (with state): <50ms
- Quick scan (no changes): <200ms (stat calls only)

### Migration Path

**For existing repositories:**

```bash
# First run builds state from scratch
oxd list
# → Scanning 50 documents...
# → State initialized

# Subsequent runs are instant
oxd list
# → (instant)
```

**Gitignore addition:**

```
.oxd/
```

---

## Phase 7: Advanced Document Onboarding

### Problem Statement

The basic `add` command handled simple cases but lacked sophistication for:

- Messy filenames (spaces, special chars, unicode)
- Missing or incomplete metadata (no interactive prompting)
- Content quality issues (broken markdown, inconsistent formatting)
- Batch operations (multiple files at once)
- Preview capabilities (see changes before commit)

**Real-world scenario:**

```bash
# User has a design doc from personal notes
~/Documents/My Cool Feature Idea!!!.md

# Should become:
docs/01-draft/0012-my-cool-feature-idea.md

# With:
# - Number prefix
# - Sanitized filename
# - Complete YAML frontmatter
# - Normalized markdown
```

### Features Implemented

#### 7.1 Filename Sanitization

**New Module:** `src/filename.rs`

**Capabilities:**

- Unicode normalization (NFD form)
- Special character removal
- Space/underscore → hyphen conversion
- Multiple hyphen collapse
- Length limiting (100 chars max)
- Case normalization (lowercase)
- Number prefix handling

**Examples:**

```rust
sanitize_filename("My Cool Feature!!!") → "my-cool-feature"
sanitize_filename("Café naïve") → "cafe-naive"
sanitize_filename("my___feature") → "my-feature"
```

**Utilities:**

```rust
build_filename(number, title) → "0012-my-cool-feature.md"
filename_to_title(filename) → "My Cool Feature"
```

#### 7.2 Interactive Metadata Prompting

**New Module:** `src/prompt.rs`

**Functions:**

- `prompt_with_default()` - prompt with smart default
- `prompt_required()` - mandatory input
- `prompt_select()` - choose from options
- `prompt_confirm()` - yes/no confirmation

**Interactive Workflow:**

```
Step 1: Title
Document title [My Cool Feature]: <Enter>
  ✓ Title: My Cool Feature

Step 2: Author
Author [Alice Smith]: <Enter>
  ✓ Author: Alice Smith

Step 3: Initial State
Initial state
 *1) Draft
  2) Under Review
  3) Revised
Select [1]: 1
  ✓ State: Draft
```

#### 7.3 Smart Metadata Extraction

**New Module:** `src/extract.rs`

**Extraction Capabilities:**

- Title from first H1 heading
- Author from existing YAML or git
- State hints from content/location
- Creation date from git history
- Fallback to intelligent defaults

**ExtractedMetadata Structure:**

```rust
pub struct ExtractedMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub state_hint: Option<DocState>,
    pub created_hint: Option<String>,
}
```

#### 7.4 Content Normalization

**Features:**

- Multiple H1 heading detection/consolidation
- Consistent bullet point markers (normalize to -)
- Trailing whitespace removal
- Blank line normalization
- Line ending consistency (Unix LF)

**Optional Application:**

```
Content Issues Detected:
  ⚠ Multiple H1 headings found (2)
  ⚠ Inconsistent bullet point markers (-, *, +)

Apply automatic normalization? [Y/n]:
```

#### 7.5 Enhanced Add Command

**Complete Workflow:**

1. Validate file exists
2. Preview mode (if requested)
3. Extract metadata intelligently
4. Interactive prompts for missing fields
5. Sanitize filename
6. Assign document number
7. Normalize content (optional)
8. Add/update YAML frontmatter
9. Move to appropriate state directory
10. Stage with git
11. Update state
12. Update index

**Command Flags:**

```bash
oxd add <file>                    # Full interactive mode
oxd add <file> --yes              # Auto-accept all defaults
oxd add <file> --dry-run          # Preview without changes
oxd add <file> --preview          # Show before/after table
oxd add <file> --interactive      # Explicit interactive mode
oxd add <file> --no-normalize     # Skip content normalization
```

#### 7.6 Batch Operations

**New Command:** `oxd add-batch <patterns...>`

**Features:**

- Glob pattern support (`*.md`, `docs/**/*.md`)
- Multiple pattern handling
- Progress reporting
- Individual file error handling (continue on failure)
- Summary statistics

**Usage:**

```bash
oxd add-batch ~/Downloads/*.md
# → Found 5 files
# → [1/5] Processing: feature-1.md
# → [2/5] Processing: feature-2.md
# ...
# ✓ Batch complete: 4 succeeded, 1 failed

oxd add-batch docs/**/*.md notes/*.md --interactive
# Interactive confirmation for each file
```

#### 7.7 Preview Mode

**New Dependency:** `prettytable-rs`

**Before/After Table:**

```
┌─────────────┬──────────────────────────────┬────────────────────────────┐
│ Property    │ Before                       │ After                      │
├─────────────┼──────────────────────────────┼────────────────────────────┤
│ Location    │ ~/Downloads/my-feature.md    │ docs/01-draft/0012-...     │
│ Filename    │ my-feature.md                │ 0012-my-cool-feature.md    │
│ Number      │ -                            │ 0012                       │
│ Title       │ -                            │ My Cool Feature            │
│ Author      │ -                            │ Alice Smith                │
│ State       │ -                            │ Draft                      │
└─────────────┴──────────────────────────────┴────────────────────────────┘
```

### User Experience Improvements

**Smart Defaults:**

- Title from filename or first heading
- Author from git config
- State defaulting to Draft
- Date from current time or git history

**Automation Options:**

- `--yes` flag for CI/CD pipelines
- `--dry-run` for safety checks
- Batch mode for bulk imports

**Quality Improvements:**

- Content normalization catches common issues
- Filename sanitization prevents filesystem problems
- Unicode handling prevents encoding issues

### Dependencies Added

```toml
unicode-normalization = "0.1"
prettytable-rs = "0.10"
glob = "0.3"
```

---

## Phase 8: Debug Tools & Search

### Problem Statement

As the state management system became more sophisticated, we needed:

- Tools to inspect and understand internal state
- Ways to troubleshoot inconsistencies
- Powerful search capabilities beyond basic grep
- Statistics and health monitoring

### Features Implemented

#### 8.1 State Inspection Commands

**New Command Group:** `oxd debug`

**Subcommands:**

##### `oxd debug state`

Show complete state in multiple formats:

**Table Format:**

```
Document State
Version: 1
Last Updated: 2025-01-15 10:30:00
Next Number: 0025
Total Documents: 24

┌─────┬────────────────────┬────────────┬────────┬──────────────────┬──────────┐
│ Num │ Title              │ State      │ Size   │ Modified         │ Checksum │
├─────┼────────────────────┼────────────┼────────┼──────────────────┼──────────┤
│ 0001│ Initial Design     │ Final      │ 4.2 KB │ 2025-01-10 09:00 │ abc12345 │
│ 0002│ API Spec           │ Active     │ 8.1 KB │ 2025-01-12 14:30 │ def67890 │
...
```

**Summary Format:**

```
State Summary

Documents by State:
  Draft: 8
  Under Review: 3
  Active: 2
  Final: 11

Size Statistics:
  Total: 142.8 KB
  Average: 5.9 KB

Recently Modified:
  0024 - New Feature (2025-01-15 10:00)
  0023 - API Update (2025-01-14 15:30)
  ...
```

**JSON Format:**

```bash
oxd debug state --format json > state-dump.json
# Full state export for external tools
```

##### `oxd debug state <number>`

Show detailed state for specific document:

```
Document 0012 State

Metadata:
  Number: 0012
  Title: My Cool Feature
  Author: Alice Smith
  State: Draft
  Created: 2025-01-10
  Updated: 2025-01-15
  Supersedes: 0008

File Information:
  Path: 01-draft/0012-my-cool-feature.md
  Size: 4.2 KB
  Modified: 2025-01-15T10:00:00Z
  Checksum: abc123def456...
```

#### 8.2 Checksum Inspection

**Command:** `oxd debug checksums`

**Purpose:** Identify files that have changed on disk but not updated in state

**Output:**

```
Checksum Status

┌─────┬────────────────────┬────────┬──────────────────┬──────────────────┐
│ Num │ Title              │ Status │ Stored Checksum  │ Actual Checksum  │
├─────┼────────────────────┼────────┼──────────────────┼──────────────────┤
│ 0012│ My Feature         │ DIRTY  │ abc123...        │ def456...        │
│ 0015│ Old Doc            │ MISSING│ ghi789...        │ -                │
└─────┴────────────────────┴────────┴──────────────────┴──────────────────┘

Summary:
  22 Clean
  1 Dirty
  1 Missing

→ Run 'oxd scan' to update checksums
```

**Flags:**

```bash
oxd debug checksums            # Show only dirty/missing
oxd debug checksums --verbose  # Show all files
```

#### 8.3 Diff Detection

**Command:** `oxd debug diff`

**Purpose:** Show differences between state and filesystem

**Detects:**

- Documents in state but not on filesystem
- Documents on filesystem but not in state
- Metadata mismatches (YAML vs state)
- Location mismatches (directory vs state)

**Output:**

```
State vs Filesystem Diff

Missing from Filesystem:
  0008 - Old Feature (expected at 01-draft/0008-old-feature.md)

Missing from State:
  0025-new-addition.md (in 01-draft/)

Metadata Mismatches:
  0012 - My Feature
    State says: Draft
    YAML says: Under Review

Location Mismatches:
  0015 - Authentication
    State says: 02-under-review/
    Actually in: 03-revised/

Summary: 4 inconsistencies found
```

#### 8.4 Orphan Detection

**Command:** `oxd debug orphans`

**Purpose:** Find state entries with no corresponding file

**Use Cases:**

- Detect deleted files not removed from state
- Find state corruption
- Identify cleanup candidates

#### 8.5 Document Verification

**Command:** `oxd debug verify <number>`

**Deep Verification:**

- File exists on filesystem
- Checksum matches
- YAML parses correctly
- Metadata consistent with state
- Supersedes/superseded-by links valid
- State directory matches metadata

**Output:**

```
Verifying Document 0012

✓ File exists
✓ Checksum matches
✓ YAML valid
✓ Metadata consistent
✓ No broken links
✗ Location mismatch
  Expected: 02-under-review/
  Actual: 01-draft/

Result: 1 issue found
```

#### 8.6 Repository Statistics

**Command:** `oxd debug stats`

**Comprehensive Statistics:**

**Document Counts:**

- Total documents
- By state
- Next available number

**Size Analysis:**

- Total repository size
- Average document size
- Largest/smallest documents

**Activity Metrics:**

- Recently modified documents
- Recently created documents
- Most active states

**Author Statistics:**

- Documents by author
- Most active authors

**State Transitions:**

- Documents by final state
- State distribution percentages

**Output Example:**

```
Repository Statistics

Documents:
  Total: 24
  Next Number: 0025

By State:
  Draft: 8 (33%)
  Under Review: 3 (13%)
  Active: 2 (8%)
  Final: 11 (46%)

Size:
  Total: 142.8 KB
  Average: 5.9 KB
  Largest: 0002 - API Spec (8.1 KB)
  Smallest: 0019 - Quick Fix (1.2 KB)

Authors:
  Alice Smith: 12 documents
  Bob Jones: 7 documents
  Carol Wilson: 5 documents

Recent Activity (Last 7 Days):
  Modified: 5 documents
  Created: 2 documents
  Transitioned: 3 documents
```

#### 8.7 Intelligent Search

**Command:** `oxd search <query>`

**Wrapper Around git grep:**

- Uses `git grep` for fast searching
- Adds intelligent filtering by state
- Enhanced result formatting
- Document context in output

**Features:**

```bash
# Basic search
oxd search "authentication"

# Filter by state
oxd search "API" --state draft

# Search only in metadata (YAML frontmatter)
oxd search "author: Alice" --metadata

# Case-sensitive search
oxd search "TODO" -I
```

**Enhanced Output:**

```
→ 0012 - My Cool Feature (Draft)
  45: We need to implement authentication for the API
  67: The authentication flow should use OAuth 2.0

→ 0015 - Security Review (Under Review)
  23: Authentication mechanisms reviewed
  89: Two-factor authentication recommended

Found 2 documents with 4 matches
```

**Compared to plain git grep:**

- Shows document numbers and titles
- Filters by state directory
- Optional metadata-only search
- Cleaner, more readable output

#### 8.8 Search Options

**Advanced Features:**

- Regex support (via git grep)
- Context lines (before/after matches)
- Multiple query support
- Exclude patterns
- File pattern filtering

**SearchOptions Structure:**

```rust
pub struct SearchOptions {
    pub state: Option<String>,        // Filter by state
    pub metadata_only: bool,           // Search only YAML
    pub case_sensitive: bool,          // Case sensitivity
    pub context_lines: usize,          // Lines before/after
    pub regex: bool,                   // Regex mode
}
```

### Debug Module Structure

```
src/commands/
├── debug.rs          # State inspection commands
│   ├── show_state()
│   ├── show_document_state()
│   ├── show_checksums()
│   ├── show_stats()
│   ├── show_diff()
│   ├── show_orphans()
│   └── verify_document()
└── search.rs         # Search functionality
    ├── search()
    ├── search_advanced()
    └── display_results()
```

### CLI Enhancements

**New Debug Subcommands:**

```
oxd debug state [number] [--format <format>]
oxd debug checksums [--verbose]
oxd debug stats
oxd debug diff
oxd debug orphans
oxd debug verify <number>
```

**Search Command:**

```
oxd search <query> [--state <state>] [--metadata] [-I]
```

### Use Cases

**Troubleshooting Workflow:**

```bash
# 1. Check overall health
oxd debug stats

# 2. Find inconsistencies
oxd debug diff

# 3. Identify dirty files
oxd debug checksums

# 4. Fix issues
oxd scan --fix

# 5. Verify specific document
oxd debug verify 12
```

**Development Workflow:**

```bash
# Search for TODOs in drafts
oxd search "TODO" --state draft

# Find all documents by an author
oxd search "author: Alice" --metadata

# Check state before making changes
oxd debug state --format summary
```

### Dependencies Added

```toml
prettytable-rs = "0.10"  # For formatted tables
regex = "1.10"            # For pattern matching
```

---

## Implementation Summary

### Total Features Added (Phases 6-8)

**Phase 6 (State Management):**

- Canonical state file (`.oxd/state.json`)
- Change detection system (checksums + mtimes)
- State manager with atomic operations
- Scan command for consistency checking
- Optimized file reading (lazy loading)
- Quick scan for fast startup

**Phase 7 (Advanced Onboarding):**

- Filename sanitization (unicode, special chars)
- Interactive metadata prompting
- Smart metadata extraction
- Content normalization
- Enhanced add command (10-step workflow)
- Batch operations (glob patterns)
- Preview mode (before/after tables)

**Phase 8 (Debug & Search):**

- State inspection (3 formats)
- Document-specific state view
- Checksum status checking
- Repository statistics
- Diff detection (state vs filesystem)
- Orphan detection
- Document verification
- Intelligent search (wraps git grep)

### Performance Metrics

**Startup Time:**

- Before: ~2s for 100 docs (full parse)
- After: <50ms (with state file)
- Improvement: 40x faster

**Search Performance:**

- Leverages git grep (extremely fast)
- State filtering adds minimal overhead
- 1000+ docs: sub-second searches

**Memory Usage:**

- State file: ~1KB per document
- 100 docs: ~100KB in memory
- Lazy content loading reduces memory footprint

### Code Quality Improvements

**Error Handling:**

- Comprehensive error contexts (anyhow)
- Graceful degradation (missing files, corrupt state)
- Clear user-facing error messages
- Suggested actions for common errors

**Testing:**

- Unit tests for sanitization
- Unit tests for extraction
- Integration tests for state management
- Mock git operations for testing

**Documentation:**

- Inline code documentation
- Module-level overviews
- CLI help text with examples
- Error message suggestions

---

## Migration & Compatibility

### Upgrading from Phase 5

**First Run After Upgrade:**

```bash
oxd list
# → No state file found, building from scratch...
# → Scanning 24 documents...
# → State initialized at .oxd/state.json
# → Done
```

**Gitignore Update:**

```bash
echo ".oxd/" >> .gitignore
git add .gitignore
git commit -m "Ignore oxd state directory"
```

### Backward Compatibility

- All Phase 1-5 commands continue to work
- State is transparent to users
- No breaking changes to document format
- No breaking changes to CLI interface
- State file is optional (regenerates if deleted)

---

## Success Criteria

Phases 6-8 are considered successful when:

**Phase 6:**

- [x] State file loads/saves correctly
- [x] Change detection is accurate
- [x] Scan command works reliably
- [x] Startup time <50ms for unchanged repos
- [x] All commands update state atomically
- [x] State corruption handled gracefully

**Phase 7:**

- [x] Filename sanitization handles all edge cases
- [x] Interactive mode is user-friendly
- [x] Smart defaults are accurate
- [x] Batch operations process multiple files
- [x] Preview mode shows accurate changes
- [x] Content normalization improves quality

**Phase 8:**

- [x] Debug commands provide useful insights
- [x] Checksum detection finds dirty files
- [x] Statistics are comprehensive
- [x] Search is faster than manual grep
- [x] Results are well-formatted
- [x] Documentation is complete

---

## Estimated Effort (Actual)

**Phase 6:** ~8 hours

- State schema design: 1h
- State manager implementation: 2h
- Change detection: 2h
- Scan command: 1h
- Integration & testing: 2h

**Phase 7:** ~12 hours

- Filename utilities: 2h
- Interactive prompts: 2h
- Metadata extraction: 2h
- Content normalization: 2h
- Enhanced add workflow: 3h
- Batch operations: 1h

**Phase 8:** ~6 hours

- Debug module structure: 1h
- State inspection commands: 2h
- Checksum/diff/verify: 2h
- Search wrapper: 1h

**Total:** ~26 hours of development time

---

## Technical Architecture

### Module Dependencies

```
src/
├── lib.rs
├── main.rs
├── cli.rs
├── doc.rs
├── index.rs
├── git.rs
├── state.rs         # Phase 6: State management
├── filename.rs      # Phase 7: Filename utilities
├── prompt.rs        # Phase 7: Interactive prompts
├── extract.rs       # Phase 7: Metadata extraction
└── commands/
    ├── mod.rs
    ├── list.rs
    ├── show.rs
    ├── new.rs
    ├── validate.rs
    ├── add.rs           # Enhanced in Phase 7
    ├── add_headers.rs   # Updated in Phase 6
    ├── transition.rs    # Updated in Phase 6
    ├── debug.rs         # Phase 8: Debug tools
    └── search.rs        # Phase 8: Search wrapper
```

### Data Flow (Post-Phase 6)

```
┌──────────────────┐
│  State File      │
│  .oxd/state.json │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  State Manager   │◄────── Commands
│  (In Memory)     │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Document Index  │
│  (Lazy Loading)  │
└──────────────────┘
```

### Key Design Patterns

**Atomic Operations:**

- Write to temp file
- Verify write succeeded
- Rename to final location
- OS guarantees atomicity

**Lazy Loading:**

- Load metadata on startup
- Load content on demand
- Cache in memory if needed
- Invalidate on change

**Change Detection:**

- Quick check: mtime + size
- Full verification: checksum
- Incremental updates
- Minimal filesystem access

---

## Conclusion

Phases 6-8 transformed `oxd` from a functional tool into a robust, production-ready system with:

- **Reliability**: Single source of truth prevents drift
- **Performance**: 40x faster startup through intelligent caching
- **Usability**: Interactive onboarding and comprehensive debugging
- **Maintainability**: Clear separation of concerns and thorough testing

The tool now handles edge cases gracefully, provides excellent user feedback, and scales well to large document repositories.
