---
number: 0008
title: "Claude Code Implementation Guide: oxur-ast Phases 0-1"
author: Duncan McGreggor & Claude
created: 2025-12-25
updated: 2025-12-25
state: Draft
supersedes: None
superseded-by: None
---

# Claude Code Implementation Guide: oxur-ast Phases 0-1

**Target:** Claude Code (AI coding assistant)  
**Goal:** Implement oxur-ast Phases 0 and 1 from design specifications  
**Estimated Time:** 2-3 work sessions  
**Prerequisites:** Design documents 002 (spec), 003 (Phase 0), 004 (Phase 1)

---

## Overview for Claude Code

You are implementing **oxur-ast**, a library that provides bidirectional conversion between Rust's AST and S-expressions. This is the foundation for Oxur, a Lisp dialect that compiles to Rust.

**What you're building:**
- Phase 0: S-expression lexer, parser, printer, and AST
- Phase 1: Rust AST types and builder (S-expr → Rust AST)

**End goal:** Parse S-expressions and build Rust AST nodes from them, specifically enough to handle "Hello World":

```rust
fn main() {
    println!("Hello, world!");
}
```

---

## Session Structure

This guide is split into discrete sessions. Complete each session before moving to the next:

- **Session 1:** Workspace setup + Phase 0 foundation
- **Session 2:** Phase 0 completion + Phase 1 start
- **Session 3:** Phase 1 completion + testing

---

# Session 1: Workspace Setup + Phase 0 Foundation

## Step 1: Create Workspace Structure

Create a new Rust workspace:

```bash
mkdir -p oxur
cd oxur
```

Create workspace `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "design",
    "oxur-ast",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Duncan McGreggor <duncan@cogitat.io>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/oxur/oxur-ast"

[workspace.dependencies]
thiserror = "1.0"
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"
syn = { version = "2.0", features = ["full", "parsing", "extra-traits"] }
quote = "1.0"
criterion = "0.5"
```

## Step 2: Create oxur-ast Crate

```bash
cargo new --lib oxur-ast
cd oxur-ast
```

Update `oxur-ast/Cargo.toml`:

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
# Will add test dependencies later
```

## Step 3: Create Directory Structure

Create all directories needed for Phase 0:

```bash
cd oxur-ast
mkdir -p src/sexp
mkdir -p src/ast
mkdir -p src/builder
mkdir -p tests
mkdir -p examples
mkdir -p benches
```

Your structure should look like:
```
oxur-ast/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── error.rs          # (to create)
│   ├── sexp/
│   │   └── mod.rs        # (to create)
│   ├── ast/
│   │   └── mod.rs        # (placeholder)
│   └── builder/
│       └── mod.rs        # (placeholder)
├── tests/
├── examples/
└── benches/
```

## Step 4: Implement Error Types

**Reference:** Design doc 003, Part 1

Create `src/error.rs`:

<details>
<summary>Click to see complete error.rs implementation</summary>

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

</details>

**Test it compiles:**
```bash
cargo build
```

## Step 5: Implement S-Expression Types

**Reference:** Design doc 003, Part 2

Create `src/sexp/types.rs`:

<details>
<summary>Click to see complete types.rs implementation</summary>

```rust
use crate::error::Position;

/// An S-expression value
#[derive(Debug, Clone, PartialEq)]
pub enum SExp {
    Symbol(Symbol),
    Keyword(Keyword),
    String(StringLit),
    Number(Number),
    Nil(Nil),
    List(List),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub value: String,
    pub pos: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Keyword {
    pub name: String,  // Without the ':'
    pub pos: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringLit {
    pub value: String,  // Unescaped value
    pub pos: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Number {
    pub value: String,  // Keep as string for now
    pub pos: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Nil {
    pub pos: Position,
}

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

</details>

Update `src/sexp/mod.rs`:

```rust
pub mod types;

pub use types::*;
```

**Test it compiles:**
```bash
cargo build
```

## Step 6: Implement Lexer

**Reference:** Design doc 003, Part 3

Create `src/sexp/lexer.rs` with the complete implementation from the design doc.

<details>
<summary>Implementation checklist</summary>

The lexer should have:
- [ ] `Token` and `TokenType` types
- [ ] `Lexer` struct with state (position, line, column)
- [ ] `tokenize()` method returning `Result<Vec<Token>>`
- [ ] `next_token()` private method
- [ ] Methods for each token type:
  - [ ] `read_keyword()`
  - [ ] `read_string()` with escape sequences
  - [ ] `read_number()`
  - [ ] `read_symbol()`
- [ ] Helper methods:
  - [ ] `skip_whitespace_and_comments()`
  - [ ] `is_symbol_start()`, `is_symbol_char()`
  - [ ] `current_char()`, `peek()`, `advance()`
  - [ ] `is_at_end()`, `current_position()`

</details>

**Key implementation notes:**
- Track line and column in `advance()`
- Handle escape sequences: `\n`, `\t`, `\r`, `\\`, `\"`
- Comments start with `;` and go to end of line
- `nil` is a special symbol

Update `src/sexp/mod.rs`:

```rust
pub mod types;
pub mod lexer;

pub use types::*;
pub use lexer::*;
```

**Test it:**

Create `tests/lexer_tests.rs`:

```rust
use oxur_ast::sexp::lexer::{Lexer, TokenType};

#[test]
fn test_parens() {
    let tokens = Lexer::new("()").tokenize().unwrap();
    assert_eq!(tokens[0].typ, TokenType::LParen);
    assert_eq!(tokens[1].typ, TokenType::RParen);
}

#[test]
fn test_symbols() {
    let tokens = Lexer::new("foo bar").tokenize().unwrap();
    assert_eq!(tokens[0].typ, TokenType::Symbol);
    assert_eq!(tokens[0].lexeme, "foo");
}

#[test]
fn test_keywords() {
    let tokens = Lexer::new(":name :kind").tokenize().unwrap();
    assert_eq!(tokens[0].typ, TokenType::Keyword);
    assert_eq!(tokens[0].lexeme, "name");
}
```

Run tests:
```bash
cargo test lexer_tests
```

---

## Session 1 Checkpoint

At this point, you should have:
- ✅ Workspace created
- ✅ Error types implemented
- ✅ S-expression types implemented
- ✅ Lexer implemented and tested

**Verify:**
```bash
cargo build
cargo test
cargo clippy -- -D warnings
```

All should pass with no warnings.

---

# Session 2: Phase 0 Completion + Phase 1 Start

## Step 7: Implement Parser

**Reference:** Design doc 003, Part 4

Create `src/sexp/parser.rs`:

<details>
<summary>Implementation checklist</summary>

The parser should have:
- [ ] `Parser` struct with tokens and current position
- [ ] `new()` constructor
- [ ] `parse_str()` convenience method
- [ ] `parse()` method returning `Result<SExp>`
- [ ] `parse_sexp()` recursive method
- [ ] Methods for each S-expression type:
  - [ ] `parse_list()`
  - [ ] `parse_symbol()`
  - [ ] `parse_keyword()`
  - [ ] `parse_string()`
  - [ ] `parse_number()`
  - [ ] `parse_nil()`
- [ ] Helper methods:
  - [ ] `current_token()`
  - [ ] `check()`, `advance()`
  - [ ] `is_at_end()`

</details>

Update `src/sexp/mod.rs`:

```rust
pub mod types;
pub mod lexer;
pub mod parser;

pub use types::*;
pub use lexer::*;
pub use parser::Parser;
```

**Test it:**

Create `tests/parser_tests.rs`:

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
fn test_parse_list() {
    let sexp = Parser::parse_str("(foo bar)").unwrap();
    match sexp {
        SExp::List(l) => assert_eq!(l.elements.len(), 2),
        _ => panic!("Expected List"),
    }
}
```

Run tests:
```bash
cargo test parser_tests
```

## Step 8: Implement Printer

**Reference:** Design doc 003, Part 5

Create `src/sexp/printer.rs`:

<details>
<summary>Implementation checklist</summary>

The printer should have:
- [ ] `Printer` struct with indent settings
- [ ] `new()` and `with_indent()` constructors
- [ ] `print()` method returning `String`
- [ ] `print_sexp()` private method
- [ ] `print_list()` with smart formatting
- [ ] Helper:
  - [ ] `current_indent()`
  - [ ] `escape_string()` function
- [ ] `print_sexp()` convenience function

</details>

Update `src/sexp/mod.rs`:

```rust
pub mod types;
pub mod lexer;
pub mod parser;
pub mod printer;

pub use types::*;
pub use parser::Parser;
pub use printer::{Printer, print_sexp};
```

**Test it:**

Create `tests/printer_tests.rs` and `tests/round_trip_tests.rs`:

```rust
// printer_tests.rs
use oxur_ast::sexp::{Parser, print_sexp};

#[test]
fn test_print_symbol() {
    let sexp = Parser::parse_str("foo").unwrap();
    assert_eq!(print_sexp(&sexp), "foo");
}

// round_trip_tests.rs
use oxur_ast::sexp::{Parser, print_sexp};

fn round_trip(input: &str) {
    let parsed = Parser::parse_str(input).unwrap();
    let printed = print_sexp(&parsed);
    let reparsed = Parser::parse_str(&printed).unwrap();
    assert_eq!(parsed, reparsed);
}

#[test]
fn test_round_trip_simple() {
    round_trip("foo");
    round_trip(":name");
    round_trip("(foo bar)");
}
```

## Step 9: Update lib.rs for Phase 0

Update `src/lib.rs`:

```rust
pub mod error;
pub mod sexp;

// Re-export commonly used items
pub use error::{ParseError, LexError, Position, Result};
pub use sexp::{SExp, Parser, Printer, print_sexp};
```

**Phase 0 Complete! Verify:**

```bash
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

Create example `examples/parse_example.rs`:

```rust
use oxur_ast::sexp::Parser;
use oxur_ast::sexp::print_sexp;

fn main() {
    let input = r#"(Crate :items ())"#;
    
    match Parser::parse_str(input) {
        Ok(sexp) => {
            println!("Parsed successfully!");
            println!("{}", print_sexp(&sexp));
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }
}
```

Run it:
```bash
cargo run --example parse_example
```

## Step 10: Begin Phase 1 - AST Types

**Reference:** Design doc 004, Part 1-2

Create `src/ast/types.rs`:

<details>
<summary>Implementation checklist - Core types</summary>

Implement these types:
- [ ] `NodeId` struct with `DUMMY` constant
- [ ] `AttrVec` type alias
- [ ] `Attribute` struct (simplified for Phase 1)
- [ ] `TokenStream` enum (Source/Empty variants)
- [ ] `Defaultness` enum
- [ ] `Safety` enum
- [ ] `Constness` enum
- [ ] `Extern` enum
- [ ] `CoroutineKind` enum

</details>

Create `src/ast/span.rs`:

<details>
<summary>Implementation checklist - Span types</summary>

Implement:
- [ ] `Span` struct with lo, hi, ctxt fields
- [ ] `Span::DUMMY` constant
- [ ] `Span::new()` and `with_ctxt()` methods
- [ ] `ModSpans` struct
- [ ] `DelSpan` struct

</details>

Update `src/ast/mod.rs`:

```rust
pub mod types;
pub mod span;

pub use types::*;
pub use span::*;
```

**Test compilation:**
```bash
cargo build
```

---

## Session 2 Checkpoint

At this point you should have:
- ✅ Phase 0 completely implemented
- ✅ All Phase 0 tests passing
- ✅ Phase 1 AST core types started

**Verify:**
```bash
cargo test
cargo run --example parse_example
```

---

# Session 3: Phase 1 Completion

## Step 11: Implement Path Types

**Reference:** Design doc 004, Part 3

Create `src/ast/path.rs`:

<details>
<summary>Implementation checklist</summary>

Implement:
- [ ] `Ident` struct
- [ ] `Path` struct with `from_ident()` method
- [ ] `PathSegment` struct with `from_ident()` method
- [ ] `GenericArgs` placeholder struct
- [ ] `Visibility` enum (Public/Restricted/Inherited)
- [ ] `VisRestrictionKind` enum

</details>

Update `src/ast/mod.rs`:

```rust
pub mod types;
pub mod span;
pub mod path;

pub use types::*;
pub use span::*;
pub use path::*;
```

## Step 12: Implement Item Types

**Reference:** Design doc 004, Part 4

Create `src/ast/item.rs`:

<details>
<summary>Implementation checklist</summary>

Implement (Phase 1 subset):
- [ ] `Item` struct
- [ ] `ItemKind` enum (only `Fn` variant for Phase 1)
- [ ] `Fn` struct
- [ ] `FnSig`, `FnHeader`, `FnDecl` structs
- [ ] `Param` struct
- [ ] `FnRetTy` enum
- [ ] `Generics` struct with `empty()` method
- [ ] `GenericParam` placeholder
- [ ] `WhereClause` struct with `empty()` method
- [ ] `WherePredicate` placeholder
- [ ] `Ty` struct
- [ ] `TyKind` enum (only `Path` variant for Phase 1)
- [ ] `QSelf` placeholder
- [ ] `Pat` struct
- [ ] `PatKind` enum (only `Ident` variant for Phase 1)

</details>

Update `src/ast/mod.rs`:

```rust
pub mod types;
pub mod span;
pub mod path;
pub mod item;

pub use types::*;
pub use span::*;
pub use path::*;
pub use item::*;
```

## Step 13: Implement Expression Types

**Reference:** Design doc 004, Part 5

Create `src/ast/expr.rs`:

<details>
<summary>Implementation checklist</summary>

Implement (Phase 1 subset):
- [ ] `Expr` struct
- [ ] `ExprKind` enum (MacCall, Lit, Path variants)
- [ ] `MacCall` struct with `new()` method
- [ ] `MacArgs` enum (Empty, Delimited variants)
- [ ] `Delimiter` enum
- [ ] `Lit` struct
- [ ] `LitKind` enum (Str, Int variants)
- [ ] `Block` struct with `new()` method
- [ ] `BlockCheckMode` enum

</details>

## Step 14: Implement Statement Types

**Reference:** Design doc 004, Part 6

Create `src/ast/stmt.rs`:

<details>
<summary>Implementation checklist</summary>

Implement (Phase 1 subset):
- [ ] `Stmt` struct
- [ ] `StmtKind` enum (Expr, Semi, Let, Item, MacCall, Empty)
- [ ] `Local` struct
- [ ] `LocalInit` struct
- [ ] `MacCallStmt` struct
- [ ] `MacStmtStyle` enum

</details>

## Step 15: Implement Crate Type

**Reference:** Design doc 004, Part 7

Update `src/ast/mod.rs`:

```rust
pub mod types;
pub mod span;
pub mod path;
pub mod item;
pub mod expr;
pub mod stmt;

pub use types::*;
pub use span::*;
pub use path::*;
pub use item::*;
pub use expr::*;
pub use stmt::*;

/// The root of the AST
#[derive(Debug, Clone, PartialEq)]
pub struct Crate {
    pub attrs: AttrVec,
    pub items: Vec<Item>,
    pub spans: ModSpans,
    pub id: NodeId,
    pub is_placeholder: bool,
}

impl Crate {
    pub fn new(items: Vec<Item>, spans: ModSpans, id: NodeId) -> Self {
        Self {
            attrs: Vec::new(),
            items,
            spans,
            id,
            is_placeholder: false,
        }
    }
}
```

Update `src/lib.rs`:

```rust
pub mod error;
pub mod sexp;
pub mod ast;

pub use error::{ParseError, LexError, Position, Result};
pub use sexp::{SExp, Parser, Printer, print_sexp};
pub use ast::Crate;
```

**Verify AST types compile:**
```bash
cargo build
```

## Step 16: Implement Builder Infrastructure

**Reference:** Design doc 004, Part 8-9

Create `src/builder/helpers.rs` - implement all helper functions from the design doc.

Create `src/builder/build.rs` - implement `AstBuilder` struct with:
- [ ] `new()` method
- [ ] `next_id()` method
- [ ] `build_crate()` method
- [ ] Private helper methods for basic types

Create `src/builder/mod.rs`:

```rust
mod build;
mod helpers;

pub use build::AstBuilder;
```

## Step 17: Implement Item Builder

**Reference:** Design doc 004, Part 10

Create `src/builder/item.rs` - implement all item building methods:

<details>
<summary>Key methods to implement</summary>

- [ ] `build_item()`
- [ ] `build_visibility()`
- [ ] `build_ident()`
- [ ] `build_item_kind()`
- [ ] `build_fn()`
- [ ] `build_fn_sig()`
- [ ] `build_fn_header()`
- [ ] `build_fn_decl()`
- [ ] `build_fn_ret_ty()`
- [ ] `build_generics()`
- [ ] `build_where_clause()`
- [ ] Helper methods for enums (safety, constness, etc.)

</details>

## Step 18: Implement Expression Builder

**Reference:** Design doc 004, Part 11

Create `src/builder/expr.rs`:

<details>
<summary>Key methods to implement</summary>

- [ ] `build_block()`
- [ ] `build_expr()`
- [ ] `build_expr_kind()`
- [ ] `build_mac_call()`
- [ ] `build_path()`
- [ ] `build_path_segment()`
- [ ] `build_mac_args()`
- [ ] `build_del_span()`
- [ ] `build_delimiter()`
- [ ] `build_token_stream()`

</details>

## Step 19: Implement Statement Builder

**Reference:** Design doc 004, Part 12

Create `src/builder/stmt.rs`:

<details>
<summary>Key methods to implement</summary>

- [ ] `build_stmt()`
- [ ] `build_stmt_kind()`

</details>

Update `src/builder/mod.rs`:

```rust
mod build;
mod helpers;
mod item;
mod expr;
mod stmt;

pub use build::AstBuilder;
```

Update `src/lib.rs`:

```rust
pub mod error;
pub mod sexp;
pub mod ast;
pub mod builder;

pub use error::{ParseError, LexError, Position, Result};
pub use sexp::{SExp, Parser, Printer, print_sexp};
pub use ast::Crate;
pub use builder::AstBuilder;
```

## Step 20: Comprehensive Testing

**Reference:** Design doc 004, Part 14

Create `tests/builder_tests.rs` with tests for:
- [ ] `test_build_span()`
- [ ] `test_build_ident()`
- [ ] `test_build_path()`
- [ ] `test_build_simple_crate()`

Run all tests:
```bash
cargo test
```

## Step 21: Create Example

**Reference:** Design doc 004, Part 15

Create `examples/build_hello.rs` - the complete Hello World example from the design doc.

Run it:
```bash
cargo run --example build_hello
```

You should see:
```
✓ Successfully built Hello World AST!
  Items: 1
  Function name: main
  Statements: 1
```

---

# Session 3 Checkpoint - Phase 1 Complete!

**Final verification:**

```bash
# Build
cargo build

# All tests
cargo test

# Clippy
cargo clippy -- -D warnings

# Format
cargo fmt --check

# Examples
cargo run --example parse_example
cargo run --example build_hello
```

**Success criteria:**
- ✅ All Phase 0 and Phase 1 code implemented
- ✅ All tests passing
- ✅ Examples running successfully
- ✅ No warnings from clippy
- ✅ Code properly formatted

---

# What You've Built

Congratulations! You've implemented:

1. **S-expression Infrastructure (Phase 0)**
   - Complete lexer with position tracking
   - Recursive descent parser
   - Pretty printer with formatting
   - Full round-trip support

2. **Rust AST Types (Phase 1)**
   - All types needed for Hello World
   - Proper structure and relationships
   - Type-safe construction

3. **AST Builder (Phase 1)**
   - S-expression → Rust AST conversion
   - Error handling with good messages
   - Comprehensive helpers

**You can now:**
- Parse S-expressions
- Build Rust AST nodes from S-expressions
- Pretty-print S-expressions
- Round-trip: S-expr → AST → S-expr

**Next steps:**
- Phase 2: Generator (AST → S-expr)
- Phase 3: Integration with syn
- Phase 4: Complete AST coverage

---

# Troubleshooting

## Common Issues

**Compilation errors:**
1. Check all module declarations in `mod.rs` files
2. Verify all `pub use` statements in `lib.rs`
3. Ensure all types are public where needed

**Test failures:**
1. Check Position tracking in lexer
2. Verify error messages match expectations
3. Ensure round-trip preserves structure

**Clippy warnings:**
1. Add `#[allow(dead_code)]` only if temporary
2. Fix unused imports
3. Address any performance suggestions

## Getting Help

If stuck:
1. Re-read the relevant design document section
2. Check the examples in the design docs
3. Look at similar patterns in completed code
4. Verify types match the spec exactly

---

# Notes for Future Sessions

**Phase 2 (Generator):**
- Will add `src/generator/` directory
- Mirror structure of `builder/`
- Add reverse conversion tests

**Phase 3 (Integration):**
- Add `syn` dependency
- Create `src/integration/` directory
- Add CLI tool in `src/bin/`

**Phase 4 (Complete Coverage):**
- Expand all enum variants
- Add remaining expression types
- Implement full code generation

---

*"Step by step, type by type, test by test - the foundation is built."*
