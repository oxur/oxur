//! Stage 1: Parse
//!
//! Converts raw Oxur source text into Surface Forms (S-expression AST).
//! Handles tokenization, reader, and reader macros.

use crate::Result;
use oxur_smap::Span;

/// Parser converts Oxur source text into Surface Forms
pub struct Parser {
    source: String,
    position: usize,  // Byte offset in source
    line: usize,      // Current line (1-indexed)
    column: usize,    // Current column (1-indexed)
    filename: String, // Source filename (or "<repl>")
}

impl Parser {
    pub fn new(source: String) -> Self {
        Self {
            source,
            position: 0,
            line: 1,   // 1-indexed
            column: 1, // 1-indexed
            filename: "<repl>".to_string(),
        }
    }

    /// Create a parser for a named file
    pub fn new_file(source: String, filename: String) -> Self {
        Self { source, position: 0, line: 1, column: 1, filename }
    }

    /// Parse the source into Surface Forms
    pub fn parse(&mut self) -> Result<Vec<SurfaceForm>> {
        let mut forms = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace();
            if self.is_at_end() {
                break;
            }
            forms.push(self.parse_form()?);
        }

        Ok(forms)
    }

    fn parse_form(&mut self) -> Result<SurfaceForm> {
        self.skip_whitespace();

        if self.is_at_end() {
            return Err(crate::Error::Syntax("Unexpected end of input".to_string()));
        }

        let ch = self.current_char();

        match ch {
            '(' => self.parse_list(),
            '"' => self.parse_string(),
            '0'..='9' | '-' => self.parse_number(),
            _ => self.parse_symbol(),
        }
    }

    fn parse_list(&mut self) -> Result<SurfaceForm> {
        let (start_line, start_column) = self.mark_position();

        self.advance(); // consume '('
        let mut elements = Vec::new();

        loop {
            self.skip_whitespace();

            if self.is_at_end() {
                return Err(crate::Error::Syntax("Unclosed list".to_string()));
            }

            if self.current_char() == ')' {
                self.advance(); // consume ')'
                break;
            }

            elements.push(self.parse_form()?);
        }

        let span = self.make_span(start_line, start_column);
        Ok(SurfaceForm::List { span, elements })
    }

    fn parse_string(&mut self) -> Result<SurfaceForm> {
        let (start_line, start_column) = self.mark_position();

        self.advance(); // consume opening '"'
        let start = self.position;

        while !self.is_at_end() && self.current_char() != '"' {
            self.advance();
        }

        if self.is_at_end() {
            return Err(crate::Error::Syntax("Unclosed string".to_string()));
        }

        let value = self.source[start..self.position].to_string();
        self.advance(); // consume closing '"'

        let span = self.make_span(start_line, start_column);
        Ok(SurfaceForm::String { span, value })
    }

    fn parse_number(&mut self) -> Result<SurfaceForm> {
        let (start_line, start_column) = self.mark_position();
        let start = self.position;

        if self.current_char() == '-' {
            self.advance();
        }

        while !self.is_at_end() && self.current_char().is_ascii_digit() {
            self.advance();
        }

        let num_str = &self.source[start..self.position];
        let value = num_str
            .parse::<i64>()
            .map_err(|_| crate::Error::Syntax(format!("Invalid number: {}", num_str)))?;

        let span = self.make_span(start_line, start_column);
        Ok(SurfaceForm::Number { span, value })
    }

    fn parse_symbol(&mut self) -> Result<SurfaceForm> {
        let (start_line, start_column) = self.mark_position();
        let start = self.position;

        while !self.is_at_end() && self.is_symbol_char(self.current_char()) {
            self.advance();
        }

        let name = self.source[start..self.position].to_string();
        let span = self.make_span(start_line, start_column);
        Ok(SurfaceForm::Symbol { span, name })
    }

    fn is_symbol_char(&self, ch: char) -> bool {
        !ch.is_whitespace() && ch != '(' && ch != ')' && ch != '"'
    }

    fn current_char(&self) -> char {
        self.source.chars().nth(self.position).unwrap()
    }

    fn advance(&mut self) {
        if self.position < self.source.len() {
            let ch = self.current_char();
            self.position += 1;

            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() && self.current_char().is_whitespace() {
            self.advance();
        }
    }

    /// Get current position as (line, column) tuple
    fn current_pos(&self) -> (u32, u32) {
        (self.line as u32, self.column as u32)
    }

    /// Mark current position for span tracking
    fn mark_position(&self) -> (u32, u32) {
        self.current_pos()
    }

    /// Create a span from start position to current position
    fn make_span(&self, start_line: u32, start_column: u32) -> Span {
        let (end_line, end_column) = self.current_pos();
        Span::new(self.filename.clone(), start_line, start_column, end_line, end_column)
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }
}

/// Surface Forms - parsed S-expressions before macro expansion
///
/// Each variant includes a Span tracking its source location for
/// error reporting and debugging.
#[derive(Debug, Clone)]
pub enum SurfaceForm {
    /// A symbol (identifier, operator, etc.)
    Symbol { span: Span, name: String },

    /// A numeric literal
    Number { span: Span, value: i64 },

    /// A string literal
    String { span: Span, value: String },

    /// A list (parenthesized expression)
    List { span: Span, elements: Vec<SurfaceForm> },
}

impl SurfaceForm {
    /// Get the span of this surface form
    pub fn span(&self) -> &Span {
        match self {
            SurfaceForm::Symbol { span, .. } => span,
            SurfaceForm::Number { span, .. } => span,
            SurfaceForm::String { span, .. } => span,
            SurfaceForm::List { span, .. } => span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = Parser::new("(+ 1 2)".to_string());
        assert_eq!(parser.position, 0);
    }

    #[test]
    fn test_parse_empty() {
        let mut parser = Parser::new("".to_string());
        let result = parser.parse();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_surface_form_symbol() {
        let span = Span::repl(1, 1, 1, 5);
        let form = SurfaceForm::Symbol { span, name: "test".to_string() };
        match form {
            SurfaceForm::Symbol { name, .. } => assert_eq!(name, "test"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_surface_form_number() {
        let span = Span::repl(1, 1, 1, 3);
        let form = SurfaceForm::Number { span, value: 42 };
        match form {
            SurfaceForm::Number { value, .. } => assert_eq!(value, 42),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_surface_form_string() {
        let span = Span::repl(1, 1, 1, 7);
        let form = SurfaceForm::String { span, value: "hello".to_string() };
        match form {
            SurfaceForm::String { value, .. } => assert_eq!(value, "hello"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_surface_form_list() {
        let span = Span::repl(1, 1, 1, 3);
        let form = SurfaceForm::List { span, elements: vec![] };
        match form {
            SurfaceForm::List { elements, .. } => assert_eq!(elements.len(), 0),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_parse_hello_world() {
        let source = r#"(deffn main ()
  (println! "Hello, world!"))"#;
        let mut parser = Parser::new(source.to_string());
        let result = parser.parse();

        assert!(result.is_ok());
        let forms = result.unwrap();
        assert_eq!(forms.len(), 1);

        // Should be a list starting with 'deffn'
        if let SurfaceForm::List { elements, .. } = &forms[0] {
            assert!(elements.len() >= 3);
            if let SurfaceForm::Symbol { name, .. } = &elements[0] {
                assert_eq!(name, "deffn");
            } else {
                panic!("Expected Symbol(deffn)");
            }
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_parse_simple_list() {
        let mut parser = Parser::new("(+ 1 2)".to_string());
        let result = parser.parse();

        assert!(result.is_ok());
        let forms = result.unwrap();
        assert_eq!(forms.len(), 1);

        if let SurfaceForm::List { elements, .. } = &forms[0] {
            assert_eq!(elements.len(), 3);
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_parse_string() {
        let mut parser = Parser::new(r#""hello""#.to_string());
        let result = parser.parse();

        assert!(result.is_ok());
        let forms = result.unwrap();
        assert_eq!(forms.len(), 1);

        if let SurfaceForm::String { value, .. } = &forms[0] {
            assert_eq!(value, "hello");
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_parse_number() {
        let mut parser = Parser::new("42".to_string());
        let result = parser.parse();

        assert!(result.is_ok());
        let forms = result.unwrap();
        assert_eq!(forms.len(), 1);

        if let SurfaceForm::Number { value, .. } = &forms[0] {
            assert_eq!(*value, 42);
        } else {
            panic!("Expected Number");
        }
    }

    #[test]
    fn test_parse_symbol() {
        let mut parser = Parser::new("println!".to_string());
        let result = parser.parse();

        assert!(result.is_ok());
        let forms = result.unwrap();
        assert_eq!(forms.len(), 1);

        if let SurfaceForm::Symbol { name, .. } = &forms[0] {
            assert_eq!(name, "println!");
        } else {
            panic!("Expected Symbol");
        }
    }

    #[test]
    fn test_span_tracking_symbol() {
        let mut parser = Parser::new("hello".to_string());
        let forms = parser.parse().unwrap();

        if let SurfaceForm::Symbol { span, name } = &forms[0] {
            assert_eq!(name, "hello");
            assert_eq!(span.start_line, 1);
            assert_eq!(span.start_column, 1);
            assert_eq!(span.end_line, 1);
            assert_eq!(span.end_column, 6); // After 'o'
        } else {
            panic!("Expected Symbol");
        }
    }

    #[test]
    fn test_span_tracking_list() {
        let mut parser = Parser::new("(+ 1 2)".to_string());
        let forms = parser.parse().unwrap();

        if let SurfaceForm::List { span, elements } = &forms[0] {
            assert_eq!(elements.len(), 3);
            assert_eq!(span.start_line, 1);
            assert_eq!(span.start_column, 1);
            assert_eq!(span.end_line, 1);
            assert_eq!(span.end_column, 8); // After ')'
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_span_tracking_multiline() {
        let source = r#"(deffn main ()
  (println! "test"))"#;
        let mut parser = Parser::new(source.to_string());
        let forms = parser.parse().unwrap();

        if let SurfaceForm::List { span, .. } = &forms[0] {
            assert_eq!(span.start_line, 1);
            assert_eq!(span.start_column, 1);
            assert_eq!(span.end_line, 2);
            // Should span to end of second line
            assert!(span.end_line > span.start_line);
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_parser_new_file() {
        let parser = Parser::new_file("(+ 1 2)".to_string(), "test.oxur".to_string());
        assert_eq!(parser.filename, "test.oxur");
        assert_eq!(parser.line, 1);
        assert_eq!(parser.column, 1);
    }

    #[test]
    fn test_current_position() {
        let parser = Parser::new("hello".to_string());
        let (line, col) = parser.current_pos();
        assert_eq!(line, 1);
        assert_eq!(col, 1);
    }
}
