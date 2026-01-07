//! Stage 1: Parse
//!
//! Converts raw Oxur source text into Surface Forms (S-expression AST).
//! Handles tokenization, reader, and reader macros.

use crate::Result;

/// Parser converts Oxur source text into Surface Forms
pub struct Parser {
    #[allow(dead_code)]
    source: String,
    #[allow(dead_code)]
    position: usize,
}

impl Parser {
    pub fn new(source: String) -> Self {
        Self { source, position: 0 }
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

        Ok(SurfaceForm::List(elements))
    }

    fn parse_string(&mut self) -> Result<SurfaceForm> {
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

        Ok(SurfaceForm::String(value))
    }

    fn parse_number(&mut self) -> Result<SurfaceForm> {
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

        Ok(SurfaceForm::Number(value))
    }

    fn parse_symbol(&mut self) -> Result<SurfaceForm> {
        let start = self.position;

        while !self.is_at_end() && self.is_symbol_char(self.current_char()) {
            self.advance();
        }

        let name = self.source[start..self.position].to_string();
        Ok(SurfaceForm::Symbol(name))
    }

    fn is_symbol_char(&self, ch: char) -> bool {
        !ch.is_whitespace() && ch != '(' && ch != ')' && ch != '"'
    }

    fn current_char(&self) -> char {
        self.source.chars().nth(self.position).unwrap()
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() && self.current_char().is_whitespace() {
            self.advance();
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }
}

/// Surface Forms - parsed S-expressions before macro expansion
#[derive(Debug, Clone)]
pub enum SurfaceForm {
    Symbol(String),
    Number(i64),
    String(String),
    List(Vec<SurfaceForm>),
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
        let form = SurfaceForm::Symbol("test".to_string());
        match form {
            SurfaceForm::Symbol(s) => assert_eq!(s, "test"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_surface_form_number() {
        let form = SurfaceForm::Number(42);
        match form {
            SurfaceForm::Number(n) => assert_eq!(n, 42),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_surface_form_string() {
        let form = SurfaceForm::String("hello".to_string());
        match form {
            SurfaceForm::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_surface_form_list() {
        let form = SurfaceForm::List(vec![]);
        match form {
            SurfaceForm::List(l) => assert_eq!(l.len(), 0),
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
        if let SurfaceForm::List(elements) = &forms[0] {
            assert!(elements.len() >= 3);
            if let SurfaceForm::Symbol(name) = &elements[0] {
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

        if let SurfaceForm::List(elements) = &forms[0] {
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

        if let SurfaceForm::String(s) = &forms[0] {
            assert_eq!(s, "hello");
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

        if let SurfaceForm::Number(n) = &forms[0] {
            assert_eq!(*n, 42);
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

        if let SurfaceForm::Symbol(s) = &forms[0] {
            assert_eq!(s, "println!");
        } else {
            panic!("Expected Symbol");
        }
    }
}
