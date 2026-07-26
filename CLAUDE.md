# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Oxur is a **Lisp dialect that treats Rust as its compilation target and runtime**. It compiles through a 6-stage pipeline: Parse (lexer/reader) → Expand (macros/desugar) → Lower (Lisp→Rust concepts) → De-S-expression (S-expr→syn AST) → Generate (pretty-print Rust) → Compile (rustc/LLVM).

**Core principles:** 100% Rust interop, Rust semantics with Lisp syntax, canonical S-expressions, round-trip preservation, type-first design, TDD with 95%+ coverage, all architecture documented as ODDs.

## Rust Skill Guidelines

**Before writing, refactoring, or reviewing Rust code**, load the Rust programming skill:

- **Skill:** `assets/ai/ai-rust/skills/claude/SKILL.md`
- **Guides:** `assets/ai/ai-rust/guides/*.md` (anti-patterns, idioms, API design, etc.)
- **Always start with:** `11-anti-patterns.md`, then `01-core-idioms.md`, then topic-specific guides

If `assets/ai/ai-rust` doesn't exist (it may be a symlink — try with trailing slash), ask permission to clone:
```bash
git clone https://github.com/oxur/ai-rust assets/ai/ai-rust
```

Additional AI docs: `assets/ai/CLAUDE-CODE-COVERAGE.md` (testing guide), `assets/ai/OXUR-SESSION-BOOTSTRAP.md` (session template).

## Workspace Structure

Cargo workspace with 12 crates (resolver v2, edition 2021, workspace version 0.2.1):

### Compilation Pipeline
| Crate | Binary | Purpose | Status |
|-------|--------|---------|--------|
| **oxur-lang** | — | Frontend: parser, macro expander, Core Forms IR | Early stage |
| **oxur-comp** | `oxurc` | Backend: lowers Core Forms to Rust, generates binaries | Early stage |
| **oxur-ast** | `aster` | Bidirectional Rust AST ↔ S-expression conversion (syn-based) | ~95% complete |

### Foundation & Tooling
| Crate | Binary | Purpose | Status |
|-------|--------|---------|--------|
| **oxur-smap** | — | Source mapping across compilation stages (zero deps) | Active |
| **oxur-pretty** | `oxurfmt` | Configurable S-expression pretty-printer | Active |
| **oxur-testing** | — | Shared testing infrastructure and utilities | Active |
| **oxur-cli** | `oxur` | Unified CLI with common I/O, colored output, progress, tables | Active |
| **oxur-repl** | `oxur-repl-subprocess` | REPL: protocol, client, server with tiered execution | Early stage |
| **cargo-oxur** | `cargo-oxur` | Cargo subcommand for building Oxur projects | Early stage |
| **oxur** | — | Umbrella crate re-exporting oxur-lang, oxur-comp, oxur-ast, etc. | Scaffold |
| **design** | — | Documentation-only crate housing ODDs | Active |

### Publish order (dependency resolution)
`oxur-smap → oxur-testing → oxur-lang → oxur-comp → oxur-repl → oxur-cli → oxur-ast → oxur-pretty → cargo-oxur → oxur`

## Build & Development Commands

```bash
# Building
make build                          # Build all binaries (output in ./bin/)
make build-release                  # Optimized release build
cargo check --all                   # Type-check only

# Testing
cargo test                          # All tests
cargo test --package oxur-ast       # Single crate
cargo test --lib                    # Library tests only
make test                           # All tests via Makefile

# Coverage (target: 95%+)
make coverage                       # Summary report
make coverage-html                  # HTML report
cargo llvm-cov --html               # Direct HTML generation

# Linting & Formatting (max_width = 100)
make lint                           # Clippy + rustfmt check
make format                         # Format all code

# Combined checks
make check                          # Build + lint + test
make check-all                      # Build + lint + coverage

# Design docs (odm now lives in its own repo: github.com/oxur/odm)
# Install it with: cargo install oxur-odm
odm list                            # List all ODDs
odm show 0003                       # Show specific doc
odm new "Title"                     # Create new doc

# AST tools
./bin/aster to-ast -i file.rs -o file.sexp
./bin/aster to-rust -i file.sexp -o file.rs
./bin/aster verify file.rs          # Round-trip test

# Git
make push                           # Push to both Codeberg and Github
```

## Oxur-Specific Patterns

### Crate Naming
Format: `oxur-component` (hyphenated). Examples: `oxur-ast`, `oxur-lang`, `oxur-comp`.

### Error Handling with Position Tracking
All parse/build errors must include source position:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub offset: usize,  // Byte offset
    pub line: usize,    // 1-based
    pub column: usize,  // 1-based
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected token {token:?} at {pos}")]
    UnexpectedToken { token: String, pos: Position },
}
```

### CLI Output (oxur-cli)
All Oxur CLIs (aster, oxur) use shared colored output:
```rust
use oxur_cli::common::output::{success, error, info, warning};
```

### S-Expression Format (ODD-0003)
Canonical format for Rust AST representation:
```lisp
(Item
  :vis Public
  :ident (Ident :name "Point" :span (Span :lo 0 :hi 5))
  :kind (Struct
    :fields (Fields
      :named [(Field :vis Public :ident (Ident :name "x") :ty (Type :path "i32"))])))
```

### Builder Pattern (oxur-ast)
AST builders organize methods: public API at top → item builders → expression builders → helpers at bottom.

### Test Data Organization (oxur-ast)
```
test-data/
├── examples/{simple,intermediate,complex}/   # .rs and .sexp pairs
└── fixtures/{crate,item,expr,error-cases}/   # By AST node type
```

Use `CARGO_MANIFEST_DIR` for test data paths:
```rust
let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/examples").join(name);
```

### Round-Trip Testing (critical for oxur-ast)
```rust
#[test]
fn test_round_trip_struct() {
    let ast1 = parse_rust(src).unwrap();
    let sexp = generate_sexp(&ast1).unwrap();
    let ast2 = build_ast(&sexp).unwrap();
    assert_ast_equivalent(&ast1, &ast2);
}
```

## Testing Conventions

- **Coverage target:** 95%+ overall, 90%+ per module, 100% error paths and public API
- **Test naming:** `test_<function>_<scenario>_<expectation>` (e.g., `test_build_item_missing_kind_returns_error`)
- **Property testing:** `proptest` for invariant testing across random inputs
- **Benchmarks:** `criterion` in `benches/` directories, run with `cargo bench`
- **Detailed guide:** `assets/ai/CLAUDE-CODE-COVERAGE.md`

## Design Documentation (ODDs)

**Location:** `crates/design/docs/` organized by state:
`01-draft/ → 02-under-review/ → 03-revised/ → 04-accepted/ → 05-active/ → 06-final/` (also: `07-deferred/`, `08-rejected/`, `09-withdrawn/`, `10-superseded/`)

**Key documents:**
- **0001:** Oxur Letter of Intent — vision and philosophy
- **0003:** Canonical S-Expression Format — Rust AST ↔ S-expr spec
- **0013:** Compilation Chain Architecture — 6-stage pipeline
- **0004-0007:** oxur-ast Phase Documents — AST library implementation

**YAML frontmatter** required: `number`, `title`, `author`, `created`, `updated`, `state`.

**In code**, reference ODDs: `// See ODD-0003 section 3.2 for Item format specification`

## Git Conventions

- **Commit style:** Free-form descriptive, imperative mood, explain WHY not WHAT
- **Before submitting:** `cargo test --all` + `make lint` + `make format` + `make coverage` (95%+)
- **Design docs:** Reference ODD numbers in commits when relevant

## Before Starting Work

1. Check relevant design docs (`odm list`, `odm show <number>`; install via `cargo install oxur-odm`)
2. Load Rust skill guides (always `11-anti-patterns.md` first)
3. Read existing code in the target module
4. Understand existing test coverage
