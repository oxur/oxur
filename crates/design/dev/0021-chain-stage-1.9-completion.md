# Stage 1.9 Completion: rustc Diagnostic Parser

**Date:** 2026-01-12
**Stage:** 1.9 from Phase 1 (Source Mapping Infrastructure)
**Deliverable:** Parse rustc JSON output to extract error positions
**Estimated Time:** 2 hours
**Dependencies:** Stage 1.8 (Core → syn mapping) ✅
**Status:** ✅ COMPLETE

---

## Summary

Stage 1.9 implemented a rustc diagnostic parser that extracts error positions from rustc's JSON output. This enables the compiler to present structured error information with file:line:col positions, preparing for Stage 1.10's error translation back to Oxur source positions.

**Key Achievement:** The compiler can now parse rustc's JSON diagnostic format and extract structured error information including primary error locations.

## Implementation Details

### Code Changes

**1. New Module: rustc_diagnostic.rs** (`crates/oxur-comp/src/rustc_diagnostic.rs` - 303 lines):

Created complete module for parsing rustc JSON diagnostics:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RustcDiagnostic {
    pub message: String,
    pub code: Option<RustcCode>,
    pub level: String,  // "error", "warning", "note", "help"
    pub spans: Vec<RustcSpan>,
    pub children: Vec<RustcDiagnostic>,
    pub rendered: Option<String>,
}

impl RustcDiagnostic {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn from_json_lines(json_lines: &str) -> Result<Vec<Self>, serde_json::Error> {
        json_lines
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(serde_json::from_str)
            .collect()
    }

    pub fn primary_span(&self) -> Option<&RustcSpan> {
        self.spans.iter().find(|s| s.is_primary)
    }

    pub fn primary_position(&self) -> Option<(String, usize, usize)> {
        self.primary_span().map(|span| (span.file_name.clone(), span.line_start, span.column_start))
    }

    pub fn is_error(&self) -> bool {
        self.level == "error"
    }

    pub fn is_warning(&self) -> bool {
        self.level == "warning"
    }
}
```

**Key structures:**
- `RustcDiagnostic`: Top-level diagnostic with message, level, spans, children
- `RustcSpan`: Source location with file_name, byte/line/column positions, is_primary flag
- `RustcCode`: Error code (e.g., "E0425") with optional explanation
- `RustcSpanText`: Text snippets with highlight positions
- `RustcExpansion`: Macro expansion context

**2. Updated lib.rs** (`crates/oxur-comp/src/lib.rs:11-16`):

```rust
pub mod rustc_diagnostic;

pub use rustc_diagnostic::{RustcDiagnostic, RustcSpan};
```

Added module declaration and public exports for external use.

**3. Updated Cargo.toml** (`crates/oxur-comp/Cargo.toml:24`):

```toml
serde = { version = "1.0", features = ["derive"] }
serde_json.workspace = true
```

Added serde dependency with derive feature (serde_json was already present).

**4. Updated compile_with_rustc()** (`crates/oxur-comp/src/compiler.rs:48-83`):

```rust
fn compile_with_rustc(&self, source: &Path, output: &Path) -> Result<()> {
    let output_result = Command::new("rustc")
        .arg(source)
        .arg("-o")
        .arg(output)
        .arg("--error-format=json")
        .output()?;

    if !output_result.status.success() {
        // Parse JSON diagnostics from stderr
        let stderr = String::from_utf8_lossy(&output_result.stderr);
        let diagnostics =
            crate::RustcDiagnostic::from_json_lines(&stderr).unwrap_or_else(|_| vec![]);

        // Format error message with file:line:col positions
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

**Changes made:**
- Added `--error-format=json` flag to rustc invocation
- Changed from `.status()` to `.output()` to capture stderr
- Parse stderr with `RustcDiagnostic::from_json_lines()`
- Format error messages with file:line:col positions
- Filter to only show error-level diagnostics (not warnings/notes)

**Note:** This currently shows Rust source positions. Stage 1.10 will use the SourceMap to translate these back to Oxur source positions.

### Tests Added (6 new tests)

All tests in `rustc_diagnostic.rs` (lines 146-298):

**1. `test_parse_simple_error`** (lines 150-198):
- Parses a rustc error diagnostic from JSON
- Verifies message, level, error code extraction
- Checks span information (file, line, column)
- Verifies `is_error()` and `is_warning()` helpers

**2. `test_primary_position`** (lines 200-232):
- Tests `primary_position()` method
- Extracts (file, line, col) tuple from primary span
- Verifies 1-indexed line and column numbers

**3. `test_parse_warning`** (lines 234-248):
- Parses a warning-level diagnostic
- Verifies `is_warning()` returns true
- Verifies `is_error()` returns false

**4. `test_parse_multiple_diagnostics`** (lines 250-259):
- Tests `from_json_lines()` for multiple diagnostics
- Parses JSON lines format (one diagnostic per line)
- Verifies all diagnostics are parsed correctly

**5. `test_no_primary_span`** (lines 261-275):
- Tests diagnostics without primary span
- Verifies `primary_span()` returns None
- Verifies `primary_position()` returns None

**6. `test_error_code_extraction`** (lines 277-296):
- Tests error code parsing (e.g., "E0425")
- Verifies code and explanation extraction
- Tests optional explanation field

### Test Results

**Before Stage 1.9:**
- oxur-comp tests: 16 tests

**After Stage 1.9:**
- oxur-comp tests: **22 tests** (16 existing + 6 new)
- All tests passing: ✅
- Clippy clean: ✅
- Formatting correct: ✅

### Files Modified

1. **`crates/oxur-comp/src/rustc_diagnostic.rs`** (NEW)
   - Complete module with data structures and parsing
   - 6 comprehensive unit tests
   - Lines: 303 total

2. **`crates/oxur-comp/src/lib.rs`**
   - Added module declaration and exports
   - Lines changed: 4 added

3. **`crates/oxur-comp/Cargo.toml`**
   - Added serde dependency with derive feature
   - Lines changed: 1 added

4. **`crates/oxur-comp/src/compiler.rs`**
   - Updated `compile_with_rustc()` to use JSON format
   - Parse and format error messages with positions
   - Lines changed: ~35 modified

5. **`crates/design/dev/0020-chain-stage-1.9-implementation-plan.md`** (CREATED PREVIOUSLY)
   - Comprehensive implementation plan

6. **`crates/design/dev/0021-chain-stage-1.9-completion.md`** (THIS FILE)
   - Completion documentation

## Technical Notes

### rustc JSON Diagnostic Format

**Command:**
```bash
rustc --error-format=json source.rs
```

**Output format:**
- Multiple JSON objects (one per line)
- Each object is a complete diagnostic
- Stderr contains JSON lines
- Stdout remains for normal output

**Example JSON:**
```json
{
  "message": "cannot find value `x` in this scope",
  "code": {
    "code": "E0425",
    "explanation": null
  },
  "level": "error",
  "spans": [
    {
      "file_name": "test.rs",
      "byte_start": 42,
      "byte_end": 43,
      "line_start": 3,
      "line_end": 3,
      "column_start": 5,
      "column_end": 6,
      "is_primary": true,
      "text": [
        {
          "text": "    x",
          "highlight_start": 5,
          "highlight_end": 6
        }
      ],
      "label": "not found in this scope",
      "suggested_replacement": null,
      "suggestion_applicability": null,
      "expansion": null
    }
  ],
  "children": [],
  "rendered": null
}
```

### Position Indexing

**rustc conventions (confirmed via testing):**
- **Line numbers:** 1-indexed (first line is 1)
- **Column numbers:** 1-indexed (first column is 1)
- **Byte offsets:** 0-indexed

**Oxur conventions (matching):**
- **Line numbers:** 1-indexed
- **Column numbers:** 1-indexed
- **Byte offsets:** 0-indexed

This alignment simplifies error translation in Stage 1.10.

### Primary Span Identification

**Multiple spans per diagnostic:**
- Each diagnostic can have multiple spans
- Only one span has `is_primary: true`
- Primary span is the main error location
- Other spans provide context (e.g., related definitions)

**Helper methods:**
- `primary_span()`: Returns the primary span
- `primary_position()`: Extracts (file, line, col) from primary span

### Error Level Filtering

**Diagnostic levels:**
- `"error"`: Compilation errors (must fix)
- `"warning"`: Warnings (can ignore)
- `"note"`: Additional context
- `"help"`: Suggestions

**Current implementation:**
- `compile_with_rustc()` filters to only show errors
- Warnings and notes are parsed but not displayed
- Future enhancement could show all levels with color coding

### Stage 1.10 Preview

**How Stage 1.10 will use this:**
1. Parse rustc diagnostics (Stage 1.9 - done)
2. Extract Rust file:line:col from primary position
3. Look up Rust position in SourceMap's lowering mappings
4. Find corresponding Core NodeId
5. Look up Core NodeId in SourceMap's surface mappings
6. Find original Oxur file:line:col
7. Format error message with Oxur positions

**Example flow:**
```
rustc error: generated.rs:5:10: "cannot find value `x`"
    ↓ Stage 1.10 translation
    ↓ Look up rust position in lowerings
    ↓ Find Core NodeId 200
    ↓ Look up NodeId 200 in surface nodes
    ↓ Find original position
Oxur error: example.oxur:2:8: "cannot find value `x`"
```

## Success Criteria Met

✅ **RustcDiagnostic data structures defined with serde Deserialize**
✅ **from_json() and from_json_lines() parsing methods implemented**
✅ **primary_span() and primary_position() extraction methods implemented**
✅ **is_error() and is_warning() helper methods implemented**
✅ **6 comprehensive unit tests verify all functionality**
✅ **compile_with_rustc() updated to use JSON format**
✅ **Error messages include file:line:col positions**
✅ **All 22 tests passing (16 existing + 6 new)**
✅ **Clippy clean, formatting correct**
✅ **serde dependency added to Cargo.toml**

## Phase 1 Progress

### Week 1: Position Tracking in Parser ✅ COMPLETE
- Stage 1.1: Span types ✅
- Stage 1.2-1.4: SurfaceForm with Span, position tracking ✅
- Stage 1.5: Position tracking tests ✅

### Week 2: Mapping Chains Through Pipeline 🚧 IN PROGRESS
- Stage 1.6: SourceMap recording API ✅ (Already existed)
- Stage 1.7: Surface → Core mapping ✅
- Stage 1.8: Core → syn mapping ✅
- Stage 1.9: rustc diagnostic parser ✅ **COMPLETE**
- Stage 1.10: Error translator ⏳ (Next)
- Stage 1.11: Error translation tests ⏳

**Phase 1 Overall Progress:** **9/11 stages complete (82%)**

## Next Steps

**Stage 1.10:** Error translator (3 hours estimated)
- Implement ErrorTranslator struct
- Use SourceMap to translate Rust positions to Oxur positions
- Update compile_with_rustc() to use translator
- Translate both primary and secondary spans
- Format error messages with Oxur source positions

**Remaining Week 2 Tasks:**
- Stage 1.11: End-to-end error translation tests (2 hours)

**After Phase 1 Complete:**
- Phase 2: Stage 4 Integration (Oxur AST buffer zone)
- Phase 3: Core Forms Expansion
- Phase 5: Core Macros Library

## Related Documents

- Implementation plan: `crates/design/dev/0020-chain-stage-1.9-implementation-plan.md`
- Stage breakdown: `crates/design/dev/0012-pipeline-chain-completion-stages.md`
- Main plan: `crates/design/dev/0011-pipeline-chain-completion.md`
- Previous stage: Stage 1.8 (Core → syn mapping)
- Next stage: Stage 1.10 (Error translator)

---

**Completion Date:** 2026-01-12
**Time Spent:** ~1.5 hours (under 2h estimate)
**Quality:** All 22 tests pass, clippy clean, formatted
**Status:** Ready for Stage 1.10 (Error translator)
