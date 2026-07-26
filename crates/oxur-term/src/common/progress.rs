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
/// use oxur_term::common::progress::ProgressTracker;
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
