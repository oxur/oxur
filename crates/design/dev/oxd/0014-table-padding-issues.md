# Claude Code Instructions: Fix Table Padding Issue

## Problem Summary

The table rows in `oxd list` command have inconsistent widths:
- Title/Header rows: 103 characters (correct)
- Data rows: 82-84 characters (incorrect)

**Root Cause**: ANSI escape codes (from `.bold()` and color formatting) are embedded in cell data BEFORE the table structure is built. This causes `tabled` to miscalculate column widths because it counts the invisible ANSI codes as characters.

**Solution**: Build the table with plain text, then apply formatting using `tabled`'s cell modifier system AFTER the table structure is created.

---

## Implementation Steps

### Step 1: Locate the Current Implementation

**File**: `crates/design/src/commands/list.rs`

Find the section where the table is built. It should look something like:

```rust
let mut builder = Builder::default();
builder.push_record(["DESIGN DOCUMENTS", "", ""]);
builder.push_record(["Number", "Title", "State"]);

for doc in docs {
    builder.push_record([
        doc_number().to_string(),      // ❌ Contains ANSI codes
        doc.metadata.title.clone(),
        state_badge().to_string(),     // ❌ Contains ANSI codes
    ]);
}
```

### Step 2: Add Required Imports

At the top of `list.rs`, add these imports if not already present:

```rust
use tabled::settings::object::{Columns, Rows, Segment};
use tabled::settings::{Modify, Format};
use colored::*;
```

### Step 3: Create Helper Functions

Add these helper functions to `list.rs` (after the imports, before the main function):

```rust
/// Apply bold formatting to document numbers (column 0)
fn apply_doc_number_formatting(table: &mut Table) {
    table.modify(
        Columns::single(0)
            .not(Rows::first())      // Skip title row
            .not(Rows::new(1..2)),   // Skip header row
        Format::content(|s| preserve_bg(s.bold()))
    );
}

/// Apply state badge colors and formatting (column 2)
fn apply_state_formatting(table: &mut Table) {
    table.modify(
        Columns::single(2)
            .not(Rows::first())      // Skip title row
            .not(Rows::new(1..2)),   // Skip header row
        Format::content(|state_str| {
            let colored = match state_str.to_lowercase().as_str() {
                "draft" => state_str.yellow(),
                "under review" | "under-review" => state_str.cyan(),
                "revised" => state_str.blue(),
                "accepted" => state_str.green(),
                "active" => state_str.green().bold(),
                "final" => state_str.green().bold(),
                "deferred" => state_str.magenta(),
                "rejected" => state_str.red(),
                "withdrawn" => state_str.red(),
                "superseded" => state_str.red(),
                _ => state_str.white(),
            };
            preserve_bg(colored)
        })
    );
}

/// Apply formatting to any date columns if present
fn apply_date_formatting(table: &mut Table, column_index: usize) {
    table.modify(
        Columns::single(column_index)
            .not(Rows::first())
            .not(Rows::new(1..2)),
        Format::content(|s| preserve_bg(s.dimmed()))
    );
}
```

**Note**: The `preserve_bg()` function should already exist in the file. If not, ensure it's defined:

```rust
/// Replace full ANSI reset with foreground-only reset to preserve background color
fn preserve_bg(colored: ColoredString) -> String {
    colored.to_string().replace("\x1b[0m", "\x1b[39m")
}
```

### Step 4: Refactor Table Building

**Find** the table building code and **replace** it with this:

```rust
let mut builder = Builder::default();

// Title row - PLAIN TEXT ONLY
builder.push_record(["DESIGN DOCUMENTS", "", ""]);

// Header row - PLAIN TEXT ONLY
builder.push_record(["Number", "Title", "State"]);

// Data rows - PLAIN TEXT ONLY (no formatting yet)
for doc in &docs {
    builder.push_record([
        format!("{:04}", doc.metadata.number),  // Plain text, no .bold()
        doc.metadata.title.clone(),
        doc.metadata.state.as_str().to_string(), // Plain text, no state_badge()
    ]);
}

// Footer row - PLAIN TEXT ONLY
builder.push_record(["", "", ""]);

// Build the table structure (width calculation happens here)
let mut table = builder.build();

// Apply the theme (background colors, padding, justification)
let theme = TableStyleConfig::default();
theme.apply_to_table::<String>(&mut table);

// NOW apply formatting - AFTER width calculation
apply_doc_number_formatting(&mut table);  // Column 0: bold numbers
apply_state_formatting(&mut table);        // Column 2: colored states

// Print the table
println!("{}", table);
```

### Step 5: Handle the "Removed Documents" Table (if present)

If there's a separate table for removed documents, apply the same pattern:

```rust
// Build removed docs table - PLAIN TEXT
let mut builder = Builder::default();
builder.push_record(["REMOVED DOCUMENTS", "", ""]);
builder.push_record(["Number", "Title", "State"]);

for doc in &removed_docs {
    builder.push_record([
        format!("{:04}", doc.metadata.number),
        doc.metadata.title.clone(),
        doc.metadata.state.as_str().to_string(),
    ]);
}

builder.push_record(["", "", ""]);

// Build and format
let mut table = builder.build();
let theme = TableStyleConfig::default();
theme.apply_to_table::<String>(&mut table);

apply_doc_number_formatting(&mut table);
apply_state_formatting(&mut table);

println!("{}", table);
```

### Step 6: Remove Old Helper Functions (if they exist)

**Search for and remove** these functions if they exist, as they're no longer needed:

- `doc_number()` - replaced by inline `format!("{:04}", doc.metadata.number)`
- `state_badge()` - replaced by `apply_state_formatting()`

Unless these functions are used elsewhere in the codebase, in which case keep them but don't use them in the table building.

### Step 7: Test the Changes

Run these commands to verify the fix:

```bash
# Build the project
cargo build

# Test the list command
CLICOLOR_FORCE=1 ./target/debug/oxd list

# Check row widths
CLICOLOR_FORCE=1 ./target/debug/oxd list | sed 's/\x1b\[[0-9;]*m//g' | awk '{print NR, length($0)}'
```

**Expected output**: All rows should have consistent width (103 characters).

### Step 8: Verify Visual Output

Run without stripping ANSI codes to verify formatting is preserved:

```bash
./target/debug/oxd list
```

**Verify**:
- ✅ Document numbers are bold
- ✅ State badges have correct colors
- ✅ Background color extends all the way across all rows
- ✅ Row widths are consistent

---

## Additional Considerations

### If there are more columns in your actual table

If your table has additional columns (like dates, authors, etc.), add more formatting functions:

```rust
// Example for a "Date" column (column 3)
fn apply_date_formatting(table: &mut Table) {
    table.modify(
        Columns::single(3)
            .not(Rows::first())
            .not(Rows::new(1..2)),
        Format::content(|s| preserve_bg(s.dimmed()))
    );
}

// Then call it after theme application:
apply_date_formatting(&mut table);
```

### If state badges need icons

If your state badges include icons (like ✓, ✗, ●), add them in the `apply_state_formatting()` function:

```rust
fn apply_state_formatting(table: &mut Table) {
    table.modify(
        Columns::single(2)
            .not(Rows::first())
            .not(Rows::new(1..2)),
        Format::content(|state_str| {
            let (icon, colored) = match state_str.to_lowercase().as_str() {
                "accepted" => ("✓ ", state_str.green()),
                "active" => ("● ", state_str.green().bold()),
                "final" => ("✓ ", state_str.green().bold()),
                "rejected" => ("✗ ", state_str.red()),
                _ => ("", state_str.white()),
            };
            preserve_bg(format!("{}{}", icon, colored.to_string()).as_str().into())
        })
    );
}
```

### If there are different table formats (verbose mode, etc.)

Apply the same pattern to all table building code:
1. Build with plain text
2. Apply theme
3. Apply custom formatting

---

## Expected Results

### Before Fix
```
Title row:   103 chars ✓
Header row:  103 chars ✓
Data row 1:   84 chars ✗
Data row 2:   82 chars ✗
Data row 3:   84 chars ✗
Footer row:  103 chars ✓
```

### After Fix
```
Title row:   103 chars ✓
Header row:  103 chars ✓
Data row 1:  103 chars ✓
Data row 2:  103 chars ✓
Data row 3:  103 chars ✓
Footer row:  103 chars ✓
```

---

## Troubleshooting

### If rows are still inconsistent:

1. **Check that ALL cell data is plain text** when pushing records
2. **Verify all formatting is applied via `Format::content()`** after table build
3. **Ensure `preserve_bg()` is used in all `Format::content()` callbacks**
4. **Check for any other places where colored strings are created** before table build

### If colors are missing:

1. **Verify imports** include `colored::*`
2. **Check that formatting functions are called** after `theme.apply_to_table()`
3. **Ensure `preserve_bg()` is used** in the formatting callbacks

### If bold is missing:

1. **Verify `apply_doc_number_formatting()` is called**
2. **Check that `.bold()` is inside the `Format::content()` callback**, not in the data

---

## Summary of Changes

| File | Changes |
|------|---------|
| `crates/design/src/commands/list.rs` | ✏️ Modified table building to use plain text |
| | ➕ Added `apply_doc_number_formatting()` function |
| | ➕ Added `apply_state_formatting()` function |
| | ✏️ Moved all formatting to AFTER table build |
| | ✏️ Applied formatters after theme application |

---

## Final Checklist

- [ ] All `builder.push_record()` calls use plain text (no ANSI codes)
- [ ] `apply_doc_number_formatting()` function created and called
- [ ] `apply_state_formatting()` function created and called
- [ ] All formatting applied AFTER `theme.apply_to_table()`
- [ ] `preserve_bg()` used in all formatting callbacks
- [ ] Tests pass: `cargo test`
- [ ] Visual output verified: `oxd list` shows correct formatting
- [ ] Width verified: all rows are 103 characters
- [ ] Background colors extend full width on all rows

---

## Questions?

If you encounter any issues:
1. Check that the `preserve_bg()` function exists and is being used
2. Verify the imports are correct
3. Ensure all table building uses plain text
4. Make sure formatting functions are called in the right order

The key principle: **Plain text in → formatted text out**
