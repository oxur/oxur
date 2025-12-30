//! Oxur CLI library and binary
//!
//! This crate provides two things:
//!
//! 1. **Library**: Common utilities for building Oxur CLI tools
//!    - File I/O helpers (stdin/stdout/file handling)
//!    - Colored terminal output (success, error, info, warnings)
//!    - Progress tracking for long-running operations
//!
//! 2. **Binary**: The unified `oxur` command-line tool
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
