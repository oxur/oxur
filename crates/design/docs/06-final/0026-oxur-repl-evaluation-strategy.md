---
number: 26
title: "Oxur REPL Evaluation Strategy"
author: "resisting it"
created: 2026-01-02
updated: 2026-01-03
state: Final
supersedes: null
superseded-by: null
version: 1.0
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

## The Interpretation vs. Compilation Tradeoff

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

## Proposed Evaluation Strategy

### Two-Tier Execution Model

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
│  Compile: Everything else               │
│  Examples: Variables, functions, IO     │
│  Time: 50-200ms first, ~0ms cached      │
│  Code: Reuse compilation pipeline       │
│  Risk: None - Rust safety guaranteed    │
└─────────────────────────────────────────┘
```

**Note:** The previous REPL protocol design mentioned "Tier 3: JIT Compilation" as distinct from Tier 2. This evaluation strategy simplifies to just two tiers - there's no meaningful distinction between "cached compilation" and "JIT compilation" in our architecture.

### Tier 1: Calculator Mode

**Only interpret literal arithmetic:**

```rust
pub enum InterpretableForm {
    Literal(Literal),
    BinaryOp {
        op: BinOp,
        left: Box<InterpretableForm>,
        right: Box<InterpretableForm>,
    },
}

impl Calculator {
    pub fn can_interpret(&self, form: &CoreForm) -> bool {
        match form {
            CoreForm::Literal(_) => true,
            CoreForm::BinaryOp { left, right, .. } => {
                self.is_literal(left) && self.is_literal(right)
            }
            _ => false,
        }
    }

    pub fn eval(&self, form: &CoreForm) -> Result<i64> {
        match form {
            CoreForm::Literal(Literal::Integer(n)) => Ok(*n),
            CoreForm::BinaryOp { op, left, right, .. } => {
                let l = self.eval(left)?;
                let r = self.eval(right)?;
                self.apply_op(*op, l, r)
            }
            _ => Err(Error::NotInterpretable),
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

### Tier 2: Cached Compilation

**Compile everything else through the full pipeline:**

```rust
pub struct CachedCompiler {
    cache: HashMap<CodeHash, CompiledCode>,
    temp_dir: TempDir,
}

impl CachedCompiler {
    pub async fn eval(&mut self, form: CoreForm) -> Result<EvalResult> {
        let code_hash = hash(&form);

        // Check cache
        if let Some(compiled) = self.cache.get(&code_hash) {
            return self.execute(compiled);
        }

        // Compile (with progress indicator for slow compiles)
        let compiled = self.compile(form).await?;
        self.cache.insert(code_hash, compiled.clone());

        self.execute(&compiled)
    }

    async fn compile(&self, form: CoreForm) -> Result<CompiledCode> {
        // Full pipeline from compilation chain doc:
        // 1. Lower Core Forms → Rust AST
        // 2. Generate Rust source
        // 3. Compile to dynamic library
        // 4. Load with libloading

        let rust_code = lower_and_generate(&form)?;
        let lib_path = self.compile_dylib(&rust_code).await?;

        Ok(CompiledCode::load(lib_path)?)
    }
}
```

**Benefits:**

- First compile: 50-200ms (acceptable for non-trivial code)
- Cached: ~0ms (instant function call)
- Full Rust safety guarantees
- Identical semantics to compiled code

## Handling the Two REPL Modes

### Lisp Syntax Mode (Default)

For user-facing development:

```rust
pub async fn eval_lisp(&mut self, source: &str) -> Result<Response> {
    // Stage 1: Parse Oxur syntax
    let surface_forms = parse_oxur_syntax(source)?;

    // Stage 2: Expand macros
    let core_forms = expand_macros(surface_forms)?;

    // Try calculator mode (Tier 1)
    if let Ok(result) = self.calculator.eval(&core_forms[0]) {
        return Ok(Response {
            value: Some(json!(result)),
            status: vec![Status::Done],
            ..Default::default()
        });
    }

    // Fall through to compilation (Tier 2)
    self.compile_and_run(core_forms[0]).await
}
```

### S-expression Mode (Debug)

For compiler debugging - **always compile**, never interpret:

```rust
pub async fn eval_sexpr(&mut self, source: &str) -> Result<Response> {
    // Parse Core Forms directly (skip macro expansion)
    let core_forms = parse_core_forms(source)?;

    // ALWAYS compile (no calculator fast path)
    // This mode is for debugging the compiler itself
    self.compile_and_run(core_forms[0]).await
}
```

**Rationale:** S-expression mode is for inspecting what the compiler sees after macro expansion. It's a debugging tool, not a performance-critical feature. Users should rarely use it. Consistency with the compilation pipeline is more important than speed.

## Integration with evcxr

### What We Should Use

#### 1. `evcxr_runtime` (Definitely Use)

For value representation and execution in Tier 2:

```rust
use evcxr_runtime::{Runtime, EvalResult};

impl CachedCompiler {
    async fn execute(&self, compiled: &CompiledCode) -> Result<EvalResult> {
        let runtime = Runtime::new();

        // Execute compiled code with runtime
        let result = unsafe { compiled.call_with_runtime(&runtime) }?;

        Ok(result)
    }
}
```

**Benefits:**

- Battle-tested value representation
- Display/Debug formatting for REPL output
- Error handling for runtime failures
- Output capture mechanisms

#### 2. Compilation Patterns (Study and Adapt)

From `evcxr` and `evcxr_repl`:

- **Output capture:** How to redirect stdout/stderr during evaluation
- **Temporary file management:** Safe handling of generated .rs files
- **Dynamic library loading:** Safety checks when using `libloading`
- **rustc invocation:** Correct flags for dynamic libraries
- **Error parsing:** Translating rustc errors to user-friendly messages

### What We Should Build Ourselves

#### 1. REPL Server (Our Protocol Design is Better)

evcxr's REPL is monolithic and single-threaded. Our protocol design (Doc 0018) is superior:

- ✅ Multi-transport (TCP, Unix sockets, named pipes, in-process)
- ✅ Session isolation with explicit session IDs
- ✅ Dual-mode evaluation (Lisp syntax + s-expr AST)
- ✅ Structured protocol with postcard serialization
- ✅ Streaming responses for long operations

**Decision:** Build our own REPL server using our protocol design.

#### 2. Tiered Execution (Our Innovation)

evcxr compiles everything immediately (~200ms even for `2 + 2`). Our two-tier approach is smarter:

- Tier 1: Calculator mode (<1ms for literal arithmetic)
- Tier 2: Cached compilation (instant after first compile)

**Decision:** Implement our tiered strategy from scratch.

#### 3. Source Map Integration (Our Advantage)

evcxr forwards rustc errors as-is (users see generated Rust code). Our source map architecture (Doc 0013) enables:

```
Error at generated.rs:42
  ← Node 5000 (Rust AST)
  ← Node 200 (Core Form)
  ← Node 100 (Surface Form)
  → test.ox:5:10: undefined variable: foo
```

**Decision:** Use our source map architecture for error translation.

## Required Preparatory Work

Before detailed REPL design, we need hands-on experience. The following work is **required prerequisites**:

### 1. Audit evcxr_repl

**Goal:** Extract useful patterns, assess against Oxur's needs

**Tasks:**

- [ ] Document evcxr's compilation workflow
- [ ] Identify output capture mechanism
- [ ] Extract temporary file handling patterns
- [ ] Analyze dynamic library loading safety checks
- [ ] Document rustc invocation (flags, options)
- [ ] Study error message parsing
- [ ] Rate each pattern: Essential / Useful / Not Needed

**Deliverable:** `evcxr-audit.md` with prioritized patterns

### 2. Audit evcxr_runtime

**Goal:** Identify what we need from the runtime

**Tasks:**

- [ ] Document runtime value representation
- [ ] Identify display/debug formatting utilities
- [ ] Extract error handling patterns
- [ ] Find output capture mechanisms
- [ ] Assess integration points with our compiler
- [ ] Determine minimal API we need

**Deliverable:** `evcxr-runtime-integration.md` with API usage plan

### 3. Define Prototype Oxur Lisp Syntax

**Goal:** Create test cases for end-to-end compilation

**Tasks:**

- [ ] Define syntax for 2-3 simple forms (e.g., arithmetic, let bindings)
- [ ] Write example `.ox` files using these forms
- [ ] Document expected Rust output for each example
- [ ] Create test suite with expected results

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

### 4. Build Prototype Compiler

**Goal:** End-to-end compilation of prototype syntax

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

## Why This Preparatory Work Matters

### Avoid Premature Design Decisions

Without hands-on experience:

- We might design APIs that don't fit actual usage
- We might miss important integration points
- We might overcommit to patterns that don't work

### Validate Assumptions

The prototype will reveal:

- Whether our two-tier strategy actually works
- How well evcxr_runtime integrates with our compiler
- What compilation performance really looks like
- What error messages users actually see

### Build Intuition

Working code provides:

- Concrete examples for documentation
- Test cases for the real implementation
- Muscle memory for Oxur idioms
- Confidence in design decisions

## Implementation Phases (Tentative)

These phases are **subject to revision** after preparatory work:

### Phase 0: Preparatory Work (Weeks 1-3)

- Audit evcxr projects
- Define prototype syntax
- Build prototype compiler
- **Validate:** Can compile 3 test cases end-to-end

### Phase 1: Tier 2 Only (Weeks 4-5)

- Implement compilation-only REPL
- Integrate evcxr_runtime
- Get output capture working
- **Validate:** Can evaluate all test cases via REPL

### Phase 2: Add Calculator (Week 6)

- Implement Tier 1 (calculator mode)
- Add fast path for literal arithmetic
- **Validate:** Calculator <1ms, compilation <200ms

### Phase 3: Protocol Server (Weeks 7-8)

- Implement REPL protocol (Doc 0018)
- Add session management
- Support TCP transport
- **Validate:** Remote clients can connect and evaluate

### Phase 4: Polish (Week 9)

- Error translation with source maps
- Progress indicators for slow compiles
- Comprehensive testing
- Documentation

## Success Criteria

The REPL evaluation strategy is successful when:

1. ✅ Calculator mode handles literal arithmetic in <1ms
2. ✅ Compilation produces identical results to compiled code
3. ✅ Cached compilation is instant (<10ms)
4. ✅ Error messages trace back to original Oxur source
5. ✅ Users never encounter "works in REPL but fails when compiled"
6. ✅ Calculator code is <200 lines
7. ✅ No divergent execution paths to maintain

## Open Questions

These will be answered during preparatory work:

1. **Output Capture:** What's the best mechanism for capturing stdout/stderr?
2. **Cache Invalidation:** When should we evict compiled code from cache?
3. **Progress Indicators:** At what threshold should we show "Compiling..."?
4. **Error Translation:** How accurate can we make rustc → Oxur error mapping?
5. **Memory Management:** How do we safely unload old dynamic libraries?

## References

- [Design Doc 0013: Compilation Chain Architecture](./0013-oxur-compilation-chain-architecture.md)
- [Design Doc 0018: REPL Protocol Design](./0018-oxur-remote-repl-protocol-design.md)
- [evcxr Project](https://github.com/evcxr/evcxr)
- [evcxr_runtime](https://docs.rs/evcxr_runtime/)

## Conclusion

Oxur's REPL will succeed by **embracing Rust's strengths** rather than reimplementing them:

- **Minimal interpretation** keeps code simple and safe
- **Compilation through Rust** ensures consistency and correctness
- **Caching** makes compilation feel instant for repeated use
- **evcxr_runtime** provides battle-tested evaluation infrastructure

The slippery slope of building a full interpreter is real and dangerous. By resisting it, we get a simpler, safer, more maintainable REPL that leverages Rust's guarantees instead of bypassing them.

**Next Step:** Begin preparatory work (evcxr audits, prototype syntax, prototype compiler).

---

**"Interpret the trivial. Compile the real. Trust Rust."**

---

*End of Document*
