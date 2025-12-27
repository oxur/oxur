use crate::error::{ParseError, Result};
use crate::sexp::lexer::{Lexer, Token, TokenType};
use crate::sexp::types::*;
use std::path::Path;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse_str(input: &str) -> Result<SExp> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    /// Parse an S-expression from a file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use oxur_ast::sexp::Parser;
    ///
    /// // Parse from a file path
    /// let sexp = Parser::parse_file("example.sexp")?;
    /// # Ok::<(), oxur_ast::ParseError>(())
    /// ```
    ///
    /// ```no_run
    /// use oxur_ast::sexp::Parser;
    /// use std::path::PathBuf;
    ///
    /// // Parse from a PathBuf
    /// let path = PathBuf::from("data/crate.sexp");
    /// let sexp = Parser::parse_file(&path)?;
    /// # Ok::<(), oxur_ast::ParseError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ParseError::FileReadError` if the file cannot be read.
    /// Returns other `ParseError` variants if the content is invalid.
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<SExp> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            ParseError::FileReadError {
                path: path.as_ref().to_path_buf(),
                source: e,
            }
        })?;
        Self::parse_str(&content)
    }

    pub fn parse(&mut self) -> Result<SExp> {
        if self.is_at_end() {
            return Err(ParseError::EmptyInput);
        }

        self.parse_sexp()
    }

    fn parse_sexp(&mut self) -> Result<SExp> {
        let token = self.current_token()?;

        match token.typ {
            TokenType::LParen => self.parse_list(),
            TokenType::Symbol => self.parse_symbol(),
            TokenType::Keyword => self.parse_keyword(),
            TokenType::String => self.parse_string(),
            TokenType::Number => self.parse_number(),
            TokenType::Nil => self.parse_nil(),
            TokenType::RParen => Err(ParseError::UnexpectedCloseParen { pos: token.pos }),
            TokenType::Eof => Err(ParseError::EmptyInput),
        }
    }

    fn parse_list(&mut self) -> Result<SExp> {
        let open_paren = self.current_token()?.clone();
        let pos = open_paren.pos;

        self.advance(); // Skip '('

        let mut elements = Vec::new();

        loop {
            if self.is_at_end() {
                return Err(ParseError::UnterminatedList { pos });
            }

            if self.check(&TokenType::RParen) {
                self.advance(); // Skip ')'
                break;
            }

            elements.push(self.parse_sexp()?);
        }

        Ok(SExp::List(List::new(elements, pos)))
    }

    fn parse_symbol(&mut self) -> Result<SExp> {
        let token = self.current_token()?.clone();
        self.advance();

        Ok(SExp::Symbol(Symbol::new(token.lexeme, token.pos)))
    }

    fn parse_keyword(&mut self) -> Result<SExp> {
        let token = self.current_token()?.clone();
        self.advance();

        Ok(SExp::Keyword(Keyword::new(token.lexeme, token.pos)))
    }

    fn parse_string(&mut self) -> Result<SExp> {
        let token = self.current_token()?.clone();
        self.advance();

        Ok(SExp::String(StringLit::new(token.lexeme, token.pos)))
    }

    fn parse_number(&mut self) -> Result<SExp> {
        let token = self.current_token()?.clone();
        self.advance();

        Ok(SExp::Number(Number::new(token.lexeme, token.pos)))
    }

    fn parse_nil(&mut self) -> Result<SExp> {
        let token = self.current_token()?.clone();
        self.advance();

        Ok(SExp::Nil(Nil::new(token.pos)))
    }

    fn current_token(&self) -> Result<&Token> {
        self.tokens.get(self.current).ok_or(ParseError::EmptyInput)
    }

    fn check(&self, typ: &TokenType) -> bool {
        if let Ok(token) = self.current_token() {
            &token.typ == typ
        } else {
            false
        }
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current_token(), Ok(token) if token.typ == TokenType::Eof)
            || self.current >= self.tokens.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_parse_file_valid() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "(foo bar baz)").unwrap();

        let result = Parser::parse_file(temp_file.path());
        assert!(result.is_ok());

        let sexp = result.unwrap();
        match sexp {
            SExp::List(list) => assert_eq!(list.elements.len(), 3),
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_parse_file_nonexistent() {
        let result = Parser::parse_file("/nonexistent/path/to/file.sexp");
        assert!(result.is_err());

        match result.unwrap_err() {
            ParseError::FileReadError { path, .. } => {
                assert!(path.to_string_lossy().contains("nonexistent"));
            }
            _ => panic!("Expected FileReadError"),
        }
    }

    #[test]
    fn test_parse_file_empty() {
        let temp_file = NamedTempFile::new().unwrap();
        // Empty file

        let result = Parser::parse_file(temp_file.path());
        assert!(result.is_err());

        match result.unwrap_err() {
            ParseError::EmptyInput => {}
            e => panic!("Expected EmptyInput, got {:?}", e),
        }
    }

    #[test]
    fn test_parse_file_complex_sexp() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "(Crate\n  :items ()\n  :span (Span :lo 0 :hi 0))").unwrap();

        let result = Parser::parse_file(temp_file.path());
        assert!(result.is_ok());
    }
}
