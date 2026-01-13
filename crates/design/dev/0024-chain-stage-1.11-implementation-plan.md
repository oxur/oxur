# Stage 1.11 Implementation Plan: Error Translation Tests

**Date:** 2026-01-12
**Stage:** 1.11 from Phase 1 (Source Mapping Infrastructure)
**Deliverable:** Comprehensive end-to-end error translation tests
**Estimated Time:** 2 hours (120 minutes)
**Dependencies:** Stage 1.10 (Error translator) ✅
**Status:** 🚧 IN PROGRESS

---

## Objective

Create comprehensive end-to-end tests for the error translation pipeline. Verify that rustc errors are correctly parsed, formatted, and presented through the ErrorTranslator. This completes Phase 1 (Source Mapping Infrastructure) by ensuring the entire error reporting chain works robustly.

## Background

### What We've Built (Stages 1.1-1.10)

**Complete pipeline:**
1. **Stage 1.1-1.5:** Position tracking in Parser (Span types, Surface Forms with positions)
2. **Stage 1.6:** SourceMap recording API (already existed)
3. **Stage 1.7:** Surface → Core mapping (Expander records mappings)
4. **Stage 1.8:** Core → Rust mapping (Lowerer records mappings)
5. **Stage 1.9:** rustc diagnostic parser (parse JSON output)
6. **Stage 1.10:** Error translator (format errors with positions)

### What Stage 1.11 Tests

**Error translation flow:**
```
Oxur source with error
    ↓
Parse → Expand → Lower → Generate → Compile (rustc fails)
    ↓
rustc JSON diagnostics
    ↓
RustcDiagnostic parser
    ↓
ErrorTranslator
    ↓
Formatted error message
```

**What we're testing:**
- End-to-end compilation of invalid Oxur code
- rustc error parsing and formatting
- ErrorTranslator output structure
- Various error types (undefined vars, type errors, etc.)
- Error message clarity and usefulness

**What we're NOT testing yet:**
- Full position translation (requires reverse index - Phase 2)
- Currently shows Rust positions with note about translation

## Implementation Steps

### Step 1: Create Test Module (15 min)

**Location:** `crates/oxur-comp/tests/error_translation.rs`

Create integration test file for end-to-end error translation testing.

**Structure:**
```rust
//! Integration tests for error translation
//!
//! Tests the complete error reporting pipeline:
//! Oxur source → Parse → Expand → Lower → Generate → Compile → Error translation

use oxur_comp::{Compiler, ErrorTranslator};
use oxur_lang::{Expander, Parser};
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to compile Oxur code and capture error
fn compile_and_get_error(source: &str) -> Result<String, String> {
    let mut parser = Parser::new(source.to_string());
    let surface_forms = parser.parse().map_err(|e| format!("Parse error: {}", e))?;

    let mut expander = Expander::new();
    let core_forms = expander.expand(surface_forms).map_err(|e| format!("Expand error: {}", e))?;
    let source_map = expander.source_map().clone();

    let temp_dir = TempDir::new().map_err(|e| format!("TempDir error: {}", e))?;
    let output_dir = temp_dir.path().join("build");
    std::fs::create_dir_all(&output_dir).map_err(|e| format!("Create dir error: {}", e))?;

    let mut compiler = Compiler::new(output_dir);
    let binary_path = temp_dir.path().join("test_binary");

    match compiler.compile(core_forms, source_map, &binary_path) {
        Ok(_) => Err("Expected compilation to fail but it succeeded".to_string()),
        Err(e) => Ok(e.to_string()),
    }
}

// Tests go here...
```

### Step 2: Test Undefined Variable Error (15 min)

**Test case:** Use undefined variable in function body

```rust
#[test]
fn test_undefined_variable_error() {
    let source = r#"(deffn main ()
  (println! x))"#;

    let error = compile_and_get_error(source).expect("Should produce error");

    // Should contain rustc exit code
    assert!(error.contains("rustc failed"), "Error should mention rustc failure");

    // Should contain error message
    assert!(
        error.contains("cannot find value") || error.contains("not found"),
        "Error should mention undefined variable"
    );

    // Should contain position information (generated.rs for now)
    assert!(error.contains("generated.rs"), "Error should include file position");

    // Should contain note about translation
    assert!(
        error.contains("translation not yet implemented"),
        "Error should note translation status"
    );
}
```

### Step 3: Test Type Error (15 min)

**Test case:** Type mismatch (if rustc catches it in generated code)

```rust
#[test]
fn test_multiple_errors() {
    let source = r#"(deffn main ()
  (println! x)
  (println! y))"#;

    let error = compile_and_get_error(source).expect("Should produce error");

    // Should mention multiple undefined variables
    // Note: rustc might only report first error, which is fine
    assert!(
        error.contains("cannot find value") || error.contains("not found"),
        "Error should mention undefined variable(s)"
    );
}
```

### Step 4: Test Error Formatting (15 min)

**Test case:** Verify error message structure

```rust
#[test]
fn test_error_message_format() {
    let source = r#"(deffn main ()
  (println! undefined_var))"#;

    let error = compile_and_get_error(source).expect("Should produce error");

    // Error should have structured format
    assert!(error.contains("error:"), "Should start with 'error:'");

    // Should have file:line:col format
    let has_position = error.contains(":") &&
                       error.chars().filter(|&c| c == ':').count() >= 3;
    assert!(has_position, "Should contain file:line:col position");
}
```

### Step 5: Test Valid Code (No Error) (10 min)

**Test case:** Verify valid code compiles successfully

```rust
#[test]
fn test_valid_code_no_error() {
    let source = r#"(deffn main ()
  (println! "Hello, world!"))"#;

    let mut parser = Parser::new(source.to_string());
    let surface_forms = parser.parse().unwrap();

    let mut expander = Expander::new();
    let core_forms = expander.expand(surface_forms).unwrap();
    let source_map = expander.source_map().clone();

    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("build");
    std::fs::create_dir_all(&output_dir).unwrap();

    let mut compiler = Compiler::new(output_dir);
    let binary_path = temp_dir.path().join("test_binary");

    // Should compile successfully
    let result = compiler.compile(core_forms, source_map, &binary_path);
    assert!(result.is_ok(), "Valid code should compile: {:?}", result.err());

    // Binary should exist
    assert!(binary_path.exists(), "Binary should be created");
}
```

### Step 6: Test Error Code Extraction (10 min)

**Test case:** Verify rustc error codes are included

```rust
#[test]
fn test_error_code_included() {
    let source = r#"(deffn main ()
  (println! nonexistent))"#;

    let error = compile_and_get_error(source).expect("Should produce error");

    // Should contain rustc error code (e.g., E0425 for undefined variable)
    // Note: Exact code may vary by rustc version
    assert!(
        error.contains("code:") || error.contains("E0"),
        "Error should include rustc error code"
    );
}
```

### Step 7: Test SourceMap Integration (15 min)

**Test case:** Verify SourceMap is passed through pipeline

```rust
#[test]
fn test_source_map_passed_through() {
    let source = r#"(deffn main ()
  (println! "Test"))"#;

    let mut parser = Parser::new(source.to_string());
    let surface_forms = parser.parse().unwrap();

    let mut expander = Expander::new();
    let core_forms = expander.expand(surface_forms).unwrap();
    let source_map = expander.source_map().clone();

    // SourceMap should have surface mappings
    let stats = source_map.stats();
    assert!(stats.surface_nodes > 0, "Should have surface mappings");

    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("build");
    std::fs::create_dir_all(&output_dir).unwrap();

    let mut compiler = Compiler::new(output_dir);
    let binary_path = temp_dir.path().join("test_binary");

    // Compile should return SourceMap with lowering mappings added
    let result_map = compiler.compile(core_forms, source_map, &binary_path).unwrap();

    // Should have both surface and lowering mappings
    let result_stats = result_map.stats();
    assert!(result_stats.surface_nodes > 0, "Should preserve surface mappings");
    assert!(result_stats.lowerings > 0, "Should add lowering mappings");
}
```

### Step 8: Document Error Translation Behavior (15 min)

**Create:** `crates/oxur-comp/tests/README.md`

Document the error translation tests and current behavior:

```markdown
# Error Translation Integration Tests

These tests verify the complete error reporting pipeline from Oxur source code
to formatted rustc error messages.

## Test Coverage

- **Undefined variable errors**: Tests that rustc errors for undefined variables
  are correctly parsed and formatted
- **Error message format**: Verifies structured error output with positions
- **Valid code compilation**: Ensures valid Oxur code compiles successfully
- **Error codes**: Checks that rustc error codes (e.g., E0425) are included
- **SourceMap integration**: Verifies SourceMap flows through entire pipeline
- **Multiple errors**: Tests handling of multiple compilation errors

## Current Behavior (Phase 1)

Error messages currently show:
- rustc error message
- Generated Rust file position (e.g., `generated.rs:5:10`)
- rustc error code (e.g., `E0425`)
- Note that position translation is not yet implemented

Example:
```
rustc failed with exit code: Some(1)

error: cannot find value `x` in this scope
  --> generated.rs:5:10
  (Note: Error position translation not yet implemented)
  code: E0425
```

## Future Enhancement (Phase 2)

When reverse index is implemented, errors will show:
- Oxur source file position (e.g., `example.oxur:2:8`)
- No note about translation (it will be working)

## Running Tests

```bash
# Run all error translation tests
cargo test --test error_translation

# Run with output to see error messages
cargo test --test error_translation -- --nocapture
```
```

### Step 9: Code Quality Checks (10 min)

**Run tests:**
```bash
cargo test --test error_translation
```

**Run clippy:**
```bash
cargo clippy --package oxur-comp --tests -- -D warnings
```

**Run formatting:**
```bash
cargo fmt --package oxur-comp
```

### Step 10: Update Phase 1 Completion Document (15 min)

**Create:** `crates/design/dev/0025-phase-1-completion.md`

Document the completion of Phase 1 (Source Mapping Infrastructure):

- Summary of all 11 stages
- What was accomplished
- Test coverage statistics
- What's next (Phase 2)

## Success Criteria

After completion, verify:

- [ ] Integration test file created: `tests/error_translation.rs`
- [ ] 6+ end-to-end error translation tests
- [ ] Test for undefined variable error
- [ ] Test for valid code (no error)
- [ ] Test for error message format
- [ ] Test for error code extraction
- [ ] Test for SourceMap integration
- [ ] Test for multiple errors
- [ ] All tests passing
- [ ] Clippy clean
- [ ] Formatted correctly
- [ ] Documentation created: `tests/README.md`
- [ ] Phase 1 completion document created

## Expected Test Results

**Before Stage 1.11:**
- oxur-comp tests: 29 unit tests
- No integration tests

**After Stage 1.11:**
- oxur-comp tests: 29 unit tests
- oxur-comp integration tests: 6+ tests
- All tests passing
- Complete Phase 1 test coverage

## Files to Create/Modify

1. **`crates/oxur-comp/tests/error_translation.rs`** (NEW)
   - Integration tests for error translation
   - 6+ comprehensive test cases
   - ~200-300 lines

2. **`crates/oxur-comp/tests/README.md`** (NEW)
   - Documentation for error translation tests
   - Current behavior and future enhancements
   - ~50 lines

3. **`crates/design/dev/0024-chain-stage-1.11-implementation-plan.md`** (THIS FILE)
   - Implementation plan

4. **`crates/design/dev/0025-chain-stage-1.11-completion.md`** (CREATED AFTER)
   - Stage completion documentation

5. **`crates/design/dev/0026-phase-1-completion.md`** (CREATED AFTER)
   - Phase 1 overall completion documentation
   - Summary of all 11 stages
   - Statistics and achievements

## Time Breakdown

1. Create test module: 15 min
2. Test undefined variable: 15 min
3. Test type error: 15 min
4. Test error formatting: 15 min
5. Test valid code: 10 min
6. Test error codes: 10 min
7. Test SourceMap: 15 min
8. Documentation: 15 min
9. Code quality: 10 min
10. Completion docs: 15 min

**Total: 135 minutes (~2.25 hours, under 2h estimate if we're efficient)**

## Next Steps (After Stage 1.11)

**Phase 1 COMPLETE! 🎉**

Move to Phase 2: Stage 4 Integration
- Implement Oxur AST buffer zone
- Add reverse index for position translation
- Enable full error translation (Rust position → Oxur position)
- Replace syn with Oxur AST nodes

## Related Documents

- Stage breakdown: `crates/design/dev/0012-pipeline-chain-completion-stages.md`
- Main plan: `crates/design/dev/0011-pipeline-chain-completion.md`
- Previous stage: Stage 1.10 (Error translator)
- Phase completion: Phase 1 (Source Mapping Infrastructure)

---

**Created:** 2026-01-12
**Status:** Ready for implementation
**Estimated Time:** 2 hours
**Final Stage of Phase 1!** 🚀
