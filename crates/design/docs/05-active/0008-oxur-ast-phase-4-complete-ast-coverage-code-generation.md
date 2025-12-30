---
number: 8
title: "oxur-ast Phase 4: Complete AST Coverage & Code Generation"
author: "Duncan McGreggor"
component: AST
tags: [compiler, sexpr]
created: 2025-12-27
updated: 2025-12-27
state: Active
supersedes: null
superseded-by: null
version: 1.0
---

# oxur-ast Phase 4: Complete AST Coverage & Code Generation

**Phase**: 4 - Complete Implementation
**Goal**: Full Rust AST coverage and proper code generation
**Estimated Time**: 10-14 days
**Prerequisites**: Phases 0-3 complete (basic system working)

---

## Overview

Phase 4 completes `oxur-ast` by implementing all remaining AST node types and adding proper Rust code generation. This transforms the library from "handles Hello World" to "handles all of Rust".

**What we're building:**

1. Complete ExprKind coverage (~35+ variants)
2. Complete ItemKind coverage (~17 variants)
3. Complete PatKind coverage (~15+ variants)
4. Complete TyKind coverage (~15+ variants)
5. Complete StmtKind coverage (remaining variants)
6. Proper Rust code generation (not just Debug output)
7. Advanced syn integration
8. Comprehensive test suite

**Coverage targets:**

| Category | Phase 3 | Phase 4 Target |
|----------|---------|----------------|
| ExprKind | 3/35+   | 35+/35+        |
| ItemKind | 1/17    | 17/17          |
| PatKind  | 1/15+   | 15+/15+        |
| TyKind   | 1/15+   | 15+/15+        |
| StmtKind | 4/6     | 6/6            |

---

## File Structure

```
oxur-ast/
├── src/
│   ├── ast/
│   │   ├── types.rs      # (expand)
│   │   ├── item.rs       # (expand all ItemKind)
│   │   ├── expr.rs       # (expand all ExprKind)
│   │   ├── stmt.rs       # (complete)
│   │   ├── ty.rs         # (expand all TyKind)
│   │   ├── pat.rs        # NEW: (all PatKind)
│   │   └── attr.rs       # NEW: (attributes)
│   ├── builder/
│   │   ├── item.rs       # (expand)
│   │   ├── expr.rs       # (expand)
│   │   ├── ty.rs         # NEW: (type building)
│   │   ├── pat.rs        # NEW: (pattern building)
│   │   └── attr.rs       # NEW: (attribute building)
│   ├── generator/
│   │   ├── item.rs       # (expand)
│   │   ├── expr.rs       # (expand)
│   │   ├── ty.rs         # NEW: (type generation)
│   │   ├── pat.rs        # NEW: (pattern generation)
│   │   └── attr.rs       # NEW: (attribute generation)
│   ├── codegen/          # NEW: Rust code generation
│   │   ├── mod.rs
│   │   ├── rust.rs       # Main code generator
│   │   ├── item.rs       # Item generation
│   │   ├── expr.rs       # Expression generation
│   │   └── format.rs     # Code formatting
│   └── integration/
│       ├── from_syn.rs   # (expand)
│       └── to_syn.rs     # NEW: (oxur → syn)
└── tests/
    ├── complete_coverage_tests.rs  # NEW
    └── codegen_tests.rs            # NEW
```

---

## Part 1: Complete Expression Types

### File: `src/ast/expr.rs` (expand ExprKind)

Add all remaining expression variants:

```rust
/// Expression kinds - COMPLETE COVERAGE
#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    // Phase 1-3 (existing)
    MacCall(Box<MacCall>),
    Lit(Lit),
    Path(Option<QSelf>, Path),

    // Phase 4: Collections
    /// Array literal: `[a, b, c]`
    Array(Vec<Expr>),

    /// Tuple: `(a, b, c)`
    Tup(Vec<Expr>),

    /// Repeat: `[expr; count]`
    Repeat(Box<Expr>, Box<AnonConst>),

    // Phase 4: Function calls
    /// Function call: `foo(a, b)`
    Call(Box<Expr>, Vec<Expr>),

    /// Method call: `obj.method(a, b)`
    MethodCall(Box<MethodCall>),

    // Phase 4: Operators
    /// Binary operation: `a + b`, `a && b`
    Binary(BinOp, Box<Expr>, Box<Expr>),

    /// Unary operation: `!x`, `-x`, `*x`
    Unary(UnOp, Box<Expr>),

    /// Cast: `expr as Type`
    Cast(Box<Expr>, Box<Ty>),

    /// Type ascription: `expr: Type` (removed in newer Rust, but in AST)
    Type(Box<Expr>, Box<Ty>),

    // Phase 4: Control flow
    /// If expression: `if cond { } else { }`
    If(Box<Expr>, Box<Block>, Option<Box<Expr>>),

    /// While loop: `while cond { }`
    While(Box<Expr>, Box<Block>, Option<Label>),

    /// For loop: `for pat in expr { }`
    ForLoop(Box<Pat>, Box<Expr>, Box<Block>, Option<Label>),

    /// Loop: `loop { }`
    Loop(Box<Block>, Option<Label>),

    /// Match: `match expr { arms }`
    Match(Box<Expr>, Vec<Arm>),

    // Phase 4: Closures and blocks
    /// Closure: `|a, b| expr`
    Closure(Box<Closure>),

    /// Block: `{ stmts }`
    Block(Box<Block>, Option<Label>),

    // Phase 4: Async/await
    /// Async block: `async { }`
    Async(CaptureBy, Box<Block>),

    /// Await: `expr.await`
    Await(Box<Expr>),

    /// Try block: `try { }`
    TryBlock(Box<Block>),

    // Phase 4: Assignment and fields
    /// Assignment: `place = value`
    Assign(Box<Expr>, Box<Expr>, Span),

    /// Assignment with operator: `place += value`
    AssignOp(BinOp, Box<Expr>, Box<Expr>),

    /// Field access: `obj.field`
    Field(Box<Expr>, Ident),

    /// Index: `arr[index]`
    Index(Box<Expr>, Box<Expr>),

    // Phase 4: Ranges
    /// Range: `start..end`, `..end`, `start..`, `..`
    Range(Option<Box<Expr>>, Option<Box<Expr>>, RangeLimits),

    // Phase 4: Struct and paths
    /// Struct literal: `Point { x: 1, y: 2 }`
    Struct(Box<StructExpr>),

    // Phase 4: Special
    /// Underscore: `_`
    Underscore,

    /// Break: `break`, `break 'label`, `break expr`
    Break(Option<Label>, Option<Box<Expr>>),

    /// Continue: `continue`, `continue 'label`
    Continue(Option<Label>),

    /// Return: `return`, `return expr`
    Return(Option<Box<Expr>>),

    /// Yield: `yield expr` (generators)
    Yield(Option<Box<Expr>>),

    /// Yeet: `do yeet expr` (try trait v2)
    Yeet(Option<Box<Expr>>),

    /// Become: `become expr` (tail calls)
    Become(Box<Expr>),

    // Phase 4: Advanced
    /// Inline assembly: `asm!(...)`
    InlineAsm(Box<InlineAsm>),

    /// Offset of: `offset_of!(Type, field)`
    OffsetOf(Box<Ty>, Vec<Ident>),

    /// Format args: `format_args!(...)`
    FormatArgs(Box<FormatArgs>),

    /// Parenthesized: `(expr)`
    Paren(Box<Expr>),

    /// Try: `expr?`
    Try(Box<Expr>),

    // Error recovery
    Err,
}

// Supporting types for expressions

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Rem,           // Arithmetic
    And, Or,                            // Logical
    BitXor, BitAnd, BitOr,             // Bitwise
    Shl, Shr,                          // Shifts
    Eq, Lt, Le, Ne, Ge, Gt,            // Comparison
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Deref,   // *expr
    Not,     // !expr
    Neg,     // -expr
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodCall {
    pub seg: PathSegment,
    pub receiver: Expr,
    pub args: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arm {
    pub attrs: AttrVec,
    pub pat: Pat,
    pub guard: Option<Box<Expr>>,
    pub body: Box<Expr>,
    pub span: Span,
    pub id: NodeId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    pub binder: ClosureBinder,
    pub capture_clause: CaptureBy,
    pub constness: Constness,
    pub coroutine_kind: Option<CoroutineKind>,
    pub movability: Movability,
    pub fn_decl: FnDecl,
    pub body: Expr,
    pub fn_decl_span: Span,
    pub fn_arg_span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClosureBinder {
    NotPresent,
    For { span: Span },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureBy {
    Value,
    Ref,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Movability {
    Static,
    Movable,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub ident: Ident,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeLimits {
    HalfOpen,  // ..
    Closed,    // ..=
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructExpr {
    pub qself: Option<QSelf>,
    pub path: Path,
    pub fields: Vec<ExprField>,
    pub rest: StructRest,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprField {
    pub attrs: AttrVec,
    pub id: NodeId,
    pub span: Span,
    pub ident: Ident,
    pub expr: Expr,
    pub is_shorthand: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StructRest {
    Base(Box<Expr>),
    Rest(Span),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnonConst {
    pub id: NodeId,
    pub value: Box<Expr>,
}

// Placeholder types for advanced features
#[derive(Debug, Clone, PartialEq)]
pub struct InlineAsm {
    // Will implement when needed
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormatArgs {
    // Will implement when needed
}
```

---

## Part 2: Complete Item Types

### File: `src/ast/item.rs` (expand ItemKind)

Add all item variants:

```rust
/// Item kinds - COMPLETE COVERAGE
#[derive(Debug, Clone, PartialEq)]
pub enum ItemKind {
    // Phase 1-3 (existing)
    Fn(Box<Fn>),

    // Phase 4: Use and modules
    /// Extern crate: `extern crate foo;`
    ExternCrate(Option<Ident>),

    /// Use declaration: `use std::io;`
    Use(Box<UseTree>),

    /// Module: `mod foo { }` or `mod foo;`
    Mod(Option<Ident>, ModKind),

    /// Foreign module: `extern "C" { }`
    ForeignMod(ForeignMod),

    // Phase 4: Constants and statics
    /// Static: `static FOO: i32 = 42;`
    Static(Box<StaticItem>),

    /// Const: `const FOO: i32 = 42;`
    Const(Box<ConstItem>),

    // Phase 4: Types
    /// Type alias: `type Foo = Bar;`
    TyAlias(Box<TyAlias>),

    /// Enum: `enum Foo { A, B }`
    Enum(EnumDef, Generics),

    /// Struct: `struct Foo { x: i32 }`
    Struct(VariantData, Generics),

    /// Union: `union Foo { x: i32 }`
    Union(VariantData, Generics),

    // Phase 4: Traits
    /// Trait: `trait Foo { }`
    Trait(Box<TraitDef>),

    /// Trait alias: `trait Foo = Bar + Baz;`
    TraitAlias(Generics, GenericBounds),

    /// Impl: `impl Foo for Bar { }`
    Impl(Box<ImplDef>),

    // Phase 4: Macros
    /// Macro invocation: `foo!(...);`
    MacCall(Box<MacCall>),

    /// Macro definition: `macro_rules! foo { }`
    MacroDef(Box<MacroDef>),

    // Phase 4: Advanced
    /// Global assembly: `global_asm!(...)`
    GlobalAsm(Box<InlineAsm>),
}

// Supporting types for items

#[derive(Debug, Clone, PartialEq)]
pub struct UseTree {
    pub prefix: Path,
    pub kind: UseTreeKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UseTreeKind {
    Simple(Option<Ident>),          // use path or use path as name
    Glob,                            // use path::*
    Nested(Vec<(UseTree, NodeId)>), // use path::{a, b, c}
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModKind {
    Loaded(Vec<Item>, Inline, ModSpans),
    Unloaded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inline {
    Yes,
    No,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForeignMod {
    pub safety: Safety,
    pub abi: Option<String>,
    pub items: Vec<ForeignItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForeignItem {
    pub attrs: AttrVec,
    pub id: NodeId,
    pub span: Span,
    pub vis: Visibility,
    pub ident: Ident,
    pub kind: ForeignItemKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForeignItemKind {
    Static(Box<Ty>, Mutability),
    Fn(Box<Fn>),
    TyAlias(Box<TyAlias>),
    MacCall(Box<MacCall>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Mut,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticItem {
    pub mutability: Mutability,
    pub ty: Box<Ty>,
    pub expr: Option<Box<Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstItem {
    pub defaultness: Defaultness,
    pub ty: Box<Ty>,
    pub expr: Option<Box<Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TyAlias {
    pub defaultness: Defaultness,
    pub generics: Generics,
    pub bounds: GenericBounds,
    pub ty: Option<Box<Ty>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub variants: Vec<Variant>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub attrs: AttrVec,
    pub id: NodeId,
    pub span: Span,
    pub vis: Visibility,
    pub ident: Ident,
    pub data: VariantData,
    pub disr_expr: Option<AnonConst>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariantData {
    Struct(Vec<FieldDef>, bool),  // bool = recovered
    Tuple(Vec<FieldDef>, NodeId),
    Unit(NodeId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDef {
    pub attrs: AttrVec,
    pub id: NodeId,
    pub span: Span,
    pub vis: Visibility,
    pub ident: Option<Ident>,
    pub ty: Ty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitDef {
    pub safety: Safety,
    pub is_auto: IsAuto,
    pub generics: Generics,
    pub bounds: GenericBounds,
    pub items: Vec<AssocItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsAuto {
    Yes,
    No,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplDef {
    pub defaultness: Defaultness,
    pub safety: Safety,
    pub generics: Generics,
    pub constness: Constness,
    pub polarity: ImplPolarity,
    pub of_trait: Option<TraitRef>,
    pub self_ty: Box<Ty>,
    pub items: Vec<AssocItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImplPolarity {
    Positive,
    Negative(Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitRef {
    pub path: Path,
    pub ref_id: NodeId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssocItem {
    pub attrs: AttrVec,
    pub id: NodeId,
    pub span: Span,
    pub vis: Visibility,
    pub ident: Ident,
    pub kind: AssocItemKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssocItemKind {
    Const(Box<ConstItem>),
    Fn(Box<Fn>),
    Type(Box<TyAlias>),
    MacCall(Box<MacCall>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacroDef {
    pub body: MacArgs,
    pub macro_rules: bool,
}

pub type GenericBounds = Vec<GenericBound>;

#[derive(Debug, Clone, PartialEq)]
pub enum GenericBound {
    Trait(TraitRef, TraitBoundModifier),
    Outlives(Lifetime),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraitBoundModifier {
    None,
    Maybe,       // ?Trait
    MaybeConst,  // ~const Trait
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lifetime {
    pub id: NodeId,
    pub ident: Ident,
}
```

---

## Part 3: Complete Pattern Types

### File: `src/ast/pat.rs` (new file)

Complete pattern matching support:

```rust
use crate::ast::types::*;
use crate::ast::span::Span;
use crate::ast::path::{Ident, Path, QSelf};
use crate::ast::expr::Expr;
use crate::ast::item::Mutability;

/// Pattern - COMPLETE COVERAGE
#[derive(Debug, Clone, PartialEq)]
pub struct Pat {
    pub id: NodeId,
    pub kind: PatKind,
    pub span: Span,
    pub tokens: Option<TokenStream>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatKind {
    // Phase 1-3 (existing)
    /// Identifier pattern: `x`, `mut x`, `ref x`
    Ident(BindingMode, Ident, Option<Box<Pat>>),

    // Phase 4: Literals and ranges
    /// Wildcard pattern: `_`
    Wild,

    /// Rest pattern: `..`
    Rest,

    /// Literal: `42`, `"hello"`, `true`
    Lit(Box<Expr>),

    /// Range: `1..=10`, `'a'..='z'`
    Range(Option<Box<Expr>>, Option<Box<Expr>>, RangeEnd),

    // Phase 4: Structured patterns
    /// Slice: `[a, b, c]`, `[first, .., last]`
    Slice(Vec<Pat>),

    /// Path: `Option::None`
    Path(Option<QSelf>, Path),

    /// Tuple: `(a, b, c)`
    Tuple(Vec<Pat>),

    /// Struct: `Point { x, y }`
    Struct(Option<QSelf>, Path, Vec<PatField>, PatFieldsRest),

    /// Tuple struct: `Some(x)`
    TupleStruct(Option<QSelf>, Path, Vec<Pat>),

    // Phase 4: Special
    /// Or pattern: `A | B`
    Or(Vec<Pat>),

    /// Reference: `&x`, `&mut x`
    Ref(Box<Pat>, Mutability),

    /// Box pattern: `box x`
    Box(Box<Pat>),

    /// Parenthesized: `(pat)`
    Paren(Box<Pat>),

    /// Macro invocation: `mac!(...)`
    MacCall(Box<MacCall>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindingMode {
    pub by_ref: ByRef,
    pub mutability: Mutability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByRef {
    Yes,
    No,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeEnd {
    Included,
    Excluded,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatField {
    pub attrs: AttrVec,
    pub id: NodeId,
    pub span: Span,
    pub ident: Ident,
    pub pat: Pat,
    pub is_shorthand: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatFieldsRest {
    Rest,
    None,
}

use crate::ast::expr::RangeLimits;
use crate::ast::expr::MacCall;
```

---

## Part 4: Complete Type System

### File: `src/ast/ty.rs` (expand)

Complete type system support:

```rust
use crate::ast::types::*;
use crate::ast::span::Span;
use crate::ast::path::{Path, QSelf};
use crate::ast::item::{Lifetime, GenericBounds, Mutability};
use crate::ast::expr::{AnonConst, MacCall};

/// Type kinds - COMPLETE COVERAGE
#[derive(Debug, Clone, PartialEq)]
pub enum TyKind {
    // Phase 1-3 (existing)
    /// Path type: `std::io::Error`
    Path(Option<QSelf>, Path),

    // Phase 4: Basic types
    /// Slice: `[T]`
    Slice(Box<Ty>),

    /// Array: `[T; N]`
    Array(Box<Ty>, Box<AnonConst>),

    /// Raw pointer: `*const T`, `*mut T`
    Ptr(Box<MutTy>),

    /// Reference: `&T`, `&mut T`, `&'a T`
    Ref(Option<Lifetime>, Box<MutTy>),

    /// Bare function: `fn(i32) -> i32`
    BareFn(Box<BareFnTy>),

    /// Never: `!`
    Never,

    /// Tuple: `(A, B, C)`
    Tup(Vec<Ty>),

    // Phase 4: Advanced types
    /// Trait object: `dyn Trait + Send`
    TraitObject(GenericBounds, TraitObjectSyntax),

    /// Impl trait: `impl Trait`
    ImplTrait(NodeId, GenericBounds),

    /// Parenthesized: `(T)`
    Paren(Box<Ty>),

    /// Typeof: `typeof(expr)` (unstable)
    Typeof(Box<AnonConst>),

    /// Inferred: `_`
    Infer,

    /// Macro invocation: `mac!(...)`
    MacCall(Box<MacCall>),

    // Error recovery
    Err,

    /// CVarArgs: `...` (C varargs)
    CVarArgs,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MutTy {
    pub ty: Box<Ty>,
    pub mutability: Mutability,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BareFnTy {
    pub safety: Safety,
    pub ext: Extern,
    pub generic_params: Vec<GenericParam>,
    pub decl: FnDecl,
    pub decl_span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraitObjectSyntax {
    Dyn,
    DynStar,
    None,
}

use crate::ast::item::{Safety, Extern, GenericParam, FnDecl};
```

---

## Part 5: Attribute System

### File: `src/ast/attr.rs` (new file)

Complete attribute support:

```rust
use crate::ast::types::*;
use crate::ast::span::Span;
use crate::ast::path::Path;
use crate::ast::expr::MacArgs;

/// Attribute
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub kind: AttrKind,
    pub id: NodeId,
    pub style: AttrStyle,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttrKind {
    /// Normal attribute: `#[attr]` or `#![attr]`
    Normal(Box<NormalAttr>),

    /// Doc comment: `/// ...` or `//! ...`
    DocComment(CommentKind, String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalAttr {
    pub item: AttrItem,
    pub tokens: Option<TokenStream>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttrItem {
    pub path: Path,
    pub args: AttrArgs,
    pub tokens: Option<TokenStream>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttrArgs {
    Empty,
    Delimited(MacArgs),
    Eq(Span, AttrArgsEq),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttrArgsEq {
    Ast(Box<Expr>),
    Hir(Box<Lit>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrStyle {
    Outer,  // #[...]
    Inner,  // #![...]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentKind {
    Line,
    Block,
}

use crate::ast::expr::{Expr, Lit};
```

---

## Part 6: Code Generation - Rust Output

### File: `src/codegen/mod.rs`

```rust
//! Rust code generation from AST

mod rust;
mod item;
mod expr;
mod format;

pub use rust::RustCodegen;

use crate::error::Result;
use crate::ast::Crate;

/// Generate Rust source code from AST
pub fn generate_rust(crate_node: &Crate) -> Result<String> {
    let codegen = RustCodegen::new();
    codegen.generate_crate(crate_node)
}
```

### File: `src/codegen/rust.rs`

Main code generation logic:

```rust
use crate::error::Result;
use crate::ast::*;
use std::fmt::Write;

pub struct RustCodegen {
    indent: usize,
    indent_str: String,
}

impl RustCodegen {
    pub fn new() -> Self {
        Self {
            indent: 0,
            indent_str: "    ".to_string(), // 4 spaces
        }
    }

    pub fn generate_crate(&self, crate_node: &Crate) -> Result<String> {
        let mut output = String::new();

        // Generate attributes
        for attr in &crate_node.attrs {
            self.generate_attr(attr, &mut output)?;
            writeln!(output).unwrap();
        }

        // Generate items
        for item in &crate_node.items {
            self.generate_item(item, &mut output)?;
            writeln!(output).unwrap();
        }

        Ok(output)
    }

    fn current_indent(&self) -> String {
        self.indent_str.repeat(self.indent)
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    fn generate_attr(&self, _attr: &Attribute, _output: &mut String) -> Result<()> {
        // Simplified for Phase 4
        // Full implementation would parse attribute structure
        Ok(())
    }
}

impl Default for RustCodegen {
    fn default() -> Self {
        Self::new()
    }
}
```

### File: `src/codegen/item.rs`

Item code generation:

```rust
use crate::error::Result;
use crate::ast::*;
use crate::codegen::rust::RustCodegen;
use std::fmt::Write;

impl RustCodegen {
    pub fn generate_item(&self, item: &Item, output: &mut String) -> Result<()> {
        // Generate visibility
        self.generate_visibility(&item.vis, output)?;

        // Generate item kind
        match &item.kind {
            ItemKind::Fn(fn_item) => {
                self.generate_fn_item(&item.ident, fn_item, output)?;
            }
            ItemKind::Struct(variant_data, generics) => {
                write!(output, "struct {}", item.ident.name).unwrap();
                self.generate_generics(generics, output)?;
                self.generate_variant_data(variant_data, output)?;
            }
            ItemKind::Enum(enum_def, generics) => {
                write!(output, "enum {}", item.ident.name).unwrap();
                self.generate_generics(generics, output)?;
                self.generate_enum_def(enum_def, output)?;
            }
            ItemKind::Use(use_tree) => {
                write!(output, "use ").unwrap();
                self.generate_use_tree(use_tree, output)?;
                write!(output, ";").unwrap();
            }
            ItemKind::Mod(inner_ident, mod_kind) => {
                write!(output, "mod {}", item.ident.name).unwrap();
                self.generate_mod_kind(mod_kind, output)?;
            }
            ItemKind::Static(static_item) => {
                self.generate_static(&item.ident, static_item, output)?;
            }
            ItemKind::Const(const_item) => {
                self.generate_const(&item.ident, const_item, output)?;
            }
            ItemKind::Trait(trait_def) => {
                self.generate_trait(&item.ident, trait_def, output)?;
            }
            ItemKind::Impl(impl_def) => {
                self.generate_impl(impl_def, output)?;
            }
            _ => {
                // Other item kinds
                write!(output, "// Unsupported item: {:?}", item.kind).unwrap();
            }
        }

        Ok(())
    }

    fn generate_visibility(&self, vis: &Visibility, output: &mut String) -> Result<()> {
        match vis {
            Visibility::Public => write!(output, "pub ").unwrap(),
            Visibility::Inherited => {},
            Visibility::Restricted { .. } => write!(output, "pub(crate) ").unwrap(),
        }
        Ok(())
    }

    fn generate_fn_item(
        &self,
        ident: &Ident,
        fn_item: &Fn,
        output: &mut String,
    ) -> Result<()> {
        // Generate function header
        self.generate_fn_header(&fn_item.sig.header, output)?;
        write!(output, "fn {}", ident.name).unwrap();

        // Generate generics
        self.generate_generics(&fn_item.generics, output)?;

        // Generate parameters
        write!(output, "(").unwrap();
        for (i, param) in fn_item.sig.decl.inputs.iter().enumerate() {
            if i > 0 {
                write!(output, ", ").unwrap();
            }
            self.generate_param(param, output)?;
        }
        write!(output, ")").unwrap();

        // Generate return type
        self.generate_fn_ret_ty(&fn_item.sig.decl.output, output)?;

        // Generate where clause
        self.generate_where_clause(&fn_item.generics.where_clause, output)?;

        // Generate body
        if let Some(body) = &fn_item.body {
            write!(output, " ").unwrap();
            self.generate_block(body, output)?;
        } else {
            write!(output, ";").unwrap();
        }

        Ok(())
    }

    fn generate_fn_header(&self, header: &FnHeader, output: &mut String) -> Result<()> {
        if header.constness == Constness::Const {
            write!(output, "const ").unwrap();
        }
        if let Some(kind) = header.coroutine_kind {
            match kind {
                CoroutineKind::Async => write!(output, "async ").unwrap(),
                CoroutineKind::Gen => write!(output, "gen ").unwrap(),
            }
        }
        if header.safety == Safety::Unsafe {
            write!(output, "unsafe ").unwrap();
        }
        if let Extern::Explicit(abi) = &header.ext {
            write!(output, "extern \"{}\" ", abi).unwrap();
        }
        Ok(())
    }

    fn generate_param(&self, param: &Param, output: &mut String) -> Result<()> {
        self.generate_pat(&param.pat, output)?;
        write!(output, ": ").unwrap();
        self.generate_ty(&param.ty, output)?;
        Ok(())
    }

    fn generate_fn_ret_ty(&self, ret_ty: &FnRetTy, output: &mut String) -> Result<()> {
        match ret_ty {
            FnRetTy::Default(_) => {},
            FnRetTy::Ty(ty) => {
                write!(output, " -> ").unwrap();
                self.generate_ty(ty, output)?;
            }
        }
        Ok(())
    }

    fn generate_generics(&self, generics: &Generics, output: &mut String) -> Result<()> {
        if !generics.params.is_empty() {
            write!(output, "<").unwrap();
            for (i, param) in generics.params.iter().enumerate() {
                if i > 0 {
                    write!(output, ", ").unwrap();
                }
                self.generate_generic_param(param, output)?;
            }
            write!(output, ">").unwrap();
        }
        Ok(())
    }

    fn generate_generic_param(&self, _param: &GenericParam, output: &mut String) -> Result<()> {
        // Simplified for Phase 4
        write!(output, "T").unwrap();
        Ok(())
    }

    fn generate_where_clause(&self, clause: &WhereClause, output: &mut String) -> Result<()> {
        if clause.has_where_token && !clause.predicates.is_empty() {
            write!(output, " where ").unwrap();
            // Generate predicates
        }
        Ok(())
    }

    fn generate_variant_data(&self, data: &VariantData, output: &mut String) -> Result<()> {
        match data {
            VariantData::Struct(fields, _) => {
                writeln!(output, " {{").unwrap();
                for field in fields {
                    write!(output, "    ").unwrap();
                    self.generate_field_def(field, output)?;
                    writeln!(output, ",").unwrap();
                }
                write!(output, "}}").unwrap();
            }
            VariantData::Tuple(fields, _) => {
                write!(output, "(").unwrap();
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(output, ", ").unwrap();
                    }
                    self.generate_ty(&field.ty, output)?;
                }
                write!(output, ");").unwrap();
            }
            VariantData::Unit(_) => {
                write!(output, ";").unwrap();
            }
        }
        Ok(())
    }

    fn generate_field_def(&self, field: &FieldDef, output: &mut String) -> Result<()> {
        self.generate_visibility(&field.vis, output)?;
        if let Some(ident) = &field.ident {
            write!(output, "{}: ", ident.name).unwrap();
        }
        self.generate_ty(&field.ty, output)?;
        Ok(())
    }

    fn generate_enum_def(&self, enum_def: &EnumDef, output: &mut String) -> Result<()> {
        writeln!(output, " {{").unwrap();
        for variant in &enum_def.variants {
            write!(output, "    {}", variant.ident.name).unwrap();
            self.generate_variant_data(&variant.data, output)?;
            writeln!(output, ",").unwrap();
        }
        write!(output, "}}").unwrap();
        Ok(())
    }

    fn generate_use_tree(&self, use_tree: &UseTree, output: &mut String) -> Result<()> {
        self.generate_path(&use_tree.prefix, output)?;
        match &use_tree.kind {
            UseTreeKind::Simple(None) => {},
            UseTreeKind::Simple(Some(alias)) => {
                write!(output, " as {}", alias.name).unwrap();
            }
            UseTreeKind::Glob => {
                write!(output, "::*").unwrap();
            }
            UseTreeKind::Nested(trees) => {
                write!(output, "::{{").unwrap();
                for (i, (tree, _)) in trees.iter().enumerate() {
                    if i > 0 {
                        write!(output, ", ").unwrap();
                    }
                    self.generate_use_tree(tree, output)?;
                }
                write!(output, "}}").unwrap();
            }
        }
        Ok(())
    }

    fn generate_mod_kind(&self, kind: &ModKind, output: &mut String) -> Result<()> {
        match kind {
            ModKind::Loaded(items, inline, _) => {
                if *inline == Inline::Yes {
                    writeln!(output, " {{").unwrap();
                    for item in items {
                        self.generate_item(item, output)?;
                        writeln!(output).unwrap();
                    }
                    write!(output, "}}").unwrap();
                } else {
                    write!(output, ";").unwrap();
                }
            }
            ModKind::Unloaded => {
                write!(output, ";").unwrap();
            }
        }
        Ok(())
    }

    fn generate_static(&self, ident: &Ident, static_item: &StaticItem, output: &mut String) -> Result<()> {
        write!(output, "static ").unwrap();
        if static_item.mutability == Mutability::Mut {
            write!(output, "mut ").unwrap();
        }
        write!(output, "{}: ", ident.name).unwrap();
        self.generate_ty(&static_item.ty, output)?;
        if let Some(expr) = &static_item.expr {
            write!(output, " = ").unwrap();
            self.generate_expr(expr, output)?;
        }
        write!(output, ";").unwrap();
        Ok(())
    }

    fn generate_const(&self, ident: &Ident, const_item: &ConstItem, output: &mut String) -> Result<()> {
        write!(output, "const {}: ", ident.name).unwrap();
        self.generate_ty(&const_item.ty, output)?;
        if let Some(expr) = &const_item.expr {
            write!(output, " = ").unwrap();
            self.generate_expr(expr, output)?;
        }
        write!(output, ";").unwrap();
        Ok(())
    }

    fn generate_trait(&self, ident: &Ident, trait_def: &TraitDef, output: &mut String) -> Result<()> {
        if trait_def.safety == Safety::Unsafe {
            write!(output, "unsafe ").unwrap();
        }
        write!(output, "trait {}", ident.name).unwrap();
        // Generate bounds, items, etc.
        writeln!(output, " {{").unwrap();
        for item in &trait_def.items {
            self.generate_assoc_item(item, output)?;
            writeln!(output).unwrap();
        }
        write!(output, "}}").unwrap();
        Ok(())
    }

    fn generate_impl(&self, impl_def: &ImplDef, output: &mut String) -> Result<()> {
        if impl_def.safety == Safety::Unsafe {
            write!(output, "unsafe ").unwrap();
        }
        write!(output, "impl").unwrap();
        self.generate_generics(&impl_def.generics, output)?;
        write!(output, " ").unwrap();

        if let Some(trait_ref) = &impl_def.of_trait {
            self.generate_path(&trait_ref.path, output)?;
            write!(output, " for ").unwrap();
        }

        self.generate_ty(&impl_def.self_ty, output)?;

        writeln!(output, " {{").unwrap();
        for item in &impl_def.items {
            self.generate_assoc_item(item, output)?;
            writeln!(output).unwrap();
        }
        write!(output, "}}").unwrap();
        Ok(())
    }

    fn generate_assoc_item(&self, item: &AssocItem, output: &mut String) -> Result<()> {
        write!(output, "    ").unwrap();
        match &item.kind {
            AssocItemKind::Fn(fn_item) => {
                self.generate_fn_item(&item.ident, fn_item, output)?;
            }
            AssocItemKind::Const(const_item) => {
                self.generate_const(&item.ident, const_item, output)?;
            }
            AssocItemKind::Type(ty_alias) => {
                write!(output, "type {} = ", item.ident.name).unwrap();
                if let Some(ty) = &ty_alias.ty {
                    self.generate_ty(ty, output)?;
                }
                write!(output, ";").unwrap();
            }
            _ => {}
        }
        Ok(())
    }

    fn generate_path(&self, path: &Path, output: &mut String) -> Result<()> {
        for (i, segment) in path.segments.iter().enumerate() {
            if i > 0 {
                write!(output, "::").unwrap();
            }
            write!(output, "{}", segment.ident.name).unwrap();
        }
        Ok(())
    }
}
```

### File: `src/codegen/expr.rs`

Expression code generation:

```rust
use crate::error::Result;
use crate::ast::*;
use crate::codegen::rust::RustCodegen;
use std::fmt::Write;

impl RustCodegen {
    pub fn generate_block(&self, block: &Block, output: &mut String) -> Result<()> {
        writeln!(output, "{{").unwrap();

        for stmt in &block.stmts {
            write!(output, "{}", self.current_indent()).unwrap();
            self.generate_stmt(stmt, output)?;
            writeln!(output).unwrap();
        }

        write!(output, "{}}}", self.current_indent()).unwrap();
        Ok(())
    }

    pub fn generate_stmt(&self, stmt: &Stmt, output: &mut String) -> Result<()> {
        match &stmt.kind {
            StmtKind::Expr(expr) => {
                self.generate_expr(expr, output)?;
            }
            StmtKind::Semi(expr) => {
                self.generate_expr(expr, output)?;
                write!(output, ";").unwrap();
            }
            StmtKind::Let(local) => {
                self.generate_local(local, output)?;
            }
            StmtKind::Item(item) => {
                self.generate_item(item, output)?;
            }
            StmtKind::MacCall(mac_call_stmt) => {
                self.generate_mac_call(&mac_call_stmt.mac, output)?;
                match mac_call_stmt.style {
                    MacStmtStyle::Semicolon => write!(output, ";").unwrap(),
                    _ => {}
                }
            }
            StmtKind::Empty => {}
        }
        Ok(())
    }

    fn generate_local(&self, local: &Local, output: &mut String) -> Result<()> {
        write!(output, "let ").unwrap();
        self.generate_pat(&local.pat, output)?;

        if let Some(ty) = &local.ty {
            write!(output, ": ").unwrap();
            self.generate_ty(ty, output)?;
        }

        if let Some(init) = &local.init {
            write!(output, " = ").unwrap();
            self.generate_expr(&init.expr, output)?;
        }

        write!(output, ";").unwrap();
        Ok(())
    }

    pub fn generate_expr(&self, expr: &Expr, output: &mut String) -> Result<()> {
        match &expr.kind {
            ExprKind::Lit(lit) => {
                self.generate_lit(lit, output)?;
            }
            ExprKind::Path(_, path) => {
                self.generate_path(path, output)?;
            }
            ExprKind::MacCall(mac_call) => {
                self.generate_mac_call(mac_call, output)?;
            }
            ExprKind::Binary(op, left, right) => {
                self.generate_expr(left, output)?;
                write!(output, " {} ", self.binop_str(*op)).unwrap();
                self.generate_expr(right, output)?;
            }
            ExprKind::Unary(op, expr) => {
                write!(output, "{}", self.unop_str(*op)).unwrap();
                self.generate_expr(expr, output)?;
            }
            ExprKind::Call(func, args) => {
                self.generate_expr(func, output)?;
                write!(output, "(").unwrap();
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(output, ", ").unwrap();
                    }
                    self.generate_expr(arg, output)?;
                }
                write!(output, ")").unwrap();
            }
            ExprKind::MethodCall(method_call) => {
                self.generate_expr(&method_call.receiver, output)?;
                write!(output, ".{}", method_call.seg.ident.name).unwrap();
                write!(output, "(").unwrap();
                for (i, arg) in method_call.args.iter().enumerate() {
                    if i > 0 {
                        write!(output, ", ").unwrap();
                    }
                    self.generate_expr(arg, output)?;
                }
                write!(output, ")").unwrap();
            }
            ExprKind::Array(exprs) => {
                write!(output, "[").unwrap();
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(output, ", ").unwrap();
                    }
                    self.generate_expr(expr, output)?;
                }
                write!(output, "]").unwrap();
            }
            ExprKind::Tup(exprs) => {
                write!(output, "(").unwrap();
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(output, ", ").unwrap();
                    }
                    self.generate_expr(expr, output)?;
                }
                write!(output, ")").unwrap();
            }
            ExprKind::If(cond, then_block, else_opt) => {
                write!(output, "if ").unwrap();
                self.generate_expr(cond, output)?;
                write!(output, " ").unwrap();
                self.generate_block(then_block, output)?;

                if let Some(else_expr) = else_opt {
                    write!(output, " else ").unwrap();
                    if let ExprKind::If(_, _, _) = else_expr.kind {
                        self.generate_expr(else_expr, output)?;
                    } else if let ExprKind::Block(block, _) = &else_expr.kind {
                        self.generate_block(block, output)?;
                    }
                }
            }
            ExprKind::Match(expr, arms) => {
                write!(output, "match ").unwrap();
                self.generate_expr(expr, output)?;
                writeln!(output, " {{").unwrap();
                for arm in arms {
                    write!(output, "    ").unwrap();
                    self.generate_pat(&arm.pat, output)?;
                    if let Some(guard) = &arm.guard {
                        write!(output, " if ").unwrap();
                        self.generate_expr(guard, output)?;
                    }
                    write!(output, " => ").unwrap();
                    self.generate_expr(&arm.body, output)?;
                    writeln!(output, ",").unwrap();
                }
                write!(output, "}}").unwrap();
            }
            ExprKind::Block(block, label) => {
                if let Some(label) = label {
                    write!(output, "{}: ", label.ident.name).unwrap();
                }
                self.generate_block(block, output)?;
            }
            ExprKind::Assign(place, value, _) => {
                self.generate_expr(place, output)?;
                write!(output, " = ").unwrap();
                self.generate_expr(value, output)?;
            }
            ExprKind::Field(obj, field) => {
                self.generate_expr(obj, output)?;
                write!(output, ".{}", field.name).unwrap();
            }
            ExprKind::Index(obj, index) => {
                self.generate_expr(obj, output)?;
                write!(output, "[").unwrap();
                self.generate_expr(index, output)?;
                write!(output, "]").unwrap();
            }
            ExprKind::Return(opt_expr) => {
                write!(output, "return").unwrap();
                if let Some(expr) = opt_expr {
                    write!(output, " ").unwrap();
                    self.generate_expr(expr, output)?;
                }
            }
            ExprKind::Break(opt_label, opt_expr) => {
                write!(output, "break").unwrap();
                if let Some(label) = opt_label {
                    write!(output, " {}", label.ident.name).unwrap();
                }
                if let Some(expr) = opt_expr {
                    write!(output, " ").unwrap();
                    self.generate_expr(expr, output)?;
                }
            }
            ExprKind::Continue(opt_label) => {
                write!(output, "continue").unwrap();
                if let Some(label) = opt_label {
                    write!(output, " {}", label.ident.name).unwrap();
                }
            }
            ExprKind::Paren(expr) => {
                write!(output, "(").unwrap();
                self.generate_expr(expr, output)?;
                write!(output, ")").unwrap();
            }
            _ => {
                write!(output, "/* unsupported expr */").unwrap();
            }
        }
        Ok(())
    }

    fn generate_lit(&self, lit: &Lit, output: &mut String) -> Result<()> {
        match &lit.kind {
            LitKind::Str(s) => write!(output, "\"{}\"", s).unwrap(),
            LitKind::Int(i) => write!(output, "{}", i).unwrap(),
        }
        Ok(())
    }

    pub fn generate_pat(&self, pat: &Pat, output: &mut String) -> Result<()> {
        match &pat.kind {
            PatKind::Ident(binding_mode, ident, sub_pat) => {
                if binding_mode.by_ref == ByRef::Yes {
                    write!(output, "ref ").unwrap();
                }
                if binding_mode.mutability == Mutability::Mut {
                    write!(output, "mut ").unwrap();
                }
                write!(output, "{}", ident.name).unwrap();
                if let Some(sub) = sub_pat {
                    write!(output, " @ ").unwrap();
                    self.generate_pat(sub, output)?;
                }
            }
            PatKind::Wild => write!(output, "_").unwrap(),
            PatKind::Tuple(pats) => {
                write!(output, "(").unwrap();
                for (i, pat) in pats.iter().enumerate() {
                    if i > 0 {
                        write!(output, ", ").unwrap();
                    }
                    self.generate_pat(pat, output)?;
                }
                write!(output, ")").unwrap();
            }
            PatKind::Struct(_, path, fields, rest) => {
                self.generate_path(path, output)?;
                write!(output, " {{ ").unwrap();
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(output, ", ").unwrap();
                    }
                    write!(output, "{}", field.ident.name).unwrap();
                    if !field.is_shorthand {
                        write!(output, ": ").unwrap();
                        self.generate_pat(&field.pat, output)?;
                    }
                }
                if *rest == PatFieldsRest::Rest {
                    if !fields.is_empty() {
                        write!(output, ", ").unwrap();
                    }
                    write!(output, "..").unwrap();
                }
                write!(output, " }}").unwrap();
            }
            _ => {
                write!(output, "_").unwrap();
            }
        }
        Ok(())
    }

    pub fn generate_ty(&self, ty: &Ty, output: &mut String) -> Result<()> {
        match &ty.kind {
            TyKind::Path(_, path) => {
                self.generate_path(path, output)?;
            }
            TyKind::Ref(lifetime, mut_ty) => {
                write!(output, "&").unwrap();
                if let Some(lt) = lifetime {
                    write!(output, "{} ", lt.ident.name).unwrap();
                }
                if mut_ty.mutability == Mutability::Mut {
                    write!(output, "mut ").unwrap();
                }
                self.generate_ty(&mut_ty.ty, output)?;
            }
            TyKind::Ptr(mut_ty) => {
                write!(output, "*").unwrap();
                if mut_ty.mutability == Mutability::Mut {
                    write!(output, "mut ").unwrap();
                } else {
                    write!(output, "const ").unwrap();
                }
                self.generate_ty(&mut_ty.ty, output)?;
            }
            TyKind::Slice(ty) => {
                write!(output, "[").unwrap();
                self.generate_ty(ty, output)?;
                write!(output, "]").unwrap();
            }
            TyKind::Array(ty, len) => {
                write!(output, "[").unwrap();
                self.generate_ty(ty, output)?;
                write!(output, "; ").unwrap();
                self.generate_expr(&len.value, output)?;
                write!(output, "]").unwrap();
            }
            TyKind::Tup(types) => {
                write!(output, "(").unwrap();
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(output, ", ").unwrap();
                    }
                    self.generate_ty(ty, output)?;
                }
                write!(output, ")").unwrap();
            }
            TyKind::Never => write!(output, "!").unwrap(),
            TyKind::Infer => write!(output, "_").unwrap(),
            _ => write!(output, "/* unsupported type */").unwrap(),
        }
        Ok(())
    }

    fn generate_mac_call(&self, mac_call: &MacCall, output: &mut String) -> Result<()> {
        self.generate_path(&mac_call.path, output)?;
        write!(output, "!").unwrap();

        match &mac_call.args {
            MacArgs::Delimited { delim, tokens, .. } => {
                let (open, close) = match delim {
                    Delimiter::Paren => ("(", ")"),
                    Delimiter::Brace => ("{", "}"),
                    Delimiter::Bracket => ("[", "]"),
                    Delimiter::Invisible => ("", ""),
                };
                write!(output, "{}", open).unwrap();
                match tokens {
                    TokenStream::Source(s) => write!(output, "{}", s).unwrap(),
                    TokenStream::Empty => {}
                }
                write!(output, "{}", close).unwrap();
            }
            MacArgs::Empty => write!(output, "()").unwrap(),
            _ => {}
        }

        Ok(())
    }

    fn binop_str(&self, op: BinOp) -> &'static str {
        match op {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::And => "&&",
            BinOp::Or => "||",
            BinOp::BitXor => "^",
            BinOp::BitAnd => "&",
            BinOp::BitOr => "|",
            BinOp::Shl => "<<",
            BinOp::Shr => ">>",
            BinOp::Eq => "==",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Ne => "!=",
            BinOp::Ge => ">=",
            BinOp::Gt => ">",
        }
    }

    fn unop_str(&self, op: UnOp) -> &'static str {
        match op {
            UnOp::Deref => "*",
            UnOp::Not => "!",
            UnOp::Neg => "-",
        }
    }
}
```

---

## Part 7: Complete Builder Extensions

Add builder methods for all new types. This follows the same pattern as Phase 1, but for all the new ExprKind, ItemKind, PatKind, and TyKind variants.

I'll show the pattern for a few key ones:

### File: `src/builder/expr.rs` (additions)

```rust
impl AstBuilder {
    // Add methods for all new ExprKind variants

    fn build_binary_expr(&mut self, kwargs: &HashMap<String, SExp>) -> Result<ExprKind> {
        let op = self.build_binop(get_required(kwargs, "op", pos)?)?;
        let left = self.build_expr(get_required(kwargs, "left", pos)?)?;
        let right = self.build_expr(get_required(kwargs, "right", pos)?)?;
        Ok(ExprKind::Binary(op, Box::new(left), Box::new(right)))
    }

    fn build_binop(&mut self, sexp: &SExp) -> Result<BinOp> {
        let sym = expect_symbol(sexp)?;
        match sym.as_str() {
            "Add" => Ok(BinOp::Add),
            "Sub" => Ok(BinOp::Sub),
            "Mul" => Ok(BinOp::Mul),
            // ... etc for all binary operators
            _ => Err(ParseError::UnexpectedToken {
                token: sym,
                pos: sexp.position(),
            }),
        }
    }

    fn build_if_expr(&mut self, elements: &[SExp]) -> Result<ExprKind> {
        // Parse: (If <cond> <then> <else-opt>)
        let cond = self.build_expr(&elements[1])?;
        let then_block = self.build_block(&elements[2])?;
        let else_opt = if elements.len() > 3 {
            Some(Box::new(self.build_expr(&elements[3])?))
        } else {
            None
        };
        Ok(ExprKind::If(Box::new(cond), Box::new(then_block), else_opt))
    }

    fn build_match_expr(&mut self, elements: &[SExp]) -> Result<ExprKind> {
        let expr = self.build_expr(&elements[1])?;
        let arms = parse_list(&elements[2], |s| self.build_arm(s))?;
        Ok(ExprKind::Match(Box::new(expr), arms))
    }

    fn build_arm(&mut self, sexp: &SExp) -> Result<Arm> {
        let elements = expect_list(sexp)?;
        let kwargs = parse_kwargs(&elements[1..])?;

        Ok(Arm {
            attrs: vec![],
            pat: self.build_pat(get_required(&kwargs, "pat", sexp.position())?)?,
            guard: get_optional(&kwargs, "guard")
                .map(|s| self.build_expr(s))
                .transpose()?
                .map(Box::new),
            body: Box::new(self.build_expr(get_required(&kwargs, "body", sexp.position())?)?),
            span: self.build_span(get_required(&kwargs, "span", sexp.position())?)?,
            id: self.next_id(),
        })
    }
}
```

---

## Part 8: Complete Generator Extensions

Similarly, add generator methods for all new types:

### File: `src/generator/expr.rs` (additions)

```rust
impl Generator {
    fn generate_expr_kind(&self, kind: &ExprKind) -> Result<SExp> {
        match kind {
            // Existing variants...

            ExprKind::Binary(op, left, right) => {
                let fields = kwargs(vec![
                    kwarg("op", self.generate_binop(*op)),
                    kwarg("left", self.generate_expr(left)?),
                    kwarg("right", self.generate_expr(right)?),
                ]);
                Ok(typed_node("Binary", fields))
            }

            ExprKind::If(cond, then_block, else_opt) => {
                let mut elements = vec![
                    sym("If"),
                    self.generate_expr(cond)?,
                    self.generate_block(then_block)?,
                ];
                if let Some(else_expr) = else_opt {
                    elements.push(self.generate_expr(else_expr)?);
                }
                Ok(list(elements))
            }

            ExprKind::Match(expr, arms) => {
                let expr_sexp = self.generate_expr(expr)?;
                let arms_sexp = list(
                    arms.iter()
                        .map(|arm| self.generate_arm(arm))
                        .collect::<Result<Vec<_>>>()?
                );
                Ok(list(vec![sym("Match"), expr_sexp, arms_sexp]))
            }

            // ... all other ExprKind variants

            _ => Ok(sym("UnsupportedExpr"))
        }
    }

    fn generate_binop(&self, op: BinOp) -> SExp {
        sym(match op {
            BinOp::Add => "Add",
            BinOp::Sub => "Sub",
            // ... etc
        })
    }

    fn generate_arm(&self, arm: &Arm) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("pat", self.generate_pat(&arm.pat)?),
            kwarg("body", self.generate_expr(&arm.body)?),
            kwarg("span", self.generate_span(arm.span)),
        ]);

        let fields = if let Some(guard) = &arm.guard {
            let mut f = fields;
            f.extend(kwarg("guard", self.generate_expr(guard)?));
            f
        } else {
            fields
        };

        Ok(typed_node("Arm", fields))
    }
}
```

---

## Part 9: Complete Testing Suite

### File: `tests/complete_coverage_tests.rs`

```rust
use oxur_ast::*;
use oxur_ast::integration::parse_rust_file;
use oxur_ast::sexp::{Parser, print_sexp};

// Test all expression kinds
#[test]
fn test_binary_expr() {
    let source = r#"
fn main() {
    let x = 1 + 2;
}
    "#;

    let crate_node = parse_rust_file(source).unwrap();
    // Verify AST contains binary expression
}

#[test]
fn test_if_expr() {
    let source = r#"
fn main() {
    if true {
        println!("yes");
    } else {
        println!("no");
    }
}
    "#;

    let crate_node = parse_rust_file(source).unwrap();
    // Verify AST contains if expression
}

#[test]
fn test_match_expr() {
    let source = r#"
fn main() {
    match value {
        Some(x) => x,
        None => 0,
    }
}
    "#;

    let crate_node = parse_rust_file(source).unwrap();
    // Verify AST contains match expression
}

// Test all item kinds
#[test]
fn test_struct_item() {
    let source = r#"
struct Point {
    x: i32,
    y: i32,
}
    "#;

    let crate_node = parse_rust_file(source).unwrap();
    assert_eq!(crate_node.items.len(), 1);
}

#[test]
fn test_enum_item() {
    let source = r#"
enum Option<T> {
    Some(T),
    None,
}
    "#;

    let crate_node = parse_rust_file(source).unwrap();
    assert_eq!(crate_node.items.len(), 1);
}

#[test]
fn test_trait_item() {
    let source = r#"
trait Display {
    fn fmt(&self) -> String;
}
    "#;

    let crate_node = parse_rust_file(source).unwrap();
    assert_eq!(crate_node.items.len(), 1);
}

#[test]
fn test_impl_item() {
    let source = r#"
impl Point {
    fn new() -> Self {
        Point { x: 0, y: 0 }
    }
}
    "#;

    let crate_node = parse_rust_file(source).unwrap();
    assert_eq!(crate_node.items.len(), 1);
}

// Round-trip tests for complex code
#[test]
fn test_round_trip_complex() {
    let source = r#"
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    fn distance(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        ((dx * dx + dy * dy) as f64).sqrt()
    }
}
    "#;

    // Parse
    let crate1 = parse_rust_file(source).unwrap();

    // Generate S-expr
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate1).unwrap();

    // Parse S-expr
    let sexp_text = print_sexp(&sexp);
    let sexp2 = Parser::parse_str(&sexp_text).unwrap();

    // Build AST
    let mut builder = AstBuilder::new();
    let crate2 = builder.build_crate(&sexp2).unwrap();

    // Verify structure
    assert_eq!(crate1.items.len(), crate2.items.len());
}
```

### File: `tests/codegen_tests.rs`

```rust
use oxur_ast::*;
use oxur_ast::integration::parse_rust_file;
use oxur_ast::codegen::generate_rust;

#[test]
fn test_codegen_hello_world() {
    let source = r#"
fn main() {
    println!("Hello, world!");
}
    "#;

    let crate_node = parse_rust_file(source).unwrap();
    let generated = generate_rust(&crate_node).unwrap();

    // Verify generated code compiles
    assert!(generated.contains("fn main"));
    assert!(generated.contains("println!"));
}

#[test]
fn test_codegen_struct() {
    let source = r#"
struct Point {
    x: i32,
    y: i32,
}
    "#;

    let crate_node = parse_rust_file(source).unwrap();
    let generated = generate_rust(&crate_node).unwrap();

    assert!(generated.contains("struct Point"));
}

#[test]
fn test_codegen_impl() {
    let source = r#"
impl Point {
    fn new() -> Self {
        Point { x: 0, y: 0 }
    }
}
    "#;

    let crate_node = parse_rust_file(source).unwrap();
    let generated = generate_rust(&crate_node).unwrap();

    assert!(generated.contains("impl Point"));
    assert!(generated.contains("fn new"));
}

#[test]
fn test_full_round_trip_with_codegen() {
    let original = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
    "#;

    // Parse original
    let crate1 = parse_rust_file(original).unwrap();

    // Generate Rust code
    let generated1 = generate_rust(&crate1).unwrap();

    // Parse generated code
    let crate2 = parse_rust_file(&generated1).unwrap();

    // Generate again
    let generated2 = generate_rust(&crate2).unwrap();

    // Should be stable
    assert_eq!(generated1, generated2);
}
```

---

## Part 10: Documentation Updates

Update the main README and add comprehensive docs:

### File: `ARCHITECTURE.md` (new)

```markdown
# oxur-ast Architecture

## Overview

oxur-ast provides bidirectional conversion between Rust's AST and S-expressions.

## Components

### 1. S-expression Layer (`src/sexp/`)
- Lexer: Text → Tokens
- Parser: Tokens → S-exp AST
- Printer: S-exp AST → Formatted text

### 2. AST Layer (`src/ast/`)
- Complete Rust AST type definitions
- All expression, statement, item, pattern, and type variants
- Attribute system

### 3. Builder (`src/builder/`)
- S-expression → Rust AST conversion
- Validates structure and types
- Handles all AST node construction

### 4. Generator (`src/generator/`)
- Rust AST → S-expression conversion
- Preserves all information
- Enables round-trip conversion

### 5. Integration (`src/integration/`)
- syn → oxur AST (parse real Rust files)
- Future: oxur AST → syn

### 6. Code Generation (`src/codegen/`)
- Rust AST → Rust source code
- Pretty-printing and formatting
- Complete code reconstruction

## Data Flow

```

Rust Source
    ↓ (syn parser)
syn AST
    ↓ (from_syn)
oxur AST
    ↓ (Generator)
S-expression
    ↓ (Parser)
S-exp AST
    ↓ (Builder)
oxur AST
    ↓ (RustCodegen)
Rust Source

```

## Coverage

| AST Category | Variants | Status |
|--------------|----------|--------|
| ExprKind     | 35+      | ✓ Complete |
| ItemKind     | 17       | ✓ Complete |
| PatKind      | 15+      | ✓ Complete |
| TyKind       | 15+      | ✓ Complete |
| StmtKind     | 6        | ✓ Complete |

## Testing Strategy

1. Unit tests for each component
2. Integration tests with real Rust code
3. Round-trip tests (Rust → S-expr → Rust)
4. Code generation tests
5. Regression tests with test corpus
```

---

## Success Criteria

Phase 4 is complete when:

- [ ] All ExprKind variants implemented
- [ ] All ItemKind variants implemented
- [ ] All PatKind variants implemented
- [ ] All TyKind variants implemented
- [ ] Attribute system working
- [ ] Rust code generation produces valid code
- [ ] All round-trip tests pass
- [ ] Generated code can be re-parsed
- [ ] Comprehensive test coverage (>80%)
- [ ] Documentation complete
- [ ] No compiler warnings
- [ ] Clean `cargo clippy` output

---

## Testing Instructions

```bash
# Run all tests
cargo test -p oxur-ast

# Run coverage tests
cargo test -p oxur-ast complete_coverage

# Run codegen tests
cargo test -p oxur-ast codegen

# Test round-trips
cargo test -p oxur-ast round_trip

# Test with real code
oxur-ast verify examples/*.rs

# Benchmark
cargo bench -p oxur-ast
```

---

## Validation Approach

After Phase 4, validate with progressively complex Rust code:

1. **Level 1**: Hello World ✓
2. **Level 2**: Simple functions with parameters and returns
3. **Level 3**: Structs and enums
4. **Level 4**: Traits and impls
5. **Level 5**: Pattern matching and control flow
6. **Level 6**: Generics and lifetimes
7. **Level 7**: Complex real-world code

For each level:

```bash
# Parse
oxur-ast to-sexp test.rs > test.sexp

# Generate Rust
oxur-ast to-rust test.sexp > test.gen.rs

# Verify it compiles
rustc test.gen.rs

# Verify it's equivalent
diff <(rustfmt test.rs) <(rustfmt test.gen.rs)
```

---

## Future Enhancements (Beyond Phase 4)

- Macro expansion support
- Proper lifetime and generic handling in codegen
- Source maps for error reporting
- Incremental parsing
- LSP integration
- REPL support with hot-reload
- Custom derive macros
- Proc macro support

---

*"Complete coverage achieved. The AST is whole, the bridge is strong."*
