//! S-expression parser for the oxur-pretty formatter.

use crate::error::{FormatterError, Result};

/// An S-expression node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SExpr {
    /// An atomic value (keyword, symbol, string, number, etc.)
    /// For strings, the quotes are preserved: `Atom("\"hello\"")`
    Atom(String),
    /// A list of S-expressions
    List(Vec<SExpr>),
}

impl SExpr {
    /// Check if this is an atom.
    pub fn is_atom(&self) -> bool {
        matches!(self, SExpr::Atom(_))
    }

    /// Check if this is a list.
    pub fn is_list(&self) -> bool {
        matches!(self, SExpr::List(_))
    }

    /// Get the atom value if this is an atom.
    pub fn as_atom(&self) -> Option<&str> {
        match self {
            SExpr::Atom(s) => Some(s),
            _ => None,
        }
    }

    /// Get the list items if this is a list.
    pub fn as_list(&self) -> Option<&[SExpr]> {
        match self {
            SExpr::List(items) => Some(items),
            _ => None,
        }
    }
}

/// Parse an S-expression from a string.
pub fn parse(input: &str) -> Result<SExpr> {
    let mut parser = Parser::new(input);
    parser.parse_expr()
}

/// S-expression parser.
struct Parser {
    input: Vec<char>,
    pos: usize,
}

impl Parser {
    /// Create a new parser for the given input.
    fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    /// Parse a single S-expression.
    fn parse_expr(&mut self) -> Result<SExpr> {
        self.skip_whitespace_and_comments();

        if self.is_at_end() {
            return Err(FormatterError::empty_input());
        }

        match self.peek() {
            Some('(') => self.parse_list(),
            Some('"') => self.parse_string(),
            Some(_) => self.parse_atom(),
            None => Err(FormatterError::empty_input()),
        }
    }

    /// Parse a list: `(...)`.
    fn parse_list(&mut self) -> Result<SExpr> {
        let open_pos = self.pos;
        self.expect('(')?;

        let mut items = Vec::new();

        loop {
            self.skip_whitespace_and_comments();

            if self.is_at_end() {
                return Err(FormatterError::unmatched_open(open_pos));
            }

            if self.peek() == Some(')') {
                self.advance();
                break;
            }

            items.push(self.parse_expr()?);
        }

        Ok(SExpr::List(items))
    }

    /// Parse an atom (keyword, symbol, number, etc.).
    fn parse_atom(&mut self) -> Result<SExpr> {
        let start = self.pos;

        while let Some(ch) = self.peek() {
            if ch.is_whitespace() || ch == '(' || ch == ')' || ch == ';' {
                break;
            }
            self.advance();
        }

        if start == self.pos {
            return Err(FormatterError::empty_input());
        }

        let atom: String = self.input[start..self.pos].iter().collect();
        Ok(SExpr::Atom(atom))
    }

    /// Parse a quoted string: `"..."`.
    /// Preserves the quotes in the result: `Atom("\"hello\"")`
    fn parse_string(&mut self) -> Result<SExpr> {
        let start_pos = self.pos;
        self.expect('"')?;

        let mut result = String::from("\"");

        loop {
            match self.peek() {
                Some('"') => {
                    result.push('"');
                    self.advance();
                    break;
                }
                Some('\\') => {
                    result.push('\\');
                    self.advance();

                    match self.peek() {
                        Some(ch @ ('"' | '\\' | 'n' | 't' | 'r')) => {
                            result.push(ch);
                            self.advance();
                        }
                        Some(ch) => {
                            return Err(FormatterError::invalid_escape(ch, self.pos));
                        }
                        None => {
                            return Err(FormatterError::unterminated_string(start_pos));
                        }
                    }
                }
                Some(ch) => {
                    result.push(ch);
                    self.advance();
                }
                None => {
                    return Err(FormatterError::unterminated_string(start_pos));
                }
            }
        }

        Ok(SExpr::Atom(result))
    }

    /// Skip whitespace and comments.
    /// Comments start with `;` and go to end of line.
    fn skip_whitespace_and_comments(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else if ch == ';' {
                // Skip until end of line
                while let Some(ch) = self.peek() {
                    self.advance();
                    if ch == '\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    /// Peek at the current character without consuming it.
    fn peek(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    /// Advance to the next character.
    fn advance(&mut self) {
        if self.pos < self.input.len() {
            self.pos += 1;
        }
    }

    /// Expect a specific character and consume it.
    fn expect(&mut self, expected: char) -> Result<()> {
        match self.peek() {
            Some(ch) if ch == expected => {
                self.advance();
                Ok(())
            }
            Some(ch) => Err(FormatterError::invalid_config(format!(
                "Expected '{}', found '{}' at position {}",
                expected, ch, self.pos
            ))),
            None => Err(FormatterError::invalid_config(format!(
                "Expected '{}', found end of input",
                expected
            ))),
        }
    }

    /// Check if we're at the end of the input.
    fn is_at_end(&self) -> bool {
        self.pos >= self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sexpr_is_atom() {
        let atom = SExpr::Atom("foo".to_string());
        assert!(atom.is_atom());
        assert!(!atom.is_list());
    }

    #[test]
    fn test_sexpr_is_list() {
        let list = SExpr::List(vec![]);
        assert!(list.is_list());
        assert!(!list.is_atom());
    }

    #[test]
    fn test_sexpr_as_atom() {
        let atom = SExpr::Atom("foo".to_string());
        assert_eq!(atom.as_atom(), Some("foo"));

        let list = SExpr::List(vec![]);
        assert_eq!(list.as_atom(), None);
    }

    #[test]
    fn test_sexpr_as_list() {
        let list = SExpr::List(vec![SExpr::Atom("foo".to_string())]);
        assert_eq!(list.as_list().map(|l| l.len()), Some(1));

        let atom = SExpr::Atom("foo".to_string());
        assert_eq!(atom.as_list(), None);
    }

    #[test]
    fn test_parse_simple_atom() {
        let result = parse("foo").unwrap();
        assert_eq!(result, SExpr::Atom("foo".to_string()));
    }

    #[test]
    fn test_parse_keyword() {
        let result = parse(":name").unwrap();
        assert_eq!(result, SExpr::Atom(":name".to_string()));
    }

    #[test]
    fn test_parse_number() {
        let result = parse("42").unwrap();
        assert_eq!(result, SExpr::Atom("42".to_string()));
    }

    #[test]
    fn test_parse_empty_list() {
        let result = parse("()").unwrap();
        assert_eq!(result, SExpr::List(vec![]));
    }

    #[test]
    fn test_parse_simple_list() {
        let result = parse("(a b c)").unwrap();
        assert_eq!(
            result,
            SExpr::List(vec![
                SExpr::Atom("a".to_string()),
                SExpr::Atom("b".to_string()),
                SExpr::Atom("c".to_string()),
            ])
        );
    }

    #[test]
    fn test_parse_nested_list() {
        let result = parse("(a (b c) d)").unwrap();
        assert_eq!(
            result,
            SExpr::List(vec![
                SExpr::Atom("a".to_string()),
                SExpr::List(vec![
                    SExpr::Atom("b".to_string()),
                    SExpr::Atom("c".to_string()),
                ]),
                SExpr::Atom("d".to_string()),
            ])
        );
    }

    #[test]
    fn test_parse_string() {
        let result = parse(r#""hello""#).unwrap();
        assert_eq!(result, SExpr::Atom(r#""hello""#.to_string()));
    }

    #[test]
    fn test_parse_string_with_escapes() {
        let result = parse(r#""hello\nworld""#).unwrap();
        assert_eq!(result, SExpr::Atom(r#""hello\nworld""#.to_string()));
    }

    #[test]
    fn test_parse_string_with_quote_escape() {
        let result = parse(r#""say \"hi\"""#).unwrap();
        assert_eq!(result, SExpr::Atom(r#""say \"hi\"""#.to_string()));
    }

    #[test]
    fn test_parse_string_with_backslash_escape() {
        let result = parse(r#""path\\to\\file""#).unwrap();
        assert_eq!(result, SExpr::Atom(r#""path\\to\\file""#.to_string()));
    }

    #[test]
    fn test_parse_whitespace_handling() {
        let result = parse("  ( a   b    c  )  ").unwrap();
        assert_eq!(
            result,
            SExpr::List(vec![
                SExpr::Atom("a".to_string()),
                SExpr::Atom("b".to_string()),
                SExpr::Atom("c".to_string()),
            ])
        );
    }

    #[test]
    fn test_parse_comment() {
        let result = parse("; this is a comment\n(a b)").unwrap();
        assert_eq!(
            result,
            SExpr::List(vec![
                SExpr::Atom("a".to_string()),
                SExpr::Atom("b".to_string()),
            ])
        );
    }

    #[test]
    fn test_parse_inline_comment() {
        let result = parse("(a ; comment\n b)").unwrap();
        assert_eq!(
            result,
            SExpr::List(vec![
                SExpr::Atom("a".to_string()),
                SExpr::Atom("b".to_string()),
            ])
        );
    }

    #[test]
    fn test_parse_keyword_value_pairs() {
        let result = parse("(Span :lo 0 :hi 10)").unwrap();
        assert_eq!(
            result,
            SExpr::List(vec![
                SExpr::Atom("Span".to_string()),
                SExpr::Atom(":lo".to_string()),
                SExpr::Atom("0".to_string()),
                SExpr::Atom(":hi".to_string()),
                SExpr::Atom("10".to_string()),
            ])
        );
    }

    #[test]
    fn test_parse_empty_input_error() {
        let result = parse("");
        assert!(matches!(result, Err(FormatterError::EmptyInput)));
    }

    #[test]
    fn test_parse_whitespace_only_error() {
        let result = parse("   \n  \t  ");
        assert!(matches!(result, Err(FormatterError::EmptyInput)));
    }

    #[test]
    fn test_parse_unterminated_string_error() {
        let result = parse(r#""hello"#);
        assert!(matches!(
            result,
            Err(FormatterError::UnterminatedString { pos: 0 })
        ));
    }

    #[test]
    fn test_parse_invalid_escape_error() {
        let result = parse(r#""hello\x""#);
        assert!(matches!(
            result,
            Err(FormatterError::InvalidEscape { ch: 'x', pos: 7 })
        ));
    }

    #[test]
    fn test_parse_unmatched_open_error() {
        let result = parse("(a b c");
        assert!(matches!(
            result,
            Err(FormatterError::UnmatchedOpen { pos: 0 })
        ));
    }

    #[test]
    fn test_parse_complex_expression() {
        let input = r#"(Ident :name "use" :span (Span :lo 0 :hi 3))"#;
        let result = parse(input).unwrap();

        assert!(result.is_list());
        let items = result.as_list().unwrap();
        assert_eq!(items.len(), 5);
        assert_eq!(items[0], SExpr::Atom("Ident".to_string()));
        assert_eq!(items[1], SExpr::Atom(":name".to_string()));
        assert_eq!(items[2], SExpr::Atom(r#""use""#.to_string()));
        assert_eq!(items[3], SExpr::Atom(":span".to_string()));

        // Check nested Span
        assert!(items[4].is_list());
        let span_items = items[4].as_list().unwrap();
        assert_eq!(span_items.len(), 5);
        assert_eq!(span_items[0], SExpr::Atom("Span".to_string()));
    }
}
