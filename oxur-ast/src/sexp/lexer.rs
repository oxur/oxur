use crate::error::{LexError, Position};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    LParen,
    RParen,
    Symbol,
    Keyword,
    String,
    Number,
    Nil,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub typ: TokenType,
    pub lexeme: String,
    pub pos: Position,
}

pub struct Lexer {
    input: Vec<char>,
    current: usize,
    offset: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self { input: input.chars().collect(), current: 0, offset: 0, line: 1, column: 1 }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace_and_comments();

            if self.is_at_end() {
                tokens.push(Token {
                    typ: TokenType::Eof,
                    lexeme: String::new(),
                    pos: self.current_position(),
                });
                break;
            }

            tokens.push(self.next_token()?);
        }

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        let ch = self.current_char().ok_or(LexError::UnexpectedEof)?;
        let pos = self.current_position();

        match ch {
            '(' => {
                self.advance();
                Ok(Token { typ: TokenType::LParen, lexeme: "(".to_string(), pos })
            }
            ')' => {
                self.advance();
                Ok(Token { typ: TokenType::RParen, lexeme: ")".to_string(), pos })
            }
            ':' => self.read_keyword(),
            '"' => self.read_string(),
            _ if ch.is_ascii_digit()
                || (ch == '-' && self.peek().is_some_and(|c| c.is_ascii_digit())) =>
            {
                self.read_number()
            }
            _ if self.is_symbol_start(ch) => self.read_symbol(),
            _ => Err(LexError::UnexpectedChar { ch, pos }),
        }
    }

    fn read_keyword(&mut self) -> Result<Token, LexError> {
        let pos = self.current_position();
        self.advance(); // Skip ':'

        let start = self.current;
        while let Some(ch) = self.current_char() {
            if self.is_symbol_char(ch) {
                self.advance();
            } else {
                break;
            }
        }

        let lexeme: String = self.input[start..self.current].iter().collect();

        Ok(Token { typ: TokenType::Keyword, lexeme, pos })
    }

    fn read_string(&mut self) -> Result<Token, LexError> {
        let pos = self.current_position();
        self.advance(); // Skip opening '"'

        let mut value = String::new();

        loop {
            match self.current_char() {
                None => return Err(LexError::UnterminatedString { pos }),
                Some('"') => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    self.advance();
                    match self.current_char() {
                        None => return Err(LexError::UnterminatedString { pos }),
                        Some('n') => {
                            value.push('\n');
                            self.advance();
                        }
                        Some('t') => {
                            value.push('\t');
                            self.advance();
                        }
                        Some('r') => {
                            value.push('\r');
                            self.advance();
                        }
                        Some('\\') => {
                            value.push('\\');
                            self.advance();
                        }
                        Some('"') => {
                            value.push('"');
                            self.advance();
                        }
                        Some(ch) => {
                            return Err(LexError::InvalidEscape {
                                ch,
                                pos: self.current_position(),
                            });
                        }
                    }
                }
                Some(ch) => {
                    value.push(ch);
                    self.advance();
                }
            }
        }

        Ok(Token { typ: TokenType::String, lexeme: value, pos })
    }

    fn read_number(&mut self) -> Result<Token, LexError> {
        let pos = self.current_position();
        let start = self.current;

        // Handle optional minus sign
        if self.current_char() == Some('-') {
            self.advance();
        }

        // Read digits
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        let lexeme: String = self.input[start..self.current].iter().collect();

        Ok(Token { typ: TokenType::Number, lexeme, pos })
    }

    fn read_symbol(&mut self) -> Result<Token, LexError> {
        let pos = self.current_position();
        let start = self.current;

        while let Some(ch) = self.current_char() {
            if self.is_symbol_char(ch) {
                self.advance();
            } else {
                break;
            }
        }

        let lexeme: String = self.input[start..self.current].iter().collect();

        // Check for 'nil'
        let typ = if lexeme == "nil" { TokenType::Nil } else { TokenType::Symbol };

        Ok(Token { typ, lexeme, pos })
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.current_char() {
                Some(ch) if ch.is_whitespace() => {
                    self.advance();
                }
                Some(';') => {
                    // Skip comment until end of line
                    while let Some(ch) = self.current_char() {
                        self.advance();
                        if ch == '\n' {
                            break;
                        }
                    }
                }
                _ => break,
            }
        }
    }

    fn is_symbol_start(&self, ch: char) -> bool {
        ch.is_alphabetic()
            || ch == '_'
            || ch == '-'
            || ch == '+'
            || ch == '*'
            || ch == '/'
            || ch == '='
            || ch == '<'
            || ch == '>'
            || ch == '!'
            || ch == '?'
            || ch == '&'
    }

    fn is_symbol_char(&self, ch: char) -> bool {
        ch.is_alphanumeric()
            || ch == '_'
            || ch == '-'
            || ch == '+'
            || ch == '*'
            || ch == '/'
            || ch == '='
            || ch == '<'
            || ch == '>'
            || ch == '!'
            || ch == '?'
            || ch == '&'
            || ch == '\''
    }

    fn current_char(&self) -> Option<char> {
        if self.current < self.input.len() {
            Some(self.input[self.current])
        } else {
            None
        }
    }

    fn peek(&self) -> Option<char> {
        if self.current + 1 < self.input.len() {
            Some(self.input[self.current + 1])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        if let Some(ch) = self.current_char() {
            self.current += 1;
            self.offset += ch.len_utf8();

            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.input.len()
    }

    fn current_position(&self) -> Position {
        Position::new(self.offset, self.line, self.column)
    }
}
