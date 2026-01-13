# Stage 1.7 Implementation Plan: Surface → Core Mapping

**Date:** 2026-01-12
**Stage:** 1.7 from Phase 1 (Source Mapping Infrastructure)
**Deliverable:** Update Expander to record mappings from Surface Span to Core NodeId
**Estimated Time:** 3 hours
**Dependencies:** Stage 1.6 (SourceMap recording API) ✅

---

## Current State Analysis

### What Exists ✅

**Expander Structure** (`crates/oxur-lang/src/expander.rs:11-14`):
```rust
pub struct Expander {
    source_map: SourceMap,
}
```

**SurfaceForm with Span** (`crates/oxur-lang/src/parser.rs:200-212`):
```rust
#[derive(Debug, Clone)]
pub enum SurfaceForm {
    Symbol { span: Span, name: String },
    Number { span: Span, value: i64 },
    String { span: Span, value: String },
    List { span: Span, elements: Vec<SurfaceForm> },
}

impl SurfaceForm {
    pub fn span(&self) -> &Span { /* ... */ }
}
```

**CoreForm with NodeId** (`crates/oxur-lang/src/core_forms.rs`):
```rust
pub enum CoreForm {
    Symbol { id: NodeId, name: String },
    Number { id: NodeId, value: i64 },
    String { id: NodeId, value: String },
    List { id: NodeId, elements: Vec<CoreForm> },
    DefineFunc { id: NodeId, name: String, params: Vec<String>, body: Box<CoreForm> },
    // ...
}
```

**SourceMap API** (`crates/oxur-smap/src/source_map.rs`):
- `record_surface_node(NodeId, SourcePos)` - Record Core NodeId → Surface SourcePos mapping
- `Span` → `SourcePos` conversion: `impl From<&Span> for SourcePos` (single-line spans only)

**Current Expansion** (`crates/oxur-lang/src/expander.rs:32-60`):
- Creates NodeIds for Core Forms ✓
- Does NOT record mappings to SourceMap ❌
- SourceMap field exists but is unused ❌

### What's Missing 🔲

1. **Span → SourcePos conversion** for multi-line spans
2. **Recording mappings** in `expand_form()`
3. **Recording mappings** in `expand_deffn()`
4. **Recording mappings** in `expand_list()`
5. **Public accessor** to get the SourceMap after expansion
6. **Tests** to verify mappings are recorded correctly

---

## Design Decisions

### Mapping Strategy: Core NodeId → Surface Span

**Pattern:**
```rust
let surface_span = form.span();
let core_id = oxur_smap::new_node_id();

// Convert Span → SourcePos and record
let source_pos: SourcePos = surface_span.into(); // May panic if multi-line
self.source_map.record_surface_node(core_id, source_pos);

// Create CoreForm with the NodeId
Ok(CoreForm::Symbol { id: core_id, name: value })
```

**Why not Surface NodeId → Core NodeId?**
- SurfaceForm is temporary (only exists during parse → expand)
- CoreForm is persistent (canonical representation)
- Direct mapping Core → Source position is more efficient
- Matches the pattern in ODD-0011 design document

### Handling Multi-line Spans

**Problem:** The `From<&Span> for SourcePos` implementation currently panics for multi-line spans:
```rust
impl From<&Span> for SourcePos {
    fn from(span: &Span) -> Self {
        assert!(span.is_single_line(), "Cannot convert multi-line Span to SourcePos");
        // ...
    }
}
```

**Solution Options:**

**Option A: Record start position only (RECOMMENDED)**
- Use span.start_line and span.start_column
- Create SourcePos manually without using From trait
- Simple, always works
- Good enough for error reporting (points to start of construct)

**Option B: Make From trait handle multi-line**
- Modify oxur-smap to make conversion non-panicking
- Requires changes to oxur-smap API
- Out of scope for Stage 1.7

**Decision: Use Option A for Stage 1.7**

### Recording in Nested Structures

When expanding a List:
```rust
SurfaceForm::List { span, elements } => {
    let core_id = oxur_smap::new_node_id();

    // Record the list node itself
    let pos = SourcePos::new(span.file.clone(), span.start_line, span.start_column, 1);
    self.source_map.record_surface_node(core_id, pos);

    // Recursively expand elements (each will record its own mapping)
    let mut expanded_elements = Vec::new();
    for element in elements {
        expanded_elements.push(self.expand_form(element)?);
    }

    Ok(CoreForm::List { id: core_id, elements: expanded_elements })
}
```

---

## Implementation Details

### File 1: `crates/oxur-lang/src/expander.rs` - Update `expand_form()`

**Location:** Lines 32-60

**Add helper method:**
```rust
impl Expander {
    /// Convert a Span to SourcePos, using start position for multi-line spans
    fn span_to_source_pos(span: &Span) -> SourcePos {
        // Use start position and length 1 (exact length not critical for error reporting)
        SourcePos::new(
            span.file.clone(),
            span.start_line,
            span.start_column,
            1, // Length placeholder
        )
    }
}
```

**Update Symbol variant:**
```rust
SurfaceForm::Symbol { name, span } => {
    let id = oxur_smap::new_node_id();
    let pos = Self::span_to_source_pos(&span);
    self.source_map.record_surface_node(id, pos);
    Ok(CoreForm::Symbol { id, name })
}
```

**Update Number variant:**
```rust
SurfaceForm::Number { value, span } => {
    let id = oxur_smap::new_node_id();
    let pos = Self::span_to_source_pos(&span);
    self.source_map.record_surface_node(id, pos);
    Ok(CoreForm::Number { id, value })
}
```

**Update String variant:**
```rust
SurfaceForm::String { value, span } => {
    let id = oxur_smap::new_node_id();
    let pos = Self::span_to_source_pos(&span);
    self.source_map.record_surface_node(id, pos);
    Ok(CoreForm::String { id, value })
}
```

**Update List variant:**
```rust
SurfaceForm::List { elements, span } => {
    // Check for special forms
    if !elements.is_empty() {
        if let SurfaceForm::Symbol { ref name, .. } = elements[0] {
            if name == "deffn" {
                return self.expand_deffn(elements, span);
            }
        }
    }

    // Regular list expansion
    self.expand_list(elements, span)
}
```

### File 2: `crates/oxur-lang/src/expander.rs` - Update `expand_deffn()`

**Location:** Lines 62-102

**Update signature to accept span:**
```rust
fn expand_deffn(&mut self, elements: Vec<SurfaceForm>, span: Span) -> Result<CoreForm> {
    // ... existing validation ...

    let id = oxur_smap::new_node_id();
    let pos = Self::span_to_source_pos(&span);
    self.source_map.record_surface_node(id, pos);

    // ... rest of expansion ...

    Ok(CoreForm::DefineFunc { id, name, params, body })
}
```

### File 3: `crates/oxur-lang/src/expander.rs` - Update `expand_list()`

**Location:** Lines 104-113

**Update signature to accept span:**
```rust
fn expand_list(&mut self, elements: Vec<SurfaceForm>, span: Span) -> Result<CoreForm> {
    let id = oxur_smap::new_node_id();
    let pos = Self::span_to_source_pos(&span);
    self.source_map.record_surface_node(id, pos);

    let mut expanded_elements = Vec::new();
    for element in elements {
        expanded_elements.push(self.expand_form(element)?);
    }

    Ok(CoreForm::List { id, elements: expanded_elements })
}
```

### File 4: `crates/oxur-lang/src/expander.rs` - Add Public Accessor

**Location:** After line 124 (after `is_empty()`)

```rust
/// Get a reference to the source map
///
/// This allows access to the populated source map after expansion
/// for error reporting and debugging.
pub fn source_map(&self) -> &SourceMap {
    &self.source_map
}
```

**Note:** Method already exists at line 116! No need to add.

### File 5: `crates/oxur-lang/src/expander.rs` - Add Tests

**Location:** In `mod tests` section (after line 256)

```rust
#[test]
fn test_source_map_symbol() {
    use crate::parser::Parser;

    let source = "hello";
    let mut parser = Parser::new(source.to_string());
    let forms = parser.parse().unwrap();

    let mut expander = Expander::new();
    let core_forms = expander.expand(forms).unwrap();

    // Get the NodeId from the CoreForm
    if let CoreForm::Symbol { id, .. } = &core_forms[0] {
        // Look up the source position via the SourceMap
        let source_map = expander.source_map();
        let pos = source_map.get_surface_position(id);

        assert!(pos.is_some(), "Mapping should be recorded");
        let pos = pos.unwrap();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);
    } else {
        panic!("Expected Symbol");
    }
}

#[test]
fn test_source_map_number() {
    use crate::parser::Parser;

    let source = "42";
    let mut parser = Parser::new(source.to_string());
    let forms = parser.parse().unwrap();

    let mut expander = Expander::new();
    let core_forms = expander.expand(forms).unwrap();

    if let CoreForm::Number { id, .. } = &core_forms[0] {
        let source_map = expander.source_map();
        let pos = source_map.get_surface_position(id);

        assert!(pos.is_some());
        let pos = pos.unwrap();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);
    } else {
        panic!("Expected Number");
    }
}

#[test]
fn test_source_map_list() {
    use crate::parser::Parser;

    let source = "(+ 1 2)";
    let mut parser = Parser::new(source.to_string());
    let forms = parser.parse().unwrap();

    let mut expander = Expander::new();
    let core_forms = expander.expand(forms).unwrap();

    if let CoreForm::List { id, elements, .. } = &core_forms[0] {
        let source_map = expander.source_map();

        // Check list node mapping
        let list_pos = source_map.get_surface_position(id);
        assert!(list_pos.is_some());
        assert_eq!(list_pos.unwrap().line, 1);

        // Check first element ('+' symbol) mapping
        if let CoreForm::Symbol { id: elem_id, .. } = &elements[0] {
            let elem_pos = source_map.get_surface_position(elem_id);
            assert!(elem_pos.is_some());
            assert_eq!(elem_pos.unwrap().column, 2); // After '('
        } else {
            panic!("Expected Symbol");
        }
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_source_map_deffn() {
    use crate::parser::Parser;

    let source = "(deffn main () 42)";
    let mut parser = Parser::new(source.to_string());
    let forms = parser.parse().unwrap();

    let mut expander = Expander::new();
    let core_forms = expander.expand(forms).unwrap();

    if let CoreForm::DefineFunc { id, .. } = &core_forms[0] {
        let source_map = expander.source_map();
        let pos = source_map.get_surface_position(id);

        assert!(pos.is_some());
        assert_eq!(pos.unwrap().line, 1);
        assert_eq!(pos.unwrap().column, 1);
    } else {
        panic!("Expected DefineFunc");
    }
}

#[test]
fn test_source_map_stats() {
    use crate::parser::Parser;

    let source = "(+ 1 2)"; // 4 nodes: list, +, 1, 2
    let mut parser = Parser::new(source.to_string());
    let forms = parser.parse().unwrap();

    let mut expander = Expander::new();
    let _core_forms = expander.expand(forms).unwrap();

    let source_map = expander.source_map();
    let stats = source_map.stats();

    // Should have 4 surface nodes recorded
    assert_eq!(stats.surface_nodes, 4);
    assert_eq!(stats.expansions, 0); // No expansion chains yet (Stage 1.7 only)
    assert_eq!(stats.lowerings, 0);  // No lowerings yet
}
```

---

## Implementation Steps

1. **Add helper method `span_to_source_pos()`** (10 min)
   - Add method to Expander impl block
   - Test with single-line and multi-line spans

2. **Update `expand_form()` to record mappings** (20 min)
   - Update Symbol variant
   - Update Number variant
   - Update String variant
   - Update List variant to pass span to helpers

3. **Update `expand_deffn()` to accept and record span** (15 min)
   - Add `span: Span` parameter
   - Record mapping before creating DefineFunc
   - Update call site in `expand_form()`

4. **Update `expand_list()` to accept and record span** (10 min)
   - Add `span: Span` parameter
   - Record mapping before expanding elements
   - Update call site in `expand_form()`

5. **Add mapping verification tests** (60 min)
   - test_source_map_symbol
   - test_source_map_number
   - test_source_map_list
   - test_source_map_deffn
   - test_source_map_stats

6. **Run full test suite and verify** (20 min)
   - Run `cargo test --package oxur-lang`
   - Check coverage with `cargo llvm-cov --package oxur-lang`
   - Verify no regressions

7. **Code quality checks** (10 min)
   - Run `cargo clippy --package oxur-lang`
   - Run `cargo fmt --package oxur-lang`

8. **Documentation and commit** (15 min)
   - Write completion document
   - Create detailed commit message
   - Tag as Stage 1.7 complete

**Total Estimated Time:** 160 minutes (~2.7 hours)

---

## Success Criteria

✅ All CoreForm nodes have mappings recorded in SourceMap
✅ `source_map.get_surface_position(core_id)` returns accurate positions
✅ Nested structures (lists, deffn) record mappings correctly
✅ 5 new tests pass verifying mapping accuracy
✅ All existing tests still pass (no regressions)
✅ Coverage ≥ 85% for expander.rs
✅ Clippy clean, formatting correct
✅ Public accessor `source_map()` available

---

## Testing Strategy

**Unit Tests:**
1. Verify Symbol mapping records line/column
2. Verify Number mapping records line/column
3. Verify List mapping records start position
4. Verify DefineFunc mapping records start position
5. Verify nested elements have individual mappings
6. Verify stats show correct number of recorded nodes

**Integration Tests:**
- Parse → Expand → Check SourceMap contains all nodes
- Verify multi-line constructs record start position

**Coverage Target:**
- Expander module: ≥ 85% lines
- New code: 100% coverage

---

## Notes

**Multi-line Span Handling:**
- Current implementation uses start position only
- This is sufficient for error reporting (user sees where construct starts)
- Future enhancement: Track span length for better error highlighting

**SourceMap Population:**
- Stage 1.7 populates: Core NodeId → Surface SourcePos
- Stage 1.8 will populate: Core NodeId → Rust NodeId (lowering chains)
- Stage 1.10 will use both chains for error translation

**Next Stage:**
After Stage 1.7, proceed to **Stage 1.8: Core → syn mapping** (Update Lowerer to record mappings)

---

## Related Documents

- Stage breakdown: `crates/design/dev/0012-pipeline-chain-completion-stages.md`
- Main plan: `crates/design/dev/0011-pipeline-chain-completion.md`
- Previous stage: Stage 1.6 (SourceMap recording API - already complete)
- Next stage: Stage 1.8 (Core → syn mapping)

---

**Plan Created:** 2026-01-12
**Estimated Completion:** Stage 1.7 implementation
