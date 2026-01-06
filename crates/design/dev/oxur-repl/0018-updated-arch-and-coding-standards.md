# Oxur REPL Enhancement Plan

**Date:** 2026-01-05
**Audited By:** Claude Sonnet 4.5
**Design Doc:** ODD-0026 v2.0 (Oxur REPL Evaluation Strategy)
**Rust Guidelines:** ai-rust-skill guides (11-anti-patterns.md, 01-core-idioms.md)

## Executive Summary

Audit of `oxur-repl` crate reveals significant gaps between current implementation and design specification ODD-0026 v2.0. The codebase needs:

1. **Architecture alignment** - Missing 7 core components specified in design doc
2. **Tier model correction** - Implementing 3-tier vs current 2-tier execution
3. **Type integration** - Replace local Position with oxur-smap::SourcePos
4. **Caching strategy** - Implement two-level caching (disk + memory)
5. **Rust best practices** - Fix several anti-pattern violations

**Impact:** Implementation is in early stage with stub/simulation code. Plan brings it into full alignment with approved architecture.

---

## Critical Issues

### Issue 1: Missing Components (Architecture Gap)

**Severity:** CRITICAL
**Violates:** ODD-0026 Sections 2.1, 3.1-3.4, 5.1-5.2, 9

**Current State:**
- Only `EvalContext` exists with inline simulation code
- No separate compiler, executor, cache, or subprocess components
- Tier 2 execution is stubbed with `tokio::time::sleep()`

**Required Components (all missing):**

1. **CachedCompiler** (`src/compiler/cached.rs`)
   - Manages compilation pipeline
   - Integrates with ArtifactCache
   - Delegates to SubprocessExecutor
   - Location specified: ODD-0026 Section 2.1, line 261

2. **SubprocessExecutor** (`src/executor/subprocess.rs`)
   - MANDATORY for Ctrl-C support (ODD-0038 Decision 3)
   - Manages child process lifecycle
   - Loads dynamic libraries
   - IPC protocol over stdin/stdout
   - Location specified: ODD-0026 Section 3.2, line 394

3. **ArtifactCache** (`src/cache/artifact.rs`)
   - Global disk-based cache (~/.cache/oxur/artifacts/)
   - SHA256 content-addressed storage
   - Persistence across sessions
   - MANDATORY Phase 0 (ODD-0038 Decision 5)
   - Location specified: ODD-0026 Section 5.1, line 640

4. **SessionDir** (`src/session/dir.rs`)
   - Tmpfs-backed session directories
   - Graceful fallback to OS temp
   - Environment variable override
   - Location specified: ODD-0026 Section 9, line 940

5. **RustAstWrapper** (`src/wrapper.rs`)
   - Wraps Rust AST with REPL scaffolding
   - Not responsible for lowering (that's oxur-comp)
   - Generates entry points for dylib
   - Location specified: ODD-0026 Section 2.1, line 303

6. **TypeInference** (`src/type_inference.rs`)
   - Uses rust-analyzer for REPL variable types
   - Avoids 4 years of compiler error hacks (evcxr lesson)
   - ODD-0038 Decision 6
   - Location specified: ODD-0026 Section 2.1, line 269

7. **VariableStore** (`src/subprocess/variable_store.rs`)
   - Type-erased storage pattern from evcxr
   - Lives in subprocess, accessed via global static
   - `Box<dyn Any + 'static>` storage
   - Location specified: ODD-0026 Section 3.1, line 348

8. **Subprocess Binary** (`src/bin/subprocess.rs`)
   - Separate binary target within oxur-repl crate
   - Loads dylibs and executes user code
   - IPC protocol handler
   - Location specified: ODD-0026 Section 3.3, line 460

**Rust Best Practice Violations:**
- **AP-09:** Inline stubs with `unwrap()` in library code (context.rs:315, 324)
- **AP-24:** Premature inline implementation without architecture
- **ID-31:** Logic that should be in separate modules is in EvalContext

**Action Required:**
Create all 8 missing components with proper separation of concerns

**Effort:** 5-7 days (Phase 0 work per ODD-0026 Section 8)

---

### Issue 2: Tier Model Mismatch

**Severity:** HIGH
**Violates:** ODD-0026 Section 2 (Three-Tier Execution Model), ID-29 (Enum completeness)

**Current State:**
```rust
// crates/oxur-repl/src/eval/context.rs:79-88
pub enum ExecutionTier {
    /// Tier 1: Calculator mode
    Calculator,

    /// Tier 2: Cached compilation
    CachedCompilation,
}
```

**Issue:** Missing distinction between:
- **Tier 2:** Cache hit, library already loaded in subprocess (~1-5ms)
- **Tier 3:** Cache miss or not loaded, must compile/load (~50-300ms)

**Design Doc Specification (lines 100-129):**
```
Tier 1: Calculator Mode (interpret literal arithmetic)
    ↓ (if not literal arithmetic)
Tier 2: Cached Compilation (execute from loaded library)
    ↓ (if not cached or not loaded)
Tier 3: JIT Compilation (compile, cache, load, execute)
```

**Performance Impact:**
- Users can't distinguish between "instant" cached execution and "slow" compilation
- No ability to show progress indicators for >200ms compiles
- Cache effectiveness metrics are incorrect

**Rust Best Practice Violations:**
- **ID-01:** Enum should be `#[non_exhaustive]` for future extensibility
- **AP-42:** Poor variant naming obscures semantic differences

**Action Required:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExecutionTier {
    /// Tier 1: Interpret literal arithmetic (<1ms)
    Calculator,

    /// Tier 2: Execute from loaded library (~1-5ms)
    CachedLoaded,

    /// Tier 3: Compile and load (~50-300ms)
    JustInTime,
}
```

Update EvalContext to track library load state and make tier decision accordingly.

**Effort:** 2 hours (requires SubprocessExecutor to track loaded libraries)

---

### Issue 3: Position vs SourcePos Type Mismatch

**Severity:** MEDIUM
**Violates:** ODD-0026 Section 2.1, DRY principle, ID-08 (avoid duplicate types)

**Current State:**
- Local `Position` type defined in `context.rs:13-50`
- oxur-smap crate provides `SourcePos` with identical purpose
- Both types have same fields: offset, line, column
- Both implement Display, Default, Clone, PartialEq, Eq, Hash

**Design Doc Requirement (line 161):**
```rust
// Location: oxur-repl/src/eval/context.rs
// Part of: EvalContext

use oxur_smap::{SourceMap, SourcePos};  // Should use SourcePos!
```

**Code Duplication:**
```rust
// Current: context.rs
pub struct Position {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

// Should use: oxur-smap/src/source_pos.rs (already exists!)
pub struct SourcePos {
    // Same fields, same purpose
}
```

**Consequences:**
- Violates DRY principle
- Breaks integration with source mapping pipeline
- Error messages won't trace back to original Oxur source
- Two types that should be unified

**Rust Best Practice Violations:**
- **AP-30:** Using custom string-like type when standard exists
- **Type proliferation:** Creates unnecessary conversion overhead
- **ID-08:** Should leverage existing types from dependencies

**Action Required:**
1. Remove local `Position` type from `context.rs`
2. Replace all `Position` uses with `oxur_smap::SourcePos`
3. Update `EvalError` variants to use `SourcePos`
4. Update tests to use `SourcePos::repl()` constructor

**Effort:** 1 hour (straightforward type replacement)

---

### Issue 4: Inadequate Cache Implementation

**Severity:** HIGH
**Violates:** ODD-0026 Section 5 (Two-Level Caching Strategy)

**Current State:**
```rust
// crates/oxur-repl/src/eval/context.rs:130
cache: HashMap<String, String>,  // In-memory only, string keys, no persistence
```

**Design Doc Specification (lines 640-740):**

1. **Global Artifact Cache** (disk-based, persistent)
   - Location: `~/.cache/oxur/artifacts/`
   - Keys: SHA256(source + deps + opt_level + source_map)
   - Storage: `.so`/`.dylib`/`.dll` files per cache key
   - Shared across all sessions
   - Mandatory Phase 0 per ODD-0038 Decision 5

2. **Session Library Cache** (in-memory)
   - Tracks which libraries loaded in current subprocess
   - Enables Tier 2 vs Tier 3 decision
   - Part of SubprocessExecutor
   - `HashSet<String>` of loaded cache keys

**Current Limitations:**
- No persistence (lose all cached compilations on restart)
- Simple hash instead of SHA256 content addressing
- No actual compiled artifacts, just string results
- No cross-session sharing
- No disk storage

**Rust Best Practice Violations:**
- **AP-08:** Using `DefaultHasher` instead of cryptographic hash
  ```rust
  // context.rs:378-386
  use std::collections::hash_map::DefaultHasher;  // WRONG for cache keys
  ```
- **AP-63:** Should use crypto hash for content addressing
- **ID-36:** Missing proper persistent cache with RAII cleanup

**Action Required:**
1. Create `ArtifactCache` with SHA256 keying
2. Implement disk persistence in `~/.cache/oxur/`
3. Add cache index for fast lookups
4. Integrate with `SubprocessExecutor` for loaded library tracking
5. Add `OXUR_CACHE_DIR` environment variable override

**Dependencies to Add:**
```toml
sha2 = "0.10"           # For SHA256 hashing
dirs = "5.0"            # For cache directory discovery
```

**Effort:** 1 day (ArtifactCache implementation + integration)

---

### Issue 5: Missing Subprocess Binary Target

**Severity:** CRITICAL (blocks execution isolation)
**Violates:** ODD-0026 Section 3.3, ODD-0038 Decision 3

**Current State:**
- No `[[bin]]` section in Cargo.toml
- No `src/bin/subprocess.rs` file
- Subprocess execution is simulated in EvalContext

**Design Doc Requirement (lines 462-505):**

**Cargo.toml:**
```toml
[[bin]]
name = "oxur-repl-subprocess"
path = "src/bin/subprocess.rs"
```

**Binary Implementation:**
```rust
// src/bin/subprocess.rs
// IPC protocol: LOAD <path>, RUN <cache_key>
// Response markers: OXUR_EXECUTION_COMPLETE, OXUR_RUNTIME_ERROR, OXUR_PANIC_LOCATION
```

**Why MANDATORY (ODD-0038 Decision 3):**
- Rust threads cannot be interrupted (no pthread_cancel)
- Ctrl-C support requires process isolation
- User code crashes don't corrupt REPL state
- evcxr evidence: Subprocess from day one, never changed

**Consequences of Missing:**
- No Ctrl-C interruption support
- Crashes in user code kill entire REPL
- No isolation of compilation artifacts
- Cannot implement VariableStore pattern

**Rust Best Practice Violations:**
- **ID-18:** Missing RAII cleanup (subprocess lifecycle)
- **AP-09:** Cannot catch panics in user code without subprocess
- **Safety:** Violates isolation principle

**Action Required:**
1. Add `[[bin]]` section to Cargo.toml
2. Implement `src/bin/subprocess.rs` with:
   - IPC protocol handler (stdin/stdout)
   - Dynamic library loading (libloading crate)
   - Function execution with panic catching
   - Output redirection
3. Update SubprocessExecutor to spawn this binary
4. Implement protocol markers (OXUR_* not EVCXR_*)

**Dependencies to Add:**
```toml
libloading = "0.8"      # For dylib loading in subprocess
```

**Effort:** 2 days (binary + protocol + testing)

---

## Medium Priority Issues

### Issue 6: Incomplete Error Handling

**Severity:** MEDIUM
**Violates:** AP-09, AP-28, ID-13

**Current Issues:**
1. `unwrap()` in simulation code (context.rs:315, 324)
2. Placeholder error messages lack position info
3. No error translation from rustc errors back to Oxur source

**Action Required:**
- Replace all `unwrap()` with proper error propagation
- Implement error translation using SourceMap
- Add context to all error messages

**Effort:** 4 hours

---

### Issue 7: Missing Dependencies

**Severity:** MEDIUM
**Violates:** Build requirements

**Current Cargo.toml Missing:**
```toml
sha2 = "0.10"           # SHA256 for cache keys
dirs = "5.0"            # Cache dir discovery
libloading = "0.8"      # Dylib loading
```

**Action Required:**
Add missing dependencies to support full implementation

**Effort:** 10 minutes

---

## Low Priority Issues

### Issue 8: Documentation Completeness

**Severity:** LOW
**Violates:** ID-14, ID-19

**Issues:**
- Some functions lack doc comments
- Examples in docs don't show full context
- Missing module-level documentation in some modules

**Action Required:**
- Add doc comments to all public items
- Ensure first sentence is one line ~15 words
- Add runnable examples where appropriate

**Effort:** 3 hours

---

### Issue 9: Missing Standard Derives

**Severity:** LOW
**Violates:** ID-11

**Issues:**
- `ExecutionStats` could have more derives
- Some error types could benefit from PartialEq for testing

**Action Required:**
Add appropriate derives where semantically correct

**Effort:** 30 minutes

---

## Implementation Plan

### Phase 0: Foundation (Week 1)

**Prerequisites:**
- [x] Audit complete
- [ ] Enhancement plan reviewed
- [ ] Dependencies added

**Tasks:**
1. Add missing dependencies (sha2, dirs, libloading)
2. Remove local Position type, use oxur-smap::SourcePos
3. Update ExecutionTier enum to 3-tier model
4. Create stub modules for missing components

**Deliverable:** Compiles with updated types, no functionality change

**Effort:** 1 day

---

### Phase 1: Core Components (Week 1-2)

**Tasks:**
1. Implement `ArtifactCache` with disk persistence
2. Implement `SessionDir` with tmpfs strategy
3. Implement `SubprocessExecutor` (stub, no actual subprocess yet)
4. Implement `VariableStore` with type erasure

**Deliverable:** Components exist and tested in isolation

**Effort:** 3 days

---

### Phase 2: Subprocess Integration (Week 2)

**Tasks:**
1. Implement `src/bin/subprocess.rs` binary
2. Add `[[bin]]` target to Cargo.toml
3. Implement IPC protocol (stdin/stdout)
4. Integrate SubprocessExecutor with child process
5. Test library loading and execution

**Deliverable:** Can execute compiled code in subprocess

**Effort:** 2 days

---

### Phase 3: Compilation Pipeline (Week 3)

**Tasks:**
1. Implement `CachedCompiler`
2. Implement `RustAstWrapper`
3. Implement `TypeInference` (stub for now)
4. Wire up full compilation pipeline
5. Integrate with EvalContext

**Deliverable:** End-to-end compilation working

**Effort:** 3 days

---

### Phase 4: Polish & Testing (Week 3-4)

**Tasks:**
1. Comprehensive testing of all tiers
2. Error message improvements
3. Documentation updates
4. Performance optimization
5. Cache eviction strategy

**Deliverable:** Production-ready REPL evaluation

**Effort:** 2 days

---

## Total Effort Estimate

- **Phase 0:** 1 day
- **Phase 1:** 3 days
- **Phase 2:** 2 days
- **Phase 3:** 3 days
- **Phase 4:** 2 days

**Total:** 11 working days (~2.2 weeks)

---

## Rust Best Practices Summary

### Violations Found

1. **AP-08:** DefaultHasher instead of SHA256 (context.rs:380)
2. **AP-09:** unwrap() in library code (context.rs:315, 324)
3. **AP-24:** Premature inline implementation
4. **AP-30:** Custom type when standard exists (Position vs SourcePos)
5. **AP-63:** Unnecessary clones in cache lookup
6. **ID-01:** Missing #[non_exhaustive] on ExecutionTier
7. **ID-08:** Type duplication (Position)
8. **ID-11:** Missing standard derives
9. **ID-13:** Missing proper panic handling (no subprocess)
10. **ID-14:** Incomplete documentation
11. **ID-18:** Missing RAII patterns (subprocess lifecycle)
12. **ID-19:** First sentence not one line in some docs
13. **ID-31:** Too much logic in associated functions vs modules

### Compliance Summary

- **Critical Violations:** 5 (AP-09, AP-24, ID-13, ID-18, ID-31)
- **High Priority:** 4 (AP-08, AP-30, AP-63, ID-01)
- **Low Priority:** 4 (ID-08, ID-11, ID-14, ID-19)

---

## Success Criteria

### Architecture Alignment

- [ ] All 8 components exist and integrate correctly
- [ ] Three-tier execution model implemented
- [ ] Subprocess isolation working with Ctrl-C support
- [ ] Two-level caching (disk + memory) operational
- [ ] SourcePos integration complete

### Performance Targets (from ODD-0026)

- [ ] Tier 1 (Calculator): <1ms
- [ ] Tier 2 (Cached, loaded): <5ms
- [ ] Tier 3 (JIT): <300ms cold, <100ms warm with cache
- [ ] Cache hit rate: >80% for typical usage

### Quality Targets

- [ ] All critical Rust anti-patterns fixed
- [ ] Test coverage: ≥95%
- [ ] All public APIs documented
- [ ] No compiler warnings
- [ ] Clippy passes with no warnings

---

## References

- **Design Doc:** ODD-0026 v2.0 (Oxur REPL Evaluation Strategy)
- **Architecture:** ODD-0038 v1.2 (REPL Architecture Overview)
- **Rust Guidelines:** `assets/ai/ai-rust/guides/`
  - 11-anti-patterns.md
  - 01-core-idioms.md
  - 02-api-design.md
  - 03-error-handling.md
- **Oxur Conventions:** CLAUDE.md

---

## Appendix: Code Locations

### Existing Files (to be updated)
- `crates/oxur-repl/src/lib.rs`
- `crates/oxur-repl/src/eval/context.rs`
- `crates/oxur-repl/src/eval/mod.rs`
- `crates/oxur-repl/Cargo.toml`

### Files to Create
- `crates/oxur-repl/src/compiler/mod.rs`
- `crates/oxur-repl/src/compiler/cached.rs`
- `crates/oxur-repl/src/executor/mod.rs`
- `crates/oxur-repl/src/executor/subprocess.rs`
- `crates/oxur-repl/src/cache/mod.rs`
- `crates/oxur-repl/src/cache/artifact.rs`
- `crates/oxur-repl/src/session/mod.rs`
- `crates/oxur-repl/src/session/dir.rs`
- `crates/oxur-repl/src/wrapper.rs`
- `crates/oxur-repl/src/type_inference.rs`
- `crates/oxur-repl/src/subprocess/mod.rs`
- `crates/oxur-repl/src/subprocess/variable_store.rs`
- `crates/oxur-repl/src/bin/subprocess.rs`

---

**End of Enhancement Plan**
