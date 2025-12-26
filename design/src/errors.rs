//! Error handling utilities

use colored::*;

/// Print a formatted error message
pub fn print_error(context: &str, error: &anyhow::Error) {
    eprintln!("{} {}", "Error:".red().bold(), context);
    eprintln!("  {}", error.to_string().red());

    // Show chain of causes
    let mut current = error.source();
    while let Some(cause) = current {
        eprintln!("  {} {}", "Caused by:".dimmed(), cause.to_string().dimmed());
        current = std::error::Error::source(cause);
    }
}

/// Print an error with a suggestion
pub fn print_error_with_suggestion(context: &str, error: &anyhow::Error, suggestion: &str) {
    print_error(context, error);
    eprintln!("\n{} {}", "Suggestion:".cyan().bold(), suggestion);
}

/// Print a warning message
pub fn print_warning(message: &str) {
    eprintln!("{} {}", "Warning:".yellow().bold(), message);
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    // Helper to create a chain of errors
    fn create_error_chain() -> anyhow::Error {
        anyhow!("root cause")
            .context("intermediate error")
            .context("top level error")
    }

    #[test]
    fn test_print_error_simple() {
        // Simple error without chain
        let error = anyhow!("simple error message");

        // Just verify it doesn't panic - output goes to stderr
        print_error("operation failed", &error);
    }

    #[test]
    fn test_print_error_with_chain() {
        // Error with chain of causes
        let error = create_error_chain();

        // Verify it doesn't panic with chained errors
        print_error("complex operation failed", &error);
    }

    #[test]
    fn test_print_error_with_suggestion_simple() {
        let error = anyhow!("file not found");
        let suggestion = "Make sure the file exists and you have permission to read it";

        // Verify it doesn't panic
        print_error_with_suggestion("read operation failed", &error, suggestion);
    }

    #[test]
    fn test_print_error_with_suggestion_chain() {
        let error = create_error_chain();
        let suggestion = "Try running with --verbose for more details";

        // Verify it doesn't panic with chained errors
        print_error_with_suggestion("operation failed", &error, suggestion);
    }

    #[test]
    fn test_print_warning() {
        // Simple warning
        print_warning("this is a warning message");
    }

    #[test]
    fn test_print_warning_empty() {
        // Empty warning
        print_warning("");
    }

    #[test]
    fn test_print_warning_special_chars() {
        // Warning with special characters
        print_warning("warning: path/to/file contains special chars: @#$%");
    }
}
