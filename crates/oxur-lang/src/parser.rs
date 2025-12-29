//! Stage 1: Parse
//!
//! Converts raw Oxur source text into Surface Forms (S-expression AST).
//! Handles tokenization, reader, and reader macros.

use crate::{Result};

/// Parser converts Oxur source text into Surface Forms
pub struct Parser {
    source: String,
    position: usize,
}

impl Parser {
    pub fn new(source: String) -> Self {
        Self {
            source,
            position: 0,
        }
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
}
