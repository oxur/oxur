# CLI Tool Naming

**Decision Date:** 2025-12-29

## Name: `aster`

The CLI tool for oxur-ast will be called **`aster`**.

**Rationale:**
- Wordplay on "AST-er" (AST tool)
- Short, memorable, easy to type
- Fits the Oxur project naming style
- Clear connection to the AST manipulation purpose

**Scope:**
- AST manipulation and inspection
- Low-level S-expression handling
- Conversions between S-expressions and Rust AST
- **NOT** a REPL (that will be a separate CLI tool)

**Command Naming Philosophy:**
Since the context is clearly Oxur (Lisp) and Rust, we can drop redundant parts:
- ~~`rust-to-sexp`~~ → **`to-ast`** (implies S-expression in Oxur context)
- ~~`sexp-to-rust`~~ → **`to-rust`** (input is obviously S-expression)

**Proposed usage examples:**
```bash
# Convert Rust to S-expression AST
aster to-ast hello.rs

# Convert S-expression to Rust
aster to-rust hello.sexp

# Round-trip verification
aster verify hello.rs

# Show AST structure
aster inspect hello.rs
```

**TODO:**
- [ ] Discuss and finalize command names when Phase 3 begins
- [ ] Consider additional commands for AST manipulation/inspection

**Related:**
- Phase 3 design: `0007-oxur-ast-phase-3-integration-testing-cli.md`
- Will be implemented in `crates/aster/` (to be created)
- REPL will be a separate tool (TBD)
