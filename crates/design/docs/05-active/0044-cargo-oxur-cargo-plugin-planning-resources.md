---
number: 44
title: "cargo-oxur: Cargo Plugin Planning & Resources"
author: "Duncan McGreggor"
component: All
tags: [change-me]
created: 2026-01-07
updated: 2026-01-07
state: Active
supersedes: null
superseded-by: null
version: 1.0
---

# cargo-oxur: Cargo Plugin Planning & Resources

**Status**: Answers added based on working compiler implementation
**Date**: 2026-01-07
**Context**: After successfully implementing complete Oxur → Rust → Binary pipeline

---

## Executive Summary

This document provides resources for building `cargo-oxur`, a Cargo subcommand that wraps Cargo's build functionality to compile Oxur projects. The plugin will enable commands like `cargo oxur build`, `cargo oxur run`, etc.

**NEW in v0.2**: Added concrete answers to all discussion questions based on the working compiler we built today.

---

## Part 1: Cargo Plugin Development Resources

### Official Documentation

The official Rust documentation on extending Cargo is surprisingly brief:

> "Cargo is designed so that you can extend it with new subcommands without having to modify it. If a binary in your `$PATH` is named `cargo-something`, you can run it as if it were a Cargo subcommand by running `cargo something`. Custom commands like this are also listed when you run `cargo --list`."
> — [The Rust Programming Language Book, Chapter 14.5](https://doc.rust-lang.org/book/ch14-05-extending-cargo.html)

### Core Mechanism

The mechanism is elegantly simple:

1. Create a binary named `cargo-{subcommand}` (e.g., `cargo-oxur`)
2. Put it in `$PATH` or `~/.cargo/bin/`
3. Run with `cargo {subcommand}` (e.g., `cargo oxur`)

When Cargo sees `cargo oxur build`, it translates this to invoking `cargo-oxur build`.

### Key Crates for Cargo Plugin Development

| Crate | Purpose | Notes |
|-------|---------|-------|
| **`cargo_metadata`** | Parse `cargo metadata` output | ~127M downloads, battle-tested. Provides structured access to workspace, packages, dependencies |
| **`clap`** | CLI argument parsing | The standard. Has specific patterns for cargo subcommands |
| **`clap-cargo`** | Cargo-specific clap helpers | Mimics cargo's interface (--package, --workspace flags, etc.) |
| **`escargot`** | Invoke cargo commands | For programmatically running `cargo build`, `cargo run`, etc. |

---

## Part 2: Discussion Topics with Answers

### Topic 1: Command Surface Area ✅ ANSWERED

**Question**: What commands should `cargo oxur` support?

**ANSWER** (based on current implementation):

**Phase 1 - Minimum Viable (IMPLEMENT FIRST)**:

```bash
cargo oxur build    # Compile Oxur project (wraps oxurc)
cargo oxur run      # Build and execute (build + run binary)
cargo oxur check    # Quick syntax check (parse + expand only, no rustc)
```

**Phase 2 - Extended Set**:

```bash
cargo oxur test     # Run tests (detect test functions)
cargo oxur new      # Create new Oxur project (template with Cargo.toml)
cargo oxur repl     # Start REPL session (use oxur-repl we built)
```

**Phase 3 - Tooling Integration**:

```bash
cargo oxur fmt      # Format Oxur source (use oxurfmt)
cargo oxur doc      # Generate documentation
cargo oxur expand   # Show expanded macros (Surface → Core Forms)
cargo oxur ast      # Show AST (use aster)
```

**Answers to Sub-Questions**:

1. **Do we mirror all cargo commands or just build-related ones?**
   - Start with build-related only (build, run, check, test)
   - Add tooling commands as separate phase
   - Skip commands that don't make sense (publish, yank, search)

2. **How do we handle mixed Oxur/Rust projects?**
   - Detect `.ox`/`.oxr`/`.oxur`/`.lisp` files → compile to `.rs` first
   - Regular `.rs` files pass through to cargo unchanged
   - Both can coexist in same project
   - Generated `.rs` files go to `target/oxur-gen/` (see Topic 2)

3. **Should `cargo build` in an Oxur project "just work" (auto-detect)?**
   - **NO** - require explicit `cargo oxur build`
   - Reason: Avoids magic/surprises, explicit is better
   - Users choose when to use Oxur compilation
   - In future: could add build.rs helper for auto-detection

---

### Topic 2: Project Structure ✅ ANSWERED

**Question**: What does an Oxur project look like?

**ANSWER** (based on current implementation):

**Recommended: Option A - Oxur-only project**

```
my-oxur-project/
├── Cargo.toml          # Standard Cargo.toml
├── src/
│   ├── lib.oxr         # Oxur library (or .ox, .oxur, .lisp)
│   └── main.oxr        # Oxur binary
└── target/
    ├── oxur-gen/       # Generated .rs files (gitignored)
    │   ├── lib.rs
    │   └── main.rs
    └── debug/          # Standard cargo output
        └── my-oxur-project
```

**Supported: Option B - Mixed Rust/Oxur project**

```
my-mixed-project/
├── Cargo.toml
└── src/
    ├── lib.rs          # Rust code (unchanged)
    ├── utils.oxr       # Oxur code → compiles to utils.rs in target/oxur-gen/
    └── main.rs         # Can use: mod utils; (from generated code)
```

**Future: Option C - Oxur workspace member**

```
workspace/
├── Cargo.toml          # workspace definition
├── crates/
│   ├── rust-lib/       # Pure Rust crate
│   └── oxur-lib/       # Pure Oxur crate
└── apps/
    └── mixed-app/      # Uses both via normal dependencies
```

**Answers to Sub-Questions**:

1. **How do we identify Oxur files?**
   - **Extension-based**: `.oxr`, `.oxur`, `.ox`, `.lisp` (all supported)
   - Primary recommendation: `.oxr` (short, unambiguous)
   - No metadata required in Cargo.toml (scan for files)
   - Future: optional `[package.metadata.oxur]` for configuration

2. **Do we generate .rs files in-place or in a build directory?**
   - **Build directory**: `target/oxur-gen/`
   - Reasons:
     - Keeps source tree clean
     - Clear separation of source vs generated
     - Easy to gitignore entire directory
     - Matches our current oxurc implementation
   - Generated files mirror source structure:

     ```
     src/lib.oxr     → target/oxur-gen/src/lib.rs
     src/foo/bar.oxr → target/oxur-gen/src/foo/bar.rs
     ```

3. **How do we handle incremental compilation?**
   - Hash-based: Compare hash of `.oxr` file content
   - Skip recompilation if:
     - Generated `.rs` exists
     - Hash matches
     - Newer timestamp than source
   - Store hashes in `target/oxur-cache/hashes.json`
   - Leverage cargo's existing incremental compilation for `.rs` → binary

---

### Topic 3: Build Pipeline Integration ✅ ANSWERED

**Question**: How does Oxur integrate with Cargo's build process?

**ANSWER** (based on current working implementation):

**Recommended: Approach 1 - Pre-build hook** (SIMPLEST, WORKS NOW)

```
cargo oxur build
    └── [1] Scan src/ for .oxr/.ox/.oxur/.lisp files
        └── [2] For each Oxur file:
            └── Check target/oxur-cache/hashes.json
            └── If changed or missing:
                └── oxurc compile to target/oxur-gen/
        └── [3] Update Cargo.toml or use --manifest-path trick
            └── Point cargo at target/oxur-gen/ as lib
        └── [4] cargo build with generated sources
        └── [5] Post-process errors through source maps
```

**Implementation Details**:

```rust
// Pseudocode for cargo-oxur build
fn handle_build(args: BuildArgs) -> Result<()> {
    let manifest = find_manifest(&args.manifest_path)?;
    let project_root = manifest.parent().unwrap();

    // Step 1: Find Oxur files
    let oxur_files = find_oxur_files(project_root)?;

    // Step 2: Compile Oxur → Rust (with caching)
    for file in oxur_files {
        if needs_recompile(&file)? {
            compile_oxur_file(&file, "target/oxur-gen/")?;
        }
    }

    // Step 3: Invoke cargo build
    let status = Command::new("cargo")
        .arg("build")
        .args(&args.cargo_args)
        .env("CARGO_TARGET_DIR", "target") // Use standard target
        .status()?;

    // Step 4: Map errors if failed
    if !status.success() {
        map_and_display_errors()?;
    }

    Ok(())
}
```

**Why Not Other Approaches (for now)**:

- **Approach 2 (RUSTC_WRAPPER)**: More complex, harder to debug, but consider for Phase 2
  - Benefit: Tighter cargo integration
  - Cost: Must intercept every rustc call, parse arguments

- **Approach 3 (build.rs)**: Requires user boilerplate in every project
  - Benefit: Works with normal `cargo build`
  - Cost: Breaks explicitness principle, adds magic

**Decision**: Start with Approach 1, evaluate RUSTC_WRAPPER for v2.0

---

### Topic 4: Error Handling & Source Maps ✅ ANSWERED

**Question**: How do we report errors in terms of original Oxur source?

**ANSWER** (based on oxur-smap and oxur-repl implementation):

**WE ALREADY HAVE THE INFRASTRUCTURE!**

The oxur-smap crate provides:

- `SourcePos` tracking through entire pipeline
- `SourceMap` for transformation tracking
- Already integrated in oxur-lang and oxur-comp

**The Solution**:

```
Oxur source (foo.oxr, line 10, col 5)
    ↓ Parser (Stage 1)
    Surface Form + SourcePos(file=foo.oxr, line=10, col=5, node=123)
    ↓ Expander (Stage 2)
    Core Form + SourcePos(maintained through expansion)
    ↓ Lowerer (Stage 3)
    Rust AST + NodeId → SourcePos mapping
    ↓ CodeGenerator (Stage 4)
    Rust source with /* oxur_node=123 */ comments
    ↓ rustc (Stage 5)
    Rust error: "line 47: expected `;`"
    ↓ Error Mapper (NEW component for cargo-oxur)
    Parse /* oxur_node=123 */ from line 47
    Lookup NodeId(123) → SourcePos(file=foo.oxr, line=10, col=5)
    ↓ Error Formatter (NEW component)
    Display: "foo.oxr:10:5: expected `;` after expression"
```

**Implementation Plan**:

1. **Enhance CodeGenerator** (oxur-comp/src/codegen.rs):

   ```rust
   // Already generates formatted Rust, ADD node comments:
   pub fn generate(&self, file: &syn::File, source_map: &SourceMap) -> Result<String> {
       let mut code = prettyplease::unparse(file);

       // Insert oxur_node comments
       for (node_id, source_pos) in source_map.iter() {
           // Find corresponding line in generated code
           // Insert: /* oxur_node=NODE_ID */
       }

       Ok(code)
   }
   ```

2. **Persist Source Maps**:

   ```rust
   // Save alongside generated .rs files
   // target/oxur-gen/src/lib.rs
   // target/oxur-gen/src/lib.rs.oxurmap (JSON or binary format)

   #[derive(Serialize, Deserialize)]
   struct PersistedSourceMap {
       mappings: Vec<(NodeId, SourcePos)>,
       oxur_file: PathBuf,
       rust_file: PathBuf,
   }
   ```

3. **Parse rustc JSON Errors** (in cargo-oxur):

   ```rust
   // Use rustc --error-format=json
   cargo build --message-format=json 2>&1 | cargo-oxur process-errors

   #[derive(Deserialize)]
   struct RustcError {
       message: String,
       spans: Vec<Span>,
   }

   fn map_error(rust_error: RustcError, source_map: &PersistedSourceMap) -> OxurError {
       // Extract line from rust_error.spans
       // Find /* oxur_node=N */ comment near that line
       // Lookup in source_map
       // Return error with original Oxur position
   }
   ```

4. **Format with ariadne** (already in oxur-repl):

   ```rust
   use ariadne::{Report, ReportKind, Label, Source};

   // Display beautiful rustc-style errors pointing to Oxur source
   Report::build(ReportKind::Error, source_pos.file, source_pos.offset)
       .with_message("type mismatch")
       .with_label(Label::new((source_pos.file, span))
           .with_message("expected i32, found &str"))
       .finish()
       .eprint(Source::from(oxur_source))?;
   ```

**Status**:

- ✅ oxur-smap exists and is integrated
- ✅ ariadne is already a dependency in oxur-repl
- ⏳ Need to add node comments to codegen
- ⏳ Need to persist source maps
- ⏳ Need error mapper in cargo-oxur

---

### Topic 5: Crate Naming & Installation ✅ ANSWERED

**Question**: What's the binary called and how is it installed?

**ANSWER** (based on current project structure):

**Naming** (following Rust conventions):

```
cargo-oxur           # Cargo subcommand binary
```

**Installation** (Phase 1):

```bash
# From source (development)
cd oxur/crates/oxur-cli
cargo install --path .

# This installs to ~/.cargo/bin/:
~/.cargo/bin/cargo-oxur
```

**Installation** (Phase 2 - crates.io):

```bash
cargo install cargo-oxur
```

**Binary Layout** (current and proposed):

```
~/.cargo/bin/
├── cargo-oxur       # NEW: Cargo subcommand
├── oxurc            # EXISTS: Standalone compiler
├── oxd              # EXISTS: Design doc tool
├── aster            # EXISTS: AST tool
└── oxurfmt          # EXISTS: Formatter
```

**Answers to Sub-Questions**:

1. **Should cargo-oxur be a separate crate or part of oxur-cli?**
   - **Separate crate**: `crates/cargo-oxur/`
   - Reasons:
     - Clear separation of concerns
     - Different dependencies (cargo_metadata, clap-cargo)
     - Can be installed independently
     - Follows pattern of cargo-expand, cargo-miri, etc.
   - Structure:

     ```
     crates/
     ├── cargo-oxur/          # NEW crate
     │   ├── Cargo.toml
     │   └── src/
     │       └── main.rs      # Implements cargo subcommand
     ├── oxur-cli/            # Existing
     ├── oxur-comp/           # Existing (has oxurc binary)
     └── ...
     ```

2. **How do we handle version synchronization?**
   - All oxur crates use workspace version: `version.workspace = true`
   - cargo-oxur depends on: `oxur-comp`, `oxur-lang`, `oxur-smap`
   - Single source of truth in root Cargo.toml

3. **What's the relationship between `oxur build` and `cargo oxur build`?**
   - **They're different tools for different use cases**:

   ```bash
   # oxurc: Standalone compiler (like rustc)
   # Use when: Compiling a single file, scripting, direct control
   oxurc -o hello main.oxr

   # cargo oxur: Project build tool (like cargo)
   # Use when: Building a Cargo project, managing dependencies
   cargo oxur build --release
   ```

   - `cargo-oxur` internally calls `oxurc` (via oxur-comp API)
   - Similar to: rustc vs cargo build

---

### Topic 6: Testing Strategy ✅ ANSWERED

**Question**: How do we test the cargo plugin?

**ANSWER** (based on current test infrastructure):

**Test Levels**:

1. **Unit Tests** (in crates/cargo-oxur/src/lib.rs):

   ```rust
   #[cfg(test)]
   mod tests {
       #[test]
       fn test_find_oxur_files() { ... }

       #[test]
       fn test_needs_recompile_with_hash_change() { ... }

       #[test]
       fn test_error_mapping() { ... }
   }
   ```

2. **Integration Tests** (in crates/cargo-oxur/tests/):

   ```rust
   // tests/build_test.rs
   #[test]
   fn test_build_simple_project() {
       let temp_dir = TempDir::new().unwrap();
       copy_fixture("simple-lib", &temp_dir);

       let output = Command::new("cargo-oxur")
           .arg("build")
           .current_dir(&temp_dir)
           .output()
           .unwrap();

       assert!(output.status.success());
       assert!(temp_dir.join("target/debug/simple-lib").exists());
   }
   ```

3. **End-to-End Tests** (like oxur-comp's test_compile_hello_world):

   ```rust
   #[test]
   fn test_e2e_hello_world() {
       // Create project
       // Write Oxur source
       // Run cargo oxur build
       // Execute binary
       // Verify output
   }
   ```

**Test Fixtures** (leverage existing patterns from oxur-ast):

```
crates/cargo-oxur/test-data/
├── fixtures/
│   ├── simple-lib/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.oxr
│   ├── simple-bin/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.oxr
│   ├── mixed-project/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs        # Rust
│   │       └── utils.oxr     # Oxur
│   ├── workspace/
│   │   ├── Cargo.toml        # workspace
│   │   ├── pure-rust/
│   │   └── pure-oxur/
│   ├── compile-error/
│   │   └── src/
│   │       └── bad.oxr       # Intentional error
│   └── with-deps/
│       ├── Cargo.toml        # Has external deps
│       └── src/
│           └── lib.oxr       # Uses clap, serde, etc.
```

**Using oxur-testing crate**:

```rust
use oxur_testing::{test_file, env_lock::with_env_lock};

#[test]
fn test_compile_fixture() {
    with_env_lock(|| {
        let fixture = test_file!("fixtures/simple-lib/src/lib.oxr");
        // Test compilation
    });
}
```

**Snapshot Testing for Generated Code**:

```rust
use insta::assert_snapshot;

#[test]
fn test_generated_rust_code() {
    let oxur_source = "(deffn foo () (println! \"test\"))";
    let rust_code = compile_to_rust(oxur_source).unwrap();

    assert_snapshot!(rust_code);
}
```

**Answers to Sub-Questions**:

1. **How do we test error message quality?**
   - Snapshot tests for error output
   - Verify source positions are correct
   - Check that suggestions are helpful

2. **Do we need snapshot testing for generated Rust code?**
   - **YES** - use `insta` crate (already used in Rust ecosystem)
   - Catches unintended codegen changes
   - Documents expected output

3. **How do we test source map accuracy?**
   - Round-trip tests: Oxur pos → Rust pos → Oxur pos
   - Error mapping tests: Inject error → verify reported position
   - Already have patterns from oxur-smap tests

---

## Part 3: Initial Architecture Sketch

```
┌─────────────────────────────────────────────────────────────┐
│                      cargo oxur build                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     cargo-oxur binary                       │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  CLI (clap + clap-cargo)                              │  │
│  │  - Parse: cargo oxur [command] [args]                 │  │
│  │  - Dispatch to: build, run, check, test               │  │
│  └───────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Project Discovery (cargo_metadata)                   │  │
│  │  - Find Cargo.toml via cargo_metadata                 │  │
│  │  - Scan for .oxr/.ox/.oxur/.lisp files                │  │
│  │  - Build dependency graph                             │  │
│  └───────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Compilation Cache                                    │  │
│  │  - Load target/oxur-cache/hashes.json                 │  │
│  │  - Check if .oxr changed (hash comparison)            │  │
│  │  - Skip unchanged files                               │  │
│  └───────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Build Orchestration                                  │  │
│  │  - For each .oxr: oxurc compile → target/oxur-gen/    │  │
│  │  - Persist source maps (.oxurmap files)               │  │
│  │  - Invoke: cargo build --message-format=json          │  │
│  └───────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Error Post-Processing                                │  │
│  │  - Parse rustc JSON errors                            │  │
│  │  - Load .oxurmap for affected files                   │  │
│  │  - Map Rust positions → Oxur positions                │  │
│  │  - Display with ariadne                               │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
            │
            ├─────────────────┬─────────────────┬─────────────
            ▼                 ▼                 ▼
      ┌──────────┐     ┌──────────┐     ┌──────────┐
      │ oxur-lang│     │ oxur-comp│     │   cargo  │
      │ Parser   │────▶│ Compiler │────▶│  build   │
      │ Expander │     │ (oxurc)  │     │          │
      └──────────┘     └──────────┘     └──────────┘
            │                 │                 │
            │                 │                 │
      ┌──────────┐     ┌──────────┐     ┌──────────┐
      │ oxur-smap│     │ Source   │     │  Binary  │
      │ SourcePos│     │ Maps     │     │  Output  │
      └──────────┘     └──────────┘     └──────────┘
```

---

## Part 4: Questions to Resolve ✅ ALL ANSWERED

### 1. Scope ✅

**Question**: Is cargo-oxur just a thin wrapper, or does it add significant functionality?

**ANSWER**: **Thin wrapper with smart caching**

cargo-oxur is primarily an orchestrator that:

- ✅ Discovers Oxur files
- ✅ Manages incremental compilation (hash-based cache)
- ✅ Invokes oxurc for Oxur → Rust compilation
- ✅ Invokes cargo for Rust → Binary compilation
- ✅ Maps errors back to Oxur source

It does NOT:

- ❌ Re-implement compilation (delegates to oxurc)
- ❌ Provide new language features
- ❌ Manage dependencies differently than cargo

**Think of it as**: cargo for Oxur projects (like cargo is rustc + project management)

---

### 2. Detection ✅

**Question**: How do we identify an "Oxur project"?

**ANSWER**: **Extension-based scanning**

Detection strategy (in order):

1. Scan `src/` for files with extensions: `.oxr`, `.oxr`, `.oxur`, `.lisp`
2. If found → Oxur project (or mixed Rust/Oxur)
3. No special metadata required in Cargo.toml

Optional future enhancement:

```toml
[package.metadata.oxur]
enabled = true
extensions = ["oxr", "ox"]  # Override defaults
source_dirs = ["src", "oxur-src"]  # Additional dirs
```

**Rationale**:

- Simple, no magic
- Works with existing Cargo projects
- Easy to understand: "If you have .oxr files, you need cargo oxur build"

---

### 3. Generation ✅

**Question**: Where do generated `.rs` files go?

**ANSWER**: **`target/oxur-gen/` directory**

Structure:

```
project/
├── src/
│   ├── lib.oxr
│   └── foo/
│       └── bar.oxr
└── target/
    ├── oxur-gen/           # Generated Rust sources
    │   ├── src/
    │   │   ├── lib.rs
    │   │   └── foo/
    │   │       └── bar.rs
    │   └── .oxurmap/       # Source maps
    │       ├── src/
    │       │   ├── lib.rs.oxurmap
    │       │   └── foo/
    │       │       └── bar.rs.oxurmap
    ├── oxur-cache/         # Compilation cache
    │   └── hashes.json
    └── debug/              # Standard cargo output
        └── myproject
```

**Rationale**:

- Keeps source tree clean
- Easy to gitignore: `target/`
- Mirrors source structure for easy correlation
- Matches our working oxurc implementation

---

### 4. Caching ✅

**Question**: How do we avoid re-compiling unchanged Oxur files?

**ANSWER**: **Hash-based caching with timestamps**

Implementation:

```rust
// target/oxur-cache/hashes.json
{
  "src/lib.oxr": {
    "content_hash": "a1b2c3d4...",
    "timestamp": 1704672000,
    "output": "target/oxur-gen/src/lib.rs"
  }
}

fn needs_recompile(oxur_file: &Path, cache: &Cache) -> bool {
    let current_hash = hash_file(oxur_file);

    match cache.get(oxur_file) {
        Some(entry) => {
            // Recompile if hash changed OR output missing
            entry.content_hash != current_hash ||
            !Path::new(&entry.output).exists()
        }
        None => true, // No cache entry, must compile
    }
}
```

**Multi-level caching**:

1. **Our cache**: Oxur → Rust (hash-based)
2. **Cargo's cache**: Rust → Binary (cargo's incremental compilation)

**Result**: Only recompile what actually changed

---

### 5. Dependencies ✅

**Question**: Can Oxur code depend on Rust crates directly?

**ANSWER**: **YES - through normal Cargo.toml**

Oxur compiles to Rust, so dependencies work naturally:

```toml
# Cargo.toml
[package]
name = "my-oxur-project"
version = "0.1.0"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
clap = "4.5"
```

```lisp
; src/lib.oxr
(use
  (serde Serialize Deserialize)
  (clap Parser))

(defstruct config
  (derive serialize deserialize parser)
  (name:String)
  (port:u16))
```

This generates:

```rust
// target/oxur-gen/src/lib.rs
use serde::{Serialize, Deserialize};
use clap::Parser;

#[derive(Serialize, Deserialize, Parser)]
struct Config {
    name: String,
    port: u16,
}
```

**How it works**:

1. User adds dependencies to Cargo.toml (normal Cargo workflow)
2. Oxur code generates Rust that uses those crates
3. Cargo resolves and compiles everything together

**No special handling needed!**

---

### 6. Interop ✅

**Question**: Can Rust code call Oxur code and vice versa?

**ANSWER**: **YES - seamless bidirectional interop**

**Oxur calling Rust**:

```rust
// src/utils.rs (Rust)
pub fn helper(x: i32) -> i32 {
    x * 2
}
```

```lisp
; src/lib.oxr (Oxur)
(use (crate::utils helper))

(deffn main ()
  (let (result (helper 21))
    (println! "Result: {}" result)))
```

**Rust calling Oxur**:

```lisp
; src/oxur_utils.oxr (Oxur)
(deffn pub calculate (x:i32) (:> i32)
  (* x x))
```

```rust
// src/main.rs (Rust)
mod oxur_utils; // From target/oxur-gen/src/oxur_utils.rs

fn main() {
    let result = oxur_utils::calculate(5);
    println!("Result: {}", result);
}
```

**Key insight**: Oxur is Rust at the end of the day!

- All Oxur code becomes Rust code
- Rust modules system handles everything
- No FFI, no wrappers, just normal Rust interop

**Requirements**:

- Oxur files must use proper visibility: `pub` for exports
- Type signatures must be Rust-compatible
- Module structure mirrors file structure

---

## Part 5: Implementation Roadmap

### Phase 1: MVP (Weeks 1-2)

**Goal**: `cargo oxur build` compiles a simple Oxur project

```bash
# Create new crate
cargo new --bin crates/cargo-oxur

# Implement minimal functionality:
# 1. CLI parsing (clap)
# 2. Find .oxr files
# 3. Call oxurc for each file
# 4. Call cargo build

# Test with:
cd test-project
cargo oxur build
./target/debug/test-project
```

**Deliverables**:

- ✅ crates/cargo-oxur/ crate created
- ✅ Basic CLI structure (build command only)
- ✅ Oxur file discovery
- ✅ Direct oxurc invocation
- ✅ 5 integration tests

---

### Phase 2: Caching & Performance (Week 3)

**Goal**: Fast incremental builds

```bash
# Implement:
# 1. Hash-based cache (target/oxur-cache/hashes.json)
# 2. Skip unchanged files
# 3. Timestamp checks

# Test with:
cargo oxur build          # Compiles everything
# Edit one file
cargo oxur build          # Only recompiles that file (fast!)
```

**Deliverables**:

- ✅ Cache implementation
- ✅ Benchmark showing speedup
- ✅ 10 cache-related tests

---

### Phase 3: Error Mapping (Week 4)

**Goal**: Errors point to Oxur source, not generated Rust

```bash
# Implement:
# 1. Add /* oxur_node=N */ comments to codegen
# 2. Persist .oxurmap files
# 3. Parse rustc JSON errors
# 4. Map positions back to Oxur
# 5. Display with ariadne

# Test with:
# Write buggy Oxur code
cargo oxur build
# See error pointing to Oxur source!
```

**Deliverables**:

- ✅ Enhanced codegen with node comments
- ✅ Source map persistence
- ✅ Error mapper
- ✅ Beautiful error display
- ✅ 15 error mapping tests

---

### Phase 4: Additional Commands (Week 5)

**Goal**: `cargo oxur run`, `cargo oxur check`, `cargo oxur test`

```bash
cargo oxur run              # Build and execute
cargo oxur run -- --help    # Pass args to binary
cargo oxur check            # Fast syntax check (no rustc)
cargo oxur test             # Run tests
```

**Deliverables**:

- ✅ run command (build + execute)
- ✅ check command (parse + expand only)
- ✅ test command (detect test functions)
- ✅ Argument passing

---

### Phase 5: Polish & Documentation (Week 6)

**Goal**: Production-ready release

```bash
# Implement:
# 1. Comprehensive error messages
# 2. Progress indicators
# 3. Verbose mode (-v)
# 4. Documentation

# Test with:
cargo oxur --help    # Clear, helpful docs
cargo oxur build -v  # Shows what it's doing
```

**Deliverables**:

- ✅ User guide
- ✅ API documentation
- ✅ Example projects
- ✅ crates.io release

---

## Part 6: Next Steps (IMMEDIATE ACTIONS)

Based on successful compiler implementation, here's what to do next:

### Step 1: Create cargo-oxur Crate ⏭️ DO THIS FIRST

```bash
cd crates/
cargo new --bin cargo-oxur

# Add to workspace Cargo.toml:
# members = [..., "crates/cargo-oxur"]

# Add dependencies:
cd cargo-oxur
cargo add clap --features derive
cargo add clap-cargo
cargo add cargo_metadata
cargo add anyhow
cargo add oxur-comp --path ../oxur-comp
cargo add oxur-lang --path ../oxur-lang
cargo add oxur-smap --path ../oxur-smap
```

---

### Step 2: Write Minimal CLI ⏭️ DO THIS SECOND

```rust
// crates/cargo-oxur/src/main.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
struct Cli {
    #[command(subcommand)]
    command: CargoCommands,
}

#[derive(Subcommand)]
enum CargoCommands {
    /// Oxur build system
    Oxur(OxurArgs),
}

#[derive(Parser)]
struct OxurArgs {
    #[command(subcommand)]
    command: OxurCommands,
}

#[derive(Subcommand)]
enum OxurCommands {
    /// Build the current Oxur project
    Build {
        #[arg(long)]
        release: bool,
    },
}

fn main() {
    let Cli { command } = Cli::parse();

    match command {
        CargoCommands::Oxur(args) => match args.command {
            OxurCommands::Build { release } => {
                println!("Would build in {} mode",
                    if release { "release" } else { "debug" });
                // TODO: Implement
            }
        }
    }
}
```

---

### Step 3: Create Test Fixture ⏭️ DO THIS THIRD

```bash
mkdir -p crates/cargo-oxur/test-data/fixtures/simple-bin

# Create Cargo.toml
cat > crates/cargo-oxur/test-data/fixtures/simple-bin/Cargo.toml << 'EOF'
[package]
name = "simple-bin"
version = "0.1.0"
edition = "2021"
EOF

# Create main.oxr
cat > crates/cargo-oxur/test-data/fixtures/simple-bin/src/main.oxr << 'EOF'
(deffn main ()
  (println! "Hello from cargo oxur!"))
EOF
```

---

### Step 4: Test It! ⏭️ DO THIS FOURTH

```bash
# Build cargo-oxur
cd crates/cargo-oxur
cargo build

# Test with fixture
cd test-data/fixtures/simple-bin
../../../../target/debug/cargo-oxur oxur build

# Should see:
# Would build in debug mode
```

---

## Conclusion

We now have **concrete answers** to all planning questions based on:

- ✅ Working oxurc compiler (Stages 1-5 implemented)
- ✅ 34 passing tests
- ✅ End-to-end compilation proven
- ✅ Support for .oxr, .oxur, .ox, .lisp extensions
- ✅ Existing oxur-smap infrastructure for error mapping
- ✅ Clean separation of concerns

**The path forward is clear**: Build cargo-oxur as an orchestration layer over our proven compiler.

**Next document**: Create `ODD-00XX-cargo-oxur-design.md` with formal specification based on these decisions.

---

**Document Status**: Ready for design doc creation
**Last Updated**: 2026-01-07
**Version**: 0.2 (with all answers)
