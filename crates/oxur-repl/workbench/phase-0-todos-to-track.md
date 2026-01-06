# Phase 0: TODO Items to Track as Issues

**Date:** 2026-01-06
**Purpose:** Convert TODO markers to tracked issues per ODD-0040 Phase 0 Task 0.3

---

## TODO Items Found in Source Code

### Issue #1: Track compilation dependencies in CachedCompiler
**File:** `src/compiler/cached.rs:121`
**Context:**
```rust
let content_key = self.cache.generate_key(
    source,
    &[], // TODO: Track dependencies when needed
    opt_level,
    "default",
);
```
**Description:** The cache key generation currently doesn't track external dependencies. We need to:
- Track which crates/modules the compiled code depends on
- Include dependency versions in cache key
- Invalidate cache when dependencies change

**Priority:** Medium (for cache correctness)
**Related:** ODD-0038 Section 2.3 (CachedCompiler)

---

### Issue #2: Add SourceMap configuration to cache keys
**File:** `src/compiler/cached.rs:123`
**Context:**
```rust
let content_key = self.cache.generate_key(
    source,
    &[],
    opt_level,
    "default", // TODO: Add source map configuration
);
```
**Description:** Cache keys should include SourceMap configuration:
- Whether source mapping is enabled
- SourceMap generation options
- Ensures cache invalidation when source map settings change

**Priority:** High (needed for Phase 4 SourceMap integration)
**Related:** ODD-0038 Section 4 (SourceMap Integration), ODD-0040 Phase 4

---

### Issue #3: Integrate SourceMap with ErrorTranslator
**File:** `src/compiler/error_translator.rs:221`
**Context:**
```rust
// TODO: Implement actual source map lookup
```
**Description:** ErrorTranslator needs to use SourceMap for position translation:
- Look up original Oxur position from Rust AST position
- Traverse SourceMap chain: Rust → Core → Surface
- Return original file/line/column for error display

**Priority:** CRITICAL (Phase 4 blocker)
**Related:** ODD-0038 Section 11 (Error Translation), ODD-0040 Phase 4 Task 4.2

---

### Issue #4: Extract suggestions from rustc diagnostics
**File:** `src/compiler/error_translator.rs:213`
**Context:**
```rust
suggestion: None, // TODO: Extract suggestions
```
**Description:** Parse and extract rustc's suggested fixes:
- Parse suggestion spans from JSON
- Format suggestions for Oxur code
- Display "help: try this" messages

**Priority:** Low (nice-to-have for error UX)
**Related:** ODD-0038 Section 11 (Error Translation)

---

### Issue #5: Track session creation time
**File:** `src/server/session.rs:197, 226`
**Context:**
```rust
created_at: 0, // TODO: Track creation time
```
**Description:** Track when sessions are created for:
- Session timeout implementation
- Cleanup of stale sessions
- Session management metrics

**Priority:** Medium (needed for production robustness)
**Related:** ODD-0040 Phase 0 Task 0.3

---

### Issue #6: Implement macro expansion in EvalContext
**File:** `src/eval/context.rs:482`
**Context:**
```rust
// TODO: Use oxur_lang::Expander to expand macros
```
**Description:** Add macro expansion step in eval pipeline:
- Call `oxur_lang::expand()` after parsing
- Pass SourceMap for transformation tracking
- Handle expansion errors gracefully

**Priority:** CRITICAL (Phase 3 requirement)
**Related:** ODD-0038 Section 5 (EvalContext), ODD-0040 Phase 3 Task 3.1

---

## Summary

**Total TODOs:** 7
**Critical:** 2 (Issues #3, #6)
**High:** 1 (Issue #2)
**Medium:** 2 (Issues #1, #5)
**Low:** 1 (Issue #4)

## Recommended Actions

1. **Create GitHub issues** for all 7 items above
2. **Label appropriately:**
   - CRITICAL/High → Phase 3 or Phase 4 milestones
   - Medium → Phase 5 (polish) milestone
   - Low → "enhancement" label
3. **Remove TODO comments** from source code
4. **Add issue references** in code comments:
   ```rust
   // See issue #X: Track compilation dependencies
   &[]
   ```

## Next Steps

After creating issues:
- [ ] Update source code to reference issues instead of TODOs
- [ ] Prioritize issues in project backlog
- [ ] Assign to appropriate implementation phases

---

**Generated:** 2026-01-06 during Phase 0 quality tasks
**Tool:** Claude Code automated analysis
