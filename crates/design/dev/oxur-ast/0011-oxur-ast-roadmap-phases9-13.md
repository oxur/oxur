# oxur-ast Implementation Roadmap: Phases 9-13

**Document Version**: 1.0
**Created**: 2026-01-03
**Status**: Planning Complete, Ready for Implementation

---

## Executive Summary

This roadmap defines the path to achieving **97-99% coverage** of Rust syntax in oxur-ast. Based on comprehensive gap analysis, the work is organized into **5 phases** of incremental improvements, each building on the previous.

### Current State (Post-Phase 8)
- **Coverage**: ~50% of common Rust syntax
- **Can parse**: Simple functions, structs, enums, basic expressions
- **Cannot parse**: Closures, iterators, match, most patterns, async/await

### Target State (Post-Phase 13)
- **Coverage**: 97-99% of Rust syntax
- **Can parse**: Virtually all real-world Rust code
- **Ready for**: Production use on real Rust codebases

### Total Estimated Effort
**10-15 days** of focused development work for an experienced Rust developer

---

## Phase Overview

| Phase | Name | Effort | Coverage Gain | Priority |
|-------|------|--------|---------------|----------|
| **9** | Connect Existing AST | 1-2 days | +40-50% → 70% | **QUICK WIN** |
| **10** | Critical Expressions | 3-4 days | +20% → 90% | **HIGH IMPACT** |
| **11** | Literals & Patterns | 1-2 days | +3-5% → 93-95% | MEDIUM |
| **12** | Advanced Types | 2-3 days | +2-3% → 95-97% | MEDIUM |
| **13** | Async & Modern Features | 2-3 days | +2-3% → 97-99% | MEDIUM |

---

## Phase 9: Connect Existing AST Definitions

**📄 Full Plan**: [`PHASE_9_CONNECT_EXISTING_AST.md`](./PHASE_9_CONNECT_EXISTING_AST.md)

### Quick Summary
Connect AST types that are **already defined** but not yet wired to the parser. This is the biggest "bang for buck" - massive coverage gain with minimal new code.

### What Gets Added
**Expressions (11 types)**:
- Array, Tuple, Struct literals
- Range expressions (`0..10`, `..=5`)
- Assignment (`x = 5`)
- Parenthesized expressions
- Try operator (`expr?`)
- Type casts (`x as i32`)
- Break, Continue, Return

**Patterns (12 types)**:
- Wildcard (`_`) ⭐ Critical for match
- Tuple patterns `(a, b, c)`
- Struct patterns `Point { x, y }`
- Tuple struct `Some(x)`
- Path patterns `None`, `Some`
- Literal patterns `42`, `"hello"`
- Range patterns `1..=5`
- Slice patterns `[a, b, ..]`
- Reference patterns `&x`
- Or-patterns `A | B`
- Rest pattern `..`
- Parenthesized patterns

### Key Benefits
- ✅ Low complexity - straightforward pattern matching
- ✅ Huge coverage boost (+40-50%)
- ✅ Enables Phase 10 (match needs patterns)
- ✅ Most types already have generators

### Files to Modify
- `crates/oxur-ast/src/integration/from_syn.rs` (primary)
- Test files (3 new files, ~40 tests)

### Success Criteria
- Parse struct literals and patterns
- Wildcard pattern works in match
- Array and tuple expressions work
- Try operator (`?`) works
- 70% coverage achieved

---

## Phase 10: Critical Expressions (Closures, Loops, Match)

**📄 Full Plan**: [`PHASE_10_CRITICAL_EXPRESSIONS.md`](./PHASE_10_CRITICAL_EXPRESSIONS.md)

### Quick Summary
Implement the **most critical** missing features that appear in 80%+ of Rust code. These are essential for idiomatic Rust.

### What Gets Added
**Expressions (5 types)**:
1. **Closures** `|x| x * 2` ⭐ Blocks 95% of iterator code
2. **Match** ⭐ Core Rust pattern matching
3. **For loops** `for x in iter` ⭐ Most common iteration
4. **Loop** `loop { }` Infinite loops
5. **While** `while cond { }` Conditional loops

### Key Benefits
- ✅ Unlocks iterator chains (`.map(|x| ...)`)
- ✅ Enables pattern matching on Option/Result
- ✅ Makes 90% of Rust code parseable
- ✅ Critical for real-world codebases

### Real-World Impact
**Before Phase 10**: Cannot parse
```rust
items.iter().filter(|x| x > 0).map(|x| x * 2).collect()

match option {
    Some(x) => x,
    None => 0,
}
```

**After Phase 10**: ✅ Parses perfectly

### Files to Modify
- `crates/oxur-ast/src/integration/from_syn.rs` (~200 lines)
- `crates/oxur-ast/src/ast/expr.rs` (verify Arm type exists)
- Generator updates
- Test files (4 new files, ~30 tests)

### Success Criteria
- Iterator chains with closures work
- Match with all pattern types works
- Labeled loops work
- Can parse 90%+ of typical Rust code

---

## Phase 11: Literal Completeness and Pattern Refinement

**📄 Full Plan**: [`PHASE_11_LITERALS_COMPLETENESS.md`](./PHASE_11_LITERALS_COMPLETENESS.md)

### Quick Summary
Complete all basic data type support and refine pattern matching. Ensures all primitive types work correctly.

### What Gets Added
**Literals (7 types)**:
- `Bool` (`true`, `false`) ⭐ Very common
- `Float` (`3.14`, `1.0e10`) ⭐ Common
- `Char` (`'a'`, `'\n'`) ⭐ Common
- `Byte` (`b'A'`)
- `ByteStr` (`b"hello"`)
- `CStr` (`c"hello"`)
- `Verbatim` (fallback)

**Pattern Refinements**:
- Box patterns
- Macro patterns
- Const patterns
- Shorthand detection (`Point { x, y }` vs `Point { x: x, y: y }`)

### Key Benefits
- ✅ Boolean logic works everywhere
- ✅ Floating-point arithmetic supported
- ✅ Character literals in match patterns
- ✅ Shorthand syntax preserved in round-trips
- ✅ Completes basic type support

### Files to Modify
- `crates/oxur-ast/src/integration/from_syn.rs`
- `crates/oxur-ast/src/ast/expr.rs` (add LitKind variants)
- Generator updates (escape sequences)
- Test files (3 new files, ~25 tests)

### Success Criteria
- Boolean and float literals work
- Shorthand struct syntax preserved
- All round-trip tests pass
- 93-95% coverage achieved

---

## Phase 12: Advanced Type System

**📄 Full Plan**: [`PHASE_12_ADVANCED_TYPES.md`](./PHASE_12_ADVANCED_TYPES.md)

### Quick Summary
Complete the type system with modern Rust type features. Essential for API design and trait-based programming.

### What Gets Added
**Type Features (5 types)**:
1. **Impl Trait** `impl Iterator<Item = i32>` ⭐ Modern APIs
2. **Trait Objects** `dyn Display + Send` ⭐ Dynamic dispatch
3. **Bare Function Types** `fn(i32) -> bool` Function pointers
4. **Parenthesized Types** `(Box<dyn Display>)` Clarity
5. **Macro Types** `vec![i32]` Macro in type position

### Key Benefits
- ✅ Modern API patterns work
- ✅ Dynamic dispatch supported
- ✅ FFI function pointers work
- ✅ Complete type system

### Real-World Impact
**Before Phase 12**: Cannot parse
```rust
fn numbers() -> impl Iterator<Item = i32> {
    vec![1, 2, 3].into_iter()
}

let display: Box<dyn Display> = Box::new(42);
type Callback = fn(i32) -> bool;
```

**After Phase 12**: ✅ Parses perfectly

### Files to Modify
- `crates/oxur-ast/src/integration/from_syn.rs`
- Generator updates (type formatting)
- Test files (3 new files, ~20 tests)

### Success Criteria
- Impl Trait in return and parameter positions works
- Trait objects with multiple bounds parse
- FFI function pointers work
- 95-97% coverage achieved

---

## Phase 13: Async/Await and Modern Features

**📄 Full Plan**: [`PHASE_13_ASYNC_MODERN_FEATURES.md`](./PHASE_13_ASYNC_MODERN_FEATURES.md)

### Quick Summary
**Final phase**: Add async/await support and complete remaining modern Rust features. Achieves near-complete coverage.

### What Gets Added
**Async Features**:
- `async { }` blocks
- `.await` expressions
- `async fn` functions

**Remaining Expressions**:
- Let expressions (`if let`, `while let`)
- Array repeat (`[0; 100]`)
- Unsafe blocks
- Yield (generators - unstable)
- Const blocks

### Key Benefits
- ✅ Async Rust fully supported
- ✅ Can parse tokio/async-std code
- ✅ 97-99% coverage - production ready
- ✅ Virtually all real Rust code parses

### Real-World Impact
**Before Phase 13**: Cannot parse
```rust
async fn fetch() -> Result<String, Error> {
    let response = client.get("url").await?;
    Ok(response.text().await?)
}
```

**After Phase 13**: ✅ Parses perfectly

### Files to Modify
- `crates/oxur-ast/src/integration/from_syn.rs`
- `crates/oxur-ast/src/ast/expr.rs` (add async variants)
- Generator updates (async syntax)
- Test files (3 new files, ~25 tests)

### Success Criteria
- Async blocks and functions work
- Await in all contexts works
- Can parse real async crates (tokio, etc.)
- **97-99% coverage achieved** 🎉

---

## Implementation Strategy

### Recommended Order

**Option A: Fastest Time to Value** (Recommended)
```
Phase 9 → Phase 10 → Phase 11 → Phase 12 → Phase 13
  ^         ^          ^           ^           ^
Quick    Critical   Polish    Modern     Complete
wins     features   details   types      async
```

**Why**: Each phase builds on the previous, and priorities are ordered by impact.

**Option B: Highest Impact First**
```
Phase 10 (Closures) → Phase 9 → Phase 11 → Phase 12 → Phase 13
```
**Why**: Gets closures/match working immediately, but Phase 9 patterns are helpful for Phase 10.

**Option C: Parallel Development**
```
Developer 1: Phases 9 → 11 (Low-hanging fruit)
Developer 2: Phases 10 → 12 (Complex features)
Both: Phase 13 (Final integration)
```
**Why**: If you have 2 developers, maximize parallelism.

### Dependencies

```
Phase 9 (Patterns) ──→ Required for Phase 10 (Match needs patterns)
    ↓
Phase 10 (Critical) ──→ Makes everything else useful
    ↓
Phase 11 (Literals) ──→ Independent, can be done anytime
    ↓
Phase 12 (Types) ──→ Independent, can be done anytime
    ↓
Phase 13 (Async) ──→ Final polish
```

---

## Testing Strategy

### Test Coverage Goals

Each phase must achieve:
- ✅ **Unit tests**: 95%+ coverage of new code
- ✅ **Integration tests**: Real Rust code examples
- ✅ **Round-trip tests**: Parse → Generate → Parse produces same AST
- ✅ **Real-world tests**: Parse actual crates

### Cumulative Test Count

| After Phase | Unit Tests | Integration Tests | Total Tests |
|-------------|------------|-------------------|-------------|
| 9 | +40 | +10 | ~350 |
| 10 | +30 | +15 | ~395 |
| 11 | +25 | +10 | ~430 |
| 12 | +20 | +10 | ~460 |
| 13 | +25 | +15 | ~500 |

### Real-World Validation

After each phase, test against:
1. **Own codebase**: `crates/oxur-lang/`, `crates/oxur-comp/`
2. **Rust stdlib**: Selected files from `std`
3. **Popular crates**: serde, tokio, etc.
4. **Measure coverage**: Percentage of successfully parsed code

---

## Success Metrics

### Per-Phase Metrics

| Phase | Coverage Target | Key Capability Unlocked |
|-------|----------------|------------------------|
| 9 | 70% | Struct literals, patterns |
| 10 | 90% | Iterator chains, match |
| 11 | 93-95% | All basic types |
| 12 | 95-97% | Modern APIs |
| 13 | **97-99%** | **Production ready** |

### Final Success Criteria

After Phase 13, oxur-ast should:
- ✅ Parse 97-99% of Rust code in the wild
- ✅ Handle all Rust 2021 edition features
- ✅ Support async/await fully
- ✅ Round-trip all parsed code correctly
- ✅ Pass 500+ tests
- ✅ Parse oxur's own source code completely
- ✅ Parse popular crates: tokio, serde, axum, etc.

---

## Risk Assessment

### Low Risk
- **Phases 9, 11**: Mostly mechanical pattern matching
- **Generators**: Most already exist, minor updates needed

### Medium Risk
- **Phase 10**: Closures are complex, many edge cases
- **Phase 12**: Type system has subtle details

### High Risk
- **Phase 13**: Async is complex, interactions with other features

### Mitigation
- Start with comprehensive tests
- Test with real code early and often
- Use existing syn crate as reference
- Incremental implementation with validation

---

## File Organization

### Plans
```
workbench/
  ├── IMPLEMENTATION_ROADMAP.md ← You are here
  ├── PHASE_9_CONNECT_EXISTING_AST.md
  ├── PHASE_10_CRITICAL_EXPRESSIONS.md
  ├── PHASE_11_LITERALS_COMPLETENESS.md
  ├── PHASE_12_ADVANCED_TYPES.md
  └── PHASE_13_ASYNC_MODERN_FEATURES.md
```

### Tests (to be created)
```
crates/oxur-ast/tests/
  ├── phase9_expressions_tests.rs
  ├── phase9_patterns_tests.rs
  ├── phase9_integration_tests.rs
  ├── phase10_closures_tests.rs
  ├── phase10_match_tests.rs
  ├── phase10_loops_tests.rs
  ├── phase10_real_world_tests.rs
  ├── phase11_literals_tests.rs
  ├── phase11_shorthand_tests.rs
  ├── phase11_round_trip_tests.rs
  ├── phase12_impl_trait_tests.rs
  ├── phase12_trait_object_tests.rs
  ├── phase12_bare_fn_tests.rs
  ├── phase13_async_tests.rs
  ├── phase13_remaining_exprs_tests.rs
  └── phase13_real_world_async_tests.rs
```

---

## Quick Start Guide

### To Start Implementation

1. **Read this roadmap** (you're doing it!)
2. **Choose starting phase** (recommend Phase 9)
3. **Open detailed plan**: e.g., `PHASE_9_CONNECT_EXISTING_AST.md`
4. **Follow task-by-task**: Each plan has detailed steps
5. **Write tests first**: TDD approach recommended
6. **Implement**: Follow the code examples in each plan
7. **Verify**: Run tests, check coverage
8. **Move to next phase**: When all criteria met

### Daily Workflow

```bash
# 1. Read current phase plan
cat workbench/PHASE_X_*.md

# 2. Create test file
touch crates/oxur-ast/tests/phase${X}_${feature}_tests.rs

# 3. Write tests (TDD)
# ... write failing tests ...

# 4. Implement feature
# Edit: crates/oxur-ast/src/integration/from_syn.rs

# 5. Run tests
cd crates/oxur-ast && cargo test

# 6. Verify with real code
./bin/aster to-ast crates/oxur-lang/src/lib.rs

# 7. Update generators if needed
# Edit: crates/oxur-ast/src/gen_rs/expr.rs

# 8. Final validation
cargo test && cargo build --bin aster
```

---

## Appendix: Gap Analysis Summary

### What's Missing (Pre-Phase 9)

**Expressions**: 28/40 missing (70% gap)
- Closures ⭐⭐⭐ (blocks 95% of code)
- Match ⭐⭐⭐ (core Rust)
- For loops ⭐⭐⭐ (most common iteration)
- Struct literals ⭐⭐ (very common)
- Boolean literals ⭐⭐ (everywhere)
- And 23 more...

**Patterns**: 12/14 missing (86% gap!)
- Wildcard `_` ⭐⭐⭐ (required for match)
- Tuple patterns ⭐⭐ (common destructuring)
- Struct patterns ⭐⭐ (common destructuring)
- And 9 more...

**Types**: 8/16 missing (50% gap)
- Impl Trait ⭐⭐ (modern APIs)
- Trait Objects ⭐⭐ (dynamic dispatch)
- Bare functions ⭐ (FFI)
- And 5 more...

### What's Already There (Ready to Connect)

**Good News**: Many types are already defined in AST!
- 7 expression types defined but not connected
- 12 pattern types defined but not connected
- 5 type variants defined but not connected

**Impact**: Phase 9 alone gives +40-50% coverage just by connecting existing code!

---

## Conclusion

This roadmap provides a clear, actionable path from **50% → 99% coverage** in **10-15 days** of focused work. Each phase is:

- ✅ **Self-contained**: Can be implemented independently
- ✅ **Well-documented**: Detailed plans with code examples
- ✅ **Testable**: Clear success criteria
- ✅ **Incremental**: Each phase adds value
- ✅ **Realistic**: Based on actual gap analysis

**Next step**: Open `PHASE_9_CONNECT_EXISTING_AST.md` and begin!

Good luck! 🚀

---

**Document Maintenance**:
- Update this roadmap as phases complete
- Mark phases as ✅ Done
- Record actual time taken vs. estimates
- Note any deviations or discoveries
- Keep as living document for future reference
