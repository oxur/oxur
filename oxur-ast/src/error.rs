use std::fmt;

/// Position in source text
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub offset: usize, // Byte offset
    pub line: usize,   // Line number (1-based)
    pub column: usize, // Column number (1-based)
}

impl Position {
    pub fn new(offset: usize, line: usize, column: usize) -> Self {
        Self { offset, line, column }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

/// Lexer errors
#[derive(Debug, thiserror::Error)]
pub enum LexError {
    #[error("Unexpected character '{ch}' at {pos}")]
    UnexpectedChar { ch: char, pos: Position },

    #[error("Unterminated string at {pos}")]
    UnterminatedString { pos: Position },

    #[error("Invalid escape sequence '\\{ch}' at {pos}")]
    InvalidEscape { ch: char, pos: Position },

    #[error("Unexpected end of input")]
    UnexpectedEof,
}

/// Parser errors
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Unexpected token {token:?} at {pos}")]
    UnexpectedToken { token: String, pos: Position },

    #[error("Expected {expected}, found {found} at {pos}")]
    Expected { expected: String, found: String, pos: Position },

    #[error("Unterminated list at {pos}")]
    UnterminatedList { pos: Position },

    #[error("Unexpected closing parenthesis at {pos}")]
    UnexpectedCloseParen { pos: Position },

    #[error("Empty input")]
    EmptyInput,

    #[error("Lexer error: {0}")]
    LexError(#[from] LexError),
}

pub type Result<T> = std::result::Result<T, ParseError>;
