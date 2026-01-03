---
number: 20
title: "oxur-ast Phase 5: Complete Pattern & Type System Coverage"
author: "implementing the"
created: 2025-12-31
updated: 2026-01-03
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# oxur-ast Phase 5: Complete Pattern & Type System Coverage

**Phase**: 5 - Complete AST Foundation
**Goal**: Finish pattern matching and type system coverage for production readiness
**Estimated Time**: 3-4 days (25-30 hours)
**Prerequisites**: Phase 4 complete (60-70% AST coverage achieved)

---

## Executive Summary

Phase 5 completes the foundational AST coverage by implementing the two critical gaps identified in Phase 4:

1. **Pattern Matching**: Currently 25% → Target 100% (15+ variants)
2. **Type System**: Currently 25% → Target 100% (15+ variants)

Additionally, we'll fill in commonly-used expression variants and complete code generation for all implemented AST nodes.

**Why This is a Separate Phase:**

Phase 4 successfully implemented 60-70% of the complete AST, delivering:
- 10/17 ItemKind variants (59%)
- 20+/35+ ExprKind variants (55%)
- 6/6 StmtKind variants (100%)
- Complete infrastructure (builder, generator, codegen, CLI)

However, two critical gaps prevent full production use:
- **Patterns**: Essential for match expressions, function parameters, destructuring
- **Types**: Required for complex type annotations, generics, trait bounds

Phase 5 focuses exclusively on completing these foundations rather than expanding into advanced features.

---

## Table of Contents

1. [Current State Assessment](#1-current-state-assessment)
2. [Phase 5 Scope](#2-phase-5-scope)
3. [Pattern System Implementation](#3-pattern-system-implementation)
4. [Type System Implementation](#4-type-system-implementation)
5. [Expression Gap Filling](#5-expression-gap-filling)
6. [Code Generation Completion](#6-code-generation-completion)
7. [Testing Strategy](#7-testing-strategy)
8. [Success Criteria](#8-success-criteria)
9. [Implementation Roadmap](#9-implementation-roadmap)

---

## 1. Current State Assessment

### What Works Well ✅

From Phase 4 completion:
- **Infrastructure**: All layers complete (sexp, ast, builder, generator, codegen, integration)
- **Items**: Structs, enums, traits, impls, functions, use statements (10/17 variants)
- **Control Flow**: if, match, while, for, loop all working
- **Operators**: All binary and unary operators (14 + 3 variants)
- **Statements**: Complete coverage (6/6 variants)
- **Basic Patterns**: Identifiers and wildcards work
- **Basic Types**: Path types and references work

### Critical Gaps ⚠️

**Pattern Matching (3-4 of 15+ = 25%)**

Currently implemented:
- `PatIdent` - Variable binding (`x`, `mut x`, `ref x`)
- `PatWild` - Wildcard (`_`)
- `PatRange` - Range patterns (partial)

Missing:
- `PatLit` - Literal patterns (`1`, `"hello"`, `true`)
- `PatTuple` - Tuple patterns (`(a, b, c)`)
- `PatStruct` - Struct patterns (`Point { x, y }`)
- `PatPath` - Path patterns (`Option::None`)
- `PatSlice` - Slice patterns (`[first, .., last]`)
- `PatOr` - Or patterns (`A | B | C`)
- `PatRef` - Reference patterns (`&x`, `&mut x`)
- `PatBox` - Box patterns (`box x`)
- `PatRest` - Rest patterns (`..`)
- `PatTupleStruct` - Tuple struct patterns (`Some(x)`)
- `PatType` - Type ascription (`x: i32`)
- `PatConst` - Const block patterns
- `PatMacro` - Macro patterns
- `PatParen` - Parenthesized patterns

**Type System (3-4 of 15+ = 25%)**

Currently implemented:
- `TypePath` - Path types (`Vec<T>`, `std::vec::Vec`)
- `TypeReference` - Reference types (`&T`, `&mut T`)
- `TypePtr` - Raw pointer types (`*const T`, `*mut T`)

Missing:
- `TypeArray` - Array types (`[T; N]`)
- `TypeSlice` - Slice types (`[T]`)
- `TypeTuple` - Tuple types (`(A, B, C)`)
- `TypeBareFn` - Function pointer types (`fn(T) -> U`)
- `TypeNever` - Never type (`!`)
- `TypeInfer` - Inference placeholder (`_`)
- `TypeImplTrait` - Impl trait (`impl Trait`)
- `TypeTraitObject` - Trait objects (`dyn Trait`)
- `TypeMacro` - Macro types
- `TypeGroup` - Grouped types
- `TypeParen` - Parenthesized types

**Common Expression Gaps**
- `ExprParen` - Parenthesized expressions (very common)
- `ExprTry` - Try operator `?` (essential for error handling)
- `ExprCast` - Type casts `as` (common)
- `ExprBreak`, `ExprContinue`, `ExprReturn` - Control flow with values

---

## 2. Phase 5 Scope

### Priority 1: Pattern Matching (HIGH) - 6-8 hours

**Goal**: Implement all 15+ PatKind variants for complete pattern matching support

**Rationale**: Patterns are used everywhere in Rust:
- Match expressions (our existing `ExprMatch` needs full patterns)
- Function parameters (`fn foo((x, y): (i32, i32))`)
- Let bindings (`let Some(x) = opt`)
- For loop iteration (`for (key, value) in map`)

**Deliverables**:
1. Pattern AST types (`src/ast/pat.rs`)
2. Pattern builder (`src/builder/pat.rs`)
3. Pattern generator (`src/generator/pat.rs`)
4. Pattern code generation (`src/codegen/pat.rs`)
5. Comprehensive pattern tests

### Priority 2: Type System (HIGH) - 5-7 hours

**Goal**: Implement all 15+ TypeKind variants for complete type annotation support

**Rationale**: Types are fundamental to Rust:
- Function signatures (`fn process(data: [u8; 32]) -> Result<(), Box<dyn Error>>`)
- Struct fields (`data: Vec<T>`)
- Type aliases (`type Handler = fn(Event) -> bool`)
- Generic bounds (`T: Iterator<Item = &str>`)

**Deliverables**:
1. Type AST types (extend `src/ast/types.rs`)
2. Type builder (`src/builder/ty.rs`)
3. Type generator (`src/generator/ty.rs`)
4. Type code generation (`src/codegen/ty.rs`)
5. Comprehensive type tests

### Priority 3: Expression Gap Filling (MEDIUM) - 4-6 hours

**Goal**: Add the most commonly-used missing expression types

**Focus on**:
- `ExprParen` - Required for precedence
- `ExprTry` - Essential for `?` operator (very common in modern Rust)
- `ExprCast` - Type casting with `as`
- `ExprBreak`/`ExprContinue`/`ExprReturn` - Control flow with optional values

**Deliverables**:
1. AST types (in `src/ast/expr.rs`)
2. Builder support (in `src/builder/expr.rs`)
3. Generator support (in `src/generator/expr.rs`)
4. Code generation (in `src/codegen/expr.rs`)
5. Tests for each variant

### Priority 4: Code Generation Completion (MEDIUM) - 3-4 hours

**Goal**: Ensure all implemented AST types can generate valid Rust code

**Current state**: Some AST types are built but have incomplete codegen with `todo!()` or comments

**Deliverables**:
1. Complete codegen for all implemented expressions
2. Complete codegen for all implemented items
3. Complete codegen for patterns (from Priority 1)
4. Complete codegen for types (from Priority 2)
5. Round-trip tests verifying generated code compiles

### Priority 5: Testing & Documentation (MEDIUM) - 5-6 hours

**Goal**: Ensure quality and maintainability

**Deliverables**:
1. Test coverage for all new pattern types
2. Test coverage for all new type variants
3. Round-trip tests with complex real-world code
4. Update ARCHITECTURE.md with pattern/type handling
5. Update Phase 4 status to "Complete with Notes"
6. Create Phase 5 completion report

---

## 3. Pattern System Implementation

### File Structure

Create dedicated pattern module:

```
oxur-ast/src/ast/
└── pat.rs              # NEW: Pattern type definitions

oxur-ast/src/builder/
└── pat.rs              # NEW: Pattern building

oxur-ast/src/generator/
└── pat.rs              # NEW: Pattern generation

oxur-ast/src/codegen/
└── pat.rs              # NEW: Pattern code generation
```

### Part 3.1: Pattern AST Types

**File**: `src/ast/pat.rs`

```rust
use crate::ast::{Ident, Path, Expr, Ty, NodeId, Span, AttrVec};

/// Pattern - appears in match arms, let bindings, function parameters
#[derive(Debug, Clone, PartialEq)]
pub struct Pat {
    pub id: NodeId,
    pub kind: PatKind,
    pub span: Span,
    pub tokens: Option<TokenStream>,
}

/// Pattern kinds - COMPLETE COVERAGE
#[derive(Debug, Clone, PartialEq)]
pub enum PatKind {
    /// Wildcard pattern: `_`
    Wild,

    /// Identifier pattern: `x`, `mut x`, `ref x`, `ref mut x`
    Ident(BindingMode, Ident, Option<Box<Pat>>),

    /// Struct pattern: `Variant { x, y, .. }`
    Struct(Option<QSelf>, Path, Vec<FieldPat>, PatFieldsRest),

    /// Tuple struct pattern: `Variant(x, y, ..)`
    TupleStruct(Option<QSelf>, Path, Vec<Pat>),

    /// Or-pattern: `A | B | C`
    Or(Vec<Pat>),

    /// Path pattern: `None`, `Some`, `std::option::Option::None`
    Path(Option<QSelf>, Path),

    /// Tuple pattern: `(a, b)`
    Tuple(Vec<Pat>),

    /// Box pattern: `box pat`
    Box(Box<Pat>),

    /// Reference pattern: `&pat` or `&mut pat`
    Ref(Box<Pat>, Mutability),

    /// Literal pattern: `1`, `"hello"`, `true`, `'a'`
    Lit(Box<Expr>),

    /// Range pattern: `1..=5`, `..=10`, `5..`
    Range(Option<Box<Expr>>, Option<Box<Expr>>, RangeEnd),

    /// Slice pattern: `[a, b, rest @ ..]`
    Slice(Vec<Pat>),

    /// Rest pattern: `..` in a tuple or slice
    Rest,

    /// Parenthesized pattern: `(pat)`
    Paren(Box<Pat>),

    /// Type ascription: `x: i32`
    Type(Box<Pat>, Box<Ty>),

    /// Const block: `const { expr }`
    Const(Box<ConstBlock>),

    /// Macro invocation
    MacCall(Box<MacCall>),

    /// Error recovery
    Err,
}

/// Binding mode for identifier patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingMode {
    /// `x` or `mut x`
    ByValue(Mutability),
    /// `ref x` or `ref mut x`
    ByRef(Mutability),
}

/// Mutability
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Mut,
    Not,
}

/// Field pattern in struct patterns
#[derive(Debug, Clone, PartialEq)]
pub struct FieldPat {
    pub attrs: AttrVec,
    pub id: NodeId,
    pub ident: Ident,
    pub pat: Box<Pat>,
    pub is_shorthand: bool,
    pub span: Span,
}

/// Whether struct pattern has `..` rest
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatFieldsRest {
    /// Has `..`
    Rest,
    /// No `..`
    None,
}

/// Range end style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeEnd {
    /// `..=`
    Included,
    /// `..`
    Excluded,
}

/// Const block in pattern position
#[derive(Debug, Clone, PartialEq)]
pub struct ConstBlock {
    pub id: NodeId,
    pub value: Box<Expr>,
}

// Convenience constructors
impl Pat {
    pub fn new(kind: PatKind, span: Span) -> Self {
        Self {
            id: NodeId::new(),
            kind,
            span,
            tokens: None,
        }
    }

    pub fn wild(span: Span) -> Self {
        Self::new(PatKind::Wild, span)
    }

    pub fn ident(name: Ident, span: Span) -> Self {
        Self::new(
            PatKind::Ident(
                BindingMode::ByValue(Mutability::Not),
                name,
                None,
            ),
            span,
        )
    }
}
```

**S-expression Representation**:

```lisp
;; Wildcard
(Pat :id 1 :kind (Wild) :span ...)

;; Identifier
(Pat :id 2 :kind (Ident
  :mode (ByValue Mut)
  :ident (Ident :name "x")
  :subpat nil))

;; Struct pattern
(Pat :id 3 :kind (Struct
  :qself nil
  :path (Path ...)
  :fields ((FieldPat :ident ... :pat ...) ...)
  :rest Rest))

;; Tuple pattern
(Pat :id 4 :kind (Tuple
  :pats ((Pat ...) (Pat ...) (Pat ...))))

;; Or pattern
(Pat :id 5 :kind (Or
  :pats ((Pat :kind (Lit ...)) (Pat :kind (Lit ...)))))

;; Reference pattern
(Pat :id 6 :kind (Ref
  :pat (Pat :kind (Ident ...))
  :mutability Mut))

;; Range pattern
(Pat :id 7 :kind (Range
  :start (Some (Expr :kind (Lit ...)))
  :end (Some (Expr :kind (Lit ...)))
  :limits Included))

;; Slice pattern
(Pat :id 8 :kind (Slice
  :pats ((Pat ...) (Pat :kind (Rest)) (Pat ...))))
```

### Part 3.2: Pattern Builder

**File**: `src/builder/pat.rs`

```rust
use crate::ast::{Pat, PatKind, BindingMode, Mutability, FieldPat, PatFieldsRest, RangeEnd};
use crate::builder::{AstBuilder, BuildError};
use crate::sexp::SExp;

impl AstBuilder {
    /// Build a Pat from S-expression
    pub fn build_pat(&mut self, sexp: &SExp) -> Result<Pat, BuildError> {
        let list = self.expect_list(sexp, "Pat")?;
        let mut id = None;
        let mut kind = None;
        let mut span = None;

        // Parse keyword arguments
        let mut i = 1; // Skip node name
        while i < list.elements.len() {
            let kw = self.expect_keyword(&list.elements[i])?;
            i += 1;

            match kw.as_str() {
                "id" => id = Some(self.build_node_id(&list.elements[i])?),
                "kind" => kind = Some(self.build_pat_kind(&list.elements[i])?),
                "span" => span = Some(self.build_span(&list.elements[i])?),
                "tokens" => { /* Skip for now */ }
                _ => return Err(BuildError::UnknownField {
                    node: "Pat".to_string(),
                    field: kw,
                }),
            }
            i += 1;
        }

        Ok(Pat {
            id: id.ok_or_else(|| BuildError::MissingField {
                node: "Pat".to_string(),
                field: "id".to_string(),
            })?,
            kind: kind.ok_or_else(|| BuildError::MissingField {
                node: "Pat".to_string(),
                field: "kind".to_string(),
            })?,
            span: span.unwrap_or_default(),
            tokens: None,
        })
    }

    fn build_pat_kind(&mut self, sexp: &SExp) -> Result<PatKind, BuildError> {
        let list = self.expect_list(sexp, "PatKind")?;
        let variant = self.expect_symbol(&list.elements[0])?;

        match variant.as_str() {
            "Wild" => Ok(PatKind::Wild),

            "Ident" => {
                let mode = self.build_binding_mode(&self.get_field(&list, "mode")?)?;
                let ident = self.build_ident(&self.get_field(&list, "ident")?)?;
                let subpat = match self.get_optional_field(&list, "subpat")? {
                    Some(sexp) => Some(Box::new(self.build_pat(sexp)?)),
                    None => None,
                };
                Ok(PatKind::Ident(mode, ident, subpat))
            }

            "Struct" => {
                let qself = self.build_optional_qself(&self.get_field(&list, "qself")?)?;
                let path = self.build_path(&self.get_field(&list, "path")?)?;
                let fields = self.build_field_pats(&self.get_field(&list, "fields")?)?;
                let rest = self.build_pat_fields_rest(&self.get_field(&list, "rest")?)?;
                Ok(PatKind::Struct(qself, path, fields, rest))
            }

            "TupleStruct" => {
                let qself = self.build_optional_qself(&self.get_field(&list, "qself")?)?;
                let path = self.build_path(&self.get_field(&list, "path")?)?;
                let pats = self.build_pat_list(&self.get_field(&list, "pats")?)?;
                Ok(PatKind::TupleStruct(qself, path, pats))
            }

            "Tuple" => {
                let pats = self.build_pat_list(&self.get_field(&list, "pats")?)?;
                Ok(PatKind::Tuple(pats))
            }

            "Or" => {
                let pats = self.build_pat_list(&self.get_field(&list, "pats")?)?;
                Ok(PatKind::Or(pats))
            }

            "Path" => {
                let qself = self.build_optional_qself(&self.get_field(&list, "qself")?)?;
                let path = self.build_path(&self.get_field(&list, "path")?)?;
                Ok(PatKind::Path(qself, path))
            }

            "Box" => {
                let pat = Box::new(self.build_pat(&self.get_field(&list, "pat")?)?);
                Ok(PatKind::Box(pat))
            }

            "Ref" => {
                let pat = Box::new(self.build_pat(&self.get_field(&list, "pat")?)?);
                let mutability = self.build_mutability(&self.get_field(&list, "mutability")?)?;
                Ok(PatKind::Ref(pat, mutability))
            }

            "Lit" => {
                let expr = Box::new(self.build_expr(&self.get_field(&list, "lit")?)?);
                Ok(PatKind::Lit(expr))
            }

            "Range" => {
                let start = match self.get_optional_field(&list, "start")? {
                    Some(sexp) => Some(Box::new(self.build_expr(sexp)?)),
                    None => None,
                };
                let end = match self.get_optional_field(&list, "end")? {
                    Some(sexp) => Some(Box::new(self.build_expr(sexp)?)),
                    None => None,
                };
                let limits = self.build_range_end(&self.get_field(&list, "limits")?)?;
                Ok(PatKind::Range(start, end, limits))
            }

            "Slice" => {
                let pats = self.build_pat_list(&self.get_field(&list, "pats")?)?;
                Ok(PatKind::Slice(pats))
            }

            "Rest" => Ok(PatKind::Rest),

            "Paren" => {
                let pat = Box::new(self.build_pat(&self.get_field(&list, "pat")?)?);
                Ok(PatKind::Paren(pat))
            }

            "Type" => {
                let pat = Box::new(self.build_pat(&self.get_field(&list, "pat")?)?);
                let ty = Box::new(self.build_ty(&self.get_field(&list, "ty")?)?);
                Ok(PatKind::Type(pat, ty))
            }

            "MacCall" => {
                let mac = Box::new(self.build_mac_call(&self.get_field(&list, "mac")?)?);
                Ok(PatKind::MacCall(mac))
            }

            _ => Err(BuildError::UnsupportedVariant {
                enum_name: "PatKind".to_string(),
                variant: variant,
            }),
        }
    }

    fn build_binding_mode(&mut self, sexp: &SExp) -> Result<BindingMode, BuildError> {
        let list = self.expect_list(sexp, "BindingMode")?;
        let variant = self.expect_symbol(&list.elements[0])?;

        match variant.as_str() {
            "ByValue" => {
                let mutability = self.build_mutability(&list.elements[1])?;
                Ok(BindingMode::ByValue(mutability))
            }
            "ByRef" => {
                let mutability = self.build_mutability(&list.elements[1])?;
                Ok(BindingMode::ByRef(mutability))
            }
            _ => Err(BuildError::UnsupportedVariant {
                enum_name: "BindingMode".to_string(),
                variant,
            }),
        }
    }

    fn build_mutability(&mut self, sexp: &SExp) -> Result<Mutability, BuildError> {
        let sym = self.expect_symbol(sexp)?;
        match sym.as_str() {
            "Mut" => Ok(Mutability::Mut),
            "Not" => Ok(Mutability::Not),
            _ => Err(BuildError::InvalidValue {
                expected: "Mut or Not".to_string(),
                found: sym,
            }),
        }
    }

    fn build_pat_list(&mut self, sexp: &SExp) -> Result<Vec<Pat>, BuildError> {
        let list = self.expect_list(sexp, "pattern list")?;
        list.elements.iter().map(|e| self.build_pat(e)).collect()
    }

    fn build_field_pats(&mut self, sexp: &SExp) -> Result<Vec<FieldPat>, BuildError> {
        let list = self.expect_list(sexp, "field patterns")?;
        list.elements.iter().map(|e| self.build_field_pat(e)).collect()
    }

    fn build_field_pat(&mut self, sexp: &SExp) -> Result<FieldPat, BuildError> {
        // Similar to build_pat but for FieldPat structure
        todo!("Implement field pattern building")
    }

    fn build_pat_fields_rest(&mut self, sexp: &SExp) -> Result<PatFieldsRest, BuildError> {
        let sym = self.expect_symbol(sexp)?;
        match sym.as_str() {
            "Rest" => Ok(PatFieldsRest::Rest),
            "None" => Ok(PatFieldsRest::None),
            _ => Err(BuildError::InvalidValue {
                expected: "Rest or None".to_string(),
                found: sym,
            }),
        }
    }

    fn build_range_end(&mut self, sexp: &SExp) -> Result<RangeEnd, BuildError> {
        let sym = self.expect_symbol(sexp)?;
        match sym.as_str() {
            "Included" => Ok(RangeEnd::Included),
            "Excluded" => Ok(RangeEnd::Excluded),
            _ => Err(BuildError::InvalidValue {
                expected: "Included or Excluded".to_string(),
                found: sym,
            }),
        }
    }
}
```

### Part 3.3: Pattern Generator

**File**: `src/generator/pat.rs`

```rust
use crate::ast::{Pat, PatKind, BindingMode, Mutability, PatFieldsRest, RangeEnd};
use crate::generator::{Generator, GeneratorError};
use crate::sexp::{SExp, List};

impl Generator {
    pub fn generate_pat(&self, pat: &Pat) -> Result<SExp, GeneratorError> {
        let mut fields = vec![
            self.kw("id"),
            self.num(pat.id.as_u32()),
            self.kw("kind"),
            self.generate_pat_kind(&pat.kind)?,
            self.kw("span"),
            self.generate_span(&pat.span),
        ];

        Ok(self.list_with("Pat", fields))
    }

    fn generate_pat_kind(&self, kind: &PatKind) -> Result<SExp, GeneratorError> {
        match kind {
            PatKind::Wild => Ok(self.list_with("Wild", vec![])),

            PatKind::Ident(mode, ident, subpat) => {
                let mut fields = vec![
                    self.kw("mode"),
                    self.generate_binding_mode(*mode),
                    self.kw("ident"),
                    self.generate_ident(ident),
                    self.kw("subpat"),
                ];

                match subpat {
                    Some(pat) => fields.push(self.generate_pat(pat)?),
                    None => fields.push(self.nil()),
                }

                Ok(self.list_with("Ident", fields))
            }

            PatKind::Struct(qself, path, fields, rest) => {
                Ok(self.list_with("Struct", vec![
                    self.kw("qself"),
                    self.generate_optional_qself(qself.as_ref()),
                    self.kw("path"),
                    self.generate_path(path),
                    self.kw("fields"),
                    self.generate_field_pats(fields)?,
                    self.kw("rest"),
                    self.generate_pat_fields_rest(*rest),
                ]))
            }

            PatKind::TupleStruct(qself, path, pats) => {
                Ok(self.list_with("TupleStruct", vec![
                    self.kw("qself"),
                    self.generate_optional_qself(qself.as_ref()),
                    self.kw("path"),
                    self.generate_path(path),
                    self.kw("pats"),
                    self.generate_pat_list(pats)?,
                ]))
            }

            PatKind::Tuple(pats) => {
                Ok(self.list_with("Tuple", vec![
                    self.kw("pats"),
                    self.generate_pat_list(pats)?,
                ]))
            }

            PatKind::Or(pats) => {
                Ok(self.list_with("Or", vec![
                    self.kw("pats"),
                    self.generate_pat_list(pats)?,
                ]))
            }

            PatKind::Path(qself, path) => {
                Ok(self.list_with("Path", vec![
                    self.kw("qself"),
                    self.generate_optional_qself(qself.as_ref()),
                    self.kw("path"),
                    self.generate_path(path),
                ]))
            }

            PatKind::Box(pat) => {
                Ok(self.list_with("Box", vec![
                    self.kw("pat"),
                    self.generate_pat(pat)?,
                ]))
            }

            PatKind::Ref(pat, mutability) => {
                Ok(self.list_with("Ref", vec![
                    self.kw("pat"),
                    self.generate_pat(pat)?,
                    self.kw("mutability"),
                    self.generate_mutability(*mutability),
                ]))
            }

            PatKind::Lit(expr) => {
                Ok(self.list_with("Lit", vec![
                    self.kw("lit"),
                    self.generate_expr(expr)?,
                ]))
            }

            PatKind::Range(start, end, limits) => {
                Ok(self.list_with("Range", vec![
                    self.kw("start"),
                    match start {
                        Some(e) => self.generate_expr(e)?,
                        None => self.nil(),
                    },
                    self.kw("end"),
                    match end {
                        Some(e) => self.generate_expr(e)?,
                        None => self.nil(),
                    },
                    self.kw("limits"),
                    self.generate_range_end(*limits),
                ]))
            }

            PatKind::Slice(pats) => {
                Ok(self.list_with("Slice", vec![
                    self.kw("pats"),
                    self.generate_pat_list(pats)?,
                ]))
            }

            PatKind::Rest => Ok(self.list_with("Rest", vec![])),

            PatKind::Paren(pat) => {
                Ok(self.list_with("Paren", vec![
                    self.kw("pat"),
                    self.generate_pat(pat)?,
                ]))
            }

            PatKind::Type(pat, ty) => {
                Ok(self.list_with("Type", vec![
                    self.kw("pat"),
                    self.generate_pat(pat)?,
                    self.kw("ty"),
                    self.generate_ty(ty)?,
                ]))
            }

            PatKind::MacCall(mac) => {
                Ok(self.list_with("MacCall", vec![
                    self.kw("mac"),
                    self.generate_mac_call(mac)?,
                ]))
            }

            PatKind::Const(_) => {
                // Implement when needed
                Ok(self.sym("TODO_PatConst"))
            }

            PatKind::Err => Ok(self.sym("Err")),
        }
    }

    fn generate_binding_mode(&self, mode: BindingMode) -> SExp {
        match mode {
            BindingMode::ByValue(mutability) => {
                self.list_with("ByValue", vec![self.generate_mutability(mutability)])
            }
            BindingMode::ByRef(mutability) => {
                self.list_with("ByRef", vec![self.generate_mutability(mutability)])
            }
        }
    }

    fn generate_mutability(&self, mutability: Mutability) -> SExp {
        match mutability {
            Mutability::Mut => self.sym("Mut"),
            Mutability::Not => self.sym("Not"),
        }
    }

    fn generate_pat_list(&self, pats: &[Pat]) -> Result<SExp, GeneratorError> {
        let sexps: Result<Vec<_>, _> = pats.iter()
            .map(|p| self.generate_pat(p))
            .collect();
        Ok(SExp::List(List::new(sexps?, Position::default())))
    }

    fn generate_pat_fields_rest(&self, rest: PatFieldsRest) -> SExp {
        match rest {
            PatFieldsRest::Rest => self.sym("Rest"),
            PatFieldsRest::None => self.sym("None"),
        }
    }

    fn generate_range_end(&self, end: RangeEnd) -> SExp {
        match end {
            RangeEnd::Included => self.sym("Included"),
            RangeEnd::Excluded => self.sym("Excluded"),
        }
    }
}
```

### Part 3.4: Pattern Tests

**File**: `tests/builder_pat_comprehensive_tests.rs`

```rust
use oxur_ast::ast::{Pat, PatKind, BindingMode, Mutability};
use oxur_ast::builder::AstBuilder;
use oxur_ast::sexp::Parser;

#[test]
fn test_wildcard_pattern() {
    let sexp = r#"(Pat :id 1 :kind (Wild) :span (Span :lo 0 :hi 1))"#;
    let parsed = Parser::parse_str(sexp).unwrap();
    
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&parsed).unwrap();
    
    assert!(matches!(pat.kind, PatKind::Wild));
}

#[test]
fn test_ident_pattern() {
    let sexp = r#"
    (Pat :id 1
      :kind (Ident
        :mode (ByValue Not)
        :ident (Ident :name "x" :span (Span :lo 0 :hi 1))
        :subpat nil)
      :span (Span :lo 0 :hi 1))
    "#;
    
    let parsed = Parser::parse_str(sexp).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&parsed).unwrap();
    
    match pat.kind {
        PatKind::Ident(mode, ident, subpat) => {
            assert!(matches!(mode, BindingMode::ByValue(Mutability::Not)));
            assert_eq!(ident.name, "x");
            assert!(subpat.is_none());
        }
        _ => panic!("Expected Ident pattern"),
    }
}

#[test]
fn test_tuple_pattern() {
    let sexp = r#"
    (Pat :id 1
      :kind (Tuple
        :pats (
          (Pat :id 2 :kind (Wild) :span (Span :lo 1 :hi 2))
          (Pat :id 3 :kind (Wild) :span (Span :lo 4 :hi 5))))
      :span (Span :lo 0 :hi 6))
    "#;
    
    let parsed = Parser::parse_str(sexp).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&parsed).unwrap();
    
    match pat.kind {
        PatKind::Tuple(pats) => {
            assert_eq!(pats.len(), 2);
        }
        _ => panic!("Expected Tuple pattern"),
    }
}

#[test]
fn test_or_pattern() {
    let sexp = r#"
    (Pat :id 1
      :kind (Or
        :pats (
          (Pat :id 2 :kind (Wild) :span (Span :lo 0 :hi 1))
          (Pat :id 3 :kind (Wild) :span (Span :lo 4 :hi 5))))
      :span (Span :lo 0 :hi 5))
    "#;
    
    let parsed = Parser::parse_str(sexp).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&parsed).unwrap();
    
    match pat.kind {
        PatKind::Or(pats) => {
            assert_eq!(pats.len(), 2);
        }
        _ => panic!("Expected Or pattern"),
    }
}

#[test]
fn test_ref_pattern() {
    let sexp = r#"
    (Pat :id 1
      :kind (Ref
        :pat (Pat :id 2 :kind (Wild) :span (Span :lo 1 :hi 2))
        :mutability Mut)
      :span (Span :lo 0 :hi 2))
    "#;
    
    let parsed = Parser::parse_str(sexp).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&parsed).unwrap();
    
    match pat.kind {
        PatKind::Ref(inner, mutability) => {
            assert!(matches!(mutability, Mutability::Mut));
        }
        _ => panic!("Expected Ref pattern"),
    }
}

#[test]
fn test_range_pattern() {
    let sexp = r#"
    (Pat :id 1
      :kind (Range
        :start (Some (Expr :id 2 :kind (Lit (LitInt :value "1")) :span ...))
        :end (Some (Expr :id 3 :kind (Lit (LitInt :value "10")) :span ...))
        :limits Included)
      :span (Span :lo 0 :hi 6))
    "#;
    
    // Test range pattern building
    // ...
}

#[test]
fn test_slice_pattern() {
    let sexp = r#"
    (Pat :id 1
      :kind (Slice
        :pats (
          (Pat :id 2 :kind (Wild) :span ...)
          (Pat :id 3 :kind (Rest) :span ...)
          (Pat :id 4 :kind (Wild) :span ...)))
      :span ...)
    "#;
    
    // Test slice pattern with rest
    // ...
}
```

---

## 4. Type System Implementation

### Part 4.1: Type AST Extensions

**File**: `src/ast/types.rs` (extend existing)

```rust
/// Type - appears in function signatures, struct fields, type aliases
#[derive(Debug, Clone, PartialEq)]
pub struct Ty {
    pub id: NodeId,
    pub kind: TyKind,
    pub span: Span,
    pub tokens: Option<TokenStream>,
}

/// Type kinds - COMPLETE COVERAGE
#[derive(Debug, Clone, PartialEq)]
pub enum TyKind {
    /// Slice type: `[T]`
    Slice(Box<Ty>),

    /// Array type: `[T; N]`
    Array(Box<Ty>, Box<AnonConst>),

    /// Raw pointer type: `*const T` or `*mut T`
    Ptr(Box<MutTy>),

    /// Reference type: `&'a T` or `&'a mut T`
    Rptr(Option<Lifetime>, Box<MutTy>),

    /// Bare function type: `fn(usize) -> bool`
    BareFn(Box<BareFnTy>),

    /// Never type: `!`
    Never,

    /// Tuple type: `(A, B, C, D)`
    Tup(Vec<Ty>),

    /// Path type: `std::vec::Vec<T>`
    Path(Option<QSelf>, Path),

    /// Trait object type: `dyn Trait + Send`
    TraitObject(Vec<GenericBound>, TraitObjectSyntax),

    /// Impl trait type: `impl Trait + Send`
    ImplTrait(NodeId, Vec<GenericBound>),

    /// Parenthesized type: `(T)`
    Paren(Box<Ty>),

    /// Typeof type (unstable): `typeof(expr)`
    Typeof(Box<AnonConst>),

    /// Inferred type: `_`
    Infer,

    /// Impl trait type in trait bounds: `impl Trait`
    ImplTraitPlaceholder(NodeId),

    /// Macro invocation
    MacCall(Box<MacCall>),

    /// Error recovery
    Err,
}

/// Mutable type for pointers and references
#[derive(Debug, Clone, PartialEq)]
pub struct MutTy {
    pub ty: Ty,
    pub mutbl: Mutability,
}

/// Bare function type
#[derive(Debug, Clone, PartialEq)]
pub struct BareFnTy {
    pub unsafety: Safety,
    pub ext: Extern,
    pub generic_params: Vec<GenericParam>,
    pub decl: FnDecl,
}

/// Trait object syntax
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraitObjectSyntax {
    /// `dyn Trait`
    Dyn,
    /// Just `Trait` (old syntax, deprecated)
    None,
}

/// Generic bound (trait bound or lifetime)
#[derive(Debug, Clone, PartialEq)]
pub enum GenericBound {
    /// Trait bound: `Trait + 'a`
    Trait(PolyTraitRef, TraitBoundModifier),
    /// Lifetime bound: `'a`
    Outlives(Lifetime),
}

/// Polymorphic trait reference
#[derive(Debug, Clone, PartialEq)]
pub struct PolyTraitRef {
    pub bound_generic_params: Vec<GenericParam>,
    pub trait_ref: TraitRef,
    pub span: Span,
}

/// Trait reference
#[derive(Debug, Clone, PartialEq)]
pub struct TraitRef {
    pub path: Path,
    pub ref_id: NodeId,
}

/// Trait bound modifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraitBoundModifier {
    /// `?Trait`
    Maybe,
    /// `Trait`
    None,
    /// `~const Trait`
    MaybeConst,
}

/// Lifetime
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lifetime {
    pub id: NodeId,
    pub ident: Ident,
}
```

**S-expression Examples**:

```lisp
;; Slice type: [T]
(Ty :id 1 :kind (Slice
  :elem (Ty :id 2 :kind (Path ...))))

;; Array type: [T; 10]
(Ty :id 1 :kind (Array
  :elem (Ty :id 2 :kind (Path ...))
  :len (AnonConst :id 3 :value (Expr ...))))

;; Function pointer: fn(i32) -> bool
(Ty :id 1 :kind (BareFn
  :unsafety Safe
  :ext (None)
  :decl (FnDecl
    :inputs ((Param :ty (Ty ...)))
    :output (Ty (Ty :id 5 :kind (Path ...))))))

;; Tuple type: (i32, String, bool)
(Ty :id 1 :kind (Tup
  :elems (
    (Ty :id 2 :kind (Path ...))
    (Ty :id 3 :kind (Path ...))
    (Ty :id 4 :kind (Path ...)))))

;; Never type: !
(Ty :id 1 :kind (Never))

;; Impl trait: impl Iterator<Item = T>
(Ty :id 1 :kind (ImplTrait
  :id 2
  :bounds (
    (GenericBound (Trait ...)))))

;; Trait object: dyn Display + Send
(Ty :id 1 :kind (TraitObject
  :bounds (
    (GenericBound (Trait ...))
    (GenericBound (Trait ...)))
  :syntax Dyn))

;; Reference type: &'a mut T
(Ty :id 1 :kind (Rptr
  :lifetime (Some (Lifetime :id 2 :ident (Ident :name "a")))
  :ty (MutTy
    :ty (Ty :id 3 :kind (Path ...))
    :mutbl Mut)))
```

### Part 4.2: Type Builder

**File**: `src/builder/ty.rs`

```rust
use crate::ast::{
    Ty, TyKind, MutTy, BareFnTy, TraitObjectSyntax,
    GenericBound, PolyTraitRef, TraitRef, TraitBoundModifier,
    Lifetime, Mutability,
};
use crate::builder::{AstBuilder, BuildError};
use crate::sexp::SExp;

impl AstBuilder {
    pub fn build_ty(&mut self, sexp: &SExp) -> Result<Ty, BuildError> {
        let list = self.expect_list(sexp, "Ty")?;
        let mut id = None;
        let mut kind = None;
        let mut span = None;

        let mut i = 1;
        while i < list.elements.len() {
            let kw = self.expect_keyword(&list.elements[i])?;
            i += 1;

            match kw.as_str() {
                "id" => id = Some(self.build_node_id(&list.elements[i])?),
                "kind" => kind = Some(self.build_ty_kind(&list.elements[i])?),
                "span" => span = Some(self.build_span(&list.elements[i])?),
                "tokens" => { /* Skip */ }
                _ => return Err(BuildError::UnknownField {
                    node: "Ty".to_string(),
                    field: kw,
                }),
            }
            i += 1;
        }

        Ok(Ty {
            id: id.ok_or_else(|| BuildError::MissingField {
                node: "Ty".to_string(),
                field: "id".to_string(),
            })?,
            kind: kind.ok_or_else(|| BuildError::MissingField {
                node: "Ty".to_string(),
                field: "kind".to_string(),
            })?,
            span: span.unwrap_or_default(),
            tokens: None,
        })
    }

    fn build_ty_kind(&mut self, sexp: &SExp) -> Result<TyKind, BuildError> {
        let list = self.expect_list(sexp, "TyKind")?;
        let variant = self.expect_symbol(&list.elements[0])?;

        match variant.as_str() {
            "Slice" => {
                let elem = Box::new(self.build_ty(&self.get_field(&list, "elem")?)?);
                Ok(TyKind::Slice(elem))
            }

            "Array" => {
                let elem = Box::new(self.build_ty(&self.get_field(&list, "elem")?)?);
                let len = Box::new(self.build_anon_const(&self.get_field(&list, "len")?)?);
                Ok(TyKind::Array(elem, len))
            }

            "Ptr" => {
                let mut_ty = Box::new(self.build_mut_ty(&self.get_field(&list, "ty")?)?);
                Ok(TyKind::Ptr(mut_ty))
            }

            "Rptr" => {
                let lifetime = match self.get_optional_field(&list, "lifetime")? {
                    Some(sexp) => Some(self.build_lifetime(sexp)?),
                    None => None,
                };
                let mut_ty = Box::new(self.build_mut_ty(&self.get_field(&list, "ty")?)?);
                Ok(TyKind::Rptr(lifetime, mut_ty))
            }

            "BareFn" => {
                let bare_fn = Box::new(self.build_bare_fn_ty(&self.get_field(&list, "fn")?)?);
                Ok(TyKind::BareFn(bare_fn))
            }

            "Never" => Ok(TyKind::Never),

            "Tup" => {
                let elems = self.build_ty_list(&self.get_field(&list, "elems")?)?;
                Ok(TyKind::Tup(elems))
            }

            "Path" => {
                let qself = self.build_optional_qself(&self.get_field(&list, "qself")?)?;
                let path = self.build_path(&self.get_field(&list, "path")?)?;
                Ok(TyKind::Path(qself, path))
            }

            "TraitObject" => {
                let bounds = self.build_generic_bounds(&self.get_field(&list, "bounds")?)?;
                let syntax = self.build_trait_object_syntax(&self.get_field(&list, "syntax")?)?;
                Ok(TyKind::TraitObject(bounds, syntax))
            }

            "ImplTrait" => {
                let id = self.build_node_id(&self.get_field(&list, "id")?)?;
                let bounds = self.build_generic_bounds(&self.get_field(&list, "bounds")?)?;
                Ok(TyKind::ImplTrait(id, bounds))
            }

            "Paren" => {
                let ty = Box::new(self.build_ty(&self.get_field(&list, "ty")?)?);
                Ok(TyKind::Paren(ty))
            }

            "Infer" => Ok(TyKind::Infer),

            "MacCall" => {
                let mac = Box::new(self.build_mac_call(&self.get_field(&list, "mac")?)?);
                Ok(TyKind::MacCall(mac))
            }

            _ => Err(BuildError::UnsupportedVariant {
                enum_name: "TyKind".to_string(),
                variant,
            }),
        }
    }

    fn build_mut_ty(&mut self, sexp: &SExp) -> Result<MutTy, BuildError> {
        let list = self.expect_list(sexp, "MutTy")?;
        
        Ok(MutTy {
            ty: self.build_ty(&self.get_field(&list, "ty")?)?,
            mutbl: self.build_mutability(&self.get_field(&list, "mutbl")?)?,
        })
    }

    fn build_ty_list(&mut self, sexp: &SExp) -> Result<Vec<Ty>, BuildError> {
        let list = self.expect_list(sexp, "type list")?;
        list.elements.iter().map(|e| self.build_ty(e)).collect()
    }

    fn build_bare_fn_ty(&mut self, sexp: &SExp) -> Result<BareFnTy, BuildError> {
        let list = self.expect_list(sexp, "BareFnTy")?;
        
        Ok(BareFnTy {
            unsafety: self.build_safety(&self.get_field(&list, "unsafety")?)?,
            ext: self.build_extern(&self.get_field(&list, "ext")?)?,
            generic_params: vec![], // Simplified for now
            decl: self.build_fn_decl(&self.get_field(&list, "decl")?)?,
        })
    }

    fn build_trait_object_syntax(&mut self, sexp: &SExp) -> Result<TraitObjectSyntax, BuildError> {
        let sym = self.expect_symbol(sexp)?;
        match sym.as_str() {
            "Dyn" => Ok(TraitObjectSyntax::Dyn),
            "None" => Ok(TraitObjectSyntax::None),
            _ => Err(BuildError::InvalidValue {
                expected: "Dyn or None".to_string(),
                found: sym,
            }),
        }
    }

    fn build_generic_bounds(&mut self, sexp: &SExp) -> Result<Vec<GenericBound>, BuildError> {
        let list = self.expect_list(sexp, "generic bounds")?;
        list.elements.iter()
            .map(|e| self.build_generic_bound(e))
            .collect()
    }

    fn build_generic_bound(&mut self, sexp: &SExp) -> Result<GenericBound, BuildError> {
        // Simplified implementation
        todo!("Implement generic bound building")
    }

    fn build_lifetime(&mut self, sexp: &SExp) -> Result<Lifetime, BuildError> {
        let list = self.expect_list(sexp, "Lifetime")?;
        
        Ok(Lifetime {
            id: self.build_node_id(&self.get_field(&list, "id")?)?,
            ident: self.build_ident(&self.get_field(&list, "ident")?)?,
        })
    }
}
```

### Part 4.3: Type Generator

**File**: `src/generator/ty.rs`

```rust
use crate::ast::{
    Ty, TyKind, MutTy, BareFnTy, TraitObjectSyntax,
    GenericBound, Mutability,
};
use crate::generator::{Generator, GeneratorError};
use crate::sexp::SExp;

impl Generator {
    pub fn generate_ty(&self, ty: &Ty) -> Result<SExp, GeneratorError> {
        Ok(self.list_with("Ty", vec![
            self.kw("id"),
            self.num(ty.id.as_u32()),
            self.kw("kind"),
            self.generate_ty_kind(&ty.kind)?,
            self.kw("span"),
            self.generate_span(&ty.span),
        ]))
    }

    fn generate_ty_kind(&self, kind: &TyKind) -> Result<SExp, GeneratorError> {
        match kind {
            TyKind::Slice(elem) => {
                Ok(self.list_with("Slice", vec![
                    self.kw("elem"),
                    self.generate_ty(elem)?,
                ]))
            }

            TyKind::Array(elem, len) => {
                Ok(self.list_with("Array", vec![
                    self.kw("elem"),
                    self.generate_ty(elem)?,
                    self.kw("len"),
                    self.generate_anon_const(len)?,
                ]))
            }

            TyKind::Ptr(mut_ty) => {
                Ok(self.list_with("Ptr", vec![
                    self.kw("ty"),
                    self.generate_mut_ty(mut_ty)?,
                ]))
            }

            TyKind::Rptr(lifetime, mut_ty) => {
                Ok(self.list_with("Rptr", vec![
                    self.kw("lifetime"),
                    match lifetime {
                        Some(lt) => self.generate_lifetime(lt),
                        None => self.nil(),
                    },
                    self.kw("ty"),
                    self.generate_mut_ty(mut_ty)?,
                ]))
            }

            TyKind::BareFn(bare_fn) => {
                Ok(self.list_with("BareFn", vec![
                    self.kw("fn"),
                    self.generate_bare_fn_ty(bare_fn)?,
                ]))
            }

            TyKind::Never => {
                Ok(self.list_with("Never", vec![]))
            }

            TyKind::Tup(elems) => {
                Ok(self.list_with("Tup", vec![
                    self.kw("elems"),
                    self.generate_ty_list(elems)?,
                ]))
            }

            TyKind::Path(qself, path) => {
                Ok(self.list_with("Path", vec![
                    self.kw("qself"),
                    self.generate_optional_qself(qself.as_ref()),
                    self.kw("path"),
                    self.generate_path(path),
                ]))
            }

            TyKind::TraitObject(bounds, syntax) => {
                Ok(self.list_with("TraitObject", vec![
                    self.kw("bounds"),
                    self.generate_generic_bounds(bounds)?,
                    self.kw("syntax"),
                    self.generate_trait_object_syntax(*syntax),
                ]))
            }

            TyKind::ImplTrait(id, bounds) => {
                Ok(self.list_with("ImplTrait", vec![
                    self.kw("id"),
                    self.num(id.as_u32()),
                    self.kw("bounds"),
                    self.generate_generic_bounds(bounds)?,
                ]))
            }

            TyKind::Paren(ty) => {
                Ok(self.list_with("Paren", vec![
                    self.kw("ty"),
                    self.generate_ty(ty)?,
                ]))
            }

            TyKind::Infer => {
                Ok(self.list_with("Infer", vec![]))
            }

            TyKind::MacCall(mac) => {
                Ok(self.list_with("MacCall", vec![
                    self.kw("mac"),
                    self.generate_mac_call(mac)?,
                ]))
            }

            _ => Ok(self.sym("TODO_Type")),
        }
    }

    fn generate_mut_ty(&self, mut_ty: &MutTy) -> Result<SExp, GeneratorError> {
        Ok(self.list_with("MutTy", vec![
            self.kw("ty"),
            self.generate_ty(&mut_ty.ty)?,
            self.kw("mutbl"),
            self.generate_mutability(mut_ty.mutbl),
        ]))
    }

    fn generate_ty_list(&self, types: &[Ty]) -> Result<SExp, GeneratorError> {
        let sexps: Result<Vec<_>, _> = types.iter()
            .map(|t| self.generate_ty(t))
            .collect();
        Ok(SExp::List(List::new(sexps?, Position::default())))
    }

    fn generate_trait_object_syntax(&self, syntax: TraitObjectSyntax) -> SExp {
        match syntax {
            TraitObjectSyntax::Dyn => self.sym("Dyn"),
            TraitObjectSyntax::None => self.sym("None"),
        }
    }

    fn generate_lifetime(&self, lifetime: &Lifetime) -> SExp {
        self.list_with("Lifetime", vec![
            self.kw("id"),
            self.num(lifetime.id.as_u32()),
            self.kw("ident"),
            self.generate_ident(&lifetime.ident),
        ])
    }
}
```

---

## 5. Expression Gap Filling

Implement the most critical missing expressions:

### Priority Expression Variants

1. **ExprParen** - Parenthesized expressions
2. **ExprTry** - The `?` operator (very common)
3. **ExprCast** - Type casts with `as`
4. **ExprBreak** - Break with optional value
5. **ExprContinue** - Continue with optional label
6. **ExprReturn** - Return with optional value

**Implementation**: Extend existing files:
- `src/ast/expr.rs` - Add variants to `ExprKind`
- `src/builder/expr.rs` - Add builder methods
- `src/generator/expr.rs` - Add generator methods
- `src/codegen/expr.rs` - Add code generation

**Example** (ExprTry):

```rust
// AST type (in expr.rs)
pub enum ExprKind {
    // ... existing ...
    
    /// Try expression: `expr?`
    Try(Box<Expr>),
}

// Builder (in builder/expr.rs)
"Try" => {
    let expr = Box::new(self.build_expr(&self.get_field(&list, "expr")?)?);
    Ok(ExprKind::Try(expr))
}

// Generator (in generator/expr.rs)
ExprKind::Try(expr) => {
    Ok(self.list_with("Try", vec![
        self.kw("expr"),
        self.generate_expr(expr)?,
    ]))
}

// Codegen (in codegen/expr.rs)
ExprKind::Try(expr) => {
    write!(f, "{}?", self.generate_expr(expr)?)
}
```

---

## 6. Code Generation Completion

### Goal

Ensure every implemented AST type can generate valid Rust code.

### Current Gaps

Based on Phase 4 assessment:
- Some expression variants have `todo!()` or comments
- Pattern code generation needs implementation
- Type code generation needs implementation
- Some edge cases not handled

### Implementation Strategy

**File**: `src/codegen/pat.rs` (NEW)

```rust
use crate::ast::{Pat, PatKind, BindingMode, Mutability};
use std::fmt;

pub struct PatCodegen<'a> {
    pat: &'a Pat,
}

impl<'a> PatCodegen<'a> {
    pub fn new(pat: &'a Pat) -> Self {
        Self { pat }
    }

    pub fn generate(&self) -> String {
        self.generate_pat_kind(&self.pat.kind)
    }

    fn generate_pat_kind(&self, kind: &PatKind) -> String {
        match kind {
            PatKind::Wild => "_".to_string(),

            PatKind::Ident(mode, ident, subpat) => {
                let mut result = String::new();
                
                match mode {
                    BindingMode::ByValue(Mutability::Mut) => result.push_str("mut "),
                    BindingMode::ByRef(Mutability::Not) => result.push_str("ref "),
                    BindingMode::ByRef(Mutability::Mut) => result.push_str("ref mut "),
                    _ => {}
                }
                
                result.push_str(&ident.name);
                
                if let Some(sub) = subpat {
                    result.push_str(" @ ");
                    result.push_str(&PatCodegen::new(sub).generate());
                }
                
                result
            }

            PatKind::Tuple(pats) => {
                let inner = pats.iter()
                    .map(|p| PatCodegen::new(p).generate())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({})", inner)
            }

            PatKind::Or(pats) => {
                pats.iter()
                    .map(|p| PatCodegen::new(p).generate())
                    .collect::<Vec<_>>()
                    .join(" | ")
            }

            PatKind::Ref(pat, mutability) => {
                let mut_str = match mutability {
                    Mutability::Mut => " mut",
                    Mutability::Not => "",
                };
                format!("&{}{}", mut_str, PatCodegen::new(pat).generate())
            }

            // ... implement all other variants ...

            _ => todo!("Pattern codegen for {:?}", kind),
        }
    }
}
```

**File**: `src/codegen/ty.rs` (NEW)

```rust
use crate::ast::{Ty, TyKind, MutTy, Mutability};
use std::fmt;

pub struct TyCodegen<'a> {
    ty: &'a Ty,
}

impl<'a> TyCodegen<'a> {
    pub fn new(ty: &'a Ty) -> Self {
        Self { ty }
    }

    pub fn generate(&self) -> String {
        self.generate_ty_kind(&self.ty.kind)
    }

    fn generate_ty_kind(&self, kind: &TyKind) -> String {
        match kind {
            TyKind::Slice(elem) => {
                format!("[{}]", TyCodegen::new(elem).generate())
            }

            TyKind::Array(elem, len) => {
                // Simplified: should extract actual length
                format!("[{}; N]", TyCodegen::new(elem).generate())
            }

            TyKind::Ptr(mut_ty) => {
                let mutbl = match mut_ty.mutbl {
                    Mutability::Mut => "mut",
                    Mutability::Not => "const",
                };
                format!("*{} {}", mutbl, TyCodegen::new(&mut_ty.ty).generate())
            }

            TyKind::Rptr(lifetime, mut_ty) => {
                let lt = lifetime.as_ref()
                    .map(|l| format!("'{} ", l.ident.name))
                    .unwrap_or_default();
                let mutbl = match mut_ty.mutbl {
                    Mutability::Mut => "mut ",
                    Mutability::Not => "",
                };
                format!("&{}{}{}", lt, mutbl, TyCodegen::new(&mut_ty.ty).generate())
            }

            TyKind::Never => "!".to_string(),

            TyKind::Tup(elems) => {
                let inner = elems.iter()
                    .map(|t| TyCodegen::new(t).generate())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({})", inner)
            }

            TyKind::Path(_, path) => {
                // Simplified path generation
                path.segments.iter()
                    .map(|seg| seg.ident.name.clone())
                    .collect::<Vec<_>>()
                    .join("::")
            }

            TyKind::ImplTrait(_, bounds) => {
                format!("impl Trait") // Simplified
            }

            TyKind::TraitObject(bounds, syntax) => {
                match syntax {
                    TraitObjectSyntax::Dyn => format!("dyn Trait"), // Simplified
                    TraitObjectSyntax::None => "Trait".to_string(),
                }
            }

            TyKind::Infer => "_".to_string(),

            // ... implement all other variants ...

            _ => todo!("Type codegen for {:?}", kind),
        }
    }
}
```

---

## 7. Testing Strategy

### Test Organization

```
oxur-ast/tests/
├── builder_pat_comprehensive_tests.rs    # NEW: Pattern building
├── builder_ty_comprehensive_tests.rs     # NEW: Type building  
├── generator_pat_tests.rs                # NEW: Pattern generation
├── generator_ty_tests.rs                 # NEW: Type generation
├── codegen_pat_tests.rs                  # NEW: Pattern code generation
├── codegen_ty_tests.rs                   # NEW: Type code generation
├── round_trip_complex_tests.rs           # NEW: Complex round-trips
└── regression_comprehensive.rs           # NEW: Real-world code
```

### Test Coverage Goals

1. **Unit Tests**: Every pattern/type variant
2. **Integration Tests**: Complex combinations
3. **Round-Trip Tests**: Parse → Generate → Parse
4. **Code Generation Tests**: Generated code compiles
5. **Real-World Tests**: Actual Rust files from the wild

### Example Round-Trip Test

```rust
#[test]
fn test_match_with_complex_patterns() {
    let source = r#"
    fn process(value: Option<(i32, String)>) -> i32 {
        match value {
            Some((x, ref s)) if x > 0 => x,
            Some((x, _)) => -x,
            None => 0,
        }
    }
    "#;

    // Parse Rust
    let crate_ast = parse_rust_file(source).unwrap();

    // Generate S-expression
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_ast).unwrap();

    // Parse S-expression
    let sexp_text = print_sexp(&sexp);
    let sexp_parsed = Parser::parse_str(&sexp_text).unwrap();

    // Build AST
    let mut builder = AstBuilder::new();
    let crate_rebuilt = builder.build_crate(&sexp_parsed).unwrap();

    // Generate Rust code
    let generated = generate_rust(&crate_rebuilt).unwrap();

    // Verify it compiles
    assert!(generated.contains("match value"));
    assert!(generated.contains("Some((x, ref s))"));
}
```

---

## 8. Success Criteria

Phase 5 is complete when:

### Pattern Matching ✅
- [ ] All 15+ PatKind variants implemented
- [ ] Pattern builder complete
- [ ] Pattern generator complete
- [ ] Pattern code generation complete
- [ ] 100+ pattern tests passing
- [ ] Complex match expressions work

### Type System ✅
- [ ] All 15+ TyKind variants implemented
- [ ] Type builder complete
- [ ] Type generator complete
- [ ] Type code generation complete
- [ ] 100+ type tests passing
- [ ] Complex type annotations work

### Expression Gaps ✅
- [ ] ExprParen implemented
- [ ] ExprTry implemented (? operator)
- [ ] ExprCast implemented (as operator)
- [ ] ExprBreak/Continue/Return with values implemented
- [ ] Tests for all new expressions

### Code Generation ✅
- [ ] No `todo!()` or stubs in codegen
- [ ] All patterns generate valid code
- [ ] All types generate valid code
- [ ] Round-trip tests all pass
- [ ] Generated code compiles with rustc

### Testing & Quality ✅
- [ ] 500+ tests total (including new tests)
- [ ] >85% code coverage
- [ ] All clippy warnings addressed
- [ ] Documentation updated
- [ ] Examples work

### Integration ✅
- [ ] Can parse complex real-world Rust files
- [ ] Can generate S-expressions for real code
- [ ] Can rebuild AST from S-expressions
- [ ] Can generate compilable Rust code
- [ ] Benchmarks show acceptable performance

---

## 9. Implementation Roadmap

### Week 1: Pattern System (8-10 hours)

**Days 1-2: Pattern Types & Builder**
- Implement `src/ast/pat.rs` with all PatKind variants (3 hours)
- Implement `src/builder/pat.rs` builder methods (3 hours)
- Basic pattern tests (2 hours)

**Days 2-3: Pattern Generator & Codegen**
- Implement `src/generator/pat.rs` (2 hours)
- Implement `src/codegen/pat.rs` (2 hours)
- Comprehensive pattern tests (2 hours)

### Week 1-2: Type System (7-9 hours)

**Days 3-4: Type Extensions**
- Extend `src/ast/types.rs` with all TyKind variants (2 hours)
- Implement `src/builder/ty.rs` (3 hours)
- Basic type tests (2 hours)

**Days 4-5: Type Generator & Codegen**
- Implement `src/generator/ty.rs` (2 hours)
- Implement `src/codegen/ty.rs` (2 hours)
- Comprehensive type tests (2 hours)

### Week 2: Expression Gaps & Codegen (5-7 hours)

**Day 5-6: Expression Filling**
- Implement missing expression variants (3 hours)
- Tests for new expressions (2 hours)

**Day 6-7: Code Generation Completion**
- Fill in codegen stubs (2 hours)
- Round-trip tests (2 hours)

### Week 2: Testing & Polish (4-6 hours)

**Day 7-8: Comprehensive Testing**
- Integration tests with real code (2 hours)
- Edge case coverage (2 hours)
- Performance benchmarks (1 hour)

**Day 8: Documentation**
- Update ARCHITECTURE.md (1 hour)
- Update design docs (1 hour)
- Create Phase 5 completion report (1 hour)

### Total Time Estimate

- **Pattern System**: 8-10 hours
- **Type System**: 7-9 hours
- **Expression Gaps**: 3-4 hours
- **Codegen Completion**: 2-3 hours
- **Testing & Polish**: 4-6 hours
- **TOTAL**: 24-32 hours (3-4 working days)

---

## 10. Beyond Phase 5

### What Phase 5 Delivers

After Phase 5, `oxur-ast` will have:
- ✅ Complete pattern matching support
- ✅ Complete type system coverage
- ✅ All common expressions working
- ✅ Full code generation capability
- ✅ Production-ready quality

### What's Still Missing (Future Phases)

**Advanced Items** (Phase 6 candidate):
- Union types
- Foreign function interface (FFI)
- Global assembly
- Trait aliases
- Macro definitions

**Advanced Features** (Phase 7+ candidates):
- Const generics
- Advanced lifetime handling
- Macro expansion
- Procedural macros
- Full attribute parsing (derives, doc comments, etc.)

**Tooling** (Future):
- LSP integration
- Incremental parsing
- Source maps
- REPL integration
- Hot-reload support

### Migration to Production

Once Phase 5 is complete:
1. Update version to 0.5.0 (or 1.0.0 if ready)
2. Mark Phase 4 as "Complete" in design docs
3. Mark Phase 5 as "Complete" in design docs
4. Create Phase 6 plan for advanced features
5. Begin using in oxur-comp and oxur-lang

---

## Success Metrics

### Quantitative

- Pattern coverage: 3/15 → 15/15 (100%)
- Type coverage: 3/15 → 15/15 (100%)
- Expression coverage: 20/35+ → 25+/35+ (70%+)
- Code generation: 70% → 95%+
- Test count: ~8,000 lines → ~10,000+ lines
- Overall Phase 4 targets: 60-70% → 90-95%

### Qualitative

- Can handle real-world Rust patterns
- Match expressions fully supported
- Complex type signatures work
- Error handling patterns (`?`) work
- Generated code compiles without modification
- Round-trips preserve semantics

---

## Conclusion

Phase 5 focuses on completing the foundational AST coverage by implementing patterns and types—the two most critical gaps identified in Phase 4. This focused approach allows us to:

1. **Complete the Foundation**: Pattern matching and type system are core to Rust
2. **Enable Production Use**: With these complete, oxur-ast can handle most real code
3. **Set Up for Advanced Features**: Clean foundation for Phases 6-7
4. **Maintain Momentum**: Achievable in 3-4 days of focused work

After Phase 5, we'll have a production-ready AST library that can:
- Parse complex Rust code
- Represent it as S-expressions
- Rebuild the AST perfectly
- Generate compilable Rust code
- Support the full Oxur compilation pipeline

**Next**: Begin implementation with pattern system, as it's the highest-impact work.

---

*"Patterns and types are the language of intent. Complete them, and the AST speaks clearly."*
