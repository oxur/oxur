---
number: 5
title: "oxur-ast Phase 1: Rust AST Types & Builder"
author: "Duncan McGreggor"
component: AST
tags: [compiler, syntax, types]
created: 2025-12-27
updated: 2025-12-27
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# oxur-ast Phase 1: Rust AST Types & Builder

**Phase**: 1 - AST Types
**Goal**: Define Rust AST types and build S-expression → Rust AST converter
**Estimated Time**: 5-7 days
**Prerequisites**: Phase 0 complete (S-expression infrastructure working)

---

## Overview

Phase 1 builds the Rust AST types needed for "Hello, World!" and the builder that converts S-expressions into these types. This is the core bidirectional conversion layer.

**What we're building:**

1. Rust AST type definitions (structs & enums)
2. AST builder (S-expr → Rust AST)
3. Node ID management
4. Span handling
5. Builder tests

**AST Coverage for Phase 1:**

```rust
fn main() {
    println!("Hello, world!");
}
```

This requires implementing:

- `Crate`, `ModSpans`
- `Item`, `ItemKind::Fn`
- `FnSig`, `FnHeader`, `FnDecl`, `Param`, `FnRetTy`
- `Block`, `BlockCheckMode`
- `Stmt`, `StmtKind`
- `Expr`, `ExprKind::MacCall`
- `MacCall`, `MacArgs`, `DelSpan`, `Delimiter`
- `Path`, `PathSegment`
- `Ident`
- `Visibility`
- `Generics`, `WhereClause`
- `Defaultness`, `Safety`, `Constness`, `Extern`
- Supporting types (`NodeId`, `Span`, `AttrVec`, etc.)

---

## File Structure

Extend `oxur-ast` with:

```
oxur-ast/
├── src/
│   ├── lib.rs
│   ├── error.rs           # (existing)
│   ├── sexp/              # (existing)
│   ├── ast/
│   │   ├── mod.rs         # AST module exports
│   │   ├── types.rs       # Core type definitions
│   │   ├── item.rs        # Item, ItemKind
│   │   ├── expr.rs        # Expr, ExprKind
│   │   ├── stmt.rs        # Stmt, StmtKind
│   │   ├── ty.rs          # Type system (minimal for Phase 1)
│   │   ├── path.rs        # Path, PathSegment
│   │   └── span.rs        # Span, Position
│   └── builder/
│       ├── mod.rs         # Builder module exports
│       ├── build.rs       # Main builder logic
│       ├── item.rs        # Item building
│       ├── expr.rs        # Expr building
│       ├── stmt.rs        # Stmt building
│       └── helpers.rs     # Shared utilities
├── tests/
│   ├── ast_tests.rs       # AST type tests
│   └── builder_tests.rs   # Builder tests
└── examples/
    └── build_hello.rs     # Build Hello World AST
```

---

## Part 1: Core AST Types

### File: `src/ast/types.rs`

Define fundamental types used throughout the AST:

```rust
/// Node ID for AST nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

impl NodeId {
    pub const DUMMY: NodeId = NodeId(usize::MAX);

    pub fn new(id: usize) -> Self {
        NodeId(id)
    }
}

/// Attribute vector (simplified for Phase 1)
pub type AttrVec = Vec<Attribute>;

/// Attribute (placeholder for Phase 1)
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub id: NodeId,
    // Will expand in future phases
}

impl Attribute {
    pub fn empty() -> Self {
        Self { id: NodeId::DUMMY }
    }
}

/// Token stream (simplified for Phase 1)
#[derive(Debug, Clone, PartialEq)]
pub enum TokenStream {
    /// Source representation (for macros)
    Source(String),
    /// Empty token stream
    Empty,
}

impl TokenStream {
    pub fn from_str(s: impl Into<String>) -> Self {
        TokenStream::Source(s.into())
    }
}

/// Defaultness (for trait impls)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Defaultness {
    Default,  // Not a default impl
    Final,    // Final impl
}

/// Safety qualifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Safety {
    Unsafe,
    Safe,
    Default,  // No explicit keyword
}

/// Constness qualifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constness {
    Const,
    NotConst,
}

/// Extern specifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Extern {
    None,
    Explicit(String),  // ABI string (e.g., "C")
}

/// Coroutine kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoroutineKind {
    Async,
    Gen,
}
```

---

## Part 2: Span Types

### File: `src/ast/span.rs`

```rust
use crate::error::Position;

/// Byte span in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub lo: u32,   // Start byte offset
    pub hi: u32,   // End byte offset
    pub ctxt: u32, // Syntax context (hygiene)
}

impl Span {
    pub const DUMMY: Span = Span { lo: 0, hi: 0, ctxt: 0 };

    pub fn new(lo: u32, hi: u32) -> Self {
        Self { lo, hi, ctxt: 0 }
    }

    pub fn with_ctxt(lo: u32, hi: u32, ctxt: u32) -> Self {
        Self { lo, hi, ctxt }
    }

    /// Convert to Position (for error reporting)
    pub fn to_position(&self) -> Position {
        // For Phase 1, we don't have line/column info
        // This would require a SourceMap in future phases
        Position::new(self.lo as usize, 1, 1)
    }
}

/// Module spans (for Crate)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModSpans {
    pub inner_span: Span,
    pub inject_use_span: Span,
}

impl ModSpans {
    pub fn new(inner_span: Span, inject_use_span: Span) -> Self {
        Self { inner_span, inject_use_span }
    }
}

/// Delimiter span (for delimited constructs)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DelSpan {
    pub open: Span,
    pub close: Span,
}

impl DelSpan {
    pub fn new(open: Span, close: Span) -> Self {
        Self { open, close }
    }
}
```

---

## Part 3: Path Types

### File: `src/ast/path.rs`

```rust
use crate::ast::types::*;
use crate::ast::span::Span;

/// Identifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

impl Ident {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }
}

/// Path (like `std::io::Write`)
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub span: Span,
    pub segments: Vec<PathSegment>,
    pub tokens: Option<TokenStream>,
}

impl Path {
    pub fn new(span: Span, segments: Vec<PathSegment>) -> Self {
        Self {
            span,
            segments,
            tokens: None,
        }
    }

    pub fn from_ident(ident: Ident) -> Self {
        Self {
            span: ident.span,
            segments: vec![PathSegment::from_ident(ident)],
            tokens: None,
        }
    }
}

/// Path segment
#[derive(Debug, Clone, PartialEq)]
pub struct PathSegment {
    pub ident: Ident,
    pub id: NodeId,
    pub args: Option<GenericArgs>,  // Placeholder for now
}

impl PathSegment {
    pub fn new(ident: Ident, id: NodeId) -> Self {
        Self {
            ident,
            id,
            args: None,
        }
    }

    pub fn from_ident(ident: Ident) -> Self {
        Self::new(ident, NodeId::DUMMY)
    }
}

/// Generic arguments (placeholder for Phase 1)
#[derive(Debug, Clone, PartialEq)]
pub struct GenericArgs {
    // Will expand in future phases
}

/// Visibility
#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Restricted {
        path: Path,
        shorthand: VisRestrictionKind,
        span: Span,
    },
    Inherited,  // Private
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisRestrictionKind {
    Crate,
    Super,
    In,
}
```

---

## Part 4: Item Types

### File: `src/ast/item.rs`

```rust
use crate::ast::types::*;
use crate::ast::span::Span;
use crate::ast::path::{Ident, Visibility};
use crate::ast::expr::Block;

/// Top-level item
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub attrs: AttrVec,
    pub id: NodeId,
    pub span: Span,
    pub vis: Visibility,
    pub ident: Ident,
    pub kind: ItemKind,
    pub tokens: Option<TokenStream>,
}

/// Item kinds
#[derive(Debug, Clone, PartialEq)]
pub enum ItemKind {
    Fn(Box<Fn>),
    // More variants in future phases:
    // ExternCrate, Use, Static, Const, Mod, ForeignMod,
    // GlobalAsm, TyAlias, Enum, Struct, Union, Trait,
    // TraitAlias, Impl, MacCall, MacroDef
}

/// Function item
#[derive(Debug, Clone, PartialEq)]
pub struct Fn {
    pub defaultness: Defaultness,
    pub sig: FnSig,
    pub generics: Generics,
    pub body: Option<Block>,
}

/// Function signature
#[derive(Debug, Clone, PartialEq)]
pub struct FnSig {
    pub header: FnHeader,
    pub decl: FnDecl,
    pub span: Span,
}

/// Function header
#[derive(Debug, Clone, PartialEq)]
pub struct FnHeader {
    pub safety: Safety,
    pub coroutine_kind: Option<CoroutineKind>,
    pub constness: Constness,
    pub ext: Extern,
}

impl FnHeader {
    pub fn default() -> Self {
        Self {
            safety: Safety::Default,
            coroutine_kind: None,
            constness: Constness::NotConst,
            ext: Extern::None,
        }
    }
}

/// Function declaration
#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    pub inputs: Vec<Param>,
    pub output: FnRetTy,
}

/// Function parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub attrs: AttrVec,
    pub ty: Ty,
    pub pat: Pat,
    pub id: NodeId,
    pub span: Span,
    pub is_placeholder: bool,
}

/// Return type
#[derive(Debug, Clone, PartialEq)]
pub enum FnRetTy {
    Default(Span),  // No return type (unit)
    Ty(Box<Ty>),    // Explicit return type
}

/// Generics (simplified for Phase 1)
#[derive(Debug, Clone, PartialEq)]
pub struct Generics {
    pub params: Vec<GenericParam>,
    pub where_clause: WhereClause,
    pub span: Span,
}

impl Generics {
    pub fn empty(span: Span) -> Self {
        Self {
            params: Vec::new(),
            where_clause: WhereClause::empty(span),
            span,
        }
    }
}

/// Generic parameter (placeholder)
#[derive(Debug, Clone, PartialEq)]
pub struct GenericParam {
    // Will expand in future phases
}

/// Where clause
#[derive(Debug, Clone, PartialEq)]
pub struct WhereClause {
    pub has_where_token: bool,
    pub predicates: Vec<WherePredicate>,
    pub span: Span,
}

impl WhereClause {
    pub fn empty(span: Span) -> Self {
        Self {
            has_where_token: false,
            predicates: Vec::new(),
            span,
        }
    }
}

/// Where predicate (placeholder)
#[derive(Debug, Clone, PartialEq)]
pub struct WherePredicate {
    // Will expand in future phases
}

/// Type (minimal for Phase 1)
#[derive(Debug, Clone, PartialEq)]
pub struct Ty {
    pub id: NodeId,
    pub kind: TyKind,
    pub span: Span,
    pub tokens: Option<TokenStream>,
}

/// Type kinds (minimal for Phase 1)
#[derive(Debug, Clone, PartialEq)]
pub enum TyKind {
    // Will expand in future phases
    Path(Option<QSelf>, Path),
    // More variants: Slice, Array, Ptr, Ref, BareFn, Never, Tup, etc.
}

/// Qualified self (for associated types)
#[derive(Debug, Clone, PartialEq)]
pub struct QSelf {
    // Will expand in future phases
}

/// Pattern (minimal for Phase 1)
#[derive(Debug, Clone, PartialEq)]
pub struct Pat {
    pub id: NodeId,
    pub kind: PatKind,
    pub span: Span,
    pub tokens: Option<TokenStream>,
}

/// Pattern kinds (minimal for Phase 1)
#[derive(Debug, Clone, PartialEq)]
pub enum PatKind {
    // Will expand in future phases
    Ident(Ident),
    // More variants: Wild, Rest, Lit, Range, Slice, Path, Tuple, Struct, etc.
}
```

---

## Part 5: Expression Types

### File: `src/ast/expr.rs`

```rust
use crate::ast::types::*;
use crate::ast::span::{Span, DelSpan};
use crate::ast::path::Path;
use crate::ast::stmt::Stmt;

/// Expression
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
    pub span: Span,
    pub attrs: AttrVec,
    pub tokens: Option<TokenStream>,
}

/// Expression kinds
#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    MacCall(Box<MacCall>),
    Lit(Lit),
    Path(Option<QSelf>, Path),
    // More variants in future phases:
    // Array, Call, MethodCall, Tup, Binary, Unary, Cast, If,
    // While, ForLoop, Loop, Match, Closure, Block, Await, Assign,
    // Field, Index, Range, Struct, Repeat, Paren, Try, Yield, etc.
}

use crate::ast::item::QSelf;

/// Macro call
#[derive(Debug, Clone, PartialEq)]
pub struct MacCall {
    pub path: Path,
    pub args: MacArgs,
    pub prior_type_ascription: Option<(usize, bool)>,
}

impl MacCall {
    pub fn new(path: Path, args: MacArgs) -> Self {
        Self {
            path,
            args,
            prior_type_ascription: None,
        }
    }
}

/// Macro arguments
#[derive(Debug, Clone, PartialEq)]
pub enum MacArgs {
    Empty,
    Delimited {
        dspan: DelSpan,
        delim: Delimiter,
        tokens: TokenStream,
    },
    Eq {
        eq_span: Span,
        tokens: TokenStream,
    },
}

/// Delimiter types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    Paren,      // ()
    Brace,      // {}
    Bracket,    // []
    Invisible,  // No delimiter
}

/// Literal (minimal for Phase 1)
#[derive(Debug, Clone, PartialEq)]
pub struct Lit {
    pub kind: LitKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LitKind {
    Str(String),
    Int(String),
    // More in future: Float, Char, Bool, Byte, ByteStr, etc.
}

/// Block
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub id: NodeId,
    pub rules: BlockCheckMode,
    pub span: Span,
    pub tokens: Option<TokenStream>,
    pub could_be_bare_literal: bool,
}

impl Block {
    pub fn new(stmts: Vec<Stmt>, id: NodeId, span: Span) -> Self {
        Self {
            stmts,
            id,
            rules: BlockCheckMode::Default,
            span,
            tokens: None,
            could_be_bare_literal: false,
        }
    }
}

/// Block check mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockCheckMode {
    Default,
    Unsafe,
}
```

---

## Part 6: Statement Types

### File: `src/ast/stmt.rs`

```rust
use crate::ast::types::*;
use crate::ast::span::Span;
use crate::ast::expr::Expr;
use crate::ast::item::Item;

/// Statement
#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub id: NodeId,
    pub kind: StmtKind,
    pub span: Span,
}

/// Statement kinds
#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    /// Expression statement (no semicolon)
    Expr(Box<Expr>),

    /// Expression with semicolon
    Semi(Box<Expr>),

    /// Let binding
    Let(Box<Local>),

    /// Item declaration
    Item(Box<Item>),

    /// Macro invocation
    MacCall(Box<MacCallStmt>),

    /// Empty statement
    Empty,
}

/// Local variable declaration (for `let`)
#[derive(Debug, Clone, PartialEq)]
pub struct Local {
    pub id: NodeId,
    pub pat: Pat,
    pub ty: Option<Ty>,
    pub init: Option<LocalInit>,
    pub span: Span,
    pub attrs: AttrVec,
    pub tokens: Option<TokenStream>,
}

use crate::ast::item::{Pat, Ty};

/// Local initializer
#[derive(Debug, Clone, PartialEq)]
pub struct LocalInit {
    pub expr: Box<Expr>,
    pub els: Option<Box<Block>>,
}

use crate::ast::expr::{Block, MacCall};

/// Macro call statement
#[derive(Debug, Clone, PartialEq)]
pub struct MacCallStmt {
    pub mac: MacCall,
    pub style: MacStmtStyle,
    pub attrs: AttrVec,
    pub tokens: Option<TokenStream>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacStmtStyle {
    Semicolon,
    Braces,
    NoBraces,
}
```

---

## Part 7: Crate Type

### File: `src/ast/mod.rs`

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

// Re-export commonly used items
pub use error::{ParseError, LexError, Position, Result};
pub use sexp::{SExp, Parser, Printer, print_sexp};
pub use ast::Crate;
```

---

## Part 8: Builder Infrastructure

### File: `src/builder/helpers.rs`

Helper functions for building:

```rust
use crate::error::{ParseError, Position, Result};
use crate::sexp::{SExp, HasPosition};

/// Expect a list
pub fn expect_list(sexp: &SExp) -> Result<&Vec<SExp>> {
    match sexp {
        SExp::List(list) => Ok(&list.elements),
        _ => Err(ParseError::Expected {
            expected: "list".to_string(),
            found: format!("{:?}", sexp),
            pos: sexp.position(),
        }),
    }
}

/// Expect a symbol
pub fn expect_symbol(sexp: &SExp) -> Result<String> {
    match sexp {
        SExp::Symbol(s) => Ok(s.value.clone()),
        _ => Err(ParseError::Expected {
            expected: "symbol".to_string(),
            found: format!("{:?}", sexp),
            pos: sexp.position(),
        }),
    }
}

/// Expect a keyword
pub fn expect_keyword(sexp: &SExp) -> Result<String> {
    match sexp {
        SExp::Keyword(k) => Ok(k.name.clone()),
        _ => Err(ParseError::Expected {
            expected: "keyword".to_string(),
            found: format!("{:?}", sexp),
            pos: sexp.position(),
        }),
    }
}

/// Expect a string
pub fn expect_string(sexp: &SExp) -> Result<String> {
    match sexp {
        SExp::String(s) => Ok(s.value.clone()),
        _ => Err(ParseError::Expected {
            expected: "string".to_string(),
            found: format!("{:?}", sexp),
            pos: sexp.position(),
        }),
    }
}

/// Expect a number
pub fn expect_number(sexp: &SExp) -> Result<String> {
    match sexp {
        SExp::Number(n) => Ok(n.value.clone()),
        _ => Err(ParseError::Expected {
            expected: "number".to_string(),
            found: format!("{:?}", sexp),
            pos: sexp.position(),
        }),
    }
}

/// Parse keyword arguments from a list
/// Returns a HashMap of keyword name → value SExp
pub fn parse_kwargs(elements: &[SExp]) -> Result<std::collections::HashMap<String, SExp>> {
    use std::collections::HashMap;

    let mut kwargs = HashMap::new();
    let mut i = 0;

    while i < elements.len() {
        let key = expect_keyword(&elements[i])?;
        i += 1;

        if i >= elements.len() {
            return Err(ParseError::Expected {
                expected: "value for keyword".to_string(),
                found: "end of list".to_string(),
                pos: elements[i - 1].position(),
            });
        }

        kwargs.insert(key, elements[i].clone());
        i += 1;
    }

    Ok(kwargs)
}

/// Get a required keyword argument
pub fn get_required<'a>(
    kwargs: &'a std::collections::HashMap<String, SExp>,
    key: &str,
    pos: Position,
) -> Result<&'a SExp> {
    kwargs.get(key).ok_or_else(|| ParseError::Expected {
        expected: format!(":{} field", key),
        found: "missing".to_string(),
        pos,
    })
}

/// Get an optional keyword argument
pub fn get_optional<'a>(
    kwargs: &'a std::collections::HashMap<String, SExp>,
    key: &str,
) -> Option<&'a SExp> {
    kwargs.get(key)
}

/// Parse a list of items using a builder function
pub fn parse_list<T, F>(sexp: &SExp, mut f: F) -> Result<Vec<T>>
where
    F: FnMut(&SExp) -> Result<T>,
{
    let elements = expect_list(sexp)?;
    elements.iter().map(|e| f(e)).collect()
}
```

---

## Part 9: Main Builder

### File: `src/builder/build.rs`

Core builder logic:

```rust
use crate::error::Result;
use crate::sexp::SExp;
use crate::ast::*;
use crate::builder::helpers::*;

pub struct AstBuilder {
    next_node_id: usize,
}

impl AstBuilder {
    pub fn new() -> Self {
        Self { next_node_id: 0 }
    }

    pub fn next_id(&mut self) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id += 1;
        NodeId::new(id)
    }

    /// Build a complete Crate from S-expression
    pub fn build_crate(&mut self, sexp: &SExp) -> Result<Crate> {
        let elements = expect_list(sexp)?;

        // First element should be "Crate"
        if elements.is_empty() {
            return Err(ParseError::Expected {
                expected: "Crate".to_string(),
                found: "empty list".to_string(),
                pos: sexp.position(),
            });
        }

        let type_name = expect_symbol(&elements[0])?;
        if type_name != "Crate" {
            return Err(ParseError::Expected {
                expected: "Crate".to_string(),
                found: type_name,
                pos: elements[0].position(),
            });
        }

        // Parse keyword arguments
        let kwargs = parse_kwargs(&elements[1..])?;

        // Extract fields
        let attrs = self.build_attr_vec(
            get_optional(&kwargs, "attrs").unwrap_or(&SExp::List(List::new(vec![], sexp.position())))
        )?;

        let items = self.build_items(get_required(&kwargs, "items", sexp.position())?)?;

        let spans = self.build_mod_spans(get_required(&kwargs, "spans", sexp.position())?)?;

        let id = self.build_node_id(get_required(&kwargs, "id", sexp.position())?)?;

        let is_placeholder = get_optional(&kwargs, "is-placeholder")
            .and_then(|s| match s {
                SExp::Symbol(sym) => Some(sym.value == "true"),
                _ => None,
            })
            .unwrap_or(false);

        Ok(Crate {
            attrs,
            items,
            spans,
            id,
            is_placeholder,
        })
    }

    fn build_attr_vec(&mut self, sexp: &SExp) -> Result<AttrVec> {
        let elements = expect_list(sexp)?;
        // For Phase 1, just return empty vec
        // Will implement in future phases
        Ok(vec![])
    }

    fn build_items(&mut self, sexp: &SExp) -> Result<Vec<Item>> {
        parse_list(sexp, |s| self.build_item(s))
    }

    fn build_mod_spans(&mut self, sexp: &SExp) -> Result<ModSpans> {
        let elements = expect_list(sexp)?;
        let type_name = expect_symbol(&elements[0])?;

        if type_name != "ModSpans" {
            return Err(ParseError::Expected {
                expected: "ModSpans".to_string(),
                found: type_name,
                pos: elements[0].position(),
            });
        }

        let kwargs = parse_kwargs(&elements[1..])?;

        let inner_span = self.build_span(get_required(&kwargs, "inner-span", sexp.position())?)?;
        let inject_use_span = self.build_span(get_required(&kwargs, "inject-use-span", sexp.position())?)?;

        Ok(ModSpans::new(inner_span, inject_use_span))
    }

    fn build_span(&mut self, sexp: &SExp) -> Result<Span> {
        let elements = expect_list(sexp)?;
        let type_name = expect_symbol(&elements[0])?;

        if type_name != "Span" {
            return Err(ParseError::Expected {
                expected: "Span".to_string(),
                found: type_name,
                pos: elements[0].position(),
            });
        }

        let kwargs = parse_kwargs(&elements[1..])?;

        let lo = expect_number(get_required(&kwargs, "lo", sexp.position())?)?.parse().unwrap();
        let hi = expect_number(get_required(&kwargs, "hi", sexp.position())?)?.parse().unwrap();
        let ctxt = get_optional(&kwargs, "ctxt")
            .and_then(|s| expect_number(s).ok())
            .and_then(|n| n.parse().ok())
            .unwrap_or(0);

        Ok(Span::with_ctxt(lo, hi, ctxt))
    }

    fn build_node_id(&mut self, sexp: &SExp) -> Result<NodeId> {
        let num = expect_number(sexp)?;
        Ok(NodeId::new(num.parse().unwrap()))
    }
}

impl Default for AstBuilder {
    fn default() -> Self {
        Self::new()
    }
}

use crate::sexp::{List, Keyword};
use crate::error::ParseError;
```

---

## Part 10: Item Builder

### File: `src/builder/item.rs`

```rust
use crate::error::Result;
use crate::sexp::SExp;
use crate::ast::*;
use crate::builder::helpers::*;
use crate::builder::build::AstBuilder;

impl AstBuilder {
    pub fn build_item(&mut self, sexp: &SExp) -> Result<Item> {
        let elements = expect_list(sexp)?;
        let type_name = expect_symbol(&elements[0])?;

        if type_name != "Item" {
            return Err(ParseError::Expected {
                expected: "Item".to_string(),
                found: type_name,
                pos: elements[0].position(),
            });
        }

        let kwargs = parse_kwargs(&elements[1..])?;

        let attrs = self.build_attr_vec(
            get_optional(&kwargs, "attrs").unwrap_or(&SExp::List(List::new(vec![], sexp.position())))
        )?;

        let id = self.build_node_id(get_required(&kwargs, "id", sexp.position())?)?;
        let span = self.build_span(get_required(&kwargs, "span", sexp.position())?)?;
        let vis = self.build_visibility(get_required(&kwargs, "vis", sexp.position())?)?;
        let ident = self.build_ident(get_required(&kwargs, "ident", sexp.position())?)?;
        let kind = self.build_item_kind(get_required(&kwargs, "kind", sexp.position())?)?;
        let tokens = None; // Phase 1

        Ok(Item {
            attrs,
            id,
            span,
            vis,
            ident,
            kind,
            tokens,
        })
    }

    fn build_visibility(&mut self, sexp: &SExp) -> Result<Visibility> {
        let elements = expect_list(sexp)?;
        let variant = expect_symbol(&elements[0])?;

        match variant.as_str() {
            "Public" => Ok(Visibility::Public),
            "Inherited" => Ok(Visibility::Inherited),
            "Restricted" => {
                // Will implement in future phases
                todo!("Restricted visibility")
            }
            _ => Err(ParseError::UnexpectedToken {
                token: variant,
                pos: elements[0].position(),
            }),
        }
    }

    pub fn build_ident(&mut self, sexp: &SExp) -> Result<Ident> {
        let elements = expect_list(sexp)?;
        let type_name = expect_symbol(&elements[0])?;

        if type_name != "Ident" {
            return Err(ParseError::Expected {
                expected: "Ident".to_string(),
                found: type_name,
                pos: elements[0].position(),
            });
        }

        let kwargs = parse_kwargs(&elements[1..])?;

        let name = expect_string(get_required(&kwargs, "name", sexp.position())?)?;
        let span = get_optional(&kwargs, "span")
            .map(|s| self.build_span(s))
            .transpose()?
            .unwrap_or(Span::DUMMY);

        Ok(Ident::new(name, span))
    }

    fn build_item_kind(&mut self, sexp: &SExp) -> Result<ItemKind> {
        let elements = expect_list(sexp)?;
        let variant = expect_symbol(&elements[0])?;

        match variant.as_str() {
            "Fn" => {
                let kwargs = parse_kwargs(&elements[1..])?;
                let fn_item = self.build_fn(&kwargs, sexp.position())?;
                Ok(ItemKind::Fn(Box::new(fn_item)))
            }
            _ => Err(ParseError::UnexpectedToken {
                token: format!("ItemKind::{}", variant),
                pos: elements[0].position(),
            }),
        }
    }

    fn build_fn(
        &mut self,
        kwargs: &std::collections::HashMap<String, SExp>,
        pos: crate::error::Position,
    ) -> Result<Fn> {
        let defaultness = get_optional(kwargs, "defaultness")
            .map(|s| self.build_defaultness(s))
            .transpose()?
            .unwrap_or(Defaultness::Final);

        let sig = self.build_fn_sig(get_required(kwargs, "sig", pos)?)?;
        let generics = self.build_generics(get_required(kwargs, "generics", pos)?)?;
        let body = get_optional(kwargs, "body")
            .map(|s| self.build_block(s))
            .transpose()?;

        Ok(Fn {
            defaultness,
            sig,
            generics,
            body,
        })
    }

    fn build_defaultness(&mut self, sexp: &SExp) -> Result<Defaultness> {
        let sym = expect_symbol(sexp)?;
        match sym.as_str() {
            "Default" => Ok(Defaultness::Default),
            "Final" => Ok(Defaultness::Final),
            _ => Err(ParseError::UnexpectedToken {
                token: sym,
                pos: sexp.position(),
            }),
        }
    }

    fn build_fn_sig(&mut self, sexp: &SExp) -> Result<FnSig> {
        let elements = expect_list(sexp)?;
        let type_name = expect_symbol(&elements[0])?;

        if type_name != "FnSig" {
            return Err(ParseError::Expected {
                expected: "FnSig".to_string(),
                found: type_name,
                pos: elements[0].position(),
            });
        }

        let kwargs = parse_kwargs(&elements[1..])?;

        let header = self.build_fn_header(get_required(&kwargs, "header", sexp.position())?)?;
        let decl = self.build_fn_decl(get_required(&kwargs, "decl", sexp.position())?)?;
        let span = self.build_span(get_required(&kwargs, "span", sexp.position())?)?;

        Ok(FnSig { header, decl, span })
    }

    fn build_fn_header(&mut self, sexp: &SExp) -> Result<FnHeader> {
        let elements = expect_list(sexp)?;
        let type_name = expect_symbol(&elements[0])?;

        if type_name != "FnHeader" {
            return Err(ParseError::Expected {
                expected: "FnHeader".to_string(),
                found: type_name,
                pos: elements[0].position(),
            });
        }

        let kwargs = parse_kwargs(&elements[1..])?;

        let safety = self.build_safety(get_required(&kwargs, "safety", sexp.position())?)?;
        let constness = self.build_constness(get_required(&kwargs, "constness", sexp.position())?)?;
        let ext = self.build_extern(get_required(&kwargs, "ext", sexp.position())?)?;
        let coroutine_kind = None; // Phase 1

        Ok(FnHeader {
            safety,
            coroutine_kind,
            constness,
            ext,
        })
    }

    fn build_safety(&mut self, sexp: &SExp) -> Result<Safety> {
        let sym = expect_symbol(sexp)?;
        match sym.as_str() {
            "Unsafe" => Ok(Safety::Unsafe),
            "Safe" => Ok(Safety::Safe),
            "Default" => Ok(Safety::Default),
            _ => Err(ParseError::UnexpectedToken {
                token: sym,
                pos: sexp.position(),
            }),
        }
    }

    fn build_constness(&mut self, sexp: &SExp) -> Result<Constness> {
        let sym = expect_symbol(sexp)?;
        match sym.as_str() {
            "Const" => Ok(Constness::Const),
            "NotConst" => Ok(Constness::NotConst),
            _ => Err(ParseError::UnexpectedToken {
                token: sym,
                pos: sexp.position(),
            }),
        }
    }

    fn build_extern(&mut self, sexp: &SExp) -> Result<Extern> {
        match sexp {
            SExp::Symbol(s) if s.value == "None" => Ok(Extern::None),
            SExp::List(_) => {
                // (Explicit "C")
                todo!("Explicit extern")
            }
            _ => Err(ParseError::UnexpectedToken {
                token: format!("{:?}", sexp),
                pos: sexp.position(),
            }),
        }
    }

    fn build_fn_decl(&mut self, sexp: &SExp) -> Result<FnDecl> {
        let elements = expect_list(sexp)?;
        let type_name = expect_symbol(&elements[0])?;

        if type_name != "FnDecl" {
            return Err(ParseError::Expected {
                expected: "FnDecl".to_string(),
                found: type_name,
                pos: elements[0].position(),
            });
        }

        let kwargs = parse_kwargs(&elements[1..])?;

        let inputs = parse_list(get_required(&kwargs, "inputs", sexp.position())?, |s| {
            self.build_param(s)
        })?;

        let output = self.build_fn_ret_ty(get_required(&kwargs, "output", sexp.position())?)?;

        Ok(FnDecl { inputs, output })
    }

    fn build_param(&mut self, _sexp: &SExp) -> Result<Param> {
        // Phase 1: Hello World has no params
        todo!("build_param")
    }

    fn build_fn_ret_ty(&mut self, sexp: &SExp) -> Result<FnRetTy> {
        let elements = expect_list(sexp)?;
        let variant = expect_symbol(&elements[0])?;

        match variant.as_str() {
            "Default" => {
                let span = self.build_span(&elements[1])?;
                Ok(FnRetTy::Default(span))
            }
            "Ty" => {
                todo!("Ty return type")
            }
            _ => Err(ParseError::UnexpectedToken {
                token: variant,
                pos: elements[0].position(),
            }),
        }
    }

    fn build_generics(&mut self, sexp: &SExp) -> Result<Generics> {
        let elements = expect_list(sexp)?;
        let type_name = expect_symbol(&elements[0])?;

        if type_name != "Generics" {
            return Err(ParseError::Expected {
                expected: "Generics".to_string(),
                found: type_name,
                pos: elements[0].position(),
            });
        }

        let kwargs = parse_kwargs(&elements[1..])?;

        let params = vec![]; // Phase 1: no generic params
        let where_clause = self.build_where_clause(get_required(&kwargs, "where-clause", sexp.position())?)?;
        let span = self.build_span(get_required(&kwargs, "span", sexp.position())?)?;

        Ok(Generics {
            params,
            where_clause,
            span,
        })
    }

    fn build_where_clause(&mut self, sexp: &SExp) -> Result<WhereClause> {
        let elements = expect_list(sexp)?;
        let type_name = expect_symbol(&elements[0])?;

        if type_name != "WhereClause" {
            return Err(ParseError::Expected {
                expected: "WhereClause".to_string(),
                found: type_name,
                pos: elements[0].position(),
            });
        }

        let kwargs = parse_kwargs(&elements[1..])?;

        let has_where_token = get_optional(&kwargs, "has-where-token")
            .map(|s| match s {
                SExp::Symbol(sym) => sym.value == "true",
                _ => false,
            })
            .unwrap_or(false);

        let predicates = vec![]; // Phase 1
        let span = self.build_span(get_required(&kwargs, "span", sexp.position())?)?;

        Ok(WhereClause {
            has_where_token,
            predicates,
            span,
        })
    }
}

use crate::sexp::List;
use crate::error::ParseError;
```

---

## Part 11: Expression Builder

### File: `src/builder/expr.rs`

```rust
use crate::error::Result;
use crate::sexp::SExp;
use crate::ast::*;
use crate::builder::helpers::*;
use crate::builder::build::AstBuilder;

impl AstBuilder {
    pub fn build_block(&mut self, sexp: &SExp) -> Result<Block> {
        let elements = expect_list(sexp)?;
        let type_name = expect_symbol(&elements[0])?;

        if type_name != "Block" {
            return Err(ParseError::Expected {
                expected: "Block".to_string(),
                found: type_name,
                pos: elements[0].position(),
            });
        }

        let kwargs = parse_kwargs(&elements[1..])?;

        let stmts = parse_list(get_required(&kwargs, "stmts", sexp.position())?, |s| {
            self.build_stmt(s)
        })?;

        let id = self.build_node_id(get_required(&kwargs, "id", sexp.position())?)?;
        let span = self.build_span(get_required(&kwargs, "span", sexp.position())?)?;
        let rules = BlockCheckMode::Default; // Phase 1
        let tokens = None;
        let could_be_bare_literal = false;

        Ok(Block {
            stmts,
            id,
            rules,
            span,
            tokens,
            could_be_bare_literal,
        })
    }

    pub fn build_expr(&mut self, sexp: &SExp) -> Result<Expr> {
        let elements = expect_list(sexp)?;
        let type_name = expect_symbol(&elements[0])?;

        if type_name != "Expr" {
            return Err(ParseError::Expected {
                expected: "Expr".to_string(),
                found: type_name,
                pos: elements[0].position(),
            });
        }

        let kwargs = parse_kwargs(&elements[1..])?;

        let id = self.build_node_id(get_required(&kwargs, "id", sexp.position())?)?;
        let kind = self.build_expr_kind(get_required(&kwargs, "kind", sexp.position())?)?;
        let span = self.build_span(get_required(&kwargs, "span", sexp.position())?)?;
        let attrs = vec![]; // Phase 1
        let tokens = None;

        Ok(Expr {
            id,
            kind,
            span,
            attrs,
            tokens,
        })
    }

    fn build_expr_kind(&mut self, sexp: &SExp) -> Result<ExprKind> {
        let elements = expect_list(sexp)?;
        let variant = expect_symbol(&elements[0])?;

        match variant.as_str() {
            "MacCall" => {
                let mac_call = self.build_mac_call(&elements[1])?;
                Ok(ExprKind::MacCall(Box::new(mac_call)))
            }
            _ => Err(ParseError::UnexpectedToken {
                token: format!("ExprKind::{}", variant),
                pos: elements[0].position(),
            }),
        }
    }

    pub fn build_mac_call(&mut self, sexp: &SExp) -> Result<MacCall> {
        let elements = expect_list(sexp)?;
        let type_name = expect_symbol(&elements[0])?;

        if type_name != "MacCall" {
            return Err(ParseError::Expected {
                expected: "MacCall".to_string(),
                found: type_name,
                pos: elements[0].position(),
            });
        }

        let kwargs = parse_kwargs(&elements[1..])?;

        let path = self.build_path(get_required(&kwargs, "path", sexp.position())?)?;
        let args = self.build_mac_args(get_required(&kwargs, "args", sexp.position())?)?;

        Ok(MacCall::new(path, args))
    }

    pub fn build_path(&mut self, sexp: &SExp) -> Result<Path> {
        let elements = expect_list(sexp)?;
        let type_name = expect_symbol(&elements[0])?;

        if type_name != "Path" {
            return Err(ParseError::Expected {
                expected: "Path".to_string(),
                found: type_name,
                pos: elements[0].position(),
            });
        }

        let kwargs = parse_kwargs(&elements[1..])?;

        let span = self.build_span(get_required(&kwargs, "span", sexp.position())?)?;
        let segments = parse_list(get_required(&kwargs, "segments", sexp.position())?, |s| {
            self.build_path_segment(s)
        })?;

        Ok(Path::new(span, segments))
    }

    fn build_path_segment(&mut self, sexp: &SExp) -> Result<PathSegment> {
        let elements = expect_list(sexp)?;
        let type_name = expect_symbol(&elements[0])?;

        if type_name != "PathSegment" {
            return Err(ParseError::Expected {
                expected: "PathSegment".to_string(),
                found: type_name,
                pos: elements[0].position(),
            });
        }

        let kwargs = parse_kwargs(&elements[1..])?;

        let ident = self.build_ident(get_required(&kwargs, "ident", sexp.position())?)?;
        let id = self.build_node_id(get_required(&kwargs, "id", sexp.position())?)?;

        Ok(PathSegment::new(ident, id))
    }

    fn build_mac_args(&mut self, sexp: &SExp) -> Result<MacArgs> {
        let elements = expect_list(sexp)?;
        let variant = expect_symbol(&elements[0])?;

        match variant.as_str() {
            "Delimited" => {
                let kwargs = parse_kwargs(&elements[1..])?;

                let dspan = self.build_del_span(get_required(&kwargs, "dspan", sexp.position())?)?;
                let delim = self.build_delimiter(get_required(&kwargs, "delim", sexp.position())?)?;
                let tokens = self.build_token_stream(get_required(&kwargs, "tokens", sexp.position())?)?;

                Ok(MacArgs::Delimited { dspan, delim, tokens })
            }
            "Empty" => Ok(MacArgs::Empty),
            _ => Err(ParseError::UnexpectedToken {
                token: format!("MacArgs::{}", variant),
                pos: elements[0].position(),
            }),
        }
    }

    fn build_del_span(&mut self, sexp: &SExp) -> Result<DelSpan> {
        let elements = expect_list(sexp)?;
        let type_name = expect_symbol(&elements[0])?;

        if type_name != "DelSpan" {
            return Err(ParseError::Expected {
                expected: "DelSpan".to_string(),
                found: type_name,
                pos: elements[0].position(),
            });
        }

        let kwargs = parse_kwargs(&elements[1..])?;

        let open = self.build_span(get_required(&kwargs, "open", sexp.position())?)?;
        let close = self.build_span(get_required(&kwargs, "close", sexp.position())?)?;

        Ok(DelSpan::new(open, close))
    }

    fn build_delimiter(&mut self, sexp: &SExp) -> Result<Delimiter> {
        let sym = expect_symbol(sexp)?;
        match sym.as_str() {
            "Paren" => Ok(Delimiter::Paren),
            "Brace" => Ok(Delimiter::Brace),
            "Bracket" => Ok(Delimiter::Bracket),
            "Invisible" => Ok(Delimiter::Invisible),
            _ => Err(ParseError::UnexpectedToken {
                token: sym,
                pos: sexp.position(),
            }),
        }
    }

    fn build_token_stream(&mut self, sexp: &SExp) -> Result<TokenStream> {
        match sexp {
            SExp::String(s) => Ok(TokenStream::from_str(s.value.clone())),
            SExp::List(list) => {
                // (TokenStream :source "...")
                let elements = &list.elements;
                let type_name = expect_symbol(&elements[0])?;

                if type_name != "TokenStream" {
                    return Err(ParseError::Expected {
                        expected: "TokenStream".to_string(),
                        found: type_name,
                        pos: elements[0].position(),
                    });
                }

                let kwargs = parse_kwargs(&elements[1..])?;
                let source = expect_string(get_required(&kwargs, "source", sexp.position())?)?;
                Ok(TokenStream::from_str(source))
            }
            _ => Err(ParseError::Expected {
                expected: "TokenStream".to_string(),
                found: format!("{:?}", sexp),
                pos: sexp.position(),
            }),
        }
    }
}

use crate::error::ParseError;
```

---

## Part 12: Statement Builder

### File: `src/builder/stmt.rs`

```rust
use crate::error::Result;
use crate::sexp::SExp;
use crate::ast::*;
use crate::builder::helpers::*;
use crate::builder::build::AstBuilder;

impl AstBuilder {
    pub fn build_stmt(&mut self, sexp: &SExp) -> Result<Stmt> {
        let elements = expect_list(sexp)?;
        let type_name = expect_symbol(&elements[0])?;

        if type_name != "Stmt" {
            return Err(ParseError::Expected {
                expected: "Stmt".to_string(),
                found: type_name,
                pos: elements[0].position(),
            });
        }

        let kwargs = parse_kwargs(&elements[1..])?;

        let id = self.build_node_id(get_required(&kwargs, "id", sexp.position())?)?;
        let kind = self.build_stmt_kind(get_required(&kwargs, "kind", sexp.position())?)?;
        let span = self.build_span(get_required(&kwargs, "span", sexp.position())?)?;

        Ok(Stmt { id, kind, span })
    }

    fn build_stmt_kind(&mut self, sexp: &SExp) -> Result<StmtKind> {
        let elements = expect_list(sexp)?;
        let variant = expect_symbol(&elements[0])?;

        match variant.as_str() {
            "Semi" => {
                let expr = self.build_expr(&elements[1])?;
                Ok(StmtKind::Semi(Box::new(expr)))
            }
            "Expr" => {
                let expr = self.build_expr(&elements[1])?;
                Ok(StmtKind::Expr(Box::new(expr)))
            }
            "Empty" => Ok(StmtKind::Empty),
            _ => Err(ParseError::UnexpectedToken {
                token: format!("StmtKind::{}", variant),
                pos: elements[0].position(),
            }),
        }
    }
}

use crate::error::ParseError;
```

---

## Part 13: Module Exports

### File: `src/builder/mod.rs`

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

---

## Part 14: Tests

### File: `tests/builder_tests.rs`

```rust
use oxur_ast::*;
use oxur_ast::sexp::Parser;

#[test]
fn test_build_span() {
    let sexp_str = "(Span :lo 0 :hi 10)";
    let sexp = Parser::parse_str(sexp_str).unwrap();

    let mut builder = AstBuilder::new();
    let span = builder.build_span(&sexp).unwrap();

    assert_eq!(span.lo, 0);
    assert_eq!(span.hi, 10);
}

#[test]
fn test_build_ident() {
    let sexp_str = r#"(Ident :name "main" :span (Span :lo 3 :hi 7))"#;
    let sexp = Parser::parse_str(sexp_str).unwrap();

    let mut builder = AstBuilder::new();
    let ident = builder.build_ident(&sexp).unwrap();

    assert_eq!(ident.name, "main");
    assert_eq!(ident.span.lo, 3);
    assert_eq!(ident.span.hi, 7);
}

#[test]
fn test_build_path() {
    let sexp_str = r#"
(Path
  :span (Span :lo 17 :hi 24)
  :segments (
    (PathSegment
      :ident (Ident :name "println" :span (Span :lo 17 :hi 24))
      :id 0)))
    "#;

    let sexp = Parser::parse_str(sexp_str).unwrap();
    let mut builder = AstBuilder::new();
    let path = builder.build_path(&sexp).unwrap();

    assert_eq!(path.segments.len(), 1);
    assert_eq!(path.segments[0].ident.name, "println");
}

#[test]
fn test_build_simple_crate() {
    let sexp_str = r#"
(Crate
  :attrs ()
  :items ()
  :spans (ModSpans
           :inner-span (Span :lo 0 :hi 10)
           :inject-use-span (Span :lo 0 :hi 0))
  :id 0
  :is-placeholder false)
    "#;

    let sexp = Parser::parse_str(sexp_str).unwrap();
    let mut builder = AstBuilder::new();
    let crate_node = builder.build_crate(&sexp).unwrap();

    assert_eq!(crate_node.items.len(), 0);
    assert_eq!(crate_node.id.0, 0);
}
```

---

## Part 15: Example

### File: `examples/build_hello.rs`

```rust
use oxur_ast::*;
use oxur_ast::sexp::Parser;

fn main() {
    // Simplified Hello World S-expression
    let sexp_str = r#"
(Crate
  :attrs ()
  :items (
    (Item
      :attrs ()
      :id 0
      :span (Span :lo 0 :hi 50)
      :vis (Inherited)
      :ident (Ident :name "main" :span (Span :lo 3 :hi 7))
      :kind (Fn
              :defaultness Final
              :sig (FnSig
                     :header (FnHeader
                               :safety Default
                               :constness NotConst
                               :ext None)
                     :decl (FnDecl
                             :inputs ()
                             :output (Default (Span :lo 10 :hi 10)))
                     :span (Span :lo 0 :hi 10))
              :generics (Generics
                          :params ()
                          :where-clause (WhereClause
                                          :has-where-token false
                                          :predicates ()
                                          :span (Span :lo 10 :hi 10))
                          :span (Span :lo 7 :hi 10))
              :body (Block
                      :stmts (
                        (Stmt
                          :id 1
                          :kind (Semi
                                  (Expr
                                    :id 2
                                    :kind (MacCall
                                            (MacCall
                                              :path (Path
                                                      :span (Span :lo 17 :hi 24)
                                                      :segments (
                                                        (PathSegment
                                                          :ident (Ident
                                                                   :name "println"
                                                                   :span (Span :lo 17 :hi 24))
                                                          :id 0)))
                                              :args (Delimited
                                                      :dspan (DelSpan
                                                               :open (Span :lo 24 :hi 25)
                                                               :close (Span :lo 42 :hi 43))
                                                      :delim Paren
                                                      :tokens (TokenStream
                                                                :source "\"Hello, world!\""))))))
                                    :span (Span :lo 17 :hi 44)))
                          :span (Span :lo 17 :hi 44)))
                      :id 3
                      :span (Span :lo 13 :hi 48)))))
  :spans (ModSpans
           :inner-span (Span :lo 0 :hi 50)
           :inject-use-span (Span :lo 0 :hi 0))
  :id 0
  :is-placeholder false)
    "#;

    println!("Parsing S-expression...");
    let sexp = Parser::parse_str(sexp_str).expect("Failed to parse S-expression");

    println!("Building Rust AST...");
    let mut builder = AstBuilder::new();
    let crate_node = builder.build_crate(&sexp).expect("Failed to build AST");

    println!("\n✓ Successfully built Hello World AST!");
    println!("  Items: {}", crate_node.items.len());

    if let Some(item) = crate_node.items.first() {
        println!("  Function name: {}", item.ident.name);

        if let ast::ItemKind::Fn(fn_item) = &item.kind {
            if let Some(body) = &fn_item.body {
                println!("  Statements: {}", body.stmts.len());
            }
        }
    }
}
```

---

## Success Criteria

Phase 1 is complete when:

- [ ] All AST types defined and compiling
- [ ] Builder can parse complete Hello World S-expression
- [ ] Builder correctly constructs all node types
- [ ] All tests pass
- [ ] Example runs and outputs success message
- [ ] No compiler warnings
- [ ] Clean `cargo clippy` output
- [ ] Code formatted with `cargo fmt`

---

## Testing Instructions

```bash
# Run all tests
cargo test -p oxur-ast

# Run builder tests specifically
cargo test -p oxur-ast --test builder_tests

# Run example
cargo run -p oxur-ast --example build_hello

# Check
cargo fmt --check -p oxur-ast
cargo clippy -p oxur-ast -- -D warnings
```

---

## Next Phase Preview

**Phase 2: Generator (Rust AST → S-expr)**

Once Phase 1 is complete, we'll build the reverse direction:

- Walk Rust AST nodes
- Generate S-expressions
- Implement visitor pattern
- Add pretty-printing
- Round-trip testing

This will complete the bidirectional conversion!

---

*"From S-expressions to AST - the bridge is built."*
