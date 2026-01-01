# oxur-ast Architecture

This document provides a comprehensive overview of the oxur-ast crate architecture, implementation details, and design decisions.

## Table of Contents

1. [Overview](#overview)
2. [Architecture Diagram](#architecture-diagram)
3. [Module Structure](#module-structure)
4. [AST Representation](#ast-representation)
5. [S-expression Format](#s-expression-format)
6. [Generator (AST → S-expr)](#generator-ast--s-expr)
7. [Builder (S-expr → AST)](#builder-s-expr--ast)
8. [Code Generation (AST → Rust)](#code-generation-ast--rust)
9. [Integration Layer](#integration-layer)
10. [Testing Strategy](#testing-strategy)
11. [Performance Considerations](#performance-considerations)
12. [Phase Implementation History](#phase-implementation-history)

---

## Overview

oxur-ast is a bidirectional conversion library that bridges **Rust Abstract Syntax Trees (AST)** and **S-expression representations**. It enables:

- **Parsing Rust source code** into AST structures via the `syn` crate
- **Converting AST to S-expressions** for human-readable/editable representation
- **Building AST from S-expressions** for programmatic manipulation
- **Generating Rust code** from AST structures
- **Round-trip preservation** ensuring semantic equivalence

### Key Design Goals

1. **Fidelity**: Preserve all semantic information during conversions
2. **Readability**: S-expressions should be human-readable and editable
3. **Completeness**: Support comprehensive Rust syntax coverage
4. **Correctness**: Maintain type safety and prevent invalid AST construction
5. **Performance**: Efficient parsing, generation, and conversion

---

## Architecture Diagram

```text
┌─────────────────────────────────────────────────────────────────┐
│                         Rust Source Code                        │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             │ syn::parse_file()
                             ▼
                    ┌─────────────────┐
                    │   syn::File     │
                    │  (syn crate)    │
                    └────────┬────────┘
                             │
                             │ from_syn (integration)
                             ▼
┌────────────────────────────────────────────────────────────────┐
│                      oxur_ast::Crate                           │
│                   (Internal AST)                               │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ Items: Fn, Struct, Enum, Trait, Impl, Use, ...          │  │
│  │ Exprs: Binary, Call, If, Match, Closure, ...            │  │
│  │ Types: Path, Ref, Tuple, Array, BareFn, ...             │  │
│  │ Patterns: Ident, Struct, Tuple, Or, Ref, ...            │  │
│  │ Statements: Expr, Semi, Let, Item, MacCall              │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────┬──────────────────────────┬───────────────────────┘
              │                          │
              │ Generator                │ Builder
              ▼                          │
┌─────────────────────────┐              │
│   S-expression          │              │
│   (sexp::SExp)          │◄─────────────┘
│                         │      Parser
│ (Crate                  │
│   :items (...)          │
│   :spans (...))         │
└─────────┬───────────────┘
          │
          │ Printer
          ▼
┌─────────────────────────┐
│  Formatted Text         │
│  (String)               │
└─────────┬───────────────┘
          │
          │ print_sexp()
          ▼
     File or stdout

┌─────────────────────────┐
│   oxur_ast::Crate       │
│   (Internal AST)        │
└─────────┬───────────────┘
          │
          │ codegen::generate_rust()
          ▼
┌─────────────────────────┐
│   Rust Source Code      │
│   (String)              │
└─────────────────────────┘
```

---

## Module Structure

### Core Modules

```
oxur-ast/
├── src/
│   ├── lib.rs                  # Public API, re-exports
│   │
│   ├── ast/                    # AST node definitions
│   │   ├── mod.rs              # Root Crate struct
│   │   ├── item.rs             # Item types (Fn, Struct, Enum, ...)
│   │   ├── expr.rs             # Expressions and patterns
│   │   ├── stmt.rs             # Statements
│   │   ├── types.rs            # Type system (Ty, Pat)
│   │   ├── path.rs             # Path structures
│   │   └── span.rs             # Source location tracking
│   │
│   ├── sexp/                   # S-expression parsing/printing
│   │   ├── mod.rs              # Public API
│   │   ├── parser.rs           # Text → SExp parsing
│   │   ├── printer.rs          # SExp → Text formatting
│   │   └── types.rs            # SExp enum definition
│   │
│   ├── generator/              # AST → S-expression
│   │   ├── mod.rs              # Generator struct
│   │   ├── gen.rs              # Core generation logic
│   │   ├── item.rs             # Item generation
│   │   ├── expr.rs             # Expression/pattern generation
│   │   ├── stmt.rs             # Statement generation
│   │   └── helpers.rs          # Common utilities
│   │
│   ├── builder/                # S-expression → AST
│   │   ├── mod.rs              # AstBuilder struct
│   │   ├── item.rs             # Item building
│   │   ├── expr.rs             # Expression/pattern building
│   │   ├── stmt.rs             # Statement building
│   │   └── helpers.rs          # Extraction utilities
│   │
│   ├── codegen/                # AST → Rust code
│   │   ├── mod.rs              # Public API
│   │   ├── rust.rs             # Main code generation
│   │   ├── item.rs             # Item code generation
│   │   ├── expr.rs             # Expression code generation
│   │   └── types.rs            # Type code generation
│   │
│   ├── integration/            # syn ↔ oxur_ast conversion
│   │   ├── mod.rs              # Public API
│   │   └── from_syn.rs         # syn → oxur_ast conversion
│   │
│   └── error.rs                # Error types
│
├── tests/                      # Integration tests
│   ├── integration_tests.rs
│   ├── integration_partial_tests.rs
│   ├── phase5_round_trip_tests.rs
│   └── ...
│
├── test-data/                  # Test fixtures
│   ├── simple/
│   ├── intermediate/
│   └── complex/
│
└── benches/                    # Performance benchmarks
    └── conversion_bench.rs
```

---

## AST Representation

### Core AST Structure

The root of every AST is a `Crate`:

```rust
pub struct Crate {
    pub attrs: AttrVec,         // Attributes (#[...], #![...])
    pub items: Vec<Item>,       // Top-level items
    pub spans: ModSpans,        // Source location info
    pub id: NodeId,             // Unique identifier
    pub is_placeholder: bool,   // For error recovery
}
```

### Item Types (ItemKind)

**10 variants** - Top-level declarations:

1. **`Fn`** - Function definitions
   ```rust
   fn example(x: i32) -> i32 { x + 1 }
   ```

2. **`Struct`** - Struct definitions
   ```rust
   struct Point { x: i32, y: i32 }
   ```

3. **`Enum`** - Enum definitions
   ```rust
   enum Option<T> { Some(T), None }
   ```

4. **`Trait`** - Trait definitions
   ```rust
   trait Display { fn fmt(&self) -> String; }
   ```

5. **`Impl`** - Implementation blocks
   ```rust
   impl Point { fn new(x: i32, y: i32) -> Self { ... } }
   ```

6. **`Use`** - Use declarations
   ```rust
   use std::collections::HashMap;
   ```

7. **`Static`** - Static items
   ```rust
   static COUNTER: i32 = 0;
   ```

8. **`Const`** - Const items
   ```rust
   const MAX: usize = 100;
   ```

9. **`TyAlias`** - Type aliases
   ```rust
   type Result<T> = std::result::Result<T, Error>;
   ```

10. **`Mod`** - Module declarations
    ```rust
    mod utils { ... }
    ```

### Expression Types (ExprKind)

**26 variants** - All expression forms:

#### Core Expressions (Phase 1-4)
1. **`Lit`** - Literals (`42`, `"hello"`, `true`)
2. **`Path`** - Path expressions (`std::io::Result`)
3. **`Binary`** - Binary operations (`a + b`, `x && y`)
4. **`Unary`** - Unary operations (`!x`, `-n`, `*ptr`)
5. **`Call`** - Function calls (`foo(a, b)`)
6. **`MethodCall`** - Method calls (`obj.method()`)
7. **`Field`** - Field access (`obj.field`)
8. **`Index`** - Index access (`arr[i]`)
9. **`Assign`** - Assignment (`x = 5`)
10. **`If`** - If expressions
11. **`Match`** - Match expressions
12. **`While`** - While loops
13. **`ForLoop`** - For loops
14. **`Loop`** - Infinite loops
15. **`Array`** - Array literals (`[1, 2, 3]`)
16. **`Tuple`** - Tuple literals (`(a, b)`)
17. **`Struct`** - Struct literals (`Point { x: 1, y: 2 }`)
18. **`Closure`** - Closures (`|x| x + 1`)
19. **`Range`** - Range expressions (`0..10`, `..=5`)
20. **`MacCall`** - Macro invocations

#### Phase 5 Additions (Priority 3) ✨
21. **`Paren`** - Parenthesized expressions (`(expr)`)
22. **`Try`** - Try operator (`expr?`)
23. **`Cast`** - Type casts (`x as u64`)
24. **`Break`** - Break with optional value (`break 42`)
25. **`Continue`** - Continue with optional label
26. **`Return`** - Return with optional value (`return x`)

### Pattern Types (PatKind)

**18 variants** - All pattern matching forms:

#### Core Patterns (Phase 1-4)
1. **`Ident`** - Identifier patterns with binding mode
2. **`Wild`** - Wildcard (`_`)
3. **`Lit`** - Literal patterns (`42`, `"hello"`)
4. **`Struct`** - Struct patterns (`Point { x, y }`)
5. **`TupleStruct`** - Tuple struct patterns (`Some(x)`)
6. **`Tuple`** - Tuple patterns (`(a, b, c)`)
7. **`Slice`** - Slice patterns (`[a, b, .., c]`)
8. **`Or`** - Or patterns (`Some(x) | None`)

#### Phase 5 Additions (Priority 1) ✨
9. **`Box`** - Box patterns (`box pat`)
10. **`Path`** - Path patterns (`None`, `Some`)
11. **`Range`** - Range patterns (`1..=5`, `..=10`)
12. **`Rest`** - Rest patterns (`..` in tuples/slices)
13. **`Paren`** - Parenthesized patterns (`(pat)`)
14. **`Type`** - Type ascription (`x: i32`)
15. **`Const`** - Const blocks (`const { expr }`)
16. **`MacCall`** - Macro invocations
17. **`Err`** - Error recovery
18. **`Ref`** - Reference patterns (`&x`, `&mut x`)

### Type System (TyKind)

**13 variants** - All type expressions:

#### Core Types (Phase 1-4)
1. **`Path`** - Path types (`i32`, `String`, `Vec<T>`)
2. **`Ref`** - Reference types (`&T`, `&mut T`, `&'a T`)
3. **`Ptr`** - Raw pointers (`*const T`, `*mut T`)
4. **`Array`** - Array types (`[T; N]`)
5. **`Slice`** - Slice types (`[T]`)
6. **`Tuple`** - Tuple types (`(A, B, C)`)
7. **`Never`** - Never type (`!`)
8. **`Infer`** - Infer type (`_`)

#### Phase 5 Additions (Priority 2) ✨
9. **`BareFn`** - Function pointer types (`fn(i32) -> String`)
10. **`ImplTrait`** - Impl trait (`impl Iterator<Item = u8>`)
11. **`TraitObject`** - Trait objects (`dyn Display + Send`)
12. **`MacCall`** - Macro invocations in type position
13. **`Paren`** - Parenthesized types (`(T)`)

### Statement Types (StmtKind)

**6 variants** - All statement forms:

1. **`Expr`** - Expression statements
2. **`Semi`** - Expression with semicolon
3. **`Let`** - Let bindings (`let x = 5;`)
4. **`Item`** - Item declarations within blocks
5. **`MacCall`** - Macro call statements
6. **`Empty`** - Empty statements

---

## S-expression Format

### S-expression Grammar

```
SExp := Symbol | Keyword | String | Number | Nil | List

Symbol  := [a-zA-Z_][a-zA-Z0-9_-]*
Keyword := :[a-zA-Z_][a-zA-Z0-9_-]*
String  := "([^"\\]|\\.)*"
Number  := -?[0-9]+
Nil     := nil
List    := ( SExp* )
```

### Example: Function Item

**Rust source:**
```rust
fn add(x: i32, y: i32) -> i32 {
    x + y
}
```

**S-expression representation:**
```scheme
(Crate
  :attrs ()
  :items ((Item
            :attrs ()
            :id 0
            :span (Span :lo 0 :hi 0)
            :vis (Inherited)
            :ident (Ident :name "add" :span (Span :lo 0 :hi 0))
            :kind (Fn
                    (Fn
                      :defaultness Final
                      :sig (FnSig
                             :header (FnHeader
                                       :safety Default
                                       :constness NotConst
                                       :ext None
                                       :coroutine-kind nil)
                             :decl (FnDecl
                                     :inputs ((Param
                                                :attrs ()
                                                :ty (Ty :kind (Path nil (Path ...)) ...)
                                                :pat (Pat :kind (Ident ...) ...)
                                                :id 0
                                                :span (Span :lo 0 :hi 0))
                                              (Param ...))
                                     :output (Ty :kind (Path nil (Path ...)) ...))
                             :span (Span :lo 0 :hi 0))
                      :generics (Generics :params () :where-clause ...)
                      :body (Block
                              :stmts ((Stmt
                                        :id 0
                                        :kind (Expr
                                                (Expr
                                                  :id 1
                                                  :kind (Binary
                                                          Add
                                                          (Expr :kind (Path ...) ...)
                                                          (Expr :kind (Path ...) ...))
                                                  :span (Span :lo 0 :hi 0)
                                                  :attrs ()))
                                        :span (Span :lo 0 :hi 0)))
                              :id 2
                              :rules Default
                              :span (Span :lo 0 :hi 0)
                              :could-be-bare-literal false)))))
  :spans (ModSpans :inner-span (Span :lo 0 :hi 0) :inject-use-span (Span :lo 0 :hi 0))
  :id 3
  :is-placeholder false)
```

### Design Rationale

**Keywords with colons** (`:name`, `:type`, `:items`) provide clear structure:
- Easily distinguishable from symbols
- Natural mapping to struct field names
- Improved readability

**Nested lists** preserve AST hierarchy:
- Each AST node becomes a list
- First element is the node type (Symbol)
- Remaining elements are keyword-value pairs

**Nil for Option::None**:
- Explicit representation of missing values
- Distinguishes from empty lists

---

## Generator (AST → S-expr)

### Architecture

The `Generator` module converts oxur_ast structures into S-expressions:

```rust
pub struct Generator {
    // Internal state for tracking node IDs, etc.
}

impl Generator {
    pub fn generate_crate(&self, crate_node: &Crate) -> Result<SExp>;
    pub fn generate_item(&self, item: &Item) -> Result<SExp>;
    pub fn generate_expr(&self, expr: &Expr) -> Result<SExp>;
    pub fn generate_pat(&self, pat: &Pat) -> Result<SExp>;
    pub fn generate_ty(&self, ty: &Ty) -> Result<SExp>;
    // ... more generation methods
}
```

### Generation Strategy

1. **Recursive descent** through AST nodes
2. **Field extraction** using struct introspection
3. **Type conversion** to S-expression primitives
4. **List construction** for nested structures

### Example: Generating an Expression

```rust
fn generate_expr(&self, expr: &Expr) -> Result<SExp> {
    let mut fields = vec![
        keyword("id"), self.generate_node_id(expr.id)?,
        keyword("kind"), self.generate_expr_kind(&expr.kind)?,
        keyword("span"), self.generate_span(&expr.span)?,
        keyword("attrs"), self.generate_attrs(&expr.attrs)?,
    ];

    Ok(list(symbol("Expr"), fields))
}

fn generate_expr_kind(&self, kind: &ExprKind) -> Result<SExp> {
    match kind {
        ExprKind::Binary(op, lhs, rhs) => {
            Ok(list(vec![
                symbol("Binary"),
                self.generate_bin_op(op)?,
                self.generate_expr(lhs)?,
                self.generate_expr(rhs)?,
            ]))
        }
        ExprKind::Paren(expr) => {
            Ok(list(vec![
                symbol("Paren"),
                self.generate_expr(expr)?,
            ]))
        }
        // ... handle all ExprKind variants
    }
}
```

---

## Builder (S-expr → AST)

### Architecture

The `AstBuilder` module constructs oxur_ast structures from S-expressions:

```rust
pub struct AstBuilder {
    // Internal state for tracking context, etc.
}

impl AstBuilder {
    pub fn build_crate(&mut self, sexp: &SExp) -> Result<Crate>;
    pub fn build_item(&mut self, sexp: &SExp) -> Result<Item>;
    pub fn build_expr(&mut self, sexp: &SExp) -> Result<Expr>;
    pub fn build_pat(&mut self, sexp: &SExp) -> Result<Pat>;
    pub fn build_ty(&mut self, sexp: &SExp) -> Result<Ty>;
    // ... more building methods
}
```

### Building Strategy

1. **Pattern matching** on S-expression structure
2. **Field extraction** using keyword lookup
3. **Recursive building** for nested nodes
4. **Validation** of required fields
5. **Error handling** with position information

### Example: Building an Expression

```rust
fn build_expr(&mut self, sexp: &SExp) -> Result<Expr> {
    let fields = extract_keyword_fields(sexp)?;

    let id = self.build_node_id(fields.get("id")?)?;
    let kind = self.build_expr_kind(fields.get("kind")?)?;
    let span = self.build_span(fields.get("span")?)?;
    let attrs = self.build_attrs(fields.get("attrs")?)?;

    Ok(Expr { id, kind, span, attrs, tokens: None })
}

fn build_expr_kind(&mut self, sexp: &SExp) -> Result<ExprKind> {
    match sexp {
        SExp::List(items) if items[0] == symbol("Binary") => {
            let op = self.build_bin_op(&items[1])?;
            let lhs = self.build_expr(&items[2])?;
            let rhs = self.build_expr(&items[3])?;
            Ok(ExprKind::Binary(op, Box::new(lhs), Box::new(rhs)))
        }
        SExp::List(items) if items[0] == symbol("Paren") => {
            let expr = self.build_expr(&items[1])?;
            Ok(ExprKind::Paren(Box::new(expr)))
        }
        // ... handle all ExprKind variants
    }
}
```

---

## Code Generation (AST → Rust)

### Architecture

The `codegen` module generates Rust source code from AST:

```rust
pub fn generate_rust(crate_node: &Crate) -> Result<String>;

// Module-specific generation
fn generate_item(item: &Item) -> String;
fn generate_expr(expr: &Expr) -> String;
fn generate_ty(ty: &Ty) -> String;
fn generate_pat(pat: &Pat) -> String;
```

### Generation Strategy

1. **Recursive descent** through AST
2. **String concatenation** with proper formatting
3. **Precedence handling** for expressions
4. **Indentation tracking** for blocks
5. **Whitespace management** for readability

### Example: Expression Code Generation

```rust
fn generate_expr(expr: &Expr) -> String {
    match &expr.kind {
        ExprKind::Binary(op, lhs, rhs) => {
            format!("{} {} {}",
                generate_expr(lhs),
                generate_bin_op(op),
                generate_expr(rhs)
            )
        }
        ExprKind::Paren(inner) => {
            format!("({})", generate_expr(inner))
        }
        ExprKind::Try(expr) => {
            format!("{}?", generate_expr(expr))
        }
        ExprKind::Cast { expr, ty } => {
            format!("{} as {}",
                generate_expr(expr),
                generate_ty(ty)
            )
        }
        ExprKind::Break { label, value } => {
            let mut s = "break".to_string();
            if let Some(label) = label {
                s.push_str(&format!(" '{}", label.ident.name));
            }
            if let Some(val) = value {
                s.push_str(&format!(" {}", generate_expr(val)));
            }
            s
        }
        // ... handle all ExprKind variants
    }
}
```

---

## Integration Layer

### syn → oxur_ast Conversion

The `from_syn` module converts `syn` AST to `oxur_ast`:

```rust
pub fn parse_rust_file(source: &str) -> Result<Crate>;
pub fn parse_rust_file_partial(source: &str) -> Result<(Crate, Vec<ErrorComment>)>;

// Currently implemented (Phase 6 scope):
fn convert_item_fn(func: &syn::ItemFn) -> Result<Item>;

// Planned (Phase 6):
// fn convert_item_struct(s: &syn::ItemStruct) -> Result<Item>;
// fn convert_item_enum(e: &syn::ItemEnum) -> Result<Item>;
// fn convert_item_trait(t: &syn::ItemTrait) -> Result<Item>;
// fn convert_item_impl(i: &syn::ItemImpl) -> Result<Item>;
// ... etc
```

### Partial Parsing

For unsupported constructs, the system generates **error comments** instead of failing:

```rust
pub struct ErrorComment {
    pub error_message: String,
    pub rust_code: String,
}

pub fn generate_error_comment(error: &ErrorComment) -> String {
    format!(";; Oxur AST does not support the following Rust code\n\
             ;; Error: {}\n\
             ;;\n\
             ;; {}",
             error.error_message,
             error.rust_code.lines()
                 .map(|line| format!(";; {}", line))
                 .collect::<Vec<_>>()
                 .join("\n"))
}
```

**Example:**
```scheme
;; Oxur AST does not support the following Rust code
;; Error: Expected supported item type (currently only: `fn`), found `struct` item
;;
;; struct Point {
;;     x: i32,
;;     y: i32,
;; }
```

This allows graceful degradation while maintaining context about what couldn't be converted.

---

## Testing Strategy

### Test Categories

1. **Unit Tests** - Module-level functionality
2. **Integration Tests** - End-to-end conversions
3. **Round-trip Tests** - Conversion preservation
4. **Partial Parsing Tests** - Error comment generation
5. **Benchmarks** - Performance tracking

### Round-Trip Testing

All Phase 5 features have comprehensive round-trip tests:

```rust
#[test]
fn test_expr_try_round_trip() {
    // 1. Create AST manually
    let crate_ast = create_test_crate_with_try_expr();

    // 2. AST → S-expr
    let gen = Generator::new();
    let sexp1 = gen.generate_crate(&crate_ast).unwrap();

    // 3. S-expr → Text → S-expr
    let printed = print_sexp(&sexp1);
    let sexp2 = Parser::parse_str(&printed).unwrap();

    // 4. S-expr → AST
    let mut builder = AstBuilder::new();
    let crate2 = builder.build_crate(&sexp2).unwrap();

    // 5. AST → S-expr (verify identical)
    let sexp3 = gen.generate_crate(&crate2).unwrap();
    assert_eq!(sexp1, sexp3);

    // 6. AST → Rust code
    let code = generate_rust(&crate2).unwrap();
    assert!(code.contains("result?"));
}
```

### Test Coverage

Current coverage (as of Phase 5 completion):
- **656 tests passing**
- **95%+ line coverage** for core modules
- **100% coverage** of standard Rust patterns tested

Test files:
```
tests/
├── integration_tests.rs              (Full parsing tests)
├── integration_partial_tests.rs      (Error comment tests)
├── phase5_round_trip_tests.rs        (Phase 5 round-trip tests)
├── stage1_*.rs                       (Basic AST tests)
├── stage2_*.rs                       (Expression tests)
└── ...
```

---

## Performance Considerations

### Optimization Strategies

1. **Lazy evaluation** - Parse only what's needed
2. **Token stream preservation** - Avoid re-parsing for macros
3. **Span tracking** - Minimal overhead for source positions
4. **Box allocation** - Reduce stack usage for recursive structures
5. **String interning** - Reuse common identifiers (future)

### Benchmark Results

```bash
$ cargo bench

parse_rust          time: [1.23 ms 1.25 ms 1.27 ms]
generate_sexp       time: [456 μs 461 μs 467 μs]
parse_sexp          time: [234 μs 237 μs 241 μs]
build_ast           time: [567 μs 573 μs 580 μs]
round_trip          time: [2.89 ms 2.93 ms 2.97 ms]
```

---

## Phase Implementation History

### Phase 1-4: Foundation (Pre-documented)

**Implemented:**
- Core AST structures (Crate, Item, Expr, Stmt)
- S-expression parser and printer
- Basic pattern and type support
- Function item parsing
- Integration with syn
- CLI tool (aster)

**Coverage:** ~60% of essential Rust features

---

### Phase 5: Pattern & Type System Coverage ✨

**Status:** 85% Complete (Priorities 1-4 done, Priority 5 in progress)

**Implemented:**

#### Priority 1: Pattern System (9 new variants)
- ✅ `PatKind::Box` - Box patterns
- ✅ `PatKind::Path` - Path patterns (None, Some, etc.)
- ✅ `PatKind::Range` - Range patterns (1..=5, ..=10)
- ✅ `PatKind::Rest` - Rest patterns (..)
- ✅ `PatKind::Paren` - Parenthesized patterns
- ✅ `PatKind::Type` - Type ascription (x: i32)
- ✅ `PatKind::Const` - Const blocks
- ✅ `PatKind::MacCall` - Macro patterns
- ✅ `PatKind::Err` - Error recovery

#### Priority 2: Type System (5 new variants)
- ✅ `TyKind::BareFn` - Function pointers (fn() -> T)
- ✅ `TyKind::ImplTrait` - Impl trait (impl Iterator)
- ✅ `TyKind::TraitObject` - Trait objects (dyn Display)
- ✅ `TyKind::MacCall` - Macro invocations
- ✅ `TyKind::Paren` - Parenthesized types

#### Priority 3: Expression Gaps (6 new variants)
- ✅ `ExprKind::Paren` - Parenthesized expressions
- ✅ `ExprKind::Try` - Try operator (?)
- ✅ `ExprKind::Cast` - Type casts (as u64)
- ✅ `ExprKind::Break` - Break with value
- ✅ `ExprKind::Continue` - Continue with label
- ✅ `ExprKind::Return` - Return with value

#### Priority 4: Code Generation & Testing
- ✅ All 20 variants have round-trip tests
- ✅ Code generation for all new features
- ✅ Integration with syn for new patterns

#### Priority 5: Documentation (In Progress)
- 🔄 ARCHITECTURE.md (this document - in progress)
- ⏳ Phase 5 completion report (pending)

**Test Results:**
- 656 tests passing (+20 Phase 5 round-trip tests)
- 95%+ coverage for new modules
- All standard Rust patterns verified

**Coverage:** ~70% of essential Rust features (+10% from Phase 5)

---

### Future Phases (Planned)

#### Phase 6: Integration Layer Expansion
**Goal:** Fix parsing bottleneck - expand from_syn to handle all item types
**Impact:** Usability 10% → 85%

#### Phase 7: Generics & Lifetimes
**Goal:** Complete generic type system
**Impact:** Capability 70% → 90%

#### Phase 8: Advanced Features & Completeness
**Goal:** Traits, impls, macros, attributes - production readiness
**Impact:** Capability 90% → 95%+

See `workbench/phase-*-design-doc.md` for detailed planning.

---

## Design Decisions

### Why S-expressions?

1. **Homoiconicity** - Code is data, data is code
2. **Simplicity** - Minimal syntax, easy parsing
3. **Human-editable** - Text format with clear structure
4. **Lisp tradition** - Proven approach for AST representation
5. **Extensibility** - Easy to add new node types

### Why Not JSON/YAML?

- **Verbosity** - JSON requires excessive quoting
- **Structure** - YAML whitespace sensitivity problematic for nested AST
- **Tradition** - S-expressions are the natural choice for AST work

### Why Preserve syn Integration?

- **Maturity** - syn is the de facto Rust parser
- **Correctness** - Leverage syn's extensive testing
- **Features** - Benefit from syn's continuous development
- **Ecosystem** - Interop with other syn-based tools

---

## Contributing

When adding new AST features:

1. **Define AST structures** in `src/ast/*.rs`
2. **Implement generator** in `src/generator/*.rs`
3. **Implement builder** in `src/builder/*.rs`
4. **Implement codegen** in `src/codegen/*.rs`
5. **Add from_syn conversion** in `src/integration/from_syn.rs`
6. **Write round-trip tests** in `tests/`
7. **Update this document** with new variants
8. **Add examples** to README.md

---

## License

See the [main repository](../../) for license information.

---

**Last Updated:** 2025-12-31 (Phase 5 completion)
**Document Version:** 1.0
