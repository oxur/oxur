# Stage 1.2 Implementation Plan: SurfaceForm with Span

**Date:** 2026-01-12
**Stage:** 1.2 from Phase 1 (Source Mapping Infrastructure)
**Deliverable:** Update all SurfaceForm variants to include Span
**Estimated Time:** 2 hours
**Dependencies:** Stage 1.1 (Span types) ✅

---

## Current State Analysis

### What Exists ✅

**SurfaceForm** (`crates/oxur-lang/src/parser.rs:147-153`):
```rust
#[derive(Debug, Clone)]
pub enum SurfaceForm {
    Symbol(String),
    Number(i64),
    String(String),
    List(Vec<SurfaceForm>),
}
```

**Current characteristics:**
- ✅ Tuple variants (single unnamed field)
- ❌ No position tracking
- ❌ No Span information
- ✅ Used by Parser to create AST
- ✅ Consumed by Expander to produce CoreForm

**Parser Structure** (`crates/oxur-lang/src/parser.rs:8-14`):
```rust
pub struct Parser {
    source: String,
    position: usize,  // Byte offset only, not line/column
}
```

**Parse Methods:**
- `parse()` - Main entry point (line 22)
- `parse_form()` - Dispatches to specific parsers (line 36)
- `parse_list()` - Parses lists (line 53)
- `parse_string()` - Parses strings (line 75)
- `parse_number()` - Parses numbers (line 93)
- `parse_symbol()` - Parses symbols (line 112)

**Expander Usage** (`crates/oxur-lang/src/expander.rs:32-60`):
- Pattern matches on SurfaceForm variants
- Extracts values from tuple variants
- Creates CoreForm with NodeIds
- **39 locations** where SurfaceForm is pattern matched

**Dependencies** (`crates/oxur-lang/Cargo.toml:11`):
- ✅ Already depends on `oxur-smap`

**Test Coverage:**
- 11 parser tests (lines 156-296)
- 8 expander tests (lines 134-247)
- **19 total tests** that will need updating

### What's Missing 🔲

1. **Span fields in SurfaceForm** - Need to add Span to each variant
2. **Line/column tracking in Parser** - Need to track more than just byte offset
3. **Span recording during parsing** - Need to capture start/end positions
4. **Updated pattern matches** - 39 locations in expander need updating
5. **Updated tests** - 19 tests need to work with new structure

---

## Design Decisions

### SurfaceForm Variant Design

**Option A: Struct Variants with Named Fields (RECOMMENDED)**

```rust
#[derive(Debug, Clone)]
pub enum SurfaceForm {
    Symbol {
        span: Span,
        name: String,
    },
    Number {
        span: Span,
        value: i64,
    },
    String {
        span: Span,
        value: String,
    },
    List {
        span: Span,
        elements: Vec<SurfaceForm>,
    },
}
```

**Pros:**
- Clear field names (no ambiguity)
- Easy to extend later (can add more fields)
- Self-documenting
- Matches CoreForm style (which already uses struct variants with NodeId)
- Pattern matching is explicit: `Symbol { span, name } => ...`

**Cons:**
- More verbose pattern matching
- Breaks existing code (but that's unavoidable)

**Option B: Keep Tuple Variants, Add Span as First Field (NOT RECOMMENDED)**

```rust
pub enum SurfaceForm {
    Symbol(Span, String),
    Number(Span, i64),
    String(Span, String),
    List(Span, Vec<SurfaceForm>),
}
```

**Pros:**
- Slightly less verbose

**Cons:**
- Unclear which field is which
- Easy to accidentally swap fields
- Harder to extend
- Inconsistent with CoreForm style
- Pattern matching is positional: `Symbol(span, name) => ...`

**Decision: Use Option A (Struct Variants)**

**Rationale:**
1. Consistency with CoreForm (both use struct variants)
2. Future-proofing (easy to add more fields)
3. Clarity (named fields are self-documenting)
4. Safety (can't accidentally swap fields)

### Parser Position Tracking

**Current:**
```rust
struct Parser {
    source: String,
    position: usize,  // Byte offset only
}
```

**Need to add:**
```rust
struct Parser {
    source: String,
    position: usize,    // Byte offset
    line: usize,        // Current line (1-indexed)
    column: usize,      // Current column (1-indexed)
}
```

**Plus helper methods:**
- `current_position() -> (usize, usize)` - Returns (line, column)
- `advance()` - Updates position, line, column
- `mark_position()` - Captures start position for span
- `make_span(start_line, start_column) -> Span` - Creates span from start to current

---

## Implementation Details

### File 1: `crates/oxur-lang/src/parser.rs` - Update SurfaceForm

**Location:** Lines 147-153

**Before:**
```rust
#[derive(Debug, Clone)]
pub enum SurfaceForm {
    Symbol(String),
    Number(i64),
    String(String),
    List(Vec<SurfaceForm>),
}
```

**After:**
```rust
use oxur_smap::Span;

/// Surface Forms - parsed S-expressions before macro expansion
///
/// Each variant includes a Span tracking its source location for
/// error reporting and debugging.
#[derive(Debug, Clone)]
pub enum SurfaceForm {
    /// A symbol (identifier, operator, etc.)
    Symbol {
        span: Span,
        name: String,
    },

    /// A numeric literal
    Number {
        span: Span,
        value: i64,
    },

    /// A string literal
    String {
        span: Span,
        value: String,
    },

    /// A list (parenthesized expression)
    List {
        span: Span,
        elements: Vec<SurfaceForm>,
    },
}
```

**Add helper method:**
```rust
impl SurfaceForm {
    /// Get the span of this surface form
    pub fn span(&self) -> &Span {
        match self {
            SurfaceForm::Symbol { span, .. } => span,
            SurfaceForm::Number { span, .. } => span,
            SurfaceForm::String { span, .. } => span,
            SurfaceForm::List { span, .. } => span,
        }
    }
}
```

### File 2: `crates/oxur-lang/src/parser.rs` - Update Parser Structure

**Location:** Lines 8-14

**Before:**
```rust
pub struct Parser {
    #[allow(dead_code)]
    source: String,
    #[allow(dead_code)]
    position: usize,
}
```

**After:**
```rust
pub struct Parser {
    source: String,
    position: usize,    // Byte offset in source
    line: usize,        // Current line (1-indexed)
    column: usize,      // Current column (1-indexed)
    filename: String,   // Source filename (or "<repl>")
}
```

### File 3: `crates/oxur-lang/src/parser.rs` - Update Parser::new()

**Location:** Line 17-19

**Before:**
```rust
pub fn new(source: String) -> Self {
    Self { source, position: 0 }
}
```

**After:**
```rust
pub fn new(source: String) -> Self {
    Self {
        source,
        position: 0,
        line: 1,         // 1-indexed
        column: 1,       // 1-indexed
        filename: "<repl>".to_string(),
    }
}

/// Create a parser for a named file
pub fn new_file(source: String, filename: String) -> Self {
    Self {
        source,
        position: 0,
        line: 1,
        column: 1,
        filename,
    }
}
```

### File 4: `crates/oxur-lang/src/parser.rs` - Add Position Helpers

**Location:** After line 143 (before `is_at_end()`)

```rust
/// Get current position as (line, column) tuple
fn current_pos(&self) -> (u32, u32) {
    (self.line as u32, self.column as u32)
}

/// Mark current position for span tracking
fn mark_position(&self) -> (u32, u32) {
    self.current_pos()
}

/// Create a span from start position to current position
fn make_span(&self, start_line: u32, start_column: u32) -> Span {
    let (end_line, end_column) = self.current_pos();
    Span::new(
        self.filename.clone(),
        start_line,
        start_column,
        end_line,
        end_column,
    )
}
```

### File 5: `crates/oxur-lang/src/parser.rs` - Update advance()

**Location:** Lines 131-133

**Before:**
```rust
fn advance(&mut self) {
    self.position += 1;
}
```

**After:**
```rust
fn advance(&mut self) {
    if self.position < self.source.len() {
        let ch = self.current_char();
        self.position += 1;

        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }
}
```

### File 6: `crates/oxur-lang/src/parser.rs` - Update Parse Methods

**parse_list()** - Lines 53-73

**Before:**
```rust
fn parse_list(&mut self) -> Result<SurfaceForm> {
    self.advance(); // consume '('
    let mut elements = Vec::new();

    loop {
        self.skip_whitespace();
        if self.is_at_end() {
            return Err(crate::Error::Syntax("Unclosed list".to_string()));
        }
        if self.current_char() == ')' {
            self.advance(); // consume ')'
            break;
        }
        elements.push(self.parse_form()?);
    }

    Ok(SurfaceForm::List(elements))
}
```

**After:**
```rust
fn parse_list(&mut self) -> Result<SurfaceForm> {
    let (start_line, start_column) = self.mark_position();

    self.advance(); // consume '('
    let mut elements = Vec::new();

    loop {
        self.skip_whitespace();
        if self.is_at_end() {
            return Err(crate::Error::Syntax("Unclosed list".to_string()));
        }
        if self.current_char() == ')' {
            self.advance(); // consume ')'
            break;
        }
        elements.push(self.parse_form()?);
    }

    let span = self.make_span(start_line, start_column);
    Ok(SurfaceForm::List { span, elements })
}
```

**parse_string()** - Lines 75-91

**After:**
```rust
fn parse_string(&mut self) -> Result<SurfaceForm> {
    let (start_line, start_column) = self.mark_position();

    self.advance(); // consume opening '"'
    let start = self.position;

    while !self.is_at_end() && self.current_char() != '"' {
        self.advance();
    }

    if self.is_at_end() {
        return Err(crate::Error::Syntax("Unclosed string".to_string()));
    }

    let value = self.source[start..self.position].to_string();
    self.advance(); // consume closing '"'

    let span = self.make_span(start_line, start_column);
    Ok(SurfaceForm::String { span, value })
}
```

**parse_number()** - Lines 93-110

**After:**
```rust
fn parse_number(&mut self) -> Result<SurfaceForm> {
    let (start_line, start_column) = self.mark_position();
    let start = self.position;

    if self.current_char() == '-' {
        self.advance();
    }

    while !self.is_at_end() && self.current_char().is_ascii_digit() {
        self.advance();
    }

    let num_str = &self.source[start..self.position];
    let value = num_str
        .parse::<i64>()
        .map_err(|_| crate::Error::Syntax(format!("Invalid number: {}", num_str)))?;

    let span = self.make_span(start_line, start_column);
    Ok(SurfaceForm::Number { span, value })
}
```

**parse_symbol()** - Lines 112-121

**After:**
```rust
fn parse_symbol(&mut self) -> Result<SurfaceForm> {
    let (start_line, start_column) = self.mark_position();
    let start = self.position;

    while !self.is_at_end() && self.is_symbol_char(self.current_char()) {
        self.advance();
    }

    let name = self.source[start..self.position].to_string();
    let span = self.make_span(start_line, start_column);
    Ok(SurfaceForm::Symbol { span, name })
}
```

### File 7: `crates/oxur-lang/src/parser.rs` - Update Tests

**All tests** (lines 156-296) need updating to use struct variants.

**Pattern to follow:**

**Before:**
```rust
if let SurfaceForm::Symbol(s) = &form {
    assert_eq!(s, "test");
}
```

**After:**
```rust
if let SurfaceForm::Symbol { name, .. } = &form {
    assert_eq!(name, "test");
}
```

**List of tests to update (11 tests):**
1. `test_surface_form_symbol` (line 174)
2. `test_surface_form_number` (line 183)
3. `test_surface_form_string` (line 192)
4. `test_surface_form_list` (line 201)
5. `test_parse_hello_world` (line 210)
6. `test_parse_simple_list` (line 234)
7. `test_parse_string` (line 250)
8. `test_parse_number` (line 266)
9. `test_parse_symbol` (line 282)
10. `test_parser_creation` (line 160) - No changes needed
11. `test_parse_empty` (line 166) - No changes needed

### File 8: `crates/oxur-lang/src/expander.rs` - Update Pattern Matches

**39 locations** where SurfaceForm is matched need updating.

**Pattern to follow:**

**Before:**
```rust
SurfaceForm::Symbol(name) => { ... }
```

**After:**
```rust
SurfaceForm::Symbol { name, .. } => { ... }
```

**Key locations:**
1. `expand_form()` - Lines 34, 38, 42, 46, 49
2. `expand_deffn()` - Lines 74, 80, 83
3. Test cases - Lines 212, 225, 238

**Example - expand_form() update:**

**Before:**
```rust
fn expand_form(&mut self, form: SurfaceForm) -> Result<CoreForm> {
    match form {
        SurfaceForm::Symbol(name) => {
            let id = oxur_smap::new_node_id();
            Ok(CoreForm::Symbol { id, name })
        }
        SurfaceForm::Number(value) => {
            let id = oxur_smap::new_node_id();
            Ok(CoreForm::Number { id, value })
        }
        SurfaceForm::String(value) => {
            let id = oxur_smap::new_node_id();
            Ok(CoreForm::String { id, value })
        }
        SurfaceForm::List(elements) => {
            // ...
        }
    }
}
```

**After:**
```rust
fn expand_form(&mut self, form: SurfaceForm) -> Result<CoreForm> {
    match form {
        SurfaceForm::Symbol { name, .. } => {
            let id = oxur_smap::new_node_id();
            Ok(CoreForm::Symbol { id, name })
        }
        SurfaceForm::Number { value, .. } => {
            let id = oxur_smap::new_node_id();
            Ok(CoreForm::Number { id, value })
        }
        SurfaceForm::String { value, .. } => {
            let id = oxur_smap::new_node_id();
            Ok(CoreForm::String { id, value })
        }
        SurfaceForm::List { elements, .. } => {
            // ...
        }
    }
}
```

---

## Testing Strategy

### Unit Tests (in `parser.rs`)

**Coverage goals: Maintain existing coverage**

**Tests that need updating (11 parser tests):**

1. **test_surface_form_symbol** - Update pattern match
2. **test_surface_form_number** - Update pattern match
3. **test_surface_form_string** - Update pattern match
4. **test_surface_form_list** - Update pattern match
5. **test_parse_hello_world** - Update all pattern matches
6. **test_parse_simple_list** - Update pattern matches
7. **test_parse_string** - Update pattern match
8. **test_parse_number** - Update pattern match
9. **test_parse_symbol** - Update pattern match

**New tests to add (span verification):**

```rust
#[test]
fn test_span_tracking_symbol() {
    let mut parser = Parser::new("hello".to_string());
    let forms = parser.parse().unwrap();

    if let SurfaceForm::Symbol { span, name } = &forms[0] {
        assert_eq!(name, "hello");
        assert_eq!(span.start_line, 1);
        assert_eq!(span.start_column, 1);
        assert_eq!(span.end_line, 1);
        assert_eq!(span.end_column, 6); // After 'o'
    } else {
        panic!("Expected Symbol");
    }
}

#[test]
fn test_span_tracking_list() {
    let mut parser = Parser::new("(+ 1 2)".to_string());
    let forms = parser.parse().unwrap();

    if let SurfaceForm::List { span, elements } = &forms[0] {
        assert_eq!(elements.len(), 3);
        assert_eq!(span.start_line, 1);
        assert_eq!(span.start_column, 1);
        assert_eq!(span.end_line, 1);
        assert_eq!(span.end_column, 8); // After ')'
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_span_tracking_multiline() {
    let source = r#"(deffn main ()
  (println! "test"))"#;
    let mut parser = Parser::new(source.to_string());
    let forms = parser.parse().unwrap();

    if let SurfaceForm::List { span, .. } = &forms[0] {
        assert_eq!(span.start_line, 1);
        assert_eq!(span.start_column, 1);
        assert_eq!(span.end_line, 2);
        // Should span to end of second line
        assert!(span.end_line > span.start_line);
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_parser_new_file() {
    let parser = Parser::new_file("(+ 1 2)".to_string(), "test.oxur".to_string());
    assert_eq!(parser.filename, "test.oxur");
    assert_eq!(parser.line, 1);
    assert_eq!(parser.column, 1);
}

#[test]
fn test_current_position() {
    let parser = Parser::new("hello".to_string());
    let (line, col) = parser.current_pos();
    assert_eq!(line, 1);
    assert_eq!(col, 1);
}
```

### Integration Tests (in `expander.rs`)

**Tests that need updating (8 expander tests):**

All tests that construct `SurfaceForm` directly need updating:

1. **test_expand_symbol** (line 210) - Update construction
2. **test_expand_number** (line 223) - Update construction
3. **test_expand_string** (line 236) - Update construction

**Pattern for test construction:**

**Before:**
```rust
let surface = SurfaceForm::Symbol("test".to_string());
```

**After:**
```rust
let span = Span::repl(1, 1, 1, 5); // line 1, col 1 to col 5
let surface = SurfaceForm::Symbol {
    span,
    name: "test".to_string(),
};
```

---

## Success Criteria

1. ✅ SurfaceForm uses struct variants with Span fields
2. ✅ Parser tracks line and column positions
3. ✅ All parse methods create Spans correctly
4. ✅ All 11 parser tests pass
5. ✅ All 8 expander tests pass
6. ✅ New span verification tests pass (5 new tests)
7. ✅ No clippy warnings
8. ✅ Code formatted with rustfmt
9. ✅ Documentation builds without warnings
10. ✅ All patterns in expander updated (39 locations)

---

## Implementation Steps

### Step 1: Update SurfaceForm Definition (20 min)
1. Change enum to use struct variants
2. Add Span field to each variant
3. Add `span()` helper method
4. Add proper documentation

### Step 2: Update Parser Structure (15 min)
1. Add `line`, `column`, `filename` fields
2. Update `new()` constructor
3. Add `new_file()` constructor
4. Add position helper methods:
   - `current_pos()`
   - `mark_position()`
   - `make_span()`

### Step 3: Update advance() Method (10 min)
1. Track newlines and update line/column
2. Handle column reset on newline
3. Test with multi-line input

### Step 4: Update Parse Methods (30 min)
1. Update `parse_list()` to capture spans
2. Update `parse_string()` to capture spans
3. Update `parse_number()` to capture spans
4. Update `parse_symbol()` to capture spans

### Step 5: Update Parser Tests (20 min)
1. Update all 11 existing tests
2. Change pattern matches to struct style
3. Ensure all tests pass

### Step 6: Add New Span Tests (15 min)
1. Add `test_span_tracking_symbol()`
2. Add `test_span_tracking_list()`
3. Add `test_span_tracking_multiline()`
4. Add `test_parser_new_file()`
5. Add `test_current_position()`

### Step 7: Update Expander (15 min)
1. Update `expand_form()` pattern matches (5 locations)
2. Update `expand_deffn()` pattern matches (3 locations)
3. Update `expand_list()` if needed

### Step 8: Update Expander Tests (10 min)
1. Update test constructions (3 tests)
2. Use `Span::repl()` for test data
3. Ensure all tests pass

### Step 9: Verify (5 min)
1. Run `cargo test --package oxur-lang`
2. Run `cargo clippy --package oxur-lang`
3. Run `cargo fmt --package oxur-lang`
4. Check all success criteria

---

## Impact Analysis

### Breaking Changes

**SurfaceForm API:**
- ✅ **Breaking:** All pattern matches must update to struct style
- ✅ **Breaking:** Construction must use named fields

**Mitigation:**
- All breaking changes are internal to oxur-lang
- No external crates depend on SurfaceForm yet
- Changes are mechanical and straightforward

### Files Modified

```
crates/oxur-lang/src/parser.rs    (MODIFIED - ~200 line changes)
crates/oxur-lang/src/expander.rs  (MODIFIED - ~30 line changes)
```

### Backward Compatibility

**Not applicable** - SurfaceForm is not yet part of public API.

---

## Next Stage Preparation

**Stage 1.3** will need:
- Parser's position tracking infrastructure (provided by this stage) ✅
- Span fields in SurfaceForm (provided by this stage) ✅
- Line/column helpers in Parser (provided by this stage) ✅

**Stage 1.3** will add:
- `current_position()` public API
- More sophisticated span calculation
- Position tracking tests

---

## Notes

### Design Rationale: Struct Variants

We chose struct variants over tuple variants for several reasons:

1. **Consistency:** CoreForm already uses struct variants with NodeId
2. **Clarity:** Named fields are self-documenting
3. **Extensibility:** Easy to add fields later without breaking changes
4. **Safety:** Can't accidentally swap field positions

### Parser Position Tracking

The parser now tracks three position types:

1. **byte offset** (`position: usize`) - For string slicing
2. **line number** (`line: usize`) - For span creation
3. **column number** (`column: usize`) - For span creation

This enables accurate span creation for error reporting.

### Span Semantics

- `start_column` - Points to first character
- `end_column` - Points *after* last character (exclusive)
- Example: `"hello"` at column 1 has `end_column = 6`

---

## Estimated Time Breakdown

| Task | Time |
|------|------|
| Update SurfaceForm definition | 20 min |
| Update Parser structure | 15 min |
| Update advance() method | 10 min |
| Update parse methods | 30 min |
| Update parser tests | 20 min |
| Add new span tests | 15 min |
| Update expander pattern matches | 15 min |
| Update expander tests | 10 min |
| Verify and test | 5 min |
| **Total** | **140 min (2h 20min)** |

---

## References

- **Stage breakdown:** `crates/design/dev/0012-pipeline-chain-completion-stages.md`
- **Development plan:** `crates/design/dev/0011-pipeline-chain-completion.md`
- **Previous stage:** `crates/design/dev/0013-chain-stage-1.1-implementation-plan.md`
- **oxur-smap plan:** `crates/design/docs/06-final/0039-oxur-smap-implementation-plan.md`
- **Architecture spec:** `crates/design/docs/01-draft/0013-oxur-compilation-chain-architecture.md`
