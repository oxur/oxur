# Stage 1.7 Completion: Surface → Core Mapping

**Date:** 2026-01-12
**Stage:** 1.7 from Phase 1 (Source Mapping Infrastructure)
**Deliverable:** Update Expander to record mappings from Surface Span to Core NodeId
**Estimated Time:** 3 hours
**Dependencies:** Stage 1.6 (SourceMap recording API) ✅
**Status:** ✅ COMPLETE

---

## Summary

Stage 1.7 implemented source mapping in the Expander, recording the transformation from Surface Forms (with Span positions) to Core Forms (with NodeIds). This enables tracking the origin of every Core Form node back to its original source code position.

## Implementation Details

### Code Changes

**1. Added Helper Method** (`crates/oxur-lang/src/expander.rs:68-77`):
```rust
/// Convert a Span to SourcePos, using start position for multi-line spans
fn span_to_source_pos(span: &oxur_smap::Span) -> oxur_smap::SourcePos {
    // Use start position and length 1 (exact length not critical for error reporting)
    oxur_smap::SourcePos::new(
        span.file.clone(),
        span.start_line,
        span.start_column,
        1, // Length placeholder
    )
}
```

**2. Updated `expand_form()` to Record Mappings** (lines 32-66):
- Symbol variant: Record Core NodeId → Surface SourcePos
- Number variant: Record Core NodeId → Surface SourcePos
- String variant: Record Core NodeId → Surface SourcePos
- List variant: Pass span to helper methods

**3. Updated `expand_deffn()` Signature and Implementation** (lines 79-121):
- Added `span: oxur_smap::Span` parameter
- Records mapping before creating DefineFunc CoreForm
- Called with span from `expand_form()`

**4. Updated `expand_list()` Signature and Implementation** (lines 123-136):
- Added `span: oxur_smap::Span` parameter
- Records mapping before expanding elements
- Each element recursively records its own mapping

### Tests Added (5 new tests)

**1. `test_source_map_symbol`** - Verify symbol NodeId mapping
- Parses "hello" → Symbol
- Checks Core NodeId has recorded source position
- Verifies line=1, column=1

**2. `test_source_map_number`** - Verify number NodeId mapping
- Parses "42" → Number
- Checks Core NodeId has recorded source position
- Verifies line=1, column=1

**3. `test_source_map_list`** - Verify list and nested element mappings
- Parses "(+ 1 2)" → List with 3 elements
- Checks list node has mapping (line=1)
- Checks first element ('+') has mapping (column=2)
- Demonstrates nested structure mappings work

**4. `test_source_map_deffn`** - Verify DefineFunc mapping
- Parses "(deffn main () 42)" → DefineFunc
- Checks DefineFunc NodeId has recorded source position
- Verifies line=1, column=1

**5. `test_source_map_stats`** - Verify mapping statistics
- Parses "(+ 1 2)" with 4 nodes (list, +, 1, 2)
- Checks stats show 4 surface nodes recorded
- Verifies no expansion or lowering chains yet (those are Stage 1.8+)

### Test Results

**Before Stage 1.7:**
- Total tests: 48 (25 parser + 8 expander + 11 core_forms + 4 lib)

**After Stage 1.7:**
- Total tests: **53** (25 parser + **13 expander** + 11 core_forms + 4 lib)
- New tests: +5 expander tests
- All tests passing: ✅

### Coverage Results

**Expander Module (`crates/oxur-lang/src/expander.rs`):**
- Region Coverage: **91.80%** (500 regions, 41 missed)
- Function Coverage: **100.00%** (22 functions, 0 missed)
- Line Coverage: **89.57%** (230 lines, 24 missed)

**Overall oxur-lang Package:**
- Region Coverage: **~76%** (up from 74.53%)
- Function Coverage: **~75%** (up from 73.04%)
- Line Coverage: **~76%** (up from 74.00%)

**Quality Checks:**
- Clippy: ✅ Clean (no warnings)
- Formatting: ✅ Correct (rustfmt applied)
- All 53 tests passing: ✅

## Technical Notes

### Multi-line Span Handling

**Approach Used:**
- Record start position only (start_line, start_column)
- Set length to 1 (placeholder)

**Rationale:**
- Sufficient for error reporting (points to start of construct)
- Avoids complexity of multi-line SourcePos conversion
- The Span → SourcePos trait conversion panics on multi-line spans, so we manually construct SourcePos

**Future Enhancement:**
- Could extend SourcePos to support multi-line spans
- Would enable better error highlighting (underline entire construct)
- Out of scope for v1.0

### Mapping Architecture

**Current State (After Stage 1.7):**
```
Surface Forms (Span) → Core Forms (NodeId)
                       ↓
                  SourceMap records: NodeId → SourcePos
```

**Next Stage (1.8) Will Add:**
```
Surface Forms (Span) → Core Forms (NodeId) → Rust AST (NodeId)
                       ↓                      ↓
                  SourceMap records:     SourceMap records:
                  NodeId → SourcePos     Core NodeId → Rust NodeId
```

**Final Error Translation (Stage 1.10):**
```
Rust Compiler Error (file:line:col)
    ↓ map to Rust NodeId
    ↓ lookup in SourceMap (Rust → Core)
    ↓ lookup in SourceMap (Core → SourcePos)
    ↓ display Oxur error with original source position
```

### Files Modified

1. **`crates/oxur-lang/src/expander.rs`**
   - Added `span_to_source_pos()` helper method
   - Updated `expand_form()` to record mappings (4 variants)
   - Updated `expand_deffn()` signature and implementation
   - Updated `expand_list()` signature and implementation
   - Added 5 new test functions
   - Lines changed: ~50 lines modified, ~130 lines added

2. **`crates/design/dev/0016-chain-stage-1.7-implementation-plan.md`** (NEW)
   - Comprehensive implementation plan created before coding

3. **`crates/design/dev/0017-chain-stage-1.7-completion.md`** (THIS FILE)
   - Completion documentation

## Success Criteria Met

✅ **All CoreForm nodes have mappings recorded in SourceMap**
✅ **`source_map.get_surface_position(core_id)` returns accurate positions**
✅ **Nested structures (lists, deffn) record mappings correctly**
✅ **5 new tests pass verifying mapping accuracy**
✅ **All existing tests still pass (no regressions)**
✅ **Coverage ≥ 85% for expander.rs** (91.80% regions, 89.57% lines)
✅ **Clippy clean, formatting correct**
✅ **Public accessor `source_map()` available** (already existed)

## Phase 1 Progress

### Week 1: Position Tracking in Parser ✅ COMPLETE
- Stage 1.1: Span types ✅
- Stage 1.2: SurfaceForm with Span ✅
- Stage 1.3: Position tracking foundation ✅
- Stage 1.4: Position tracking in parse methods ✅
- Stage 1.5: Position tracking tests ✅

### Week 2: Mapping Chains Through Pipeline 🚧 IN PROGRESS
- Stage 1.6: SourceMap recording API ✅ (Already existed)
- Stage 1.7: Surface → Core mapping ✅ **COMPLETE**
- Stage 1.8: Core → syn mapping (Next: Update Lowerer)
- Stage 1.9: rustc diagnostic parser
- Stage 1.10: Error translator
- Stage 1.11: Error translation tests

## Next Steps

**Stage 1.8:** Core → syn mapping (3 hours estimated)
- Update Lowerer to record Core NodeId → Rust NodeId mappings
- Similar pattern to Stage 1.7 but in oxur-comp crate
- Will complete the transformation chain for error translation

**Remaining Week 2 Tasks:**
- Stage 1.9: Parse rustc JSON diagnostic output (2 hours)
- Stage 1.10: Implement error translator (3 hours)
- Stage 1.11: End-to-end error translation tests (2 hours)

**Phase 1 Total Progress:** 7/11 stages complete (64%)

---

## Related Documents

- Implementation plan: `crates/design/dev/0016-chain-stage-1.7-implementation-plan.md`
- Stage breakdown: `crates/design/dev/0012-pipeline-chain-completion-stages.md`
- Main plan: `crates/design/dev/0011-pipeline-chain-completion.md`
- Previous stage: Stage 1.6 (SourceMap recording API)
- Next stage: Stage 1.8 (Core → syn mapping)

---

**Completion Date:** 2026-01-12
**Time Spent:** ~2 hours (under 3h estimate)
**Quality:** All tests pass, 91.80% coverage, clippy clean
