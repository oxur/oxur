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
        // Placeholder implementation
        Ok(vec![])
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
}
