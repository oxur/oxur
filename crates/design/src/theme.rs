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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_color() {
        let colored = success("test message");
        assert_eq!(colored.to_string().contains("test message"), true);
    }

    #[test]
    fn test_error_color() {
        let colored = error("error message");
        assert_eq!(colored.to_string().contains("error message"), true);
    }

    #[test]
    fn test_warning_color() {
        let colored = warning("warning message");
        assert_eq!(colored.to_string().contains("warning message"), true);
    }

    #[test]
    fn test_info_color() {
        let colored = info("info message");
        assert_eq!(colored.to_string().contains("info message"), true);
    }

    #[test]
    fn test_doc_number_formatting() {
        let colored = doc_number(1);
        assert_eq!(colored.to_string().contains("0001"), true);

        let colored = doc_number(42);
        assert_eq!(colored.to_string().contains("0042"), true);

        let colored = doc_number(9999);
        assert_eq!(colored.to_string().contains("9999"), true);
    }

    #[test]
    fn test_doc_number_zero() {
        let colored = doc_number(0);
        assert_eq!(colored.to_string().contains("0000"), true);
    }

    #[test]
    fn test_state_badge_draft() {
        let colored = state_badge("draft");
        assert_eq!(colored.to_string().contains("draft"), true);

        let colored = state_badge("Draft");
        assert_eq!(colored.to_string().contains("Draft"), true);
    }

    #[test]
    fn test_state_badge_under_review() {
        let colored = state_badge("under review");
        assert_eq!(colored.to_string().contains("under review"), true);

        let colored = state_badge("Under Review");
        assert_eq!(colored.to_string().contains("Under Review"), true);
    }

    #[test]
    fn test_state_badge_revised() {
        let colored = state_badge("revised");
        assert_eq!(colored.to_string().contains("revised"), true);
    }

    #[test]
    fn test_state_badge_accepted() {
        let colored = state_badge("accepted");
        assert_eq!(colored.to_string().contains("accepted"), true);
    }

    #[test]
    fn test_state_badge_active() {
        let colored = state_badge("active");
        assert_eq!(colored.to_string().contains("active"), true);
    }

    #[test]
    fn test_state_badge_final() {
        let colored = state_badge("final");
        assert_eq!(colored.to_string().contains("final"), true);
    }

    #[test]
    fn test_state_badge_deferred() {
        let colored = state_badge("deferred");
        assert_eq!(colored.to_string().contains("deferred"), true);
    }

    #[test]
    fn test_state_badge_rejected() {
        let colored = state_badge("rejected");
        assert_eq!(colored.to_string().contains("rejected"), true);
    }

    #[test]
    fn test_state_badge_withdrawn() {
        let colored = state_badge("withdrawn");
        assert_eq!(colored.to_string().contains("withdrawn"), true);
    }

    #[test]
    fn test_state_badge_superseded() {
        let colored = state_badge("superseded");
        assert_eq!(colored.to_string().contains("superseded"), true);
    }

    #[test]
    fn test_state_badge_unknown() {
        // Unknown states should use default white color
        let colored = state_badge("unknown");
        assert_eq!(colored.to_string().contains("unknown"), true);

        let colored = state_badge("invalid-state");
        assert_eq!(colored.to_string().contains("invalid-state"), true);
    }

    #[test]
    fn test_success_symbol() {
        assert_eq!(success_symbol(), "✓");
    }

    #[test]
    fn test_error_symbol() {
        assert_eq!(error_symbol(), "✗");
    }

    #[test]
    fn test_warning_symbol() {
        assert_eq!(warning_symbol(), "⚠");
    }

    #[test]
    fn test_info_symbol() {
        assert_eq!(info_symbol(), "→");
    }

    #[test]
    fn test_empty_strings() {
        // Test with empty strings
        let _colored = success("");
        let _colored = error("");
        let _colored = warning("");
        let _colored = info("");
        let _colored = state_badge("");
    }

    #[test]
    fn test_special_characters() {
        // Test with special characters
        let colored = success("✓ all good!");
        assert_eq!(colored.to_string().contains("✓"), true);

        let colored = error("path/to/file@#$%");
        assert_eq!(colored.to_string().contains("path/to/file@#$%"), true);
    }
}
