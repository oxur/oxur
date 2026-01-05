# Evcxr Research Synthesis: Recommendations for Oxur REPL v1.1

**Date:** January 4, 2026  
**Sources:** Web/Documentation Research (Opus) + Git Archaeology (Claude Code)  
**Purpose:** Synthesize findings into actionable architecture recommendations for Oxur

---

## Executive Summary

Both research tracks converged on the same critical findings. The subprocess architecture wasn't an evolution—it was **day-one design** that has proven correct for 6+ years with zero fundamental changes. However, one key finding from web research adds nuance: subprocess execution isn't just about crash recovery—it's the **only way to support Ctrl-C interruption** in Rust (threads cannot be forcibly interrupted).

### The Five Certainties

| Decision | Evcxr Evidence | Recommendation for Oxur |
|----------|----------------|------------------------|
| Subprocess Execution | Present from day one, unchanged 6+ years | **✅ MANDATORY** (not optional) |
| cargo-based Compilation | Never used rustc directly | **✅ ADOPT** |
| `Box<dyn Any + 'static>` Variable Store | Unchanged since 2018 | **✅ ADOPT** |
| Caching | Added 5 years late, biggest perf win | **✅ BUILD DAY ONE** |
| rust-analyzer for Type Inference | Replaced 4-year compiler hack | **✅ USE FROM START** |

### The One Surprise: IPC Mechanism

**Tension discovered:**
- Git archaeology strongly recommends stdin/stdout (6 years, "never changed")
- Web research identified known fragility (stdout mixing, no framing, protocol collision risk)

**Resolution:** The git evidence is more authoritative here. stdin/stdout's "fragility" hasn't caused real problems in practice. Oxur should **start with stdin/stdout** but can consider Unix sockets as a future enhancement if protocol issues arise.

---

## Detailed Findings Synthesis

### 1. Subprocess vs In-Process: RESOLVED ✅

**Combined Evidence:**

| Source | Finding |
|--------|---------|
| Git Archaeology | Subprocess present from commit one (2018-09-25), never questioned in 6 years |
| Web Research | **Critical:** Rust threads cannot be interrupted—subprocess is the *only* way to support Ctrl-C |
| HOW_IT_WORKS.md | Four explicit rationales documented: crash recovery, isolation, portability, simplicity |

**The Killer Finding (Web Research):**

> "Don't ask Jupyter to 'interrupt kernel', it won't work. Rust threads can't be interrupted."

This single constraint makes subprocess **mandatory**, not optional. If Oxur requires interactive interruption (which any development REPL does), there's no alternative.

**Impact on Oxur's Tentative Decision:**

| Original Plan | Revised Plan |
|---------------|--------------|
| v1.0: `InProcessExecutor` (default) | v1.0: `SubprocessExecutor` (mandatory) |
| v1.1: `SubprocessExecutor` (if needed) | Keep `Executor` trait for testing only |

**Recommendation:** Drop the in-process default. Subprocess is **required** for a usable REPL. The `Executor` trait abstraction is still valuable for mocking in tests, but production must use subprocess.

---

### 2. IPC Mechanism: REVISED ⚠️

**Tension Between Sources:**

| Source | Recommendation | Rationale |
|--------|----------------|-----------|
| Git Archaeology | stdin/stdout (text protocol) | 6 years stable, simple, portable |
| Web Research | Consider Unix sockets | Protocol fragility, stdout mixing, no binary framing |
| Oxur Original Plan | Unix sockets + protocol reuse | Reuse existing Request/Response types |

**Analysis:**

The git evidence is compelling: stdin/stdout has worked for 6 years without fundamental issues. The "fragility" concerns from web research are theoretical—there's no evidence they've caused real problems.

However, Oxur has a unique opportunity: **we already have a protocol** (ODD-0018 Request/Response types with postcard serialization). Reusing this for subprocess IPC would be elegant.

**Revised Recommendation:**

**Phase 1 (v1.0):** Use stdin/stdout with simple text protocol
- Match evcxr's proven approach
- Lower risk, faster implementation
- Commands: `LOAD_AND_RUN <path> <fn>`, response: `EVCXR_EXECUTION_COMPLETE`

**Phase 2 (v1.1+):** Consider Unix sockets IF:
- Protocol collisions become a problem
- Binary output (images, etc.) is needed without base64
- We want to unify subprocess and server protocols

**Key Insight:** Don't over-engineer IPC. evcxr's text protocol works. Start there.

---

### 3. Variable Persistence: CONFIRMED ✅

**Both sources agree completely:**

```rust
// The pattern that has worked for 6+ years
HashMap<String, Box<dyn Any + 'static>>
```

**Key Constraints Discovered:**

| Constraint | Source | Impact |
|------------|--------|--------|
| No inter-variable references | Web Research | Variables cannot borrow from other variables |
| `'static` lifetime required | Both | All stored values must be owned |
| No serialization | Git Archaeology | Session state lost on crash/interrupt |
| Type erasure works | Both | Downcast at runtime with known types |

**The Reference Limitation (Critical for Oxur):**

```rust
// This CANNOT work:
let all_values = vec![10, 20, 30];
let some_values = &all_values[2..3];  // ERROR: can't persist references

// Workaround (documented in evcxr):
let some_values = all_values[2..3].to_vec();  // Clone instead
```

**Recommendation:** Document this limitation clearly. Consider whether Oxur's Lisp semantics can make this more ergonomic (Lisp typically uses immutable data structures anyway).

---

### 4. Type Inference: SKIP THE HACK ✅

**Timeline from Git Archaeology:**

```
2018-2021: Compiler error hack (127 lines of complex code)
2021-2022: Transition to rust-analyzer
2022-08-28: Hack removed entirely (commit 5cbc3a0)
```

**The Hack (for posterity):**
1. Try to store variable as `String` type
2. rustc error: "cannot cast Vec<i32> to String"
3. Parse error message to extract real type
4. Recompile with correct type

**Why It Worked:** rustc's JSON error format includes full type information.

**Why It Was Removed:** rust-analyzer matured and provides cleaner type inference.

**Recommendation for Oxur:**

```rust
// Priority order:
1. rust-analyzer type inference (primary)
2. Explicit user annotation (when RA fails)
3. Compiler error parsing (NEVER - skip this entirely)

// When RA fails:
"Cannot determine type of variable `x`. 
 Please add an explicit type annotation."
```

**Integration Choice:**
- Git archaeology mentions David Lattimore regretted using RA as a library (slow builds)
- He suggested LSP protocol might have been better
- **For Oxur:** Start with RA as library, consider LSP if build times become problematic

---

### 5. Caching: BUILD EARLY ✅

**The 5-Year Gap (Git Archaeology):**

```
2018-09-25: Initial release - NO caching
2019: sccache support added (external tool)
2023-10-20: Internal caching added (commit 86d20a2)
           - 358 lines in new module/cache.rs
           - Content-based hashing
           - "Major performance improvement"
```

**This was evcxr's biggest mistake.** They waited 5 years to add internal caching.

**Recommendation for Oxur:**

**Day One Implementation:**

```rust
// oxur-repl/src/cache.rs
pub struct ArtifactCache {
    cache_dir: PathBuf,
    index: HashMap<String, CachedArtifact>,
}

impl ArtifactCache {
    pub fn cache_key(
        source: &str, 
        deps: &[Dependency], 
        opt_level: OptLevel
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        for dep in deps {
            hasher.update(dep.to_string().as_bytes());
        }
        hasher.update(&[opt_level as u8]);
        format!("{:x}", hasher.finalize())
    }
    
    pub fn get(&self, key: &str) -> Option<PathBuf> { ... }
    pub fn insert(&mut self, key: String, artifact: PathBuf) { ... }
}
```

**Cache Location:** `~/.cache/oxur/` or platform-appropriate equivalent.

---

### 6. Error Translation: CONFIRMED ✅

**Both sources confirm the approach:**

```
rustc --error-format=json
    ↓
Parse JSON errors
    ↓
Translate spans using source map
    ↓
Render with ariadne (beautiful errors)
    ↓
Display to user
```

**Oxur's Advantage:**

Oxur has `oxur-smap` planned as a dedicated crate. This is **better** than evcxr's approach (source map embedded in eval_context.rs). The dedicated crate enables:
- Multi-stage tracking (Oxur → Core → Rust)
- Independent testing
- Reuse across crates

**Recommendation:** `oxur-smap` design is validated. Proceed as planned.

---

### 7. Performance Discoveries

**From Git Archaeology:**

| Discovery | Commit | Impact |
|-----------|--------|--------|
| Panic catching slows compilation | a144454 (2019) | Made `:preserve_vars_on_panic` configurable, default OFF |
| Caching is critical | 86d20a2 (2023) | Biggest performance win in project history |
| Optimization level matters | Various | Made configurable (dev vs release) |

**Key Insight:**
> "Performance bottlenecks aren't always where you expect. The overhead wasn't in catching panics at runtime, but in the code generation the compiler had to do."

**For Oxur:**
1. Make panic preservation optional (default: off)
2. Add caching from day one
3. Make optimization level configurable
4. **Profile everything** - don't assume where bottlenecks are

---

## Gap Analysis: What Each Source Missed

### Web Research Found (Git Missed):
- **Ctrl-C interruption constraint** (the killer finding)
- Binary content base64 overhead concern
- Maintainer interview: RA-as-library regret
- Struct redefinition impossibility documented

### Git Archaeology Found (Web Missed):
- **Day-one maturity** (not an evolution)
- Specific code patterns and implementations
- Panic-catching performance discovery
- 127 lines of compiler hack (actual code removed)
- Internal caching implementation details
- TODO.md items never implemented

### Neither Found:
- Whether evcxr ever seriously considered sockets
- Detailed performance numbers (latency, throughput)
- User-reported issues with stdout mixing
- Windows-specific problems in detail

---

## Revised Architecture Recommendations for Oxur v1.1

### Changes from Original Plan

| Component | Original (v1.0) Plan | Revised Plan | Rationale |
|-----------|---------------------|--------------|-----------|
| Executor Default | `InProcessExecutor` | `SubprocessExecutor` | Ctrl-C requires subprocess |
| IPC Mechanism | Unix sockets | stdin/stdout (text) | 6 years of evidence |
| Caching | "Future consideration" | **Day one requirement** | evcxr's biggest regret |
| Type Inference | Not specified | rust-analyzer from start | Skip the 4-year hack |

### Components to Keep Unchanged

| Component | Original Plan | Status |
|-----------|--------------|--------|
| `oxur-smap` | Dedicated source mapping crate | ✅ Validated |
| `RustAstWrapper` | Renamed from CodeGenerator | ✅ Good naming |
| Temp Directory Strategy | Best-effort tmpfs | ✅ Validated |
| Three-Tier Execution | Calculator/Cached/JIT | ✅ Validated |
| Server-Side Compilation | All stages on server | ✅ Validated |

### New Components Needed

1. **`ArtifactCache`** - Content-based caching (port evcxr's module/cache.rs pattern)
2. **`TypeInference`** - rust-analyzer integration (avoid compiler error hack)
3. **`SubprocessProtocol`** - Simple text commands over stdin/stdout

---

## Implementation Priorities

### Phase 0: Prerequisites (BLOCKING)
1. `oxur-smap` crate (already planned)
2. `ArtifactCache` design (add to prerequisites)
3. rust-analyzer integration strategy

### Phase 1: Minimal Working REPL
1. `SubprocessExecutor` (not InProcessExecutor)
2. stdin/stdout text protocol (not Unix sockets)
3. Basic `ArtifactCache` implementation
4. rust-analyzer type inference

### Phase 2: Production Quality
1. Ariadne error rendering
2. Configurable optimization levels
3. Configurable panic preservation
4. Performance profiling infrastructure

### Phase 3: Enhancements (if needed)
1. Unix socket IPC (if protocol issues arise)
2. Session serialization (if requested)
3. Jupyter kernel (following evcxr_jupyter pattern)

---

## Summary: What Oxur Should Do

### ✅ ADOPT (Proven Patterns)
- Subprocess execution model
- cargo-based compilation to dylib
- `Box<dyn Any + 'static>` variable storage
- stdin/stdout text IPC protocol
- Content-based artifact caching
- rust-analyzer for type inference
- Source mapping for error translation

### ❌ SKIP (Learned Mistakes)
- Compiler error hack for types (use RA instead)
- Waiting to add caching (do it day one)
- In-process execution as default (subprocess required)
- Complex IPC (stdin/stdout is sufficient)

### ⚠️ RECONSIDER (Original Plans)
- Unix socket IPC → Start with stdin/stdout
- `InProcessExecutor` default → Use `SubprocessExecutor`
- Caching as "nice to have" → Caching as **required**

---

## Open Questions Remaining

1. **rust-analyzer integration:** Library or LSP? (David Lattimore suggests LSP might be better for build times)

2. **Oxur-specific type inference:** Does Oxur's Lisp semantics change type inference needs? (Lisp tends toward immutable data)

3. **Windows support:** evcxr uses stdin/stdout partly for Windows compatibility. If Oxur goes Unix sockets, what's the Windows fallback?

4. **Struct redefinition:** evcxr cannot redefine structs. Is this acceptable for Oxur, or should we explore alternatives?

5. **Async support:** evcxr's async support required significant Tokio runtime management. Does Oxur need async in REPL?

---

## Conclusion

The research overwhelmingly validates evcxr's core architecture as the right approach:

> **"The subprocess model with stdin/stdout, cargo-based compilation, and type-erased variable storage has proven to be the correct architectural choice and has required no fundamental changes over 6 years."**

Oxur should:
1. **Follow this proven architecture**
2. **Skip the known mistakes** (compiler hack, delayed caching)
3. **Start simple** and add complexity only when profiling demands it

The most important revision to Oxur's plan: **Subprocess execution is mandatory, not optional.** The `Executor` trait abstraction is still useful for testing, but v1.0 must ship with subprocess execution to support interactive interruption.

---

**Document Status:** Ready for review and integration into Oxur architecture v1.1
