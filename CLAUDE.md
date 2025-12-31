# AI Assistant Guide for Oxur Development

**Version:** 1.0
**Last Updated:** 2025-12-31
**Purpose:** Comprehensive guidelines for AI assistants working with the Oxur Rust project

## About This Document

This document provides essential guidance for AI assistants (like Claude Code) when working with the Oxur codebase. It covers project-specific conventions, patterns, workflows, and best practices.

### When to Use This Guide

- **This file (CLAUDE.md)**: Primary reference for general Rust development in Oxur
- **assets/ai/OXUR-SESSION-BOOTSTRAP.md**: Template for starting new Claude sessions with project context
- **assets/ai/CLAUDE-CODE-COVERAGE.md**: Comprehensive guide specifically for test coverage work
- **assets/ai/ai-rust/skills/claude/SKILL.md**: An advanced Rust programming skill
- **assets/ai/ai-rust/guides/NN-*.md**: Supporting subject-specific documents for the above advanced Rust programming skill

Note: if `assets/ai/ai-rust` does not exist on the file system, ask permission to `git clone https://github.com/oxur/ai-rust` to that localtion.

### Quick Navigation

- [Project Overview](#project-overview)
- [Development Environment](#development-environment--tools)
- [Code Organization & Patterns](#code-organization--patterns)
- [Rust Best Practices](#rust-best-practices-for-this-project)
- [Testing Requirements](#testing-requirements)
- [Design Documentation](#design-documentation-integration)
- [Common Tasks & Workflows](#common-tasks--workflows)
- [Git Conventions](#git--commit-conventions)
- [AI Assistant Guidelines](#ai-assistant-specific-guidelines)
- [Resources](#resources--references)
- [Quick Reference Checklists](#quick-reference-checklists)

---

## Project Overview

### What is Oxur?

Oxur is a **Lisp dialect that treats Rust as its compilation target and runtime**. It provides Lisp's expressiveness and metaprogramming power while leveraging Rust's type system, ownership model, and ecosystem.

**Key Philosophy:** Write Rust code using Lisp syntax with 100% bidirectional interoperability.

### Architectural Vision

```
┌─────────────────────────────────────────────────────────────────┐
│                    OXUR COMPILATION PIPELINE                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Oxur Source (Lisp)                                             │
│         ↓                                                       │
│  ┌──────────────────┐                                           │
│  │  oxur-lang       │  Stage 1: Parse → Surface Forms           │
│  │  (Lisp Compiler) │  Stage 2: Expand → Core Forms (IR)        │
│  └──────────────────┘                                           │
│         ↓                                                       │
│  Core Forms (Canonical S-expressions)                           │
│         ↓                                                       │
│  ┌──────────────────┐                                           │
│  │  oxur-comp       │  Stage 3: Lower → Rust AST                │
│  │  (Backend)       │  Stage 4: Codegen → Rust Source           │
│  └──────────────────┘  Stage 5: Compile → Binary (via rustc)    │
│         ↓                                                       │
│  Rust Binary                                                    │
│                                                                 │
│  ┌──────────────────┐                                           │
│  │  oxur-ast        │  Supporting: Bidirectional Rust AST ↔     │
│  │  (AST Library)   │  S-expression conversion                  │
│  └──────────────────┘                                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Workspace Structure

The project is organized as a Cargo workspace with 6 main crates:

```
/Users/oubiwann/lab/oxur/oxur/
├── crates/
│   ├── design/          # Design documentation management (oxd CLI)
│   ├── oxur-ast/        # Rust AST ↔ S-expression (aster CLI) [IN PROGRESS]
│   ├── oxur-cli/        # CLI infrastructure & unified tool [EARLY STAGE]
│   ├── oxur-lang/       # Oxur Lisp compiler [PLANNING]
│   ├── oxur-comp/       # Backend compiler [PLANNING]
│   └── oxur-repl/       # REPL server/client [PLANNING]
├── Cargo.toml           # Workspace configuration
├── Makefile             # Development targets
├── README.md            # Project overview
├── CLAUDE.md            # This file
└── assets/
    └── ai/              # AI assistant documentation
```

### Crate Status & Purpose

#### oxur-ast (In Progress - ~80% Complete)

**Binary:** `aster`
**Purpose:** Bidirectional conversion between Rust AST and canonical S-expressions

- Core functionality: Parse Rust → AST → S-expr and S-expr → AST → Rust
- Integration with `syn` crate for Rust parsing
- Test data organization: examples/ and fixtures/ by complexity
- Commands: to-ast, to-rust, verify, format
- **Key File:** `crates/oxur-ast/src/lib.rs`

#### design (Active - Feature Complete)

**Binary:** `oxd`
**Purpose:** Manage Oxur Design Documents (ODDs) with state tracking

- Document lifecycle: draft → under-review → revised → accepted → active → final
- Git integration for version control
- Auto-indexing and metadata management
- Comprehensive CLI for document operations
- **Key File:** `crates/design/src/lib.rs`

#### oxur-cli (Early Stage)

**Binary:** `oxur` (planned unified CLI)
**Purpose:** Common CLI infrastructure and utilities

- File I/O helpers (stdin/stdout/file handling)
- Colored terminal output
- Progress tracking
- Table formatting (via `tabled` integration)
- **Key File:** `crates/oxur-cli/src/lib.rs`

#### oxur-lang (Planning)

**Purpose:** The Oxur Lisp dialect compiler (frontend)

- Stage 1: Parse Oxur source → Surface Forms
- Stage 2: Macro expansion → Core Forms (canonical IR)
- Type inference and checking
- **Status:** Design phase, not yet implemented

#### oxur-comp (Planning)

**Purpose:** Backend compiler

- Stage 3: Lower Core Forms → Rust AST
- Stage 4: Code generation → Rust source
- Stage 5: Compilation → Binary (via rustc integration)
- **Status:** Design phase, not yet implemented

#### oxur-repl (Planning)

**Purpose:** REPL with three-tier execution strategy

- Tier 1: Interpreter (direct interpretation, <1ms)
- Tier 2: Cached (previously compiled, ~0ms)
- Tier 3: JIT (full compilation, 50-200ms first time)
- **Status:** Design phase, protocol designed

### Core Design Principles

1. **100% Rust Interoperability** - Can call any Rust code, Rust can call any Oxur code
2. **Rust Semantics, Lisp Syntax** - Not Lisp semantics adapted to Rust
3. **Canonical S-expressions** - Single authoritative format for AST representation
4. **Round-trip Preservation** - X → transform → X must preserve meaning
5. **Type-First Design** - Leverage Rust's type system fully
6. **Test-Driven Development** - 95%+ coverage target
7. **Design Documentation** - All architectural decisions documented as ODDs

---

## Development Environment & Tools

### Required Tools

**Rust:**

- Version: 1.75+ (stable channel)
- Edition: 2021
- Toolchain file: `rust-toolchain.toml` in root

**Essential Tools:**

```bash
rustup component add rustfmt clippy
cargo install cargo-llvm-cov
```

### Makefile Targets

The project includes a comprehensive Makefile for common development tasks:

```bash
make build        # Build all binaries
make clean        # Clean build artifacts
make clean-all    # Full clean (cargo clean)
make lint         # Run clippy and rustfmt check
make format       # Format all code with rustfmt
make test         # Run all tests
make coverage     # Generate coverage report
make check        # Build + lint + test
make check-all    # Build + lint + coverage
```

### Development Workflow

Standard development cycle:

```bash
# 1. Make changes
vim crates/oxur-ast/src/builder.rs

# 2. Format code
make format

# 3. Check compilation
cargo check --all

# 4. Run tests
cargo test --all

# 5. Lint
make lint

# 6. Check coverage
make coverage
```

### Testing Framework

**Unit Tests:**

```bash
cargo test                    # All tests
cargo test --lib              # Library tests only
cargo test --package oxur-ast # Specific crate
```

**Coverage:**

```bash
cargo llvm-cov --html                    # Generate HTML report
cargo llvm-cov --summary-only            # Quick summary
open target/llvm-cov/html/index.html     # View report
```

**Property Testing:**

- Using `proptest` for property-based tests
- Define generators for custom types
- Test invariants across random inputs

**Benchmarking:**

- Using `criterion` for performance benchmarks
- Located in `benches/` directory
- Run with `cargo bench`

### Formatting Configuration

From `.rustfmt.toml`:

```toml
edition = "2021"
max_width = 100
use_small_heuristics = "Max"
```

**Important:** Line length is 100 characters, not 80!

---

## Code Organization & Patterns

### Module Organization

#### Workspace Conventions

**Crate naming:** Hyphenated (e.g., `oxur-ast`, `oxur-lang`, NOT `rast` or `oxur_ast`)

**Module structure:**

```rust
// lib.rs - Crate root
pub mod builder;
pub mod generator;
pub mod error;
pub mod sexp;

pub use error::Result;
pub use builder::Builder;
```

**Visibility guidelines:**

- `pub` - Public API, stable across versions
- `pub(crate)` - Internal to crate, can change freely
- `pub(super)` - Visible to parent module only
- (no modifier) - Private to module

**Re-export strategy:**

```rust
// Expose commonly-used types at crate root
pub use builder::Builder;
pub use error::{Result, Error};

// Keep specialized items in modules
pub mod helpers {
    pub fn advanced_function() { }
}
```

#### Crate Structure Pattern

Standard layout for a crate:

```
crates/example/
├── src/
│   ├── lib.rs           # Crate root with public API
│   ├── error.rs         # Error types
│   ├── module_a.rs      # Feature module
│   ├── module_b/        # Complex module as directory
│   │   ├── mod.rs
│   │   ├── submodule.rs
│   │   └── tests.rs     # Module-specific tests
│   └── bin/
│       └── main.rs      # Binary if applicable
├── tests/               # Integration tests
│   └── integration_test.rs
├── benches/             # Benchmarks
│   └── benchmark.rs
├── test-data/           # Test fixtures
│   ├── examples/
│   └── fixtures/
├── Cargo.toml
└── README.md
```

### Error Handling

#### Using thiserror

**Pattern:**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected token {token:?} at {pos}")]
    UnexpectedToken { token: String, pos: Position },

    #[error("Empty input at {pos}")]
    EmptyInput { pos: Position },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid syntax: {message}")]
    InvalidSyntax { message: String },
}

pub type Result<T> = std::result::Result<T, ParseError>;
```

**Key points:**

- One error enum per module (e.g., `ParseError`, `BuildError`, `GenerateError`)
- Use `#[from]` for automatic conversion from other error types
- Include context in error variants (positions, values)
- Provide helpful error messages
- Create type alias `Result<T>` for convenience

#### Position Tracking

**Pattern from oxur-ast:**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub offset: usize,  // Byte offset in source
    pub line: usize,    // Line number (1-based)
    pub column: usize,  // Column number (1-based)
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}
```

**Usage:**

```rust
return Err(ParseError::UnexpectedToken {
    token: token.to_string(),
    pos: Position { offset: 42, line: 3, column: 15 },
});
```

**Why:** Helps users locate errors in their source code.

#### Error Message Conventions

**Good error messages:**

```rust
❌ "Invalid input"
✅ "Unexpected token 'fn' at line 42, column 15. Expected 'struct', 'enum', or 'trait'"

❌ "Parse failed"
✅ "Failed to parse S-expression: Empty input at line 1, column 1"

❌ "Bad type"
✅ "Type mismatch: expected 'Expr', found 'Item' at line 23"
```

**Guidelines:**

- Be specific about what went wrong
- Include location information (line, column)
- Suggest what was expected vs. what was found
- Use consistent terminology from the domain

### Testing Patterns

#### Unit Test Location

**Colocated in modules:**

```rust
// src/builder.rs

pub fn build_item(sexp: &SExp) -> Result<Item> {
    // implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_item_struct() {
        // test implementation
    }

    #[test]
    fn test_build_item_error_invalid_kind() {
        // test error path
    }
}
```

**Benefits:**

- Tests live next to code they test
- Easy to keep in sync
- Access to private items for testing

#### Integration Test Organization

```
tests/
├── builder_tests.rs      # Builder integration tests
├── generator_tests.rs    # Generator integration tests
├── round_trip_tests.rs   # End-to-end round-trip tests
└── common/               # Shared test utilities
    └── mod.rs
```

#### File-Based Test Data

**From oxur-ast pattern:**

```
test-data/
├── examples/
│   ├── simple/           # Basic examples
│   │   ├── hello_world.rs
│   │   └── hello_world.sexp
│   ├── intermediate/     # Moderate complexity
│   └── complex/          # Advanced features
└── fixtures/
    ├── crate/            # By AST type
    ├── item/
    ├── expr/
    └── error-cases/      # Invalid inputs
```

**Test helper pattern:**

```rust
use std::path::PathBuf;

fn parse_example(path: &str) -> SExp {
    let full_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test-data/examples")
        .join(path);
    Parser::parse_file(&full_path)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path, e))
}

#[test]
fn test_hello_world() {
    let sexp = parse_example("simple/hello_world.sexp");
    // assertions
}
```

#### Test Naming Conventions

**Pattern:** `test_<function>_<scenario>_<expectation>`

**Examples:**

```rust
#[test]
fn test_build_item_struct_success() { }

#[test]
fn test_build_item_invalid_kind_returns_error() { }

#[test]
fn test_parse_empty_input_returns_empty_input_error() { }

#[test]
fn test_generate_expr_if_with_else_block() { }
```

**Benefits:**

- Clear what's being tested
- Easy to find related tests
- Self-documenting test purpose

### Builder Patterns

#### When to Use Builders

Use builders when:

- Constructing complex types with many fields
- Step-by-step validation is needed
- Multiple construction paths exist
- Default values make sense for some fields

**Example from oxur-ast:**

```rust
pub struct Builder {
    // internal state
}

impl Builder {
    pub fn new() -> Self {
        Self { /* defaults */ }
    }

    pub fn build_item(&self, sexp: &SExp) -> Result<Item> {
        self.validate_node(sexp)?;
        let kind = self.build_item_kind(sexp)?;
        let vis = self.extract_visibility(sexp)?;
        // ...
        Ok(Item { kind, vis, /* ... */ })
    }

    fn build_item_kind(&self, sexp: &SExp) -> Result<ItemKind> {
        // focused implementation
    }

    fn validate_node(&self, sexp: &SExp) -> Result<()> {
        // validation logic
    }
}
```

#### Builder Method Organization

**Strategy:** Split by responsibility

```rust
impl Builder {
    // === Public API ===
    pub fn build_crate(&self, sexp: &SExp) -> Result<Crate> { }
    pub fn build_item(&self, sexp: &SExp) -> Result<Item> { }
    pub fn build_expr(&self, sexp: &SExp) -> Result<Expr> { }

    // === Item Builders (private) ===
    fn build_item_kind(&self, sexp: &SExp) -> Result<ItemKind> { }
    fn build_fn_item(&self, sexp: &SExp) -> Result<ItemFn> { }
    fn build_struct_item(&self, sexp: &SExp) -> Result<ItemStruct> { }

    // === Expression Builders (private) ===
    fn build_expr_kind(&self, sexp: &SExp) -> Result<ExprKind> { }
    fn build_if_expr(&self, sexp: &SExp) -> Result<ExprIf> { }

    // === Helpers (private) ===
    fn extract_field<T>(&self, sexp: &SExp, field: &str) -> Result<T> { }
    fn validate_node(&self, sexp: &SExp) -> Result<()> { }
}
```

**Benefits:**

- Clear public API surface
- Internal methods grouped by purpose
- Easy to navigate and maintain

#### Validation Strategies

**Validate early:**

```rust
pub fn build_item(&self, sexp: &SExp) -> Result<Item> {
    // Validate structure upfront
    self.validate_is_list(sexp)?;
    self.validate_has_head(sexp, "Item")?;
    self.validate_required_fields(sexp, &[":kind", ":vis"])?;

    // Then build
    let kind = self.build_item_kind(sexp)?;
    let vis = self.extract_visibility(sexp)?;
    Ok(Item { kind, vis })
}
```

**Return specific errors:**

```rust
fn validate_has_head(&self, sexp: &SExp, expected: &str) -> Result<()> {
    match sexp.head() {
        Some(head) if head == expected => Ok(()),
        Some(other) => Err(BuildError::UnexpectedHead {
            expected: expected.to_string(),
            found: other.to_string(),
        }),
        None => Err(BuildError::MissingHead),
    }
}
```

### CLI Patterns

#### Using clap with Derive

**Pattern:**

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aster")]
#[command(about = "AST manipulation and conversion tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert Rust source to S-expression AST
    ToAst {
        /// Input Rust file
        #[arg(short, long)]
        input: PathBuf,

        /// Output S-expression file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Compact output (no pretty printing)
        #[arg(short, long)]
        compact: bool,
    },

    /// Convert S-expression AST to Rust source
    ToRust {
        #[arg(short, long)]
        input: PathBuf,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}
```

#### Command Dispatch Pattern

```rust
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    execute_command(cli.command)
}

fn execute_command(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::ToAst { input, output, compact } => {
            commands::to_ast(input, output, compact)
        }
        Commands::ToRust { input, output } => {
            commands::to_rust(input, output)
        }
    }
}
```

#### Colored Output Standards

**Using the colored crate:**

```rust
use colored::*;

// Success messages (green)
println!("{}", "✓ Conversion successful".green());

// Error messages (red)
eprintln!("{}", "✗ Error: invalid syntax".red());

// Info messages (cyan)
println!("{}", "→ Processing file...".cyan());

// Warning messages (yellow)
println!("{}", "⚠ Warning: deprecated syntax".yellow());
```

**oxur-cli wrapper functions:**

```rust
use oxur_cli::common::output::{success, error, info, warning};

success("Operation completed!");
error("Something went wrong");
info("Processing files...");
warning("Deprecated API usage");
```

---

## Rust Best Practices for This Project

### Naming Conventions

**Crates:**

- Format: `oxur-component` (hyphenated)
- Examples: `oxur-ast`, `oxur-lang`, `oxur-comp`
- NOT: `rast`, `oxur_ast`, `oxurAST`

**Modules:**

- Format: `snake_case`
- Examples: `builder`, `ast_types`, `error_handling`

**Types:**

- Format: `PascalCase`
- Examples: `Item`, `ExprKind`, `ParseError`
- Traits: `Builder`, `Generator`, `Visitor`
- Enums: `ItemKind`, `Visibility`, `Mutability`

**Functions:**

- Format: `snake_case`
- Examples: `build_item`, `parse_file`, `generate_code`
- Constructors: `new`, `with_capacity`, `from_parts`

**Constants:**

- Format: `SCREAMING_SNAKE_CASE`
- Examples: `MAX_DEPTH`, `DEFAULT_CAPACITY`

**Type Parameters:**

- Single letter for generic: `T`, `E`, `K`, `V`
- Descriptive for specific: `Item`, `Expr` (when constrained)

### Code Style

#### Line Length

**100 characters maximum** (configured in .rustfmt.toml)

```rust
// Good - under 100 characters
pub fn build_item_from_sexp(sexp: &SExp) -> Result<Item> {

// Break long lines
pub fn complex_function_with_many_parameters(
    param1: Type1,
    param2: Type2,
    param3: Type3,
) -> Result<ReturnType> {
```

#### Import Organization

**Groups (separated by blank lines):**

1. Standard library
2. External crates
3. Internal crate modules
4. Parent/sibling modules (relative imports)

```rust
// Standard library
use std::collections::HashMap;
use std::path::PathBuf;

// External crates
use anyhow::Result;
use colored::*;
use syn::Item;

// Internal modules
use crate::builder::Builder;
use crate::error::ParseError;
use crate::sexp::SExp;

// Relative
use super::helpers;
```

#### Documentation Comments

**Module-level:**

```rust
//! Builder module for constructing Rust AST from S-expressions.
//!
//! This module provides the `Builder` type which handles the conversion
//! from canonical S-expression format to Rust's AST types via the `syn` crate.
//!
//! # Examples
//!
//! ```no_run
//! use oxur_ast::builder::Builder;
//! use oxur_ast::sexp::Parser;
//!
//! let sexp = Parser::parse_str("(Item ...)")?;
//! let builder = Builder::new();
//! let item = builder.build_item(&sexp)?;
//! ```
```

**Function-level:**

```rust
/// Builds a Rust AST `Item` from an S-expression node.
///
/// # Arguments
///
/// * `sexp` - S-expression representing an Item node
///
/// # Returns
///
/// Returns `Ok(Item)` on success, or a `BuildError` if the S-expression
/// is malformed or missing required fields.
///
/// # Examples
///
/// ```no_run
/// let sexp = parse_example("item.sexp");
/// let item = builder.build_item(&sexp)?;
/// assert!(matches!(item.kind, ItemKind::Fn(_)));
/// ```
pub fn build_item(&self, sexp: &SExp) -> Result<Item> {
    // implementation
}
```

#### Visibility Guidelines

**pub (public API):**

- Use for types/functions meant to be used by external crates
- Consider stability - harder to change later
- Document thoroughly

**pub(crate) (internal to crate):**

- Use for cross-module helpers
- Can change freely without breaking external users
- Good default for "might be useful elsewhere"

**pub(super) (parent module only):**

- Use for tightly coupled parent-child modules
- Rare in practice

**(no modifier) - private:**

- Default - use unless there's a reason for visibility
- Implementation details
- Helper functions

**Example:**

```rust
// Public API
pub struct Builder { }

// Internal to crate
pub(crate) fn validate_node(sexp: &SExp) -> Result<()> { }

// Private
fn parse_field(sexp: &SExp, name: &str) -> Option<SExp> { }
```

### Type System Usage

#### Specific vs Generic Types

**Prefer specific types for clarity:**

```rust
// Good - clear what each parameter is
pub fn build_item(item_sexp: &SExp, context: &Context) -> Result<Item> {

// Avoid - too generic
pub fn build<T, C>(data: &T, ctx: &C) -> Result<Output> {
```

**Use generics when genuinely reusable:**

```rust
// Good - truly generic collection operation
pub fn map_nodes<F, T>(sexps: &[SExp], f: F) -> Vec<T>
where
    F: Fn(&SExp) -> T,
{
    sexps.iter().map(f).collect()
}
```

#### Newtypes for Domain Concepts

**Pattern:**

```rust
// Wrap primitive types for type safety
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocNumber(usize);

impl DocNumber {
    pub fn new(n: usize) -> Self {
        DocNumber(n)
    }

    pub fn value(&self) -> usize {
        self.0
    }
}

// Now can't accidentally mix up different numeric types
fn get_document(num: DocNumber) -> Result<Document> {
    // implementation
}
```

**Benefits:**

- Type safety (can't pass wrong kind of number)
- Self-documenting code
- Can add domain-specific methods

#### enum vs struct

**Use enum when:**

- Represents mutually exclusive variants
- Needs match exhaustiveness checking
- Each variant may have different data

```rust
pub enum ItemKind {
    Fn(ItemFn),
    Struct(ItemStruct),
    Enum(ItemEnum),
    Trait(ItemTrait),
}
```

**Use struct when:**

- Represents a single concept with multiple aspects
- All fields always present (or Option if sometimes absent)

```rust
pub struct Item {
    pub vis: Visibility,
    pub ident: Ident,
    pub kind: ItemKind,
    pub attrs: Vec<Attribute>,
}
```

#### Trait Design

**Keep traits focused:**

```rust
// Good - single responsibility
pub trait Builder {
    type Output;
    fn build(&self, sexp: &SExp) -> Result<Self::Output>;
}

// Avoid - too many responsibilities
pub trait BuilderGeneratorValidator {
    fn build(&self) -> Result<Output>;
    fn generate(&self) -> String;
    fn validate(&self) -> bool;
}
```

**Use associated types for related types:**

```rust
pub trait Parser {
    type Output;
    type Error;

    fn parse(&self, input: &str) -> Result<Self::Output, Self::Error>;
}
```

### Performance Considerations

#### &str vs String

**&str for:**

- Function parameters (borrowed)
- Literals
- Slices of strings

**String for:**

- Owned data
- Building strings
- Return values (when ownership needed)

```rust
// Good
pub fn process_name(name: &str) -> String {
    format!("Hello, {}", name)
}

// Avoid - forces allocation
pub fn process_name(name: String) -> String {
    format!("Hello, {}", name)
}
```

#### Vec vs Slice

**&[T] for parameters:**

```rust
// Good - accepts Vec, array, or slice
pub fn process_items(items: &[Item]) -> usize {
    items.len()
}

// Avoid - only accepts Vec
pub fn process_items(items: &Vec<Item>) -> usize {
    items.len()
}
```

**Vec\<T\> for:**

- Owned collections
- Growable data
- Return values

#### Clone vs Reference

**Prefer borrowing:**

```rust
// Good - no allocation
pub fn build_item(&self, sexp: &SExp) -> Result<Item> {

// Avoid - unnecessary clone
pub fn build_item(&self, sexp: SExp) -> Result<Item> {
```

**Clone when needed:**

```rust
// OK - need owned copy for thread
let data = data.clone();
thread::spawn(move || process(data));

// OK - need to mutate without affecting original
let mut modified = original.clone();
modified.update();
```

#### Allocation Patterns

**Pre-allocate when size known:**

```rust
// Good
let mut items = Vec::with_capacity(expected_count);

// Works but may reallocate
let mut items = Vec::new();
```

**Avoid unnecessary allocations:**

```rust
// Good - reuse string
let mut result = String::new();
for item in items {
    result.push_str(&item.to_string());
}

// Avoid - many temporary strings
let result = items.iter()
    .map(|i| i.to_string())
    .collect::<Vec<String>>()
    .join("");
```

---

## Testing Requirements

### Coverage Targets

**Minimum requirements:**

- **Overall coverage:** ≥ 95%
- **Module coverage:** ≥ 90% (no stragglers)
- **Error paths:** 100% tested
- **Public API:** 100% tested

**For comprehensive testing guidance, see:** `assets/ai/CLAUDE-CODE-COVERAGE.md`

### Test Quality Standards

#### Test Naming

**Format:** `test_<function>_<scenario>_<expectation>`

**Examples:**

```rust
#[test]
fn test_build_item_struct_succeeds() {
    // Test successful struct building
}

#[test]
fn test_build_item_missing_kind_returns_error() {
    // Test error case
}

#[test]
fn test_parse_empty_file_returns_empty_vec() {
    // Test edge case
}
```

#### Assertion Quality

**Be specific:**

```rust
// Good
assert_eq!(result.len(), 3);
assert!(matches!(result, Ok(Item { .. })));
assert_eq!(error.to_string(), "Expected 'Fn', found 'Struct' at line 42");

// Avoid
assert!(result.is_ok());  // Too vague
assert!(x);  // What does x represent?
```

**Test behavior, not implementation:**

```rust
// Good - tests behavior
#[test]
fn test_builder_creates_valid_item() {
    let item = builder.build_item(&sexp).unwrap();
    assert_eq!(item.ident, "main");
    assert!(matches!(item.vis, Visibility::Public));
}

// Avoid - tests internal details
#[test]
fn test_builder_uses_hashmap_internally() {
    // Testing implementation detail
}
```

#### Test Data Organization

**From oxur-ast pattern:**

```
test-data/
├── examples/           # End-to-end examples
│   ├── simple/        # Basic cases (hello_world, single_function)
│   ├── intermediate/  # Moderate complexity
│   └── complex/       # Advanced features
├── fixtures/          # Targeted test cases
│   ├── crate/        # Crate-level nodes
│   ├── item/         # Item-level nodes
│   ├── expr/         # Expression-level nodes
│   ├── stmt/         # Statement-level nodes
│   └── error-cases/  # Invalid inputs that should fail
└── README.md         # Documentation of test data
```

**Usage pattern:**

```rust
#[test]
fn test_parse_hello_world() {
    let sexp = parse_example("simple/hello_world.sexp");
    let item = builder.build_item(&sexp).unwrap();
    assert_eq!(item.ident, "main");
}

#[test]
fn test_parse_invalid_returns_error() {
    let result = parse_fixture("error-cases/missing_kind.sexp");
    assert!(result.is_err());
}
```

#### Round-Trip Testing

**Critical for AST conversions:**

```rust
#[test]
fn test_round_trip_struct_definition() {
    // Rust → AST → S-expr → AST → Rust should equal original
    let original_rust = "struct Point { x: i32, y: i32 }";

    // Parse to AST
    let ast1 = parse_rust(original_rust).unwrap();

    // Convert to S-expression
    let sexp = generate_sexp(&ast1).unwrap();

    // Convert back to AST
    let ast2 = build_ast(&sexp).unwrap();

    // Generate Rust source
    let generated_rust = generate_rust(&ast2).unwrap();

    // Should be semantically equivalent (formatting may differ)
    assert_eq!(normalize(original_rust), normalize(generated_rust));
}
```

### Test Types

#### Unit Tests

**Characteristics:**

- Fast (< 1s for all unit tests)
- Isolated (no I/O, no network)
- Colocated with code (#[cfg(test)] mod tests)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_functionality() {
        let result = function_under_test(input);
        assert_eq!(result, expected);
    }
}
```

#### Integration Tests

**Characteristics:**

- Test multiple modules together
- Located in `tests/` directory
- Test public API only

```rust
// tests/builder_integration_test.rs
use oxur_ast::builder::Builder;
use oxur_ast::sexp::Parser;

#[test]
fn test_build_complete_crate() {
    let sexp = Parser::parse_file("test-data/examples/simple/hello_world.sexp")?;
    let builder = Builder::new();
    let krate = builder.build_crate(&sexp)?;
    assert_eq!(krate.items.len(), 1);
}
```

#### Property Tests

**Using proptest:**

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_parse_generated_sexp_never_panics(s in "\\PC*") {
        // Should never panic, even on invalid input
        let _ = Parser::parse_str(&s);
    }

    #[test]
    fn test_round_trip_preserves_meaning(
        ident in "[a-z][a-z0-9_]*",
        value in any::<i32>(),
    ) {
        let original = format!("const {}: i32 = {};", ident, value);
        let round_tripped = round_trip(&original)?;
        assert_eq!(normalize(&original), normalize(&round_tripped));
    }
}
```

#### Benchmarks

**Using criterion:**

```rust
// benches/builder_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oxur_ast::builder::Builder;

fn benchmark_build_item(c: &mut Criterion) {
    let sexp = load_test_sexp("fixtures/item/struct.sexp");
    let builder = Builder::new();

    c.bench_function("build_item_struct", |b| {
        b.iter(|| {
            builder.build_item(black_box(&sexp))
        })
    });
}

criterion_group!(benches, benchmark_build_item);
criterion_main!(benches);
```

---

## Design Documentation Integration

### Oxur Design Documents (ODDs)

**Location:** `crates/design/docs/`

**Purpose:** All significant architectural decisions and designs are documented as ODDs before implementation.

### State-Based Organization

Documents progress through states:

```
01-draft/           # Initial drafts, work in progress
02-under-review/    # Ready for review
03-revised/         # Revisions based on feedback
04-accepted/        # Approved but not yet implemented
05-active/          # Currently being implemented
06-final/           # Implemented and stable
07-deferred/        # Postponed
08-rejected/        # Decided against
09-withdrawn/       # Author withdrew
10-superseded/      # Replaced by newer doc
```

### YAML Frontmatter

**Required fields:**

```yaml
---
number: 1
title: "Document Title"
author: "Author Name"
created: 2024-01-01
updated: 2024-01-15
state: Active
---
```

**Optional fields:**

```yaml
tags: [Phase-0, Architecture]
component: oxur-ast
supersedes: 0002
superseded_by: null
```

### Key Design Documents

**Essential reading for AI assistants:**

1. **0001: Oxur Letter of Intent** (`05-active/`)
   - Overall vision and philosophy
   - Why Oxur exists
   - Design principles

2. **0003: Canonical S-Expression Format** (`05-active/`)
   - Specification for Rust AST ↔ S-expr format
   - Syntax and semantics
   - Examples for all AST node types

3. **0013: Compilation Chain Architecture** (`05-active/`)
   - Five-stage compilation pipeline
   - Core Forms as IR
   - How all components fit together

4. **0004-0007: oxur-ast Phase Documents** (`06-final/`)
   - Phase 0: S-expression infrastructure
   - Phase 1: AST types and builder
   - Phase 2: Generator (AST → S-expr)
   - Phase 3: Integration, testing, CLI

5. **0020: Pattern & Type System Coverage** (`04-accepted/`)
   - Complete coverage of Rust patterns and types
   - Phase 5 implementation guide

### Document Lifecycle

**Creating a new design doc:**

```bash
./bin/oxd new "My Design Title"
# Creates draft in 01-draft/ with next available number
```

**Transitioning states:**

```bash
./bin/oxd transition 0042 under-review
./bin/oxd transition 0042 accepted
./bin/oxd transition 0042 active
./bin/oxd transition 0042 final
```

### Using Design Docs

**Before implementing a feature:**

1. Check if design doc exists: `./bin/oxd list`
2. Read relevant docs: `./bin/oxd show 0003`
3. If no doc exists and feature is non-trivial, create one
4. Discuss design before writing code

**When making architectural changes:**

1. Update the relevant design doc
2. Transition to `revised` if significant changes
3. Reference doc number in commit messages

**In code comments:**

```rust
// Implementation of S-expression format spec (ODD-0003)
pub fn parse_item(sexp: &SExp) -> Result<Item> {
    // See ODD-0003 section 3.2 for Item format specification
}
```

---

## Common Tasks & Workflows

### Adding a New Feature

**Step-by-step:**

1. **Check for design doc:**

   ```bash
   ./bin/oxd list | grep -i "feature name"
   ```

2. **Read relevant docs:**

   ```bash
   ./bin/oxd show <number>
   ```

3. **Create design doc if needed:**

   ```bash
   ./bin/oxd new "Feature: My New Feature"
   # Edit the created file
   ./bin/oxd transition <number> accepted
   ```

4. **Write tests first (TDD):**

   ```rust
   #[test]
   fn test_new_feature_basic_case() {
       // This should fail initially
       let result = new_feature(input);
       assert_eq!(result, expected);
   }
   ```

5. **Implement feature:**

   ```rust
   pub fn new_feature(input: &Input) -> Result<Output> {
       // Implementation
   }
   ```

6. **Run tests:**

   ```bash
   cargo test
   ```

7. **Check coverage:**

   ```bash
   make coverage
   # Ensure feature is 95%+ covered
   ```

8. **Run linting:**

   ```bash
   make lint
   make format
   ```

9. **Update documentation:**
   - Add doc comments to public items
   - Update README if public API changed
   - Update design doc to `active` or `final`

### Refactoring Code

**Step-by-step:**

1. **Ensure comprehensive test coverage:**

   ```bash
   cargo llvm-cov --html
   # Open report, verify area to refactor is well-tested
   ```

2. **Make incremental changes:**
   - Refactor one function/module at a time
   - Keep changes focused and reviewable

3. **Run tests after each change:**

   ```bash
   cargo test --lib  # Fast feedback
   ```

4. **Verify no behavioral changes:**
   - All tests should still pass
   - Coverage should not decrease

5. **Update design docs if architecture changed:**

   ```bash
   # Edit relevant design doc
   ./bin/oxd transition <number> revised
   ```

6. **Final verification:**

   ```bash
   make check-all
   ```

### Fixing Bugs

**Step-by-step:**

1. **Write test that reproduces bug:**

   ```rust
   #[test]
   fn test_bug_xyz_reproduction() {
       // This should fail initially, reproducing the bug
       let result = function_with_bug(problematic_input);
       assert_eq!(result, expected_correct_output);
   }
   ```

2. **Fix the bug:**
   - Understand root cause
   - Fix implementation
   - Avoid band-aid solutions

3. **Ensure test passes:**

   ```bash
   cargo test test_bug_xyz_reproduction
   ```

4. **Add regression test:**
   - Keep the reproduction test
   - Add related edge cases

5. **Check for similar bugs:**
   - Search codebase for similar patterns
   - Add tests for related cases

6. **Verify coverage:**

   ```bash
   make coverage
   ```

### Adding Tests

**Systematic approach:**

1. **Run coverage report:**

   ```bash
   cargo llvm-cov --html
   open target/llvm-cov/html/index.html
   ```

2. **Identify uncovered lines:**
   - Red lines: never executed
   - Yellow lines: partially covered (some branches)

3. **Understand why uncovered:**
   - Error path not tested?
   - Edge case not tested?
   - Dead code that should be removed?

4. **Write tests for missing paths:**

   ```rust
   #[test]
   fn test_error_path_invalid_input() {
       let result = function(invalid_input);
       assert!(result.is_err());
   }
   ```

5. **Verify coverage improved:**

   ```bash
   cargo llvm-cov --summary-only
   # Should show increased percentage
   ```

6. **Repeat until ≥ 95%:**
   - See `assets/ai/CLAUDE-CODE-COVERAGE.md` for comprehensive guide

### Working with AST

#### Using the aster CLI

**Convert Rust to S-expression:**

```bash
./bin/aster to-ast -i examples/hello.rs -o examples/hello.sexp
```

**Convert S-expression to Rust:**

```bash
./bin/aster to-rust -i examples/hello.sexp -o examples/hello_generated.rs
```

**Verify round-trip:**

```bash
./bin/aster verify examples/hello.rs
# Checks: Rust → S-expr → Rust produces equivalent code
```

**Format S-expression:**

```bash
./bin/aster format -i input.sexp -o formatted.sexp
```

#### S-Expression Format

**Basic structure (from ODD-0003):**

```lisp
(NodeType
  :field1 value1
  :field2 value2
  :field3 (NestedNode
            :nested-field value))
```

**Example - Struct definition:**

```lisp
(Item
  :vis Public
  :ident (Ident :name "Point" :span (Span :lo 0 :hi 5))
  :kind (Struct
    :fields (Fields
      :named [(Field
                :vis Public
                :ident (Ident :name "x")
                :ty (Type :path "i32"))
              (Field
                :vis Public
                :ident (Ident :name "y")
                :ty (Type :path "i32"))])
    :generics (Generics :params [])
    :span (Span :lo 0 :hi 30)))
```

#### Round-Trip Testing

**Pattern:**

```rust
#[test]
fn test_round_trip_struct() {
    let original = r#"
        pub struct Point {
            pub x: i32,
            pub y: i32,
        }
    "#;

    // Rust → AST
    let ast1 = parse_rust(original)?;

    // AST → S-expr
    let sexp = generator.generate(&ast1)?;

    // S-expr → AST
    let ast2 = builder.build(&sexp)?;

    // AST → Rust
    let generated = codegen.generate_rust(&ast2)?;

    // Compare semantically (not textually - formatting may differ)
    assert_ast_equivalent(&ast1, &ast2);
}
```

#### Test Data Organization

**File naming:**

```
test-data/
  examples/
    simple/
      hello_world.rs          # Original Rust
      hello_world.sexp        # S-expression equivalent
    intermediate/
      generic_function.rs
      generic_function.sexp
```

**Loading test data:**

```rust
fn load_test_pair(name: &str) -> (String, SExp) {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test-data/examples");

    let rust_src = fs::read_to_string(base.join(format!("{}.rs", name)))?;
    let sexp = Parser::parse_file(&base.join(format!("{}.sexp", name)))?;

    (rust_src, sexp)
}
```

---

## Git & Commit Conventions

### Commit Messages

**Format options:**

1. **Free-form descriptive** (preferred for Oxur):

   ```
   Add comprehensive builder tests for Item types

   Implements test cases for struct, enum, and trait item building.
   Achieves 95%+ coverage on builder.rs.

   Related: ODD-0004
   ```

2. **Conventional commits** (optional):

   ```
   feat: add S-expression file I/O capabilities
   fix: correct position tracking in parser
   docs: update README with aster CLI examples
   test: add property tests for round-trip conversion
   refactor: split builder into focused methods
   ```

**Guidelines:**

- **Subject line**: Imperative mood ("Add feature" not "Added feature" or "Adds feature")
- **Body**: Explain WHY, not WHAT (code shows what)
- **References**: Link to design docs when relevant (e.g., "Implements ODD-0003")
- **Length**: Subject ≤ 72 chars, body wrapped at 100 chars

**Examples:**

```
Good:
✓ "Implement Position tracking for parse errors"
✓ "Refactor builder into separate methods by AST type"
✓ "Add test coverage for error paths in generator"

Avoid:
✗ "Fixed stuff"
✗ "Updated files"
✗ "WIP"  (in final commits)
```

### Branch Strategy

**Main branch:**

- Always stable and passing tests
- Protected (requires reviews for direct pushes)

**Feature branches:**

- Format: `feature/descriptive-name` or `descriptive-name`
- Examples: `feature/round-trip-tests`, `builder-refactor`

**Bug fix branches:**

- Format: `fix/bug-description`
- Examples: `fix/position-tracking`, `fix/parse-error-handling`

**Workflow:**

```bash
# Create branch
git checkout -b feature/my-feature

# Work on feature
git add .
git commit -m "Implement first part of feature"

# Keep synced with main
git fetch origin
git rebase origin/main

# Push when ready
git push -u origin feature/my-feature
```

### Pull Request Guidelines

**Before creating PR:**

- [ ] All tests pass (`cargo test --all`)
- [ ] Coverage ≥ 95% (`make coverage`)
- [ ] Linting passes (`make lint`)
- [ ] Code formatted (`make format`)
- [ ] No warnings in build
- [ ] Design docs updated (if architectural changes)
- [ ] README/docs updated (if public API changed)

**PR Description should include:**

- Summary of changes
- Motivation (why this change)
- Testing performed
- Related design docs (if any)
- Breaking changes (if any)

**Example PR description:**

```markdown
## Summary
Refactors the Builder implementation to split methods by AST node type,
improving code organization and maintainability.

## Motivation
The original Builder had all build methods in one file (500+ lines).
This splits them into focused modules, making the code easier to navigate
and test.

## Changes
- Created builder/items.rs for item building
- Created builder/exprs.rs for expression building
- Moved validation helpers to builder/helpers.rs
- Updated tests to match new structure

## Testing
- All existing tests pass
- Coverage remains at 97.2%
- Added tests for edge cases uncovered during refactor

## Related
Implements approach discussed in ODD-0011

## Breaking Changes
None - purely internal refactoring
```

### Commit Etiquette

**Commit frequently:**

- Small, focused commits are better than large ones
- Each commit should be a logical unit
- Should compile and pass tests (when practical)

**Avoid:**

- Committing commented-out code
- Committing debug print statements
- Committing TODO comments without tracking
- Committing merge artifacts

**Amending commits:**

```bash
# Fix something in last commit
git add .
git commit --amend --no-edit

# Reword last commit message
git commit --amend
```

**Interactive rebase for cleanup:**

```bash
# Clean up last 3 commits before pushing
git rebase -i HEAD~3

# Options: pick, squash, reword, drop
```

---

## AI Assistant Specific Guidelines

### When Writing Code

#### Always Read Existing Code First

**Before modifying any file:**

```
1. Read the file completely
2. Understand existing patterns
3. Check related files in same module
4. Look for similar implementations elsewhere
```

**Example:**

```
User: "Add a build_trait_item method"

Steps:
1. Read src/builder.rs completely
2. Find existing build_*_item methods
3. Understand the pattern they follow
4. Implement new method following same pattern
```

#### Follow Established Patterns

**Don't introduce new patterns without reason:**

```rust
// Project uses this pattern for builders:
fn build_item(&self, sexp: &SExp) -> Result<Item> {
    self.validate_node(sexp)?;
    let kind = self.build_item_kind(sexp)?;
    // ...
}

// So new builders should follow it:
fn build_trait_item(&self, sexp: &SExp) -> Result<ItemTrait> {
    self.validate_node(sexp)?;  // Same pattern
    let kind = self.build_trait_kind(sexp)?;
    // ...
}
```

#### Don't Over-Engineer

**Keep it simple:**

```rust
// User asks: "Add validation for empty identifier"

// Good - simple and direct
fn validate_ident(ident: &str) -> Result<()> {
    if ident.is_empty() {
        return Err(Error::EmptyIdentifier);
    }
    Ok(())
}

// Avoid - over-engineered
struct IdentValidator {
    rules: Vec<Box<dyn ValidationRule>>,
    cache: HashMap<String, bool>,
}
// ... unless project has this pattern already
```

#### Reference Similar Code

**Look for examples in the codebase:**

```
User: "Add tests for the new feature"

Steps:
1. Find similar feature tests
2. Look at test organization
3. Check test naming convention
4. Follow the same structure
```

### When Testing

#### Target 95%+ Coverage Systematically

**Process:**

```
1. Run: cargo llvm-cov --html
2. Open: target/llvm-cov/html/index.html
3. Find uncovered lines in current module
4. Write tests for those lines
5. Verify tests pass
6. Re-run coverage
7. Repeat until ≥ 95%
```

**See `assets/ai/CLAUDE-CODE-COVERAGE.md` for comprehensive guide.**

#### Don't Skip "Trivial" Tests

```rust
// Even trivial getters should be tested
pub fn name(&self) -> &str {
    &self.name
}

#[test]
fn test_name_returns_correct_value() {
    let item = create_test_item("foo");
    assert_eq!(item.name(), "foo");
}
```

**Why:**

- Ensures behavior is documented
- Catches future changes that break assumptions
- Coverage tools count every line

#### Test Error Paths Thoroughly

```rust
// For every error variant, have a test
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Empty input")]
    EmptyInput,

    #[error("Unexpected token {0}")]
    UnexpectedToken(String),
}

// Tests:
#[test]
fn test_parse_empty_input_returns_error() {
    assert!(matches!(
        parse(""),
        Err(ParseError::EmptyInput)
    ));
}

#[test]
fn test_parse_invalid_token_returns_error() {
    let result = parse("invalid");
    assert!(matches!(
        result,
        Err(ParseError::UnexpectedToken(_))
    ));
}
```

#### Use Existing Test Patterns

```rust
// Project has helper for loading test data:
fn parse_example(path: &str) -> SExp { }

// Use it, don't reinvent:
#[test]
fn test_new_feature() {
    let sexp = parse_example("simple/new_feature.sexp");  // ✓ Use helper
    // not:
    // let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))...  // ✗ Duplicate
}
```

### When Refactoring

#### Preserve Existing Behavior

**Unless explicitly asked to change:**

```
User: "Refactor the builder module"

Do:
- Reorganize code structure
- Improve naming
- Split large functions
- Keep all tests passing

Don't:
- Change error messages
- Modify return types
- Add new features
- Remove existing functionality
```

#### Make Incremental Changes

```
Large refactor strategy:
1. Run tests (all pass)
2. Make small change
3. Run tests (verify still pass)
4. Commit
5. Repeat

Not:
1. Rewrite entire module
2. Hope tests pass
```

#### Run Tests Frequently

```bash
# After each logical change:
cargo test --lib

# Before committing:
cargo test --all
make lint
```

#### Keep Changes Focused

```
User: "Refactor builder module"

Do:
- Only refactor builder module
- Don't fix unrelated linting warnings
- Don't add new features "while you're there"
- Don't update dependencies

Unless:
- User specifically asks for it
- It's required for the refactor to work
```

### When Stuck

#### Check Design Documents

```bash
# List all design docs
./bin/oxd list

# Search for relevant topic
./bin/oxd list | grep -i "builder"

# Read the doc
./bin/oxd show 0004
```

#### Look for Similar Patterns

**Search codebase:**

```bash
# Find similar function names
rg "build_.*_item" --type rust

# Find similar patterns
rg "validate_node" --type rust -A 3
```

#### Ask Clarifying Questions

**Good questions:**

```
- "Should this error case return a specific error variant, or can I use a generic one?"
- "I see two patterns for this in the codebase. Which one should I follow?"
- "The design doc mentions X, but the code does Y. Which is correct?"
```

**Avoid:**

- Making assumptions about requirements
- Guessing at error handling strategy
- Inventing new patterns without asking

### Code Review Mindset

**Before suggesting changes, verify:**

1. **Against specs:**
   - Does this match the design doc?
   - Is this the intended behavior?

2. **Error handling:**
   - Are all error cases handled?
   - Are error messages helpful?
   - Are errors propagated correctly?

3. **Test coverage:**
   - Is this code path tested?
   - Are error cases tested?
   - Are edge cases covered?

4. **Documentation:**
   - Are public items documented?
   - Are complex algorithms explained?
   - Is the purpose clear?

5. **Edge cases:**
   - Empty input?
   - Maximum values?
   - Null/None cases?
   - Concurrent access (if applicable)?

---

## Resources & References

### Project Documentation

**Essential reading:**

- **Main README:** `/Users/oubiwann/lab/oxur/oxur/README.md`
  - Project overview and getting started
  - Architecture summary
  - Build instructions

- **Design Docs Index:** `crates/design/docs/index.md`
  - Complete list of design documents
  - Organized by state

- **Crate READMEs:**
  - `crates/oxur-ast/README.md` - AST library and aster CLI
  - `crates/design/README.md` - Design docs and oxd CLI
  - `crates/oxur-cli/README.md` - CLI infrastructure
  - `crates/oxur-lang/README.md` - Language design
  - `crates/oxur-comp/README.md` - Compiler design
  - `crates/oxur-repl/README.md` - REPL design

### External Resources

**Rust fundamentals:**

- [The Rust Book](https://doc.rust-lang.org/book/) - Complete Rust guide
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) - Learn by examples
- [Rust Reference](https://doc.rust-lang.org/reference/) - Language reference

**Key dependencies:**

- [syn docs](https://docs.rs/syn/) - Rust AST parsing
- [quote docs](https://docs.rs/quote/) - Rust AST generation
- [clap docs](https://docs.rs/clap/) - CLI argument parsing
- [thiserror docs](https://docs.rs/thiserror/) - Error handling
- [proptest docs](https://docs.rs/proptest/) - Property testing
- [criterion docs](https://docs.rs/criterion/) - Benchmarking

**Testing:**

- [Rust testing guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) - Coverage tool

### Project-Specific Resources

**AI Assistant Documentation:**

- **This file:** `CLAUDE.md` - General development guide
- **Session bootstrap:** `assets/ai/OXUR-SESSION-BOOTSTRAP.md` - New session template
- **Coverage guide:** `assets/ai/CLAUDE-CODE-COVERAGE.md` - Comprehensive testing guide

**Key Design Documents:**

- **ODD-0001:** Letter of Intent (vision and philosophy)
- **ODD-0003:** Canonical S-Expression Format specification
- **ODD-0013:** Compilation Chain Architecture
- **ODD-0004 to 0007:** oxur-ast implementation phases
- **ODD-0020:** Pattern & Type System Coverage

**Find design docs:**

```bash
./bin/oxd list              # All docs
./bin/oxd show 0003         # Read specific doc
./bin/oxd list --state active  # Only active docs
```

### Quick Command Reference

**Building:**

```bash
cargo build                    # Debug build
cargo build --release          # Optimized build
make build                     # Build all binaries
```

**Testing:**

```bash
cargo test                     # All tests
cargo test --lib               # Library tests only
cargo test --package oxur-ast  # Specific crate
make test                      # Run tests via Makefile
```

**Coverage:**

```bash
cargo llvm-cov --html                    # Generate HTML report
cargo llvm-cov --summary-only            # Quick summary
make coverage                            # Via Makefile
open target/llvm-cov/html/index.html     # View report
```

**Linting & Formatting:**

```bash
cargo clippy                   # Run clippy
cargo fmt                      # Format code
make lint                      # Check linting and format
make format                    # Format all code
```

**Design Docs:**

```bash
./bin/oxd list                 # List all docs
./bin/oxd show 0003            # Show specific doc
./bin/oxd new "Title"          # Create new doc
./bin/oxd transition 5 active  # Change state
```

**AST Tools:**

```bash
./bin/aster to-ast -i file.rs -o file.sexp   # Rust → S-expr
./bin/aster to-rust -i file.sexp -o file.rs  # S-expr → Rust
./bin/aster verify file.rs                   # Round-trip test
```

---

## Quick Reference Checklists

### Before Starting Work

- [ ] Read relevant design docs (`./bin/oxd show <number>`)
- [ ] Understand existing code patterns (read related files)
- [ ] Check test coverage of related code (`cargo llvm-cov`)
- [ ] Identify similar existing implementations (`rg pattern`)
- [ ] Understand the "why" behind the task
- [ ] Ask clarifying questions if anything is unclear

### Before Submitting Changes

- [ ] All tests pass (`cargo test --all`)
- [ ] Coverage ≥ 95% (`cargo llvm-cov --summary-only`)
- [ ] Linting passes (`make lint`)
- [ ] Code formatted (`make format`)
- [ ] No compiler warnings (`cargo build --all`)
- [ ] Documentation updated (doc comments on public items)
- [ ] Design docs updated (if architectural changes)
- [ ] README updated (if public API changed)
- [ ] Commit message is clear and descriptive
- [ ] Changes are focused on the requested task

### When Adding a New Module

- [ ] Module documented with `//!` comments at top
- [ ] Public API has `///` doc comments with examples
- [ ] Tests colocated in `#[cfg(test)] mod tests`
- [ ] Integration tests added if cross-module
- [ ] Examples added if public crate
- [ ] README updated if standalone crate
- [ ] Error types defined with `thiserror`
- [ ] `Result<T>` type alias created
- [ ] Module exposed in parent `mod.rs` or `lib.rs`
- [ ] Re-exports added if commonly used

### When Working with Errors

- [ ] Using `thiserror` for custom error types
- [ ] Each module has its own error enum
- [ ] Position tracking included where relevant (line, column)
- [ ] Error messages are helpful and specific
- [ ] All error cases tested (100% error path coverage)
- [ ] Error propagation is correct (`?` operator or explicit handling)
- [ ] `#[from]` used for automatic conversions
- [ ] Error Display impl is user-friendly

### When Writing Tests

- [ ] Test naming follows `test_<function>_<scenario>_<expectation>`
- [ ] Happy path tested
- [ ] Error paths tested (all error variants)
- [ ] Edge cases tested (empty, null, boundary values)
- [ ] Test data uses project helpers (`parse_example`, etc.)
- [ ] Assertions are specific, not vague
- [ ] Test is isolated (no external dependencies)
- [ ] Test is deterministic (same input always same output)
- [ ] Round-trip tests for conversions
- [ ] Coverage verified after adding tests

### When Reviewing Code

- [ ] Code follows project patterns
- [ ] Error handling is correct and tested
- [ ] Test coverage is adequate (≥95%)
- [ ] Documentation is clear and complete
- [ ] No over-engineering or unnecessary abstractions
- [ ] Naming is consistent with project conventions
- [ ] Edge cases are handled
- [ ] No TODO comments without tracking
- [ ] No commented-out code
- [ ] Performance is reasonable (no obvious inefficiencies)

### When Refactoring

- [ ] Comprehensive test coverage exists before starting
- [ ] Changes are incremental (one logical change at a time)
- [ ] Tests run after each change
- [ ] No behavioral changes (unless explicitly requested)
- [ ] Design docs updated if architecture changed
- [ ] All tests still pass
- [ ] Coverage did not decrease
- [ ] Code is simpler/clearer than before
- [ ] Commit history is clean

---

## Appendix: Common Patterns

### Error Handling Pattern

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModuleError {
    #[error("Invalid input: {message} at {pos}")]
    InvalidInput { message: String, pos: Position },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ModuleError>;
```

### Builder Pattern

```rust
pub struct Builder {
    // config/state
}

impl Builder {
    pub fn new() -> Self {
        Self { /* defaults */ }
    }

    pub fn build_thing(&self, input: &Input) -> Result<Thing> {
        self.validate(input)?;
        let part1 = self.build_part1(input)?;
        let part2 = self.build_part2(input)?;
        Ok(Thing { part1, part2 })
    }

    fn validate(&self, input: &Input) -> Result<()> {
        // validation
    }
}
```

### CLI Pattern

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Action {
        #[arg(short, long)]
        param: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Action { param } => execute_action(param),
    }
}
```

### Test Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> TestContext {
        // common setup
    }

    #[test]
    fn test_feature_happy_path() {
        let ctx = setup();
        let result = function_under_test(&ctx);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_feature_error_case() {
        let ctx = setup();
        let result = function_under_test_invalid(&ctx);
        assert!(matches!(result, Err(Error::Specific(_))));
    }
}
```

---

**Document End**

For questions or updates to this guide, see the repository maintainers.

**Last Updated:** 2025-12-31
**Version:** 1.0
