---
number: 15
title: "oxur-table API (re)Design"
author: "Duncan McGreggor & Claude"
component: Utility
tags: [ascii, ansi, theme, terminal, cli, ui]
created: 2025-12-28
updated: 2025-12-28
state: Accepted
supersedes: null
superseded-by: null
version: 1.0
---

# oxur-table API Design

Note that this is "Phase 2" of a cleanup effort being done after a wild and messy few days figuring out how to best theme and use the tabled Rust library.

## Design Principles

1. **Make simple things simple**: The 80% use case (basic themed table) should be one line
2. **Make complex things possible**: Cell-specific styling should be achievable with helpers
3. **Separation of concerns**: Theme = visual styling, Layout = widths/structure
4. **Type safety**: Leverage Rust's type system to prevent incorrect usage
5. **No hardcoded values**: Remove the hardcoded `Width::list([10, 75, 15])`

## Proposed Module Structure

```
crates/oxur-table/src/
├── lib.rs           # Public API and re-exports
├── config.rs        # Theme configuration (cleaned up)
├── themes.rs        # Embedded themes (no changes needed)
└── helpers.rs       # NEW: Helper functions for advanced usage
```

## API Design

### Simple Case (80% Use Case)

```rust
use oxur_table::{OxurTable, Tabled};

#[derive(Tabled)]
struct MyRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
}

let data = vec![
    MyRow { id: "001".to_string(), name: "Alice".to_string() },
    MyRow { id: "002".to_string(), name: "Bob".to_string() },
];

// That's it! One line to render with default Oxur theme
let output = OxurTable::new(data).render();
println!("{}", output);
```

### Advanced Case (20% Use Case) - Cell-Specific Coloring

```rust
use oxur_table::{OxurTable, Tabled, helpers};
use tabled::builder::Builder;

// Build table manually (not from Tabled struct)
let mut builder = Builder::default();
builder.push_record(["DESIGN", "DOCUMENTS", ""]);
builder.push_record(["Number", "Title", "State"]);

for doc in &docs {
    builder.push_record([
        &format!(" {:04}", doc.metadata.number),
        &doc.metadata.title,
        &doc.metadata.state,
    ]);
}

builder.push_record(["Total:", &format!("{} documents", docs.len()), ""]);

let mut table = builder.build();

// Apply theme to get base styling
let theme = TableStyleConfig::default();
theme.apply_to_table_builder(&mut table);

// Get reference to theme's row colors for helpers
let row_bg_colors = helpers::parse_row_bg_colors(&theme);

// Apply cell-specific foreground colors
for (i, doc) in docs.iter().enumerate() {
    let row_idx = 2 + i;  // Data rows start at index 2

    if let Some(fg_color) = helpers::state_to_fg_color(&doc.metadata.state) {
        // Get the alternating background color for this row
        let bg_color = helpers::get_data_row_bg_color(i, &row_bg_colors);

        // Apply combined color to specific cell
        helpers::apply_cell_color(&mut table, row_idx, 2, fg_color, bg_color);
    }
}

println!("{}", table);
```

## New Public API (lib.rs)

### Re-exports for Advanced Usage

```rust
// Current re-exports (keep these)
pub use config::TableStyleConfig;
pub use tabled::Tabled;

// NEW re-exports for advanced usage
pub use tabled::builder::Builder;
pub use tabled::settings::object::Cell;
pub use tabled::settings::Color as TabledColor;

// NEW: helpers module
pub mod helpers;
```

### OxurTable (Keep Current Implementation)

```rust
pub struct OxurTable<T: Tabled> {
    data: Vec<T>,
    theme: TableStyleConfig,
}

impl<T: Tabled> OxurTable<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self {
            data,
            theme: TableStyleConfig::default(),
        }
    }

    pub fn render(self) -> String {
        let mut table = Table::new(&self.data);
        self.theme.apply_to_table::<T>(&mut table);
        table.to_string()
    }
}
```

**Note**: No changes needed to `OxurTable` - it already handles the simple case perfectly.

## New helpers.rs Module

```rust
use tabled::builder::Builder;
use tabled::settings::{
    object::Cell,
    Color as TabledColor,
    Modify,
};
use crate::config::{TableStyleConfig, parse_bg_color, parse_fg_color};

/// Parse row background colors from theme config
/// Returns a Vec of background colors for alternating rows
pub fn parse_row_bg_colors(theme: &TableStyleConfig) -> Vec<TabledColor> {
    theme.rows.colors
        .iter()
        .map(|rc| parse_bg_color(&rc.bg))
        .collect()
}

/// Get the background color for a data row (0-indexed within data section)
/// Uses modulo to handle alternating row colors
pub fn get_data_row_bg_color(
    data_row_index: usize,
    row_bg_colors: &[TabledColor],
) -> TabledColor {
    let color_idx = data_row_index % row_bg_colors.len();
    row_bg_colors[color_idx].clone()
}

/// Apply a foreground color to a cell while preserving its background
pub fn apply_cell_color(
    table: &mut Builder,
    row_idx: usize,
    col_idx: usize,
    fg_color: TabledColor,
    bg_color: TabledColor,
) {
    let combined = fg_color | bg_color;
    table.modify(Cell::new(row_idx, col_idx), combined);
}

/// Map state string to foreground color (domain-specific helper example)
/// Note: This could live in the consuming crate instead
pub fn state_to_fg_color(state: &str) -> Option<TabledColor> {
    match state.to_lowercase().as_str() {
        "draft" => Some(TabledColor::FG_YELLOW),
        "under review" | "under-review" => Some(TabledColor::FG_CYAN),
        "revised" => Some(TabledColor::FG_BLUE),
        "accepted" => Some(TabledColor::FG_GREEN),
        "active" => Some(TabledColor::FG_BRIGHT_GREEN),
        "final" => Some(TabledColor::FG_GREEN),
        "deferred" => Some(TabledColor::FG_MAGENTA),
        "rejected" => Some(TabledColor::FG_RED),
        "withdrawn" => Some(TabledColor::FG_RED),
        "superseded" => Some(TabledColor::FG_RED),
        _ => None,
    }
}

/// Map deleted boolean to foreground color (domain-specific helper example)
pub fn deleted_to_fg_color(deleted: bool) -> TabledColor {
    if deleted {
        TabledColor::FG_RED
    } else {
        TabledColor::FG_GREEN
    }
}
```

## Changes to config.rs

### Critical Fix: Remove Hardcoded Width

**Current (WRONG)**:

```rust
// Line 151 in config.rs
table.with(Width::list([10, 75, 15]));  // ← HARDCODED!
```

**Solution**: **Remove this line entirely**

**Rationale**:

- Column widths are **layout concerns**, not **theme concerns**
- The theme should handle visual styling (colors, padding, borders)
- If widths are needed, the caller can apply them after theme application
- This makes oxur-table truly general-purpose

### Rename Method for Clarity

**Current**:

```rust
pub fn apply_to_table<T: Tabled>(&self, table: &mut Table) { ... }
```

**Proposed**: Keep the name but add a second method:

```rust
// For tables built from Tabled structs
pub fn apply_to_table<T: Tabled>(&self, table: &mut Table) { ... }

// For tables built with Builder (manual construction)
pub fn apply_to_table_builder(&self, table: &mut Builder) { ... }
```

**Wait, problem**: Looking at the code, `apply_to_table()` already works with `Builder` - it takes a `&mut Table` which is what Builder returns. So we don't need two methods. The generic `<T: Tabled>` is only used for row counting, not for the table parameter.

**Better solution**: Keep one method, just fix the signature:

```rust
pub fn apply_to_table(&self, table: &mut Table, row_count: usize) { ... }
```

Actually, looking more carefully, the current signature already works fine. The issue is just the hardcoded width. Let's keep it simple.

## Migration Path for list.rs

### Before (Current Messy Code)

```rust
// Multiple dead functions, ColoredString in structs, etc.
// See lines 19-153 in current list.rs
```

### After (Clean Code)

```rust
use oxur_table::{helpers, TableStyleConfig, Builder, Cell, TabledColor};

// Clean helper function using new helpers module
fn apply_state_cell_colors(
    table: &mut Table,
    docs: &[&design::doc::DesignDoc],
    theme: &TableStyleConfig,
) {
    let row_bg_colors = helpers::parse_row_bg_colors(theme);

    for (i, doc) in docs.iter().enumerate() {
        let row_idx = 2 + i;

        if let Some(fg_color) = helpers::state_to_fg_color(&doc.metadata.state) {
            let bg_color = helpers::get_data_row_bg_color(i, &row_bg_colors);
            helpers::apply_cell_color(table, row_idx, 2, fg_color, bg_color);
        }
    }
}
```

**Dead code removed**:

- `preserve_bg()` - Experimental, doesn't work
- `apply_title_formatting()` - Wrong approach
- `apply_removed_doc_number_formatting()` - Wrong approach
- `apply_removed_date_formatting()` - Wrong approach
- `apply_deleted_status_formatting()` - Wrong approach
- Structs with `ColoredString` fields - Violates the rule

## Decision Points

### 1. Should state_to_fg_color() be in oxur-table or in design crate?

**Option A**: In oxur-table as an example/utility

- Pro: Shows users how to write their own mappers
- Pro: Useful for other projects with similar states
- Con: Domain-specific to design doc workflows

**Option B**: In design crate only

- Pro: Domain logic stays in domain crate
- Pro: oxur-table is purely presentational
- Con: Less helpful as an example

**Recommendation**: Keep in oxur-table for now as a helper/example, document that users should copy and customize for their domain.

### 2. Should we keep parse_bg_color() and parse_fg_color() public?

**Current**: `parse_bg_color()` is public (line 411), `parse_fg_color()` is public (line 349)

**Recommendation**: Keep both public - they're needed by helpers module and potentially by advanced users.

## Success Criteria for Phase 2

- [ ] Module structure designed (lib.rs, config.rs, themes.rs, helpers.rs)
- [ ] API supports both simple and advanced use cases
- [ ] No hardcoded values remain in config.rs
- [ ] Helper functions enable cell-specific coloring without violating ANSI rules
- [ ] Clear separation between theme (visual) and layout (widths)
- [ ] Type-safe and idiomatic Rust API

## Next: Phase 3 Implementation

Once this design is approved, Phase 3 will implement:

1. Create helpers.rs with the designed functions
2. Remove hardcoded Width from config.rs
3. Update lib.rs with new re-exports
4. Add tests for helpers
