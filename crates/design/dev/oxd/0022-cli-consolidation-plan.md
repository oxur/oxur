# oxur-cli Common Utilities Consolidation Plan

**Date:** 2025-12-29
**Project:** Oxur CLI Library & Binary
**Status:** Ready for Implementation

---

## Executive Summary

This plan transforms `oxur-cli` from a placeholder binary crate into a dual-purpose crate:

1. **Library**: Provides common utilities for all Oxur CLI tools
2. **Binary**: Will eventually house the unified `oxur` command-line tool

This approach is superior to creating a separate `oxur-cli-common` crate because:

- ✅ Reduces crate proliferation
- ✅ Natural home for CLI infrastructure
- ✅ The `oxur` binary can use its own library code
- ✅ Follows idiomatic Rust library+binary pattern
- ✅ Clear conceptual model: "oxur-cli is for everything CLI-related"

---

## Current State Analysis

### Existing Structure

```
crates/oxur-cli/
├── Cargo.toml
├── README.md
└── src/
    └── main.rs
```

**Current `main.rs`**: Likely a placeholder or minimal implementation.

### Target Structure

```
crates/oxur-cli/
├── Cargo.toml          # Configures both lib and bin
├── README.md           # Updated documentation
├── src/
│   ├── lib.rs          # NEW: Public library API
│   ├── main.rs         # Existing: Future `oxur` binary (updated)
│   └── common/         # NEW: Common utilities module
│       ├── mod.rs      # Module declarations and re-exports
│       ├── io.rs       # File I/O helpers
│       ├── output.rs   # Colored terminal output
│       └── progress.rs # Progress tracking
```

---

## What Gets Extracted

### From `aster` (oxur-ast CLI)

**File I/O** (~30 LOC):

- `src/commands/to_ast.rs`: stdin/stdout/file reading and writing
- `src/commands/to_rust.rs`: stdin/stdout/file reading and writing

**Colored Output** (~15 LOC):

- `src/main.rs`: Error formatting with colored output
- `src/commands/verify.rs`: Success messages with checkmarks

**Progress Tracking** (~10 LOC):

- `src/commands/verify.rs`: Verbose mode with numbered steps

### From `oxd` (design CLI)

**Colored Output** (~20 LOC):

- `src/main.rs`: Info messages with cyan arrows
- Various commands: Success/error messages

**Potential Progress Tracking** (future):

- `src/commands/scan.rs`: Could benefit from structured progress
- `src/commands/validate.rs`: Multi-step operations

### Total Extraction

- **Immediate:** ~75 LOC with high confidence
- **Future Value:** All new Oxur CLIs bootstrap in minutes
- **Consistency:** Unified UX across all Oxur tools

---

## Implementation Plan

### Phase 1: Create Library Infrastructure (Week 1, ~6 hours)

#### Step 1.1: Update Cargo.toml

**File:** `crates/oxur-cli/Cargo.toml`

```toml
[package]
name = "oxur-cli"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "CLI infrastructure and unified command-line tool for Oxur"

# Binary configuration - the future unified `oxur` tool
[[bin]]
name = "oxur"
path = "src/main.rs"

# Library configuration - common utilities for all Oxur CLIs
[lib]
name = "oxur_cli"
path = "src/lib.rs"

[dependencies]
# For CLI utilities
anyhow.workspace = true
colored.workspace = true

# For future binary (when implemented)
clap = { workspace = true, optional = true }

[features]
default = []
# Feature for the binary (not needed by library users)
binary = ["clap"]

[dev-dependencies]
tempfile.workspace = true
```

**Rationale:**

- Library and binary can coexist in same crate
- Library has minimal dependencies (just `anyhow` and `colored`)
- `clap` is optional, only needed when building the binary
- Other CLIs only depend on the library portion

#### Step 1.2: Create Module Structure

```bash
cd crates/oxur-cli/src
mkdir common
touch lib.rs
touch common/mod.rs
touch common/io.rs
touch common/output.rs
touch common/progress.rs
```

#### Step 1.3: Implement `lib.rs`

**File:** `crates/oxur-cli/src/lib.rs`

```rust
//! Oxur CLI library and binary
//!
//! This crate provides two things:
//!
//! 1. **Library**: Common utilities for building Oxur CLI tools
//!    - File I/O helpers (stdin/stdout/file handling)
//!    - Colored terminal output (success, error, info, warnings)
//!    - Progress tracking for long-running operations
//!
//! 2. **Binary**: The unified `oxur` command-line tool (future)
//!
//! # Library Usage
//!
//! Add to your CLI's `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! oxur-cli = { path = "../oxur-cli" }
//! ```
//!
//! ## Basic I/O
//!
//! ```no_run
//! use oxur_cli::common::io::{read_input, write_output};
//! use std::path::PathBuf;
//!
//! # fn main() -> anyhow::Result<()> {
//! // Read from file or stdin
//! let content = read_input(&PathBuf::from("input.txt"))?;
//!
//! // Write to file or stdout
//! write_output(&content, Some(&PathBuf::from("output.txt")))?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Colored Output
//!
//! ```no_run
//! use oxur_cli::common::output::{success, error, info};
//!
//! success("Operation completed!");
//! error("Something went wrong");
//! info("Processing files...");
//! ```
//!
//! ## Progress Tracking
//!
//! ```no_run
//! use oxur_cli::common::progress::ProgressTracker;
//!
//! # fn main() -> anyhow::Result<()> {
//! let mut progress = ProgressTracker::new(true);
//!
//! progress.step("Loading data");
//! // ... do work ...
//! progress.done();
//!
//! progress.step("Processing data");
//! // ... do work ...
//! progress.done();
//!
//! progress.success("All done!");
//! # Ok(())
//! # }
//! ```

pub mod common;

// Re-export commonly used items for convenience
pub use common::progress::ProgressTracker;
```

#### Step 1.4: Implement `common/mod.rs`

**File:** `crates/oxur-cli/src/common/mod.rs`

```rust
//! Common utilities for Oxur CLI tools
//!
//! This module provides shared functionality for building consistent CLI
//! tools in the Oxur project.

pub mod io;
pub mod output;
pub mod progress;

// Re-exports for convenience
pub use progress::ProgressTracker;
```

#### Step 1.5: Implement `common/io.rs`

**File:** `crates/oxur-cli/src/common/io.rs`

```rust
//! File I/O utilities for CLI tools
//!
//! Provides helpers for reading from stdin/file and writing to stdout/file,
//! which is a common pattern in CLI tools that support piping.

use anyhow::Result;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

/// Read input from stdin (if path is "-") or from a file
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use oxur_cli::common::io::read_input;
///
/// // Read from file
/// let content = read_input(&PathBuf::from("input.txt"))?;
///
/// // Read from stdin (if user passes "-")
/// let content = read_input(&PathBuf::from("-"))?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn read_input(path: &Path) -> Result<String> {
    if path.to_str() == Some("-") {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Ok(buffer)
    } else {
        Ok(fs::read_to_string(path)?)
    }
}

/// Write output to stdout (if path is None or "-") or to a file
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use oxur_cli::common::io::write_output;
///
/// // Write to file
/// write_output("content", Some(&PathBuf::from("output.txt")))?;
///
/// // Write to stdout
/// write_output("content", None)?;
///
/// // Write to stdout (if user passes "-")
/// write_output("content", Some(&PathBuf::from("-")))?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn write_output(content: &str, path: Option<&Path>) -> Result<()> {
    match path {
        Some(p) if p.to_str() == Some("-") => {
            println!("{}", content);
            Ok(())
        }
        Some(p) => {
            fs::write(p, content)?;
            Ok(())
        }
        None => {
            println!("{}", content);
            Ok(())
        }
    }
}

/// Write output to stderr
///
/// Useful for progress messages and diagnostics that shouldn't be mixed with stdout.
///
/// # Examples
///
/// ```no_run
/// use oxur_cli::common::io::write_stderr;
///
/// write_stderr("Processing file...")?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn write_stderr(message: &str) -> Result<()> {
    writeln!(io::stderr(), "{}", message)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_read_input_from_file() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        let content = read_input(&file_path).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_read_input_nonexistent_file() {
        let path = PathBuf::from("nonexistent.txt");
        let result = read_input(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_output_to_file() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("output.txt");

        write_output("test content", Some(&file_path)).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_write_output_to_stdout() {
        // This test just verifies it doesn't panic
        let result = write_output("test", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_output_dash_means_stdout() {
        // Verify that "-" is treated as stdout
        let result = write_output("test", Some(&PathBuf::from("-")));
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_stderr() {
        let result = write_stderr("test message");
        assert!(result.is_ok());
    }
}
```

#### Step 1.6: Implement `common/output.rs`

**File:** `crates/oxur-cli/src/common/output.rs`

```rust
//! Colored terminal output utilities
//!
//! Provides consistent, colored output helpers for success, error, info,
//! and warning messages across all Oxur CLI tools.

use colored::*;

/// Print a success message with a green checkmark
///
/// # Examples
///
/// ```no_run
/// use oxur_cli::common::output::success;
///
/// success("Operation completed successfully");
/// // Output: ✓ Operation completed successfully (in green)
/// ```
pub fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

/// Print an error message with a red "Error:" prefix
///
/// # Examples
///
/// ```no_run
/// use oxur_cli::common::output::error;
///
/// error("Failed to open file");
/// // Output: Error: Failed to open file (in red, to stderr)
/// ```
pub fn error(msg: &str) {
    eprintln!("{} {}", "Error:".red().bold(), msg);
}

/// Print an error message with context
///
/// # Examples
///
/// ```no_run
/// use oxur_cli::common::output::error_with_context;
///
/// error_with_context("Failed to parse file", "Check the file format");
/// // Output:
/// // Error: Failed to parse file (in red)
/// // → Check the file format (in yellow)
/// ```
pub fn error_with_context(msg: &str, context: &str) {
    eprintln!("{} {}", "Error:".red().bold(), msg);
    eprintln!("{} {}", "→".yellow(), context);
}

/// Print an info message with a cyan arrow
///
/// # Examples
///
/// ```no_run
/// use oxur_cli::common::output::info;
///
/// info("Processing 5 files...");
/// // Output: → Processing 5 files... (in cyan)
/// ```
pub fn info(msg: &str) {
    println!("{} {}", "→".cyan(), msg);
}

/// Print a warning message with a yellow prefix
///
/// # Examples
///
/// ```no_run
/// use oxur_cli::common::output::warning;
///
/// warning("File already exists, skipping");
/// // Output: Warning: File already exists, skipping (in yellow)
/// ```
pub fn warning(msg: &str) {
    println!("{} {}", "Warning:".yellow().bold(), msg);
}

/// Print a numbered step in a process
///
/// # Examples
///
/// ```no_run
/// use oxur_cli::common::output::step;
///
/// step(1, "Parsing input");
/// // Output: 1. Parsing input...
/// ```
pub fn step(num: usize, msg: &str) {
    println!("{}. {}...", num, msg);
}

/// Print a step completion marker
///
/// # Examples
///
/// ```no_run
/// use oxur_cli::common::output::{step, step_done};
///
/// step(1, "Parsing input");
/// // ... do work ...
/// step_done();
/// // Output:    ✓ Done (in green, indented)
/// ```
pub fn step_done() {
    println!("   {} Done", "✓".green());
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests just verify the functions don't panic
    // Testing actual colored output would require capturing stdout/stderr

    #[test]
    fn test_success() {
        success("test message");
    }

    #[test]
    fn test_error() {
        error("test error");
    }

    #[test]
    fn test_error_with_context() {
        error_with_context("test error", "test context");
    }

    #[test]
    fn test_info() {
        info("test info");
    }

    #[test]
    fn test_warning() {
        warning("test warning");
    }

    #[test]
    fn test_step() {
        step(1, "test step");
    }

    #[test]
    fn test_step_done() {
        step_done();
    }
}
```

#### Step 1.7: Implement `common/progress.rs`

**File:** `crates/oxur-cli/src/common/progress.rs`

```rust
//! Progress tracking for long-running operations
//!
//! Provides a simple progress tracker that can show numbered steps
//! with optional verbose output.

use colored::*;

/// A progress tracker for multi-step operations
///
/// Tracks the current step and provides methods to display progress
/// with optional verbose mode.
///
/// # Examples
///
/// ```no_run
/// use oxur_cli::common::progress::ProgressTracker;
///
/// # fn main() -> anyhow::Result<()> {
/// let mut progress = ProgressTracker::new(true); // verbose mode
///
/// progress.step("Parsing input");
/// // ... do work ...
/// progress.done();
///
/// progress.step("Generating output");
/// // ... do work ...
/// progress.done();
///
/// progress.success("All operations completed!");
/// # Ok(())
/// # }
/// ```
pub struct ProgressTracker {
    current_step: usize,
    verbose: bool,
}

impl ProgressTracker {
    /// Create a new progress tracker
    ///
    /// # Arguments
    ///
    /// * `verbose` - If true, shows detailed step-by-step progress
    pub fn new(verbose: bool) -> Self {
        Self { current_step: 0, verbose }
    }

    /// Start a new step in the process
    ///
    /// In verbose mode, displays: "N. message..."
    /// In non-verbose mode, does nothing
    pub fn step(&mut self, msg: &str) {
        if self.verbose {
            self.current_step += 1;
            println!("{}. {}...", self.current_step, msg);
        }
    }

    /// Mark the current step as complete
    ///
    /// In verbose mode, displays: "   ✓ Done" (in green)
    /// In non-verbose mode, does nothing
    pub fn done(&self) {
        if self.verbose {
            println!("   {} Done", "✓".green());
        }
    }

    /// Display a final success message
    ///
    /// Always displays, regardless of verbose mode.
    /// In verbose mode, adds a blank line before the message.
    pub fn success(&self, msg: &str) {
        if self.verbose {
            println!();
        }
        println!("{} {}", "✓".green().bold(), msg);
    }

    /// Display an error message
    ///
    /// Always displays, regardless of verbose mode.
    pub fn error(&self, msg: &str) {
        eprintln!("{} {}", "Error:".red().bold(), msg);
    }

    /// Display an info message
    ///
    /// Only displays in verbose mode.
    pub fn info(&self, msg: &str) {
        if self.verbose {
            println!("{} {}", "→".cyan(), msg);
        }
    }

    /// Check if verbose mode is enabled
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_verbose() {
        let tracker = ProgressTracker::new(true);
        assert!(tracker.is_verbose());
        assert_eq!(tracker.current_step, 0);
    }

    #[test]
    fn test_new_non_verbose() {
        let tracker = ProgressTracker::new(false);
        assert!(!tracker.is_verbose());
    }

    #[test]
    fn test_step_increments_counter() {
        let mut tracker = ProgressTracker::new(true);
        assert_eq!(tracker.current_step, 0);

        tracker.step("First step");
        assert_eq!(tracker.current_step, 1);

        tracker.step("Second step");
        assert_eq!(tracker.current_step, 2);
    }

    #[test]
    fn test_step_non_verbose_no_increment() {
        let mut tracker = ProgressTracker::new(false);
        tracker.step("Step");
        assert_eq!(tracker.current_step, 0);
    }

    #[test]
    fn test_done_verbose() {
        let tracker = ProgressTracker::new(true);
        tracker.done();
    }

    #[test]
    fn test_done_non_verbose() {
        let tracker = ProgressTracker::new(false);
        tracker.done();
    }

    #[test]
    fn test_success() {
        let tracker = ProgressTracker::new(true);
        tracker.success("All done!");
    }

    #[test]
    fn test_error() {
        let tracker = ProgressTracker::new(true);
        tracker.error("Something failed");
    }

    #[test]
    fn test_info_verbose() {
        let tracker = ProgressTracker::new(true);
        tracker.info("Additional info");
    }

    #[test]
    fn test_info_non_verbose() {
        let tracker = ProgressTracker::new(false);
        tracker.info("This shouldn't display");
    }
}
```

#### Step 1.8: Update `main.rs`

**File:** `crates/oxur-cli/src/main.rs`

```rust
//! Unified Oxur command-line tool (future implementation)
//!
//! This will eventually provide a single `oxur` command that can:
//! - Manage design documents (oxd functionality)
//! - Manipulate ASTs (aster functionality)
//! - Compile Oxur Lisp code (oxc functionality)
//! - Run REPL sessions (oxr functionality)
//! - And more...
//!
//! For now, this is a placeholder that demonstrates using the library.

use oxur_cli::common::output;

fn main() {
    output::info("oxur CLI tool - coming soon!");
    output::warning("This is a placeholder. Use the individual tools for now:");
    println!("  • aster - AST manipulation");
    println!("  • oxd   - Design documentation");
    println!();
    output::success("More functionality coming soon!");
}
```

#### Step 1.9: Update README

**File:** `crates/oxur-cli/README.md`

```markdown
# oxur-cli

Unified CLI infrastructure for Oxur.

## Overview

This crate provides two things:

1. **Library**: Common utilities for building Oxur CLI tools
   - File I/O helpers (stdin/stdout/file handling)
   - Colored terminal output (success, error, info, warnings)
   - Progress tracking for long-running operations

2. **Binary**: The unified `oxur` command-line tool (future)
   - Will consolidate functionality from `aster`, `oxd`, and other tools
   - Provides a single entry point for all Oxur operations

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

## Binary Usage (Future)

The `oxur` binary will eventually provide:

```bash
# Design document management
oxur docs list
oxur docs new "My Design"

# AST manipulation
oxur ast verify file.rs
oxur ast to-sexp file.rs

# Compilation
oxur compile program.oxur

# REPL
oxur repl
```

## Development

### Build Library

```bash
cargo build --lib
```

### Build Binary

```bash
cargo build --bin oxur
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
└── Binary (unified CLI - future)
    └── main      - Entry point for `oxur` command
```

## License

MIT OR Apache-2.0

```

#### Step 1.10: Run Tests

```bash
cd crates/oxur-cli
cargo test
cargo clippy -- -D warnings
```

---

### Phase 2: Migrate `aster` CLI (Week 1, ~4 hours)

#### Step 2.1: Update Cargo.toml

**File:** `crates/oxur-ast/Cargo.toml`

Add dependency:

```toml
[dependencies]
# ... existing dependencies ...
oxur-cli = { path = "../oxur-cli" }
```

Remove `colored` from dependencies (we'll use oxur-cli's):

```toml
# Remove or comment out:
# colored.workspace = true
```

#### Step 2.2: Update `main.rs`

**File:** `crates/oxur-ast/src/main.rs`

```rust
//! AST manipulation and conversion CLI tool

use anyhow::Result;
use clap::Parser;
use oxur_ast::commands;

mod cli;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Err(e) = execute_command(cli.command) {
        oxur_cli::common::output::error(&e.to_string());
        std::process::exit(1);
    }

    Ok(())
}

fn execute_command(command: Commands) -> Result<()> {
    match command {
        Commands::ToAst { input, output, compact } => commands::to_ast(input, output, compact),
        Commands::ToRust { input, output } => commands::to_rust(input, output),
        Commands::Verify { input, verbose } => commands::verify(input, verbose),
    }
}
```

**Changes:**

- Remove `use colored::*;`
- Replace error formatting with `oxur_cli::common::output::error()`

#### Step 2.3: Update `to_ast.rs`

**File:** `crates/oxur-ast/src/commands/to_ast.rs`

```rust
use crate::integration::parse_rust_file;
use crate::sexp::print_sexp;
use crate::Generator;
use anyhow::Result;
use std::path::PathBuf;

pub fn execute(input: PathBuf, output: Option<PathBuf>, compact: bool) -> Result<()> {
    // Read input using common utility
    let source = oxur_cli::common::io::read_input(&input)?;

    // Parse Rust
    let crate_node = parse_rust_file(&source)?;

    // Generate S-expression
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node)?;

    // Format
    // Note: compact mode is not yet implemented, using pretty print for now
    let output_text = if compact {
        // TODO: Implement compact printing in Phase 4
        print_sexp(&sexp)
    } else {
        print_sexp(&sexp)
    };

    // Write output using common utility
    oxur_cli::common::io::write_output(&output_text, output.as_deref())?;

    Ok(())
}
```

**Changes:**

- Remove `use std::fs;` and `use std::io::{self, Read};`
- Replace manual I/O handling with `oxur_cli::common::io::read_input()` and `write_output()`

#### Step 2.4: Update `to_rust.rs`

**File:** `crates/oxur-ast/src/commands/to_rust.rs`

Same pattern as `to_ast.rs` - replace I/O handling with common utilities.

```rust
use crate::sexp::Parser;
use crate::AstBuilder;
use anyhow::Result;
use quote::quote;
use std::path::PathBuf;

pub fn execute(input: PathBuf, output: Option<PathBuf>) -> Result<()> {
    // Read input using common utility
    let source = oxur_cli::common::io::read_input(&input)?;

    // Parse S-expression
    let sexp = Parser::parse_str(&source)?;

    // Build AST
    let mut builder = AstBuilder::new();
    let crate_node = builder.build_crate(&sexp)?;

    // Generate Rust code
    let tokens = quote! { #crate_node };
    let rust_code = prettyplease::unparse(&syn::parse2(tokens)?);

    // Write output using common utility
    oxur_cli::common::io::write_output(&rust_code, output.as_deref())?;

    Ok(())
}
```

#### Step 2.5: Update `verify.rs`

**File:** `crates/oxur-ast/src/commands/verify.rs`

```rust
use crate::integration::parse_rust_file;
use crate::sexp::{print_sexp, Parser};
use crate::{AstBuilder, Generator};
use anyhow::Result;
use colored::*; // Keep for the file name display
use std::fs;
use std::path::PathBuf;

pub fn execute(input: PathBuf, verbose: bool) -> Result<()> {
    use oxur_cli::common::progress::ProgressTracker;

    let source = fs::read_to_string(&input)?;
    let mut progress = ProgressTracker::new(verbose);

    println!("{} {}", "Verifying round-trip for:".bold(), input.display());
    if verbose {
        println!();
    }

    progress.step("Parsing Rust source");
    let crate1 = parse_rust_file(&source)?;
    progress.done();

    progress.step("Generating S-expression");
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate1)?;
    progress.done();

    progress.step("Parsing S-expression");
    let sexp_text = print_sexp(&sexp);
    let sexp2 = Parser::parse_str(&sexp_text)?;
    progress.done();

    progress.step("Building AST from S-expression");
    let mut builder = AstBuilder::new();
    let crate2 = builder.build_crate(&sexp2)?;
    progress.done();

    progress.step("Verifying equivalence");
    if crate1.items.len() != crate2.items.len() {
        anyhow::bail!("Item count mismatch: {} vs {}", crate1.items.len(), crate2.items.len());
    }
    progress.done();

    progress.success("Round-trip verification successful!");

    Ok(())
}
```

**Changes:**

- Keep `colored` for the file name display (special formatting)
- Replace manual progress tracking with `ProgressTracker`
- Much cleaner, more maintainable code

#### Step 2.6: Run Tests

```bash
cd crates/oxur-ast
cargo test
cargo clippy
```

#### Step 2.7: Manual Testing

```bash
# Build
cargo build --release --bin aster

# Test commands
echo "fn main() {}" | target/release/aster to-ast -
target/release/aster verify tests/fixtures/hello_world.rs
target/release/aster verify tests/fixtures/hello_world.rs --verbose
```

---

### Phase 3: Migrate `oxd` CLI (Week 2, ~4 hours)

#### Step 3.1: Update Cargo.toml

**File:** `crates/design/Cargo.toml`

Add dependency:

```toml
[dependencies]
# ... existing dependencies ...
oxur-cli = { path = "../oxur-cli" }
```

**Note:** Keep `colored` dependency since `oxd` uses it extensively for custom formatting beyond the common utilities.

#### Step 3.2: Update Info Messages in `main.rs`

**File:** `crates/design/src/main.rs`

In the `scan_on_startup` function:

```rust
// BEFORE:
eprintln!(
    "{} Detected {} change(s) ({} new, {} modified, {} deleted)",
    "→".cyan(),
    total,
    result.new_files.len(),
    result.changed.len(),
    result.deleted.len()
);

// AFTER:
oxur_cli::common::output::info(&format!(
    "Detected {} change(s) ({} new, {} modified, {} deleted)",
    total,
    result.new_files.len(),
    result.changed.len(),
    result.deleted.len()
));
```

In error handling (optional, since oxd has custom error module):

```rust
// Could optionally replace simple errors:
// design::errors::print_error("Startup scan failed", &e);
// With:
// oxur_cli::common::output::error("Startup scan failed");
```

**Decision:** Keep `oxd`'s custom error module for complex cases, but use common utilities for simple info/warning messages.

#### Step 3.3: Consider Progress in Long-Running Commands

**File:** `crates/design/src/commands/scan.rs` (optional enhancement)

```rust
pub fn scan_documents(state_mgr: &mut StateManager, fix: bool, verbose: bool) -> Result<()> {
    use oxur_cli::common::progress::ProgressTracker;

    let mut progress = ProgressTracker::new(verbose);

    progress.step("Scanning filesystem");
    let result = state_mgr.scan_filesystem()?;
    progress.done();

    if fix && result.has_changes() {
        progress.step("Applying fixes");
        // ... fix logic ...
        progress.done();
    }

    progress.success("Scan completed!");

    // ... rest of function ...
}
```

**Decision:** This is optional and can be done incrementally. Start with just updating info messages, add progress tracking later.

#### Step 3.4: Run Tests

```bash
cd crates/design
cargo test
cargo clippy
```

#### Step 3.5: Manual Testing

```bash
cargo build --release --bin oxd
target/release/oxd list
target/release/oxd scan --verbose
```

---

### Phase 4: Documentation & Polish (Week 2, ~3 hours)

#### Step 4.1: Create Usage Guide

**File:** `crates/oxur-cli/docs/USAGE.md`

```markdown
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

```

#### Step 4.2: Update Root README

**File:** Root `README.md`

Add CLI tools section:

```markdown
## CLI Tools

Oxur includes several command-line tools:

- **aster** - AST manipulation (Rust ↔ S-expression conversion)
- **oxd** - Design documentation manager
- **oxur** - Unified CLI tool (future)

### Building CLI Tools

```bash
# Build all CLIs
cargo build --release --bins

# Build specific CLI
cargo build --release --bin aster
cargo build --release --bin oxd
```

### CLI Infrastructure

All CLI tools use the `oxur-cli` library for common utilities:

- File I/O helpers (stdin/stdout/file handling)
- Colored terminal output
- Progress tracking

See `crates/oxur-cli/docs/USAGE.md` for development guide.

```

#### Step 4.3: Add CHANGELOG

**File:** `crates/oxur-cli/CHANGELOG.md`

```markdown
# Changelog

All notable changes to oxur-cli will be documented in this file.

## [0.1.0] - 2025-12-29

### Added

**Library Features:**
- Common I/O utilities module (`common::io`)
  - `read_input()` - Read from stdin or file
  - `write_output()` - Write to stdout or file
  - `write_stderr()` - Write to stderr
- Colored output module (`common::output`)
  - `success()`, `error()`, `warning()`, `info()`
  - `error_with_context()` - Error with helpful context
  - `step()` and `step_done()` - Numbered step indicators
- Progress tracking module (`common::progress`)
  - `ProgressTracker` - Multi-step operation tracking with verbose mode
- Comprehensive unit tests for all modules
- Usage documentation and examples

**Binary Features:**
- Placeholder implementation for future unified `oxur` command

### Migration Notes

- `aster` CLI now uses `oxur-cli` for I/O, output, and progress tracking
- `oxd` CLI partially migrated (using output utilities)

## [Unreleased]

### Planned

- Full `oxd` integration with progress tracking
- Configuration file loading utilities
- Interactive prompt helpers
- Completion of unified `oxur` binary
```

---

## Testing Strategy

### Unit Tests

Each module has comprehensive unit tests:

```bash
# Test the library
cd crates/oxur-cli
cargo test

# Should see:
# - common::io tests (6 tests)
# - common::output tests (7 tests)
# - common::progress tests (10 tests)
```

### Integration Tests

Test in actual CLI usage:

```bash
# Test aster
cd crates/oxur-ast
cargo test
cargo build --release --bin aster

# Manual tests
echo "fn main() {}" | target/release/aster to-ast -
target/release/aster verify tests/fixtures/hello_world.rs --verbose

# Test oxd
cd crates/design
cargo test
cargo build --release --bin oxd

# Manual tests
target/release/oxd list
target/release/oxd scan --verbose
```

### Regression Prevention

Run full test suite after each phase:

```bash
# From repo root
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## Success Criteria

### Phase 1 Complete ✓

- [ ] `oxur-cli` crate has both library and binary
- [ ] All three common modules implemented with tests
- [ ] Library tests pass: `cd crates/oxur-cli && cargo test`
- [ ] Clippy clean: `cargo clippy -- -D warnings`
- [ ] Documentation complete (README, lib.rs docs, module docs)
- [ ] Placeholder binary runs without errors

### Phase 2 Complete ✓

- [ ] `aster` uses `oxur-cli` library
- [ ] All `aster` tests pass
- [ ] Manual testing confirms CLI works correctly
- [ ] No regressions in functionality
- [ ] Code is cleaner and more maintainable

### Phase 3 Complete ✓

- [ ] `oxd` uses `oxur-cli` for output utilities
- [ ] All `oxd` tests pass
- [ ] Manual testing confirms CLI works correctly
- [ ] No regressions in functionality

### Phase 4 Complete ✓

- [ ] Usage guide written (`docs/USAGE.md`)
- [ ] Root README updated with CLI tools section
- [ ] CHANGELOG created and up to date
- [ ] All documentation reviewed and polished

---

## Metrics & Impact

### Before Extraction

| Metric | aster | oxd | Total |
|--------|-------|-----|-------|
| Duplicated code (LOC) | ~60 | ~55 | ~115 |
| Custom I/O handling | Yes | Partial | 2 CLIs |
| Custom output formatting | Yes | Yes | 2 CLIs |
| Progress tracking | Custom | None | 1 CLI |

### After Extraction

| Metric | Value | Improvement |
|--------|-------|-------------|
| Shared library code | ~300 LOC | Centralized |
| LOC saved in aster | ~50 | Cleaner code |
| LOC saved in oxd | ~20 | More consistent |
| Future CLI bootstrap time | <30 min | vs ~2 hours |
| UX consistency | 100% | All use same helpers |
| Test coverage | High | Shared code well-tested |

### Real Value

The true value isn't just LOC reduction:

1. **Consistency**: All Oxur CLIs have identical UX (colors, messages, progress)
2. **Quality**: Shared code gets more testing and polish
3. **Speed**: New CLIs bootstrap in minutes, not hours
4. **Maintenance**: Bug fixes and improvements benefit all CLIs instantly
5. **Foundation**: Sets up `oxur-cli` as the home for all CLI infrastructure

---

## Risks & Mitigation

### Risk 1: Breaking Existing CLI Functionality

**Likelihood:** Low-Medium
**Impact:** High
**Mitigation:**

- Comprehensive testing before and after migration
- Incremental migration (one CLI at a time)
- Manual testing of all commands
- Git history for easy rollback

### Risk 2: Incomplete Abstraction

**Likelihood:** Low
**Impact:** Low
**Mitigation:**

- Start with high-confidence patterns only
- Keep abstractions simple and focused
- Don't force convergence where differences are justified
- Can always extend later

### Risk 3: Binary/Library Confusion

**Likelihood:** Low
**Impact:** Low
**Mitigation:**

- Clear documentation separating library from binary
- Binary is clearly marked as "future"
- Library has comprehensive examples
- README explains dual purpose

---

## Future Enhancements

### Post-Initial Release

Once the common library is established:

1. **Enhanced Error Formatting** (Priority: Medium)
   - Reconcile `oxd`'s rich formatting with `aster`'s simplicity
   - Add optional advanced error helpers
   - **When:** After both CLIs are using basic utilities

2. **Configuration Loading** (Priority: Low)
   - TOML/YAML config file helpers
   - Environment variable integration
   - **When:** Third CLI needs configuration

3. **Interactive Prompts** (Priority: Low)
   - Y/N confirmations
   - Select from list
   - Text input with validation
   - **When:** Another CLI needs interactive features

4. **Unified `oxur` Binary** (Priority: High)
   - Implement the actual `oxur` command
   - Consolidate `aster` and `oxd` functionality
   - **When:** After library is stable and proven

### Decision Criteria

**Rule of Three:** Wait until 3 CLIs need a feature before adding it to the library.

**Exceptions:**

- Foundation features (I/O, output, progress) - add immediately
- Features for the `oxur` binary - add as needed
- Requests from multiple CLI maintainers - consider carefully

---

## Appendix A: Dependency Graph

### Before

```
oxur-ast (aster)
├── clap
├── anyhow
├── colored
└── syn

design (oxd)
├── clap
├── anyhow
├── colored
├── oxur-table
└── ...

oxur-cli
└── (empty placeholder)
```

### After

```
oxur-cli (library + binary)
├── anyhow
├── colored
└── clap (optional, for binary only)

oxur-ast (aster)
├── oxur-cli ← Uses library
├── clap
├── anyhow
└── syn

design (oxd)
├── oxur-cli ← Uses library
├── oxur-table
├── clap
├── anyhow
├── colored (kept for custom formatting)
└── ...
```

---

## Appendix B: File Sizes

### New Files Created

| File | Approximate LOC | Purpose |
|------|----------------|---------|
| `crates/oxur-cli/src/lib.rs` | 70 | Library documentation and exports |
| `crates/oxur-cli/src/common/mod.rs` | 10 | Module organization |
| `crates/oxur-cli/src/common/io.rs` | 100 | I/O utilities + tests |
| `crates/oxur-cli/src/common/output.rs` | 90 | Output helpers + tests |
| `crates/oxur-cli/src/common/progress.rs` | 130 | Progress tracking + tests |
| `crates/oxur-cli/docs/USAGE.md` | 500 | Comprehensive usage guide |
| **Total** | **~900** | **New code + documentation** |

### Code Removed/Simplified

| File | LOC Removed | LOC Simplified |
|------|-------------|----------------|
| `crates/oxur-ast/src/main.rs` | 5 | 3 |
| `crates/oxur-ast/src/commands/to_ast.rs` | 20 | 5 |
| `crates/oxur-ast/src/commands/to_rust.rs` | 20 | 5 |
| `crates/oxur-ast/src/commands/verify.rs` | 30 | 10 |
| `crates/design/src/main.rs` | 5 | 2 |
| **Total** | **~80** | **~25** |

---

## Conclusion

This implementation plan transforms `oxur-cli` from an empty placeholder into the foundational library for all Oxur CLI tools, while also setting up the structure for the future unified `oxur` command.

**Key Benefits:**

1. ✅ **Reduced Crate Proliferation** - Use existing crate instead of creating new one
2. ✅ **Natural Architecture** - Library + binary in one package makes sense
3. ✅ **Immediate Value** - ~80 LOC removed from existing CLIs
4. ✅ **Future Value** - Foundation for all future CLI development
5. ✅ **Consistency** - Unified UX across all Oxur tools
6. ✅ **Quality** - Shared code gets better testing

**Total Effort:** ~17 hours over 2 weeks

**Ready to implement?** The plan is detailed, step-by-step, and can be followed by Claude Code or any developer to successfully transform `oxur-cli` into the CLI infrastructure hub for Oxur.

---

*Let's build a solid foundation for Oxur's CLI ecosystem!* 🦀✨
