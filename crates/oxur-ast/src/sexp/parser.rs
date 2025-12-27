use crate::error::{ParseError, Result};
use crate::sexp::lexer::{Lexer, Token, TokenType};
use crate::sexp::types::*;

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
