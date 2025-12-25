//! Color theme for consistent output

use colored::*;

/// Color for success messages
pub fn success(msg: &str) -> ColoredString {
    msg.green()
}

/// Color for error messages
pub fn error(msg: &str) -> ColoredString {
    msg.red()
}

/// Color for warning messages
pub fn warning(msg: &str) -> ColoredString {
    msg.yellow()
}

/// Color for info messages
pub fn info(msg: &str) -> ColoredString {
    msg.cyan()
}

/// Color for document numbers
pub fn doc_number(num: u32) -> ColoredString {
    format!("{:04}", num).bold()
}

/// Color for state badges
pub fn state_badge(state: &str) -> ColoredString {
    match state.to_lowercase().as_str() {
        "draft" => state.yellow(),
        "under review" => state.cyan(),
        "revised" => state.blue(),
        "accepted" => state.green(),
        "active" => state.green().bold(),
        "final" => state.green().bold(),
        "deferred" => state.magenta(),
        "rejected" => state.red(),
        "withdrawn" => state.red(),
        "superseded" => state.red(),
        _ => state.white(),
    }
}

/// Symbol for success
pub fn success_symbol() -> &'static str {
    "✓"
}

/// Symbol for error
pub fn error_symbol() -> &'static str {
    "✗"
}

/// Symbol for warning
pub fn warning_symbol() -> &'static str {
    "⚠"
}

/// Symbol for info
pub fn info_symbol() -> &'static str {
    "→"
}
