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
