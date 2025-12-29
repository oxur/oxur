# CLI Code Reuse and Deduplication Tracking

**Date:** 2025-12-29

## Background

The Oxur project has an existing CLI tool: **`oxd`** (Oxur Design/Document management tool)

**Key Resources:**
- Main binary: `crates/design/src/main.rs` → builds to `./bin/oxd`
- Commands help: `./bin/oxd -h`
- List command: `crates/design/src/commands/list.rs`
  - Contains colored ASCII table rendering
  - Recent updates for colored table data

## Guidelines for `aster` CLI Development

When implementing the `aster` CLI tool:

1. **Review existing patterns in `oxd`**
   - Command structure and organization
   - Argument parsing patterns
   - Table rendering (especially `list.rs`)
   - Colored output handling
   - Error handling and user feedback

2. **Track duplications**
   - Keep a list of any code/patterns being duplicated from `oxd`
   - Note common concepts that appear in both CLIs:
     - Table rendering
     - Colored terminal output
     - Command-line argument parsing
     - File I/O patterns
     - Error formatting
     - Progress indicators
     - etc.

3. **Generate deduplication report**
   When patterns emerge, create a comprehensive report documenting:
   - What code/concepts are duplicated
   - Where they appear in each codebase
   - Recommendations for generalization
   - Proposed structure for a common support crate (e.g., `oxur-cli-common`)
   - Migration strategy

## Potential Common Patterns to Watch For

- [ ] Colored ASCII table rendering
- [ ] Command-line argument parsing utilities
- [ ] Terminal color/styling abstractions
- [ ] File path handling
- [ ] Error display formatting
- [ ] Progress indicators
- [ ] Configuration file handling
- [ ] Output formatters (JSON, table, etc.)

## Action Items

- [ ] Review `oxd` command structure before implementing `aster` commands
- [ ] Study `crates/design/src/commands/list.rs` for table rendering patterns
- [ ] Track any duplicated patterns during `aster` development
- [ ] Generate deduplication report when significant overlap is identified
- [ ] Propose `oxur-cli-common` or similar support crate for shared utilities

## Notes

The goal is to build `aster` cleanly while identifying opportunities for code reuse and abstraction. Don't prematurely optimize, but do track patterns so we can refactor smartly once we understand both CLIs' needs.
