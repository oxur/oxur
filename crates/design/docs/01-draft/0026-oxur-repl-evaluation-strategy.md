---
number: 26
title: "Oxur REPL Evaluation Strategy"
author: "Duncan McGreggor and Claude"
created: 2026-01-02
updated: 2026-01-04
state: Draft
supersedes: null
superseded-by: null
version: 1.1
---
# Oxur REPL Evaluation Strategy

## Overview

This document establishes the evaluation strategy for Oxur's REPL, addressing the fundamental question: **when should we interpret vs. compile Oxur code?**

The answer: **Interpret almost nothing. Compile almost everything.**

## Executive Summary

### Core Decision

Oxur REPL will use a **minimal interpretation** strategy:

- ✅ **Interpret:** Literal arithmetic expressions only (calculator mode)
- ✅ **Compile:** Everything else through Rust pipeline
- ❌ **Never:** Build a full interpreter that duplicates Rust semantics

### Rationale

1. **Safety:** Rust's type system, borrow checker, and compiler catch errors
2. **Consistency:** REPL behavior matches compiled code exactly
3. **Simplicity:** ~100 lines of calculator code vs. ~5000+ for full interpreter
4. **Performance:** Caching makes compilation instant after first use
5. **Maintenance:** No divergent execution paths to maintain

### Integration with Existing Designs

This strategy builds upon:

- [Design Doc 0013: Compilation Chain Architecture](./0013-oxur-compilation-chain-architecture.md) - Full compilation pipeline
- [Design Doc 0018: REPL Protocol Design](./0018-oxur-remote-repl-protocol-design.md) - Network protocol and session management
- **REPL Architecture Overview** - Complete system architecture with component locations and data flow

---

## 1. The Interpretation vs. Compilation Tradeoff

### The Slippery Slope

Building an interpreter for Lisp-family languages is tempting but dangerous:

```lisp
;; It starts innocently:
oxur> (+ 1 2)
;; "We could interpret this! It's just addition!"

;; Then you need variables:
oxur> (+ x 2)
;; "Okay, we need an environment for variable lookup..."

;; Then control flow:
oxur> (if (> x 10) (println x) (println "small"))
;; "We need if-expr evaluation, side effects, IO..."

;; Then functions:
oxur> (defn factorial [n] (if (<= n 1) 1 (* n (factorial (- n 1)))))
;; "Now we're building a complete evaluation engine with recursion..."
```

**Every interpreted feature becomes:**

- Code that doesn't benefit from rustc's type checking
- Code that doesn't benefit from rustc's borrow checker
- Code that doesn't benefit from rustc's optimization
- Code you must test and maintain separately
- Code that might behave subtly differently than compiled code

### Lessons from evcxr

The evcxr project initially tried more interpretation. They backed off because:

1. **Semantic divergence:** "Works in REPL but fails when compiled"
2. **Maintenance burden:** Two execution paths to maintain and test
3. **Marginal gains:** Performance benefit was small for real work
4. **Safety loss:** Lost Rust's safety guarantees

They settled on: compile everything, optimize the compilation pipeline.

---

## 2. Proposed Evaluation Strategy

### Three-Tier Execution Model

```
┌─────────────────────────────────────────┐
│      Tier 1: Calculator Mode            │
│                                         │
│  Interpret: Literal arithmetic only     │
│  Examples: (+ 1 2), (* 3 4)             │
│  Time: <1ms                             │
│  Code: ~100 lines                       │
│  Risk: Minimal                          │
└─────────────────────────────────────────┘
                 ↓ (if not literal arithmetic)
┌─────────────────────────────────────────┐
│      Tier 2: Cached Compilation         │
│                                         │
│  Execute: Previously compiled code      │
│  Examples: Repeat evaluations           │
│  Time: 1-5ms (just function call)       │
│  Code: Reuse loaded library             │
│  Risk: None - Rust safety guaranteed    │
└─────────────────────────────────────────┘
                 ↓ (if not cached)
┌─────────────────────────────────────────┐
│      Tier 3: JIT Compilation            │
│                                         │
│  Compile: First-time code               │
│  Examples: New functions, complex forms │
│  Time: 50-300ms (compile + load)        │
│  Code: Full compilation pipeline        │
│  Risk: None - Rust safety guaranteed    │
└─────────────────────────────────────────┘
```

**Why Three Tiers?**

Tier 2 and Tier 3 have significantly different performance characteristics:

- **Tier 2 (Cached):** Library already loaded in subprocess, just call the function - very fast (~1-5ms)
- **Tier 3 (JIT):** Must invoke cargo, compile Rust code, load dylib into subprocess - much slower (~50-300ms)

This distinction is critical for user experience:

- First evaluation of `(defn square [x] (* x x))` → Tier 3 (200ms)
- Subsequent `(square 5)` calls → Tier 2 (2ms)

The tier decision logic requires cache checking and affects whether we show compilation progress indicators.

### Tier 1: Calculator Mode

**Only interpret literal arithmetic:**

```rust
// Location: oxur-repl/src/eval/context.rs
// Part of: EvalContext

pub enum InterpretableForm {
    Literal(Literal),
    BinaryOp {
        op: BinOp,
        left: Box<InterpretableForm>,
        right: Box<InterpretableForm>,
    },
}

impl EvalContext {
    fn eval_calculator(&self, form: &CoreForm) -> Result<Value> {
        match form {
            CoreForm::Literal(Literal::Integer(n)) => Ok(Value::Int(*n)),
            CoreForm::BinaryOp { op, left, right, .. } => {
                let l = self.eval_calculator(left)?;
                let r = self.eval_calculator(right)?;
                self.apply_op(*op, l, r)
            }
            _ => Err(Error::NotInterpretable),
        }
    }

    fn is_simple_arithmetic(&self, form: &CoreForm) -> bool {
        match form {
            CoreForm::Literal(_) => true,
            CoreForm::BinaryOp { left, right, .. } => {
                self.is_simple_arithmetic(left) && self.is_simple_arithmetic(right)
            }
            _ => false,
        }
    }
}
```

**Interpreted examples:**

```lisp
oxur> 42
42

oxur> (+ 1 2)
3

oxur> (* (+ 2 3) 4)
20

oxur> (/ 100 (- 15 5))
10
```

**Not interpreted (compile instead):**

```lisp
oxur> x
;; Variable reference - needs environment

oxur> (+ x 2)
;; Variable in expression - needs environment

oxur> (println "hello")
;; Side effect - needs Rust runtime

oxur> (if true 1 2)
;; Control flow - compile for consistency

oxur> (defn add [x y] (+ x y))
;; Function definition - always compiled
```

### Tier 2/3: Compilation

**Compile everything else through the full pipeline:**

```rust
// Location: oxur-repl/src/compiler/cached.rs
// Component: CachedCompiler (owned by EvalContext)

pub struct CachedCompiler {
    cache: HashMap<CodeHash, CompiledCode>,
    session_dir: SessionDir,
    state: SessionState,
    subprocess: Option<ChildProcess>,
    code_gen: CodeGenerator,
    source_map: Arc<SourceMap>,
}

impl CachedCompiler {
    pub async fn eval(&mut self, form: CoreForm) -> Result<Response> {
        let code_hash = hash(&form);

        // Check cache (Tier 2 vs Tier 3 decision)
        if let Some(compiled) = self.cache.get(&code_hash) {
            // Tier 2: Library already loaded
            return self.execute(compiled).await;
        }

        // Tier 3: Compile from scratch
        // (with progress indicator for slow compiles >200ms)
        let compiled = self.compile(form).await?;
        self.cache.insert(code_hash, compiled.clone());

        self.execute(&compiled).await
    }

    async fn compile(&self, form: CoreForm) -> Result<CompiledCode> {
        // Full pipeline from compilation chain doc:
        // 1. Lower Core Forms → Rust AST (via oxur-comp)
        // 2. Generate Rust source (via oxur-ast)
        // 3. Compile to dynamic library (cargo)
        // 4. Load with libloading (in subprocess)

        let code = self.code_gen.generate(&form, &self.state)?;
        let lib_path = self.compile_dylib(&code).await?;

        Ok(CompiledCode { lib_path, fn_name: code.fn_name })
    }

    async fn execute(&mut self, compiled: &CompiledCode) -> Result<Response> {
        // Send LOAD command to subprocess
        // Subprocess loads library and executes
        // Returns result via VariableStore
        todo!("See Architecture Overview, Section 4.1.2")
    }
}
```

**Performance:**

- **Tier 2 (Cached):** ~1-5ms (function call in loaded library)
- **Tier 3 (First compile):** 50-300ms (cargo build + load)
- **Tier 3 (Warm compile):** 50-100ms (incremental compilation)

---

## 3. Component Placement in Architecture

The evaluation strategy is implemented across these components:

| Component | Location | Responsibility |
|-----------|----------|----------------|
| **EvalContext** | `oxur-repl/src/eval/context.rs` | Tier decision, Calculator mode, Coordination |
| **CachedCompiler** | `oxur-repl/src/compiler/cached.rs` | Tier 2/3 execution, Compilation |
| **CodeGenerator** | `oxur-repl/src/codegen/generator.rs` | Core Forms → Rust source |
| **Subprocess** | `oxur-subprocess/src/main.rs` | Isolated code execution |
| **VariableStore** | Embedded in generated code | Type-erased variable storage |

**Ownership:**

- CachedCompiler is owned by EvalContext
- One EvalContext per session
- Sessions managed by SessionManager

**For complete architecture, see:** REPL Architecture Overview document.

### Tier Decision Flow

```
User Input
  ↓
EvalContext.eval(code)
  ↓
Parse (via oxur-lang)
  ↓
decide_tier(core_forms)
  ├─→ Tier 1? → eval_calculator() → Result
  ├─→ Tier 2? → compiler.eval() → cached execution → Result
  └─→ Tier 3? → compiler.eval() → compile + execute → Result
```

This tier logic lives entirely in **EvalContext**.

---

## 4. Handling the Two REPL Modes

**Component Location:** This logic lives in **EvalContext** (`oxur-repl/src/eval/context.rs`).

**Called By:** MessageHandler → SessionManager → EvalContext

### Lisp Syntax Mode (Default)

For user-facing development:

```rust
// In EvalContext
pub async fn eval(&mut self, code: &str) -> Result<Value> {
    // Stage 1: Parse based on mode
    let core_forms = match self.mode {
        ReplMode::Lisp => {
            let surface = oxur_lang::parse_lisp(code)?;
            oxur_lang::expand(surface)?
        }
        ReplMode::Sexpr => {
            oxur_lang::parse_core_forms(code)?
        }
    };

    // Stage 2: Decide tier
    let tier = self.decide_tier(&core_forms);

    // Stage 3: Execute
    let result = match tier {
        Tier::Calculator => self.eval_calculator(&core_forms),
        Tier::Cached | Tier::Jit => {
            self.compiler.eval(core_forms).await?
        }
    };

    // Stage 4: Record history
    self.record_history(code.to_string(), result.clone());

    Ok(result)
}

fn decide_tier(&self, core_forms: &CoreForm) -> Tier {
    // Check Tier 1: Calculator
    if self.is_simple_arithmetic(core_forms) {
        return Tier::Calculator;
    }

    // Check Tier 2: Cached
    let hash = hash_core_forms(core_forms);
    if self.compiler.is_cached(hash) {
        return Tier::Cached;
    }

    // Default: Tier 3: JIT
    Tier::Jit
}
```

**Integration:** CachedCompiler is owned by EvalContext. See Architecture Overview for complete data flow.

### S-expression Mode (Debug)

For compiler debugging - **always compile**, never interpret:

```rust
// In EvalContext
pub async fn eval(&mut self, code: &str) -> Result<Value> {
    let core_forms = if self.mode == ReplMode::Sexpr {
        // Parse Core Forms directly (skip macro expansion)
        oxur_lang::parse_core_forms(code)?
    } else {
        // ... (Lisp mode logic above)
    };

    // In Sexpr mode: ALWAYS compile (no calculator fast path)
    // This mode is for debugging the compiler itself
    if self.mode == ReplMode::Sexpr {
        return self.compiler.eval(core_forms).await;
    }

    // ... (normal tier decision)
}
```

**Rationale:** S-expression mode is for inspecting what the compiler sees after macro expansion. It's a debugging tool, not a performance-critical feature. Users should rarely use it. Consistency with the compilation pipeline is more important than speed.

---

## 5. Integration with evcxr Patterns

### Patterns Adopted from evcxr Audits

After comprehensive audits (see ODD-0027, ODD-0028, ODD-0029), we've adopted:

#### 5.1 Type-Erased Variable Storage (From evcxr_repl)

**Pattern:** Use `Box<dyn Any>` for variable persistence across evaluations.

```rust
pub struct VariableStore {
    variables: HashMap<String, Box<dyn Any + 'static>>,
}

impl VariableStore {
    pub fn put_variable<T: 'static>(&mut self, name: &str, value: T) {
        self.variables.insert(name.to_owned(), Box::new(value));
    }

    pub fn check_variable<T: 'static>(&mut self, name: &str) -> bool {
        if let Some(v) = self.variables.get(name) {
            v.downcast_ref::<T>().is_some()
        } else {
            true  // Variable doesn't exist yet, that's ok
        }
    }

    pub fn take_variable<T: 'static>(&mut self, name: &str) -> T {
        *self.variables.remove(name)
            .expect("Variable missing")
            .downcast()
            .expect("Variable type mismatch")
    }
}
```

**Benefits:**

- No serialization overhead
- Supports arbitrary user types (no trait bounds)
- Simple implementation (~50 lines)
- Proven in production (evcxr)

**Location:** Embedded in generated code, also in subprocess binary

**How It Works:**

Generated code integrates with VariableStore:

```rust
#[no_mangle]
pub extern "C" fn run_user_code_5(
    mut store_ptr: *mut VariableStore
) -> *mut VariableStore {
    let store = unsafe { &mut *store_ptr };

    // Load variables
    if !store.check_variable::<i32>("x") { return store_ptr; }
    let mut x = store.take_variable::<i32>("x");

    // User code
    let result = x + 1;

    // Store back
    store.put_variable("x", x);
    store.put_variable("result", result);

    store_ptr
}
```

#### 5.2 Subprocess Isolation (From evcxr_repl)

**Pattern:** Execute user code in separate process via libloading.

**Architecture:**

```
CachedCompiler (server)
  ↓ stdin: "LOAD /path/to/lib.so run_user_code_5"
Subprocess (oxur-subprocess binary)
  ↓ libloading::Library::new()
Load library
  ↓ Call run_user_code_5(variable_store_ptr)
Execute user code
  ↓ Mutate VariableStore
Return
  ↓ stdout: "EVCXR_EXECUTION_COMPLETE"
CachedCompiler receives completion
```

**Benefits:**

- User code crashes don't corrupt REPL state
- Can restart subprocess on panic without losing session
- Clean separation of concerns
- Variable state persists in subprocess's VariableStore

**Implementation:** See Architecture Overview, Section 4.2

#### 5.3 Cargo-Based Compilation (From evcxr)

**Pattern:** Use cargo as build orchestrator, parse JSON output.

```bash
cargo build \
  --target x86_64-unknown-linux-gnu \
  --message-format=json

# Environment:
CARGO_TARGET_DIR=/tmp/oxur-repl/session-abc/target
RUSTFLAGS="-C link-arg=-fuse-ld=mold"  # Fast linker
```

**Benefits:**

- Incremental compilation (3-5x speedup: 200ms → 50ms)
- Dependency management "for free"
- Standard tooling
- Only 10-20ms overhead vs direct rustc

**Cargo.toml per session:**

```toml
[package]
name = "ctx"
version = "1.0.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[profile.dev]
opt-level = 2        # Balance compile vs runtime perf
incremental = true   # Enable incremental compilation
```

**Implementation:** See ODD-0030, Section 5

### Patterns We Will Build Differently

#### 1. REPL Server (Our Protocol is Better)

evcxr has no network protocol. Our design (ODD-0018) provides:

- Multi-transport support (TCP, Unix sockets)
- Session isolation with explicit session IDs
- Dual-mode evaluation (Lisp syntax + s-expr AST)
- Structured protocol with postcard serialization

**Decision:** Use our protocol design, not evcxr's monolithic REPL.

#### 2. Output Formatting (Simplified)

**Discovery from ODD-0029:** evcxr_runtime is NOT a runtime - it's just 75 lines of MIME output formatting.

We build our own simplified display system:

```rust
pub enum DisplayValue {
    Text(String),
    Html(String),
    Image { mime: String, data: Vec<u8> },
    Custom { mime: String, content: Vec<u8> },
}

pub struct Response {
    pub value: Option<DisplayValue>,  // Rich display
    pub out: String,                   // Captured stdout
    pub err: String,                   // Captured stderr
    pub status: Vec<Status>,
}
```

**Decision:** Build our own, don't depend on evcxr_runtime.

#### 3. Source Map Integration (Our Innovation)

evcxr shows generated Rust code in error messages. We translate errors back to original Oxur source positions.

**Our Approach:**

```
rustc error at lib.rs:42
  ↓ Extract /* oxur_node=123 */ comment
SourceMap lookup
  ↓ Node 123 → test.ox:5:15
Display: Error at test.ox:5:15: cannot find value `y`
```

**Decision:** Implement source map translation (see Architecture Overview, Section 11).

---

## 6. Preparatory Work Status

### Completed ✅

#### 1. Audit evcxr Projects

**Status:** COMPLETE

**Deliverables:**

- ODD-0027: evcxr_repl Audit - Session management, subprocess pattern, compilation workflow
- ODD-0028: evcxr (Compiler) Audit - Cargo integration, incremental compilation, rustc invocation
- ODD-0029: evcxr_runtime Audit - Discovery that it's just MIME formatting (75 lines)

**Key Findings:**

- Type-erased storage pattern (adopted)
- Subprocess isolation (adopted)
- Cargo-based compilation (adopted)
- evcxr_runtime is minimal (build our own)

**Results synthesized in:** ODD-0030 (REPL Implementation Specification)

### Remaining 🔧

#### 2. Define Prototype Oxur Lisp Syntax

**Status:** IN PROGRESS

**Tasks:**

- [ ] Define syntax for 2-3 simple forms (arithmetic, let bindings, functions)
- [ ] Write example `.ox` files using these forms
- [ ] Document expected Rust output for each example
- [ ] Create test suite with expected results

**Blocker:** Requires ODD-0021 (Syntax Design) to be finalized

**Example:**

```lisp
;; test-cases/arithmetic.ox
(+ 1 2)

;; test-cases/let-binding.ox
(let [x 10
      y 20]
  (+ x y))

;; test-cases/function.ox
(defn double [n]
  (* n 2))
```

**Deliverable:** `test-cases/` directory with prototype syntax examples

#### 3. Build Prototype Compiler

**Status:** NOT STARTED

**Dependencies:**

- Prototype syntax definition (above)
- Core Forms specification
- Basic parser implementation in oxur-lang
- Basic lowering in oxur-comp

**Tasks:**

- [ ] Implement parser for prototype syntax → Surface Forms
- [ ] Implement expander for prototype forms → Core Forms
- [ ] Implement lowerer Core Forms → Rust AST (for test cases)
- [ ] Generate Rust source from AST
- [ ] Verify generated code compiles with rustc

**Deliverable:** Working prototype that compiles test cases to Rust

**Success Criteria:**

```bash
$ oxur-proto compile test-cases/arithmetic.ox
Generated: output/arithmetic.rs
Compiled: output/arithmetic

$ ./output/arithmetic
3
```

---

## 7. Why This Preparatory Work Matters

### Avoid Premature Design Decisions

Without hands-on experience:

- We might design APIs that don't fit actual usage
- We might miss important integration points
- We might overcommit to patterns that don't work

### Validate Assumptions

The prototype will reveal:

- Whether our three-tier strategy actually works in practice
- How well the VariableStore pattern integrates with our compiler
- What compilation performance really looks like
- What error messages users actually see
- Cache hit rates and effectiveness

### Build Intuition

Working code provides:

- Concrete examples for documentation
- Test cases for the real implementation
- Muscle memory for Oxur idioms
- Confidence in design decisions
- Early detection of architectural issues

---

## 8. Implementation Phases (Tentative)

These phases are **subject to revision** after remaining preparatory work:

### Phase 0: Preparatory Work (Weeks 1-3)

- ✅ Audit evcxr projects (COMPLETE)
- 🔧 Define prototype syntax (IN PROGRESS)
- 🔧 Build prototype compiler (BLOCKED)
- **Validate:** Can compile 3 test cases end-to-end

### Phase 1: Tier 3 Only (Weeks 4-5)

- Implement compilation-only REPL (no cache, no calculator)
- Integrate VariableStore pattern
- Get output capture working
- Get subprocess execution working
- **Validate:** Can evaluate all test cases via compilation

### Phase 2: Add Tier 2 (Week 6)

- Implement cache checking
- Add compiled code cache (hash-based)
- Measure cache hit rates
- **Validate:** Repeat evaluations are instant

### Phase 3: Add Tier 1 (Week 7)

- Implement calculator mode
- Add fast path for literal arithmetic
- Add tier decision logic to EvalContext
- **Validate:** Calculator <1ms, Tier 3 <300ms, Tier 2 <5ms

### Phase 4: Protocol Integration (Week 8)

- Connect EvalContext to MessageHandler
- Test via protocol (TCP clients)
- Multi-session testing
- **Validate:** Remote clients can connect and evaluate

### Phase 5: Polish (Week 9)

- Error translation with source maps
- Progress indicators for slow compiles (>200ms)
- Comprehensive testing
- Documentation
- Performance tuning

---

## 9. Success Criteria

The REPL evaluation strategy is successful when:

1. ✅ Calculator mode handles literal arithmetic in <1ms
2. ✅ Tier 2 (cached) execution is near-instant (<5ms)
3. ✅ Tier 3 (compilation) completes in <300ms (cold), <100ms (warm)
4. ✅ Compilation produces identical results to compiled code
5. ✅ Error messages trace back to original Oxur source
6. ✅ Users never encounter "works in REPL but fails when compiled"
7. ✅ Calculator code is <200 lines
8. ✅ No divergent execution paths to maintain
9. ✅ Cache hit rate >80% for typical REPL usage
10. ✅ Subprocess crashes don't lose session state

---

## 10. Open Questions

These will be answered during remaining preparatory work and implementation:

1. **Cache Size:** How many compiled libraries should we keep in memory?
2. **Cache Eviction:** When should we evict compiled code from cache? (LRU? Size limit?)
3. **Progress Indicators:** At what threshold should we show "Compiling..."? (200ms? 500ms?)
4. **Error Translation:** How accurate can we make rustc → Oxur error mapping?
5. **Memory Management:** How do we safely unload old dynamic libraries? (Or do we need to?)
6. **Subprocess Restart:** When should we restart a crashed subprocess? (Immediate? After N failures?)
7. **Incremental Compilation:** Can we measure the actual speedup in our use case?

---

## 11. References

- [Design Doc 0013: Compilation Chain Architecture](./0013-oxur-compilation-chain-architecture.md)
- [Design Doc 0018: REPL Protocol Design](./0018-oxur-remote-repl-protocol-design.md)
- [Design Doc 0027: evcxr_repl Audit](./0027-evcxr-repl-audit.md)
- [Design Doc 0028: evcxr Compiler Audit](./0028-evcxr-compiler-audit.md)
- [Design Doc 0029: evcxr_runtime Audit](./0029-evcxr-runtime-audit.md)
- [Design Doc 0030: REPL Implementation Specification](./0030-oxur-repl-implementation-specification.md)
- **REPL Architecture Overview** - Complete system architecture
- [evcxr Project](https://github.com/evcxr/evcxr)

---

## Conclusion

Oxur's REPL will succeed by **embracing Rust's strengths** rather than reimplementing them:

- **Minimal interpretation** keeps code simple and safe
- **Three-tier execution** optimizes for common patterns (calculator, cache, compile)
- **Compilation through Rust** ensures consistency and correctness
- **Caching** makes repeated compilation feel instant
- **Subprocess isolation** protects session state from user code crashes
- **VariableStore pattern** enables variable persistence without serialization

The slippery slope of building a full interpreter is real and dangerous. By resisting it, we get a simpler, safer, more maintainable REPL that leverages Rust's guarantees instead of bypassing them.

**Next Step:** Complete prototype syntax definition and build prototype compiler to validate these design decisions.

## Changes from v1.0 to v1.1

**Date:** January 3, 2026

### Critical Corrections

- **Section 3: Three-Tier Execution Model** - Corrected from two-tier to three-tier (Calculator, Cached, JIT), removed incorrect note claiming no distinction
- **Section 8: Integration with evcxr Patterns** - Replaced evcxr_runtime usage with actual patterns: VariableStore + Subprocess isolation

### Major Additions

- **Section 3.1: Component Placement (NEW)** - Shows where evaluation logic lives in architecture (EvalContext, CachedCompiler locations)
- **Architecture integration** - Added context about component ownership and data flow

### Updates

- **Section 4: Handling REPL Modes** - Added component location context, updated code to show tier decision logic
- **Section 6: Preparatory Work Status** - Marked evcxr audits as complete (ODD-0027, 0028, 0029), updated remaining tasks

### Key Improvements

- Alignment with implemented architecture from REPL Architecture Overview
- Corrected understanding of evcxr patterns based on completed audits
- Clear component placement in codebase

---

**"Interpret the trivial. Cache the compiled. Trust Rust."**

---

*End of Document*
