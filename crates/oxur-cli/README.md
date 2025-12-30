# oxur-cli

Unified CLI infrastructure for Oxur.

## Overview

This crate provides two things:

1. **Library**: Common utilities for building Oxur CLI tools
   - File I/O helpers (stdin/stdout/file handling)
   - Colored terminal output (success, error, info, warnings)
   - Progress tracking for long-running operations

2. **Binary**: The unified `oxur` command-line tool
   - Compiling and running Oxur programs
   - Starting the REPL
   - Managing Oxur projects

## Library Usage

All Oxur CLI tools use this library for consistency.

### Add Dependency

```toml
[dependencies]
oxur-cli = { path = "../oxur-cli" }
```

### File I/O

```rust
use oxur_cli::common::io::{read_input, write_output};
use std::path::PathBuf;

// Read from file or stdin (-)
let content = read_input(&PathBuf::from("input.txt"))?;

// Process...

// Write to file or stdout (-)
write_output(&result, Some(&PathBuf::from("output.txt")))?;
```

### Colored Output

```rust
use oxur_cli::common::output::{success, error, info, warning};

info("Processing files...");
// ... work ...
success("All files processed!");

// Or with errors:
error("Failed to process file");
warning("Skipping invalid entry");
```

### Progress Tracking

```rust
use oxur_cli::common::progress::ProgressTracker;

let mut progress = ProgressTracker::new(verbose);

progress.step("Loading data");
// ... work ...
progress.done();

progress.step("Processing data");
// ... work ...
progress.done();

progress.success("All done!");
```

## Binary Usage

The `oxur` binary provides unified access to Oxur functionality.

### Compile

```bash
oxur compile input.ox -o output
```

Compile an Oxur file to a native binary.

### Run

```bash
oxur run input.ox -- arg1 arg2
```

Compile and run an Oxur file with arguments.

### REPL

```bash
oxur repl
```

Start the interactive REPL.

### New

```bash
oxur new my-project
```

Create a new Oxur project with standard structure.

### Build

```bash
oxur build
oxur build --release
```

Build the current project.

### Test

```bash
oxur test
```

Run tests in the current project.

## Development

### Build Library

```bash
cargo build --lib
```

### Build Binary

```bash
cargo build --bin oxur --features binary
```

### Run Tests

```bash
cargo test
```

## Architecture

```
oxur-cli
├── Library (common utilities)
│   ├── io        - File I/O helpers
│   ├── output    - Colored terminal output
│   └── progress  - Progress tracking
│
└── Binary (unified CLI)
    └── main      - Entry point for `oxur` command
```

## License

MIT OR Apache-2.0
