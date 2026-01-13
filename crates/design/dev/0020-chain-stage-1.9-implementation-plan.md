# Stage 1.9 Implementation Plan: rustc Diagnostic Parser

**Date:** 2026-01-12
**Stage:** 1.9 from Phase 1 (Source Mapping Infrastructure)
**Deliverable:** Parse rustc JSON output to extract positions
**Estimated Time:** 2 hours
**Dependencies:** None (independent utility)

---

## Current State Analysis

### What Exists ✅

**Compiler Integration** (`crates/oxur-comp/src/compiler.rs`):
- `compile_with_rustc()` method invokes rustc
- Currently only checks exit status
- No diagnostic parsing

**Error Types** (`crates/oxur-comp/src/lib.rs`):
```rust
#[derive(Debug, Error)]
pub enum Error {
    #[error("Compilation failed: {0}")]
    Compile(String),
    // ...
}
```

### What's Missing 🔲

1. **rustc JSON diagnostic format structures** - Data types for rustc output
2. **Diagnostic parser** - Parse JSON output from rustc
3. **Position extraction** - Extract file:line:col from diagnostics
4. **Error severity handling** - Distinguish errors vs warnings
5. **Tests** - Parse sample rustc JSON output

---

## Design Decisions

### rustc JSON Format

**When to use JSON output:**
```bash
rustc --error-format=json source.rs
```

**Sample JSON output:**
```json
{
  "message": "cannot find value `x` in this scope",
  "code": {
    "code": "E0425",
    "explanation": "..."
  },
  "level": "error",
  "spans": [
    {
      "file_name": "generated.rs",
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
  "rendered": "error[E0425]: cannot find value `x` in this scope..."
}
```

**Key fields:**
- `message`: Error message text
- `level`: "error", "warning", "note", "help"
- `spans`: Array of source locations
- `spans[].file_name`: Source file path
- `spans[].line_start`, `line_end`: Line numbers (1-indexed)
- `spans[].column_start`, `column_end`: Column numbers (1-indexed)
- `spans[].is_primary`: Whether this is the main location

### Data Structures

**RustcDiagnostic:**
```rust
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RustcDiagnostic {
    pub message: String,
    pub code: Option<RustcCode>,
    pub level: String,
    pub spans: Vec<RustcSpan>,
    pub children: Vec<RustcDiagnostic>,
    pub rendered: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RustcCode {
    pub code: String,
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RustcSpan {
    pub file_name: String,
    pub byte_start: usize,
    pub byte_end: usize,
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub is_primary: bool,
    pub text: Vec<RustcSpanText>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RustcSpanText {
    pub text: String,
    pub highlight_start: usize,
    pub highlight_end: usize,
}
```

### Position Extraction Strategy

**Primary span extraction:**
```rust
pub fn primary_span(&self) -> Option<&RustcSpan> {
    self.spans.iter().find(|s| s.is_primary)
}

pub fn primary_position(&self) -> Option<(String, usize, usize)> {
    self.primary_span().map(|span| (
        span.file_name.clone(),
        span.line_start,
        span.column_start,
    ))
}
```

**Use case for error translation:**
```rust
if let Some((file, line, col)) = diagnostic.primary_position() {
    // Look up in SourceMap
    // Translate to Oxur source position
}
```

---

## Implementation Details

### File 1: Create `crates/oxur-comp/src/rustc_diagnostic.rs` (NEW)

**Location:** New file

```rust
//! rustc diagnostic parser
//!
//! Parses JSON diagnostic output from rustc to extract error positions.

use serde::Deserialize;

/// A diagnostic message from rustc
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RustcDiagnostic {
    /// The main error message
    pub message: String,

    /// Optional error code (e.g., E0425)
    pub code: Option<RustcCode>,

    /// Severity level: "error", "warning", "note", "help"
    pub level: String,

    /// Source code spans where the error occurred
    pub spans: Vec<RustcSpan>,

    /// Child diagnostics (notes, suggestions)
    pub children: Vec<RustcDiagnostic>,

    /// Rendered text output (optional)
    pub rendered: Option<String>,
}

impl RustcDiagnostic {
    /// Parse a rustc diagnostic from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Parse multiple diagnostics from JSON lines
    pub fn from_json_lines(json_lines: &str) -> Result<Vec<Self>, serde_json::Error> {
        json_lines
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line))
            .collect()
    }

    /// Get the primary span (the main location of the error)
    pub fn primary_span(&self) -> Option<&RustcSpan> {
        self.spans.iter().find(|s| s.is_primary)
    }

    /// Get the primary position as (file, line, column)
    pub fn primary_position(&self) -> Option<(String, usize, usize)> {
        self.primary_span().map(|span| {
            (
                span.file_name.clone(),
                span.line_start,
                span.column_start,
            )
        })
    }

    /// Check if this is an error (vs warning or note)
    pub fn is_error(&self) -> bool {
        self.level == "error"
    }

    /// Check if this is a warning
    pub fn is_warning(&self) -> bool {
        self.level == "warning"
    }
}

/// Error code from rustc
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RustcCode {
    /// Error code (e.g., "E0425")
    pub code: String,

    /// Long explanation text (optional)
    #[serde(default)]
    pub explanation: Option<String>,
}

/// A source code span in a rustc diagnostic
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RustcSpan {
    /// Source file path
    pub file_name: String,

    /// Byte offset start (0-indexed)
    pub byte_start: usize,

    /// Byte offset end (0-indexed)
    pub byte_end: usize,

    /// Line number start (1-indexed)
    pub line_start: usize,

    /// Line number end (1-indexed)
    pub line_end: usize,

    /// Column number start (1-indexed)
    pub column_start: usize,

    /// Column number end (1-indexed)
    pub column_end: usize,

    /// Whether this is the primary location
    pub is_primary: bool,

    /// Text snippets
    pub text: Vec<RustcSpanText>,

    /// Optional label text
    pub label: Option<String>,

    /// Optional suggested replacement
    #[serde(default)]
    pub suggested_replacement: Option<String>,

    /// Applicability of suggestion
    #[serde(default)]
    pub suggestion_applicability: Option<String>,

    /// Macro expansion context
    #[serde(default)]
    pub expansion: Option<Box<RustcExpansion>>,
}

/// Text snippet from a span
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RustcSpanText {
    /// The source text
    pub text: String,

    /// Start of highlight in text (1-indexed)
    pub highlight_start: usize,

    /// End of highlight in text (1-indexed)
    pub highlight_end: usize,
}

/// Macro expansion context (simplified)
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RustcExpansion {
    /// Span where the macro was expanded
    pub span: RustcSpan,

    /// Name of the macro
    pub macro_decl_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_error() {
        let json = r#"{
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
        }"#;

        let diagnostic = RustcDiagnostic::from_json(json).unwrap();
        assert_eq!(diagnostic.message, "cannot find value `x` in this scope");
        assert_eq!(diagnostic.level, "error");
        assert!(diagnostic.is_error());
        assert!(!diagnostic.is_warning());
        assert_eq!(diagnostic.spans.len(), 1);

        let span = &diagnostic.spans[0];
        assert_eq!(span.file_name, "test.rs");
        assert_eq!(span.line_start, 3);
        assert_eq!(span.column_start, 5);
        assert!(span.is_primary);
    }

    #[test]
    fn test_primary_position() {
        let json = r#"{
            "message": "test error",
            "code": null,
            "level": "error",
            "spans": [
                {
                    "file_name": "test.rs",
                    "byte_start": 0,
                    "byte_end": 1,
                    "line_start": 1,
                    "line_end": 1,
                    "column_start": 1,
                    "column_end": 2,
                    "is_primary": true,
                    "text": [],
                    "label": null,
                    "suggested_replacement": null,
                    "suggestion_applicability": null,
                    "expansion": null
                }
            ],
            "children": [],
            "rendered": null
        }"#;

        let diagnostic = RustcDiagnostic::from_json(json).unwrap();
        let (file, line, col) = diagnostic.primary_position().unwrap();
        assert_eq!(file, "test.rs");
        assert_eq!(line, 1);
        assert_eq!(col, 1);
    }

    #[test]
    fn test_parse_warning() {
        let json = r#"{
            "message": "unused variable",
            "code": null,
            "level": "warning",
            "spans": [],
            "children": [],
            "rendered": null
        }"#;

        let diagnostic = RustcDiagnostic::from_json(json).unwrap();
        assert!(diagnostic.is_warning());
        assert!(!diagnostic.is_error());
    }

    #[test]
    fn test_parse_multiple_diagnostics() {
        let json_lines = r#"{"message": "error 1", "code": null, "level": "error", "spans": [], "children": [], "rendered": null}
{"message": "error 2", "code": null, "level": "error", "spans": [], "children": [], "rendered": null}"#;

        let diagnostics = RustcDiagnostic::from_json_lines(json_lines).unwrap();
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].message, "error 1");
        assert_eq!(diagnostics[1].message, "error 2");
    }

    #[test]
    fn test_no_primary_span() {
        let json = r#"{
            "message": "note",
            "code": null,
            "level": "note",
            "spans": [],
            "children": [],
            "rendered": null
        }"#;

        let diagnostic = RustcDiagnostic::from_json(json).unwrap();
        assert!(diagnostic.primary_span().is_none());
        assert!(diagnostic.primary_position().is_none());
    }
}
```

### File 2: Update `crates/oxur-comp/src/lib.rs`

**Location:** Add module declaration

```rust
mod rustc_diagnostic;

pub use rustc_diagnostic::{RustcDiagnostic, RustcSpan};
```

### File 3: Update `crates/oxur-comp/Cargo.toml`

**Add serde dependency:**
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
# ... existing dependencies
```

### File 4: Update `crates/oxur-comp/src/compiler.rs` - Add JSON output

**Location:** Update `compile_with_rustc()` method

**Before:**
```rust
fn compile_with_rustc(&self, source: &Path, output: &Path) -> Result<()> {
    let status = Command::new("rustc")
        .arg(source)
        .arg("-o")
        .arg(output)
        .status()?;

    if !status.success() {
        return Err(Error::Compile(format!(
            "rustc failed with exit code: {:?}",
            status.code()
        )));
    }

    Ok(())
}
```

**After:**
```rust
fn compile_with_rustc(&self, source: &Path, output: &Path) -> Result<()> {
    let output_result = Command::new("rustc")
        .arg("--error-format=json")
        .arg(source)
        .arg("-o")
        .arg(output)
        .output()?;

    if !output_result.status.success() {
        // Parse diagnostics from stderr
        let stderr = String::from_utf8_lossy(&output_result.stderr);

        // Try to parse JSON diagnostics
        if let Ok(diagnostics) = crate::RustcDiagnostic::from_json_lines(&stderr) {
            // Format diagnostics for error message
            let errors: Vec<_> = diagnostics
                .iter()
                .filter(|d| d.is_error())
                .map(|d| {
                    if let Some((file, line, col)) = d.primary_position() {
                        format!("{}:{}:{}: {}", file, line, col, d.message)
                    } else {
                        d.message.clone()
                    }
                })
                .collect();

            if !errors.is_empty() {
                return Err(Error::Compile(format!(
                    "rustc failed with errors:\n{}",
                    errors.join("\n")
                )));
            }
        }

        // Fallback to simple error message
        return Err(Error::Compile(format!(
            "rustc failed with exit code: {:?}",
            output_result.status.code()
        )));
    }

    Ok(())
}
```

---

## Implementation Steps

1. **Create rustc_diagnostic.rs module** (45 min)
   - Define RustcDiagnostic struct
   - Define RustcSpan, RustcCode structs
   - Add serde Deserialize derives
   - Implement from_json() method
   - Implement primary_position() method

2. **Add module to lib.rs** (5 min)
   - Add mod declaration
   - Add pub use statements

3. **Update Cargo.toml** (5 min)
   - Add serde and serde_json dependencies

4. **Add comprehensive tests** (30 min)
   - test_parse_simple_error
   - test_primary_position
   - test_parse_warning
   - test_parse_multiple_diagnostics
   - test_no_primary_span

5. **Update compile_with_rustc()** (15 min)
   - Add --error-format=json flag
   - Use .output() instead of .status()
   - Parse stderr as JSON diagnostics
   - Format error messages with positions

6. **Run tests and verify** (15 min)
   - Run cargo test --package oxur-comp
   - Verify JSON parsing works
   - Test with actual rustc errors

7. **Code quality checks** (5 min)
   - Run cargo clippy
   - Run cargo fmt

**Total Estimated Time:** 120 minutes (~2 hours)

---

## Success Criteria

✅ RustcDiagnostic struct defined with serde support
✅ Can parse rustc JSON output successfully
✅ Can extract file:line:col from diagnostics
✅ Can distinguish errors vs warnings
✅ 5+ tests verify JSON parsing
✅ compile_with_rustc() uses JSON format
✅ Error messages include position information
✅ All tests passing
✅ Clippy clean, formatting correct

---

## Testing Strategy

**Unit Tests:**
1. Parse simple error with one span
2. Parse warning
3. Parse error with multiple spans
4. Extract primary position
5. Handle missing primary span
6. Parse multiple diagnostics (JSON lines)

**Integration Test:**
- Create invalid Rust file
- Compile with rustc --error-format=json
- Verify we can parse the output

---

## Notes

**rustc JSON Format:**
- Each diagnostic is a separate JSON object
- Multiple diagnostics are output as JSON lines (one per line)
- Must parse line-by-line, not as JSON array

**Position Indexing:**
- rustc uses 1-indexed line and column numbers
- Matches our Span and SourcePos conventions
- No conversion needed for Stage 1.10

**Future Enhancement (Stage 1.10):**
- Use primary_position() to look up in SourceMap
- Translate rustc positions back to Oxur source
- Generate Oxur error messages with original positions

---

## Related Documents

- Stage breakdown: `crates/design/dev/0012-pipeline-chain-completion-stages.md`
- Main plan: `crates/design/dev/0011-pipeline-chain-completion.md`
- Previous stage: Stage 1.8 (Core → syn mapping)
- Next stage: Stage 1.10 (Error translator)

---

**Plan Created:** 2026-01-12
**Estimated Completion:** Stage 1.9 implementation
