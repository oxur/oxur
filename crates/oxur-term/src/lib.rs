//! Themed terminal tables and CLI output helpers for Oxur
//!
//! This crate provides:
//!
//! - **`table`**: Styled table rendering with TOML-based theming (warm orange
//!   Oxur theme), built on [`tabled`](https://docs.rs/tabled).
//! - **`common`**: Shared CLI utilities — file I/O helpers (stdin/stdout/file
//!   handling), colored terminal output (success, error, info, warnings), and
//!   progress tracking for long-running operations.
//!
//! # Examples
//!
//! ## Colored Output
//!
//! ```no_run
//! use oxur_term::common::output::{success, error, info};
//!
//! success("Operation completed!");
//! error("Something went wrong");
//! info("Processing files...");
//! ```
//!
//! ## Styled Tables
//!
//! ```no_run
//! use oxur_term::table::{OxurTable, Tabled};
//!
//! #[derive(Tabled)]
//! struct Row {
//!     #[tabled(rename = "Name")]
//!     name: String,
//! }
//!
//! let table = OxurTable::new(vec![Row { name: "Alice".into() }]).render();
//! println!("{}", table);
//! ```

pub mod common;
pub mod table;

pub use common::progress::ProgressTracker;
