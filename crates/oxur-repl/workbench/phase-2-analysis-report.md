# Phase 2 RustAstWrapper - Analysis Report

**Date:** 2026-01-06
**Duration:** ~10 minutes (analysis)
**Status:** 🔄 IN PROGRESS (Stub → Full Implementation)

---

## Executive Summary

**Phase 2 has a STUB implementation that needs upgrading!**

The existing `RustAstWrapper` (286 lines, 11 tests) provides basic string-based wrapping. Phase 2 requires upgrading to:
- ✅ AST-based wrapping (syn/quote)
- ✅ VariableStore integration (load/store)
- ✅ Type inference integration
- ✅ Source map comment generation

**Current Status: ~30% complete** (basic scaffolding exists)

---

## Current Implementation Analysis

### File: `src/wrapper.rs` (286 lines)

**What EXISTS:**

✅ **Basic Structure** (lines 24-59)
```rust
pub struct RustAstWrapper {
    debug: bool,
}

impl RustAstWrapper {
    pub fn new() -> Self { Self { debug: false } }
    pub fn with_debug(mut self, debug: bool) -> Self { self.debug = debug; self }
}
```

✅ **String-Based Wrapping** (lines 75-112)
```rust
pub fn wrap(&self, cache_key: impl AsRef<str>, user_code: impl AsRef<str>) -> Result<String> {
    // Validates cache_key is valid identifier
    // Generates:
    // #[no_mangle]
    // pub extern "C" fn oxur_eval_{cache_key}() {
    //     {user_code}
    // }
}
```

✅ **Cache Key Validation** (lines 146-162)
```rust
fn is_valid_identifier(s: &str) -> bool {
    // Checks: alphabetic/underscore first char, alphanumeric rest
}
```

✅ **Error Handling** (lines 12-22)
```rust
#[derive(Debug, Error)]
pub enum WrapperError {
    InvalidCode(String),
    NameConflict(String),
}
```

✅ **11 Unit Tests** (lines 164-286)
- test_wrapper_creation
- test_wrapper_with_debug
- test_wrap_simple_code
- test_wrap_multiline_code
- test_wrap_with_debug
- test_wrap_invalid_cache_key
- test_wrap_valid_cache_keys
- test_is_valid_identifier
- test_extract_variables_stub
- test_wrap_with_store_stub
- test_default

**All 11 tests passing!**

---

## What's MISSING (Phase 2 Requirements)

### 1. VariableStore Integration ❌

**Current (Stub - line 114-127):**
```rust
pub fn wrap_with_store(
    &self,
    cache_key: impl AsRef<str>,
    user_code: impl AsRef<str>,
    _variables: &[String],  // IGNORED!
) -> Result<String> {
    // Phase 3: Simple wrapping without variable access
    // Phase 4: Will add variable store integration
    self.wrap(cache_key, user_code)  // Just delegates to basic wrap
}
```

**Needed:**
```rust
pub fn wrap_with_store(
    &self,
    cache_key: impl AsRef<str>,
    user_code: impl AsRef<str>,
    variables: &[(String, String)],  // (name, type)
) -> Result<String> {
    // Generate:
    // #[no_mangle]
    // pub extern "C" fn oxur_eval_{cache_key}() {
    //     // Load variables from store
    //     let x: i32 = oxur_repl::subprocess::with_store(|store| {
    //         store.get("x").cloned().unwrap()
    //     });
    //
    //     {user_code}
    //
    //     // Store variables back
    //     oxur_repl::subprocess::with_store(|store| {
    //         store.set("x".to_string(), x);
    //     });
    // }
}
```

### 2. Type Inference Integration ❌

**Current (Stub - line 129-136):**
```rust
pub fn extract_variables(&self, _user_code: impl AsRef<str>) -> Vec<String> {
    // Phase 3 stub
    Vec::new()  // Always returns empty!
}
```

**Needed:**
- Integration with `src/type_inference.rs`
- Call `TypeInference::infer_types()` to get variable types
- Return `Vec<(String, String)>` with (name, type) pairs
- Use for generating properly typed variable loads/stores

### 3. AST-Based Wrapping ❌

**Current:** String concatenation (line 88-109)

**Needed:**
- Use `syn` to parse user code into AST
- Use `quote` to generate wrapper code
- Proper AST manipulation for:
  - Inserting variable loads before user code
  - Inserting variable stores after user code
  - Preserving spans for error reporting

### 4. Source Map Comment Generation ❌

**Current:** No source map integration

**Needed:**
- Accept `SourceMap` parameter
- Generate `/* oxur_node=123 */` comments
- Embed NodeId annotations in generated code
- Enable ErrorTranslator to map Rust errors back to Oxur source

### 5. Session State Integration ❌

**Current:** No session awareness

**Needed:**
- Accept `SessionState` parameter
- Track which variables are in scope
- Generate loads only for variables that exist
- Detect variable shadowing/conflicts

---

## Architecture Gap Analysis

### Current Architecture (Stub)

```
User Code (string)
    ↓
RustAstWrapper::wrap()
    ↓
String Concatenation
    ↓
Wrapped Code (string)
    ↓
rustc compilation
```

**Limitations:**
- ❌ No variable access (isolated execution)
- ❌ No type information
- ❌ No error position mapping
- ❌ Cannot implement REPL state persistence

### Required Architecture (Phase 2)

```
User Code (string)
    ↓
parse with syn
    ↓
User AST
    ↓
TypeInference::infer_types()  ← SessionState (existing vars)
    ↓
(name, type) pairs
    ↓
RustAstWrapper::wrap_with_store()
    ↓
AST Transformation (quote!):
  - Add variable loads
  - User code
  - Add variable stores
    ↓
SourceMap::annotate()  ← Add /* oxur_node=X */ comments
    ↓
Generated AST
    ↓
quote::to_string()
    ↓
Wrapped Code (string)
    ↓
rustc compilation
```

**Benefits:**
- ✅ Variables persist across evaluations
- ✅ Type-safe variable access
- ✅ Error positions map to Oxur source
- ✅ Full REPL semantics

---

## Dependencies Analysis

### Current Dependencies

**From Cargo.toml:**
```toml
syn = { version = "2.0", features = ["full", "extra-traits"] }
quote = "1.0"
```

✅ Already available!

### Additional Dependencies Needed

None! All required dependencies already in Cargo.toml.

---

## Integration Points

### With Existing Components

**1. TypeInference (src/type_inference.rs)**
- **Status:** ✅ Already implemented (242 tests)
- **Integration:** Call `TypeInference::infer_types()` to get variable types
- **Input:** User code AST, SessionState
- **Output:** `Vec<(String, String)>` (name, type)

**2. VariableStore (src/subprocess/variable_store.rs)**
- **Status:** ✅ Fully implemented (Phase 1)
- **Integration:** Generate code that calls `with_store(|store| { ... })`
- **Operations:** `store.get()`, `store.set()`

**3. SourceMap (oxur-smap crate)**
- **Status:** ⚠️ Not yet integrated (Phase 4)
- **Integration:** Add `/* oxur_node=X */` comments to generated code
- **Purpose:** Enable error position translation

**4. SessionState (src/session/dir.rs)**
- **Status:** ✅ Already implemented
- **Integration:** Pass to wrapper for variable scope tracking
- **Purpose:** Know which variables exist in current session

---

## Phase 2 Implementation Plan

### Task 2.1: Upgrade wrap() to Use syn/quote ✅

**Goal:** Replace string concatenation with proper AST generation

**Steps:**
1. Parse user_code with `syn::parse_str::<syn::File>()`
2. Extract statements/expressions
3. Use `quote!` to generate wrapper function
4. Test round-trip (parse → generate → parse)

**Estimated:** 1-2 days

### Task 2.2: Implement Variable Extraction ✅

**Goal:** Make `extract_variables()` actually work

**Steps:**
1. Parse user code into AST
2. Walk AST to find let bindings
3. Call TypeInference to get types
4. Return `Vec<(String, String)>`

**Estimated:** 1-2 days

### Task 2.3: Implement wrap_with_store() ✅

**Goal:** Generate VariableStore load/store code

**Steps:**
1. Parse user code into AST
2. Generate variable load statements (before user code)
3. Insert user code
4. Generate variable store statements (after user code)
5. Use quote! to assemble complete function

**Estimated:** 2-3 days

### Task 2.4: Add Source Map Support (Stub) ⏸️

**Goal:** Prepare for Phase 4 integration

**Steps:**
1. Add `source_map: Option<&SourceMap>` parameter
2. If present, generate `/* oxur_node=X */` comments
3. If absent, skip (Phase 4 will provide)

**Estimated:** 1 day

**Note:** Full implementation in Phase 4

### Task 2.5: Integration Tests ✅

**Goal:** Test end-to-end with real compilation

**Steps:**
1. Generate wrapped code
2. Compile to dynamic library
3. Load library in subprocess
4. Execute and verify variable persistence

**Estimated:** 2-3 days

---

## Testing Strategy

### Current Tests (11 passing)

All test basic string wrapping functionality. Keep these!

### New Tests Needed

**Unit Tests:**
1. test_wrap_with_syn_quote() - AST-based wrapping
2. test_extract_variables_simple() - Find let bindings
3. test_extract_variables_shadowing() - Handle shadowing
4. test_wrap_with_store_single_var() - One variable
5. test_wrap_with_store_multiple_vars() - Multiple variables
6. test_wrap_with_store_typed() - Type annotations
7. test_generated_code_compiles() - Rustc validation
8. test_variable_load_code_generation() - Load statements
9. test_variable_store_code_generation() - Store statements

**Integration Tests:**
10. test_end_to_end_variable_persistence() - Full pipeline
11. test_compilation_and_execution() - Real subprocess
12. test_type_inference_integration() - TypeInference call

**Target:** 20+ unit tests, 3+ integration tests

---

## Risk Analysis

### Low Risk ✅

- **syn/quote dependencies:** Already used successfully in project
- **VariableStore API:** Already complete and tested
- **TypeInference API:** Already complete and tested

### Medium Risk ⚠️

- **AST manipulation complexity:** quote! macro can be tricky
  - **Mitigation:** Start simple, add complexity incrementally
  - **Mitigation:** Use oxur-ast test examples as reference

- **Type annotation generation:** Need to stringify types correctly
  - **Mitigation:** Use TypeInference output directly
  - **Mitigation:** Test with common types first (i32, String, Vec<T>)

### High Risk ⚠️⚠️

- **Scope handling:** Need to track variable shadowing
  - **Mitigation:** Use SessionState for scope tracking
  - **Mitigation:** Add comprehensive tests for shadowing

- **Lifetime management:** VariableStore uses 'static
  - **Mitigation:** All loaded values must be 'static
  - **Mitigation:** Document lifetime requirements

---

## Code Quality Checklist

### Rust Best Practices

- [ ] No anti-patterns (check against 11-anti-patterns.md)
- [ ] Proper error handling (thiserror ✅ already used)
- [ ] All public functions documented
- [ ] Examples in doc comments
- [ ] No unwrap() in library code
- [ ] Prefer &str over &String (AP-02)
- [ ] Use Result combinators (ID-27)

### Testing

- [ ] 95%+ code coverage
- [ ] All error paths tested
- [ ] Edge cases covered
- [ ] Integration tests with real compilation

### Documentation

- [ ] Update module-level docs
- [ ] Add examples for new methods
- [ ] Reference ODD-0038 and ODD-0040
- [ ] Update CLAUDE.md if needed

---

## Estimated Timeline

**Phase 2 Breakdown:**

| Task | Duration | Dependencies |
|------|----------|--------------|
| 2.1: syn/quote upgrade | 1-2 days | None |
| 2.2: Variable extraction | 1-2 days | Task 2.1 |
| 2.3: wrap_with_store() | 2-3 days | Tasks 2.1, 2.2 |
| 2.4: Source map stub | 1 day | Task 2.3 |
| 2.5: Integration tests | 2-3 days | Tasks 2.1-2.4 |
| **Total** | **7-11 days** | **1.5-2 weeks** |

**Fits ODD-0040 estimate:** 2-3 weeks (includes buffer)

---

## Success Criteria

### Functional Requirements

- [ ] `wrap()` uses syn/quote (not string concatenation)
- [ ] `extract_variables()` returns actual variable list with types
- [ ] `wrap_with_store()` generates VariableStore integration code
- [ ] Generated code compiles without errors
- [ ] Variables persist across multiple evaluations
- [ ] Type information flows from TypeInference

### Quality Requirements

- [ ] 20+ unit tests (all passing)
- [ ] 3+ integration tests (all passing)
- [ ] 95%+ code coverage
- [ ] Zero clippy warnings
- [ ] All documentation complete

### Integration Requirements

- [ ] Works with TypeInference
- [ ] Works with VariableStore
- [ ] Works with SessionState
- [ ] Ready for Phase 4 SourceMap integration

---

## Example Generated Code

### Input (Oxur)

```oxur
(def x 42)
(def y (+ x 10))
y
```

### After Lowering (Rust AST from oxur-comp)

```rust
let x: i32 = 42;
let y: i32 = x + 10;
y
```

### After Wrapping (Phase 2 RustAstWrapper)

```rust
// Generated by Oxur REPL wrapper
#![allow(unused)]

use oxur_repl::subprocess::with_store;

#[no_mangle]
pub extern "C" fn oxur_eval_abc123() {
    // Load variables from store
    let x: i32 = with_store(|store| {
        store.get::<i32>("x")
            .cloned()
            .expect("Variable x not found")
    });

    // User code (from oxur-comp lowering)
    let y: i32 = x + 10;
    let _result = y;

    // Store variables back
    with_store(|store| {
        store.set("x".to_string(), x);
        store.set("y".to_string(), y);
        store.set("_".to_string(), _result);
    });
}
```

### After Compilation

```
libeval_abc123.so  (dynamic library)
```

### Execution

```rust
let mut executor = SubprocessExecutor::new()?;
executor.load_library(&path, "abc123")?;
let result = executor.execute("abc123")?;  // y = 52
```

---

## Next Steps

### Immediate (Start Phase 2)

1. **Read existing TypeInference code**
   - Understand `infer_types()` API
   - Check input/output format
   - See how it integrates with SessionState

2. **Create Task 2.1 branch**
   - Start with syn/quote upgrade
   - Keep existing tests passing
   - Add new AST-based tests

3. **Incremental Development**
   - Commit after each working task
   - Run tests frequently
   - Check coverage after each task

### Phase 2 Roadmap

```
Current
    ↓
Task 2.1 (syn/quote)
    ↓
Task 2.2 (variable extraction)
    ↓
Task 2.3 (wrap_with_store)
    ↓
Task 2.4 (source map stub)
    ↓
Task 2.5 (integration tests)
    ↓
Phase 2 Complete! 🎉
    ↓
Phase 3 (EvalContext Integration)
```

---

## Comparison to Plan (ODD-0040)

### Plan Says (lines 554-756)

Phase 2 should implement:
- ✅ RustAstWrapper struct (EXISTS as stub)
- ❌ AST wrapping with syn/quote (MISSING)
- ❌ VariableStore integration (STUBBED)
- ❌ Type inference integration (STUBBED)
- ❌ Source map comment generation (MISSING)

### Actual Status

**~30% complete:**
- ✅ Basic structure (30%)
- ❌ Core functionality (0%)

**Good news:** Foundation is solid, just needs implementation!

---

## Lessons from Phase 1

### Apply These Patterns

1. **Check for existing code first** ✅ (Found stub!)
2. **Maintain test quality** ✅ (11 tests already)
3. **Document as you go** ✅ (Good docs exist)
4. **Incremental commits** ✅ (After each task)

### Avoid These Issues

1. **Don't assume code is complete** (stub ≠ complete)
2. **Check test coverage** (stubs have low coverage)
3. **Verify functionality** (passing tests ≠ working feature)

---

## Sign-Off

**Phase 2 Status:** 🔄 **IN PROGRESS** (~30% complete)

**What Exists:** ✅ Stub implementation (286 lines, 11 tests)

**What's Needed:** ❌ Full AST-based implementation with VariableStore

**Blockers:** None (all dependencies ready)

**Risk:** Medium (AST manipulation, scope handling)

**Estimated Completion:** 1.5-2 weeks

**Next Task:** Task 2.1 - Upgrade to syn/quote

**Ready to Code:** ✅ **YES** (after user approval)

---

**Report Generated By:** Claude Code (Sonnet 4.5)
**Implementation Plan:** ODD-0040 v1.0
**Architecture Spec:** ODD-0038 v1.2
**Phase 2 Analysis:** Stub → Full Implementation Required

**Time to get to work on the real coding challenge!** 💪
