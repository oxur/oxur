---
number: 4
title: "oxur-ast Phase 0: S-Expression Infrastructure"
author: "Duncan McGreggor"
component: AST
tags: [Phase-0, Infrastructure]
created: 2025-12-27
updated: 2025-12-27
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# oxur-ast Phase 0: S-Expression Infrastructure

**Phase**: 0 - Foundation  
**Goal**: Build the S-expression lexer, parser, and AST  
**Estimated Time**: 3-5 days  
**Prerequisites**: Workspace setup complete with `design` crate

---

## Overview

This phase builds the foundational S-expression infrastructure needed for `oxur-ast`. Before we can convert between Rust AST and S-expressions, we need solid tools for working with S-expressions themselves.

**What we're building:**
1. S-expression lexer (text → tokens)
2. S-expression parser (tokens → generic AST)
3. S-expression types (the AST structure)
4. S-expression printer (AST → formatted text)

**Why this comes first:**
- Self-contained and testable in isolation
- No Rust AST dependencies yet (simpler)
- Establishes patterns for later phases
- Can be thoroughly tested before moving on

---

## File Structure

Create the `oxur-ast` crate with this structure:

```
oxur-ast/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API exports
│   ├── sexp/
│   │   ├── mod.rs          # S-expression module exports
│   │   ├── types.rs        # SExp type definitions
│   │   ├── lexer.rs        # Tokenization
│   │   ├── parser.rs       # Token stream → SExp AST
│   │   └── printer.rs      # SExp AST → formatted text
│   └── error.rs            # Error types
├── tests/
│   ├── lexer_tests.rs
│   ├── parser_tests.rs
│   ├── printer_tests.rs
│   └── round_trip_tests.rs
└── examples/
    └── parse_example.rs     # Example usage
```

---

## Part 1: Error Types

### File: `src/error.rs`

Create comprehensive error types for the entire crate:

```rust
use std::fmt;

/// Position in source text
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub offset: usize,  // Byte offset
    pub line: usize,    // Line number (1-based)
    pub column: usize,  // Column number (1-based)
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
```

**Key design points:**
- `Position` tracks location for error reporting
- Separate error types for lexer vs parser
- `thiserror` for clean error messages
- Convert `LexError` → `ParseError` automatically

---

## Part 2: S-Expression Types

### File: `src/sexp/types.rs`

Define the S-expression AST:

```rust
use crate::error::Position;

/// An S-expression value
#[derive(Debug, Clone, PartialEq)]
pub enum SExp {
    /// Symbol: foo, bar, ExprKind, etc.
    Symbol(Symbol),
    
    /// Keyword: :name, :kind, :span
    Keyword(Keyword),
    
    /// String: "hello", "main"
    String(StringLit),
    
    /// Number: 42, 0, 123
    Number(Number),
    
    /// Nil: nil
    Nil(Nil),
    
    /// List: (foo bar baz)
    List(List),
}

/// Symbol (unquoted identifier)
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub value: String,
    pub pos: Position,
}

/// Keyword (starts with :)
#[derive(Debug, Clone, PartialEq)]
pub struct Keyword {
    pub name: String,  // Without the ':'
    pub pos: Position,
}

/// String literal
#[derive(Debug, Clone, PartialEq)]
pub struct StringLit {
    pub value: String,  // Unescaped value
    pub pos: Position,
}

/// Number
#[derive(Debug, Clone, PartialEq)]
pub struct Number {
    pub value: String,  // Keep as string for now, parse later if needed
    pub pos: Position,
}

/// Nil value
#[derive(Debug, Clone, PartialEq)]
pub struct Nil {
    pub pos: Position,
}

/// List of S-expressions
#[derive(Debug, Clone, PartialEq)]
pub struct List {
    pub elements: Vec<SExp>,
    pub pos: Position,  // Position of opening paren
}

// Convenience constructors
impl Symbol {
    pub fn new(value: impl Into<String>, pos: Position) -> Self {
        Self { value: value.into(), pos }
    }
}

impl Keyword {
    pub fn new(name: impl Into<String>, pos: Position) -> Self {
        Self { name: name.into(), pos }
    }
}

impl StringLit {
    pub fn new(value: impl Into<String>, pos: Position) -> Self {
        Self { value: value.into(), pos }
    }
}

impl Number {
    pub fn new(value: impl Into<String>, pos: Position) -> Self {
        Self { value: value.into(), pos }
    }
}

impl Nil {
    pub fn new(pos: Position) -> Self {
        Self { pos }
    }
}

impl List {
    pub fn new(elements: Vec<SExp>, pos: Position) -> Self {
        Self { elements, pos }
    }
}

// Position accessor trait
pub trait HasPosition {
    fn position(&self) -> Position;
}

impl HasPosition for SExp {
    fn position(&self) -> Position {
        match self {
            SExp::Symbol(s) => s.pos,
            SExp::Keyword(k) => k.pos,
            SExp::String(s) => s.pos,
            SExp::Number(n) => n.pos,
            SExp::Nil(n) => n.pos,
            SExp::List(l) => l.pos,
        }
    }
}
```

**Design rationale:**
- Each variant has its own struct with position
- `HasPosition` trait for uniform access
- Keep `Number` as string for flexibility
- `List` is the recursive structure
- Position preserved for error reporting

---

## Part 3: Lexer

### File: `src/sexp/lexer.rs`

Tokenize S-expression text:

```rust
use crate::error::{LexError, Position};

/// Token types
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    LParen,      // (
    RParen,      // )
    Symbol,      // foo, bar, ExprKind
    Keyword,     // :name, :kind
    String,      // "hello"
    Number,      // 42, 0, 123
    Nil,         // nil
    Eof,
}

/// A single token
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub typ: TokenType,
    pub lexeme: String,
    pub pos: Position,
}

/// Lexer state
pub struct Lexer {
    input: Vec<char>,
    position: usize,      // Current position
    line: usize,          // Current line (1-based)
    column: usize,        // Current column (1-based)
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
    /// Get all tokens
    pub fn tokenize(mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token()?;
            let is_eof = token.typ == TokenType::Eof;
            tokens.push(token);
            
            if is_eof {
                break;
            }
        }
        
        Ok(tokens)
    }
    
    /// Get the next token
    fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace_and_comments();
        
        if self.is_at_end() {
            return Ok(self.make_token(TokenType::Eof, ""));
        }
        
        let start_pos = self.current_position();
        let ch = self.current_char();
        
        match ch {
            '(' => {
                self.advance();
                Ok(self.make_token(TokenType::LParen, "("))
            }
            ')' => {
                self.advance();
                Ok(self.make_token(TokenType::RParen, ")"))
            }
            ':' => self.read_keyword(),
            '"' => self.read_string(),
            _ if ch.is_ascii_digit() || (ch == '-' && self.peek().map_or(false, |c| c.is_ascii_digit())) => {
                self.read_number()
            }
            _ if self.is_symbol_start(ch) => self.read_symbol(),
            _ => Err(LexError::UnexpectedChar { 
                ch, 
                pos: start_pos 
            }),
        }
    }
    
    fn skip_whitespace_and_comments(&mut self) {
        while !self.is_at_end() {
            match self.current_char() {
                ' ' | '\t' | '\r' | '\n' => {
                    self.advance();
                }
                ';' => {
                    // Skip until end of line
                    while !self.is_at_end() && self.current_char() != '\n' {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }
    
    fn read_keyword(&mut self) -> Result<Token, LexError> {
        let start_pos = self.current_position();
        self.advance(); // Skip ':'
        
        let mut name = String::new();
        while !self.is_at_end() && self.is_symbol_char(self.current_char()) {
            name.push(self.current_char());
            self.advance();
        }
        
        Ok(Token {
            typ: TokenType::Keyword,
            lexeme: name,
            pos: start_pos,
        })
    }
    
    fn read_string(&mut self) -> Result<Token, LexError> {
        let start_pos = self.current_position();
        self.advance(); // Skip opening "
        
        let mut value = String::new();
        
        while !self.is_at_end() && self.current_char() != '"' {
            if self.current_char() == '\\' {
                self.advance();
                if self.is_at_end() {
                    return Err(LexError::UnterminatedString { pos: start_pos });
                }
                
                let escaped = match self.current_char() {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '"' => '"',
                    ch => return Err(LexError::InvalidEscape { 
                        ch, 
                        pos: self.current_position() 
                    }),
                };
                
                value.push(escaped);
                self.advance();
            } else {
                value.push(self.current_char());
                self.advance();
            }
        }
        
        if self.is_at_end() {
            return Err(LexError::UnterminatedString { pos: start_pos });
        }
        
        self.advance(); // Skip closing "
        
        Ok(Token {
            typ: TokenType::String,
            lexeme: value,
            pos: start_pos,
        })
    }
    
    fn read_number(&mut self) -> Result<Token, LexError> {
        let start_pos = self.current_position();
        let mut num = String::new();
        
        // Optional leading minus
        if self.current_char() == '-' {
            num.push('-');
            self.advance();
        }
        
        // Integer part
        while !self.is_at_end() && self.current_char().is_ascii_digit() {
            num.push(self.current_char());
            self.advance();
        }
        
        Ok(Token {
            typ: TokenType::Number,
            lexeme: num,
            pos: start_pos,
        })
    }
    
    fn read_symbol(&mut self) -> Result<Token, LexError> {
        let start_pos = self.current_position();
        let mut sym = String::new();
        
        while !self.is_at_end() && self.is_symbol_char(self.current_char()) {
            sym.push(self.current_char());
            self.advance();
        }
        
        // Check for special symbol "nil"
        let typ = if sym == "nil" {
            TokenType::Nil
        } else {
            TokenType::Symbol
        };
        
        Ok(Token {
            typ,
            lexeme: sym,
            pos: start_pos,
        })
    }
    
    // Helper methods
    
    fn is_symbol_start(&self, ch: char) -> bool {
        ch.is_alphabetic() || ch == '_' || ch == '-' || ch == '+' || ch == '*' 
            || ch == '/' || ch == '<' || ch == '>' || ch == '=' || ch == '!' || ch == '?'
    }
    
    fn is_symbol_char(&self, ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '+' || ch == '*' 
            || ch == '/' || ch == '<' || ch == '>' || ch == '=' || ch == '!' || ch == '?'
    }
    
    fn current_char(&self) -> char {
        self.input[self.position]
    }
    
    fn peek(&self) -> Option<char> {
        if self.position + 1 < self.input.len() {
            Some(self.input[self.position + 1])
        } else {
            None
        }
    }
    
    fn advance(&mut self) {
        if self.is_at_end() {
            return;
        }
        
        if self.current_char() == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        
        self.position += 1;
    }
    
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
    
    fn current_position(&self) -> Position {
        Position::new(self.position, self.line, self.column)
    }
    
    fn make_token(&self, typ: TokenType, lexeme: impl Into<String>) -> Token {
        Token {
            typ,
            lexeme: lexeme.into(),
            pos: self.current_position(),
        }
    }
}
```

**Key features:**
- Tracks line/column for errors
- Handles escape sequences in strings
- Comments (semicolon to end of line)
- Special handling for `nil` symbol
- Clean error reporting with positions

---

## Part 4: Parser

### File: `src/sexp/parser.rs`

Parse tokens into S-expression AST:

```rust
use crate::error::{ParseError, Position, Result};
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
    
    /// Parse from source text
    pub fn parse_str(input: &str) -> Result<SExp> {
        let lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }
    
    /// Parse the token stream
    pub fn parse(&mut self) -> Result<SExp> {
        if self.is_at_end() {
            return Err(ParseError::EmptyInput);
        }
        
        self.parse_sexp()
    }
    
    fn parse_sexp(&mut self) -> Result<SExp> {
        let token = self.current_token();
        
        match token.typ {
            TokenType::LParen => self.parse_list(),
            TokenType::Symbol => self.parse_symbol(),
            TokenType::Keyword => self.parse_keyword(),
            TokenType::String => self.parse_string(),
            TokenType::Number => self.parse_number(),
            TokenType::Nil => self.parse_nil(),
            TokenType::RParen => Err(ParseError::UnexpectedCloseParen { 
                pos: token.pos 
            }),
            TokenType::Eof => Err(ParseError::Expected {
                expected: "S-expression".to_string(),
                found: "end of input".to_string(),
                pos: token.pos,
            }),
        }
    }
    
    fn parse_list(&mut self) -> Result<SExp> {
        let start_pos = self.current_token().pos;
        self.advance(); // Skip '('
        
        let mut elements = Vec::new();
        
        while !self.check(&TokenType::RParen) {
            if self.is_at_end() {
                return Err(ParseError::UnterminatedList { pos: start_pos });
            }
            
            elements.push(self.parse_sexp()?);
        }
        
        self.advance(); // Skip ')'
        
        Ok(SExp::List(List::new(elements, start_pos)))
    }
    
    fn parse_symbol(&mut self) -> Result<SExp> {
        let token = self.current_token();
        self.advance();
        Ok(SExp::Symbol(Symbol::new(token.lexeme, token.pos)))
    }
    
    fn parse_keyword(&mut self) -> Result<SExp> {
        let token = self.current_token();
        self.advance();
        Ok(SExp::Keyword(Keyword::new(token.lexeme, token.pos)))
    }
    
    fn parse_string(&mut self) -> Result<SExp> {
        let token = self.current_token();
        self.advance();
        Ok(SExp::String(StringLit::new(token.lexeme, token.pos)))
    }
    
    fn parse_number(&mut self) -> Result<SExp> {
        let token = self.current_token();
        self.advance();
        Ok(SExp::Number(Number::new(token.lexeme, token.pos)))
    }
    
    fn parse_nil(&mut self) -> Result<SExp> {
        let token = self.current_token();
        self.advance();
        Ok(SExp::Nil(Nil::new(token.pos)))
    }
    
    // Helper methods
    
    fn current_token(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn check(&self, typ: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        &self.current_token().typ == typ
    }
    
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() 
            || self.current_token().typ == TokenType::Eof
    }
}
```

**Design notes:**
- Recursive descent parser
- Clean error reporting
- Convenience `parse_str` method
- Position preserved from tokens

---

## Part 5: Printer

### File: `src/sexp/printer.rs`

Format S-expressions back to text:

```rust
use crate::sexp::types::*;
use std::fmt::Write;

pub struct Printer {
    indent: usize,
    indent_str: String,
}

impl Printer {
    pub fn new() -> Self {
        Self {
            indent: 0,
            indent_str: "  ".to_string(), // 2 spaces
        }
    }
    
    pub fn with_indent(indent_str: impl Into<String>) -> Self {
        Self {
            indent: 0,
            indent_str: indent_str.into(),
        }
    }
    
    /// Print S-expression to string
    pub fn print(&mut self, sexp: &SExp) -> String {
        let mut output = String::new();
        self.print_sexp(sexp, &mut output);
        output
    }
    
    fn print_sexp(&mut self, sexp: &SExp, output: &mut String) {
        match sexp {
            SExp::Symbol(s) => write!(output, "{}", s.value).unwrap(),
            SExp::Keyword(k) => write!(output, ":{}", k.name).unwrap(),
            SExp::String(s) => {
                write!(output, "\"{}\"", escape_string(&s.value)).unwrap()
            }
            SExp::Number(n) => write!(output, "{}", n.value).unwrap(),
            SExp::Nil(_) => write!(output, "nil").unwrap(),
            SExp::List(l) => self.print_list(l, output),
        }
    }
    
    fn print_list(&mut self, list: &List, output: &mut String) {
        if list.elements.is_empty() {
            write!(output, "()").unwrap();
            return;
        }
        
        // Check if this is a "simple" list (no nested lists, short)
        let is_simple = list.elements.len() <= 3 
            && list.elements.iter().all(|e| !matches!(e, SExp::List(_)));
        
        if is_simple {
            // Print on one line
            write!(output, "(").unwrap();
            for (i, elem) in list.elements.iter().enumerate() {
                if i > 0 {
                    write!(output, " ").unwrap();
                }
                self.print_sexp(elem, output);
            }
            write!(output, ")").unwrap();
        } else {
            // Print with indentation
            write!(output, "(").unwrap();
            self.indent += 1;
            
            for (i, elem) in list.elements.iter().enumerate() {
                if i > 0 {
                    write!(output, "\n{}", self.current_indent()).unwrap();
                }
                self.print_sexp(elem, output);
            }
            
            self.indent -= 1;
            write!(output, ")").unwrap();
        }
    }
    
    fn current_indent(&self) -> String {
        self.indent_str.repeat(self.indent)
    }
}

impl Default for Printer {
    fn default() -> Self {
        Self::new()
    }
}

/// Escape string for printing
fn escape_string(s: &str) -> String {
    s.chars()
        .flat_map(|ch| match ch {
            '\n' => vec!['\\', 'n'],
            '\t' => vec!['\\', 't'],
            '\r' => vec!['\\', 'r'],
            '\\' => vec!['\\', '\\'],
            '"' => vec!['\\', '"'],
            ch => vec![ch],
        })
        .collect()
}

/// Convenience function for quick printing
pub fn print_sexp(sexp: &SExp) -> String {
    Printer::new().print(sexp)
}
```

**Features:**
- Smart indentation for nested lists
- Configurable indent string
- Escape sequences handled
- Simple lists on one line

---

## Part 6: Module Exports

### File: `src/sexp/mod.rs`

```rust
pub mod types;
pub mod lexer;
pub mod parser;
pub mod printer;

pub use types::*;
pub use parser::Parser;
pub use printer::{Printer, print_sexp};
```

### File: `src/lib.rs`

```rust
pub mod error;
pub mod sexp;

// Re-export commonly used items
pub use error::{ParseError, LexError, Position, Result};
pub use sexp::{SExp, Parser, Printer, print_sexp};
```

---

## Part 7: Tests

### File: `tests/lexer_tests.rs`

```rust
use oxur_ast::sexp::lexer::{Lexer, TokenType};

#[test]
fn test_empty() {
    let tokens = Lexer::new("").tokenize().unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].typ, TokenType::Eof);
}

#[test]
fn test_parens() {
    let tokens = Lexer::new("()").tokenize().unwrap();
    assert_eq!(tokens[0].typ, TokenType::LParen);
    assert_eq!(tokens[1].typ, TokenType::RParen);
    assert_eq!(tokens[2].typ, TokenType::Eof);
}

#[test]
fn test_symbols() {
    let tokens = Lexer::new("foo bar-baz").tokenize().unwrap();
    assert_eq!(tokens[0].typ, TokenType::Symbol);
    assert_eq!(tokens[0].lexeme, "foo");
    assert_eq!(tokens[1].typ, TokenType::Symbol);
    assert_eq!(tokens[1].lexeme, "bar-baz");
}

#[test]
fn test_keywords() {
    let tokens = Lexer::new(":name :kind").tokenize().unwrap();
    assert_eq!(tokens[0].typ, TokenType::Keyword);
    assert_eq!(tokens[0].lexeme, "name");
    assert_eq!(tokens[1].typ, TokenType::Keyword);
    assert_eq!(tokens[1].lexeme, "kind");
}

#[test]
fn test_strings() {
    let tokens = Lexer::new(r#""hello" "world\n""#).tokenize().unwrap();
    assert_eq!(tokens[0].typ, TokenType::String);
    assert_eq!(tokens[0].lexeme, "hello");
    assert_eq!(tokens[1].typ, TokenType::String);
    assert_eq!(tokens[1].lexeme, "world\n");
}

#[test]
fn test_numbers() {
    let tokens = Lexer::new("42 0 -10").tokenize().unwrap();
    assert_eq!(tokens[0].typ, TokenType::Number);
    assert_eq!(tokens[0].lexeme, "42");
    assert_eq!(tokens[1].typ, TokenType::Number);
    assert_eq!(tokens[1].lexeme, "0");
    assert_eq!(tokens[2].typ, TokenType::Number);
    assert_eq!(tokens[2].lexeme, "-10");
}

#[test]
fn test_nil() {
    let tokens = Lexer::new("nil").tokenize().unwrap();
    assert_eq!(tokens[0].typ, TokenType::Nil);
}

#[test]
fn test_comments() {
    let tokens = Lexer::new("; comment\nfoo ; another\nbar").tokenize().unwrap();
    assert_eq!(tokens[0].typ, TokenType::Symbol);
    assert_eq!(tokens[0].lexeme, "foo");
    assert_eq!(tokens[1].typ, TokenType::Symbol);
    assert_eq!(tokens[1].lexeme, "bar");
}

#[test]
fn test_complex() {
    let input = r#"(Expr :id 42 :kind (Binary :op Add))"#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    assert_eq!(tokens[0].typ, TokenType::LParen);
    assert_eq!(tokens[1].typ, TokenType::Symbol);
    assert_eq!(tokens[2].typ, TokenType::Keyword);
    // ... etc
}

#[test]
fn test_unterminated_string() {
    let result = Lexer::new(r#""hello"#).tokenize();
    assert!(result.is_err());
}
```

### File: `tests/parser_tests.rs`

```rust
use oxur_ast::sexp::{Parser, SExp};

#[test]
fn test_parse_symbol() {
    let sexp = Parser::parse_str("foo").unwrap();
    match sexp {
        SExp::Symbol(s) => assert_eq!(s.value, "foo"),
        _ => panic!("Expected Symbol"),
    }
}

#[test]
fn test_parse_keyword() {
    let sexp = Parser::parse_str(":name").unwrap();
    match sexp {
        SExp::Keyword(k) => assert_eq!(k.name, "name"),
        _ => panic!("Expected Keyword"),
    }
}

#[test]
fn test_parse_string() {
    let sexp = Parser::parse_str(r#""hello""#).unwrap();
    match sexp {
        SExp::String(s) => assert_eq!(s.value, "hello"),
        _ => panic!("Expected String"),
    }
}

#[test]
fn test_parse_number() {
    let sexp = Parser::parse_str("42").unwrap();
    match sexp {
        SExp::Number(n) => assert_eq!(n.value, "42"),
        _ => panic!("Expected Number"),
    }
}

#[test]
fn test_parse_nil() {
    let sexp = Parser::parse_str("nil").unwrap();
    assert!(matches!(sexp, SExp::Nil(_)));
}

#[test]
fn test_parse_empty_list() {
    let sexp = Parser::parse_str("()").unwrap();
    match sexp {
        SExp::List(l) => assert!(l.elements.is_empty()),
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_simple_list() {
    let sexp = Parser::parse_str("(foo bar)").unwrap();
    match sexp {
        SExp::List(l) => {
            assert_eq!(l.elements.len(), 2);
            match &l.elements[0] {
                SExp::Symbol(s) => assert_eq!(s.value, "foo"),
                _ => panic!("Expected Symbol"),
            }
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_nested_list() {
    let sexp = Parser::parse_str("(foo (bar baz))").unwrap();
    match sexp {
        SExp::List(l) => {
            assert_eq!(l.elements.len(), 2);
            match &l.elements[1] {
                SExp::List(inner) => assert_eq!(inner.elements.len(), 2),
                _ => panic!("Expected nested List"),
            }
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_keyword_list() {
    let sexp = Parser::parse_str("(Expr :id 42 :name \"foo\")").unwrap();
    match sexp {
        SExp::List(l) => {
            assert_eq!(l.elements.len(), 5);
            match &l.elements[1] {
                SExp::Keyword(k) => assert_eq!(k.name, "id"),
                _ => panic!("Expected Keyword"),
            }
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_error_unterminated() {
    let result = Parser::parse_str("(foo");
    assert!(result.is_err());
}

#[test]
fn test_parse_error_unexpected_close() {
    let result = Parser::parse_str(")");
    assert!(result.is_err());
}
```

### File: `tests/printer_tests.rs`

```rust
use oxur_ast::sexp::{Parser, print_sexp};

#[test]
fn test_print_symbol() {
    let sexp = Parser::parse_str("foo").unwrap();
    assert_eq!(print_sexp(&sexp), "foo");
}

#[test]
fn test_print_keyword() {
    let sexp = Parser::parse_str(":name").unwrap();
    assert_eq!(print_sexp(&sexp), ":name");
}

#[test]
fn test_print_string() {
    let sexp = Parser::parse_str(r#""hello""#).unwrap();
    assert_eq!(print_sexp(&sexp), r#""hello""#);
}

#[test]
fn test_print_string_with_escapes() {
    let sexp = Parser::parse_str(r#""hello\nworld""#).unwrap();
    assert_eq!(print_sexp(&sexp), r#""hello\nworld""#);
}

#[test]
fn test_print_number() {
    let sexp = Parser::parse_str("42").unwrap();
    assert_eq!(print_sexp(&sexp), "42");
}

#[test]
fn test_print_nil() {
    let sexp = Parser::parse_str("nil").unwrap();
    assert_eq!(print_sexp(&sexp), "nil");
}

#[test]
fn test_print_simple_list() {
    let sexp = Parser::parse_str("(foo bar)").unwrap();
    assert_eq!(print_sexp(&sexp), "(foo bar)");
}

#[test]
fn test_print_keyword_list() {
    let sexp = Parser::parse_str("(Expr :id 42)").unwrap();
    let output = print_sexp(&sexp);
    // May have formatting differences, check contains
    assert!(output.contains("Expr"));
    assert!(output.contains(":id"));
    assert!(output.contains("42"));
}
```

### File: `tests/round_trip_tests.rs`

```rust
use oxur_ast::sexp::{Parser, print_sexp};

fn round_trip(input: &str) {
    let parsed = Parser::parse_str(input).unwrap();
    let printed = print_sexp(&parsed);
    let reparsed = Parser::parse_str(&printed).unwrap();
    assert_eq!(parsed, reparsed, "Round trip failed for: {}", input);
}

#[test]
fn test_round_trip_simple() {
    round_trip("foo");
    round_trip(":name");
    round_trip(r#""hello""#);
    round_trip("42");
    round_trip("nil");
}

#[test]
fn test_round_trip_lists() {
    round_trip("()");
    round_trip("(foo)");
    round_trip("(foo bar)");
    round_trip("(foo (bar baz))");
}

#[test]
fn test_round_trip_complex() {
    round_trip("(Expr :id 42 :kind (Binary :op Add))");
    round_trip(r#"(Item :name "main" :vis (Inherited))"#);
}
```

---

## Part 8: Cargo.toml

### File: `Cargo.toml`

```toml
[package]
name = "oxur-ast"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Rust AST ↔ S-expression conversion for Oxur"

[dependencies]
thiserror.workspace = true

[dev-dependencies]
# Add if needed for examples
```

Update workspace `Cargo.toml`:

```toml
[workspace]
members = [
    "design",
    "oxur-ast",  # Add this
]
```

---

## Part 9: Example

### File: `examples/parse_example.rs`

```rust
use oxur_ast::sexp::Parser;
use oxur_ast::sexp::print_sexp;

fn main() {
    let input = r#"
(Crate
  :items (
    (Item
      :ident (Ident :name "main")
      :kind (Fn
              :body (Block
                      :stmts ((Stmt :kind (Expr ...))))))))
    "#;
    
    match Parser::parse_str(input) {
        Ok(sexp) => {
            println!("Parsed successfully!");
            println!("\n{}", print_sexp(&sexp));
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }
}
```

---

## Success Criteria

Phase 0 is complete when:

- [ ] All files created with correct structure
- [ ] Lexer tokenizes all S-expression types
- [ ] Parser builds correct AST from tokens
- [ ] Printer formats S-expressions readably
- [ ] All tests pass
- [ ] Round-trip tests verify correctness
- [ ] Example runs successfully
- [ ] Documentation comments added
- [ ] Code formatted with `cargo fmt`
- [ ] No warnings from `cargo clippy`

---

## Testing Instructions

```bash
# Run all tests
cargo test -p oxur-ast

# Run specific test file
cargo test -p oxur-ast --test lexer_tests

# Run with output
cargo test -p oxur-ast -- --nocapture

# Run example
cargo run -p oxur-ast --example parse_example

# Check formatting
cargo fmt --check -p oxur-ast

# Run clippy
cargo clippy -p oxur-ast -- -D warnings
```

---

## Notes for Claude Code

**Implementation order:**
1. Start with `error.rs` (foundation)
2. Then `sexp/types.rs` (data structures)
3. Then `sexp/lexer.rs` (tokenization)
4. Then `sexp/parser.rs` (parsing)
5. Then `sexp/printer.rs` (output)
6. Then module files (`mod.rs`, `lib.rs`)
7. Finally tests and examples

**Testing strategy:**
- Write tests as you implement each component
- Run tests frequently
- Use `--nocapture` to see debug output
- Add more edge case tests as you discover them

**Common pitfalls:**
- Don't forget to update workspace `Cargo.toml`
- Position tracking must be accurate for good errors
- Escape sequences must round-trip correctly
- Empty lists are valid and different from nil

**Next phase:**
Once Phase 0 is complete and all tests pass, we'll move to Phase 1: building the Rust AST types and the AST builder that converts S-expressions to Rust AST nodes.

---

*"The foundation is the S-expression. Everything else builds on this."*
