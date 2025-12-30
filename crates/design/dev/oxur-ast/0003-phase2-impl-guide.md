---
title: "Claude Code Implementation Guide: oxur-ast Phase 2"
author: Duncan McGreggor & Claude
created: 2025-12-25
updated: 2025-12-25
state: Draft
supersedes: None
superseded-by: None
---

# Claude Code Implementation Guide: oxur-ast Phase 2

**Target:** Claude Code (AI coding assistant)
**Goal:** Implement Generator - Rust AST → S-expression conversion
**Estimated Time:** 2-3 work sessions
**Prerequisites:** Phases 0 and 1 complete (all tests passing)

---

## Overview for Claude Code

You've completed Phase 0 (S-expression infrastructure) and Phase 1 (AST types & builder). Now you're implementing **Phase 2: The Generator**.

**What you're building:**

- Generator that converts Rust AST back to S-expressions
- Complete the bidirectional conversion loop
- Round-trip verification (AST → S-expr → AST → S-expr)

**The complete flow after Phase 2:**

```
S-expression → [Parser] → SExp AST → [Builder] → Rust AST
Rust AST → [Generator] → SExp AST → [Printer] → S-expression text
```

**End goal:** Given a Rust AST for Hello World, generate the canonical S-expression representation.

**Caution!!** The code base has changed considerably since this document was created. The development team has attempted to update this doc with the changes, but have undoubtedly missed something. To complete the instructions provided in this document, you will need to adapt these instructions to the current layout of the oxur project. For the lastest project structure (files tracked by git), see the contents of the following:

```
./target/git-tracked-files.txt
```

---

## Session Structure

- **Session 1:** Generator infrastructure and helpers
- **Session 2:** Item, expression, and statement generation
- **Session 3:** Round-trip testing and validation

---

# Session 1: Generator Infrastructure

## Step 1: Verify Phase 1 Complete

Before starting, ensure Phase 0+1 are fully working:

```bash
# All tests should pass
cargo test

# Examples should run
cargo run --example parse_example
cargo run --example build_hello

# No warnings
cargo clippy -- -D warnings
```

If anything fails, fix it before proceeding!

## Step 2: Create Generator Directory Structure

```bash
cd crates/oxur-ast
mkdir -p src/generator
```

Your structure should now have:

```
crates/oxur-ast/
├── src/
│   ├── sexp/       # ✓ Complete (Phase 0)
│   ├── ast/        # ✓ Complete (Phase 1)
│   ├── builder/    # ✓ Complete (Phase 1)
│   └── generator/  # ← New (Phase 2)
│       └── mod.rs  # (to create)
```

## Step 3: Implement Generator Helpers

**Reference:** Design doc 005, Part 1

Create `src/generator/helpers.rs`:

```rust
use crate::sexp::{SExp, Symbol, Keyword, StringLit, Number, List};
use crate::error::Position;

/// Create a symbol S-expression
pub fn sym(name: impl Into<String>) -> SExp {
    SExp::Symbol(Symbol::new(name, Position::new(0, 1, 1)))
}

/// Create a keyword S-expression
pub fn kw(name: impl Into<String>) -> SExp {
    SExp::Keyword(Keyword::new(name, Position::new(0, 1, 1)))
}

/// Create a string S-expression
pub fn string(value: impl Into<String>) -> SExp {
    SExp::String(StringLit::new(value, Position::new(0, 1, 1)))
}

/// Create a number S-expression
pub fn num(value: impl ToString) -> SExp {
    SExp::Number(Number::new(value.to_string(), Position::new(0, 1, 1)))
}

/// Create a list S-expression
pub fn list(elements: Vec<SExp>) -> SExp {
    SExp::List(List::new(elements, Position::new(0, 1, 1)))
}

/// Create an empty list
pub fn empty_list() -> SExp {
    list(vec![])
}

/// Create a keyword-value pair
pub fn kwarg(key: &str, value: SExp) -> Vec<SExp> {
    vec![kw(key), value]
}

/// Create a typed node: (Type :field1 val1 :field2 val2 ...)
pub fn typed_node(type_name: &str, fields: Vec<SExp>) -> SExp {
    let mut elements = vec![sym(type_name)];
    elements.extend(fields);
    list(elements)
}

/// Flatten multiple kwarg pairs into a single vec
pub fn kwargs(pairs: Vec<Vec<SExp>>) -> Vec<SExp> {
    pairs.into_iter().flatten().collect()
}
```

**Test the helpers:**

Create a simple test in `src/generator/mod.rs`:

```rust
mod helpers;

pub use helpers::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sexp::print_sexp;

    #[test]
    fn test_helpers() {
        let node = typed_node("Test", kwargs(vec![
            kwarg("name", string("foo")),
            kwarg("id", num(42)),
        ]));

        let output = print_sexp(&node);
        assert!(output.contains("Test"));
        assert!(output.contains(":name"));
        assert!(output.contains("foo"));
    }
}
```

Run test:

```bash
cargo test -p oxur-ast generator::tests
```

## Step 4: Create Main Generator Structure

**Reference:** Design doc 005, Part 2

Create `src/generator/gen.rs`:

```rust
use crate::error::Result;
use crate::sexp::SExp;
use crate::ast::*;
use crate::generator::helpers::*;

pub struct Generator;

impl Generator {
    pub fn new() -> Self {
        Self
    }

    /// Generate S-expression from Crate
    pub fn generate_crate(&self, crate_node: &Crate) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("attrs", self.generate_attr_vec(&crate_node.attrs)?),
            kwarg("items", self.generate_items(&crate_node.items)?),
            kwarg("spans", self.generate_mod_spans(&crate_node.spans)?),
            kwarg("id", self.generate_node_id(crate_node.id)),
            kwarg("is-placeholder", sym(if crate_node.is_placeholder { "true" } else { "false" })),
        ]);

        Ok(typed_node("Crate", fields))
    }

    fn generate_attr_vec(&self, attrs: &AttrVec) -> Result<SExp> {
        // Phase 2: Just empty list for now
        Ok(empty_list())
    }

    fn generate_items(&self, items: &[Item]) -> Result<SExp> {
        let item_sexps: Result<Vec<SExp>> = items.iter()
            .map(|item| self.generate_item(item))
            .collect();
        Ok(list(item_sexps?))
    }

    fn generate_mod_spans(&self, spans: &ModSpans) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("inner-span", self.generate_span(spans.inner_span)),
            kwarg("inject-use-span", self.generate_span(spans.inject_use_span)),
        ]);

        Ok(typed_node("ModSpans", fields))
    }

    pub fn generate_span(&self, span: Span) -> SExp {
        let fields = kwargs(vec![
            kwarg("lo", num(span.lo)),
            kwarg("hi", num(span.hi)),
        ]);

        // Only include ctxt if non-zero
        let fields = if span.ctxt != 0 {
            let mut f = fields;
            f.extend(kwarg("ctxt", num(span.ctxt)));
            f
        } else {
            fields
        };

        typed_node("Span", fields)
    }

    fn generate_node_id(&self, id: NodeId) -> SExp {
        num(id.0)
    }

    // Placeholder - will implement in next steps
    fn generate_item(&self, _item: &Item) -> Result<SExp> {
        Ok(sym("TODO"))
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}
```

Update `src/generator/mod.rs`:

```rust
mod gen;
mod helpers;

pub use gen::Generator;
pub use helpers::*;
```

## Step 5: Update lib.rs

Update `src/lib.rs`:

```rust
pub mod error;
pub mod sexp;
pub mod ast;
pub mod builder;
pub mod generator;  // ← Add this

pub use error::{ParseError, LexError, Position, Result};
pub use sexp::{SExp, Parser, Printer, print_sexp};
pub use ast::Crate;
pub use builder::AstBuilder;
pub use generator::Generator;  // ← Add this
```

## Step 6: Write Basic Generator Tests

Create `tests/generator_tests.rs`:

```rust
use oxur_ast::*;
use oxur_ast::ast::*;
use oxur_ast::sexp::print_sexp;

#[test]
fn test_generate_span() {
    let span = Span::new(0, 10);
    let gen = Generator::new();
    let sexp = gen.generate_span(span);

    let output = print_sexp(&sexp);
    assert!(output.contains("Span"));
    assert!(output.contains(":lo"));
    assert!(output.contains("0"));
    assert!(output.contains(":hi"));
    assert!(output.contains("10"));
}

#[test]
fn test_generate_empty_crate() {
    let crate_node = Crate::new(
        vec![],
        ModSpans::new(Span::new(0, 10), Span::new(0, 0)),
        NodeId::new(0),
    );

    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node).unwrap();

    let output = print_sexp(&sexp);
    assert!(output.contains("Crate"));
    assert!(output.contains(":items"));
    assert!(output.contains(":spans"));
}
```

Run tests:

```bash
cargo test -p oxur-ast generator_tests
```

---

## Session 1 Checkpoint

At this point you should have:

- ✅ Generator helper functions
- ✅ Main generator structure
- ✅ Basic tests passing
- ✅ Can generate Spans and empty Crates

**Verify:**

```bash
make build
make lint
make test
```

---

# Session 2: Complete Generation Implementation

## Step 7: Implement Item Generation

**Reference:** Design doc 005, Part 3

Create `src/generator/item.rs`:

<details>
<summary>Implementation checklist</summary>

Implement these methods for Generator:

- [ ] `generate_item()` - Main item generation
- [ ] `generate_visibility()` - Public/Inherited/Restricted
- [ ] `generate_ident()` - Identifier with span
- [ ] `generate_item_kind()` - Dispatch to specific item types
- [ ] `generate_fn()` - Function items
- [ ] `generate_defaultness()` - Default/Final
- [ ] `generate_fn_sig()` - Function signature
- [ ] `generate_fn_header()` - Function header (safety, const, etc.)
- [ ] `generate_safety()` - Unsafe/Safe/Default
- [ ] `generate_constness()` - Const/NotConst
- [ ] `generate_extern()` - None/Explicit
- [ ] `generate_coroutine_kind()` - Async/Gen
- [ ] `generate_fn_decl()` - Parameters and return type
- [ ] `generate_params()` - Parameter list
- [ ] `generate_param()` - Single parameter
- [ ] `generate_fn_ret_ty()` - Return type
- [ ] `generate_generics()` - Generic parameters
- [ ] `generate_where_clause()` - Where clauses
- [ ] `generate_ty()` - Type generation
- [ ] `generate_ty_kind()` - Type kind dispatch
- [ ] `generate_pat()` - Pattern generation
- [ ] `generate_pat_kind()` - Pattern kind dispatch
- [ ] `generate_vis_restriction_kind()` - Crate/Super/In

</details>

**Key implementation from design doc:**

```rust
use crate::error::Result;
use crate::sexp::SExp;
use crate::ast::*;
use crate::generator::helpers::*;
use crate::generator::gen::Generator;

impl Generator {
    pub fn generate_item(&self, item: &Item) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("attrs", self.generate_attr_vec(&item.attrs)?),
            kwarg("id", self.generate_node_id(item.id)),
            kwarg("span", self.generate_span(item.span)),
            kwarg("vis", self.generate_visibility(&item.vis)),
            kwarg("ident", self.generate_ident(&item.ident)),
            kwarg("kind", self.generate_item_kind(&item.kind)?),
        ]);

        // Only include tokens if present
        let fields = if item.tokens.is_some() {
            let mut f = fields;
            f.extend(kwarg("tokens", sym("nil")));
            f
        } else {
            fields
        };

        Ok(typed_node("Item", fields))
    }

    fn generate_visibility(&self, vis: &Visibility) -> SExp {
        match vis {
            Visibility::Public => list(vec![sym("Public")]),
            Visibility::Inherited => list(vec![sym("Inherited")]),
            Visibility::Restricted { path, shorthand, span } => {
                let fields = kwargs(vec![
                    kwarg("path", self.generate_path(path)),
                    kwarg("shorthand", self.generate_vis_restriction_kind(*shorthand)),
                    kwarg("span", self.generate_span(*span)),
                ]);
                typed_node("Restricted", fields)
            }
        }
    }

    pub fn generate_ident(&self, ident: &Ident) -> SExp {
        let fields = kwargs(vec![
            kwarg("name", string(&ident.name)),
            kwarg("span", self.generate_span(ident.span)),
        ]);

        typed_node("Ident", fields)
    }

    // ... continue with all other methods from design doc
}
```

**Complete the implementation using design doc 005, Part 3 as reference.**

Update `src/generator/mod.rs`:

```rust
mod gen;
mod helpers;
mod item;

pub use gen::Generator;
pub use helpers::*;
```

## Step 8: Implement Expression Generation

**Reference:** Design doc 005, Part 4

Create `src/generator/expr.rs`:

<details>
<summary>Implementation checklist</summary>

Implement these methods:

- [ ] `generate_block()` - Code blocks
- [ ] `generate_block_check_mode()` - Default/Unsafe
- [ ] `generate_stmts()` - Statement list
- [ ] `generate_expr()` - Expression wrapper
- [ ] `generate_expr_kind()` - Dispatch to expr types
- [ ] `generate_mac_call()` - Macro calls
- [ ] `generate_path()` - Paths
- [ ] `generate_path_segments()` - Segment list
- [ ] `generate_path_segment()` - Single segment
- [ ] `generate_mac_args()` - Macro arguments
- [ ] `generate_del_span()` - Delimiter span
- [ ] `generate_delimiter()` - Paren/Brace/Bracket/Invisible
- [ ] `generate_token_stream()` - Token streams
- [ ] `generate_lit()` - Literals
- [ ] `generate_lit_kind()` - Lit kind dispatch

</details>

**Key implementation pattern:**

```rust
use crate::error::Result;
use crate::sexp::SExp;
use crate::ast::*;
use crate::generator::helpers::*;
use crate::generator::gen::Generator;

impl Generator {
    pub fn generate_block(&self, block: &Block) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("stmts", self.generate_stmts(&block.stmts)?),
            kwarg("id", self.generate_node_id(block.id)),
            kwarg("rules", self.generate_block_check_mode(block.rules)),
            kwarg("span", self.generate_span(block.span)),
        ]);

        // Only include optional fields if present
        let mut fields = fields;
        if block.tokens.is_some() {
            fields.extend(kwarg("tokens", sym("nil")));
        }
        fields.extend(kwarg("could-be-bare-literal",
            sym(if block.could_be_bare_literal { "true" } else { "false" })));

        Ok(typed_node("Block", fields))
    }

    // ... continue with all other methods
}
```

**Complete using design doc 005, Part 4.**

Update `src/generator/mod.rs`:

```rust
mod gen;
mod helpers;
mod item;
mod expr;

pub use gen::Generator;
pub use helpers::*;
```

## Step 9: Implement Statement Generation

**Reference:** Design doc 005, Part 5

Create `src/generator/stmt.rs`:

<details>
<summary>Implementation checklist</summary>

Implement these methods:

- [ ] `generate_stmt()` - Statement wrapper
- [ ] `generate_stmt_kind()` - Dispatch to stmt types
- [ ] `generate_local()` - Let bindings
- [ ] `generate_local_init()` - Initializers
- [ ] `generate_mac_call_stmt()` - Macro statements
- [ ] `generate_mac_stmt_style()` - Semicolon/Braces/NoBraces

</details>

**Implementation:**

```rust
use crate::error::Result;
use crate::sexp::SExp;
use crate::ast::*;
use crate::generator::helpers::*;
use crate::generator::gen::Generator;

impl Generator {
    pub fn generate_stmt(&self, stmt: &Stmt) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("id", self.generate_node_id(stmt.id)),
            kwarg("kind", self.generate_stmt_kind(&stmt.kind)?),
            kwarg("span", self.generate_span(stmt.span)),
        ]);

        Ok(typed_node("Stmt", fields))
    }

    fn generate_stmt_kind(&self, kind: &StmtKind) -> Result<SExp> {
        match kind {
            StmtKind::Expr(expr) => {
                Ok(list(vec![
                    sym("Expr"),
                    self.generate_expr(expr)?,
                ]))
            }
            StmtKind::Semi(expr) => {
                Ok(list(vec![
                    sym("Semi"),
                    self.generate_expr(expr)?,
                ]))
            }
            StmtKind::Empty => {
                Ok(sym("Empty"))
            }
            // ... continue with other variants
            _ => Ok(sym("TODO"))
        }
    }

    // ... continue with all other methods
}
```

**Complete using design doc 0005, Part 5.**

Update `src/generator/mod.rs`:

```rust
mod gen;
mod helpers;
mod item;
mod expr;
mod stmt;

pub use gen::Generator;
pub use helpers::*;
```

## Step 10: Comprehensive Generator Tests

Add more tests to `tests/generator_tests.rs`:

```rust
use oxur_ast::*;
use oxur_ast::ast::*;
use oxur_ast::sexp::print_sexp;

#[test]
fn test_generate_ident() {
    let ident = Ident::new("main", Span::new(3, 7));
    let gen = Generator::new();
    let sexp = gen.generate_ident(&ident);

    let output = print_sexp(&sexp);
    assert!(output.contains("Ident"));
    assert!(output.contains(":name"));
    assert!(output.contains("main"));
}

#[test]
fn test_generate_path() {
    let ident = Ident::new("println", Span::new(17, 24));
    let segment = PathSegment::from_ident(ident);
    let path = Path::new(Span::new(17, 24), vec![segment]);

    let gen = Generator::new();
    let sexp = gen.generate_path(&path);

    let output = print_sexp(&sexp);
    assert!(output.contains("Path"));
    assert!(output.contains("PathSegment"));
    assert!(output.contains("println"));
}

#[test]
fn test_generate_visibility() {
    let gen = Generator::new();

    let vis = Visibility::Public;
    let sexp = gen.generate_visibility(&vis);
    let output = print_sexp(&sexp);
    assert!(output.contains("Public"));

    let vis = Visibility::Inherited;
    let sexp = gen.generate_visibility(&vis);
    let output = print_sexp(&sexp);
    assert!(output.contains("Inherited"));
}

#[test]
fn test_generate_mac_call() {
    let ident = Ident::new("println", Span::new(17, 24));
    let segment = PathSegment::from_ident(ident);
    let path = Path::new(Span::new(17, 24), vec![segment]);

    let args = MacArgs::Delimited {
        dspan: DelSpan::new(Span::new(24, 25), Span::new(42, 43)),
        delim: Delimiter::Paren,
        tokens: TokenStream::from_str("\"Hello, world!\""),
    };

    let mac_call = MacCall::new(path, args);

    let gen = Generator::new();
    let sexp = gen.generate_mac_call(&mac_call).unwrap();

    let output = print_sexp(&sexp);
    assert!(output.contains("MacCall"));
    assert!(output.contains("println"));
    assert!(output.contains("Delimited"));
    assert!(output.contains("Hello, world!"));
}
```

Run tests:

```bash
cargo test -p oxur-ast generator_tests
```

---

## Session 2 Checkpoint

At this point you should have:

- ✅ Complete generator implementation
- ✅ All generator methods for items, expressions, statements
- ✅ Generator tests passing

**Verify:**

```bash
make build
make lint
make test
```

---

# Session 3: Round-Trip Testing

## Step 11: Implement Round-Trip Tests

**Reference:** Design doc 005, Part 8

Create `tests/round_trip_tests.rs`:

```rust
use oxur_ast::*;
use oxur_ast::sexp::{Parser, print_sexp};

#[test]
fn test_round_trip_span() {
    let original = "(Span :lo 0 :hi 10)";

    // Parse to SExp
    let sexp1 = Parser::parse_str(original).unwrap();

    // Build to AST
    let mut builder = AstBuilder::new();
    let span = builder.build_span(&sexp1).unwrap();

    // Generate back to SExp
    let gen = Generator::new();
    let sexp2 = gen.generate_span(span);

    // Parse again
    let printed = print_sexp(&sexp2);
    let sexp3 = Parser::parse_str(&printed).unwrap();

    // Should be equivalent
    assert_eq!(sexp1, sexp3);
}

#[test]
fn test_round_trip_ident() {
    let original = r#"(Ident :name "main" :span (Span :lo 3 :hi 7))"#;

    let sexp1 = Parser::parse_str(original).unwrap();

    let mut builder = AstBuilder::new();
    let ident = builder.build_ident(&sexp1).unwrap();

    let gen = Generator::new();
    let sexp2 = gen.generate_ident(&ident);

    let printed = print_sexp(&sexp2);
    let sexp3 = Parser::parse_str(&printed).unwrap();

    assert_eq!(sexp1, sexp3);
}
```

!!! IMPORTANT CHANGE !!!

This project has moved to a file-based approach for storing s-expressions. It is a new policy that s-expressions used in tests and examples should not be buried in code, but easily and quickly accessible to reivewers, tooling, etc.

The current layout of s-expressions can bee see in the following subdirs:

```shell
./crates/oxur-ast/test-data/
```

You will need to determine the correct place to store the following s-expressions, and then use the developed Rust code to extract the files required for testing.

```rust
#[test]
fn test_round_trip_path() {
    let original = r#"
(Path
  :span (Span :lo 17 :hi 24)
  :segments (
    (PathSegment
      :ident (Ident :name "println" :span (Span :lo 17 :hi 24))
      :id 0
      :args nil)))
    "#;

    let sexp1 = Parser::parse_str(original).unwrap();

    let mut builder = AstBuilder::new();
    let path = builder.build_path(&sexp1).unwrap();

    let gen = Generator::new();
    let sexp2 = gen.generate_path(&path);

    let printed = print_sexp(&sexp2);
    let sexp3 = Parser::parse_str(&printed).unwrap();

    assert_eq!(sexp1, sexp3);
}

#[test]
fn test_round_trip_simple_crate() {
    let original = r#"
(Crate
  :attrs ()
  :items ()
  :spans (ModSpans
           :inner-span (Span :lo 0 :hi 10)
           :inject-use-span (Span :lo 0 :hi 0))
  :id 0
  :is-placeholder false)
    "#;

    let sexp1 = Parser::parse_str(original).unwrap();

    let mut builder = AstBuilder::new();
    let crate_node = builder.build_crate(&sexp1).unwrap();

    let gen = Generator::new();
    let sexp2 = gen.generate_crate(&crate_node).unwrap();

    let printed = print_sexp(&sexp2);
    let sexp3 = Parser::parse_str(&printed).unwrap();

    assert_eq!(sexp1, sexp3);
}
```

## Step 12: The Big Test - Hello World Round-Trip

Add this critical test to `tests/round_trip_tests.rs`:

```rust
#[test]
fn test_round_trip_hello_world() {
    // Complete Hello World S-expression (from design doc 002)
    let original = r#"
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
                                                          :args nil)))
                                              :args (Delimited
                                                      :dspan (DelSpan
                                                               :open (Span :lo 24 :hi 25)
                                                               :close (Span :lo 42 :hi 43))
                                                      :delim Paren
                                                      :tokens (TokenStream
                                                                :source "\"Hello, world!\""))
                                              :prior-type-ascription nil))
                                    :span (Span :lo 17 :hi 44)
                                    :attrs ()))
                          :span (Span :lo 17 :hi 44)))
                      :id 3
                      :rules Default
                      :span (Span :lo 13 :hi 48)
                      :could-be-bare-literal false))))
  :spans (ModSpans
           :inner-span (Span :lo 0 :hi 50)
           :inject-use-span (Span :lo 0 :hi 0))
  :id 0
  :is-placeholder false)
    "#;

    println!("Testing Hello World round-trip...\n");

    // Parse original
    println!("1. Parsing original S-expression...");
    let sexp1 = Parser::parse_str(original).unwrap();

    // Build to AST
    println!("2. Building Rust AST...");
    let mut builder = AstBuilder::new();
    let crate_node = builder.build_crate(&sexp1).unwrap();

    // Verify basic structure
    assert_eq!(crate_node.items.len(), 1);
    assert_eq!(crate_node.items[0].ident.name, "main");
    println!("   ✓ AST structure verified");

    // Generate back to SExp
    println!("3. Generating S-expression from AST...");
    let gen = Generator::new();
    let sexp2 = gen.generate_crate(&crate_node).unwrap();

    // Parse again
    println!("4. Parsing generated S-expression...");
    let printed = print_sexp(&sexp2);
    let sexp3 = Parser::parse_str(&printed).unwrap();

    // Should be equivalent
    println!("5. Verifying equivalence...");
    assert_eq!(sexp1, sexp3);

    println!("\n✓ Round-trip successful!");
    println!("Original → AST → Generated → AST");
    println!("All structures preserved!");
}
```

Run this critical test:

```bash
cargo test -p oxur-ast round_trip_hello_world -- --nocapture
```

You should see:

```
Testing Hello World round-trip...

1. Parsing original S-expression...
2. Building Rust AST...
   ✓ AST structure verified
3. Generating S-expression from AST...
4. Parsing generated S-expression...
5. Verifying equivalence...

✓ Round-trip successful!
Original → AST → Generated → AST
All structures preserved!
```

## Step 13: Create Example

Create `examples/generate_hello.rs`:

```rust
use oxur_ast::*;
use oxur_ast::ast::*;
use oxur_ast::sexp::print_sexp;

fn main() {
    println!("Building Hello World AST manually...\n");

    // Build the AST by hand
    let println_ident = Ident::new("println", Span::new(17, 24));
    let println_segment = PathSegment::from_ident(println_ident);
    let println_path = Path::new(Span::new(17, 24), vec![println_segment]);

    let mac_args = MacArgs::Delimited {
        dspan: DelSpan::new(Span::new(24, 25), Span::new(42, 43)),
        delim: Delimiter::Paren,
        tokens: TokenStream::from_str("\"Hello, world!\""),
    };

    let mac_call = MacCall::new(println_path, mac_args);

    let expr = Expr {
        id: NodeId::new(2),
        kind: ExprKind::MacCall(Box::new(mac_call)),
        span: Span::new(17, 44),
        attrs: vec![],
        tokens: None,
    };

    let stmt = Stmt {
        id: NodeId::new(1),
        kind: StmtKind::Semi(Box::new(expr)),
        span: Span::new(17, 44),
    };

    let block = Block::new(
        vec![stmt],
        NodeId::new(3),
        Span::new(13, 48),
    );

    let fn_sig = FnSig {
        header: FnHeader::default(),
        decl: FnDecl {
            inputs: vec![],
            output: FnRetTy::Default(Span::new(10, 10)),
        },
        span: Span::new(0, 10),
    };

    let fn_item = Fn {
        defaultness: Defaultness::Final,
        sig: fn_sig,
        generics: Generics::empty(Span::new(7, 10)),
        body: Some(block),
    };

    let item = Item {
        attrs: vec![],
        id: NodeId::new(0),
        span: Span::new(0, 50),
        vis: Visibility::Inherited,
        ident: Ident::new("main", Span::new(3, 7)),
        kind: ItemKind::Fn(Box::new(fn_item)),
        tokens: None,
    };

    let crate_node = Crate::new(
        vec![item],
        ModSpans::new(Span::new(0, 50), Span::new(0, 0)),
        NodeId::new(0),
    );

    println!("✓ AST built successfully!\n");

    // Generate S-expression
    println!("Generating S-expression...\n");
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node).expect("Failed to generate S-expression");

    println!("✓ S-expression generated!\n");

    // Print it
    let output = print_sexp(&sexp);
    println!("Generated S-expression:\n");
    println!("{}", output);
}
```

Run it:

```bash
cargo run --example generate_hello
```

## Step 14: Add Compact Printer (Optional Enhancement)

**Reference:** Design doc 005, Part 10

Add to `src/sexp/printer.rs`:

```rust
impl Printer {
    /// Print in compact mode (minimal whitespace)
    pub fn print_compact(&mut self, sexp: &SExp) -> String {
        let mut output = String::new();
        self.print_sexp_compact(sexp, &mut output);
        output
    }

    fn print_sexp_compact(&mut self, sexp: &SExp, output: &mut String) {
        match sexp {
            SExp::List(l) => {
                write!(output, "(").unwrap();
                for (i, elem) in l.elements.iter().enumerate() {
                    if i > 0 {
                        write!(output, " ").unwrap();
                    }
                    self.print_sexp_compact(elem, output);
                }
                write!(output, ")").unwrap();
            }
            _ => self.print_sexp(sexp, output),
        }
    }
}

/// Convenience function for compact printing
pub fn print_sexp_compact(sexp: &SExp) -> String {
    Printer::new().print_compact(sexp)
}
```

Update exports in `src/sexp/mod.rs`:

```rust
pub use printer::{Printer, print_sexp, print_sexp_compact};
```

---

## Session 3 Checkpoint - Phase 2 Complete

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

# Run examples
cargo run --example parse_example
cargo run --example build_hello
cargo run --example generate_hello

# The critical test
cargo test round_trip_hello_world -- --nocapture
```

**Success criteria checklist:**

- [ ] All generator methods implemented
- [ ] All generator tests passing
- [ ] Round-trip tests passing (including Hello World)
- [ ] Examples run successfully
- [ ] No clippy warnings
- [ ] Code formatted
- [ ] Documentation comments added

---

# What You've Accomplished

🎉 **Phase 2 Complete!** You've implemented:

1. **Generator Infrastructure**
   - Helper functions for S-expression building
   - Main generator structure
   - Clean, composable API

2. **Complete Generation**
   - Item generation (functions, visibility, etc.)
   - Expression generation (blocks, macros, literals)
   - Statement generation (all variants)
   - Type and pattern generation

3. **Bidirectional Conversion**
   - S-expression → AST (Phase 1)
   - AST → S-expression (Phase 2)
   - **Complete round-trip verified!**

**The complete flow now works:**

```
S-expression text
    ↓ [Parser]
SExp AST
    ↓ [Builder]
Rust AST
    ↓ [Generator]
SExp AST
    ↓ [Printer]
S-expression text
```

**You can now:**

- Parse S-expressions
- Build Rust AST from S-expressions
- Generate S-expressions from Rust AST
- Round-trip: preserve all structure and information
- Pretty-print S-expressions

---

# Next Steps

**Phase 3** (Integration & CLI) will add:

- Integration with `syn` to parse real Rust files
- CLI tool for practical conversions
- Integration tests with real code
- Benchmarks
- More comprehensive examples

**You're ready to:**

- Parse actual Rust code (via syn)
- Convert it to S-expressions
- Edit S-expressions
- Convert back to Rust
- Verify correctness

---

# Troubleshooting

## Common Issues

**Round-trip failures:**

1. Check that generator mirrors builder exactly
2. Verify all fields are generated
3. Ensure optional fields handled consistently
4. Check position information matches

**Missing methods:**

1. Verify all imports in generator modules
2. Check that all impl blocks are for Generator
3. Ensure module exports are correct

**Test failures:**

1. Print the generated S-expression to debug
2. Compare structure element by element
3. Use `-- --nocapture` to see print output

## Debugging Tips

**To debug round-trip issues:**

```rust
// In your test
let sexp1 = Parser::parse_str(original).unwrap();
println!("Original: {:#?}", sexp1);

let crate_node = builder.build_crate(&sexp1).unwrap();
println!("AST: {:#?}", crate_node);

let sexp2 = gen.generate_crate(&crate_node).unwrap();
println!("Generated: {:#?}", sexp2);

let printed = print_sexp(&sexp2);
println!("Printed:\n{}", printed);
```

---

*"The circle completes. What was built can be generated. What was generated can be built. The bridge stands strong in both directions."*
