//! Syntax highlighter for S-expressions
//!
//! Provides color highlighting for Lisp/Oxur code in the REPL:
//! - Rainbow parentheses (depth-based coloring)
//! - Keyword highlighting (deffn, let, if, etc.)
//! - String literals in green
//! - Numeric literals in yellow
//! - Comments in gray

use nu_ansi_term::{Color, Style};
use reedline::{Highlighter, StyledText};

/// S-expression syntax highlighter
///
/// Implements reedline's `Highlighter` trait to provide colorized
/// syntax highlighting for Lisp/Oxur code in the REPL.
#[derive(Clone)]
pub struct SExpHighlighter {
    enabled: bool,
}

impl SExpHighlighter {
    /// Create a new S-expression highlighter
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether color highlighting is enabled
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Check if a token is a keyword
    fn is_keyword(token: &str) -> bool {
        matches!(
            token,
            "deffn"
                | "defn"
                | "let"
                | "if"
                | "cond"
                | "match"
                | "+"
                | "-"
                | "*"
                | "/"
                | "="
                | "<"
                | ">"
                | "and"
                | "or"
                | "not"
                | "true"
                | "false"
                | "nil"
        )
    }

    /// Get color for parentheses based on nesting depth
    fn paren_color(&self, depth: usize) -> Color {
        match depth % 4 {
            1 => Color::Blue,
            2 => Color::Magenta,
            3 => Color::Cyan,
            _ => Color::Yellow,
        }
    }
}

impl Highlighter for SExpHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> StyledText {
        let mut styled = StyledText::new();

        if !self.enabled {
            styled.push((Style::default(), line.to_string()));
            return styled;
        }

        let mut current_token = String::new();
        let mut in_string = false;
        let mut in_comment = false;
        let mut paren_depth = 0;

        for ch in line.chars() {
            match ch {
                ';' if !in_string => {
                    // Flush current token before starting comment
                    if !current_token.is_empty() {
                        styled.push((Style::default(), current_token.clone()));
                        current_token.clear();
                    }
                    in_comment = true;
                    styled.push((Style::new().fg(Color::DarkGray), ch.to_string()));
                }
                '\n' if in_comment => {
                    in_comment = false;
                    styled.push((Style::default(), ch.to_string()));
                }
                _ if in_comment => {
                    styled.push((Style::new().fg(Color::DarkGray), ch.to_string()));
                }
                '"' => {
                    if in_string {
                        // End of string - push the whole string including closing quote
                        current_token.push(ch);
                        styled.push((Style::new().fg(Color::Green), current_token.clone()));
                        current_token.clear();
                        in_string = false;
                    } else {
                        // Start of string - flush any pending token
                        if !current_token.is_empty() {
                            styled.push((Style::default(), current_token.clone()));
                            current_token.clear();
                        }
                        current_token.push(ch);
                        in_string = true;
                    }
                }
                _ if in_string => {
                    current_token.push(ch);
                }
                '(' if !in_string => {
                    // Flush current token before paren
                    if !current_token.is_empty() {
                        styled.push((Style::default(), current_token.clone()));
                        current_token.clear();
                    }
                    paren_depth += 1;
                    let color = self.paren_color(paren_depth);
                    styled.push((Style::new().fg(color).bold(), ch.to_string()));
                }
                ')' if !in_string => {
                    // Flush current token before paren
                    if !current_token.is_empty() {
                        if Self::is_keyword(&current_token) {
                            styled
                                .push((Style::new().fg(Color::Cyan).bold(), current_token.clone()));
                        } else if current_token.parse::<f64>().is_ok() {
                            styled.push((Style::new().fg(Color::Yellow), current_token.clone()));
                        } else {
                            styled.push((Style::default(), current_token.clone()));
                        }
                        current_token.clear();
                    }
                    let color = self.paren_color(paren_depth);
                    styled.push((Style::new().fg(color).bold(), ch.to_string()));
                    paren_depth = paren_depth.saturating_sub(1);
                }
                ' ' | '\t' | '\n' if !in_string => {
                    if !current_token.is_empty() {
                        // Style token based on type
                        if Self::is_keyword(&current_token) {
                            styled
                                .push((Style::new().fg(Color::Cyan).bold(), current_token.clone()));
                        } else if current_token.parse::<f64>().is_ok() {
                            styled.push((Style::new().fg(Color::Yellow), current_token.clone()));
                        } else {
                            styled.push((Style::default(), current_token.clone()));
                        }
                        current_token.clear();
                    }
                    styled.push((Style::default(), ch.to_string()));
                }
                _ => {
                    current_token.push(ch);
                }
            }
        }

        // Flush any remaining token
        if !current_token.is_empty() {
            if in_string {
                // Unclosed string
                styled.push((Style::new().fg(Color::Green), current_token));
            } else if Self::is_keyword(&current_token) {
                styled.push((Style::new().fg(Color::Cyan).bold(), current_token));
            } else if current_token.parse::<f64>().is_ok() {
                styled.push((Style::new().fg(Color::Yellow), current_token));
            } else {
                styled.push((Style::default(), current_token));
            }
        }

        styled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlighter_disabled() {
        let highlighter = SExpHighlighter::new(false);
        let result = highlighter.highlight("(+ 1 2)", 0);
        // When disabled, should return unstyled text
        assert!(!result.buffer.is_empty());
    }

    #[test]
    fn test_highlighter_enabled() {
        let highlighter = SExpHighlighter::new(true);
        let result = highlighter.highlight("(+ 1 2)", 0);
        // When enabled, should have styled segments
        assert!(!result.buffer.is_empty());
    }

    #[test]
    fn test_keyword_detection() {
        assert!(SExpHighlighter::is_keyword("deffn"));
        assert!(SExpHighlighter::is_keyword("+"));
        assert!(SExpHighlighter::is_keyword("if"));
        assert!(!SExpHighlighter::is_keyword("foo"));
        assert!(!SExpHighlighter::is_keyword("my-var"));
    }

    #[test]
    fn test_paren_colors_cycle() {
        let highlighter = SExpHighlighter::new(true);
        let color1 = highlighter.paren_color(1);
        let color2 = highlighter.paren_color(2);
        let color3 = highlighter.paren_color(3);
        let color4 = highlighter.paren_color(4);
        let color5 = highlighter.paren_color(5); // Should wrap to same as depth 1

        assert_eq!(color1, Color::Blue);
        assert_eq!(color2, Color::Magenta);
        assert_eq!(color3, Color::Cyan);
        assert_eq!(color4, Color::Yellow);
        assert_eq!(color5, Color::Blue);
    }

    #[test]
    fn test_highlight_simple_expression() {
        let highlighter = SExpHighlighter::new(true);
        let _result = highlighter.highlight("(+ 1 2)", 0);
        // Just verify it doesn't panic
    }

    #[test]
    fn test_highlight_with_string() {
        let highlighter = SExpHighlighter::new(true);
        let _result = highlighter.highlight(r#"(print "hello")"#, 0);
        // Just verify it doesn't panic
    }

    #[test]
    fn test_highlight_with_comment() {
        let highlighter = SExpHighlighter::new(true);
        let _result = highlighter.highlight("(+ 1 2) ; add numbers", 0);
        // Just verify it doesn't panic
    }

    #[test]
    fn test_highlight_nested_parens() {
        let highlighter = SExpHighlighter::new(true);
        let _result = highlighter.highlight("((+ 1 2))", 0);
        // Just verify it doesn't panic
    }
}
