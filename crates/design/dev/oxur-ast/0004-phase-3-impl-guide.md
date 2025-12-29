# Phase 3 Implementation Guide: Integration, Testing & CLI

**Status:** Ready to implement
**Based on:** Design doc 0007
**Prerequisites:** Phases 0, 1, and 2 complete ✓

---

## Overview

Phase 3 transforms `oxur-ast` into a production-ready library with:
- Real Rust parsing via `syn`
- CLI tool called `aster`
- Comprehensive testing
- Performance benchmarks

**Key adaptations from design doc:**
- CLI name: `aster` (not `oxur-ast`)
- Commands: `to-ast`, `to-rust` (not `to-sexp`)
- Structure: `src/main.rs` + `src/commands/` (following `oxd` pattern)

---

## Code Reuse Tracking

Before implementing, note these patterns from `oxd` that may be reusable:

**From `crates/design`:**
- [ ] Cargo.toml structure (lib + bin)
- [ ] main.rs organization pattern
- [ ] cli.rs (clap Parser/Subcommand structure)
- [ ] commands/ module organization
- [ ] Error handling with colored output
- [ ] Table rendering with `oxur-table` crate
- [ ] Colored terminal output patterns

**Track these for deduplication report:**
- Command-line argument parsing patterns
- Error display formatting
- File I/O helpers
- Progress/status indicators

---

## Phase 3 Breakdown

Phase 3 is split into **4 chunks**:

### **Chunk 1: Dependencies & Integration Module** (Session 1)
- Update Cargo.toml
- Create `src/integration/` module
- Implement `SynConverter`
- Basic parsing tests

### **Chunk 2: CLI Tool Structure** (Session 2)
- Update Cargo.toml for binary
- Create `src/main.rs` and `src/cli.rs`
- Create `src/commands/` module
- Implement basic command structure

### **Chunk 3: CLI Commands & Tests** (Session 3)
- Implement `to-ast`, `to-rust`, `verify` commands
- Create test fixtures
- Write integration tests
- Write regression tests

### **Chunk 4: Polish & Performance** (Session 4)
- Create benchmark suite
- Write examples
- Update documentation
- Final verification

---

## Chunk 1: Dependencies & Integration Module

### Step 1.1: Update Dependencies

Edit `crates/oxur-ast/Cargo.toml`:

```toml
[package]
name = "oxur-ast"
# ... existing package fields ...

[dependencies]
thiserror.workspace = true
# NEW: Add syn for Rust parsing
syn = { version = "2.0", features = ["full", "parsing", "extra-traits"] }
quote = "1.0"

[dev-dependencies]
tempfile.workspace = true
# NEW: Add criterion for benchmarks
criterion = "0.5"

[[bench]]
name = "conversion_bench"
harness = false
```

### Step 1.2: Create Integration Module Structure

Create directory and files:
```bash
mkdir -p crates/oxur-ast/src/integration
touch crates/oxur-ast/src/integration/mod.rs
touch crates/oxur-ast/src/integration/from_syn.rs
```

### Step 1.3: Implement `src/integration/mod.rs`

```rust
//! Integration with Rust's standard AST (via syn)

mod from_syn;

pub use from_syn::*;

use crate::error::Result;
use crate::ast::Crate;

/// Parse Rust source code into our AST
pub fn parse_rust_file(source: &str) -> Result<Crate> {
    let syn_file = syn::parse_file(source)
        .map_err(|e| crate::error::ParseError::Expected {
            expected: "valid Rust code".to_string(),
            found: format!("parse error: {}", e),
            pos: crate::error::Position::new(0, 1, 1),
        })?;

    from_syn_file(&syn_file)
}
```

### Step 1.4: Implement `src/integration/from_syn.rs`

**Important:** Follow the design doc's `SynConverter` implementation exactly.
This is a large file - implement these methods in order:

1. `SynConverter` struct with `next_id()` tracking
2. `convert_file()` - main entry point
3. `convert_item()` - dispatch for item types
4. `convert_item_fn()` - function items
5. Helper converters:
   - `convert_ident()`
   - `convert_visibility()`
   - `convert_fn_sig()` and related
   - `convert_block()` and `convert_stmt()`
   - `convert_expr()` and variants
   - `convert_pat()`, `convert_ty()`, `convert_path()`
   - `convert_lit()`

**Key simplifications for Phase 3:**
- Use `Span::DUMMY` (proc-macro2 integration later)
- Empty attributes for now
- Skip complex generics
- Only basic expression types

See design doc lines 132-536 for complete implementation.

### Step 1.5: Update `src/lib.rs`

Add the integration module export:

```rust
pub mod integration;  // NEW

pub use integration::parse_rust_file;  // NEW
```

### Step 1.6: Basic Integration Tests

Create `tests/integration_basic.rs`:

```rust
use oxur_ast::integration::parse_rust_file;

#[test]
fn test_parse_hello_world() {
    let source = r#"
fn main() {
    println!("Hello, world!");
}
    "#;

    let crate_node = parse_rust_file(source).expect("Failed to parse");

    assert_eq!(crate_node.items.len(), 1);
    assert_eq!(crate_node.items[0].ident.name, "main");
}

#[test]
fn test_parse_simple_function() {
    let source = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
    "#;

    let crate_node = parse_rust_file(source).expect("Failed to parse");
    assert_eq!(crate_node.items.len(), 1);
}
```

### Step 1.7: Verify Chunk 1

```bash
cargo test -p oxur-ast integration_basic
cargo clippy -p oxur-ast -- -D warnings
```

**Chunk 1 Complete!** ✓

---

## Chunk 2: CLI Tool Structure

### Step 2.1: Update Cargo.toml for Binary

Add to `crates/oxur-ast/Cargo.toml`:

```toml
# Binary configuration (following oxd pattern)
[[bin]]
name = "aster"
path = "src/main.rs"

[dependencies]
# ... existing deps ...
clap.workspace = true
anyhow.workspace = true
colored.workspace = true  # For error messages
```

### Step 2.2: Create `src/cli.rs`

**Pattern from oxd:** Separate CLI definitions from main logic

```rust
//! CLI argument parsing

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "aster")]
#[command(about = "AST manipulation and conversion tool", long_about = None)]
#[command(after_help = "Use 'aster <command> --help' for more information.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Convert Rust source to S-expression AST
    #[command(visible_alias = "ast")]
    ToAst {
        /// Input Rust file (or - for stdin)
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file (or - for stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Use compact formatting
        #[arg(short, long)]
        compact: bool,
    },

    /// Convert S-expression to Rust source
    ToRust {
        /// Input S-expression file (or - for stdin)
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file (or - for stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Verify round-trip conversion
    Verify {
        /// Input Rust file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}
```

### Step 2.3: Create `src/commands/mod.rs`

**Pattern from oxd:** Separate command implementations

```rust
//! Command implementations

pub mod to_ast;
pub mod to_rust;
pub mod verify;

pub use to_ast::execute as to_ast;
pub use to_rust::execute as to_rust;
pub use verify::execute as verify;
```

### Step 2.4: Create `src/main.rs`

**Pattern from oxd:** Clean main with command dispatch

```rust
//! AST manipulation and conversion CLI tool

use anyhow::Result;
use clap::Parser;
use colored::*;

mod cli;
mod commands;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Err(e) = execute_command(cli.command) {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }

    Ok(())
}

fn execute_command(command: Commands) -> Result<()> {
    match command {
        Commands::ToAst { input, output, compact } => {
            commands::to_ast(input, output, compact)
        }
        Commands::ToRust { input, output } => {
            commands::to_rust(input, output)
        }
        Commands::Verify { input, verbose } => {
            commands::verify(input, verbose)
        }
    }
}
```

### Step 2.5: Create Command Stubs

Create placeholder files (implement in Chunk 3):

`src/commands/to_ast.rs`:
```rust
use anyhow::Result;
use std::path::PathBuf;

pub fn execute(_input: PathBuf, _output: Option<PathBuf>, _compact: bool) -> Result<()> {
    println!("to-ast command - to be implemented");
    Ok(())
}
```

`src/commands/to_rust.rs`:
```rust
use anyhow::Result;
use std::path::PathBuf;

pub fn execute(_input: PathBuf, _output: Option<PathBuf>) -> Result<()> {
    println!("to-rust command - to be implemented");
    Ok(())
}
```

`src/commands/verify.rs`:
```rust
use anyhow::Result;
use std::path::PathBuf;

pub fn execute(_input: PathBuf, _verbose: bool) -> Result<()> {
    println!("verify command - to be implemented");
    Ok(())
}
```

### Step 2.6: Verify Chunk 2

```bash
cargo build -p oxur-ast
cargo run -p oxur-ast -- --help
cargo run -p oxur-ast -- to-ast --help
```

**Chunk 2 Complete!** ✓

---

## Chunk 3: CLI Commands & Tests

### Step 3.1: Implement `to-ast` Command

Replace `src/commands/to_ast.rs` stub:

```rust
use anyhow::Result;
use colored::*;
use oxur_ast::integration::parse_rust_file;
use oxur_ast::sexp::{print_sexp, Printer};
use oxur_ast::Generator;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

pub fn execute(input: PathBuf, output: Option<PathBuf>, compact: bool) -> Result<()> {
    // Read input
    let source = if input.to_str() == Some("-") {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        fs::read_to_string(&input)?
    };

    // Parse Rust
    let crate_node = parse_rust_file(&source)?;

    // Generate S-expression
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node)?;

    // Format
    let output_text = if compact {
        Printer::new().print_compact(&sexp)
    } else {
        print_sexp(&sexp)
    };

    // Write output
    if let Some(output_path) = output {
        if output_path.to_str() == Some("-") {
            println!("{}", output_text);
        } else {
            fs::write(output_path, output_text)?;
        }
    } else {
        println!("{}", output_text);
    }

    Ok(())
}
```

### Step 3.2: Implement `to-rust` Command

Replace `src/commands/to_rust.rs` stub:

```rust
use anyhow::Result;
use colored::*;
use oxur_ast::sexp::Parser;
use oxur_ast::AstBuilder;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

pub fn execute(input: PathBuf, output: Option<PathBuf>) -> Result<()> {
    // Read input
    let sexp_text = if input.to_str() == Some("-") {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        fs::read_to_string(&input)?
    };

    // Parse S-expression
    let sexp = Parser::parse_str(&sexp_text)?;

    // Build AST
    let mut builder = AstBuilder::new();
    let crate_node = builder.build_crate(&sexp)?;

    // Generate Rust (Phase 3: simplified - just Debug output)
    let rust_output = format!("// Generated from S-expression\n// AST: {:#?}", crate_node);

    // Write output
    if let Some(output_path) = output {
        if output_path.to_str() == Some("-") {
            println!("{}", rust_output);
        } else {
            fs::write(output_path, rust_output)?;
        }
    } else {
        println!("{}", rust_output);
    }

    Ok(())
}
```

### Step 3.3: Implement `verify` Command

Replace `src/commands/verify.rs` stub:

```rust
use anyhow::Result;
use colored::*;
use oxur_ast::integration::parse_rust_file;
use oxur_ast::sexp::{print_sexp, Parser};
use oxur_ast::{AstBuilder, Generator};
use std::fs;
use std::path::PathBuf;

pub fn execute(input: PathBuf, verbose: bool) -> Result<()> {
    let source = fs::read_to_string(&input)?;

    println!("{} {}", "Verifying round-trip for:".bold(), input.display());
    if verbose {
        println!();
    }

    // Step 1: Parse Rust
    if verbose {
        println!("1. Parsing Rust source...");
    }
    let crate1 = parse_rust_file(&source)?;
    if verbose {
        println!("   {} Parsed successfully", "✓".green());
    }

    // Step 2: Generate S-expression
    if verbose {
        println!("2. Generating S-expression...");
    }
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate1)?;
    if verbose {
        println!("   {} Generated successfully", "✓".green());
    }

    // Step 3: Parse S-expression back
    if verbose {
        println!("3. Parsing S-expression...");
    }
    let sexp_text = print_sexp(&sexp);
    let sexp2 = Parser::parse_str(&sexp_text)?;
    if verbose {
        println!("   {} Parsed successfully", "✓".green());
    }

    // Step 4: Build AST
    if verbose {
        println!("4. Building AST from S-expression...");
    }
    let mut builder = AstBuilder::new();
    let crate2 = builder.build_crate(&sexp2)?;
    if verbose {
        println!("   {} Built successfully", "✓".green());
    }

    // Step 5: Verify
    if verbose {
        println!("5. Verifying equivalence...");
    }
    if crate1.items.len() != crate2.items.len() {
        anyhow::bail!("Item count mismatch: {} vs {}",
            crate1.items.len(), crate2.items.len());
    }
    if verbose {
        println!("   {} Basic verification passed", "✓".green());
    }

    if !verbose {
        println!("{} Round-trip verification successful!", "✓".green().bold());
    } else {
        println!();
        println!("{} Round-trip verification successful!", "✓".green().bold());
    }

    Ok(())
}
```

### Step 3.4: Create Test Fixtures

Create directory and files:
```bash
mkdir -p crates/oxur-ast/tests/fixtures
```

`tests/fixtures/hello_world.rs`:
```rust
fn main() {
    println!("Hello, world!");
}
```

`tests/fixtures/simple_fn.rs`:
```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    let result = add(2, 3);
    println!("{}", result);
}
```

`tests/fixtures/let_bindings.rs`:
```rust
fn test() {
    let x = 42;
    let y = "hello";
    let z = x + 1;
}
```

### Step 3.5: Integration Tests

Create `tests/integration_tests.rs` - follow design doc lines 757-873

### Step 3.6: Regression Tests

Create `tests/regression_tests.rs` - follow design doc lines 910-947

### Step 3.7: Verify Chunk 3

```bash
cargo test -p oxur-ast
cargo run -p oxur-ast -- to-ast tests/fixtures/hello_world.rs
cargo run -p oxur-ast -- verify tests/fixtures/hello_world.rs
```

**Chunk 3 Complete!** ✓

---

## Chunk 4: Polish & Performance

### Step 4.1: Create Benchmark Suite

Create `benches/conversion_bench.rs` - follow design doc lines 956-1045

### Step 4.2: Create Examples

`examples/parse_rust_file.rs` - design doc lines 1052-1087
`examples/convert_file.rs` - design doc lines 1090-1137

### Step 4.3: Update Documentation

Create `crates/oxur-ast/README.md` - adapt design doc lines 1146-1222 with:
- CLI name: `aster` (not `oxur-ast`)
- Commands: `to-ast`, `to-rust`

### Step 4.4: Final Verification

```bash
# All tests
cargo test -p oxur-ast

# Clippy
cargo clippy -p oxur-ast -- -D warnings

# Benchmarks
cargo bench -p oxur-ast

# Examples
cargo run -p oxur-ast --example parse_rust_file tests/fixtures/simple_fn.rs
cargo run -p oxur-ast --example convert_file tests/fixtures/hello_world.rs /tmp/hello.sexp

# CLI
cargo run -p oxur-ast -- to-ast tests/fixtures/hello_world.rs -o /tmp/test.sexp
cargo run -p oxur-ast -- verify tests/fixtures/simple_fn.rs --verbose
```

**Chunk 4 Complete!** ✓

---

## Success Criteria

Phase 3 is complete when all chunks pass and:

- [ ] Can parse real Rust files using `syn` ✓
- [ ] CLI tool `aster` works for all commands ✓
- [ ] All integration tests pass ✓
- [ ] All regression tests pass ✓
- [ ] Benchmark suite runs successfully ✓
- [ ] Examples work ✓
- [ ] Documentation is complete ✓
- [ ] No compiler warnings ✓
- [ ] Clean clippy output ✓

---

## Code Reuse Report TODO

After implementation, create a deduplication report comparing `aster` and `oxd`:

**File:** `crates/design/dev/oxur-ast/CLI-DEDUPLICATION-REPORT.md`

Document:
1. Shared patterns (colored output, error handling, etc.)
2. Duplicate code locations
3. Recommendations for `oxur-cli-common` crate
4. Migration strategy

---

## Known Limitations (Phase 3)

Acceptable for Phase 3, address in Phase 4+:

1. Using `Span::DUMMY` - need proc-macro2 integration
2. Simplified attribute handling
3. No complex generics support
4. Limited expression variants
5. Rust generation is Debug output only

---

## Next Steps After Phase 3

Phase 4+ enhancements:
- Complete expression coverage
- Full generic support
- Proper Rust code generation
- Macro expansion
- Better error messages
- LSP integration
- REPL support

---

*"From library to tool - the AST is now in your hands."*
