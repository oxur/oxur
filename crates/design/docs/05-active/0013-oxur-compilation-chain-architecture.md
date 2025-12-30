---
number: 13
title: "Oxur Compilation Chain Architecture"
author: "Duncan McGreggor & Claude"
component: Compiler
tags: [architecture, compiler, source maps]
created: 2025-12-27
updated: 2025-12-27
state: Active
supersedes: null
superseded-by: null
version: 1.0
---

# Oxur Compilation Chain Architecture

**Status**: Design Specification v1.0
**Date**: December 2025
**Purpose**: Define the complete compilation pipeline for Oxur, from source code to binary

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Overview](#overview)
3. [Architecture Principles](#architecture-principles)
4. [The Compilation Pipeline](#the-compilation-pipeline)
5. [Stage 1: Parse (Oxur Syntax → Surface Forms)](#stage-1-parse-oxur-syntax--surface-forms)
6. [Stage 2: Expand (Surface Forms → Core Forms)](#stage-2-expand-surface-forms--core-forms)
7. [Stage 3: Lower (Core Forms → Rust AST)](#stage-3-lower-core-forms--rust-ast)
8. [Stage 4: Generate (Rust AST → Rust Source)](#stage-4-generate-rust-ast--rust-source)
9. [Stage 5: Compile (Rust Source → Binary)](#stage-5-compile-rust-source--binary)
10. [Source Map Architecture](#source-map-architecture)
11. [Error Reporting](#error-reporting)
12. [REPL Architecture](#repl-architecture)
13. [Macro System](#macro-system)
14. [Repository Structure](#repository-structure)
15. [Development Phases](#development-phases)
16. [Testing Strategy](#testing-strategy)
17. [Performance Considerations](#performance-considerations)
18. [Open Questions](#open-questions)

---

## Executive Summary

Oxur is a Lisp dialect that compiles to Rust with 100% interoperability. This document defines the complete compilation architecture for v1.0.

### Key Design Decisions

1. **Two-stage compilation** with stable IR (Intermediate Representation)
2. **Core Forms as canonical S-expressions** - the contract between compilation stages
3. **Node ID-based source mapping** - provenance tracking through all transformations
4. **Phased macro system** - core macros (v1.0), user macros (v2.0)
5. **No runtime library needed** - Rust's type system is powerful enough
6. **Simplified REPL** - no plugin memory leaks like Go, no supervised workers needed
7. **Three-tier execution** - interpretation for simple forms, compilation for complex ones

### Benefits of This Architecture

- ✅ **Stable intermediate representation** - experiment with syntax without breaking backend
- ✅ **Accurate error reporting** - source maps track every transformation
- ✅ **Fast REPL** - tiered execution optimizes common cases
- ✅ **Native performance** - compiles to idiomatic Rust
- ✅ **Language extensibility** - macro system designed from day one
- ✅ **Clean separation** - each stage has clear responsibilities

---

## Overview

### The Big Picture

```
Oxur Source (.ox files)
    ↓ Stage 1: Parse
Surface Forms (with sugar, macros)
    ↓ Stage 2: Expand
Core Forms (canonical S-expressions - the IR)
    ↓ Stage 3: Lower
Rust AST (syn crate structures)
    ↓ Stage 4: Generate
Rust Source (.rs files)
    ↓ Stage 5: Compile
Binary / Library (rustc)
```

### Two-Stage Compilation Philosophy

Following Zylisp's successful pattern, we split compilation into two distinct stages:

**Stage 1: Oxur Language** (`oxur/lang`)

- Parses Oxur syntax
- Expands macros and desugaring
- Produces **Core Forms** (the IR)
- Can evolve rapidly without affecting Rust interop

**Stage 2: Rust AST Layer** (`oxur/crates/oxur-ast`)

- Bidirectional Core Forms ↔ Rust AST conversion
- Stable "assembly language" for Rust
- 1:1 mapping - explicit and complete
- Rarely changes, rock solid

### Why This Separation Matters

**Experimentation**: Change Oxur syntax without touching Rust interop
**Debugging**: Inspect IR between stages
**Reusability**: Other tools can target Core Forms
**Testability**: Each stage tested in isolation
**Stability**: IR is the stable contract

---

## Architecture Principles

### 1. Homoiconicity is Sacred

Code is data. The entire compilation pipeline treats programs as data structures that can be inspected, transformed, and reasoned about.

### 2. Explicit Over Implicit

- Lifetimes visible when needed
- Type annotations where they matter
- No hidden magic - what you see is what compiles

### 3. Leverage Rust's Strengths

- Use Rust's type system fully
- Embrace traits over objects
- Pattern matching everywhere
- Zero-cost abstractions

### 4. Reliability First

Following Erlang/LFE philosophy:

- Clear error messages at every stage
- Source maps track provenance
- No silent failures
- Supervision patterns where appropriate

### 5. Performance Without Compromise

- Compile to idiomatic Rust
- No unnecessary runtime overhead
- Cache compilation artifacts
- Optimize common REPL patterns

---

## The Compilation Pipeline

### Full Pipeline Overview

```
┌──────────────────────────────────────────────────────────────┐
│                    Stage 1: Parse                            │
│                                                              │
│  Input:  Raw text (.ox files)                                │
│  Output: Surface Forms (S-expression AST)                    │
│                                                              │
│  Responsibilities:                                           │
│  • Lexical analysis (tokenization)                           │
│  • Reader (text → S-expressions)                             │
│  • Reader macros (tagged literals: #tag<...>)                │
│  • Assign unique Node IDs to every form                      │
│  • Record original source positions                          │
│  • Create first source map (input layer)                     │
└──────────────────────────────────────────────────────────────┘
                            ↓
                    Surface Forms
                            ↓
┌──────────────────────────────────────────────────────────────┐
│                    Stage 2: Expand                           │
│                                                              │
│  Input:  Surface Forms (with sugar, macros)                  │
│  Output: Core Forms (canonical S-expressions)                │
│                                                              │
│  Responsibilities:                                           │
│  • Macro expansion (core macros in v1.0)                     │
│  • Desugaring (convenience syntax → canonical forms)         │
│  • Track transformations in source map                       │
│  • Generate new Node IDs for generated forms                 │
│  • Validate syntax                                           │
│                                                              │
│  Examples:                                                   │
│  • defn → define-func                                        │
│  • when → if-expr                                            │
│  • -> threading → nested calls                               │
└──────────────────────────────────────────────────────────────┘
                            ↓
                    Core Forms (IR)
                            ↓
┌──────────────────────────────────────────────────────────────┐
│                    Stage 3: Lower                            │
│                                                              │
│  Input:  Core Forms (canonical S-expressions)                │
│  Output: Rust AST (syn crate structures)                     │
│                                                              │
│  Responsibilities:                                           │
│  • Map Core Forms to Rust AST nodes                          │
│  • Type inference/checking (optional - can defer to rustc)   │
│  • Lifetime annotation (insert explicit lifetimes)           │
│  • Trait bound resolution                                    │
│  • Track Core Form → Rust AST mapping in source map          │
│                                                              │
│  Examples:                                                   │
│  • define-func → syn::ItemFn                                 │
│  • if-expr → syn::ExprIf                                     │
│  • match-expr → syn::ExprMatch                               │
└──────────────────────────────────────────────────────────────┘
                            ↓
                    Rust AST
                            ↓
┌──────────────────────────────────────────────────────────────┐
│                    Stage 4: Generate                         │
│                                                              │
│  Input:  Rust AST                                            │
│  Output: Formatted Rust source code                          │
│                                                              │
│  Responsibilities:                                           │
│  • Pretty-print Rust code                                    │
│  • Use prettyplease or similar for formatting                │
│  • Preserve structure for debugging                          │
│                                                              │
│  Note: Source map ends at Rust AST level                     │
└──────────────────────────────────────────────────────────────┘
                            ↓
                    Rust Source (.rs)
                            ↓
┌──────────────────────────────────────────────────────────────┐
│                    Stage 5: Compile                          │
│                                                              │
│  Tool:   rustc (or cargo)                                    │
│  Input:  Generated .rs files                                 │
│  Output: Compiled binary or library                          │
│                                                              │
│  Error Handling:                                             │
│  • When rustc reports errors, use source map to translate    │
│    back to original Oxur source                              │
└──────────────────────────────────────────────────────────────┘
                            ↓
                    Binary / Library
```

### Data Flow Summary

| Stage | Input | Output | Key Operation |
|-------|-------|--------|--------------|
| 1. Parse | Text | Surface Forms | Tokenize, read S-expressions |
| 2. Expand | Surface Forms | Core Forms | Macro expansion, desugaring |
| 3. Lower | Core Forms | Rust AST | Map to Rust structures |
| 4. Generate | Rust AST | Rust Source | Pretty-print |
| 5. Compile | Rust Source | Binary | rustc |

---

## Stage 1: Parse (Oxur Syntax → Surface Forms)

### Purpose

Convert raw text into S-expression data structures that preserve all syntactic information.

### Components

#### Lexer (Tokenization)

**Responsibilities**:

- Character stream → Token stream
- Recognize literals, symbols, keywords
- Handle reader macros (tagged literals)
- Track positions (line, column, file)

**Token Types**:

```rust
pub enum Token {
    // Delimiters
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),

    // Identifiers
    Symbol(String),
    Keyword(String),  // :keyword

    // Special
    Quote,            // '
    Quasiquote,       // `
    Unquote,          // ,
    UnquoteSplice,    // ,@

    // Tagged literals
    TaggedLiteral {
        tag: String,
        content: String,
        delim: char,  // '<', '{', '[', '('
    },

    // Position tracking
    pos: SourcePos,
}

pub struct SourcePos {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub offset: usize,  // Byte offset in file
}
```

**Implementation Note**: The lexer must handle balanced delimiters for tagged literals:

```rust
impl Lexer {
    fn scan_tagged_literal(&mut self) -> Result<Token> {
        let tag = self.scan_identifier()?;
        let delim = self.current_char();

        if !matches!(delim, '<' | '{' | '[' | '(') {
            return Err(Error::InvalidTaggedLiteral);
        }

        let closing = match delim {
            '<' => '>',
            '{' => '}',
            '[' => ']',
            '(' => ')',
            _ => unreachable!(),
        };

        self.advance(); // Consume opening delimiter
        let content = self.scan_until_balanced(closing)?;

        Ok(Token::TaggedLiteral { tag, content, delim })
    }

    fn scan_until_balanced(&mut self, closing: char) -> Result<String> {
        let mut content = String::new();
        let mut depth = 1;

        while depth > 0 {
            let ch = self.current_char();

            if ch == self.matching_opener(closing) {
                depth += 1;
            } else if ch == closing {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }

            content.push(ch);
            self.advance();
        }

        self.advance(); // Consume closing delimiter
        Ok(content)
    }
}
```

#### Reader (Tokens → S-expressions)

**Responsibilities**:

- Token stream → S-expression AST
- Build list/vector/map structures
- Handle quote/quasiquote syntax sugar
- Assign unique Node IDs
- Create initial source map

**Surface Form Types**:

```rust
pub enum SurfaceForm {
    // Atomic forms
    Literal(Literal),
    Symbol(Symbol),
    Keyword(Keyword),

    // Compound forms
    List(Vec<SurfaceForm>),
    Vector(Vec<SurfaceForm>),
    Map(Vec<(SurfaceForm, SurfaceForm)>),

    // Quote forms
    Quote(Box<SurfaceForm>),
    Quasiquote(Box<SurfaceForm>),
    Unquote(Box<SurfaceForm>),
    UnquoteSplice(Box<SurfaceForm>),

    // Tagged literals (preserved for expansion)
    TaggedLiteral {
        tag: String,
        content: String,
    },

    // Metadata
    node_id: NodeId,
}

pub type NodeId = u64;

pub struct Symbol {
    pub name: String,
    pub namespace: Option<String>,  // For qualified symbols: ns/name
}

pub struct Keyword {
    pub name: String,
    pub namespace: Option<String>,
}

pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    Nil,
}
```

**Node ID Assignment**:

```rust
pub struct NodeIdGenerator {
    counter: AtomicU64,
}

impl NodeIdGenerator {
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
        }
    }

    pub fn next(&self) -> NodeId {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }
}

// Global generator for compilation session
thread_local! {
    static NODE_ID_GEN: NodeIdGenerator = NodeIdGenerator::new();
}

pub fn next_node_id() -> NodeId {
    NODE_ID_GEN.with(|gen| gen.next())
}
```

#### Parser (Orchestration)

**Responsibilities**:

- Drive lexer and reader
- Create source map (input layer)
- Handle errors with context

**Implementation**:

```rust
pub struct Parser {
    lexer: Lexer,
    source_map: SourceMap,
}

impl Parser {
    pub fn new(source: &str, filename: PathBuf) -> Self {
        let lexer = Lexer::new(source, filename.clone());
        let source_map = SourceMap::new_input_layer("parse");

        Self { lexer, source_map }
    }

    pub fn parse(&mut self) -> Result<Vec<SurfaceForm>> {
        let mut forms = Vec::new();

        while !self.lexer.is_eof() {
            let form = self.parse_form()?;
            forms.push(form);
        }

        Ok(forms)
    }

    fn parse_form(&mut self) -> Result<SurfaceForm> {
        let start_pos = self.lexer.position();
        let node_id = next_node_id();

        let token = self.lexer.next_token()?;
        let form = self.read_token(token)?;

        // Record original position for this node
        let end_pos = self.lexer.position();
        self.source_map.record_original(
            node_id,
            SourceLocation {
                file: self.lexer.filename().clone(),
                start: start_pos,
                end: end_pos,
            },
        );

        Ok(self.attach_node_id(form, node_id))
    }

    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }
}
```

### Example Input/Output

**Input**:

```lisp
(defn add [x y]
  (+ x y))
```

**Output (conceptual)**:

```rust
SurfaceForm::List {
    node_id: 1,
    items: vec![
        SurfaceForm::Symbol {
            node_id: 2,
            name: "defn"
        },
        SurfaceForm::Symbol {
            node_id: 3,
            name: "add"
        },
        SurfaceForm::Vector {
            node_id: 4,
            items: vec![
                SurfaceForm::Symbol { node_id: 5, name: "x" },
                SurfaceForm::Symbol { node_id: 6, name: "y" },
            ],
        },
        SurfaceForm::List {
            node_id: 7,
            items: vec![
                SurfaceForm::Symbol { node_id: 8, name: "+" },
                SurfaceForm::Symbol { node_id: 9, name: "x" },
                SurfaceForm::Symbol { node_id: 10, name: "y" },
            ],
        },
    ],
}
```

**Source Map (input layer)**:

```
Node 1 → test.ox:1:1-2:8   (entire defn form)
Node 2 → test.ox:1:2-1:5   (defn symbol)
Node 3 → test.ox:1:7-1:9   (add symbol)
Node 4 → test.ox:1:11-1:15 (parameter vector)
...
```

---

## Stage 2: Expand (Surface Forms → Core Forms)

### Purpose

Transform user-facing syntax (with sugar and macros) into canonical Core Forms that represent the essence of the program.

### Conceptual Model

Surface Forms contain:

- **Sugar**: Convenience syntax (e.g., `defn`, `when`, `->`)
- **Macros**: Code-generating templates
- **Metadata**: Additional information

Core Forms are:

- **Canonical**: One true representation for each concept
- **Explicit**: No hidden transformations remain
- **Complete**: Ready for lowering to Rust

### Expansion Process

```rust
pub struct Expander {
    core_macros: CoreMacroRegistry,
    source_map: SourceMap,
}

impl Expander {
    pub fn new(previous_map: &SourceMap) -> Self {
        Self {
            core_macros: CoreMacroRegistry::load_builtin(),
            source_map: SourceMap::new("expand", previous_map),
        }
    }

    pub fn expand(&mut self, form: &SurfaceForm) -> Result<CoreForm> {
        match form {
            // Macro call
            SurfaceForm::List(items) if self.is_macro_call(items) => {
                let macro_name = &items[0];
                let args = &items[1..];

                let expander = self.core_macros
                    .get(macro_name)
                    .ok_or(Error::UndefinedMacro)?;

                // Expand macro
                let expanded = expander(args)?;

                // Assign new node ID to expanded form
                let new_id = next_node_id();
                let expanded = self.attach_node_id(expanded, new_id);

                // Record transformation in source map
                self.source_map.record_transform(new_id, form.node_id());

                // Recursively expand result
                self.expand(&expanded)
            }

            // Desugar (convert convenience syntax to canonical)
            SurfaceForm::List(items) if self.is_sugar(items) => {
                let desugared = self.desugar(form)?;
                self.expand(&desugared)
            }

            // Already in canonical form, expand children
            _ => self.expand_recursively(form)
        }
    }

    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }
}
```

### Core Forms Specification

Core Forms are the **Intermediate Representation** (IR) of Oxur. They are:

- Canonical S-expressions
- Explicit and unambiguous
- 1:1 mappable to Rust AST nodes

**Design Decision**: We use **keyword arguments** for clarity, following Zetalisp style:

```rust
pub enum CoreForm {
    // Literals
    Literal(Literal),

    // Variables
    VarRef(Symbol),

    // Functions
    DefineFunc {
        name: Symbol,
        params: Vec<Param>,
        return_type: Option<Type>,
        body: Vec<CoreForm>,
        node_id: NodeId,
    },

    Lambda {
        params: Vec<Param>,
        return_type: Option<Type>,
        body: Vec<CoreForm>,
        node_id: NodeId,
    },

    // Control flow
    IfExpr {
        test: Box<CoreForm>,
        then: Box<CoreForm>,
        else_: Option<Box<CoreForm>>,
        node_id: NodeId,
    },

    Match {
        expr: Box<CoreForm>,
        arms: Vec<MatchArm>,
        node_id: NodeId,
    },

    // Bindings
    Let {
        bindings: Vec<(Symbol, CoreForm)>,
        body: Vec<CoreForm>,
        node_id: NodeId,
    },

    // Operations
    BinaryOp {
        op: BinOp,
        left: Box<CoreForm>,
        right: Box<CoreForm>,
        node_id: NodeId,
    },

    Call {
        func: Box<CoreForm>,
        args: Vec<CoreForm>,
        node_id: NodeId,
    },

    // ... more forms
}

pub struct Param {
    pub name: Symbol,
    pub ty: Type,
}

pub enum Type {
    Named(Symbol),
    Reference { lifetime: Option<Lifetime>, ty: Box<Type> },
    MutableReference { lifetime: Option<Lifetime>, ty: Box<Type> },
    Tuple(Vec<Type>),
    // ... more types
}
```

### Macro Expansion Examples

#### Example 1: `defn` → `define-func`

**Surface Form**:

```lisp
(defn add [x y]
  (+ x y))
```

**Core Form**:

```lisp
(define-func
  :name add
  :params [(param :name x :type _)
           (param :name y :type _)]
  :return-type _
  :body [(binary-op :op + :left (var-ref x) :right (var-ref y))])
```

**Note**: `_` represents type inference (let rustc figure it out)

#### Example 2: `when` → `if-expr`

**Surface Form**:

```lisp
(when (> x 10)
  (println x)
  (inc x))
```

**Core Form**:

```lisp
(if-expr
  :test (binary-op :op > :left (var-ref x) :right (literal 10))
  :then (block
          [(call (var-ref println) [(var-ref x)])
           (call (var-ref inc) [(var-ref x)])])
  :else nil)
```

#### Example 3: Threading macro `->` → nested calls

**Surface Form**:

```lisp
(-> x
    (inc)
    (* 2)
    (format "Result: {}"))
```

**Core Form**:

```lisp
(call (var-ref format)
  [(call (binary-op :op *
          :left (call (var-ref inc) [(var-ref x)])
          :right (literal 2))
         [])
   (literal "Result: {}")])
```

### Desugaring

Desugaring converts convenience syntax to canonical forms **before** macro expansion:

```rust
impl Expander {
    fn desugar(&mut self, form: &SurfaceForm) -> Result<SurfaceForm> {
        match form {
            // Vector literals become vec! calls
            SurfaceForm::Vector(items) => {
                self.desugar_vector(items)
            }

            // Map literals become hashmap! calls
            SurfaceForm::Map(pairs) => {
                self.desugar_map(pairs)
            }

            // Reader convenience
            SurfaceForm::Quote(inner) => {
                // Convert 'x to (quote x)
                self.desugar_quote(inner)
            }

            _ => Ok(form.clone())
        }
    }
}
```

### Source Map Tracking

**Critical**: Every transformation must be recorded:

```rust
// When expanding (when test body)
let when_form = /* parse "(when test body)" */;
let when_node_id = when_form.node_id(); // Say, 100

// Expand to (if-expr :test test :then (do body) :else nil)
let if_form = expand_when_to_if(when_form);
let if_node_id = next_node_id(); // Say, 200

// Record transformation
source_map.record_transform(200, 100);
// "Node 200 came from Node 100"
```

Later, if there's an error in the `if-expr`, we can trace back:

```
Error at Node 200
  → came from Node 100 (when form)
  → original source: test.ox:5:3
```

---

## Stage 3: Lower (Core Forms → Rust AST)

### Purpose

Map Core Forms (canonical S-expressions) to Rust's AST structures, preparing for code generation.

### Why Rust AST?

We use the `syn` crate's AST types directly:

- Well-documented and stable
- Matches Rust's semantics exactly
- Can leverage existing Rust tools
- `quote` macro for easy AST construction

### Lowering Strategy

```rust
pub struct Lowerer {
    source_map: SourceMap,
}

impl Lowerer {
    pub fn new(previous_map: &SourceMap) -> Self {
        Self {
            source_map: SourceMap::new("lower", previous_map),
        }
    }

    pub fn lower(&mut self, form: &CoreForm) -> Result<syn::Item> {
        match form {
            CoreForm::DefineFunc { name, params, return_type, body, node_id } => {
                let rust_fn = self.lower_function(name, params, return_type, body)?;

                // Record mapping in source map
                let rust_node_id = next_node_id();
                self.source_map.record_transform(rust_node_id, *node_id);

                Ok(syn::Item::Fn(rust_fn))
            }

            // ... other core forms
        }
    }

    fn lower_function(
        &mut self,
        name: &Symbol,
        params: &[Param],
        return_type: &Option<Type>,
        body: &[CoreForm],
    ) -> Result<syn::ItemFn> {
        let ident = self.symbol_to_ident(name);
        let inputs = self.lower_params(params)?;
        let output = self.lower_return_type(return_type)?;
        let block = self.lower_body(body)?;

        Ok(parse_quote! {
            fn #ident(#inputs) #output #block
        })
    }

    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }
}
```

### Core Form → Rust AST Mappings

| Core Form | Rust AST | Example |
|-----------|----------|---------|
| `define-func` | `syn::ItemFn` | `fn add(x: i32, y: i32) -> i32 { ... }` |
| `lambda` | `syn::ExprClosure` | `\|x, y\| x + y` |
| `if-expr` | `syn::ExprIf` | `if x > 0 { ... } else { ... }` |
| `match` | `syn::ExprMatch` | `match x { Some(v) => ..., None => ... }` |
| `let` | `syn::Local` | `let x = 42;` |
| `binary-op` | `syn::ExprBinary` | `x + y` |
| `call` | `syn::ExprCall` | `foo(1, 2, 3)` |
| `var-ref` | `syn::ExprPath` | `x` or `std::vec::Vec` |

### Type Lowering

Types in Core Forms need to be lowered to Rust types:

```rust
impl Lowerer {
    fn lower_type(&self, ty: &Type) -> Result<syn::Type> {
        match ty {
            Type::Named(sym) => {
                let path = self.symbol_to_path(sym);
                Ok(parse_quote! { #path })
            }

            Type::Reference { lifetime, ty } => {
                let inner = self.lower_type(ty)?;
                match lifetime {
                    Some(lt) => {
                        let lt_ident = format_ident!("{}", lt);
                        Ok(parse_quote! { &#lt_ident #inner })
                    }
                    None => Ok(parse_quote! { &#inner }),
                }
            }

            Type::MutableReference { lifetime, ty } => {
                let inner = self.lower_type(ty)?;
                match lifetime {
                    Some(lt) => {
                        let lt_ident = format_ident!("{}", lt);
                        Ok(parse_quote! { &#lt_ident mut #inner })
                    }
                    None => Ok(parse_quote! { &mut #inner }),
                }
            }

            Type::Tuple(types) => {
                let types: Vec<_> = types.iter()
                    .map(|t| self.lower_type(t))
                    .collect::<Result<_>>()?;
                Ok(parse_quote! { (#(#types),*) })
            }
        }
    }
}
```

### Lifetime Handling

Oxur exposes Rust's lifetime system explicitly in Core Forms:

```rust
pub struct Lifetime(String);  // 'a, 'b, 'static, etc.
```

**Example**:

```lisp
;; Core Form
(define-func
  :name first
  :lifetimes ['a]
  :params [(param :name s
                  :type (reference :lifetime 'a
                                   :type str))]
  :return-type (reference :lifetime 'a :type str)
  :body [(method-call (var-ref s) chars [])])
```

**Lowered Rust**:

```rust
fn first<'a>(s: &'a str) -> &'a str {
    s.chars().next().unwrap()
}
```

### Ownership Operations

Core Forms have explicit ownership operations:

```rust
pub enum CoreForm {
    // ... other forms

    Borrow {
        expr: Box<CoreForm>,
        node_id: NodeId,
    },

    BorrowMut {
        expr: Box<CoreForm>,
        node_id: NodeId,
    },

    Deref {
        expr: Box<CoreForm>,
        node_id: NodeId,
    },

    Move {
        expr: Box<CoreForm>,
        node_id: NodeId,
    },

    Clone {
        expr: Box<CoreForm>,
        node_id: NodeId,
    },
}
```

**Lowering**:

```rust
impl Lowerer {
    fn lower_expr(&mut self, form: &CoreForm) -> Result<syn::Expr> {
        match form {
            CoreForm::Borrow { expr, .. } => {
                let inner = self.lower_expr(expr)?;
                Ok(parse_quote! { &#inner })
            }

            CoreForm::BorrowMut { expr, .. } => {
                let inner = self.lower_expr(expr)?;
                Ok(parse_quote! { &mut #inner })
            }

            CoreForm::Deref { expr, .. } => {
                let inner = self.lower_expr(expr)?;
                Ok(parse_quote! { *#inner })
            }

            CoreForm::Clone { expr, .. } => {
                let inner = self.lower_expr(expr)?;
                Ok(parse_quote! { #inner.clone() })
            }

            // Move is the default in Rust, no special syntax
            CoreForm::Move { expr, .. } => {
                self.lower_expr(expr)
            }
        }
    }
}
```

### Trait Bounds

Core Forms can express trait bounds:

```rust
pub struct TraitBound {
    pub trait_: Symbol,
    pub lifetime: Option<Lifetime>,
}

pub enum CoreForm {
    // ... other forms

    DefineFunc {
        name: Symbol,
        type_params: Vec<TypeParam>,
        params: Vec<Param>,
        where_clause: Vec<WherePredicate>,
        // ...
    },
}

pub struct TypeParam {
    pub name: Symbol,
    pub bounds: Vec<TraitBound>,
}

pub struct WherePredicate {
    pub ty: Type,
    pub bounds: Vec<TraitBound>,
}
```

**Example**:

```lisp
;; Core Form
(define-func
  :name print-all
  :type-params [(type-param :name T
                            :bounds [(trait-bound Display)])]
  :params [(param :name items
                  :type (named Vec :args [(type-var T)]))]
  :body [...])
```

**Lowered Rust**:

```rust
fn print_all<T: Display>(items: Vec<T>) {
    // ...
}
```

---

## Stage 4: Generate (Rust AST → Rust Source)

### Purpose

Convert Rust AST structures into formatted, readable Rust source code.

### Implementation

```rust
pub struct Generator;

impl Generator {
    pub fn generate(item: &syn::Item) -> Result<String> {
        // Use prettyplease for formatting
        let file = syn::File {
            shebang: None,
            attrs: vec![],
            items: vec![item.clone()],
        };

        let source = prettyplease::unparse(&file);
        Ok(source)
    }

    pub fn generate_file(items: &[syn::Item]) -> Result<String> {
        let file = syn::File {
            shebang: None,
            attrs: vec![],
            items: items.to_vec(),
        };

        let source = prettyplease::unparse(&file);
        Ok(source)
    }
}
```

### Formatting Strategy

- Use `prettyplease` for automatic formatting
- Don't hand-optimize generated code - let rustfmt handle it
- Preserve structure for debugging (comments, spacing)

### Example Output

**Input Rust AST** (from lowering):

```rust
syn::ItemFn {
    sig: Signature {
        ident: "add",
        inputs: [
            FnArg::Typed { pat: "x", ty: "i32" },
            FnArg::Typed { pat: "y", ty: "i32" },
        ],
        output: ReturnType::Type("i32"),
    },
    block: Block {
        stmts: [
            Stmt::Expr(ExprBinary {
                left: Expr::Path("x"),
                op: BinOp::Add,
                right: Expr::Path("y"),
            })
        ]
    }
}
```

**Output Rust Source**:

```rust
fn add(x: i32, y: i32) -> i32 {
    x + y
}
```

### Notes

- **Source map ends here**: We don't track positions in generated Rust code
- Generated code should be readable for debugging
- rustc errors will reference generated .rs files
- We translate rustc errors back to Oxur source using source map

---

## Stage 5: Compile (Rust Source → Binary)

### Purpose

Invoke rustc to compile generated Rust code into executable binary or library.

### Implementation

```rust
pub struct RustCompiler {
    rustc_path: PathBuf,
    target_dir: PathBuf,
}

impl RustCompiler {
    pub fn compile(&self, source_files: &[PathBuf]) -> Result<PathBuf> {
        let mut cmd = Command::new(&self.rustc_path);

        for file in source_files {
            cmd.arg(file);
        }

        cmd.arg("--out-dir").arg(&self.target_dir);
        cmd.arg("--edition").arg("2021");

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::RustcError(stderr.to_string()));
        }

        // Return path to compiled binary
        Ok(self.target_dir.join("output"))
    }
}
```

### Error Translation

When rustc reports errors, we need to translate them back to original Oxur source:

```rust
pub struct ErrorTranslator {
    source_map: Arc<SourceMap>,
    generated_files: HashMap<PathBuf, String>,  // Generated .rs files
}

impl ErrorTranslator {
    pub fn translate_rustc_error(&self, error: &str) -> Result<String> {
        // Parse rustc error message
        let (file, line, col, message) = parse_rustc_error(error)?;

        // Find the Rust AST node at that position
        // (This is approximate - we don't have perfect position tracking in Stage 4)

        // Use source map to find original Oxur source
        // Walk backward through transformations

        // Format error with Oxur source context
        Ok(format_oxur_error(original_location, message))
    }
}
```

**Note**: Error translation is **approximate** because:

- Generated code may have been reformatted
- Multiple Oxur forms may map to same Rust code
- Some errors are in generated glue code

**Best effort approach**:

1. Use line/column from rustc error
2. Try to find corresponding Core Form via source map
3. If found, report original Oxur location
4. If not found, report generated Rust location with note

---

## Source Map Architecture

### Provenance Tracking Philosophy

Borrowed from Zylisp's successful approach: instead of trying to maintain positions through transformations, we:

1. **Assign unique IDs** to every AST node at parse time
2. **Track parent relationships** as transformations create new nodes
3. **Store original positions** only at the input layer
4. **Walk the chain backward** when reporting errors

This is how TypeScript, ClojureScript, Elm, and other compile-to-X languages work.

### Core Types

```rust
pub type NodeId = u64;

pub struct SourceLocation {
    pub file: PathBuf,
    pub start: SourcePosition,
    pub end: SourcePosition,
}

pub struct SourcePosition {
    pub line: usize,
    pub column: usize,
    pub offset: usize,  // Byte offset
}

pub struct SourceMap {
    /// Name of this transformation layer (for debugging)
    layer: String,

    /// Maps nodes in this layer to nodes in previous layer
    parent_node: HashMap<NodeId, NodeId>,

    /// Original source positions (only populated at input layer)
    original_pos: HashMap<NodeId, SourceLocation>,

    /// Link to previous transformation's source map
    previous: Option<Arc<SourceMap>>,
}
```

### Source Map Creation

Each compilation stage creates its own source map:

```rust
impl SourceMap {
    /// Create input layer source map (parser)
    pub fn new_input_layer(layer: &str) -> Self {
        Self {
            layer: layer.to_string(),
            parent_node: HashMap::new(),
            original_pos: HashMap::new(),
            previous: None,
        }
    }

    /// Create transformation layer source map
    pub fn new(layer: &str, previous: &SourceMap) -> Self {
        Self {
            layer: layer.to_string(),
            parent_node: HashMap::new(),
            original_pos: HashMap::new(),
            previous: Some(Arc::new(previous.clone())),
        }
    }

    /// Record original source position (input layer only)
    pub fn record_original(&mut self, node_id: NodeId, loc: SourceLocation) {
        self.original_pos.insert(node_id, loc);
    }

    /// Record that new_node_id was created from old_node_id
    pub fn record_transform(&mut self, new_node_id: NodeId, old_node_id: NodeId) {
        self.parent_node.insert(new_node_id, old_node_id);
    }
}
```

### Walking the Chain

To find the original source location for any node:

```rust
impl SourceMap {
    pub fn original_location(&self, node_id: NodeId) -> Option<SourceLocation> {
        let mut current = Some(self);
        let mut current_id = node_id;

        // Walk backward through transformation chain
        while let Some(map) = current {
            // Check if this layer has the original position
            if let Some(loc) = map.original_pos.get(&current_id) {
                return Some(loc.clone());
            }

            // Follow the parent link
            if let Some(&parent_id) = map.parent_node.get(&current_id) {
                current_id = parent_id;
                current = map.previous.as_deref();
            } else {
                // Dead end - no parent and no original position
                break;
            }
        }

        None
    }

    /// For debugging: show full provenance chain
    pub fn debug_trace(&self, node_id: NodeId) -> Vec<String> {
        let mut trace = Vec::new();
        let mut current = Some(self);
        let mut current_id = node_id;

        while let Some(map) = current {
            trace.push(format!("{}: node {}", map.layer, current_id));

            if let Some(loc) = map.original_pos.get(&current_id) {
                trace.push(format!(
                    "  → {}:{}:{}",
                    loc.file.display(),
                    loc.start.line,
                    loc.start.column
                ));
                break;
            }

            if let Some(&parent_id) = map.parent_node.get(&current_id) {
                current_id = parent_id;
                current = map.previous.as_deref();
            } else {
                trace.push("  → (no parent)".to_string());
                break;
            }
        }

        trace
    }
}
```

### Example Trace

```
Node 5000 in 'lower' layer
  → Node 200 in 'expand' layer
  → Node 100 in 'parse' layer
  → test.ox:5:3-5:15
```

This tells us:

- Current node (5000) was created during lowering
- It came from node 200 (created during expansion)
- That came from node 100 (created during parsing)
- The original source is at line 5, column 3 in test.ox

---

## Error Reporting

### Error Types

```rust
pub struct CompilerError {
    pub message: String,
    pub node_id: NodeId,
    pub source_map: Arc<SourceMap>,
    pub kind: ErrorKind,
}

pub enum ErrorKind {
    Parse,
    Expand,
    Lower,
    Generate,
    Compile,
}

impl CompilerError {
    pub fn new(
        message: impl Into<String>,
        node_id: NodeId,
        source_map: Arc<SourceMap>,
        kind: ErrorKind,
    ) -> Self {
        Self {
            message: message.into(),
            node_id,
            source_map,
            kind,
        }
    }
}
```

### Error Display

```rust
impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(loc) = self.source_map.original_location(self.node_id) {
            write!(
                f,
                "{}:{}:{}: {}",
                loc.file.display(),
                loc.start.line,
                loc.start.column,
                self.message
            )
        } else {
            write!(f, "(unknown location): {}", self.message)
        }
    }
}
```

### Error with Context

Show the relevant source code with a caret pointing to the error:

```rust
impl CompilerError {
    pub fn with_context(&self, source: &str) -> String {
        let Some(loc) = self.source_map.original_location(self.node_id) else {
            return self.to_string();
        };

        let lines: Vec<&str> = source.lines().collect();
        if loc.start.line < 1 || loc.start.line > lines.len() {
            return self.to_string();
        }

        let line = lines[loc.start.line - 1];

        let mut output = String::new();

        // Error message
        output.push_str(&format!(
            "{}:{}:{}: {}\n",
            loc.file.display(),
            loc.start.line,
            loc.start.column,
            self.message
        ));

        // Source line
        output.push_str(&format!("{}\n", line));

        // Caret pointing to error
        for _ in 1..loc.start.column {
            output.push(' ');
        }
        output.push('^');

        let length = loc.end.column.saturating_sub(loc.start.column);
        if length > 1 {
            for _ in 1..length {
                output.push('~');
            }
        }

        output
    }
}
```

**Example output**:

```
test.ox:5:10: undefined variable: foo
  (when (> foo 10)
           ^~~
```

### Debug Trace

For compiler developers, show the full provenance chain:

```rust
impl CompilerError {
    pub fn debug_trace(&self) -> String {
        let trace = self.source_map.debug_trace(self.node_id);
        let mut output = String::from("Error provenance:\n");
        for step in trace {
            output.push_str(&format!("  {}\n", step));
        }
        output
    }
}
```

**Example output**:

```
Error provenance:
  lower: node 5000
  expand: node 200
  parse: node 100
  → test.ox:5:10-5:13
```

---

## REPL Architecture

### The Key Advantage: No Plugin Memory Leak

Unlike Go, Rust doesn't have the plugin memory leak problem. This dramatically simplifies the REPL architecture:

**Zylisp REPL**:

- Supervised worker processes
- IPC communication
- Memory monitoring
- Restart workers when memory threshold hit
- Complex but necessary

**Oxur REPL**:

- Single process
- Direct function calls
- Simple caching
- No memory leaks!

### Three-Tier Execution Strategy

Optimize common cases while maintaining full compilation capability:

```
┌─────────────────────────────────────────┐
│         Tier 1: Interpreter             │
│                                         │
│  For simple Core Forms:                 │
│  • Literals                             │
│  • Simple arithmetic                    │
│  • Variable lookups                     │
│                                         │
│  Time: ~1ms                             │
└─────────────────────────────────────────┘
                 ↓ (if not simple)
┌─────────────────────────────────────────┐
│         Tier 2: Cached Compilation      │
│                                         │
│  For function definitions:              │
│  • Compile once to dynamic library      │
│  • Keep loaded (no memory leak!)        │
│  • Subsequent calls are instant         │
│                                         │
│  Time: ~0ms after first compile         │
└─────────────────────────────────────────┘
                 ↓ (if not cached)
┌─────────────────────────────────────────┐
│         Tier 3: JIT Compilation         │
│                                         │
│  For complex expressions:               │
│  • Full compilation pipeline            │
│  • Cache the result                     │
│  • Show progress if >50ms               │
│                                         │
│  Time: 50-200ms first time              │
└─────────────────────────────────────────┘
```

### Tier 1: Interpreter Implementation

```rust
pub struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub fn eval(&mut self, form: &CoreForm) -> Result<Value> {
        match form {
            CoreForm::Literal(lit) => Ok(Value::from(lit)),

            CoreForm::VarRef(sym) => {
                self.env.get(sym)
                    .ok_or_else(|| Error::UndefinedVariable(sym.clone()))
            }

            CoreForm::BinaryOp { op, left, right, .. } => {
                let left_val = self.eval(left)?;
                let right_val = self.eval(right)?;
                self.eval_binary_op(*op, left_val, right_val)
            }

            // For anything complex, return None to fall through to compilation
            _ => Err(Error::NotInterpretable),
        }
    }

    fn eval_binary_op(&self, op: BinOp, left: Value, right: Value) -> Result<Value> {
        match (op, left, right) {
            (BinOp::Add, Value::Int(l), Value::Int(r)) => Ok(Value::Int(l + r)),
            (BinOp::Sub, Value::Int(l), Value::Int(r)) => Ok(Value::Int(l - r)),
            (BinOp::Mul, Value::Int(l), Value::Int(r)) => Ok(Value::Int(l * r)),
            (BinOp::Div, Value::Int(l), Value::Int(r)) => Ok(Value::Int(l / r)),
            // ... more operations
            _ => Err(Error::TypeMismatch),
        }
    }
}

pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    // ... more value types
}
```

### Tier 2: Compilation Cache

```rust
pub struct CompilationCache {
    libraries: HashMap<FunctionId, CompiledFunction>,
}

pub struct CompiledFunction {
    lib: Library,  // from libloading
    func: Symbol<'static, unsafe extern "C" fn(*const Value) -> Value>,
}

impl CompilationCache {
    pub fn get_or_compile(
        &mut self,
        id: &FunctionId,
        form: &CoreForm,
    ) -> Result<&CompiledFunction> {
        if !self.libraries.contains_key(id) {
            let compiled = self.compile_function(form)?;
            self.libraries.insert(id.clone(), compiled);
        }

        Ok(&self.libraries[id])
    }

    fn compile_function(&self, form: &CoreForm) -> Result<CompiledFunction> {
        // Full compilation pipeline
        let rust_ast = lower(form)?;
        let rust_code = generate(&rust_ast)?;

        // Write to temporary file
        let temp_file = self.temp_dir.join(format!("repl_{}.rs", uuid()));
        std::fs::write(&temp_file, rust_code)?;

        // Compile to dynamic library
        let lib_path = rustc_compile_dylib(&temp_file)?;

        // Load library
        let lib = unsafe { Library::new(lib_path)? };
        let func = unsafe { lib.get(b"repl_function")? };

        Ok(CompiledFunction { lib, func })
    }
}
```

### REPL Server Architecture

```rust
pub struct ReplServer {
    interpreter: Interpreter,
    cache: CompilationCache,
    source_map: Arc<SourceMap>,
}

impl ReplServer {
    pub fn eval(&mut self, source: &str) -> Result<String> {
        // Parse
        let surface_forms = parse(source, PathBuf::from("<repl>"))?;

        // Expand
        let core_forms = expand(surface_forms)?;

        // Try Tier 1: Interpretation
        if let Ok(result) = self.interpreter.eval(&core_forms[0]) {
            return Ok(format!("{:?}", result));
        }

        // Try Tier 2: Cached compilation
        if let CoreForm::DefineFunc { name, .. } = &core_forms[0] {
            let func = self.cache.get_or_compile(name, &core_forms[0])?;
            // Store in environment for future use
            self.interpreter.env.insert(name.clone(), func);
            return Ok(format!("Defined function: {}", name));
        }

        // Tier 3: JIT compile and execute
        let result = self.compile_and_run(&core_forms[0])?;
        Ok(format!("{:?}", result))
    }
}
```

### No Supervision Needed

Because Rust doesn't have the plugin memory leak:

```rust
// Simple single-process REPL
fn main() {
    let mut repl = ReplServer::new();

    loop {
        print!("> ");
        let input = read_line();

        match repl.eval(&input) {
            Ok(result) => println!("{}", result),
            Err(err) => eprintln!("Error: {}", err.with_context(&input)),
        }
    }
}
```

No need for:

- Worker processes
- IPC communication
- Memory monitoring
- Process supervision
- Complex restart logic

**This is a huge simplification compared to Zylisp!**

### REPL Client/Server (Optional)

For remote REPL connections or multiple clients, we can still use an nREPL-style architecture:

```rust
pub struct ReplClient {
    connection: TcpStream,
}

impl ReplClient {
    pub fn eval(&mut self, source: &str) -> Result<String> {
        // Send evaluation request
        self.send_request(Request::Eval { source: source.to_string() })?;

        // Receive response
        let response = self.recv_response()?;

        Ok(response)
    }
}

pub struct ReplServer {
    listener: TcpListener,
    eval_server: ReplEvalServer,
}

impl ReplServer {
    pub fn run(&mut self) -> Result<()> {
        for stream in self.listener.incoming() {
            let stream = stream?;
            self.handle_client(stream)?;
        }
        Ok(())
    }

    fn handle_client(&mut self, mut stream: TcpStream) -> Result<()> {
        loop {
            let request: Request = serde_json::from_reader(&mut stream)?;

            let response = match request {
                Request::Eval { source } => {
                    self.eval_server.eval(&source)?
                }
            };

            serde_json::to_writer(&mut stream, &response)?;
        }
    }
}
```

But for local development, a simple single-process REPL is sufficient!

---

## Macro System

### Phase 1: Core Macros (v1.0)

**Philosophy**: Ship with a rich set of pre-compiled core macros that cover common patterns.

**Implementation**:

- Core macros are written in Oxur
- Compiled once by the Oxur team
- Shipped as `core-macros.so` dynamic library
- Loaded at compiler startup

**Core Macro Registry**:

```rust
pub struct CoreMacroRegistry {
    macros: HashMap<Symbol, MacroExpander>,
}

pub type MacroExpander = fn(&[SurfaceForm]) -> Result<CoreForm>;

impl CoreMacroRegistry {
    pub fn load_builtin() -> Self {
        let lib = unsafe {
            Library::new("core-macros.so")
                .expect("Failed to load core macros")
        };

        let mut macros = HashMap::new();

        // Load each macro expander
        unsafe {
            let when_macro: Symbol<MacroExpander> =
                lib.get(b"when_macro").unwrap();
            macros.insert(symbol("when"), *when_macro);

            let unless_macro: Symbol<MacroExpander> =
                lib.get(b"unless_macro").unwrap();
            macros.insert(symbol("unless"), *unless_macro);

            // ... more macros
        }

        Self { macros }
    }

    pub fn get(&self, name: &Symbol) -> Option<&MacroExpander> {
        self.macros.get(name)
    }
}
```

**Planned Core Macros**:

- **Control flow**: `when`, `unless`, `cond`
- **Threading**: `->`, `->>`
- **Let variants**: `when-let`, `if-let`
- **Loop helpers**: `dotimes`, `doseq`
- **Function utilities**: `partial`, `comp`
- **Debugging**: `assert`, `debug`

### Phase 2: User Macros (v2.0)

**Philosophy**: Enable users to extend the language with their own macros, compiled to native code.

**Compilation Process**:

```
User Project Files
    ↓
Pass 0: Extract Macro Definitions
    ↓
Pass 1: Build Dependency Graph
    ↓ (detect cycles, compute layers)
Pass 2: Compile Macros Layer-by-Layer
    ↓ (produces user-macros.so)
Pass 3: Load All Macro Libraries
    ↓ (core-macros.so + user-macros.so)
Pass 4+: Normal Compilation
```

**Dependency Graph**:

```rust
pub struct MacroGraph {
    nodes: HashMap<Symbol, MacroNode>,
    edges: HashMap<Symbol, Vec<Symbol>>,  // Dependencies
}

pub struct MacroNode {
    definition: MacroDefinition,
    layer: Option<usize>,
}

impl MacroGraph {
    pub fn build(macros: Vec<MacroDefinition>) -> Result<Self> {
        let mut graph = Self::new();

        for macro_def in macros {
            graph.add_node(macro_def);
        }

        // Detect cycles
        graph.detect_cycles()?;

        // Compute layers (topological sort)
        graph.compute_layers()?;

        Ok(graph)
    }

    pub fn detect_cycles(&self) -> Result<()> {
        // DFS-based cycle detection
        // Error if cycle found
    }

    pub fn compute_layers(&mut self) -> Result<()> {
        // Topological sort to determine compilation order
        // Layer 0: No dependencies
        // Layer N: Depends on things in layers < N
    }
}
```

**User Macro Definition**:

```lisp
(defmacro when-let [binding test & body]
  `(let [~binding ~test]
     (when ~binding
       ~@body)))
```

**Compilation**:

```rust
fn compile_macro_layer(
    macros: Vec<MacroDefinition>,
    previous_layers: &[MacroLibrary],
) -> Result<MacroLibrary> {
    // Generate Rust code for macro expanders
    let rust_code = generate_macro_expanders(macros)?;

    // Compile to dynamic library
    let lib_path = rustc_compile_dylib(rust_code)?;

    // Load library
    MacroLibrary::load(lib_path)
}
```

### Reader Macros (Tagged Literals)

**Phase 1 Support**: Tagged literals with user-defined expanders

**Syntax**:

```lisp
#timestamp<2025-12-27T10:30:00Z>
#uuid<550e8400-e29b-41d4-a716-446655440000>
#json{"key": "value"}
#regex</\d{3}-\d{3}-\d{4}/>
```

**Registration**:

```lisp
(register-tag-expander 'timestamp parse-timestamp)
(register-tag-expander 'uuid parse-uuid)
```

**Expansion**:

- Happens during macro expansion phase
- Tagged literals become function calls
- Functions can validate at compile time

**Example**:

```lisp
;; Definition
(deffunc parse-timestamp [s]
  (time::parse-iso8601 s))

(register-tag-expander 'timestamp parse-timestamp)

;; Usage
(let [t #timestamp<2025-12-27T10:30:00Z>]
  (time::format t "%Y-%m-%d"))

;; Expands to
(let [t (parse-timestamp "2025-12-27T10:30:00Z")]
  (time::format t "%Y-%m-%d"))
```

---

## Repository Structure

### Overview

```
github.com/oxur/crates
├── oxur-ast/           # Rust AST ↔ Core Forms (Stage 3)
├── oxur-comp/          # Oxur compiler (Stages 1-2)
├── oxur-lang/          # Oxur lang. def., core forms/macros
├── oxur-repl/          # REPL server/client
├── oxur-cli/           # CLI tool (oxur command)
└── design/             # Design documents
```

### Dependency Graph

```
cli → repl → oxur-lang → oxur-comp → oxur-ast
              ↓
        (core-macros.so compiled from oxur-lang/core-macros/)
```

**Key**: No circular dependencies!

### Repository Details

#### `oxur/crates/oxur-ast`

**Purpose**: Rust AST ↔ Core Forms bidirectional conversion

**Structure**:

```
oxur-ast/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── ast/
|   ├── ...
│   └── sexp/
└── tests/
    └── round_trip_tests.rs
```

**Key Characteristics**:

- Stable foundation
- Rarely changes
- Well-tested
- No dependencies on `oxur-comp`

#### `oxur/crates/oxur-comp`

**Purpose**: Oxur language compiler

**Structure**:

```
oxur-comp/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── parse/          # Stage 1: Parse
│   │   ├── mod.rs
│   │   ├── lexer.rs
│   │   ├── reader.rs
│   │   └── tokens.rs
│   ├── expand/         # Stage 2: Expand
│   │   ├── mod.rs
│   │   ├── expander.rs
│   │   ├── core_registry.rs
│   │   └── desugar.rs
│   ├── sourcemap/      # Source map infrastructure
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   └── idgen.rs
│   ├── error/          # Error types and reporting
│   │   ├── mod.rs
│   │   └── display.rs
│   └── compile.rs      # Orchestration
└── tests/
```

**Dependencies**:

- `oxur-ast` for Core Forms
- `syn` for Rust AST types
- `libloading` for loading macro libraries

#### `oxur/crates/oxur-lang`

```
oxur-lang/       # Oxur language definition
├── forms/       # Standard forms, sugar
├── core-macros/ # Core macro definitions (written in Oxur)
│   ├── control.oxr
│   ├── threading.oxr
│   └── lib.oxr
├── stdlib/      # Standard library (maybe later)
└── tests/
```

**Build Process**:

```rust
// build.rs
fn main() {
    // Use bootstrap Oxur compiler to compile core macros
    let macros = vec![
        "src/control.ox",
        "src/threading.ox",
        "src/let.ox",
    ];

    compile_core_macros(macros, "target/core-macros.so");
}
```

#### `oxur/crates/oxur-repl`

**Purpose**: REPL server and client

**Structure**:

```
oxur-repl/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── server/         # REPL server
│   │   ├── mod.rs
│   │   └── eval.rs
│   ├── interp/         # Tier 1 interpreter
│   │   ├── mod.rs
│   │   └── eval.rs
│   ├── cache/          # Tier 2 compilation cache
│   │   ├── mod.rs
│   │   └── compile.rs
│   └── client/         # REPL client library
│       ├── mod.rs
│       └── connection.rs
└── tests/
```

#### `oxur/crates/oxur-cli`

**Purpose**: Command-line interface

**Structure**:

```
oxur-cli/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── build.rs
│   │   ├── repl.rs
│   │   ├── check.rs
│   │   └── format.rs
│   └── config.rs
└── tests/
```

**Commands**:

- `oxur build` - Compile project
- `oxur repl` (and `oxur` with no command) - Start REPL
- `oxur check` - Type check without compiling
- `oxur format` - Format Oxur code
- `oxur test` - Run tests

---

## Development Phases

### Phase 0: Foundation (Weeks 1-2)

**Goal**: Set up project structure and basic tooling

**Deliverables**:

- [ ] Create repositories (`oxur-ast`, `oxur-lang`, `oxur-repl`, `oxur-cli`, `oxur-comp`)
- [ ] Set up CI/CD
- [ ] Define Core Forms specification (detailed document)
- [ ] Implement Node ID generator and source map types
- [ ] Write project README and contribution guidelines

### Phase 1: Parse & Source Maps (Weeks 3-4)

**Goal**: Implement Stage 1 (Parse) with source map tracking

**Deliverables**:

- [ ] Lexer implementation with position tracking
- [ ] Reader implementation (tokens → Surface Forms)
- [ ] Node ID assignment for all forms
- [ ] Input layer source map creation
- [ ] Comprehensive tests for parser
- [ ] Parse error reporting with context

### Phase 2: Core Forms & Lowering (Weeks 5-6)

**Goal**: Define Core Forms and implement Stage 3 (Lower)

**Deliverables**:

- [ ] Complete Core Form types in `oxur-ast`
- [ ] Core Form → Rust AST lowering
- [ ] Rust AST → Core Form lifting (for testing)
- [ ] Round-trip tests (Rust → Core Forms → Rust)
- [ ] Source map for lowering stage

### Phase 3: Expansion (Weeks 7-8)

**Goal**: Implement Stage 2 (Expand) with core macros

**Deliverables**:

- [ ] Expander implementation
- [ ] Desugaring for common syntax sugar
- [ ] Core macro framework
- [ ] Implement 3-5 essential core macros
- [ ] Macro expansion tests
- [ ] Source map for expansion stage

### Phase 4: Generation & End-to-End (Weeks 9-10)

**Goal**: Complete the pipeline, compile Hello World

**Deliverables**:

- [ ] Stage 4 (Generate) implementation
- [ ] Stage 5 (Compile) integration
- [ ] End-to-end compilation test
- [ ] Compile and run "Hello, World!"
- [ ] Error translation from rustc errors
- [ ] Full pipeline integration tests

### Phase 5: Core Macros Library (Weeks 11-12)

**Goal**: Build comprehensive core macro library

**Deliverables**:

- [ ] Control flow macros (`when`, `unless`, `cond`)
- [ ] Threading macros (`->`, `->>`)
- [ ] Let variants (`when-let`, `if-let`)
- [ ] Loop helpers (`dotimes`, `doseq`)
- [ ] Core macro compilation infrastructure
- [ ] Core macro tests
- [ ] Documentation for core macros

### Phase 6: REPL (Weeks 13-14)

**Goal**: Build working REPL with tiered execution

**Deliverables**:

- [ ] Tier 1 interpreter for simple forms
- [ ] Tier 2 compilation cache
- [ ] Tier 3 JIT compilation
- [ ] REPL server implementation
- [ ] REPL client (CLI)
- [ ] REPL tests
- [ ] REPL user documentation

### Phase 7: CLI & Tooling (Weeks 15-16)

**Goal**: Polish CLI and add essential tools

**Deliverables**:

- [ ] `oxur build` command
- [ ] `oxur check` command
- [ ] `oxur format` command
- [ ] Project configuration (`project.ox`)
- [ ] Build caching
- [ ] Error message improvements
- [ ] CLI documentation

### Phase 8: v1.0 Release (Weeks 17-18)

**Goal**: Polish, documentation, release

**Deliverables**:

- [ ] Complete documentation
- [ ] Tutorial and examples
- [ ] Performance benchmarks
- [ ] Release notes
- [ ] Blog post announcing v1.0
- [ ] Package for distribution

---

## Testing Strategy

### Unit Tests

Each module should have comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_integers() {
        let mut lexer = Lexer::new("42 -17 0");

        assert_eq!(lexer.next_token(), Token::Integer(42));
        assert_eq!(lexer.next_token(), Token::Integer(-17));
        assert_eq!(lexer.next_token(), Token::Integer(0));
    }

    #[test]
    fn test_source_map_chain() {
        let input_map = SourceMap::new_input_layer("parse");
        input_map.record_original(1, location(1, 1));

        let expand_map = SourceMap::new("expand", &input_map);
        expand_map.record_transform(100, 1);

        let loc = expand_map.original_location(100);
        assert_eq!(loc.unwrap().start.line, 1);
    }
}
```

### Integration Tests

Test the full pipeline with realistic examples:

```rust
#[test]
fn test_compile_simple_function() {
    let source = r#"
        (defn add [x y]
          (+ x y))
    "#;

    let result = compile_to_rust(source)?;

    assert!(result.contains("fn add"));
    assert!(result.contains("x + y"));
}

#[test]
fn test_macro_expansion() {
    let source = r#"
        (when (> x 10)
          (println x))
    "#;

    let expanded = parse_and_expand(source)?;

    // Should expand to if-expr
    assert!(matches!(expanded, CoreForm::IfExpr { .. }));
}
```

### Round-Trip Tests

Ensure bidirectional conversion works:

```rust
#[test]
fn test_rust_to_core_forms_to_rust() {
    let original_rust = r#"
        fn factorial(n: u64) -> u64 {
            if n == 0 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
    "#;

    // Parse Rust to AST
    let rust_ast = syn::parse_str::<syn::ItemFn>(original_rust)?;

    // Lift to Core Forms
    let core_form = lift_to_core_form(&rust_ast)?;

    // Lower back to Rust AST
    let lowered_ast = lower_to_rust(&core_form)?;

    // Generate Rust source
    let generated_rust = generate(&lowered_ast)?;

    // Compare (after normalizing formatting)
    assert_equivalent(original_rust, generated_rust);
}
```

### Error Reporting Tests

Ensure errors are reported correctly:

```rust
#[test]
fn test_undefined_variable_error() {
    let source = "(+ x 10)";

    let err = compile(source).unwrap_err();

    assert!(err.message.contains("undefined variable"));
    assert_eq!(err.kind, ErrorKind::Expand);

    let display = err.with_context(source);
    assert!(display.contains("(+ x 10)"));
    assert!(display.contains("^"));
}
```

### Performance Benchmarks

Track compilation speed:

```rust
#[bench]
fn bench_parse_large_file(b: &mut Bencher) {
    let source = include_str!("large_test_file.ox");

    b.iter(|| {
        parse(source, PathBuf::from("test.ox"))
    });
}

#[bench]
fn bench_repl_simple_expr(b: &mut Bencher) {
    let mut repl = ReplServer::new();

    b.iter(|| {
        repl.eval("(+ 1 2)")
    });
}
```

---

## Performance Considerations

### Compilation Speed

**Target**: Compile 1000 LOC in <1 second (excluding rustc time)

**Strategies**:

- **Incremental compilation**: Only recompile changed modules
- **Parallel compilation**: Compile independent modules in parallel
- **Caching**: Cache macro expansions, lowering results
- **Fast paths**: Optimize common patterns

### REPL Responsiveness

**Target**: <1ms for simple expressions, <50ms for compilation

**Tier Performance**:

- Tier 1 (Interpreter): <1ms
- Tier 2 (Cached): ~0ms (just function call)
- Tier 3 (JIT): 50-200ms first time, cached after

### Memory Usage

**Target**: Reasonable memory usage (<500MB for typical projects)

**Strategies**:

- **Streaming compilation**: Don't load entire project into memory
- **Efficient data structures**: Use `Arc<str>` for shared strings
- **Clean up temporary files**: Delete generated .rs files after compilation

### Generated Code Quality

**Target**: Generated Rust code should be idiomatic and performant

**Strategies**:

- Use `prettyplease` for formatting
- Let rustc optimize (don't try to optimize ourselves)
- Generate simple, straightforward Rust code
- Trust the compiler

---

## Open Questions

### 1. Type Inference vs Explicit Types

**Question**: How much type inference should Oxur have?

**Options**:

- **Full inference**: Let rustc infer everything
- **Function signatures only**: Require types on function params/returns
- **Opt-in annotations**: Support type annotations but don't require them

**Recommendation**: Start with function signatures required, inference everywhere else.

### 2. Error Message Philosophy

**Question**: Should error messages be verbose or concise?

**Consideration**:

- Verbose helps beginners
- Concise is faster for experts
- Can we have both?

**Recommendation**: Verbose by default, add `--terse` flag for experts.

### 3. Module System

**Question**: Follow Rust's module system exactly, or add Lisp-style packages?

**Options**:

- **Pure Rust**: `mod`, `use`, exactly like Rust
- **Lisp packages**: Separate namespace concept
- **Hybrid**: Rust modules + package metadata

**Recommendation**: Pure Rust for v1.0, consider packages in v2.0.

### 4. Standard Library

**Question**: What should be in Oxur's standard library?

**Consideration**:

- Re-export Rust std?
- Lisp-friendly wrappers?
- Entirely separate?

**Recommendation**: Start with thin wrappers over Rust std, grow organically.

### 5. Unsafe Code

**Question**: How do we handle Rust's `unsafe` keyword?

**Options**:

- **Forbid it**: No unsafe in Oxur
- **Special form**: `(unsafe ...)`
- **Transparent**: Just compiles to unsafe

**Recommendation**: Support with explicit `(unsafe ...)` form, add warnings.

---

## Conclusion

This architecture provides:

✅ **Clear separation of concerns** - each stage has one job
✅ **Stable intermediate representation** - Core Forms are the contract
✅ **Accurate error reporting** - source maps track every transformation
✅ **Fast REPL** - tiered execution optimizes common cases
✅ **Native performance** - compiles to idiomatic Rust
✅ **Extensibility** - macro system designed from day one
✅ **No runtime overhead** - Rust's type system is powerful enough
✅ **Simplified design** - no plugin memory leaks, no supervised workers

### Key Innovations Over Zylisp

1. **No plugin memory leak** → Simpler REPL (no workers needed)
2. **Richer type system** → No runtime library needed
3. **Better pattern matching** → More elegant lowering
4. **Cleaner AST** → Easier to work with
5. **Pre-compiled macros** → Macro system designed for native compilation

### Next Steps

1. **Review this document** with the team
2. **Create Core Forms specification** (detailed, separate document)
3. **Set up repositories** and project structure
4. **Begin Phase 0** (foundation)
5. **Start coding!** 🦀

---

**"In Lisp, code is data. In Rust, safety is fearless. In Oxur, we get both."**

---

## Appendix A: Glossary

**Surface Forms**: Parsed S-expressions before macro expansion, with all syntactic sugar preserved

**Core Forms**: Canonical S-expressions after macro expansion and desugaring - the IR

**IR (Intermediate Representation)**: The canonical form between source language and target language

**Node ID**: Unique identifier assigned to every AST node for provenance tracking

**Source Map**: Data structure tracking transformations from original source through compilation

**Lowering**: Converting high-level Core Forms to lower-level Rust AST

**Lifting**: Converting Rust AST back to Core Forms (for inspection/debugging)

**Macro Expansion**: Transforming macro calls into their expanded forms

**Desugaring**: Converting syntactic sugar into canonical forms

---

## Appendix B: References

### Inspiration from Zylisp

- Two-stage compilation architecture
- Node ID-based source mapping
- Tiered REPL execution
- Core Forms as stable IR
- Macro compilation strategy

### Rust Resources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [syn crate documentation](https://docs.rs/syn/)
- [quote macro guide](https://docs.rs/quote/)
- [libloading docs](https://docs.rs/libloading/)

### Lisp Resources

- *Lisp in Small Pieces* by Christian Queinnec
- *Let Over Lambda* by Doug Hoyte
- Common Lisp HyperSpec
- Zetalisp documentation

---

*End of Document*
