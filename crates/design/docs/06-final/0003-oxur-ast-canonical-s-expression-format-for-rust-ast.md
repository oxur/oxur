---
number: 3
title: "oxur-ast: Canonical S-Expression Format for Rust AST"
author: "Duncan McGreggor"
component: AST
tags: [sexpr, syntax]
created: 2025-12-27
updated: 2026-01-05
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# oxur-ast: Canonical S-Expression Format for Rust AST

**Version**: 0.1.0 (Phase 1 - Hello World)
**Date**: December 2025
**Status**: Design Specification

---

## Executive Summary

This document defines the canonical S-expression format for representing Rust's Abstract Syntax Tree (AST). This format provides a 1:1 bidirectional mapping between Rust AST nodes and S-expressions, serving as the stable intermediate representation (IR) between Oxur source code and compiled Rust binaries.

**Key decisions:**

- Direct mapping of Rust's `Foo` + `FooKind` enum pattern to S-expressions
- Keyword arguments for all fields (`:field value`)
- Complete preservation of position information via `Span`
- No abstraction or simplification - faithful representation of `rustc_ast`
- Stable "assembly language" for Rust

---

## Table of Contents

1. [Design Principles](#1-design-principles)
2. [Core Syntax Rules](#2-core-syntax-rules)
3. [Crate Structure (Top Level)](#3-crate-structure-top-level)
4. [Phase 1 Node Types](#4-phase-1-node-types)
5. [Span and Position Tracking](#5-span-and-position-tracking)
6. [Complete Example: Hello World](#6-complete-example-hello-world)
7. [Implementation Notes](#7-implementation-notes)
8. [Future Phases](#8-future-phases)
9. [Comparison to Go's Approach](#9-comparison-to-gos-approach)

---

## 1. Design Principles

### 1.1 Faithful Representation

**Every field in Rust's AST is represented.** We don't simplify, abstract, or "improve" the AST. If `rustc_ast` has it, we represent it.

**Why:** The canonical format is the contract between Stage 1 (Oxur compiler) and Stage 2 (Rust code generation). It must be:

- Complete (no information loss)
- Unambiguous (one AST = one S-expression)
- Stable (changes infrequently)
- Verifiable (round-trip guarantees)

### 1.2 Rust's Enum Pattern

Rust's AST uses a consistent pattern:

```rust
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
    pub span: Span,
    // ... other metadata
}

pub enum ExprKind {
    Array(Vec<P<Expr>>),
    Call(P<Expr>, Vec<P<Expr>>),
    Binary(BinOp, P<Expr>, P<Expr>),
    // ... ~50 more variants
}
```

**Our S-expression mapping:**

```lisp
(Expr
  :id 123
  :kind (Array :elems ((Lit ...) (Lit ...)))
  :span (Span :lo 10 :hi 25))
```

The `kind` field contains the nested enum variant with its own fields.

### 1.3 Keyword Arguments

All fields use keyword syntax:

```lisp
(NodeType :field1 value1 :field2 value2 :field3 value3)
```

**Benefits:**

- Self-documenting
- Order-independent parsing (though we'll maintain canonical order)
- Easy to add optional fields
- Clear distinction between field names and values
- Familiar to Zetalisp and Common Lisp developers

### 1.4 Span Preservation

Rust uses `Span` for all position tracking (not `token.Pos` like Go):

```rust
pub struct Span {
    lo: BytePos,  // Start position
    hi: BytePos,  // End position
    ctxt: SyntaxContext,  // Hygiene context
}
```

We represent this completely, allowing perfect error reporting back to original source.

### 1.5 Explicit Nulls and Options

Rust uses `Option<T>` extensively. We represent:

- `None` as `nil`
- `Some(value)` as the value directly (the `Some` is implicit in the field's type)

Empty collections are represented as `()`.

---

## 2. Core Syntax Rules

### 2.1 Node Format

```lisp
(NodeType :field1 value1 :field2 value2 ...)
```

### 2.2 Enum Variant Format

Rust enums map to S-expressions with the variant name as the node type:

```rust
// Rust
ExprKind::Binary(BinOpKind::Add, lhs, rhs)

// S-expression
(Binary :op Add :left <expr> :right <expr>)
```

### 2.3 Value Types

**Symbols (unquoted):**

```lisp
Add Sub Mul Div  ; Operators
i32 u64 bool     ; Type names
pub const mut    ; Keywords
```

**Strings (quoted):**

```lisp
"hello"
"foo"
"x"
```

**Numbers:**

```lisp
42
0
123
```

**Identifiers:**

```lisp
(Ident :name "main" :span ...)
```

**Lists:**

```lisp
:items ((Item ...) (Item ...) (Item ...))
```

**Empty lists:**

```lisp
:items ()
```

**Nil:**

```lisp
:doc nil
:recv nil
```

### 2.4 Boxed Values

Rust uses `Box<T>` and `P<T>` (a smart pointer) extensively. In S-expressions, we just represent the inner value:

```rust
// Rust
P<Expr>

// S-expression
(Expr ...)  ; No special notation for the pointer
```

The pointer is implicit in the structure.

---

## 3. Crate Structure (Top Level)

### 3.1 The Root: Crate

```lisp
(Crate
  :attrs <attributes>
  :items <items>
  :spans <mod-spans>
  :id <node-id>
  :is-placeholder <bool>)
```

**Fields:**

- `:attrs` - `AttrVec` - Crate-level attributes (derive, feature gates, etc.)
- `:items` - `ThinVec<P<Item>>` - Top-level declarations
- `:spans` - `ModSpans` - Source location information
- `:id` - `NodeId` - Unique identifier for this crate
- `:is-placeholder` - `bool` - Used during parsing

### 3.2 AttrVec (Attribute Vector)

```lisp
:attrs ((Attribute ...) (Attribute ...) ...)
```

Or for empty:

```lisp
:attrs ()
```

### 3.3 ModSpans

```lisp
(ModSpans
  :inner-span <span>
  :inject-use-span <span>)
```

This tracks the span of the module's contents.

---

## 4. Phase 1 Node Types

Phase 1 focuses on a minimal "Hello, World!" program. We need:

```rust
fn main() {
    println!("Hello, world!");
}
```

This requires:

1. `Crate` (root)
2. `Item` + `ItemKind::Fn` (function declaration)
3. `FnSig` (function signature)
4. `FnDecl` (function declaration details)
5. `Block` (function body)
6. `Stmt` + `StmtKind` (statements)
7. `Expr` + `ExprKind` (expressions)
8. `MacCall` (for `println!` macro)
9. `Attribute` (for any attributes)
10. `Ident` (identifiers)
11. `Path` (for macro paths)
12. `Span` (positions)

### 4.1 Item

```lisp
(Item
  :attrs <attr-vec>
  :id <node-id>
  :span <span>
  :vis <visibility>
  :ident <ident>
  :kind <item-kind>
  :tokens nil)  ; TokenStream, usually nil for our purposes
```

**Fields:**

- `:attrs` - Attributes on this item
- `:id` - Node ID
- `:span` - Source span
- `:vis` - Visibility (pub, pub(crate), etc.)
- `:ident` - Item name
- `:kind` - The actual item type (enum variant)
- `:tokens` - Lazy token stream (usually nil)

### 4.2 ItemKind (Enum)

For Phase 1, we only need `Fn`:

```lisp
(Fn
  :defaultness <defaultness>
  :sig <fn-sig>
  :generics <generics>
  :body <block>)
```

**Full `ItemKind` variants** (for future phases):

- `ExternCrate`
- `Use`
- `Static`
- `Const`
- `Fn` ← Phase 1
- `Mod`
- `ForeignMod`
- `GlobalAsm`
- `TyAlias`
- `Enum`
- `Struct`
- `Union`
- `Trait`
- `TraitAlias`
- `Impl`
- `MacCall`
- `MacroDef`

### 4.3 FnSig (Function Signature)

```lisp
(FnSig
  :header <fn-header>
  :decl <fn-decl>
  :span <span>)
```

### 4.4 FnHeader

```lisp
(FnHeader
  :safety <safety>
  :coroutine-kind nil
  :constness <constness>
  :ext <extern>)
```

**Fields:**

- `:safety` - `Safe` or `Unsafe` or `Default`
- `:coroutine-kind` - `None`, `Async`, or `Gen`
- `:constness` - `Const` or `NotConst`
- `:ext` - `None` or `(Explicit <abi>)`

### 4.5 FnDecl (Function Declaration)

```lisp
(FnDecl
  :inputs <params>
  :output <return-type>)
```

**Fields:**

- `:inputs` - `Vec<Param>` - Parameter list
- `:output` - `FnRetTy` - Return type

### 4.6 Param

```lisp
(Param
  :attrs <attr-vec>
  :ty <type>
  :pat <pattern>
  :id <node-id>
  :span <span>
  :is-placeholder <bool>)
```

### 4.7 FnRetTy (Return Type)

```lisp
; No return type (unit)
(Default <span>)

; Explicit return type
(Ty <type>)
```

### 4.8 Block

```lisp
(Block
  :stmts <statements>
  :id <node-id>
  :rules <block-check-mode>
  :span <span>
  :tokens nil
  :could-be-bare-literal <bool>)
```

**Fields:**

- `:stmts` - `Vec<Stmt>` - Statements in the block
- `:id` - Node ID
- `:rules` - `Default` or `Unsafe`
- `:span` - Source span
- `:tokens` - Lazy token stream
- `:could-be-bare-literal` - Parsing hint

### 4.9 Stmt + StmtKind

```lisp
(Stmt
  :id <node-id>
  :kind <stmt-kind>
  :span <span>)
```

**StmtKind variants:**

```lisp
; Expression statement
(Expr <expr>)

; Expression with semicolon
(Semi <expr>)

; Let binding
(Let <local>)

; Item declaration
(Item <item>)

; Macro invocation
(MacCall <mac-call-stmt>)

; Empty statement
Empty
```

### 4.10 Expr + ExprKind

```lisp
(Expr
  :id <node-id>
  :kind <expr-kind>
  :span <span>
  :attrs <attr-vec>
  :tokens nil)
```

**ExprKind variants for Phase 1:**

```lisp
; Macro invocation (for println!)
(MacCall <mac-call>)

; Literal
(Lit <token-lit>)

; Path (identifier)
(Path nil <path>)  ; QSelf is first arg, usually nil
```

**Other ExprKind variants** (future phases):

- `Array`, `Call`, `MethodCall`, `Tup`, `Binary`, `Unary`, `Cast`, `If`, `While`, `ForLoop`, `Loop`, `Match`, `Closure`, `Block`, `Await`, `Assign`, `Field`, `Index`, `Range`, `Struct`, `Repeat`, `Paren`, `Try`, `Yield`, `Yeet`, `Become`, `InlineAsm`, `OffsetOf`, `IncludedBytes`, `FormatArgs`, `Err`, `Dummy`

### 4.11 MacCall (Macro Invocation)

```lisp
(MacCall
  :path <path>
  :args <mac-args>
  :prior-type-ascription ((<usize> <bool>)))  ; or nil
```

**Fields:**

- `:path` - Path to the macro (e.g., `std::println`)
- `:args` - Macro arguments
- `:prior-type-ascription` - Parser state (usually nil)

### 4.12 Path

```lisp
(Path
  :span <span>
  :segments <path-segments>
  :tokens nil)
```

**PathSegment:**

```lisp
(PathSegment
  :ident <ident>
  :id <node-id>
  :args <generic-args>)  ; Usually nil for simple paths
```

Example for `println`:

```lisp
(Path
  :span (Span ...)
  :segments ((PathSegment :ident (Ident :name "println" ...) :id 0 :args nil))
  :tokens nil)
```

### 4.13 MacArgs (Macro Arguments)

```lisp
; Delimited arguments
(Delimited
  :dspan <del-span>
  :delim <delimiter>
  :tokens <token-stream>)

; Equal-delimited (macro_rules!)
(Eq
  :eq-span <span>
  :tokens <token-stream>)

; Empty
Empty
```

**DelSpan:**

```lisp
(DelSpan
  :open <span>
  :close <span>)
```

**Delimiter:**

- `Paren` - `()`
- `Brace` - `{}`
- `Bracket` - `[]`
- `Invisible` - No delimiter

### 4.14 TokenStream

For Phase 1, we can represent token streams as strings:

```lisp
(TokenStream :source "\"Hello, world!\"")
```

Or more precisely as a list of tokens (future enhancement):

```lisp
(TokenStream :tokens ((Token :kind (Literal ...) :span ...)))
```

### 4.15 Ident (Identifier)

```lisp
(Ident
  :name "main"
  :span <span>)
```

### 4.16 Visibility

```lisp
; Public
(Public)

; Restricted (pub(crate), pub(super), etc.)
(Restricted
  :path <path>
  :shorthand <vis-restriction-kind>
  :span <span>)

; Inherited (private)
(Inherited)
```

**VisRestrictionKind:**

- `Crate` - `pub(crate)`
- `Super` - `pub(super)`
- `In` - `pub(in path)`

### 4.17 Generics

```lisp
(Generics
  :params <generic-params>
  :where-clause <where-clause>
  :span <span>)
```

For Phase 1 (no generics):

```lisp
(Generics
  :params ()
  :where-clause (WhereClause :has-where-token false :predicates () :span ...)
  :span <span>)
```

### 4.18 Defaultness

```lisp
Default  ; Not a default impl
Final    ; Final impl
```

### 4.19 Safety

```lisp
Safe
Unsafe
Default
```

### 4.20 Constness

```lisp
Const
NotConst
```

### 4.21 Extern

```lisp
; No extern
None

; Explicit extern with ABI
(Explicit <str-lit> <span>)
```

### 4.22 NodeId

Simple integer:

```lisp
:id 42
```

---

## 5. Span and Position Tracking

### 5.1 Span Structure

Rust's `Span` is more complex than Go's `token.Pos`:

```rust
pub struct Span {
    base_or_index: u32,  // Encoded position
    len_or_tag: u16,     // Encoded length
    ctxt_or_tag: u16,    // Syntax context
}
```

But logically it represents:

```rust
struct SpanData {
    lo: BytePos,
    hi: BytePos,
    ctxt: SyntaxContext,
    parent: Option<LocalDefId>,
}
```

**Our S-expression representation:**

```lisp
(Span
  :lo 0
  :hi 50
  :ctxt 0
  :parent nil)
```

For Phase 1, we can simplify:

```lisp
(Span :lo 0 :hi 50)
```

And track full context when needed later.

### 5.2 BytePos

Simple integer representing byte offset:

```lisp
:lo 42
```

### 5.3 SourceFile Tracking

Unlike Go's approach with embedded FileSet, we'll handle source file mapping separately in the tooling layer. The `Span` values are sufficient for round-tripping.

---

## 6. Complete Example: Hello World

### 6.1 Rust Source

```rust
fn main() {
    println!("Hello, world!");
}
```

### 6.2 Canonical S-Expression (Formatted)

```lisp
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
                               :coroutine-kind nil
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
                                                          :id 0
                                                          :args nil))
                                                      :tokens nil)
                                              :args (Delimited
                                                      :dspan (DelSpan
                                                               :open (Span :lo 24 :hi 25)
                                                               :close (Span :lo 42 :hi 43))
                                                      :delim Paren
                                                      :tokens (TokenStream
                                                                :source "\"Hello, world!\""))
                                              :prior-type-ascription nil))
                                    :span (Span :lo 17 :hi 44)
                                    :attrs ()
                                    :tokens nil))
                          :span (Span :lo 17 :hi 44)))
                      :id 3
                      :rules Default
                      :span (Span :lo 13 :hi 48)
                      :tokens nil
                      :could-be-bare-literal false)
      :tokens nil))
  :spans (ModSpans
           :inner-span (Span :lo 0 :hi 50)
           :inject-use-span (Span :lo 0 :hi 0))
  :id 0
  :is-placeholder false)
```

### 6.3 Compact S-Expression (for illustration)

```lisp
(Crate :attrs () :items (
  (Item :vis (Inherited) :ident (Ident :name "main") :kind (Fn
    :sig (FnSig :decl (FnDecl :inputs () :output (Default)))
    :body (Block :stmts (
      (Stmt :kind (Semi (Expr :kind (MacCall
        (MacCall :path (Path :segments ((PathSegment :ident (Ident :name "println"))))
                 :args (Delimited :delim Paren :tokens "\"Hello, world!\""))))))))))))
```

---

## 7. Implementation Notes

### 7.1 Parser Strategy

**Top-down recursive descent:**

1. Expect `(Crate ...)`
2. Parse `:attrs`, `:items`, etc. as keyword arguments
3. Recursively parse nested structures
4. Build Rust AST nodes

**Error handling:**

- Track position in S-expression
- Provide clear error messages
- Validate required fields
- Check enum variant validity

### 7.2 Generator Strategy

**AST walk with visitor pattern:**

1. Match on node type
2. Write opening `(`
3. Write node type name
4. Write each field as `:field value`
5. Write closing `)`
6. Recursively generate children

### 7.3 Round-Trip Guarantee

```
Rust source → rustc parser → Rust AST → oxur-ast generator → S-expr
                                ↑                           ↓
                                └─── oxur-ast parser ←──────────┘
                                         ↓
                                    Rust AST
                                         ↓
                                   rustc printer
                                         ↓
                                    Rust source
```

**Verification:** Generated source should be semantically equivalent to original.

### 7.4 Testing Strategy

**Unit tests:**

- Individual node types
- Edge cases (empty lists, nil values)
- Complex nesting

**Integration tests:**

- Use `tests/ui/` from rust-lang/rust
- Start with `hello_world.rs`
- Progressive coverage

**Round-trip tests:**

- Parse Rust → Generate S-expr → Parse S-expr → Generate Rust
- Compare ASTs (not source text, due to formatting differences)

### 7.5 Dependencies

**For parsing Rust:**

- `syn` crate (initially, for bootstrapping)
- Or `rustc_ast` directly (requires nightly)

**For S-expression parsing:**

- Custom lexer/parser (like Zylisp)
- Simple and direct

**For generation:**

- Walk Rust AST and emit S-expressions
- Pretty-printing optional

---

## 8. Future Phases

### Phase 2: Basic Control Flow

- `if`, `while`, `for`, `loop`
- `match` (pattern matching!)
- `break`, `continue`, `return`

### Phase 3: Types and Declarations

- `struct`, `enum`, `union`
- `impl` blocks
- Type aliases

### Phase 4: Traits

- Trait definitions
- Trait implementations
- Generic bounds
- Where clauses

### Phase 5: Advanced Features

- Lifetimes
- Const generics
- Associated types
- Unsafe blocks
- Inline assembly

### Phase 6: Complete Coverage

- All remaining `ExprKind` variants
- All `ItemKind` variants
- Full token stream representation
- Complete attribute handling

---

## 9. Comparison to Go's Approach

### Similarities

- Keyword arguments for fields
- Faithful representation (no abstraction)
- Position preservation
- Round-trip guarantees

### Differences

**Rust Advantages:**

1. **Cleaner enum pattern** - `Foo` + `FooKind` is more systematic than Go's variety of node types

2. **Better position tracking** - `Span` is simpler than Go's `FileSet` + `token.Pos`

3. **More uniform structure** - Enums make the AST more regular

4. **Pattern matching built-in** - `PatKind` is first-class, not an afterthought

5. **Smaller surface area** - Despite being more expressive, Rust's AST is more compact

**Rust Challenges:**

1. **Lifetimes** - No equivalent in Go; need careful representation

2. **Hygiene/Context** - `SyntaxContext` for macro hygiene is complex

3. **Token streams** - More complex than Go's approach

4. **Const generics** - Type-level computation needs representation

**Overall:** Rust's AST is **easier to map** than Go's because of its systematic design.

---

## 10. Success Criteria

We'll know Phase 1 is successful when:

- [ ] Can parse `fn main() { println!("Hello, world!"); }` to Rust AST
- [ ] Can generate S-expression from that AST
- [ ] Can parse that S-expression back to equivalent AST
- [ ] Can generate Rust source from that AST
- [ ] Generated source compiles and runs correctly
- [ ] All position information is preserved
- [ ] Clear error messages for malformed S-expressions
- [ ] Comprehensive test coverage
- [ ] Clean, well-documented code

---

## 11. Implementation Roadmap

### Week 1: Foundation

- Set up `oxur-ast` crate structure
- Implement S-expression lexer
- Implement S-expression parser
- Create basic test framework

### Week 2: Core Types

- Implement `Crate` parsing/generation
- Implement `Item` + `ItemKind::Fn`
- Implement `Block` and `Stmt`
- Get "empty main" working

### Week 3: Expressions

- Implement `Expr` + `ExprKind`
- Focus on `MacCall` for println!
- Implement `Path` and `Ident`
- Get "Hello World" working

### Week 4: Polish

- Comprehensive testing
- Error handling
- Documentation
- Round-trip verification
- Prepare for Phase 2

---

## Appendix A: Quick Reference

### Node Type Summary

**Top Level:**

- `Crate` - Root of the AST

**Items:**

- `Item` - Top-level declaration
- `ItemKind::Fn` - Function (Phase 1 focus)

**Functions:**

- `FnSig` - Function signature
- `FnHeader` - Function modifiers
- `FnDecl` - Parameters and return type
- `Param` - Function parameter

**Code:**

- `Block` - Code block
- `Stmt` + `StmtKind` - Statement
- `Expr` + `ExprKind` - Expression

**Macros:**

- `MacCall` - Macro invocation
- `MacArgs` - Macro arguments
- `TokenStream` - Token sequence

**Supporting:**

- `Ident` - Identifier
- `Path` - Path to item
- `Span` - Position information
- `Visibility` - pub/private
- `Generics` - Generic parameters
- `Attribute` - Attributes

### Common Patterns

**Empty list:**

```lisp
:items ()
```

**Nil value:**

```lisp
:recv nil
```

**Nested structure:**

```lisp
(Outer
  :field (Inner
           :nested-field value))
```

**Vector of items:**

```lisp
:params ((Param ...) (Param ...) (Param ...))
```

---

## Conclusion

This canonical S-expression format provides a stable, complete representation of Rust's AST. It serves as:

1. **The IR** - Contract between Oxur compiler and Rust code generation
2. **The spec** - Reference for what Oxur can express
3. **The foundation** - Stable base that rarely changes
4. **The assembly** - Low-level representation of Rust semantics

With Phase 1 complete, we'll have proven the concept and can progressively add more of Rust's features while maintaining the same clean, systematic approach.

**Next:** Implement the S-expression parser and begin building out the AST generator.

---

*"In Rust's AST, enums bring clarity. In S-expressions, we preserve it."*
