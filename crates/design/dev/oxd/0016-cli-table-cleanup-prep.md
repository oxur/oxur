# Prompt for Future Claude: Tabled Theme Cleanup and Generalization

## Context: What We Accomplished

We successfully created a custom theming system for Rust's `tabled` crate that produces beautiful, consistently-formatted terminal tables with:

- Full-width rows with proper background color padding
- Custom title, header, data, and footer row styling
- Per-cell foreground color overrides (e.g., colored state values)
- Alternating row colors
- Configurable via TOML themes

**The working solution is demonstrated in `./bin/oxd list`** - run it to see the final output.

## The Problem: It's a Mess

Through extensive trial and error, we discovered several critical insights about `tabled`:

1. **ANSI codes break width calculations**: Applying bold/colors via `Format::content()` after table building causes width inconsistencies
2. **Justification ≠ Width**: Justification only fills existing space; it doesn't set row width
3. **Builder calculates widths early**: Column widths are determined when `builder.build()` is called
4. **The solution**: Use `tabled::Cell` with `tabled::Color` (not `colored` crate) to apply colors AFTER theming, combining foreground + background colors

However, the code is a disaster:

- **`crates/design/src/commands/list.rs`**: Littered with commented-out attempts, helper functions that don't work, debug code
- **`crates/oxur-table/src/config.rs`**: Contains experimental width-setting code, multiple attempted approaches
- Both files have poor organization and lots of dead code
- We iterated on a lot of potential solutions and the code ended up getting quite complex; now, provably needlessly so -- we need to refactor to remove the complexity and present a clean, bare-bones solution that exacly duplicates the current, proper solution

## The Goal: Clean, General, Reusable Solution

We need to transform this working-but-messy solution into a clean, general-purpose library that can be used by ANY CLI command that outputs tables (not just `oxd list`).

### Requirements

1. **Clean up `oxur-table` crate** to be a general-purpose tabled wrapper with:
   - TOML-based theme configuration (already exists, needs cleanup)
   - A clean API for applying themes to tables
   - Built-in support for per-cell color overrides WITHOUT breaking width calculations
   - Helper functions for common patterns (colored cells, conditional formatting, etc.)

2. **Refactor `list.rs`** to use the cleaned-up `oxur-table` API:
   - Use clean, obvious function names
   - Demonstrate best practices for using the library
   - Update the `list --removed` code path to use the new solution
   - Remove all experimental/dead code

3. **Make it extensible** for future commands:
   - `oxd search` (will need tables)
   - `oxd debug` (will need tables)
   - `oxd info` (will need tables)
   - Any future table-based output

### Key Technical Constraints

**CRITICAL - These must be preserved in the clean solution:**

1. **Never use `colored` crate's `ColoredString` in table cell data** - it embeds ANSI codes that break width calculations
2. **Always use `tabled::Color` for cell coloring** - apply via `Cell::new(row, col)` or similar AFTER theme is applied
3. **For per-cell foreground colors, combine with row background**: `fg_color | bg_color` to preserve row styling
4. **Apply colors AFTER `apply_to_table()` theme application** - theme sets structure, then cell-specific colors override

### Current Working Pattern (to preserve)

```rust
// 1. Build table with PLAIN TEXT (no ANSI codes)
let mut builder = Builder::default();
builder.push_record(["Title", "", ""]);
builder.push_record(["Col1", "Col2", "Col3"]);
for item in items {
    builder.push_record([&item.field1, &item.field2, &item.field3]);
}
builder.push_record(["Footer", "", ""]);

// 2. Build table structure
let mut table = builder.build();

// 3. Apply theme (sets backgrounds, padding, justification)
let config = TableStyleConfig::default();
config.apply_to_table::<RowType>(&mut table);

// 4. Apply per-cell foreground colors (combining with row backgrounds)
for (i, item) in items.iter().enumerate() {
    let row_idx = 2 + i;
    if let Some(fg_color) = get_cell_color(&item.field3) {
        let color_idx = i % config.rows.colors.len();
        let bg_color = parse_bg_color(&config.rows.colors[color_idx].bg);
        table.modify(Cell::new(row_idx, 2), fg_color | bg_color);
    }
}

// 5. Print
println!("{}", table);
```

## Files That Need Cleanup

### Primary Files

1. **`crates/oxur-table/src/config.rs`** - Clean up experimental width code, improve documentation
2. **`crates/oxur-table/src/lib.rs`** - Add clean API for common use cases
3. **`crates/oxur-table/src/themes.rs`** - Already clean, may need minor updates
4. **`crates/design/src/commands/list.rs`** - Major cleanup needed, refactor to use clean API

### Supporting Files (may need updates)

1. **`crates/oxur-table/Cargo.toml`** - Verify dependencies
2. **`crates/design/src/commands/` (other files)** - Future table-using commands
3. **`crates/design/src/theme.rs`** - new, `oxd`-specific, generally useful coloured table code may need to be moved here

## Deliverables

1. **Clean `oxur-table` crate** with:
   - Well-documented public API
   - Helper functions for common patterns
   - Examples in module-level docs
   - No dead/experimental code

2. **Clean `list.rs`** demonstrating:
   - How to use the oxur-table API cleanly
   - Best practices for conditional cell coloring
   - Clear, readable code

3. **Usage examples** showing:
   - Basic table with theme
   - Table with per-cell colors
   - Table with conditional formatting
   - How to add new themed tables to other commands

4. **Documentation** including:
   - README for oxur-table crate
   - Inline documentation for all public APIs
   - Examples of common patterns

## Instructions for Creating the Claude Code Implementation Plan

When creating the implementation plan for Claude Code, follow these steps:

### Step 1: Assess Current State

1. Read and analyze ALL the files mentioned above
2. Identify ALL pieces of working code that must be preserved
3. Identify ALL dead/experimental code that should be removed
4. Map out the current dependencies between files

### Step 2: Design Clean Architecture

1. Design the public API for `oxur-table` crate:
   - What structs/traits should be public?
   - What helper functions are needed?
   - What's the simplest possible API for the 80% use case?

2. Identify reusable patterns:
   - Per-cell coloring with background preservation
   - Conditional formatting based on cell values
   - Building tables with title/header/data/footer

3. Design the refactored `list.rs`:
   - What should it look like using the clean API?
   - How much simpler can it be?

### Step 3: Create Detailed Implementation Plan

Write a Claude Code implementation plan with these sections:

#### Section 1: Preparation

- Files to read for context
- Current behavior to verify (run `./bin/oxd list` to see working output)
- Test strategy (how to verify nothing breaks)

#### Section 2: oxur-table Crate Cleanup

**For each file in oxur-table:**

- What to remove (dead code, experiments)
- What to keep (working solutions)
- What to add (new helper functions)
- What to rename/reorganize (better naming)

**Specific tasks:**

- Clean up `config.rs`: Remove experimental Width code, improve docs
- Enhance `lib.rs`: Add builder pattern or helper functions for common cases
- Add `helpers.rs`: New module for cell coloring, conditional formatting
- Update `themes.rs`: Document theme format clearly

#### Section 3: list.rs Refactoring

**Step-by-step refactoring:**

1. Remove all commented-out code
2. Remove unused helper functions
3. Replace direct tabled API calls with oxur-table helpers
4. Simplify the main function logic
5. Add clear comments explaining the flow

#### Section 4: Testing & Verification

**How to verify the refactoring worked:**

1. Run `cargo build` - must compile without errors
2. Run `./bin/oxd list` - output must be IDENTICAL to current output
3. Run `cargo test` - all tests must pass
4. Test with different terminal widths
5. Test with different numbers of rows

#### Section 5: Documentation

**What documentation to add:**

1. Module-level docs for oxur-table crate
2. README.md for oxur-table with examples
3. Inline docs for all public APIs
4. Example code in lib.rs showing common patterns

### Step 4: Structure the Plan for Claude Code

Format the plan as:

```markdown
# Implementation Plan: Clean Up and Generalize Tabled Theme System

## Overview
[High-level summary of what we're doing and why]

## Success Criteria
[Specific, testable criteria - e.g., "oxd list output is identical"]

## Phase 1: Assess Current State
### Task 1.1: Read and analyze files
- [ ] Read crates/oxur-table/src/config.rs
- [ ] Read crates/oxur-table/src/lib.rs
- [ ] Read crates/design/src/commands/list.rs
- [ ] Document current working solution

### Task 1.2: Run tests
- [ ] Run ./bin/oxd list and save output
- [ ] Run cargo test
- [ ] Document current behavior

## Phase 2: Clean oxur-table Crate
### Task 2.1: Clean config.rs
- [ ] Remove lines X-Y (experimental Width::list code)
- [ ] Remove lines A-B (dead code)
- [ ] Add documentation to apply_to_table method
- [ ] Make parse_bg_color public (needed by list.rs)

### Task 2.2: Add helpers.rs
- [ ] Create new file crates/oxur-table/src/helpers.rs
- [ ] Add function: apply_cell_colors(table, row_idx, col_idx, fg, config)
- [ ] Add function: get_row_bg_color(row_idx, config) -> Color
- [ ] Export from lib.rs

### Task 2.3: Enhance lib.rs
- [ ] Add re-exports for common types
- [ ] Add module-level documentation
- [ ] Add usage examples

## Phase 3: Refactor list.rs
### Task 3.1: Remove dead code
- [ ] Remove commented-out functions (lines X-Y)
- [ ] Remove unused imports
- [ ] Remove experimental helper functions

### Task 3.2: Simplify using oxur-table helpers
- [ ] Replace apply_state_cell_colors with oxur_table::helpers::apply_cell_colors
- [ ] Use helper to get row background colors
- [ ] Simplify main function logic

## Phase 4: Testing
### Task 4.1: Verify functionality
- [ ] cargo build (must succeed)
- [ ] ./bin/oxd list (output must be identical)
- [ ] cargo test (all pass)

## Phase 5: Documentation
### Task 5.1: Add oxur-table docs
- [ ] README.md with examples
- [ ] Module docs in lib.rs
- [ ] Inline docs for public APIs

## Phase 6: Validation
- [ ] Review all changes
- [ ] Ensure no dead code remains
- [ ] Verify public API is clean and intuitive
- [ ] Confirm list.rs is readable and maintainable
```

### Step 5: Include Critical Reminders

**Add this section to the plan:**

## CRITICAL: Do Not Break These Working Solutions

1. **Cell coloring pattern**: `Cell::new(row_idx, col_idx)` with `fg | bg`
2. **No ANSI codes in cell data**: Use tabled::Color only
3. **Order of operations**: Build → Theme → Cell colors
4. **Background preservation**: Always combine fg with row bg color

## Files to Preserve Exactly

- `crates/oxur-table/src/themes.rs` - Working theme, touch only if needed
- Theme TOML format - Don't change the schema

## Testing Before Committing

Before considering any phase complete:

1. Run `./bin/oxd list` and compare output visually
2. All rows must be same width
3. Background colors must extend full width
4. State colors (green/yellow/red) must show correctly

---

## Summary for Future Claude

You're being asked to:

1. **Clean up** a working but messy tabled theming solution
2. **Generalize** it into a reusable library (oxur-table crate)
3. **Refactor** the first user (list.rs) to be clean and exemplary
4. **Document** everything so future commands can easily use it
5. **Preserve** the working solution - output must be identical

The code works perfectly now, but it's a mess. Make it beautiful while keeping it functional.

**Start by reading all the files mentioned, running `./bin/oxd list` to see the working output, then create a detailed implementation plan for Claude Code following the structure above.**
