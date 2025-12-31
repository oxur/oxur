# AI Assistant Guide for Oxur Development

**Version:** 2.0
**Last Updated:** 2025-12-31
**Purpose:** Comprehensive guidelines for AI assistants working with the Oxur Rust project

## About This Document

This document provides essential guidance for AI assistants (like Claude Code) when working with the Oxur codebase. It focuses on **Oxur-specific** conventions, patterns, and workflows, while deferring to authoritative Rust guidelines for general best practices.

### Document Hierarchy

**For Rust Code Quality:**
1. **`assets/ai/ai-rust/skills/claude/SKILL.md`** - Advanced Rust programming skill (**use this**)
2. **`assets/ai/ai-rust/guides/*.md`** - Comprehensive Rust guidelines referenced by the skill
3. **This file (CLAUDE.md)** - Oxur-specific conventions only

**For Oxur-Specific Topics:**
- **This file (CLAUDE.md)** - Project structure, ODDs, workflows, Oxur patterns
- **`assets/ai/OXUR-SESSION-BOOTSTRAP.md`** - Template for starting new sessions with context
- **`assets/ai/CLAUDE-CODE-COVERAGE.md`** - Comprehensive test coverage guide

**Important:** If `assets/ai/ai-rust` does not exist on the file system, ask permission to clone it:
```bash
git clone https://github.com/oxur/ai-rust assets/ai/ai-rust
```

### Quick Navigation

- [Project Overview](#project-overview)
- [Development Environment](#development-environment--tools)
- [Oxur-Specific Patterns](#oxur-specific-patterns)
- [Rust Best Practices](#rust-best-practices-using-the-skill)
- [Testing](#testing-requirements)
- [Design Documentation](#design-documentation-integration)
- [Common Workflows](#common-workflows)
- [Git Conventions](#git--commit-conventions)
- [AI Assistant Guidelines](#ai-assistant-guidelines)
- [Resources](#resources--references)
- [Quick Reference](#quick-reference-checklists)

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
│  Oxur Source (Lisp)                                            │
│         ↓                                                       │
│  ┌──────────────────┐                                          │
│  │  oxur-lang       │  Stage 1: Parse → Surface Forms          │
│  │  (Lisp Compiler) │  Stage 2: Expand → Core Forms (IR)       │
│  └──────────────────┘                                          │
│         ↓                                                       │
│  Core Forms (Canonical S-expressions)                          │
│         ↓                                                       │
│  ┌──────────────────┐                                          │
│  │  oxur-comp       │  Stage 3: Lower → Rust AST               │
│  │  (Backend)       │  Stage 4: Codegen → Rust Source          │
│  └──────────────────┘  Stage 5: Compile → Binary (via rustc)   │
│         ↓                                                       │
│  Rust Binary                                                    │
│                                                                 │
│  ┌──────────────────┐                                          │
│  │  oxur-ast        │  Supporting: Bidirectional Rust AST ↔    │
│  │  (AST Library)   │  S-expression conversion                 │
│  └──────────────────┘                                          │
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
        ├── ai-rust/     # Rust guidelines (symlink to external repo)
        ├── CLAUDE-CODE-COVERAGE.md
        └── OXUR-SESSION-BOOTSTRAP.md
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

#### oxur-lang, oxur-comp, oxur-repl (Planning)
**Status:** Design phase, not yet implemented

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

## Oxur-Specific Patterns

This section covers patterns that are **specific to the Oxur project**. For general Rust best practices, see the [Rust Best Practices](#rust-best-practices-using-the-skill) section.

### Naming Conventions (Oxur-Specific)

**Crates:**
- Format: `oxur-component` (hyphenated)
- Examples: `oxur-ast`, `oxur-lang`, `oxur-comp`
- NOT: `rast`, `oxur_ast`, `oxurAST`

### Error Handling with Position Tracking

**Oxur Pattern:** All parse/build errors include source position information

```rust
use thiserror::Error;

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

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected token {token:?} at {pos}")]
    UnexpectedToken { token: String, pos: Position },

    #[error("Empty input at {pos}")]
    EmptyInput { pos: Position },
}

pub type Result<T> = std::result::Result<T, ParseError>;
```

**Usage:**
```rust
return Err(ParseError::UnexpectedToken {
    token: token.to_string(),
    pos: Position { offset: 42, line: 3, column: 15 },
});
```

**Why:** Helps users locate errors in their Oxur source code.

### Test Data Organization (oxur-ast Pattern)

**File-based test data structure:**

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

### Builder Pattern for AST Construction

**Oxur-specific pattern for building Rust AST from S-expressions:**

```rust
pub struct Builder {
    // internal state
}

impl Builder {
    pub fn new() -> Self {
        Self { /* defaults */ }
    }

    // Public API - main entry points
    pub fn build_crate(&self, sexp: &SExp) -> Result<Crate> { }
    pub fn build_item(&self, sexp: &SExp) -> Result<Item> { }
    pub fn build_expr(&self, sexp: &SExp) -> Result<Expr> { }

    // Private builders - organized by AST type
    fn build_item_kind(&self, sexp: &SExp) -> Result<ItemKind> { }
    fn build_expr_kind(&self, sexp: &SExp) -> Result<ExprKind> { }

    // Validation helpers
    fn validate_node(&self, sexp: &SExp) -> Result<()> { }
    fn extract_field<T>(&self, sexp: &SExp, field: &str) -> Result<T> { }
}
```

**Method organization strategy:**
- Public API at top
- Item builders grouped together
- Expression builders grouped together
- Helper methods at bottom

### CLI Output Standards (oxur-cli)

**Using colored output wrapper functions:**

```rust
use oxur_cli::common::output::{success, error, info, warning};

success("Operation completed!");     // Green
error("Something went wrong");       // Red
info("Processing files...");         // Cyan
warning("Deprecated API usage");     // Yellow
```

**Consistency across all Oxur CLIs (aster, oxd, oxur).**

### S-Expression Format (ODD-0003)

**Canonical format for Rust AST representation:**

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
      :named [(Field :vis Public :ident (Ident :name "x") :ty (Type :path "i32"))
              (Field :vis Public :ident (Ident :name "y") :ty (Type :path "i32"))])
    :generics (Generics :params [])
    :span (Span :lo 0 :hi 30)))
```

**See design doc 0003 for complete specification.**

---

## Rust Best Practices (Using the Skill)

**For all Rust best practices, use the advanced Rust programming skill:**

📖 **Skill Definition:** `assets/ai/ai-rust/skills/claude/SKILL.md`

### When to Load the Skill

Load the Rust guidelines skill when:
- Writing new Rust code
- Refactoring existing Rust code
- Reviewing Rust code for issues
- Debugging borrow checker or lifetime errors
- Designing public APIs
- Choosing between Rust patterns

### Skill Workflow Summary

**For writing new code:**
1. Load `assets/ai/ai-rust/guides/11-anti-patterns.md` (what to avoid)
2. Load `assets/ai/ai-rust/guides/01-core-idioms.md` (standard patterns)
3. Load topic-specific guides as needed
4. Write code following guidelines
5. Self-review against anti-patterns

**For refactoring:**
1. Load `11-anti-patterns.md`
2. Scan code for violations (note pattern IDs like AP-08)
3. Load relevant guides for fixes
4. Refactor systematically

**For code review:**
1. Load `11-anti-patterns.md`
2. Check each pattern (AP-01 through AP-20)
3. Load topic guides based on code content
4. Report findings using pattern IDs

### Guide Selection Reference

| Task | Load These Guides |
|------|-------------------|
| **Any Rust code** | `11-anti-patterns.md` (always first) |
| **New code** | `01-core-idioms.md`, `11-anti-patterns.md` |
| **API design** | `02-api-design.md`, `05-type-design.md`, `06-traits.md` |
| **Error handling** | `03-error-handling.md` |
| **Ownership/lifetimes** | `04-ownership-borrowing.md` |
| **Async code** | `07-concurrency-async.md` |
| **Performance** | `08-performance.md` |
| **FFI/unsafe** | `09-unsafe-ffi.md` |
| **Macros** | `10-macros.md` |
| **Project structure** | `12-project-structure.md` |
| **Documentation** | `13-documentation.md` |

### Critical Quick Reference

These rules from the skill should be followed in ALL Rust code:

**Parameters:**
```rust
// ❌ AVOID
fn process(data: &String, items: &Vec<i32>)

// ✅ PREFER
fn process(data: &str, items: &[i32])
```

**Derives:**
```rust
// ✅ Most types should have:
#[derive(Debug, Clone, PartialEq)]
struct MyType { /* ... */ }
```

**Error Handling:**
```rust
// ❌ AVOID in library code
let value = something.unwrap();

// ✅ PREFER
let value = something?;
```

**See the skill guides for comprehensive coverage of all patterns.**

---

## Testing Requirements

### Coverage Targets

**Minimum requirements:**
- **Overall coverage:** ≥ 95%
- **Module coverage:** ≥ 90% (no stragglers)
- **Error paths:** 100% tested
- **Public API:** 100% tested

**For comprehensive testing guidance:**
- See `assets/ai/CLAUDE-CODE-COVERAGE.md` for systematic testing approach
- See `assets/ai/ai-rust/guides/01-core-idioms.md` for Rust testing patterns

### Test Naming (Oxur Convention)

**Format:** `test_<function>_<scenario>_<expectation>`

**Examples:**
```rust
#[test]
fn test_build_item_struct_succeeds() { }

#[test]
fn test_build_item_missing_kind_returns_error() { }

#[test]
fn test_parse_empty_file_returns_empty_vec() { }
```

### Round-Trip Testing (Critical for oxur-ast)

**Pattern:**
```rust
#[test]
fn test_round_trip_struct() {
    // Rust → AST → S-expr → AST → Rust should equal original
    let original_rust = "struct Point { x: i32, y: i32 }";

    let ast1 = parse_rust(original_rust).unwrap();
    let sexp = generate_sexp(&ast1).unwrap();
    let ast2 = build_ast(&sexp).unwrap();
    let generated_rust = generate_rust(&ast2).unwrap();

    assert_ast_equivalent(&ast1, &ast2);
}
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

### Key Design Documents

**Essential reading:**

1. **0001: Oxur Letter of Intent** (`05-active/`)
   - Overall vision and philosophy

2. **0003: Canonical S-Expression Format** (`05-active/`)
   - Specification for Rust AST ↔ S-expr format

3. **0013: Compilation Chain Architecture** (`05-active/`)
   - Five-stage compilation pipeline

4. **0004-0007: oxur-ast Phase Documents** (`06-final/`)
   - Implementation guides for AST library

### Using Design Docs

**Before implementing a feature:**
1. Check if design doc exists: `./bin/oxd list`
2. Read relevant docs: `./bin/oxd show 0003`
3. If no doc exists and feature is non-trivial, create one

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

## Common Workflows

### Adding a New Feature

1. **Check for design doc:** `./bin/oxd list | grep -i "feature"`
2. **Read relevant docs:** `./bin/oxd show <number>`
3. **Load Rust skill guides:** Based on what you're building (see skill workflow)
4. **Write tests first (TDD)**
5. **Implement feature following guidelines**
6. **Run tests:** `cargo test`
7. **Check coverage:** `make coverage` (ensure ≥ 95%)
8. **Run linting:** `make lint` and `make format`
9. **Update documentation**

### Refactoring Code

1. **Ensure comprehensive test coverage** (verify with `cargo llvm-cov`)
2. **Load Rust anti-patterns guide** (`11-anti-patterns.md`)
3. **Scan for violations** (note pattern IDs)
4. **Make incremental changes** (one pattern at a time)
5. **Run tests after each change**
6. **Verify no behavioral changes**
7. **Update design docs if architecture changed**

### Fixing Bugs

1. **Write test that reproduces bug**
2. **Understand root cause** (don't apply band-aids)
3. **Load relevant Rust guides** if needed
4. **Fix the bug**
5. **Ensure test passes**
6. **Add regression test**
7. **Check for similar bugs elsewhere**

### Working with AST (aster CLI)

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
```

---

## Git & Commit Conventions

### Commit Messages

**Format:**
- Free-form descriptive (preferred for Oxur)
- Subject line: Imperative mood ("Add feature" not "Added feature")
- Body: Explain WHY, not WHAT (code shows what)
- Reference design docs when relevant

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

### Pull Request Guidelines

**Before creating PR:**
- [ ] All tests pass (`cargo test --all`)
- [ ] Coverage ≥ 95% (`make coverage`)
- [ ] Linting passes (`make lint`)
- [ ] Code formatted (`make format`)
- [ ] No warnings in build
- [ ] Design docs updated (if architectural changes)
- [ ] Rust anti-patterns checked (`11-anti-patterns.md`)

---

## AI Assistant Guidelines

### General Approach

**For Rust code quality:**
1. **Always load the Rust skill first** (`assets/ai/ai-rust/skills/claude/SKILL.md`)
2. **Follow the skill workflow** for writing, refactoring, or reviewing
3. **Reference pattern IDs** in discussions (e.g., "This violates AP-08")

**For Oxur-specific work:**
1. **Read this CLAUDE.md** for project structure and conventions
2. **Check design docs** for architectural decisions
3. **Follow Oxur-specific patterns** (Position tracking, test data, etc.)

### Workflow: Writing New Code

1. **Read existing code** in the relevant module
2. **Load Rust guides:**
   - `11-anti-patterns.md` (always)
   - `01-core-idioms.md` (always)
   - Topic-specific guides as needed
3. **Check design docs** for specifications
4. **Write code** following both Rust guidelines and Oxur conventions
5. **Self-review** against anti-patterns
6. **Test thoroughly** (95%+ coverage)

### Workflow: Refactoring

1. **Ensure tests exist** (check coverage)
2. **Load `11-anti-patterns.md`**
3. **Scan for violations** in existing code
4. **Load relevant topic guides** for fixes
5. **Make incremental changes**
6. **Run tests after each change**
7. **Document pattern IDs** in commit messages

### Workflow: Code Review

1. **Load `11-anti-patterns.md`**
2. **Check each pattern** (AP-01 through AP-20)
3. **Load topic guides** based on code content
4. **Check Oxur conventions** (Position tracking, naming, etc.)
5. **Verify test coverage** (≥95%)
6. **Report findings** using pattern IDs

### When Stuck

**For Rust issues:**
- Check the relevant guide in `assets/ai/ai-rust/guides/`
- Look for pattern IDs mentioned in error messages
- Search for similar patterns in the codebase

**For Oxur issues:**
- Check design docs: `./bin/oxd list` and `./bin/oxd show <number>`
- Look for similar implementations in the codebase
- Ask clarifying questions

### Code Review Mindset

**Verify:**
1. **Rust guidelines compliance** (using pattern IDs)
2. **Oxur conventions** (Position tracking, naming, etc.)
3. **Test coverage** (≥95%)
4. **Design doc alignment** (architectural decisions)
5. **Error handling** (all paths tested)
6. **Documentation** (public API has doc comments)

---

## Resources & References

### Oxur Project Documentation

**Essential reading:**
- **Main README:** `/Users/oubiwann/lab/oxur/oxur/README.md`
- **Design Docs Index:** `crates/design/docs/index.md`
- **Crate READMEs:** `crates/*/README.md`

### Rust Guidelines (External)

**AI-Optimized Rust Guidelines:**
- **Skill Definition:** `assets/ai/ai-rust/skills/claude/SKILL.md`
- **Guides Directory:** `assets/ai/ai-rust/guides/`
  - `01-core-idioms.md` - Essential patterns
  - `02-api-design.md` - Public API design
  - `03-error-handling.md` - Result, Option, error types
  - `04-ownership-borrowing.md` - Lifetimes, borrow checker
  - `05-type-design.md` - Structs, enums, newtypes
  - `06-traits.md` - Trait design patterns
  - `07-concurrency-async.md` - Async, threading
  - `08-performance.md` - Optimization
  - `09-unsafe-ffi.md` - Unsafe code, FFI
  - `10-macros.md` - Macro patterns
  - `11-anti-patterns.md` - **What NOT to do** (critical!)
  - `12-project-structure.md` - Crate organization
  - `13-documentation.md` - Doc comments

### AI Assistant Documentation

- **This file:** `CLAUDE.md` - Oxur-specific guide
- **Session bootstrap:** `assets/ai/OXUR-SESSION-BOOTSTRAP.md`
- **Coverage guide:** `assets/ai/CLAUDE-CODE-COVERAGE.md`
- **Rust skill:** `assets/ai/ai-rust/skills/claude/SKILL.md`

### External Resources

**Rust fundamentals:**
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rust Reference](https://doc.rust-lang.org/reference/)

**Key dependencies:**
- [syn docs](https://docs.rs/syn/) - Rust AST parsing
- [clap docs](https://docs.rs/clap/) - CLI argument parsing
- [thiserror docs](https://docs.rs/thiserror/) - Error handling

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
make test                      # Run tests via Makefile
```

**Coverage:**
```bash
cargo llvm-cov --html          # Generate HTML report
make coverage                  # Via Makefile
```

**Linting & Formatting:**
```bash
make lint                      # Check linting and format
make format                    # Format all code
```

**Design Docs:**
```bash
./bin/oxd list                 # List all docs
./bin/oxd show 0003            # Show specific doc
./bin/oxd new "Title"          # Create new doc
```

**AST Tools:**
```bash
./bin/aster to-ast -i file.rs -o file.sexp    # Rust → S-expr
./bin/aster to-rust -i file.sexp -o file.rs   # S-expr → Rust
./bin/aster verify file.rs                    # Round-trip test
```

---

## Quick Reference Checklists

### Before Starting Work

- [ ] Read relevant design docs (`./bin/oxd show <number>`)
- [ ] Load Rust anti-patterns guide (`11-anti-patterns.md`)
- [ ] Load relevant Rust topic guides
- [ ] Understand existing code patterns (read related files)
- [ ] Check test coverage of related code
- [ ] Understand the "why" behind the task

### Before Submitting Changes

- [ ] All tests pass (`cargo test --all`)
- [ ] Coverage ≥ 95% (`cargo llvm-cov --summary-only`)
- [ ] Linting passes (`make lint`)
- [ ] Code formatted (`make format`)
- [ ] No compiler warnings
- [ ] Checked against Rust anti-patterns (`11-anti-patterns.md`)
- [ ] Documentation updated (doc comments on public items)
- [ ] Design docs updated (if architectural changes)
- [ ] Commit message is clear and references design docs if relevant

### When Writing Rust Code

- [ ] Loaded `11-anti-patterns.md` first
- [ ] Loaded `01-core-idioms.md` for standard patterns
- [ ] Loaded topic-specific guides as needed
- [ ] Followed established patterns in the crate
- [ ] Added Position tracking to errors (if parse/build code)
- [ ] Used project error handling patterns
- [ ] Checked against AP-01 through AP-20
- [ ] Self-reviewed before submitting

### When Testing

- [ ] Followed test naming convention: `test_<fn>_<scenario>_<expectation>`
- [ ] Used project test data helpers (`parse_example`, etc.)
- [ ] Tested happy path
- [ ] Tested all error paths
- [ ] Tested edge cases (empty, boundary values)
- [ ] Added round-trip tests (if conversion code)
- [ ] Verified coverage ≥ 95%
- [ ] See `CLAUDE-CODE-COVERAGE.md` for comprehensive approach

### When Refactoring

- [ ] Ensured tests exist before starting
- [ ] Loaded `11-anti-patterns.md`
- [ ] Identified violations with pattern IDs
- [ ] Made incremental changes
- [ ] Ran tests after each change
- [ ] Preserved existing behavior
- [ ] Updated design docs if needed
- [ ] Referenced pattern IDs in commits

### When Reviewing Code

- [ ] Loaded `11-anti-patterns.md`
- [ ] Checked each pattern (AP-01 to AP-20)
- [ ] Loaded topic guides for code content
- [ ] Verified Oxur conventions (Position, naming, etc.)
- [ ] Checked test coverage (≥95%)
- [ ] Verified design doc alignment
- [ ] Checked error handling
- [ ] Used pattern IDs in feedback

---

## Summary

**This document provides Oxur-specific guidance. For Rust best practices:**

📖 **Use the Rust Guidelines Skill:** `assets/ai/ai-rust/skills/claude/SKILL.md`

**Key takeaways:**
1. **Rust code quality** → Use the skill and guides
2. **Oxur conventions** → Use this document
3. **Testing** → Use CLAUDE-CODE-COVERAGE.md + Rust guides
4. **Architecture** → Check design docs
5. **Always** load anti-patterns guide first

**Document End**

**Last Updated:** 2025-12-31
**Version:** 2.0
