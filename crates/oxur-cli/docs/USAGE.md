# oxur-cli Library Usage Guide

## Quick Start

### 1. Add Dependency

```toml
[dependencies]
oxur-cli = { path = "../oxur-cli" }
```

### 2. Basic CLI Template

Here's a template for a new Oxur CLI tool:

```rust
//! My CLI tool

use anyhow::Result;
use clap::Parser;
use oxur_cli::common::output;

mod cli;
mod commands;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Err(e) = execute_command(cli.command) {
        output::error(&e.to_string());
        std::process::exit(1);
    }

    Ok(())
}

fn execute_command(command: Commands) -> Result<()> {
    match command {
        Commands::MyCmd { input, output } => {
            // Use I/O helpers
            let content = oxur_cli::common::io::read_input(&input)?;

            // ... process ...

            oxur_cli::common::io::write_output(&result, output.as_deref())?;
            output::success("Done!");
            Ok(())
        }
    }
}
```

## Module Guide

### `oxur_cli::common::io`

File I/O helpers for reading from stdin/file and writing to stdout/file.

**Key Functions:**

- `read_input(&Path) -> Result<String>` - Read from file or stdin (-)
- `write_output(&str, Option<&Path>) -> Result<()>` - Write to file or stdout
- `write_stderr(&str) -> Result<()>` - Write to stderr

**Example:**

```rust
use oxur_cli::common::io::{read_input, write_output};
use std::path::PathBuf;

// Read
let content = read_input(&PathBuf::from("input.txt"))?;

// Process
let result = process(&content)?;

// Write
write_output(&result, Some(&PathBuf::from("output.txt")))?;
```

### `oxur_cli::common::output`

Colored terminal output for consistent messaging.

**Key Functions:**

- `success(msg)` - Green checkmark + message
- `error(msg)` - Red "Error:" + message (stderr)
- `error_with_context(msg, context)` - Error with yellow context line
- `info(msg)` - Cyan arrow + message
- `warning(msg)` - Yellow "Warning:" + message
- `step(num, msg)` - Numbered step: "1. message..."
- `step_done()` - Indented green checkmark: "   ✓ Done"

**Example:**

```rust
use oxur_cli::common::output::{info, success, warning, error};

info("Starting process...");

if some_condition {
    warning("Non-critical issue detected");
}

if error_occurred {
    error("Operation failed");
    return Err(...);
}

success("All done!");
```

### `oxur_cli::common::progress`

Progress tracking for multi-step operations.

**Key Type:**

- `ProgressTracker` - Tracks numbered steps with verbose mode

**Example:**

```rust
use oxur_cli::common::progress::ProgressTracker;

pub fn my_command(verbose: bool) -> Result<()> {
    let mut progress = ProgressTracker::new(verbose);

    progress.step("Loading configuration");
    let config = load_config()?;
    progress.done();

    progress.step("Processing data");
    let result = process(config)?;
    progress.done();

    progress.step("Writing output");
    write_output(result)?;
    progress.done();

    progress.success("All operations completed!");
    Ok(())
}
```

## Best Practices

### 1. Always Support stdin/stdout

CLI tools should support piping with `-` for stdin/stdout:

```rust
// Good: Supports piping
let content = oxur_cli::common::io::read_input(&input)?;
oxur_cli::common::io::write_output(&result, output.as_deref())?;

// Bad: Hardcoded file reading
let content = fs::read_to_string(&input)?;
```

### 2. Use Progress Tracking for Multi-Step Operations

If your command has 3+ distinct steps, use `ProgressTracker`:

```rust
let mut progress = ProgressTracker::new(verbose);

progress.step("Step 1");
// work...
progress.done();

progress.step("Step 2");
// work...
progress.done();

progress.success("Complete!");
```

### 3. Consistent Output Messages

- **info()** - Use for progress messages: "Processing 10 files..."
- **warning()** - Use for non-critical issues: "Skipping invalid entry"
- **error()** - Use for fatal errors before returning Err
- **success()** - Use ONLY for final completion: "All done!"

### 4. Error Handling

```rust
// Simple error
if something_wrong {
    output::error("Operation failed");
    return Err(anyhow::anyhow!("details"));
}

// Error with helpful context
if parse_error {
    output::error_with_context(
        "Failed to parse configuration",
        "Check that the file is valid TOML"
    );
    return Err(...);
}
```

### 5. Verbose Mode

Use `ProgressTracker` to cleanly handle verbose/quiet modes:

```rust
let mut progress = ProgressTracker::new(verbose);

// This only shows in verbose mode:
progress.step("Internal step");
progress.done();

// This always shows:
progress.success("Done!");
```

## Patterns

### Pattern 1: File Processing Pipeline

```rust
pub fn process_file(input: PathBuf, output: Option<PathBuf>) -> Result<()> {
    use oxur_cli::common::{io, output};

    // Read
    let content = io::read_input(&input)?;

    // Process with feedback
    output::info("Processing file...");
    let result = do_processing(&content)?;

    // Write
    io::write_output(&result, output.as_deref())?;
    output::success("File processed successfully");

    Ok(())
}
```

### Pattern 2: Multi-Step Operation

```rust
pub fn complex_operation(verbose: bool) -> Result<()> {
    use oxur_cli::common::{progress::ProgressTracker, output};

    let mut progress = ProgressTracker::new(verbose);

    progress.step("Phase 1: Initialization");
    let state = initialize()?;
    progress.done();

    progress.step("Phase 2: Processing");
    let results = process(state)?;
    progress.done();

    progress.step("Phase 3: Finalization");
    finalize(results)?;
    progress.done();

    progress.success("Operation completed successfully!");
    Ok(())
}
```

### Pattern 3: Error Recovery

```rust
pub fn try_with_fallback() -> Result<()> {
    use oxur_cli::common::output;

    match try_primary_method() {
        Ok(result) => {
            output::success("Primary method succeeded");
            Ok(())
        }
        Err(e) => {
            output::warning(&format!("Primary method failed: {}", e));
            output::info("Trying fallback method...");

            match try_fallback_method() {
                Ok(_) => {
                    output::success("Fallback method succeeded");
                    Ok(())
                }
                Err(e) => {
                    output::error("Both methods failed");
                    Err(e)
                }
            }
        }
    }
}
```

## Migration Checklist

When migrating an existing CLI to use `oxur-cli`:

- [ ] Add `oxur-cli` dependency to `Cargo.toml`
- [ ] Replace manual stdin/stdout handling with `io::read_input()` / `write_output()`
- [ ] Replace error formatting with `output::error()`
- [ ] Replace success messages with `output::success()`
- [ ] Replace info messages with `output::info()`
- [ ] Replace warnings with `output::warning()`
- [ ] Consider adding `ProgressTracker` for multi-step operations
- [ ] Run tests to ensure no regressions
- [ ] Manual testing of all commands
