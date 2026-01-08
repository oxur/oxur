# Implementation Plan: Clean Up and Generalize Tabled Theme System

## Overview

This plan details the cleanup and generalization of the working-but-messy tabled theming solution in the Oxur project. The current implementation in `crates/design/src/commands/list.rs` produces beautiful, consistently-formatted terminal tables but contains significant technical debt from iterative development.

**Goal**: Transform the working solution into a clean, general-purpose library (`oxur-table`) that can be used by any CLI command requiring table output, while preserving exact functionality.

**Critical Constraint**: The output of `./bin/oxd list` must be IDENTICAL after refactoring.

## Success Criteria

1. ✅ `cargo build` compiles without errors
2. ✅ `./bin/oxd list` output is **pixel-perfect identical** to current output
3. ✅ `cargo test` passes all tests
4. ✅ No dead code, experimental code, or commented-out sections remain
5. ✅ `oxur-table` crate has clean, documented public API
6. ✅ `list.rs` demonstrates best practices using the clean API
7. ✅ Code is maintainable and extensible for future table-based commands

## Current State Analysis

### Working Solution (MUST PRESERVE)

The current implementation uses this pattern:

```rust
// 1. Build table with PLAIN TEXT (no ANSI codes)
let mut builder = Builder::default();
builder.push_record(["Title", "", ""]);
builder.push_record(["Col1", "Col2", "Col3"]);
for item in items {
    builder.push_record([plain_text_only]);
}
builder.push_record(["Footer", "", ""]);

// 2. Build table structure
let mut table = builder.build();

// 3. Apply theme (backgrounds, padding, justification)
let config = TableStyleConfig::default();
config.apply_to_table::<RowType>(&mut table);

// 4. Apply per-cell foreground colors (combining with row backgrounds)
for (i, item) in items.iter().enumerate() {
    let row_idx = 2 + i;
    if let Some(fg_color) = get_cell_color(&item.field) {
        let color_idx = i % config.rows.colors.len();
        let bg_color = parse_bg_color(&config.rows.colors[color_idx].bg);
        table.modify(Cell::new(row_idx, 2), fg_color | bg_color);
    }
}

// 5. Print
println!("{}", table);
```

### Critical Technical Insights (NEVER VIOLATE)

1. **ANSI codes break width calculations**: Never use `colored::ColoredString` in table cell data
2. **Always use `tabled::Color`**: Apply via `Cell::new(row, col)` AFTER theme
3. **Combine foreground with background**: `fg_color | bg_color` preserves row styling
4. **Order matters**: Build → Theme → Cell-specific colors
5. **No song lyrics, poems, or haikus**: Never reproduce complete creative works

### Problem Areas Identified

#### `crates/design/src/commands/list.rs` (556 lines)
**Issues**:
- Contains `preserve_bg()` function that strips ANSI codes - experimental leftover
- Multiple `apply_*_formatting()` functions that use `colored` crate - wrong approach
- Comments about width calculation issues throughout
- Dead code from experimental approaches
- Inconsistent patterns between `list_documents` and `list_removed_documents`

**Functions to Review**:
- `preserve_bg()` - Likely unnecessary with clean approach
- `apply_doc_number_formatting()` - Uses wrong approach (Format::content with colored)
- `apply_state_formatting()` - Uses wrong approach
- `apply_title_formatting()` - Uses wrong approach
- `apply_removed_doc_number_formatting()` - Uses wrong approach
- `apply_removed_date_formatting()` - Uses wrong approach
- `apply_deleted_status_formatting()` - Uses wrong approach

#### `crates/oxur-table/src/config.rs` (680 lines)
**Issues**:
- Contains experimental `Width::list([10, 75, 15])` code (line 110)
- Complex vertical separator coloring logic that may be over-engineered
- `parse_bg_color()` function not public but needed by `list.rs`
- Missing helper functions for common patterns

**Good Parts to Keep**:
- TOML deserialization structures
- Theme application framework
- Color parsing (both ANSI names and hex)
- Border coloring logic (if it works)

#### `crates/oxur-table/src/lib.rs` (44 lines)
**Issues**:
- Too minimal - needs helper functions
- No examples of per-cell coloring pattern
- No documentation of the proper workflow

**Needs**:
- Helper functions for applying cell-specific colors
- Documentation with examples
- Re-export necessary types from `tabled`

#### `crates/design/src/theme.rs` (143 lines)
**Status**: Clean, but may need state color mapping extraction

## Phase 1: Preparation and Assessment

### Task 1.1: Read and Analyze Current Implementation
**Duration**: 15 minutes

- [ ] Read `crates/design/src/commands/list.rs` completely
- [ ] Read `crates/oxur-table/src/config.rs` completely
- [ ] Read `crates/oxur-table/src/lib.rs` completely
- [ ] Read `crates/oxur-table/src/themes.rs` completely
- [ ] Read `crates/design/src/theme.rs` completely
- [ ] Document exact current behavior in notes

**Deliverable**: Written summary of:
- What works and must be preserved
- What's experimental/dead code
- Dependencies between files

### Task 1.2: Capture Current Output
**Duration**: 5 minutes

- [ ] Run `./bin/oxd list` and save complete output to file
- [ ] Run `./bin/oxd list --removed` and save output
- [ ] Run `./bin/oxd list --verbose` and save output
- [ ] Run `cargo test` and verify all tests pass
- [ ] Document current test coverage

**Deliverable**: Baseline output files for comparison

### Task 1.3: Identify the Minimal Working Solution
**Duration**: 10 minutes

From the current code, extract:
- [ ] Exact steps in the working pattern
- [ ] Which tabled types/methods are essential
- [ ] Which color combinations work
- [ ] Row index calculations (title=0, header=1, data=2+, footer=last)

**Deliverable**: Documented minimal pattern that works

## Phase 2: Design Clean Architecture

### Task 2.1: Design oxur-table Public API
**Duration**: 20 minutes

Design the ideal API that:
- Makes the 80% use case trivial
- Hides complexity but allows customization
- Prevents misuse (e.g., can't accidentally use ColoredString in data)

**Proposed API**:

```rust
// Simple case - just theme
pub struct OxurTable<T: Tabled> {
    data: Vec<T>,
    theme: TableStyleConfig,
}

impl<T: Tabled> OxurTable<T> {
    pub fn new(data: Vec<T>) -> Self;
    pub fn render(self) -> String;
}

// Advanced case - cell coloring
pub mod helpers {
    /// Get the background color for a data row index
    pub fn get_row_bg_color(row_idx: usize, config: &TableStyleConfig) -> Color;
    
    /// Apply foreground color to a cell, preserving row background
    pub fn apply_cell_color(
        table: &mut Table,
        row_idx: usize,
        col_idx: usize,
        fg_color: Color,
        config: &TableStyleConfig,
    );
    
    /// Common state color mappings
    pub fn state_to_color(state: &str) -> Option<Color>;
}
```

**Deliverable**: Documented API design with examples

### Task 2.2: Plan Code Organization
**Duration**: 10 minutes

Decide on module structure:
- [ ] What stays in `config.rs`?
- [ ] Create `helpers.rs` for cell coloring utilities?
- [ ] Update `lib.rs` to expose clean API
- [ ] Keep `themes.rs` as-is?

**Deliverable**: File-by-file responsibility matrix

## Phase 3: Clean Up oxur-table Crate

### Task 3.1: Clean config.rs
**Duration**: 30 minutes

**Remove**:
- [ ] Line 110: `Width::list([10, 75, 15])` - this is oxd-specific, not general
- [ ] Any other oxd-specific hardcoded values
- [ ] Commented-out experiments (if any)

**Keep**:
- [ ] All TOML deserialization structures
- [ ] `apply_to_table()` method (core of theme system)
- [ ] All color parsing functions (`parse_color`, `parse_bg_color`, `parse_hex_color`)
- [ ] Border coloring logic

**Modify**:
- [ ] Make `parse_bg_color()` public (needed for cell coloring)
- [ ] Add comprehensive doc comments to `apply_to_table()`
- [ ] Add doc comments to all public functions
- [ ] Clarify which row indices are which (title=0, header=1, data=2+, footer=last)

**Add**:
- [ ] Example in doc comment showing the proper usage pattern
- [ ] Note about ANSI codes breaking width calculations

**Deliverable**: Clean `config.rs` with no dead code and clear documentation

### Task 3.2: Create helpers.rs
**Duration**: 45 minutes

Create new file `crates/oxur-table/src/helpers.rs`:

```rust
//! Helper functions for common table styling patterns

use tabled::{
    settings::{
        object::{Cell},
        Color,
    },
    Table,
};
use crate::config::{TableStyleConfig, parse_bg_color};

/// Get the background color for a data row at the given index
///
/// Data rows start at index 2 (after title row 0 and header row 1).
/// This function accounts for alternating row colors.
///
/// # Arguments
/// * `data_row_idx` - The row index in the table (2 for first data row, 3 for second, etc.)
/// * `config` - The table style configuration
///
/// # Returns
/// The background Color for that row based on alternating pattern
pub fn get_data_row_bg_color(data_row_idx: usize, config: &TableStyleConfig) -> Color {
    // Data rows start at index 2 (row 0 = title, row 1 = header)
    // Calculate which color in the alternating pattern to use
    let data_row_offset = data_row_idx - 2;
    let color_idx = data_row_offset % config.rows.colors.len();
    parse_bg_color(&config.rows.colors[color_idx].bg)
}

/// Apply a foreground color to a specific cell while preserving its row background
///
/// This is the correct pattern for cell-specific coloring in tabled.
/// NEVER use `colored::ColoredString` in cell data as it breaks width calculations.
///
/// # Arguments
/// * `table` - The table to modify
/// * `row_idx` - The row index of the cell
/// * `col_idx` - The column index of the cell
/// * `fg_color` - The foreground color to apply
/// * `config` - The table configuration (used to get row background)
///
/// # Examples
/// ```no_run
/// use oxur_table::helpers::apply_cell_fg_color;
/// use tabled::settings::Color;
///
/// // Color a cell green while preserving row background
/// apply_cell_fg_color(&mut table, 2, 1, Color::FG_GREEN, &config);
/// ```
pub fn apply_cell_fg_color(
    table: &mut Table,
    row_idx: usize,
    col_idx: usize,
    fg_color: Color,
    config: &TableStyleConfig,
) {
    let bg_color = get_data_row_bg_color(row_idx, config);
    table.modify(Cell::new(row_idx, col_idx), fg_color | bg_color);
}

/// Map common state strings to appropriate foreground colors
///
/// Provides standard color scheme for document/item states.
/// Returns None if the state doesn't have a specific color mapping.
///
/// # Supported States
/// - Draft: Yellow
/// - Under Review: Cyan
/// - Revised: Blue
/// - Accepted/Active/Final: Green
/// - Deferred: Magenta
/// - Rejected/Withdrawn/Superseded: Red
///
/// # Examples
/// ```
/// use oxur_table::helpers::state_to_fg_color;
/// use tabled::settings::Color;
///
/// if let Some(color) = state_to_fg_color("draft") {
///     // color is Color::FG_YELLOW
/// }
/// ```
pub fn state_to_fg_color(state: &str) -> Option<Color> {
    match state.to_lowercase().as_str() {
        "draft" => Some(Color::FG_YELLOW),
        "under review" | "under-review" => Some(Color::FG_CYAN),
        "revised" => Some(Color::FG_BLUE),
        "accepted" | "active" | "final" => Some(Color::FG_GREEN),
        "deferred" => Some(Color::FG_MAGENTA),
        "rejected" | "withdrawn" | "superseded" => Some(Color::FG_RED),
        _ => None,
    }
}

/// Map boolean deleted status to appropriate foreground colors
///
/// # Returns
/// - `true` (deleted): Red
/// - `false` (exists): Green
pub fn deleted_to_fg_color(deleted: bool) -> Color {
    if deleted {
        Color::FG_RED
    } else {
        Color::FG_GREEN
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Add comprehensive tests for each helper function
}
```

**Tasks**:
- [ ] Create `helpers.rs` with functions above
- [ ] Add comprehensive unit tests for each helper
- [ ] Add doc comments with examples
- [ ] Export from `lib.rs`

**Deliverable**: New `helpers.rs` module with tested, documented utilities

### Task 3.3: Enhance lib.rs
**Duration**: 30 minutes

**Modify `crates/oxur-table/src/lib.rs`**:

```rust
//! Styled table rendering for Oxur tools
//!
//! Provides a flexible table builder with TOML-based theming for terminal output.
//!
//! # Quick Start
//!
//! ```no_run
//! use oxur_table::{OxurTable, Tabled};
//!
//! #[derive(Tabled)]
//! struct Employee {
//!     #[tabled(rename = "Name")]
//!     name: String,
//!     #[tabled(rename = "Role")]
//!     role: String,
//! }
//!
//! let employees = vec![
//!     Employee { name: "Alice".into(), role: "Engineer".into() },
//! ];
//!
//! let table = OxurTable::new(employees).render();
//! println!("{}", table);
//! ```
//!
//! # Advanced Usage: Per-Cell Coloring
//!
//! For tables that need cell-specific colors (like status badges):
//!
//! ```no_run
//! use oxur_table::{TableStyleConfig, helpers};
//! use tabled::{builder::Builder, settings::Color};
//!
//! // 1. Build with PLAIN TEXT only
//! let mut builder = Builder::default();
//! builder.push_record(["TITLE", "", ""]);
//! builder.push_record(["Name", "Role", "Status"]);
//! builder.push_record(["Alice", "Engineer", "active"]);
//! builder.push_record(["Bob", "Designer", "draft"]);
//! builder.push_record(["Total: 2", "", ""]);
//!
//! // 2. Build table
//! let mut table = builder.build();
//!
//! // 3. Apply theme
//! let config = TableStyleConfig::default();
//! config.apply_to_table::<YourRowType>(&mut table);
//!
//! // 4. Apply cell-specific colors
//! if let Some(color) = helpers::state_to_fg_color("active") {
//!     helpers::apply_cell_fg_color(&mut table, 2, 2, color, &config);
//! }
//! if let Some(color) = helpers::state_to_fg_color("draft") {
//!     helpers::apply_cell_fg_color(&mut table, 3, 2, color, &config);
//! }
//!
//! println!("{}", table);
//! ```
//!
//! # Critical Rules
//!
//! - **NEVER** use `colored::ColoredString` in cell data - it breaks width calculations
//! - **ALWAYS** use plain text when building the table
//! - **ALWAYS** apply theme before cell-specific colors
//! - **ALWAYS** combine foreground with background using `fg_color | bg_color`
//!
//! # Architecture
//!
//! The table has this structure (row indices):
//! - Row 0: Title (optional, enabled via config)
//! - Row 1: Header
//! - Rows 2+: Data rows
//! - Last row: Footer (optional, enabled via config)

use tabled::Table;

mod config;
pub mod helpers;
mod themes;

pub use config::TableStyleConfig;
pub use tabled::Tabled; // Re-export for convenience
pub use tabled::builder::Builder; // Re-export for advanced usage
pub use tabled::settings::Color; // Re-export for cell coloring

// ... rest of OxurTable implementation stays the same ...
```

**Tasks**:
- [ ] Add comprehensive module-level documentation
- [ ] Add "Critical Rules" section
- [ ] Add examples for both simple and advanced usage
- [ ] Re-export necessary types from tabled
- [ ] Export `helpers` module

**Deliverable**: Enhanced `lib.rs` with complete documentation and examples

### Task 3.4: Create README for oxur-table
**Duration**: 20 minutes

Create `crates/oxur-table/README.md`:

```markdown
# oxur-table

Styled table rendering for Oxur tools with the warm orange "Oxur" theme.

## Overview

`oxur-table` provides a simple, ergonomic API for creating beautifully styled terminal tables. It uses the [tabled](https://crates.io/crates/tabled) crate for core table rendering and includes an embedded default theme with warm orange sunset colors that match the Oxur brand.

## Features

- **Zero Configuration** - Works out-of-box with embedded default theme
- **Type-Safe** - Generic over any type implementing `Tabled`
- **Correct Cell Coloring** - Properly handles per-cell colors without breaking layout
- **Hex Colors** - Theme supports both ANSI color names and hex colors (`#RRGGBB`)
- **Row Styling** - Configurable header, title, data rows, and footer
- **Flexible Theming** - TOML-based theme system

## Quick Start

[Include examples from lib.rs]

## How It Works

The key insight for working with `tabled` is:

1. **Build with plain text** - Width calculations happen here
2. **Apply theme** - Sets backgrounds, padding, borders
3. **Apply cell colors** - Combine foreground with row background

## Common Patterns

### State Badges
### Deleted/Active Indicators
### Bold Numbers

## Theme Customization

[Document TOML format]

## Used By

- `oxd list` - Design document listing
- `oxd search` - Search results (planned)
- `oxd debug` - Debug info tables (planned)

## License

MIT OR Apache-2.0
```

**Tasks**:
- [ ] Create comprehensive README
- [ ] Include installation instructions
- [ ] Include API examples
- [ ] Document theme format
- [ ] Link to example usage in oxd

**Deliverable**: Complete README.md for the crate

## Phase 4: Refactor list.rs

### Task 4.1: Remove Dead Code and Experiments
**Duration**: 20 minutes

**Remove from `list.rs`**:
- [ ] `preserve_bg()` function (lines ~18-24) - Wrong approach
- [ ] `apply_doc_number_formatting()` - Wrong approach
- [ ] `apply_state_formatting()` - Wrong approach  
- [ ] `apply_title_formatting()` - Wrong approach
- [ ] `apply_removed_doc_number_formatting()` - Wrong approach
- [ ] `apply_removed_date_formatting()` - Wrong approach
- [ ] `apply_deleted_status_formatting()` - Wrong approach
- [ ] Any commented-out code
- [ ] Unused imports (especially `colored` usage in wrong places)

**Keep**:
- [ ] Test suite (will need updates)
- [ ] Core business logic (filtering, data preparation)
- [ ] Table structure (title, header, data, footer rows)

**Deliverable**: `list.rs` with only essential code remaining

### Task 4.2: Refactor list_documents() Using Clean API
**Duration**: 45 minutes

**Replace the table rendering with**:

```rust
fn list_documents_impl(
    index: &DocumentIndex,
    state_mgr: Option<&StateManager>,
    state_filter: Option<String>,
    verbose: bool,
    removed: bool,
) -> Result<()> {
    // [Existing filtering logic stays the same]
    
    if verbose {
        // [Keep existing verbose format - it's fine]
    } else {
        // NEW TABLE FORMAT
        
        // 1. Build table with PLAIN TEXT only
        let mut builder = Builder::default();
        
        // Row 0: Title
        builder.push_record(["DESIGN DOCUMENTS", "", ""]);
        
        // Row 1: Header
        builder.push_record(["Number", "Title", "State"]);
        
        // Rows 2+: Data rows - PLAIN TEXT ONLY
        for doc in &docs {
            builder.push_record([
                &format!("{:04}", doc.metadata.number),
                &doc.metadata.title,
                doc.metadata.state.as_str(),
            ]);
        }
        
        // Last row: Footer
        let total_text = format!("Total: {} documents", docs.len());
        builder.push_record([&total_text, "", ""]);
        
        // 2. Build the table structure
        let mut table = builder.build();
        
        // 3. Apply theme
        let config = TableStyleConfig::default();
        config.apply_to_table::<DocumentRow>(&mut table);
        
        // 4. Apply per-cell colors for state column
        for (i, doc) in docs.iter().enumerate() {
            let row_idx = 2 + i; // Data rows start at index 2
            
            // Color the state cell if we have a color mapping
            if let Some(fg_color) = helpers::state_to_fg_color(doc.metadata.state.as_str()) {
                helpers::apply_cell_fg_color(&mut table, row_idx, 2, fg_color, &config);
            }
        }
        
        println!();
        println!("{}", table);
        println!();
    }
    Ok(())
}
```

**Tasks**:
- [ ] Rewrite table rendering using Builder pattern with plain text
- [ ] Use `helpers::state_to_fg_color()` for state column
- [ ] Use `helpers::apply_cell_fg_color()` for applying colors
- [ ] Remove all direct usage of `colored` crate in table cells
- [ ] Verify row index calculations are correct
- [ ] Add clear comments explaining each step

**Deliverable**: Clean `list_documents_impl()` using oxur-table API

### Task 4.3: Refactor list_removed_documents() Using Clean API
**Duration**: 30 minutes

Apply same pattern to `list_removed_documents()`:

```rust
fn list_removed_documents(state_mgr: &StateManager, verbose: bool) -> Result<()> {
    // [Existing filtering and counting logic stays]
    
    // 1. Build table with PLAIN TEXT
    let mut builder = Builder::default();
    
    builder.push_record(["REMOVED DOCUMENTS", "", "", "", ""]);
    
    if verbose {
        builder.push_record(["Number", "Title", "Removed", "Deleted", "Dustbin Location"]);
    } else {
        builder.push_record(["Number", "Title", "Removed", "Deleted", ""]);
    }
    
    // Data rows - PLAIN TEXT
    for doc in &removed_docs {
        let file_exists = state_mgr.docs_dir().join(&doc.path).exists();
        // [Existing truncation logic]
        
        builder.push_record([
            &format!("{:04}", doc.metadata.number),
            &title_truncated,
            &doc.metadata.updated.to_string(),
            if file_exists { "false" } else { "true" },
            &location,
        ]);
    }
    
    // Footer
    builder.push_record([&total_text, "", "", "", ""]);
    
    // 2. Build table
    let mut table = builder.build();
    
    // 3. Apply theme
    let config = TableStyleConfig::default();
    config.apply_to_table::<RemovedDocRow>(&mut table);
    
    // 4. Apply cell-specific colors
    for (i, doc) in removed_docs.iter().enumerate() {
        let row_idx = 2 + i;
        let file_exists = state_mgr.docs_dir().join(&doc.path).exists();
        
        // Yellow number
        helpers::apply_cell_fg_color(&mut table, row_idx, 0, Color::FG_YELLOW, &config);
        
        // White date
        helpers::apply_cell_fg_color(&mut table, row_idx, 2, Color::FG_WHITE, &config);
        
        // Green/Red deleted status
        let deleted_color = helpers::deleted_to_fg_color(!file_exists);
        helpers::apply_cell_fg_color(&mut table, row_idx, 3, deleted_color, &config);
    }
    
    println!();
    println!("{}", table);
    println!();
    
    Ok(())
}
```

**Tasks**:
- [ ] Apply same refactoring pattern as `list_documents()`
- [ ] Use `helpers::deleted_to_fg_color()` for deleted column
- [ ] Use plain text in all cell data
- [ ] Verify output is identical to current

**Deliverable**: Clean `list_removed_documents()` using oxur-table API

### Task 4.4: Update Imports and Dependencies
**Duration**: 10 minutes

**Update imports in `list.rs`**:

```rust
use anyhow::Result;
use design::doc::DocState;
use design::index::DocumentIndex;
use design::state::StateManager;
use oxur_table::{TableStyleConfig, Builder, Color, helpers};
use tabled::Tabled;
```

**Remove**:
- [ ] `use colored::*;` - No longer needed in list.rs
- [ ] Any other unused imports

**Verify**:
- [ ] All necessary types are imported
- [ ] No unused imports remain

**Deliverable**: Clean, minimal import list

### Task 4.5: Update or Remove DocumentRow and RemovedDocRow
**Duration**: 15 minutes

**Current issue**: These structs use `#[derive(Tabled)]` but don't use the column names (they're empty strings). This is confusing.

**Options**:
1. Keep them but add proper column names
2. Remove them entirely since we're using Builder

**Recommended**: Keep them with proper names (needed for type parameter in `apply_to_table`):

```rust
/// Table row type for document list
#[derive(Tabled)]
struct DocumentRow {
    #[tabled(rename = "Number")]
    number: String,
    
    #[tabled(rename = "Title")]
    title: String,
    
    #[tabled(rename = "State")]
    state: String,
}

/// Table row type for removed documents
#[derive(Tabled)]
struct RemovedDocRow {
    #[tabled(rename = "Number")]
    number: String,
    
    #[tabled(rename = "Title")]
    title: String,
    
    #[tabled(rename = "Removed")]
    removed: String,
    
    #[tabled(rename = "Deleted")]
    deleted: String,
    
    #[tabled(rename = "Location")]
    location: String,
}
```

**Note**: We don't actually instantiate these, they're just type markers for `apply_to_table()`.

**Tasks**:
- [ ] Update struct definitions with proper column names
- [ ] Add doc comments explaining they're type markers
- [ ] Remove `ColoredString` types from fields

**Deliverable**: Clean struct definitions

## Phase 5: Testing and Verification

### Task 5.1: Build Verification
**Duration**: 10 minutes

- [ ] Run `cargo build` from project root
- [ ] Verify no compilation errors
- [ ] Verify no new warnings introduced
- [ ] Run `cargo clippy` and address any issues

**Success Criteria**: Clean build with no errors or warnings

### Task 5.2: Functional Testing
**Duration**: 20 minutes

- [ ] Run `./bin/oxd list` and capture output
- [ ] Compare byte-for-byte with baseline output from Phase 1
- [ ] Run `./bin/oxd list --removed` and compare
- [ ] Run `./bin/oxd list --verbose` and compare
- [ ] Run with `--state draft`, `--state final`, etc.
- [ ] Test with empty document list
- [ ] Test with single document
- [ ] Test with many documents

**Success Criteria**: Output is IDENTICAL for all test cases

### Task 5.3: Unit Test Updates
**Duration**: 30 minutes

Update tests in `list.rs`:
- [ ] Remove tests for deleted helper functions
- [ ] Add tests for new pattern if needed
- [ ] Verify all existing tests still pass
- [ ] Run `cargo test` and confirm all pass

Update/add tests in oxur-table:
- [ ] Test helpers::get_data_row_bg_color()
- [ ] Test helpers::apply_cell_fg_color()
- [ ] Test helpers::state_to_fg_color()
- [ ] Test helpers::deleted_to_fg_color()
- [ ] Verify all config.rs tests still pass

**Success Criteria**: `cargo test` passes with 100% success rate

### Task 5.4: Edge Case Testing
**Duration**: 15 minutes

Test corner cases:
- [ ] Documents with very long titles (>100 chars)
- [ ] Documents with special characters in title
- [ ] Documents with Unicode characters
- [ ] Empty state values
- [ ] All possible DocState variants

**Success Criteria**: No panics, clean handling of all cases

## Phase 6: Documentation

### Task 6.1: Add Module-Level Documentation
**Duration**: 20 minutes

**For each file, add/improve module docs**:

`crates/oxur-table/src/config.rs`:
```rust
//! Table style configuration and theme application
//!
//! This module provides the core theming system for oxur-table.
//! Themes are defined in TOML and can customize:
//! - Row colors (background and foreground)
//! - Padding
//! - Border characters and colors
//! - Title/header/footer styling
//!
//! # Theme Structure
//!
//! ```toml
//! [table]
//! padding_left = 0
//! ...
//! ```
```

`crates/oxur-table/src/helpers.rs`:
```rust
//! Helper functions for common table styling patterns
//!
//! This module provides utilities for the most common table styling needs:
//! - Applying cell-specific colors
//! - Mapping semantic states to colors
//! - Getting row background colors
//!
//! # The Golden Rule
//!
//! Never use `colored::ColoredString` in table cell data.
//! Always use plain text, then apply colors via these helpers.
```

**Tasks**:
- [ ] Add/improve module docs for config.rs
- [ ] Add module docs for helpers.rs
- [ ] Improve lib.rs module docs (done in Phase 3)
- [ ] Add module docs to themes.rs if needed

### Task 6.2: Add Inline Documentation
**Duration**: 30 minutes

**Ensure every public function has**:
- [ ] Summary description
- [ ] Parameter documentation
- [ ] Return value documentation
- [ ] At least one usage example
- [ ] Notes about important constraints

**Priority functions**:
- [ ] `TableStyleConfig::apply_to_table()`
- [ ] `helpers::apply_cell_fg_color()`
- [ ] `helpers::get_data_row_bg_color()`
- [ ] `helpers::state_to_fg_color()`
- [ ] All public functions in config.rs

### Task 6.3: Create Examples Directory
**Duration**: 20 minutes

Create `crates/oxur-table/examples/`:

**Example 1**: `simple_table.rs`
```rust
//! Simplest possible table with default theme

use oxur_table::{OxurTable, Tabled};

#[derive(Tabled)]
struct Employee {
    name: String,
    role: String,
}

fn main() {
    let employees = vec![
        Employee { name: "Alice".into(), role: "Engineer".into() },
        Employee { name: "Bob".into(), role: "Designer".into() },
    ];
    
    let table = OxurTable::new(employees).render();
    println!("{}", table);
}
```

**Example 2**: `colored_cells.rs`
```rust
//! Table with per-cell coloring (like status badges)

use oxur_table::{TableStyleConfig, Builder, Color, helpers};

fn main() {
    // Show the proper pattern for cell coloring
}
```

**Tasks**:
- [ ] Create examples directory
- [ ] Add simple_table.rs
- [ ] Add colored_cells.rs
- [ ] Add custom_theme.rs (using TOML file)
- [ ] Verify examples compile: `cargo build --examples`

### Task 6.4: Update CHANGELOG
**Duration**: 10 minutes

Add entry to `crates/oxur-table/CHANGELOG.md` (create if doesn't exist):

```markdown
## [Unreleased]

### Changed
- **BREAKING**: Removed experimental width-setting code from config.rs
- Made `parse_bg_color()` public for use in cell coloring
- Improved documentation throughout

### Added
- New `helpers` module with cell coloring utilities
- `helpers::apply_cell_fg_color()` - Apply foreground color to cell
- `helpers::get_data_row_bg_color()` - Get row background color
- `helpers::state_to_fg_color()` - Map state strings to colors
- `helpers::deleted_to_fg_color()` - Map boolean to colors
- Comprehensive examples in lib.rs and examples/
- README.md with usage guide

### Fixed
- Removed incorrect usage of `colored::ColoredString` in cell data
- Cleaned up experimental code and dead code paths

## [Previous version]
...
```

## Phase 7: Final Validation

### Task 7.1: Comprehensive Review
**Duration**: 20 minutes

**Check every file**:
- [ ] No commented-out code remains
- [ ] No TODO comments without context
- [ ] No experimental code
- [ ] Consistent code style
- [ ] All public APIs documented
- [ ] All tests passing

**Files to review**:
- [ ] `crates/oxur-table/src/config.rs`
- [ ] `crates/oxur-table/src/helpers.rs`
- [ ] `crates/oxur-table/src/lib.rs`
- [ ] `crates/oxur-table/src/themes.rs`
- [ ] `crates/design/src/commands/list.rs`

### Task 7.2: Before/After Comparison
**Duration**: 15 minutes

Create comparison document showing:
- [ ] Lines of code before/after in list.rs
- [ ] Number of functions before/after
- [ ] Complexity metrics (if available)
- [ ] Test coverage before/after

**Document improvements**:
- Reduced complexity
- Removed dead code
- Better separation of concerns
- Reusable components

### Task 7.3: Final Test Suite
**Duration**: 20 minutes

**Run complete test suite**:
- [ ] `cargo test` (all tests)
- [ ] `cargo test --package oxur-table` (table crate)
- [ ] `cargo test --package design` (design crate)
- [ ] `cargo build --release` (verify release build)
- [ ] `./bin/oxd list` (final output check)
- [ ] `cargo doc --open` (verify docs render correctly)

**Success Criteria**: Everything passes, output is identical

## Implementation Order

Execute phases in order:
1. Phase 1 (Preparation) - **MUST DO FIRST** to establish baseline
2. Phase 2 (Design) - Plan before coding
3. Phase 3 (oxur-table cleanup) - Build foundation
4. Phase 4 (list.rs refactor) - Use the foundation
5. Phase 5 (Testing) - Verify correctness
6. Phase 6 (Documentation) - Explain the solution
7. Phase 7 (Validation) - Final checks

## Critical Reminders

### NEVER VIOLATE THESE RULES

1. **Cell coloring pattern**: `Cell::new(row_idx, col_idx)` with `fg | bg`
2. **No ANSI codes in cell data**: Use `tabled::Color` only
3. **Order of operations**: Build → Theme → Cell colors
4. **Background preservation**: Always combine fg with row bg color
5. **Row indices**: Title=0, Header=1, Data=2+, Footer=last
6. **Plain text only**: When building table with Builder

### Files to Preserve Exactly

- `crates/oxur-table/src/themes.rs` - Working theme, minimal changes
- Theme TOML format - Don't change the schema
- Test suite structure - Update tests, don't remove coverage

### Testing Before Committing

Before considering ANY phase complete:

1. Run `./bin/oxd list` and compare output visually
2. All rows must be same width
3. Background colors must extend full width
4. State colors (green/yellow/red) must show correctly
5. `cargo test` must pass 100%

## Success Metrics

### Code Quality
- [ ] No dead code warnings
- [ ] No clippy warnings
- [ ] All public APIs documented
- [ ] Examples compile and run

### Functionality
- [ ] Output identical to baseline
- [ ] All tests pass
- [ ] No regressions in any command

### Maintainability
- [ ] Clear separation of concerns
- [ ] Reusable helper functions
- [ ] Well-documented patterns
- [ ] Easy to extend for new commands

## Next Steps (After Completion)

Once this cleanup is complete, the clean oxur-table API can be used for:

1. `oxd search` - Search results in table format
2. `oxd debug` - Debug information tables
3. `oxd info` - Metadata tables
4. Any future table-based output needs

The pattern established here becomes the standard for all table rendering in Oxur tools.

## Appendix A: File Dependency Map

```
crates/oxur-table/
├── src/
│   ├── lib.rs (public API, re-exports)
│   ├── config.rs (TableStyleConfig, apply_to_table, color parsing)
│   ├── helpers.rs (NEW - cell coloring utilities)
│   └── themes.rs (TOML theme definitions)
│
crates/design/
├── src/
│   ├── commands/
│   │   └── list.rs (USES oxur-table)
│   └── theme.rs (MAY use helpers::state_to_fg_color)
```

## Appendix B: Color Mapping Reference

### State Colors (helpers::state_to_fg_color)
- Draft → Yellow
- Under Review → Cyan
- Revised → Blue
- Accepted/Active/Final → Green
- Deferred → Magenta
- Rejected/Withdrawn/Superseded → Red
- Unknown → None (use default)

### Boolean Colors (helpers::deleted_to_fg_color)
- true → Red
- false → Green

## Appendix C: Row Index Calculation

**For tables built with this structure**:
```
Row 0: Title (optional via config)
Row 1: Header
Row 2: First data row
Row 3: Second data row
...
Row N-1: Last data row
Row N: Footer (optional via config)
```

**Calculating data row index**:
```rust
for (i, item) in items.iter().enumerate() {
    let row_idx = 2 + i;  // Data rows start at index 2
    // Apply colors to row_idx
}
```

**Calculating footer index**:
```rust
// If you need footer index before knowing total rows
let footer_idx = 2 + items.len();  // After all data rows
```

## Appendix D: Migration Guide for Future Commands

When adding a new command that needs tables:

1. **Define your row type**:
```rust
#[derive(Tabled)]
struct MyRow {
    #[tabled(rename = "Column 1")]
    field1: String,
    #[tabled(rename = "Column 2")]
    field2: String,
}
```

2. **Build table with plain text**:
```rust
let mut builder = Builder::default();
builder.push_record(["TITLE", ""]);
builder.push_record(["Col1", "Col2"]);
for item in items {
    builder.push_record([&item.field1, &item.field2]);
}
builder.push_record(["Footer", ""]);
```

3. **Apply theme and colors**:
```rust
let mut table = builder.build();
let config = TableStyleConfig::default();
config.apply_to_table::<MyRow>(&mut table);

for (i, item) in items.iter().enumerate() {
    if needs_coloring {
        helpers::apply_cell_fg_color(&mut table, 2 + i, col, color, &config);
    }
}

println!("{}", table);
```

---

**End of Implementation Plan**

This plan provides a complete roadmap for cleaning up and generalizing the tabled theme system. Execute phases in order, test thoroughly at each step, and maintain the cardinal rule: output must be identical after refactoring.
