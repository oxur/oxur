//! # Oxur Pretty - S-expression Formatter
//!
//! A library for pretty-printing S-expression formatted data.
//!
//! This crate provides tools for formatting S-expressions in a human-readable way,
//! with support for various formatting strategies including inline, compact structs,
//! and keyword-aligned multiline formatting.
//!
//! ## Features
//!
//! - **Zero dependencies on oxur-ast**: Avoids cyclic dependencies
//! - **Configurable formatting**: Control line width, indentation, and formatting rules
//! - **Idempotent formatting**: Formatting twice produces the same result
//! - **Multiple strategies**: Automatic selection between inline, compact, and multiline formats
//!
//! ## Example
//!
//! ```
//! use oxur_pretty::format_sexp;
//!
//! let input = "(Span :lo 0 :hi 10)";
//! let formatted = format_sexp(input).unwrap();
//! assert_eq!(formatted, "(Span :lo 0 :hi 10)");
//! ```
//!
//! ## Custom Configuration
//!
//! ```
//! use oxur_pretty::{format_sexp_with_config, FormatConfig};
//!
//! let input = "(VeryLongTypeName :key value)";
//! let config = FormatConfig::default()
//!     .with_max_width(20)
//!     .unwrap();
//!
//! let formatted = format_sexp_with_config(input, config).unwrap();
//! // Output will be multiline because it exceeds max_width
//! ```

pub mod cli;
pub mod config;
pub mod error;
pub mod formatter;
pub mod parser;
pub mod rules;

// CLI command modules (public for binary)
pub mod commands;

pub use config::FormatConfig;
pub use error::{FormatterError, Result};
pub use formatter::Formatter;
pub use parser::{parse, SExpr};

/// Format an S-expression string with default configuration.
///
/// This is a convenience function that uses the default [`FormatConfig`]
/// to format the input S-expression.
///
/// # Arguments
///
/// * `input` - The S-expression string to format
///
/// # Returns
///
/// Returns the formatted S-expression string, or a [`FormatterError`] if
/// parsing or formatting fails.
///
/// # Example
///
/// ```
/// use oxur_pretty::format_sexp;
///
/// let input = "(Span :lo 0 :hi 10)";
/// let result = format_sexp(input).unwrap();
/// assert_eq!(result, "(Span :lo 0 :hi 10)");
/// ```
pub fn format_sexp(input: &str) -> Result<String> {
    let config = FormatConfig::default();
    format_sexp_with_config(input, config)
}

/// Format an S-expression string with custom configuration.
///
/// This function allows you to specify custom formatting options via
/// a [`FormatConfig`] struct.
///
/// # Arguments
///
/// * `input` - The S-expression string to format
/// * `config` - The formatting configuration to use
///
/// # Returns
///
/// Returns the formatted S-expression string, or a [`FormatterError`] if
/// parsing or formatting fails.
///
/// # Example
///
/// ```
/// use oxur_pretty::{format_sexp_with_config, FormatConfig};
///
/// let input = "(Item :id 0 :name value)";
/// let config = FormatConfig::default()
///     .with_max_width(80)
///     .unwrap()
///     .with_tab_spaces(4)
///     .unwrap();
///
/// let result = format_sexp_with_config(input, config).unwrap();
/// ```
pub fn format_sexp_with_config(input: &str, config: FormatConfig) -> Result<String> {
    // Validate configuration
    config.validate()?;

    // Parse the input
    let expr = parse(input)?;

    // Format the parsed expression
    let formatter = Formatter::new(config);
    Ok(formatter.format(&expr))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_sexp_simple() {
        let input = "(a b c)";
        let result = format_sexp(input).unwrap();
        assert_eq!(result, "(a b c)");
    }

    #[test]
    fn test_format_sexp_compact_span() {
        let input = "(Span :lo 0 :hi 10)";
        let result = format_sexp(input).unwrap();
        assert_eq!(result, "(Span :lo 0 :hi 10)");
    }

    #[test]
    fn test_format_sexp_nested() {
        let input = r#"(Ident :name "use" :span (Span :lo 0 :hi 3))"#;
        let result = format_sexp(input).unwrap();

        // Short enough to be formatted inline
        let expected = r#"(Ident :name "use" :span (Span :lo 0 :hi 3))"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_sexp_with_config() {
        let input = "(Item :id 0 :name value)";
        let config = FormatConfig::default().with_tab_spaces(4).unwrap();

        let result = format_sexp_with_config(input, config).unwrap();
        // Short enough to be inline regardless of tab_spaces
        let expected = "(Item :id 0 :name value)";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_sexp_empty_input() {
        let result = format_sexp("");
        assert!(matches!(result, Err(FormatterError::EmptyInput)));
    }

    #[test]
    fn test_format_sexp_parse_error() {
        let result = format_sexp("(unclosed");
        assert!(matches!(result, Err(FormatterError::UnmatchedOpen { .. })));
    }

    #[test]
    fn test_format_sexp_invalid_config() {
        let config = FormatConfig {
            max_width: 0, // Invalid
            tab_spaces: 2,
            align_keywords: true,
            compact_simple_values: true,
            max_inline_items: 3,
        };

        let result = format_sexp_with_config("(a b c)", config);
        assert!(matches!(result, Err(FormatterError::InvalidConfig { .. })));
    }

    #[test]
    fn test_format_sexp_idempotent() {
        let input = "(Span :lo 0 :hi 10)";

        let formatted1 = format_sexp(input).unwrap();
        let formatted2 = format_sexp(&formatted1).unwrap();

        assert_eq!(formatted1, formatted2);
    }

    #[test]
    fn test_format_sexp_complex() {
        let input = r#"(Crate :attrs () :items ((Item :id 0 :span (Span :lo 0 :hi 0))))"#;
        let result = format_sexp(input).unwrap();

        // Should be formatted with proper indentation
        assert!(result.contains('\n'));
        assert!(result.contains("  :attrs ()"));
    }
}
