# oxur-term

Themed terminal tables and CLI output helpers for Oxur tools.

## Overview

`oxur-term` provides two things:

1. **`table`** — a themed table builder (`OxurTable`) with TOML-based
   theming, built on [`tabled`](https://crates.io/crates/tabled). Ships an
   embedded default theme with warm orange sunset colors matching the Oxur
   brand.
2. **`common`** — shared CLI utilities: file I/O helpers (stdin/stdout/file
   handling), colored terminal output (`success`/`error`/`info`/`warning`),
   and progress tracking for long-running operations.

This crate has a small, fixed dependency surface — `tabled`, `colored`,
`serde`, `toml` — and no dependency on any other Oxur crate, so tools that
just want themed output (like `odm`) can depend on it directly without
pulling in the Oxur compiler or REPL stack.

`oxur-cli` re-exports both modules (`oxur_cli::table`, `oxur_cli::common`)
for backwards compatibility with existing consumers.

## Add Dependency

```toml
[dependencies]
oxur-term = { version = "0.2.1", path = "../oxur-term" }
```

## Styled Tables

```rust
use oxur_term::table::{OxurTable, Tabled};
use colored::*;

#[derive(Tabled)]
struct Employee {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Age")]
    age: u32,
    #[tabled(rename = "Role")]
    role: ColoredString,
}

let employees = vec![
    Employee { name: "Alice".into(), age: 30, role: "Engineer".green() },
    Employee { name: "Bob".into(), age: 25, role: "Designer".cyan() },
];

let table = OxurTable::new(employees).render();
println!("{}", table);
```

See [`src/table/README.md`](src/table/README.md) for theming details
(default theme, hex colors, row styling).

## Colored Output

```rust
use oxur_term::common::output::{success, error, info, warning};

info("Processing files...");
// ... work ...
success("All files processed!");

// Or with errors:
error("Failed to process file");
warning("Skipping invalid entry");
```

## File I/O

```rust
use oxur_term::common::io::{read_input, write_output};
use std::path::PathBuf;

// Read from file or stdin (-)
let content = read_input(&PathBuf::from("input.txt"))?;

// Write to file or stdout (-)
write_output(&content, Some(&PathBuf::from("output.txt")))?;
```

## Progress Tracking

```rust
use oxur_term::common::progress::ProgressTracker;

let mut progress = ProgressTracker::new(verbose);

progress.step("Loading data");
// ... work ...
progress.done();

progress.step("Processing data");
// ... work ...
progress.done();

progress.success("All done!");
```

## Architecture

```
oxur-term
├── table    - Themed table builder (OxurTable, TableStyleConfig)
└── common
    ├── io        - File I/O helpers
    ├── output    - Colored terminal output
    └── progress  - Progress tracking
```

## History

This crate restores the standalone `oxur-table` design that ODD-0001 and
design-0015 always described. In late 2025 it was folded into `oxur-cli` as
a convenience; it was re-extracted so tools that only need themed terminal
output (like `odm`) don't have to compile the Oxur compiler/REPL stack.

## License

MIT OR Apache-2.0
