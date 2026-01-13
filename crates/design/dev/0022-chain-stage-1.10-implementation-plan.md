# Stage 1.10 Implementation Plan: Error Translator

**Date:** 2026-01-12
**Stage:** 1.10 from Phase 1 (Source Mapping Infrastructure)
**Deliverable:** Translate rustc errors to Oxur source positions
**Estimated Time:** 3 hours (180 minutes)
**Dependencies:** Stage 1.9 (rustc diagnostic parser) ✅
**Status:** 🚧 IN PROGRESS

---

## Objective

Implement an ErrorTranslator that uses the SourceMap to translate rustc error positions (in generated Rust code) back to original Oxur source positions. This completes the error reporting chain, making compiler errors point to the user's actual Oxur code instead of generated Rust.

## Background

### Current State (After Stage 1.9)

**We have:**
- SourceMap with complete transformation chain: Surface (Span) → Core (NodeId) → Rust (virtual NodeId)
- RustcDiagnostic parser that extracts file:line:col from rustc JSON output
- Compiler that shows Rust error positions (e.g., "generated.rs:5:10")

**What we need:**
- Translate "generated.rs:5:10" → "example.oxur:2:8"
- Look up Rust positions in SourceMap
- Walk back through the mapping chain to find original Oxur positions
- Format error messages with Oxur positions

### Translation Chain

```
rustc error: generated.rs:5:10: "cannot find value `x`"
    ↓
1. Parse rustc JSON (Stage 1.9 - done)
    ↓
2. Extract primary position: (file="generated.rs", line=5, col=10)
    ↓
3. Look up in SourceMap's lowering mappings
   - Find virtual Rust NodeId for this position
   - This requires new functionality: position → NodeId lookup
    ↓
4. Find Core NodeId from Rust NodeId
   - Use SourceMap's get_rust_node() method
    ↓
5. Find original Oxur position from Core NodeId
   - Use SourceMap's get_surface_position() method
    ↓
6. Format error with Oxur position
    ↓
Oxur error: example.oxur:2:8: "cannot find value `x`"
```

### Key Challenge: Position → NodeId Lookup

**Problem:** SourceMap currently stores:
- Core NodeId → Surface Position (forward mapping)
- Core NodeId → Rust NodeId (forward mapping)

But rustc gives us a **Rust position** (file:line:col), not a NodeId.

**Solution Options:**

**Option A: Reverse Index (Recommended)**
- Build a reverse index: Rust Position → Rust NodeId
- Store during lowering when we record mappings
- Fast lookup at error translation time
- Requires tracking file:line:col for each syn node

**Option B: Generated Code Markers**
- Insert comments in generated Rust with NodeIds
- Parse comments to find NodeId for position
- Simple but fragile (rustc might strip comments)

**Option C: No Translation Yet**
- Stage 1.10: Just show "Translation not yet implemented"
- Defer actual translation to future work
- Get the infrastructure in place

**Decision: Start with Option C, prepare for Option A**

For Stage 1.10, we'll:
1. Create ErrorTranslator struct with the API we need
2. Implement basic translation that shows both positions
3. Mark areas where reverse lookup would happen
4. Leave TODOs for Option A implementation in Phase 2

This gets error translation infrastructure working while acknowledging the limitation.

## Implementation Steps

### Step 1: Create error_translator.rs Module (45 min)

**Location:** `crates/oxur-comp/src/error_translator.rs`

**Data Structures:**

```rust
use crate::{RustcDiagnostic, Result};
use oxur_smap::SourceMap;

/// Translates rustc error positions to Oxur source positions
pub struct ErrorTranslator {
    source_map: SourceMap,
}

impl ErrorTranslator {
    /// Create a new error translator with the given source map
    pub fn new(source_map: SourceMap) -> Self {
        Self { source_map }
    }

    /// Translate a rustc diagnostic to Oxur source positions
    ///
    /// Returns a formatted error message with Oxur positions where possible.
    /// If translation is not possible, falls back to showing Rust positions.
    pub fn translate_diagnostic(&self, diagnostic: &RustcDiagnostic) -> String {
        let mut output = String::new();

        // Extract primary position from rustc diagnostic
        if let Some((rust_file, rust_line, rust_col)) = diagnostic.primary_position() {
            // TODO: Look up Rust position in reverse index
            // For now, try to infer from available mappings

            output.push_str(&format!("error: {}\n", diagnostic.message));

            // Show Rust position as fallback
            output.push_str(&format!(
                "  --> {}:{}:{}\n",
                rust_file, rust_line, rust_col
            ));

            // TODO: Show Oxur position when translation available
            output.push_str("  (Note: Error position translation not yet implemented)\n");
        } else {
            // No position available
            output.push_str(&format!("error: {}\n", diagnostic.message));
        }

        // Show error code if available
        if let Some(code) = &diagnostic.code {
            output.push_str(&format!("  code: {}\n", code.code));
        }

        output
    }

    /// Translate all diagnostics and format as a single error message
    pub fn translate_diagnostics(&self, diagnostics: &[RustcDiagnostic]) -> String {
        diagnostics
            .iter()
            .filter(|d| d.is_error())
            .map(|d| self.translate_diagnostic(d))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get a reference to the source map
    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }
}
```

**Why this approach:**
- Clean API that hides translation complexity
- Graceful degradation (shows Rust positions if translation unavailable)
- Prepared for future enhancement with reverse index
- Easy to test incrementally

### Step 2: Update lib.rs (5 min)

**File:** `crates/oxur-comp/src/lib.rs`

**Changes:**

```rust
pub mod error_translator;

pub use error_translator::ErrorTranslator;
```

Add module declaration and public export.

### Step 3: Update Compiler to Use ErrorTranslator (30 min)

**File:** `crates/oxur-comp/src/compiler.rs`

**Current code (compile_with_rustc):**
```rust
fn compile_with_rustc(&self, source: &Path, output: &Path) -> Result<()> {
    let output_result = Command::new("rustc")
        .arg(source)
        .arg("-o")
        .arg(output)
        .arg("--error-format=json")
        .output()?;

    if !output_result.status.success() {
        let stderr = String::from_utf8_lossy(&output_result.stderr);
        let diagnostics =
            crate::RustcDiagnostic::from_json_lines(&stderr).unwrap_or_else(|_| vec![]);

        let mut error_msg =
            format!("rustc failed with exit code: {:?}\n", output_result.status.code());

        for diag in diagnostics.iter().filter(|d| d.is_error()) {
            if let Some((file, line, col)) = diag.primary_position() {
                error_msg.push_str(&format!("  {}:{}:{}: {}\n", file, line, col, diag.message));
            } else {
                error_msg.push_str(&format!("  {}\n", diag.message));
            }
        }

        return Err(Error::Compile(error_msg));
    }

    Ok(())
}
```

**Updated signature and implementation:**
```rust
fn compile_with_rustc(
    &self,
    source: &Path,
    output: &Path,
    source_map: &oxur_smap::SourceMap,
) -> Result<()> {
    let output_result = Command::new("rustc")
        .arg(source)
        .arg("-o")
        .arg(output)
        .arg("--error-format=json")
        .output()?;

    if !output_result.status.success() {
        let stderr = String::from_utf8_lossy(&output_result.stderr);
        let diagnostics =
            crate::RustcDiagnostic::from_json_lines(&stderr).unwrap_or_else(|_| vec![]);

        // Use ErrorTranslator to convert rustc errors to Oxur positions
        let translator = crate::ErrorTranslator::new(source_map.clone());
        let translated = translator.translate_diagnostics(&diagnostics);

        let error_msg = format!(
            "rustc failed with exit code: {:?}\n\n{}",
            output_result.status.code(),
            translated
        );

        return Err(Error::Compile(error_msg));
    }

    Ok(())
}
```

**Update compile() to pass source_map:**
```rust
pub fn compile(
    &mut self,
    forms: Vec<CoreForm>,
    source_map: oxur_smap::SourceMap,
    output: &Path,
) -> Result<oxur_smap::SourceMap> {
    // Stage 3: Lower to Rust AST
    let mut lowerer = Lowerer::new(source_map);
    let (ast, source_map) = lowerer.lower(forms)?;

    // Stage 4: Generate Rust source
    let source = self.codegen.generate(&ast)?;

    // Write to temporary .rs file
    let rs_file = self.output_dir.join("generated.rs");
    std::fs::write(&rs_file, source)?;

    // Stage 5: Compile with rustc (pass source_map for error translation)
    self.compile_with_rustc(&rs_file, output, &source_map)?;

    Ok(source_map)
}
```

**Why:**
- ErrorTranslator encapsulates translation logic
- Compiler doesn't need to know translation details
- Easy to enhance translator without changing compiler

### Step 4: Add Tests for ErrorTranslator (45 min)

**File:** `crates/oxur-comp/src/error_translator.rs`

**Test cases:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::RustcDiagnostic;

    #[test]
    fn test_translator_creation() {
        let source_map = oxur_smap::SourceMap::new();
        let translator = ErrorTranslator::new(source_map);
        assert_eq!(translator.source_map().stats().surface_nodes, 0);
    }

    #[test]
    fn test_translate_diagnostic_with_position() {
        let source_map = oxur_smap::SourceMap::new();
        let translator = ErrorTranslator::new(source_map);

        let json = r#"{
            "message": "cannot find value `x` in this scope",
            "code": {
                "code": "E0425",
                "explanation": null
            },
            "level": "error",
            "spans": [
                {
                    "file_name": "generated.rs",
                    "byte_start": 42,
                    "byte_end": 43,
                    "line_start": 5,
                    "line_end": 5,
                    "column_start": 10,
                    "column_end": 11,
                    "is_primary": true,
                    "text": [],
                    "label": "not found in this scope",
                    "suggested_replacement": null,
                    "suggestion_applicability": null,
                    "expansion": null
                }
            ],
            "children": [],
            "rendered": null
        }"#;

        let diagnostic = RustcDiagnostic::from_json(json).unwrap();
        let output = translator.translate_diagnostic(&diagnostic);

        // Should contain error message
        assert!(output.contains("cannot find value `x`"));

        // Should contain Rust position (fallback)
        assert!(output.contains("generated.rs:5:10"));

        // Should contain error code
        assert!(output.contains("E0425"));

        // Should note that translation isn't implemented yet
        assert!(output.contains("translation not yet implemented"));
    }

    #[test]
    fn test_translate_diagnostic_without_position() {
        let source_map = oxur_smap::SourceMap::new();
        let translator = ErrorTranslator::new(source_map);

        let json = r#"{
            "message": "aborting due to previous error",
            "code": null,
            "level": "error",
            "spans": [],
            "children": [],
            "rendered": null
        }"#;

        let diagnostic = RustcDiagnostic::from_json(json).unwrap();
        let output = translator.translate_diagnostic(&diagnostic);

        // Should contain error message
        assert!(output.contains("aborting due to previous error"));

        // Should not contain position
        assert!(!output.contains("-->"));
    }

    #[test]
    fn test_translate_multiple_diagnostics() {
        let source_map = oxur_smap::SourceMap::new();
        let translator = ErrorTranslator::new(source_map);

        let json_lines = r#"{"message": "error 1", "code": null, "level": "error", "spans": [], "children": [], "rendered": null}
{"message": "warning 1", "code": null, "level": "warning", "spans": [], "children": [], "rendered": null}
{"message": "error 2", "code": null, "level": "error", "spans": [], "children": [], "rendered": null}"#;

        let diagnostics = RustcDiagnostic::from_json_lines(json_lines).unwrap();
        let output = translator.translate_diagnostics(&diagnostics);

        // Should contain errors but not warnings
        assert!(output.contains("error 1"));
        assert!(output.contains("error 2"));
        assert!(!output.contains("warning 1"));
    }

    #[test]
    fn test_translator_with_populated_source_map() {
        use oxur_lang::{Expander, Parser};

        // Create a source map with actual mappings
        let source = r#"(deffn main ()
  (println! "Hello"))"#;

        let mut parser = Parser::new(source.to_string());
        let surface_forms = parser.parse().unwrap();

        let mut expander = Expander::new();
        let _core_forms = expander.expand(surface_forms).unwrap();
        let source_map = expander.source_map().clone();

        // Verify source map has mappings
        let stats = source_map.stats();
        assert!(stats.surface_nodes > 0, "Should have surface mappings");

        let translator = ErrorTranslator::new(source_map);

        // Verify translator has access to source map
        assert!(translator.source_map().stats().surface_nodes > 0);
    }

    #[test]
    fn test_source_map_accessor() {
        let mut source_map = oxur_smap::SourceMap::new();
        let node_id = oxur_smap::new_node_id();
        let pos = oxur_smap::SourcePos::new("test.oxur".to_string(), 1, 1, 1);
        source_map.record_surface_node(node_id, pos);

        let translator = ErrorTranslator::new(source_map);

        // Should be able to access source map through translator
        let retrieved_pos = translator.source_map().get_surface_position(&node_id);
        assert!(retrieved_pos.is_some());
    }
}
```

**Test coverage:**
- Translator creation
- Translating diagnostic with position
- Translating diagnostic without position
- Filtering errors vs warnings
- Using populated source map
- Accessing source map through translator

### Step 5: Update Existing Tests (15 min)

**File:** `crates/oxur-comp/src/compiler.rs`

Tests need updating since `compile_with_rustc()` signature changed.

**Tests to check:**
- `test_compile_with_empty_forms` - Should still work
- `test_compile_hello_world` - Should still work

Both tests call `compile()` which internally calls `compile_with_rustc()`, so they should continue working without changes. Verify with test run.

### Step 6: Code Quality Checks (15 min)

**Run tests:**
```bash
cargo test --package oxur-comp
```

**Expected result:**
- All existing tests pass (22 tests)
- 6 new ErrorTranslator tests pass
- Total: 28 tests

**Run clippy:**
```bash
cargo clippy --package oxur-comp -- -D warnings
```

**Run formatting:**
```bash
cargo fmt --package oxur-comp
```

### Step 7: Integration Test (15 min)

Create an end-to-end test that verifies error translation:

**Add to compiler.rs tests:**

```rust
#[test]
fn test_error_translation_format() {
    use oxur_lang::{Expander, Parser};
    use tempfile::TempDir;

    // Parse code with intentional error (undefined variable)
    let source = r#"(deffn main ()
  (println! x))"#;  // `x` is undefined

    let mut parser = Parser::new(source.to_string());
    let surface_forms = parser.parse().unwrap();

    let mut expander = Expander::new();
    let core_forms = expander.expand(surface_forms).unwrap();
    let source_map = expander.source_map().clone();

    // Compile (this should fail with rustc error)
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("build");
    std::fs::create_dir_all(&output_dir).unwrap();

    let mut compiler = Compiler::new(output_dir);
    let binary_path = temp_dir.path().join("test_error");

    let result = compiler.compile(core_forms, source_map, &binary_path);

    // Should fail with compilation error
    assert!(result.is_err(), "Should fail due to undefined variable");

    if let Err(crate::Error::Compile(msg)) = result {
        // Error message should mention the error
        eprintln!("Error message:\n{}", msg);

        // Should contain rustc error code
        // (exact format depends on rustc version, but should have some structure)
        assert!(msg.len() > 0, "Should have error message");
    }
}
```

**Note:** This test verifies the error path works. Exact error format verification is deferred to Stage 1.11.

### Step 8: Documentation (15 min)

**Add module documentation:**

```rust
//! Error translation
//!
//! Translates rustc compilation errors from generated Rust code positions
//! back to original Oxur source code positions using the SourceMap.
//!
//! # Current Implementation
//!
//! Stage 1.10 provides the infrastructure for error translation but doesn't
//! yet implement full position lookup. Error messages show:
//! - The error message from rustc
//! - The generated Rust file position (as fallback)
//! - A note that full translation is not yet implemented
//!
//! # Future Enhancement (Phase 2)
//!
//! Full position translation requires a reverse index:
//! - Rust Position (file:line:col) → Rust NodeId
//! - Built during lowering when generating syn nodes
//! - Fast lookup at error translation time
//!
//! This will enable errors like:
//! ```text
//! error: cannot find value `x` in this scope
//!   --> example.oxur:2:8
//! ```
//!
//! Instead of:
//! ```text
//! error: cannot find value `x` in this scope
//!   --> generated.rs:5:10
//! ```
```

**Update compiler.rs compile() documentation:**

```rust
/// Compile Core Forms to a binary
///
/// Accepts the SourceMap from the expansion phase and returns it
/// with lowering mappings added for error reporting.
///
/// # Error Translation
///
/// If rustc compilation fails, errors are translated using the SourceMap
/// to show Oxur source positions where possible. Currently shows generated
/// Rust positions with a note that full translation is being implemented.
pub fn compile(
    &mut self,
    forms: Vec<CoreForm>,
    source_map: oxur_smap::SourceMap,
    output: &Path,
) -> Result<oxur_smap::SourceMap> {
    // ...
}
```

## Success Criteria

After completion, verify:

- [ ] ErrorTranslator struct created with clean API
- [ ] translate_diagnostic() method handles diagnostics with/without positions
- [ ] translate_diagnostics() filters and formats multiple errors
- [ ] Compiler uses ErrorTranslator for error formatting
- [ ] 6 new ErrorTranslator tests pass
- [ ] All existing tests still pass (22 tests)
- [ ] Total: 28 tests passing
- [ ] Clippy clean
- [ ] Formatted correctly
- [ ] Integration test demonstrates error translation path
- [ ] Documentation explains current state and future work

## Expected Test Results

**Before Stage 1.10:**
- oxur-comp tests: 22 passing

**After Stage 1.10:**
- oxur-comp tests: 28 passing (22 existing + 6 new)
- Error messages formatted through ErrorTranslator
- Infrastructure ready for full translation in Phase 2

## Files to Modify

1. **`crates/oxur-comp/src/error_translator.rs`** (NEW)
   - ErrorTranslator struct
   - translate_diagnostic() and translate_diagnostics() methods
   - 6 unit tests
   - ~200 lines

2. **`crates/oxur-comp/src/lib.rs`**
   - Add error_translator module
   - Export ErrorTranslator
   - ~2 lines

3. **`crates/oxur-comp/src/compiler.rs`**
   - Update compile_with_rustc() signature (add source_map param)
   - Use ErrorTranslator for error formatting
   - Update compile() to pass source_map
   - Add integration test
   - ~40 lines changed, ~30 lines added

4. **`crates/design/dev/0022-chain-stage-1.10-implementation-plan.md`** (THIS FILE)
   - Implementation plan

5. **`crates/design/dev/0023-chain-stage-1.10-completion.md`** (CREATED AFTER)
   - Completion documentation

## Time Breakdown

1. Create error_translator.rs module: 45 min
2. Update lib.rs: 5 min
3. Update Compiler: 30 min
4. Add tests: 45 min
5. Update existing tests: 15 min
6. Code quality checks: 15 min
7. Integration test: 15 min
8. Documentation: 15 min

**Total: 180 minutes (3 hours)**

## Next Steps (After Stage 1.10)

**Stage 1.11:** End-to-end error translation tests (2 hours)
- Create Oxur files with intentional errors
- Verify error messages are correctly formatted
- Test various error types (undefined vars, type errors, etc.)
- Document error translation behavior

**Phase 2 Enhancement:**
- Implement reverse index: Rust Position → Rust NodeId
- Track file:line:col for each syn node during lowering
- Enable full position translation
- Update ErrorTranslator to use reverse index

## Related Documents

- Stage breakdown: `crates/design/dev/0012-pipeline-chain-completion-stages.md`
- Main plan: `crates/design/dev/0011-pipeline-chain-completion.md`
- Previous stage: Stage 1.9 (rustc diagnostic parser)
- Next stage: Stage 1.11 (error translation tests)

---

**Created:** 2026-01-12
**Status:** Ready for implementation
**Estimated Time:** 3 hours
