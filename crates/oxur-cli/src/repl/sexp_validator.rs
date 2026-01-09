//! Validator for multi-line S-expression editing
//!
//! Checks if S-expressions have balanced parentheses, brackets, and braces
//! to enable multi-line editing in the REPL. When brackets are unbalanced,
//! reedline will show the continuation prompt instead of evaluating.

use reedline::{ValidationResult, Validator};

/// S-expression validator for multi-line editing
///
/// Implements reedline's `Validator` trait to determine if input is complete
/// or needs more lines. Checks for:
/// - Balanced parentheses `()`
/// - Balanced square brackets `[]`
/// - Balanced curly braces `{}`
/// - Properly closed strings
#[derive(Clone)]
pub struct SExpValidator;

impl SExpValidator {
    /// Create a new S-expression validator
    pub fn new() -> Self {
        Self
    }

    /// Check if input has balanced brackets and closed strings
    ///
    /// Returns `true` if all brackets are balanced and strings are closed.
    fn is_balanced(line: &str) -> bool {
        let mut paren_count = 0;
        let mut bracket_count = 0;
        let mut brace_count = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for ch in line.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '(' if !in_string => paren_count += 1,
                ')' if !in_string => paren_count -= 1,
                '[' if !in_string => bracket_count += 1,
                ']' if !in_string => bracket_count -= 1,
                '{' if !in_string => brace_count += 1,
                '}' if !in_string => brace_count -= 1,
                _ => {}
            }

            // Early exit if we have negative counts (more closing than opening)
            if paren_count < 0 || bracket_count < 0 || brace_count < 0 {
                return false;
            }
        }

        // All brackets must be balanced and strings must be closed
        paren_count == 0 && bracket_count == 0 && brace_count == 0 && !in_string
    }
}

impl Default for SExpValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator for SExpValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        if Self::is_balanced(line) {
            ValidationResult::Complete
        } else {
            ValidationResult::Incomplete
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balanced_simple_expression() {
        assert!(SExpValidator::is_balanced("(+ 1 2)"));
    }

    #[test]
    fn test_balanced_nested_parens() {
        assert!(SExpValidator::is_balanced("((+ 1 2) (* 3 4))"));
    }

    #[test]
    fn test_balanced_with_brackets() {
        assert!(SExpValidator::is_balanced("[1 2 3]"));
    }

    #[test]
    fn test_balanced_with_braces() {
        assert!(SExpValidator::is_balanced("{:a 1 :b 2}"));
    }

    #[test]
    fn test_balanced_mixed_brackets() {
        assert!(SExpValidator::is_balanced("(let [x 1] {:result x})"));
    }

    #[test]
    fn test_balanced_with_string() {
        assert!(SExpValidator::is_balanced(r#"(print "hello")"#));
    }

    #[test]
    fn test_balanced_with_escaped_quote() {
        assert!(SExpValidator::is_balanced(r#"(print "say \"hi\"")"#));
    }

    #[test]
    fn test_unbalanced_open_paren() {
        assert!(!SExpValidator::is_balanced("(+ 1 2"));
    }

    #[test]
    fn test_unbalanced_close_paren() {
        assert!(!SExpValidator::is_balanced("+ 1 2)"));
    }

    #[test]
    fn test_unbalanced_nested() {
        assert!(!SExpValidator::is_balanced("((+ 1 2)"));
    }

    #[test]
    fn test_unbalanced_bracket() {
        assert!(!SExpValidator::is_balanced("[1 2 3"));
    }

    #[test]
    fn test_unbalanced_brace() {
        assert!(!SExpValidator::is_balanced("{:a 1"));
    }

    #[test]
    fn test_unclosed_string() {
        assert!(!SExpValidator::is_balanced(r#"(print "hello"#));
    }

    #[test]
    fn test_string_with_paren_inside() {
        assert!(SExpValidator::is_balanced(r#"(print "foo(bar)")"#));
    }

    #[test]
    fn test_empty_string() {
        assert!(SExpValidator::is_balanced(""));
    }

    #[test]
    fn test_validator_complete() {
        let validator = SExpValidator::new();
        assert!(matches!(
            validator.validate("(+ 1 2)"),
            ValidationResult::Complete
        ));
    }

    #[test]
    fn test_validator_incomplete() {
        let validator = SExpValidator::new();
        assert!(matches!(
            validator.validate("(+ 1 2"),
            ValidationResult::Incomplete
        ));
    }

    #[test]
    fn test_multiline_complete() {
        let input = "(deffn square [x]\n  (* x x))";
        assert!(SExpValidator::is_balanced(input));
    }

    #[test]
    fn test_multiline_incomplete() {
        let input = "(deffn square [x]\n  (* x x)";
        assert!(!SExpValidator::is_balanced(input));
    }
}
