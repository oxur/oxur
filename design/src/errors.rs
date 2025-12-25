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
