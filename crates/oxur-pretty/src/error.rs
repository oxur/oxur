//! Error types for the oxur-pretty formatter.

use thiserror::Error;

/// Result type alias for formatter operations.
pub type Result<T> = std::result::Result<T, FormatterError>;

/// Errors that can occur during formatting operations.
#[derive(Debug, Error)]
pub enum FormatterError {
    /// Unterminated string literal
    #[error("Unterminated string at position {pos}")]
    UnterminatedString { pos: usize },

    /// Unmatched opening parenthesis
    #[error("Unmatched opening parenthesis at position {pos}")]
    UnmatchedOpen { pos: usize },

    /// Unmatched closing parenthesis
    #[error("Unmatched closing parenthesis at position {pos}")]
    UnmatchedClose { pos: usize },

    /// Invalid escape sequence
    #[error("Invalid escape sequence '\\{ch}' at position {pos}")]
    InvalidEscape { ch: char, pos: usize },

    /// Empty input provided
    #[error("Empty input provided")]
    EmptyInput,

    /// Invalid configuration
    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },

    /// I/O error occurred
    #[error("I/O error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
}

impl FormatterError {
    /// Create an unterminated string error.
    pub fn unterminated_string(pos: usize) -> Self {
        Self::UnterminatedString { pos }
    }

    /// Create an unmatched open parenthesis error.
    pub fn unmatched_open(pos: usize) -> Self {
        Self::UnmatchedOpen { pos }
    }

    /// Create an unmatched close parenthesis error.
    pub fn unmatched_close(pos: usize) -> Self {
        Self::UnmatchedClose { pos }
    }

    /// Create an invalid escape sequence error.
    pub fn invalid_escape(ch: char, pos: usize) -> Self {
        Self::InvalidEscape { ch, pos }
    }

    /// Create an empty input error.
    pub fn empty_input() -> Self {
        Self::EmptyInput
    }

    /// Create an invalid configuration error.
    pub fn invalid_config(message: impl Into<String>) -> Self {
        Self::InvalidConfig { message: message.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unterminated_string_error() {
        let err = FormatterError::unterminated_string(42);
        assert_eq!(err.to_string(), "Unterminated string at position 42");
    }

    #[test]
    fn test_unmatched_open_error() {
        let err = FormatterError::unmatched_open(10);
        assert_eq!(err.to_string(), "Unmatched opening parenthesis at position 10");
    }

    #[test]
    fn test_unmatched_close_error() {
        let err = FormatterError::unmatched_close(15);
        assert_eq!(err.to_string(), "Unmatched closing parenthesis at position 15");
    }

    #[test]
    fn test_invalid_escape_error() {
        let err = FormatterError::invalid_escape('x', 20);
        assert_eq!(err.to_string(), "Invalid escape sequence '\\x' at position 20");
    }

    #[test]
    fn test_empty_input_error() {
        let err = FormatterError::empty_input();
        assert_eq!(err.to_string(), "Empty input provided");
    }

    #[test]
    fn test_invalid_config_error() {
        let err = FormatterError::invalid_config("max_width must be > 0");
        assert_eq!(err.to_string(), "Invalid configuration: max_width must be > 0");
    }
}
