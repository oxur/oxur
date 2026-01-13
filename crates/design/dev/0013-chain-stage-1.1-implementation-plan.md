# Stage 1.1 Implementation Plan: Define Span and SourcePos Types

**Date:** 2026-01-12
**Stage:** 1.1 from Phase 1 (Source Mapping Infrastructure)
**Deliverable:** Span types defined in oxur-smap
**Estimated Time:** 1 hour
**Dependencies:** None

---

## Current State Analysis

### What Exists ✅

**oxur-smap crate** (well-established):
- ✅ `NodeId` - Unique identifier for AST nodes (complete with tests)
- ✅ `SourcePos` - Source position with file, line, column, length (complete with tests)
- ✅ `SourceMap` - Transformation chain tracking (complete with extensive tests)
- ✅ Zero-dependency design philosophy
- ✅ Comprehensive benchmarks showing <120ns lookup times

**Current SourcePos** (`crates/oxur-smap/src/source_pos.rs:1-115`):
```rust
pub struct SourcePos {
    pub file: String,
    pub line: u32,       // 1-indexed
    pub column: u32,     // 1-indexed
    pub length: u32,     // For error highlighting
}
```

**Properties:**
- Represents a **point** with a length
- Good for single-line errors
- Already has `.contains()`, `.end_column()` methods
- 100% test coverage

### What's Missing 🔲

**Span type** - Not present in oxur-smap

**Why we need Span:**
1. `SourcePos` represents a point + length (good for single-line)
2. `Span` represents explicit start/end (better for multi-line constructs)
3. Need to track AST nodes that span multiple lines (functions, blocks, etc.)
4. Allows more precise error reporting with context

### Current Position Types Across Crates

| Crate | Type | Purpose | Location |
|-------|------|---------|----------|
| oxur-ast | `Span` | Low-level byte offsets (lo, hi, ctxt) | `src/ast/span.rs:1-45` |
| oxur-lang | `Location` | Simple line/column for errors | `src/lib.rs:38-48` |
| oxur-smap | `SourcePos` | Point + length for error reporting | `src/source_pos.rs:1-115` |
| oxur-smap | **Span** | **MISSING** - Range with start/end | N/A |

---

## Design Decisions

### Span Type Design

**Option A: Span as Start/End Positions (RECOMMENDED)**
```rust
pub struct Span {
    pub file: String,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}
```

**Pros:**
- Explicit start and end (no ambiguity)
- Naturally represents multi-line constructs
- Can convert to SourcePos: `Span → SourcePos`
- Matches rustc and LSP conventions

**Cons:**
- Slightly more memory (5 fields vs 4)

**Option B: Span as Two SourcePos (NOT RECOMMENDED)**
```rust
pub struct Span {
    pub start: SourcePos,
    pub end: SourcePos,
}
```

**Pros:**
- Reuses existing type

**Cons:**
- `SourcePos` includes file twice (wasteful)
- `SourcePos` has length field (meaningless for end position)
- More memory overhead
- Awkward API

**Option C: Span as SourcePos + End (NOT RECOMMENDED)**
```rust
pub struct Span {
    pub start: SourcePos,
    pub end_line: u32,
    pub end_column: u32,
}
```

**Cons:**
- Inconsistent representation (start has length, end doesn't)
- Confusing API

**Decision: Use Option A** - Explicit start/end fields

### Design Principles (from oxur-smap)

Following existing patterns in oxur-smap:

1. **Zero dependencies** - Maintain oxur-smap as foundation crate
2. **1-indexed line/column** - Match rustc, LSP, and editor conventions
3. **Defensive programming** - Assert on invalid inputs (line=0, column=0)
4. **Comprehensive testing** - 100% coverage target
5. **Clear documentation** - Doc comments with examples

---

## Implementation Details

### File: `crates/oxur-smap/src/span.rs` (NEW FILE)

**Location:** Create new file
**Purpose:** Define `Span` type with start/end positions

```rust
/// A span representing a range in source code
///
/// Unlike `SourcePos` which represents a point with a length, `Span`
/// explicitly represents a range with start and end positions. This
/// is particularly useful for multi-line constructs like functions,
/// blocks, and expressions that span multiple lines.
///
/// # Examples
///
/// ```
/// use oxur_smap::Span;
///
/// // Single-line span
/// let span = Span::new(
///     "test.oxur".to_string(),
///     1, 5,   // start: line 1, column 5
///     1, 15,  // end: line 1, column 15
/// );
/// assert_eq!(span.num_lines(), 1);
/// assert_eq!(span.length_on_start_line(), 10);
///
/// // Multi-line span
/// let span = Span::new(
///     "test.oxur".to_string(),
///     1, 5,   // start: line 1, column 5
///     3, 10,  // end: line 3, column 10
/// );
/// assert_eq!(span.num_lines(), 3);
/// assert!(span.is_multi_line());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// Source file path (or "<repl>" for REPL input)
    pub file: String,

    /// Start line number (1-indexed)
    pub start_line: u32,

    /// Start column number (1-indexed)
    pub start_column: u32,

    /// End line number (1-indexed)
    pub end_line: u32,

    /// End column number (1-indexed, exclusive)
    ///
    /// Note: This is the column *after* the last character in the span.
    /// For example, if the span covers "hello", end_column points to
    /// the position after 'o'.
    pub end_column: u32,
}

impl Span {
    /// Create a new span
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `start_line` or `start_column` is 0 (must be 1-indexed)
    /// - `end_line` or `end_column` is 0 (must be 1-indexed)
    /// - `end_line` < `start_line` (invalid range)
    /// - `end_line` == `start_line` and `end_column` <= `start_column` (invalid range)
    ///
    /// # Examples
    ///
    /// ```
    /// use oxur_smap::Span;
    ///
    /// let span = Span::new("file.oxur".to_string(), 1, 5, 1, 15);
    /// assert_eq!(span.start_line, 1);
    /// assert_eq!(span.end_column, 15);
    /// ```
    pub fn new(
        file: String,
        start_line: u32,
        start_column: u32,
        end_line: u32,
        end_column: u32,
    ) -> Self {
        assert!(start_line > 0, "Line numbers are 1-indexed");
        assert!(start_column > 0, "Column numbers are 1-indexed");
        assert!(end_line > 0, "Line numbers are 1-indexed");
        assert!(end_column > 0, "Column numbers are 1-indexed");
        assert!(
            end_line >= start_line,
            "End line must be >= start line"
        );
        if end_line == start_line {
            assert!(
                end_column > start_column,
                "End column must be > start column on same line"
            );
        }

        Self { file, start_line, start_column, end_line, end_column }
    }

    /// Create a span for REPL input
    ///
    /// # Examples
    ///
    /// ```
    /// use oxur_smap::Span;
    ///
    /// let span = Span::repl(1, 1, 1, 10);
    /// assert_eq!(span.file, "<repl>");
    /// ```
    pub fn repl(
        start_line: u32,
        start_column: u32,
        end_line: u32,
        end_column: u32,
    ) -> Self {
        Self::new("<repl>".to_string(), start_line, start_column, end_line, end_column)
    }

    /// Check if this span is on a single line
    ///
    /// # Examples
    ///
    /// ```
    /// use oxur_smap::Span;
    ///
    /// let single = Span::new("file.oxur".to_string(), 1, 5, 1, 15);
    /// assert!(single.is_single_line());
    ///
    /// let multi = Span::new("file.oxur".to_string(), 1, 5, 3, 10);
    /// assert!(!multi.is_single_line());
    /// ```
    pub fn is_single_line(&self) -> bool {
        self.start_line == self.end_line
    }

    /// Check if this span spans multiple lines
    pub fn is_multi_line(&self) -> bool {
        !self.is_single_line()
    }

    /// Get the number of lines this span covers
    ///
    /// # Examples
    ///
    /// ```
    /// use oxur_smap::Span;
    ///
    /// let span = Span::new("file.oxur".to_string(), 1, 5, 3, 10);
    /// assert_eq!(span.num_lines(), 3);
    /// ```
    pub fn num_lines(&self) -> u32 {
        self.end_line - self.start_line + 1
    }

    /// Get the length on the start line (for single-line spans)
    ///
    /// For multi-line spans, this returns the length from start_column
    /// to the end of the start line (which is undefined without source text).
    ///
    /// # Examples
    ///
    /// ```
    /// use oxur_smap::Span;
    ///
    /// let span = Span::new("file.oxur".to_string(), 1, 5, 1, 15);
    /// assert_eq!(span.length_on_start_line(), 10);
    /// ```
    pub fn length_on_start_line(&self) -> u32 {
        if self.is_single_line() {
            self.end_column - self.start_column
        } else {
            // For multi-line, we don't know the line length without source text
            // This is a limitation - caller should check is_single_line() first
            0
        }
    }

    /// Check if this span contains another span
    ///
    /// # Examples
    ///
    /// ```
    /// use oxur_smap::Span;
    ///
    /// let outer = Span::new("file.oxur".to_string(), 1, 5, 3, 20);
    /// let inner = Span::new("file.oxur".to_string(), 2, 1, 2, 10);
    ///
    /// assert!(outer.contains(&inner));
    /// assert!(!inner.contains(&outer));
    /// ```
    pub fn contains(&self, other: &Span) -> bool {
        if self.file != other.file {
            return false;
        }

        // Check start is before or equal
        let start_ok = other.start_line > self.start_line
            || (other.start_line == self.start_line && other.start_column >= self.start_column);

        // Check end is after or equal
        let end_ok = other.end_line < self.end_line
            || (other.end_line == self.end_line && other.end_column <= self.end_column);

        start_ok && end_ok
    }

    /// Merge two spans into a single span covering both
    ///
    /// # Panics
    ///
    /// Panics if the spans are from different files.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxur_smap::Span;
    ///
    /// let span1 = Span::new("file.oxur".to_string(), 1, 5, 1, 10);
    /// let span2 = Span::new("file.oxur".to_string(), 1, 15, 2, 5);
    /// let merged = span1.merge(&span2);
    ///
    /// assert_eq!(merged.start_line, 1);
    /// assert_eq!(merged.start_column, 5);
    /// assert_eq!(merged.end_line, 2);
    /// assert_eq!(merged.end_column, 5);
    /// ```
    pub fn merge(&self, other: &Span) -> Span {
        assert_eq!(
            self.file, other.file,
            "Cannot merge spans from different files"
        );

        let (start_line, start_column) = if self.start_line < other.start_line
            || (self.start_line == other.start_line && self.start_column < other.start_column)
        {
            (self.start_line, self.start_column)
        } else {
            (other.start_line, other.start_column)
        };

        let (end_line, end_column) = if self.end_line > other.end_line
            || (self.end_line == other.end_line && self.end_column > other.end_column)
        {
            (self.end_line, self.end_column)
        } else {
            (other.end_line, other.end_column)
        };

        Span::new(self.file.clone(), start_line, start_column, end_line, end_column)
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_single_line() {
            write!(
                f,
                "{}:{}:{}-{}",
                self.file, self.start_line, self.start_column, self.end_column
            )
        } else {
            write!(
                f,
                "{}:{}:{}-{}:{}",
                self.file, self.start_line, self.start_column, self.end_line, self.end_column
            )
        }
    }
}
```

### File: `crates/oxur-smap/src/span.rs` - Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_basic() {
        let span = Span::new("test.oxur".to_string(), 1, 5, 1, 15);
        assert_eq!(span.file, "test.oxur");
        assert_eq!(span.start_line, 1);
        assert_eq!(span.start_column, 5);
        assert_eq!(span.end_line, 1);
        assert_eq!(span.end_column, 15);
    }

    #[test]
    fn test_span_display_single_line() {
        let span = Span::new("test.oxur".to_string(), 10, 5, 10, 15);
        assert_eq!(format!("{}", span), "test.oxur:10:5-15");
    }

    #[test]
    fn test_span_display_multi_line() {
        let span = Span::new("test.oxur".to_string(), 1, 5, 3, 10);
        assert_eq!(format!("{}", span), "test.oxur:1:5-3:10");
    }

    #[test]
    fn test_span_repl() {
        let span = Span::repl(1, 1, 1, 20);
        assert_eq!(span.file, "<repl>");
        assert_eq!(span.start_line, 1);
    }

    #[test]
    fn test_span_is_single_line() {
        let single = Span::new("test.oxur".to_string(), 1, 5, 1, 15);
        assert!(single.is_single_line());
        assert!(!single.is_multi_line());

        let multi = Span::new("test.oxur".to_string(), 1, 5, 3, 10);
        assert!(!multi.is_single_line());
        assert!(multi.is_multi_line());
    }

    #[test]
    fn test_span_num_lines() {
        let span1 = Span::new("test.oxur".to_string(), 1, 5, 1, 15);
        assert_eq!(span1.num_lines(), 1);

        let span2 = Span::new("test.oxur".to_string(), 1, 5, 3, 10);
        assert_eq!(span2.num_lines(), 3);

        let span3 = Span::new("test.oxur".to_string(), 5, 1, 10, 1);
        assert_eq!(span3.num_lines(), 6);
    }

    #[test]
    fn test_span_length_on_start_line() {
        let single = Span::new("test.oxur".to_string(), 1, 5, 1, 15);
        assert_eq!(single.length_on_start_line(), 10);

        let multi = Span::new("test.oxur".to_string(), 1, 5, 3, 10);
        // For multi-line, returns 0 (undefined without source text)
        assert_eq!(multi.length_on_start_line(), 0);
    }

    #[test]
    fn test_span_contains_same_line() {
        let outer = Span::new("test.oxur".to_string(), 1, 5, 1, 20);
        let inner = Span::new("test.oxur".to_string(), 1, 10, 1, 15);
        let before = Span::new("test.oxur".to_string(), 1, 1, 1, 4);
        let after = Span::new("test.oxur".to_string(), 1, 21, 1, 25);

        assert!(outer.contains(&inner));
        assert!(!outer.contains(&before));
        assert!(!outer.contains(&after));
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn test_span_contains_multi_line() {
        let outer = Span::new("test.oxur".to_string(), 1, 5, 5, 20);
        let inner = Span::new("test.oxur".to_string(), 2, 1, 3, 10);
        let overlapping = Span::new("test.oxur".to_string(), 1, 1, 2, 10);

        assert!(outer.contains(&inner));
        assert!(!outer.contains(&overlapping)); // Starts before outer
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn test_span_contains_different_files() {
        let span1 = Span::new("file1.oxur".to_string(), 1, 5, 1, 20);
        let span2 = Span::new("file2.oxur".to_string(), 1, 10, 1, 15);

        assert!(!span1.contains(&span2));
    }

    #[test]
    fn test_span_merge_same_line() {
        let span1 = Span::new("test.oxur".to_string(), 1, 5, 1, 10);
        let span2 = Span::new("test.oxur".to_string(), 1, 15, 1, 20);
        let merged = span1.merge(&span2);

        assert_eq!(merged.start_line, 1);
        assert_eq!(merged.start_column, 5);
        assert_eq!(merged.end_line, 1);
        assert_eq!(merged.end_column, 20);
    }

    #[test]
    fn test_span_merge_multi_line() {
        let span1 = Span::new("test.oxur".to_string(), 1, 5, 2, 10);
        let span2 = Span::new("test.oxur".to_string(), 3, 1, 4, 5);
        let merged = span1.merge(&span2);

        assert_eq!(merged.start_line, 1);
        assert_eq!(merged.start_column, 5);
        assert_eq!(merged.end_line, 4);
        assert_eq!(merged.end_column, 5);
    }

    #[test]
    fn test_span_merge_overlapping() {
        let span1 = Span::new("test.oxur".to_string(), 1, 5, 2, 10);
        let span2 = Span::new("test.oxur".to_string(), 2, 1, 3, 5);
        let merged = span1.merge(&span2);

        assert_eq!(merged.start_line, 1);
        assert_eq!(merged.start_column, 5);
        assert_eq!(merged.end_line, 3);
        assert_eq!(merged.end_column, 5);
    }

    #[test]
    fn test_span_merge_reversed() {
        let span1 = Span::new("test.oxur".to_string(), 3, 1, 4, 5);
        let span2 = Span::new("test.oxur".to_string(), 1, 5, 2, 10);
        let merged = span1.merge(&span2);

        // Should produce same result regardless of order
        assert_eq!(merged.start_line, 1);
        assert_eq!(merged.start_column, 5);
        assert_eq!(merged.end_line, 4);
        assert_eq!(merged.end_column, 5);
    }

    #[test]
    #[should_panic(expected = "Cannot merge spans from different files")]
    fn test_span_merge_different_files() {
        let span1 = Span::new("file1.oxur".to_string(), 1, 5, 1, 10);
        let span2 = Span::new("file2.oxur".to_string(), 1, 15, 1, 20);
        span1.merge(&span2);
    }

    #[test]
    #[should_panic(expected = "Line numbers are 1-indexed")]
    fn test_span_zero_start_line() {
        Span::new("test.oxur".to_string(), 0, 1, 1, 10);
    }

    #[test]
    #[should_panic(expected = "Column numbers are 1-indexed")]
    fn test_span_zero_start_column() {
        Span::new("test.oxur".to_string(), 1, 0, 1, 10);
    }

    #[test]
    #[should_panic(expected = "Line numbers are 1-indexed")]
    fn test_span_zero_end_line() {
        Span::new("test.oxur".to_string(), 1, 1, 0, 10);
    }

    #[test]
    #[should_panic(expected = "Column numbers are 1-indexed")]
    fn test_span_zero_end_column() {
        Span::new("test.oxur".to_string(), 1, 1, 1, 0);
    }

    #[test]
    #[should_panic(expected = "End line must be >= start line")]
    fn test_span_end_before_start_line() {
        Span::new("test.oxur".to_string(), 5, 1, 3, 10);
    }

    #[test]
    #[should_panic(expected = "End column must be > start column on same line")]
    fn test_span_end_before_start_column() {
        Span::new("test.oxur".to_string(), 1, 10, 1, 5);
    }

    #[test]
    #[should_panic(expected = "End column must be > start column on same line")]
    fn test_span_equal_positions() {
        Span::new("test.oxur".to_string(), 1, 10, 1, 10);
    }
}
```

### File: `crates/oxur-smap/src/lib.rs` - Update

**Changes needed:**
1. Add `mod span;` declaration
2. Add `pub use span::Span;` to public API
3. Update module documentation example

```rust
// At line 42, add:
mod span;

// At line 47-49, update to:
pub use node_id::{new_node_id, NodeId, NodeIdGenerator};
pub use source_map::{LookupStats, SourceMap, SourceMapStats};
pub use source_pos::SourcePos;
pub use span::Span;  // NEW

// Update module documentation example (lines 16-40) to show Span usage:
//! # Example
//!
//! ```
//! use oxur_smap::{SourceMap, new_node_id, SourcePos, Span};
//!
//! let mut map = SourceMap::new();
//!
//! // Parser creates surface node with span
//! let surface = new_node_id();
//! let span = Span::repl(1, 1, 1, 10);
//! // Convert span to SourcePos for recording
//! let pos = SourcePos::repl(span.start_line, span.start_column,
//!                           span.length_on_start_line());
//! map.record_surface_node(surface, pos);
//!
//! // Expander creates core node
//! let core = new_node_id();
//! map.record_expansion(surface, core);
//!
//! // Lowering creates rust node
//! let rust = new_node_id();
//! map.record_lowering(core, rust);
//!
//! // Error translator looks up original position
//! let original = map.lookup(&rust).unwrap();
//! assert_eq!(original.line, 1);
//! assert_eq!(original.column, 1);
//! ```
```

### File: `crates/oxur-smap/Cargo.toml` - No Changes

No changes needed - maintaining zero-dependency design.

---

## Relationship Between Span and SourcePos

### Conversion: Span → SourcePos

For **single-line** spans, we can convert to SourcePos:
```rust
impl From<&Span> for SourcePos {
    fn from(span: &Span) -> Self {
        assert!(span.is_single_line(), "Can only convert single-line Span to SourcePos");
        SourcePos::new(
            span.file.clone(),
            span.start_line,
            span.start_column,
            span.length_on_start_line(),
        )
    }
}
```

**Add this to `crates/oxur-smap/src/span.rs`** after the main Span implementation.

### When to Use Each

| Use Case | Type | Rationale |
|----------|------|-----------|
| Error highlighting (single line) | `SourcePos` | Has length for underlining |
| AST node tracking | `Span` | Nodes can span multiple lines |
| Function definitions | `Span` | Multi-line constructs |
| Token positions | `SourcePos` | Tokens are typically single-line |
| Macro expansions | `Span` | Can expand across lines |
| Recording in SourceMap | `SourcePos` | Current API uses SourcePos |

---

## Testing Strategy

### Unit Tests (in `span.rs`)

**Coverage goals: 100%**

1. ✅ Basic construction and field access
2. ✅ Display formatting (single-line and multi-line)
3. ✅ REPL span creation
4. ✅ Single-line vs multi-line detection
5. ✅ Line counting
6. ✅ Length calculation
7. ✅ Contains checking (same-line, multi-line, different files)
8. ✅ Span merging (same-line, multi-line, overlapping, reversed)
9. ✅ Invalid inputs (zero line/column, reversed ranges, equal positions)
10. ✅ Different file handling
11. ✅ Span → SourcePos conversion (single-line only)

### Integration Tests (future stages)

Will be tested in Stage 1.2 when integrating with SurfaceForm.

---

## Success Criteria

1. ✅ `Span` type compiles without errors
2. ✅ All unit tests pass (100% coverage on new code)
3. ✅ No clippy warnings
4. ✅ Documentation compiles and examples work
5. ✅ Zero new dependencies added
6. ✅ Public API exported from lib.rs
7. ✅ Follows existing code patterns in oxur-smap

---

## Implementation Steps

### Step 1: Create span.rs File (30 min)
1. Create `crates/oxur-smap/src/span.rs`
2. Implement `Span` struct with all fields
3. Implement `new()` and `repl()` constructors with assertions
4. Implement utility methods:
   - `is_single_line()`, `is_multi_line()`
   - `num_lines()`
   - `length_on_start_line()`
   - `contains()`
   - `merge()`
5. Implement `Display` trait
6. Implement `From<&Span> for SourcePos`

### Step 2: Write Tests (20 min)
1. Write all unit tests listed above
2. Ensure 100% coverage
3. Test edge cases and error conditions

### Step 3: Update lib.rs (5 min)
1. Add `mod span;`
2. Add `pub use span::Span;`
3. Update documentation example

### Step 4: Verify (5 min)
1. Run `cargo test --package oxur-smap`
2. Run `cargo clippy --package oxur-smap`
3. Run `cargo doc --package oxur-smap --open`
4. Verify all success criteria

---

## Next Stage Preparation

**Stage 1.2** will:
- Update `SurfaceForm` to include `Span` fields
- Update parser to track and record spans
- This stage provides the foundation types needed

---

## Notes

- **Design choice:** Span and SourcePos are **separate types** with different purposes
- **Memory trade-off:** Span uses 5 fields (40+ bytes) vs SourcePos's 4 fields, but provides more precision
- **Zero dependencies:** Maintains oxur-smap's philosophy as a foundation crate
- **1-indexed:** Follows Rust, LSP, and editor conventions
- **Defensive:** Assertions catch bugs early in development

---

## Estimated Time Breakdown

| Task | Time |
|------|------|
| Create span.rs with implementation | 30 min |
| Write comprehensive tests | 20 min |
| Update lib.rs and documentation | 5 min |
| Verify and test | 5 min |
| **Total** | **60 min (1 hour)** |

---

## References

- **ODD-0013:** Pipeline chain architecture specification
- **Stage breakdown:** `crates/design/dev/0011-pipeline-chain-completion-stages.md`
- **Existing code:** `crates/oxur-smap/src/source_pos.rs` (pattern to follow)
- **LSP Spec:** https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#position
