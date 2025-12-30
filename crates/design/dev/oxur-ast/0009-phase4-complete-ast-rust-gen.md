# Implementation Plan: aster Phase 4 - Complete Rust Code Generation

## Overview

Implement complete Rust code generation for aster to transform it from "handles Hello World" to "handles all of Rust". Currently, `to_rust.rs` just outputs Debug representation of the AST. Phase 4 adds actual code generation producing compilable Rust source code.

## User's Request

1. Write README examples showing bidirectional conversion (even though not fully working yet)
2. Commit the examples
3. Implement Phase 4 code generation in stages
4. Achieve 95%+ test coverage
5. Verify examples work

## Current State

**What Works:**
- ✓ S-expression parsing (lexer, parser)
- ✓ AstBuilder creates Oxur AST from S-expressions (Phase 1: functions, macros)
- ✓ Generator creates S-expressions from AST (inverse direction)
- ✓ `aster to-ast` (Rust → S-expression) fully functional
- ✓ `aster verify` (round-trip testing) works

**What's Stubbed:**
- ✗ `to_rust.rs` lines 17-19: Just outputs `format!("// AST: {:#?}", crate_node)`
- ✗ No actual Rust code generation
- ✗ No Oxur AST → Rust source conversion

**Dependencies Available:**
- `syn = "2.0"` with full features ✓
- `quote = "1.0"` ✓
- `prettyplease` - NOT in Cargo.toml, would be optional for formatting

## Implementation Strategy

**Approach:** Direct code generation (Oxur AST → String)
- Simpler than building `to_syn` converter
- More control over output format
- Follows Phase 4 design doc exactly
- No additional dependencies required (prettyplease optional)

---

## Stage 0: Prerequisites and Foundation

### Goal
Set up code generation infrastructure and wire it into `to_rust` command.

### Files to Create
1. `src/codegen/mod.rs` - Module structure, public API
2. `src/codegen/rust.rs` - Core `RustCodegen` struct with indentation management

### Files to Modify
1. `src/lib.rs` - Add `pub mod codegen;`
2. `src/commands/to_rust.rs` - Replace Debug output with `codegen::generate_rust()`
3. `Cargo.toml` - Consider adding `prettyplease` as optional dependency

### Key Components
```rust
// src/codegen/rust.rs
pub struct RustCodegen {
    output: String,
    indent: usize,
}

impl RustCodegen {
    pub fn new() -> Self { ... }
    fn indent(&mut self) { self.indent += 4; }
    fn dedent(&mut self) { self.indent -= 4; }
    fn write_indent(&mut self) { ... }
    pub fn generate_crate(&mut self, crate_node: &Crate) -> Result<String> { ... }
}
```

### Testing
- Create `tests/codegen_basic_tests.rs`
- Test: Empty crate generates empty (or minimal) Rust
- Test: Infrastructure compiles

### Success Criteria
- Code compiles
- `to_rust` command uses new codegen infrastructure
- 1-2 basic tests pass

### Estimated Coverage: ~5%

---

## Stage 1: Minimal Viable Code Generation

### Goal
Generate working Rust code for Phase 1-3 features (functions with macros).

### Files to Create
1. `src/codegen/item.rs` - Item code generation (functions)
2. `src/codegen/expr.rs` - Expression/statement code generation
3. `src/codegen/types.rs` - Type and pattern generation (minimal)

### Key Implementations

**item.rs:**
- `generate_item()` - dispatch on ItemKind::Fn
- `generate_fn_item()` - complete function generation
- `generate_fn_header()` - const/async/unsafe/extern
- `generate_param()` - parameters with types
- `generate_fn_ret_ty()` - return types
- `generate_visibility()` - pub/inherited

**expr.rs:**
- `generate_block()` - statement blocks with indentation
- `generate_stmt()` - all StmtKind variants (Semi, Expr, MacCall, Empty)
- `generate_expr()` - Lit, Path, MacCall
- `generate_lit()` - string and int literals
- `generate_mac_call()` - macro invocations
- `generate_path()` - simple paths

**types.rs:**
- `generate_pat()` - Ident patterns only
- `generate_ty()` - Path types only

### Testing Strategy
- Update `tests/commands_to_rust_tests.rs`
- Test hello_world.rs round-trip: Rust → AST → generated Rust → compiles
- Verify macro calls preserved
- Verify function signatures correct

### Success Criteria
- `fn main() { println!("..."); }` generates correctly
- Generated code compiles with rustc
- Round-trip preserves semantics

### Estimated Coverage: ~25%

---

## Stage 2: Control Flow Expressions

### Goal
Add if/match/loops - common control flow patterns.

### Files to Modify
1. `src/ast/expr.rs` - Add ExprKind::{If, Match, While, ForLoop, Loop}, Arm, Label
2. `src/builder/expr.rs` - Build methods for new variants
3. `src/generator/expr.rs` - Generate S-expressions for new variants
4. `src/integration/from_syn.rs` - Convert syn control flow to Oxur
5. `src/codegen/expr.rs` - Generate Rust for control flow

### Key Features
- If/else expressions with proper formatting
- Match expressions with arms and guards
- While, for, and loop expressions
- Break/continue with labels

### Testing
- Test nested if/else
- Test match with multiple patterns
- Test loop labels
- Round-trip tests

### Estimated Coverage: ~35%

---

## Stage 3: Binary/Unary Operations and Calls

### Goal
Operators and function/method calls.

### Files to Modify
1. `src/ast/expr.rs` - Add Binary, Unary, Call, MethodCall, BinOp, UnOp enums
2. `src/builder/expr.rs` - Build ops and calls
3. `src/generator/expr.rs` - Generate S-expressions
4. `src/integration/from_syn.rs` - Convert syn ops
5. `src/codegen/expr.rs` - Generate Rust code with operator precedence

### Operators to Support
**Binary:** +, -, *, /, %, &, |, ^, <<, >>, ==, !=, <, >, <=, >=, &&, ||
**Unary:** !, -, *

### Testing
- Arithmetic expressions
- Logical expressions
- Comparison expressions
- Function calls with multiple arguments
- Method call chains

### Estimated Coverage: ~45%

---

## Stage 4: Struct and Enum Items

### Goal
Data structure definitions.

### Files to Modify
1. `src/ast/item.rs` - Add Struct, Enum, VariantData, EnumDef, Variant, FieldDef
2. `src/builder/item.rs` - Build structs and enums
3. `src/generator/item.rs` - Generate S-expressions
4. `src/integration/from_syn.rs` - Convert syn items
5. `src/codegen/item.rs` - Generate struct/enum Rust code

### Features
- Struct items (named fields, tuple, unit)
- Enum variants (with data)
- Field visibility
- Derive attributes (simplified)

### Testing
- Named struct: `struct Point { x: i32, y: i32 }`
- Tuple struct: `struct Color(u8, u8, u8)`
- Unit struct: `struct Marker;`
- Enum with variants

### Estimated Coverage: ~55%

---

## Stage 5: Trait and Impl Items

### Goal
Traits and implementations.

### Files to Modify
1. `src/ast/item.rs` - Add Trait, Impl, TraitDef, ImplDef, AssocItem, TraitRef
2. `src/builder/item.rs` - Build traits and impls
3. `src/generator/item.rs` - Generate S-expressions
4. `src/integration/from_syn.rs` - Convert syn traits/impls
5. `src/codegen/item.rs` - Generate trait/impl Rust code

### Features
- Trait definitions with methods
- Trait implementations
- Inherent implementations
- Associated types and functions

### Testing
- Trait definition
- Trait impl for type
- Inherent impl
- Default methods

### Estimated Coverage: ~65%

---

## Stage 6: Advanced Patterns and Types

### Goal
Complete pattern matching and type system.

### Files to Create
1. `src/ast/pat.rs` - Complete PatKind enum (Wild, Tuple, Struct, Slice, Or, Ref, etc.)
2. `src/builder/pat.rs` - Pattern building
3. `src/generator/pat.rs` - Pattern S-expression generation

### Files to Modify
1. `src/ast/types.rs` - Expand TyKind (Ref, Ptr, Slice, Array, Tup, Never, etc.)
2. `src/builder/types.rs` - Type building
3. `src/codegen/types.rs` - Complete pattern and type generation

### Features
**Patterns:** Wild `_`, Tuple, Struct, Slice, Or `|`, Ref, Box, Ident with binding
**Types:** References `&T`, `&mut T`, `&'a T`, Pointers `*const T`, `*mut T`, Arrays `[T; N]`, Slices `[T]`, Tuples, Never `!`

### Testing
- Pattern matching in match arms
- Destructuring
- Reference types with lifetimes
- Array and slice types

### Estimated Coverage: ~75%

---

## Stage 7: Remaining Expressions

### Goal
Complete expression coverage (95%+).

### Files to Modify
1. `src/ast/expr.rs` - Add Array, Tup, Field, Index, Assign, Closure, Range, etc.
2. `src/builder/expr.rs` - Build remaining expressions
3. `src/generator/expr.rs` - Generate S-expressions
4. `src/codegen/expr.rs` - Generate Rust code

### Expression Types
- Array/Tuple literals: `[1, 2, 3]`, `(a, b)`
- Field access: `point.x`
- Indexing: `arr[i]`
- Assignment: `x = 5`
- Struct literals: `Point { x: 1, y: 2 }`
- Closures: `|x| x + 1` (basic)
- Ranges: `0..10`, `..=100`

### Testing
- All expression variants
- Nested expressions
- Complex combinations

### Estimated Coverage: ~85%

---

## Stage 8: Remaining Items

### Goal
Complete item coverage (95%+).

### Files to Modify
1. `src/ast/item.rs` - Add Use, Mod, Static, Const, TyAlias, etc.
2. `src/builder/item.rs` - Build remaining items
3. `src/generator/item.rs` - Generate S-expressions
4. `src/codegen/item.rs` - Generate Rust code

### Item Types
- Use declarations: `use std::io;`, `use foo::bar as baz;`
- Modules: `mod foo;`, `mod bar { ... }`
- Static/Const: `static X: i32 = 5;`
- Type aliases: `type Result<T> = std::result::Result<T, Error>;`
- Foreign items (basic)

### Testing
- Use statements (simple, aliases, glob)
- Module declarations
- Constants and statics
- Type aliases

### Estimated Coverage: ~90%

---

## Stage 9: Code Formatting and Polish

### Goal
Make generated code readable and idiomatic.

### Files to Modify/Create
1. `Cargo.toml` - Consider `prettyplease` as optional dependency
2. `src/codegen/format.rs` - Formatting utilities (optional)
3. `src/codegen/rust.rs` - Improve indentation, spacing, comments

### Features
- Proper indentation (4 spaces)
- Blank lines between items
- Trailing commas in lists
- Comment preservation (if possible)
- Optional prettyplease integration

### Testing
- Generated code is rustfmt-compatible
- Code looks idiomatic
- Human-readable output

### Estimated Coverage: ~92%

---

## Stage 10: Comprehensive Testing and Documentation

### Goal
Achieve 95%+ coverage and complete documentation.

### Files to Create
1. `tests/complete_coverage_tests.rs` - Test every variant
2. `tests/round_trip_tests.rs` - Complex real-world examples
3. `crates/oxur-ast/ARCHITECTURE.md` - Architecture documentation

### Files to Update
1. `README.md` - Update with Phase 4 features and examples
2. Add docstring examples throughout codegen module

### Testing Strategy
- Unit test every ExprKind, ItemKind, PatKind, TyKind
- Integration tests with real Rust code
- Round-trip tests: Rust → S-expr → Rust → compiles
- Edge cases and error conditions
- Performance benchmarks

### Coverage Goals
Per the plan file `assets/ai/CLAUDE-CODE-COVERAGE.md`:
- Achieve 95%+ line coverage
- Focus on critical paths first
- Test error conditions
- Use `cargo llvm-cov --summary-only`

### Documentation
- API documentation for public functions
- Architecture overview
- Usage examples in README
- Code generation design decisions

### Estimated Coverage: 95%+

---

## Critical Files

**Core Implementation:**
1. `/Users/oubiwann/lab/oxur/oxur/crates/oxur-ast/src/codegen/rust.rs` - Main code generation engine
2. `/Users/oubiwann/lab/oxur/oxur/crates/oxur-ast/src/codegen/item.rs` - Item generation
3. `/Users/oubiwann/lab/oxur/oxur/crates/oxur-ast/src/codegen/expr.rs` - Expression/statement generation
4. `/Users/oubiwann/lab/oxur/oxur/crates/oxur-ast/src/codegen/types.rs` - Type/pattern generation
5. `/Users/oubiwann/lab/oxur/oxur/crates/oxur-ast/src/commands/to_rust.rs` - Command entry point

**AST Definitions:**
6. `/Users/oubiwann/lab/oxur/oxur/crates/oxur-ast/src/ast/expr.rs` - Expression types
7. `/Users/oubiwann/lab/oxur/oxur/crates/oxur-ast/src/ast/item.rs` - Item types
8. `/Users/oubiwann/lab/oxur/oxur/crates/oxur-ast/src/ast/pat.rs` - Pattern types (new)
9. `/Users/oubiwann/lab/oxur/oxur/crates/oxur-ast/src/ast/types.rs` - Type system

**Supporting Infrastructure:**
10. `/Users/oubiwann/lab/oxur/oxur/crates/oxur-ast/src/builder/` - S-expression → AST
11. `/Users/oubiwann/lab/oxur/oxur/crates/oxur-ast/src/generator/` - AST → S-expression
12. `/Users/oubiwann/lab/oxur/oxur/crates/oxur-ast/src/integration/from_syn.rs` - syn → AST

---

## Workflow for Each Stage

1. **Implement** - Add code for the stage
2. **Test** - Write/update tests to verify functionality
3. **Quality Check** - Run `make`, `make format`, `make lint`, `make test`
4. **Commit** - Create meaningful commit with stage completion
5. **Coverage** - Add tests to improve coverage toward 95%
6. **Commit Coverage** - When >100 lines or major section complete

---

## README Examples to Write (Stage 0)

### Example 1: S-expression → Rust → Binary

Create `/tmp/oxur-hw1/` with:
- `Cargo.toml` - Binary package
- `src/main.sexp` - Hello world in S-expression format
- Show: `aster to-rust src/main.sexp -o src/main.rs`
- Show: `cargo build`
- Show: `./target/debug/oxur-hw1` output

### Example 2: Rust → S-expression → Diff

Create `/tmp/oxur-hw2/` with:
- `Cargo.toml` - Binary package
- `src/main.rs` - Hello world in Rust
- Show: `aster to-ast src/main.rs -o src/main.sexp`
- Show: `diff -u /tmp/oxur-hw1/src/main.sexp /tmp/oxur-hw2/src/main.sexp`

Both examples demonstrate bidirectional conversion and equivalence.

---

## Success Criteria

- ✓ All 10 stages implemented
- ✓ `aster to-rust` generates compilable Rust code
- ✓ Round-trip tests pass (Rust → S-expr → Rust)
- ✓ 95%+ test coverage achieved
- ✓ README examples work end-to-end
- ✓ All quality checks pass (make, lint, test)
- ✓ Phase 4 design doc requirements met

---

## Estimated Timeline

- Stage 0: Foundation (~100 lines, 1 commit)
- Stage 1: Minimal viable (~300 lines, 1 commit + coverage commits)
- Stages 2-8: Incremental expansion (~200-400 lines each, 1+ commits each)
- Stage 9: Polish (~100 lines, 1 commit)
- Stage 10: Testing/Docs (~500 lines tests, 2+ commits)

**Total: ~15-20 commits across all stages**

---

## Notes

- Each stage builds incrementally on previous stages
- Early stages focus on getting *something* working
- Later stages expand coverage systematically
- Coverage work happens throughout, not just at the end
- Commit frequently: after each stage completion and after significant coverage additions
- Follow assets/ai/CLAUDE-CODE-COVERAGE.md for coverage strategy
