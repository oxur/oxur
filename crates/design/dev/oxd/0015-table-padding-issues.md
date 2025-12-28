# Fix for config.rs - Force Column Widths with Width::increase()

## Location
File: `crates/oxur-table/src/config.rs`
Lines: Around line 143-156

## The Problem

The current code uses:
```rust
table.with(Width::list([35, 65, 20]));
```

This sets **maximum** widths (for wrapping), but doesn't force **minimum** widths.
So columns only extend as far as their content, not to the specified widths.

## The Fix

Replace these lines:

```rust
        // Ensure consistent row widths by setting column widths
        // This works correctly when table is built with plain text first
        table.with(Width::list([35, 65, 20]));
```

With:

```rust
        // Ensure consistent row widths by forcing minimum column widths
        // Use Width::increase() on each column to force them to extend
        use tabled::settings::object::Columns;
        table.modify(Columns::single(0), Width::increase(15));
        table.modify(Columns::single(1), Width::increase(65));
        table.modify(Columns::single(2), Width::increase(25)); // Wider to force padding
```

## Why This Works

- `Width::increase()` forces a **minimum** width for each column
- When combined with `Justification`, the empty space is filled with the justification character
- The background color extends because justification fills with colored spaces
- Column 2 (State) is set to 25 chars min, ensuring it extends beyond "Accepted" (8 chars)

## Additional Import Needed

Make sure at the top of the `apply_to_table` method (around line 116), you have:

```rust
use tabled::settings::object::Columns;
```

If it's not there, add it near the other `use` statements at the top of the function.

Actually, looking at the code, `Columns` is already imported via the outer scope:
```rust
use tabled::settings::object::{Object, Rows, Segment};
```

So you just need to add `Columns` to this import:
```rust
use tabled::settings::object::{Columns, Object, Rows, Segment};
```

## Testing

After making this change:

```bash
cargo build
./target/debug/oxd list
```

You should see all rows extending to the same width with background colors all the way across.

## Adjusting Column Widths

If the columns are too wide or too narrow, adjust the numbers:
- Column 0 (Number): Currently 15 - adjust if numbers need more/less space
- Column 1 (Title): Currently 65 - adjust based on typical title lengths
- Column 2 (State): Currently 25 - adjust to control total table width

The total visual width will be approximately:
- Padding: 0 left + 0 right (per cell) = 0
- Column 0: 15 chars
- Column 1: 65 chars  
- Column 2: 25 chars
- **Total: ~105 chars** (close to your desired 103)

Adjust column 2 to 23 if you want exactly 103 chars.
