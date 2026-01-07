---
number: 42
title: "Towards a Rust VM for Oxur: A Pragmatic Approach"
author: "Duncan McGreggor"
component: All
tags: [change-me]
created: 2026-01-06
updated: 2026-01-06
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# Towards a Rust VM for Oxur: A Pragmatic Approach

## Document Purpose

This brainstorm document explores what a "virtual machine" for Rust-based languages might look like, specifically in the context of Oxur. It synthesizes discussions about:

- Why Rust lacks a VM and what that means for REPL development
- How the current Oxur REPL architecture (ODD-0038) approximates VM-like capabilities
- What a true Rust VM might involve and whether it's desirable
- A pragmatic staged approach from our current architecture toward more VM-like capabilities
- Comparisons with Erlang/BEAM and what we can learn from that ecosystem

**Context:** This document originated from a conversation about how the Oxur REPL design decisions (temp directories, subprocess execution, file-based state) are essentially simulating capabilities that a VM would provide natively—a "poverty-stricken VM" built from filesystem primitives and OS processes.

**Catalyst:** Robert Virding (co-creator of Erlang and creator of LFE - Lisp Flavored Erlang) joined the Oxur Discord, prompting reflection on how our architecture compares to the BEAM VM that powers Erlang and LFE.

---

## Table of Contents

1. [The Core Insight: Why We Need VM-like Capabilities](#1-the-core-insight-why-we-need-vm-like-capabilities)
2. [What the BEAM Gives You For Free](#2-what-the-beam-gives-you-for-free)
3. [Our Current "Filesystem VM" (ODD-0038)](#3-our-current-filesystem-vm-odd-0038)
4. [What Would a True Rust VM Look Like?](#4-what-would-a-true-rust-vm-look-like)
5. [LLVM: VM or Compiler?](#5-llvm-vm-or-compiler)
6. [A Pragmatic Staged Approach](#6-a-pragmatic-staged-approach)
7. [The Long-Running Process Alternative](#7-the-long-running-process-alternative)
8. [Comparisons and Trade-offs](#8-comparisons-and-trade-offs)
9. [Recommendations](#9-recommendations)
10. [Open Questions](#10-open-questions)

---

## 1. The Core Insight: Why We Need VM-like Capabilities

### 1.1 The Problem Statement

Rust compiles to native code and has no runtime that can:

- Load new code into a running process dynamically (at least, not *safely*)
- Maintain state across compilation units
- Interrupt running code (threads cannot be killed!)
- Provide hot code reloading
- Offer process-level isolation within a single OS process

For a REPL, we need ALL of these capabilities. Every time a user types an expression:

1. New code must be compiled
2. That code must have access to previously defined variables
3. The user must be able to Ctrl-C out of infinite loops
4. Crashes shouldn't lose the entire session
5. State should persist across evaluations

### 1.2 The Erlang Contrast

In Erlang/LFE, the BEAM VM provides all of this natively:

```erlang
% In the Erlang shell, this "just works":
1> X = 42.
42
2> Y = X + 1.  % X is available from previous eval
43
3> spawn(fun() -> loop_forever() end).  % Won't block the shell
<0.84.0>
4> % Ctrl-C menu available, processes isolated, hot reload possible
```

The BEAM was *designed* for interactive, fault-tolerant, hot-reloadable development. Rust was designed for systems programming with compile-time safety guarantees.

### 1.3 The Oxur Challenge

We're building a Lisp on Rust. Lisps have a long tradition of interactive development—the REPL is not an afterthought but a core part of the development experience. We need to bridge the gap between:

- **Rust's strengths:** Memory safety, zero-cost abstractions, native performance
- **Lisp expectations:** Interactive development, live coding, exploratory programming

---

## 2. What the BEAM Gives You For Free

Before designing our approach, it's worth understanding what the BEAM VM provides that we must simulate:

### 2.1 Feature Comparison

| BEAM Capability | Description | Oxur Must Provide |
|-----------------|-------------|-------------------|
| **Process isolation** | Lightweight processes with separate heaps | Subprocess execution |
| **Hot code loading** | Replace modules in running system | Dynamic library loading |
| **Preemptive scheduling** | Reduction counting, fair scheduling | Ctrl-C via process kill |
| **Message passing** | Async communication between processes | stdin/stdout protocol |
| **ETS** | In-memory concurrent storage | VariableStore |
| **Code server** | Manages loaded modules | ArtifactCache |
| **Supervision** | Restart failed processes automatically | Subprocess restart logic |
| **Distribution** | Transparent multi-node | Not in scope (v1.0) |

### 2.2 The BEAM's Key Insight

The BEAM's fundamental insight is that **the unit of failure should be small and recoverable**. A crash in one Erlang process doesn't bring down the system—the supervisor restarts it.

For Oxur, we've adopted this at the OS process level: the subprocess can crash, and the server restarts it. But this is coarser-grained than BEAM processes (microseconds to spawn vs. milliseconds).

---

## 3. Our Current "Filesystem VM" (ODD-0038)

### 3.1 Architecture Mapping

The ODD-0038 architecture essentially simulates VM capabilities through external mechanisms:

```
┌───────────────────────────────────────────────────────────────┐
│                    "FILESYSTEM VM" COMPONENTS                 │
├───────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────────────┐      ┌─────────────────────┐         │
│  │   Session Directory │      │   ArtifactCache     │         │
│  │   /dev/shm/oxur/    │      │   ~/.cache/oxur/    │         │
│  │                     │      │                     │         │
│  │  - Cargo.toml       │      │  - Compiled .so     │         │
│  │  - src/lib.rs       │      │  - Content-addressed│         │
│  │  - target/          │      │  - Cross-session    │         │
│  └─────────────────────┘      └─────────────────────┘         │
│            ↓                             ↓                    │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                    Cargo (rustc + LLVM)                 │  │
│  │                    "JIT Compiler"                       │  │
│  └─────────────────────────────────────────────────────────┘  │
│            ↓                                                  │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                  SubprocessExecutor                     │  │
│  │                  "Process Isolation"                    │  │
│  │                                                         │  │
│  │  ┌────────────────────────────────────────────────────┐ │  │
│  │  │  VariableStore (HashMap<String, Box<dyn Any>>)     │ │  │
│  │  │  "In-Memory State"                                 │ │  │
│  │  └────────────────────────────────────────────────────┘ │  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

### 3.2 Erlang Equivalence Table

| Erlang/BEAM | Oxur Equivalent | Implementation |
|-------------|-----------------|----------------|
| BEAM process | OS subprocess | `SubprocessExecutor` |
| Process heap | VariableStore | `HashMap<String, Box<dyn Any>>` |
| Code server | ArtifactCache | `~/.cache/oxur/artifacts/` |
| Module loading | dlopen/libloading | `LOAD_AND_RUN` protocol |
| Message passing | stdin/stdout | Text protocol |
| Supervisor | Restart logic | `SubprocessExecutor::restart()` |
| ETS table | (not yet) | See ODD-0043 |
| Process dictionary | Session state | `SessionState` struct |
| Hot code upgrade | Recompile + reload | Full pipeline |

### 3.3 What We Gain

This "filesystem VM" approach provides:

- ✅ **Reliability:** Proven patterns (evcxr, 6+ years)
- ✅ **Debuggability:** Standard tools work (gdb, strace, file inspection)
- ✅ **Simplicity:** No custom runtime to maintain
- ✅ **Portability:** Works on any platform with Rust toolchain
- ✅ **Crash isolation:** Subprocess crashes don't kill server

### 3.4 What We Lose

Compared to a true VM:

- ❌ **Latency:** 50-300ms cold compile vs. microseconds for bytecode
- ❌ **Granularity:** OS process vs. lightweight process
- ❌ **Introspection:** Can't easily inspect running state
- ❌ **Hot reload:** Must recompile, can't patch in place
- ❌ **Memory overhead:** Separate address spaces

---

## 4. What Would a True Rust VM Look Like?

### 4.1 Option A: Bytecode VM (JVM/BEAM Style)

```
Rust Source → MIR → Custom Bytecode → VM Interpreter
                                         ↓
                              ┌──────────────────────┐
                              │  Rust VM Runtime     │
                              │  - GC or RC          │
                              │  - Hot code reload   │
                              │  - Debugger hooks    │
                              │  - Signal handling   │
                              │  - State management  │
                              └──────────────────────┘
```

**The fundamental challenge:** Rust's semantics (ownership, borrowing, lifetimes) are *designed* to be resolved at compile time. A bytecode VM would have to either:

1. **Lose Rust's guarantees** - Like JVM doesn't enforce Rust-style borrowing
2. **Carry ownership metadata at runtime** - Massive performance overhead
3. **Only accept "safe" subsets** - Limited usefulness

This is why **Miri** (Rust's interpreter) exists but is *slow*—it tracks all ownership/borrowing semantics at runtime.

**Verdict:** Possible but would sacrifice much of what makes Rust valuable.

### 4.2 Option B: Native Code VM with Hot Reload

```
┌───────────────────────────────────────────────────────────────┐
│                      Rust Native VM                           │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                 Address Space Manager                   │  │
│  │  - Allocate code regions                                │  │
│  │  - Track which regions are "live"                       │  │
│  │  - Enable/disable write protection                      │  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                 Dynamic Linker                          │  │
│  │  - Resolve symbols across code versions                 │  │
│  │  - Handle ABI compatibility                             │  │
│  │  - Manage vtables for trait objects                     │  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                 State Manager                           │  │
│  │  - Type-erased value storage                            │  │
│  │  - Cross-version value migration                        │  │
│  │  - Serialization/deserialization                        │  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                 Execution Control                       │  │
│  │  - Cooperative yield points                             │  │
│  │  - Signal-based interruption                            │  │
│  │  - Stack unwinding for interrupts                       │  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

**This is essentially what we're building**, but distributed across:

- Cargo (compiler)
- File system (code storage)
- Subprocess (execution isolation)
- VariableStore (state management)
- libloading (dynamic linking)

**Verdict:** We're already doing this, just with external tools rather than a unified runtime.

### 4.3 Option C: Hybrid Bytecode + JIT

```
Oxur Source
     ↓
┌─────────────────────────────────────────────────────────────────┐
│                    Oxur Bytecode VM                             │
│                                                                 │
│  ┌───────────────────┐    ┌───────────────────┐                 │
│  │  Interpreter      │    │  JIT (Cranelift)  │                 │
│  │  - Fast startup   │    │  - Hot functions  │                 │
│  │  - Yield points   │    │  - Native speed   │                 │
│  │  - Debuggable     │    │  - Optimized      │                 │
│  └───────────────────┘    └───────────────────┘                 │
│            ↓                        ↓                           │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │              Unified Runtime                              │  │
│  │  - Oxur-specific types (lists, maps, atoms)               │  │
│  │  - Cooperative scheduling                                 │  │
│  │  - Hot code swap                                          │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

This is more like LuaJIT's approach: fast interpreter for most code, JIT for hot paths.

**Pros:**

- Fast startup (no compilation wait)
- Interruptible (yield points in interpreter)
- Can still get native performance for hot code

**Cons:**

- Significant implementation effort
- Two execution paths to maintain
- Debugging complexity

**Verdict:** Interesting for v2.0+, but too ambitious for initial release.

---

## 5. LLVM: VM or Compiler?

### 5.1 The Naming Confusion

**LLVM is NOT a VM in the Erlang/JVM sense.** The "VM" in LLVM stands for "Virtual Machine," but it refers to an **abstract machine model** (the LLVM IR), not a runtime execution environment.

```
LLVM Architecture:

  Source (C, Rust, etc.)
         ↓
  ┌─────────────────┐
  │   Frontend      │  (clang, rustc frontend)
  │   (Parsing)     │
  └────────┬────────┘
           ↓
  ┌─────────────────┐
  │   LLVM IR       │  ← "Virtual Machine" instruction set
  │   (Abstract)    │     NOT executed directly!
  └────────┬────────┘
           ↓
  ┌─────────────────┐
  │   Optimizer     │  (optimization passes)
  └────────┬────────┘
           ↓
  ┌─────────────────┐
  │   Backend       │  (code generation)
  │   (x86, ARM...) │
  └────────┬────────┘
           ↓
  Native Machine Code
```

The "virtual machine" is the **target of compilation**, not execution. It's like Java bytecode, except LLVM IR is *never meant to be interpreted at scale* (though `lli` can interpret it for testing).

### 5.2 Could LLVM Power an Erlang-like VM?

**Sort of, but it would be unusual:**

**Approach 1: LLVM IR Interpreter** (exists as `lli`)

```
Oxur → Core Forms → LLVM IR → lli (LLVM interpreter)
                                  ↓
                            Very slow execution
                            But: hot reload possible!
```

**Approach 2: LLVM JIT** (fast, but no yield points)

```
Oxur → Core Forms → LLVM IR → MCJIT/ORC → Native in memory
                                            ↓
                                      Fast execution
                                      But: can't interrupt!
```

**Approach 3: Bytecode VM with LLVM-generated interpreter**

```
Oxur → Core Forms → Custom Bytecode
                           ↓
           ┌───────────────────────────────┐
           │  Generated Bytecode VM        │
           │  (interpreter loop in Rust,   │
           │   compiled with LLVM)         │
           │                               │
           │  while running:               │
           │    check_interrupt()          │
           │    dispatch(next_opcode)      │
           └───────────────────────────────┘
```

**Approach 3** is closest to what Erlang does. The BEAM is written in C, compiled with a C compiler, and interprets BEAM bytecode with reduction counting for preemption.

### 5.3 Cranelift as Alternative

Cranelift is a code generator designed for JIT compilation:

- Faster compilation than LLVM (designed for JIT latency)
- Simpler API
- Used by Wasmtime, rustc (experimental)

For Oxur, Cranelift could replace cargo for faster iteration:

```
Current:  Oxur → Rust source → cargo → rustc → LLVM → .so → dlopen
                                       ~100-300ms

Cranelift: Oxur → Cranelift IR → Cranelift → memory → call
                                       ~10-50ms
```

**Trade-off:** Cranelift produces less optimized code than LLVM, but compilation is much faster.

---

## 6. A Pragmatic Staged Approach

Rather than building a full VM, we can incrementally add VM-like capabilities:

### Phase 1: Current Architecture (ODD-0038) ✅

**Status:** Designed, implementation starting

```
┌─────────────────────────────────────────────────────────────────┐
│  Phase 1: Filesystem VM                                         │
│                                                                 │
│  - Subprocess execution (crash isolation, Ctrl-C)              │
│  - File-based compilation (cargo)                              │
│  - VariableStore (Box<dyn Any>)                                │
│  - ArtifactCache (content-addressed .so files)                 │
│  - stdin/stdout protocol                                        │
│                                                                 │
│  Latency: 50-300ms cold, 1-5ms cached                          │
│  Isolation: OS process level                                    │
└─────────────────────────────────────────────────────────────────┘
```

### Phase 2: Enhanced State Management

**Goal:** Better introspection, persistence, queryability

```
┌────────────────────────────────────────────────────────────────┐
│  Phase 2: Rich State Layer                                     │
│                                                                │
│  NEW: Tiered storage (see ODD-0043)                            │
│    - Tier 1: Lock-free in-memory (DashMap, Trie)               │
│    - Tier 2: Persistent KV (redb)                              │
│    - Tier 3: Queryable (SQLite)                                │
│                                                                │
│  NEW: Symbol table with type information                       │
│  NEW: Rich history with metadata                               │
│  NEW: DisplayValue for structured output                       │
│                                                                │
│  Latency: Same as Phase 1                                      │
│  Capability: +Persistence, +Introspection, +Query              │
└────────────────────────────────────────────────────────────────┘
```

### Phase 3: Cranelift JIT

**Goal:** Faster iteration, reduced compilation latency

```
┌────────────────────────────────────────────────────────────────┐
│  Phase 3: Direct JIT                                           │
│                                                                │
│  REPLACE: cargo compilation                                    │
│  WITH: Cranelift direct codegen                                │
│                                                                │
│  Oxur → Core Forms → Rust AST → Cranelift IR → Native          │
│                                                                │
│  Still need subprocess for:                                    │
│    - Ctrl-C support (native code can't yield)                  │
│    - Crash isolation                                           │
│                                                                │
│  Latency: 10-50ms cold, 1-5ms cached                           │
│  Capability: +Speed                                            │
└────────────────────────────────────────────────────────────────┘
```

### Phase 4: Cooperative Bytecode VM

**Goal:** True interruptibility, even faster startup

```
┌────────────────────────────────────────────────────────────────┐
│  Phase 4: Hybrid Execution                                     │
│                                                                │
│  NEW: Oxur bytecode format                                     │
│  NEW: Bytecode interpreter with yield points                   │
│                                                                │
│  Execution modes:                                              │
│    - Interpreted: Instant start, interruptible, slower         │
│    - JIT: Compile hot paths to native                          │
│    - AOT: Ahead-of-time for production                         │
│                                                                │
│  Latency: <1ms interpreted, 10-50ms JIT                        │
│  Capability: +Interruptibility, +Hot reload                    │
└────────────────────────────────────────────────────────────────┘
```

### Phase 5: Full Runtime (The Dream)

**Goal:** BEAM-like capabilities

```
┌────────────────────────────────────────────────────────────────┐
│  Phase 5: Oxur Runtime                                         │
│                                                                │
│  - Lightweight processes (green threads)                       │
│  - Preemptive scheduling (reduction counting)                  │
│  - Hot code upgrade                                            │
│  - Built-in supervision                                        │
│  - Optional distribution                                       │
│                                                                │
│  This is essentially writing a new VM from scratch.            │
│  Only pursue if Oxur gains significant traction.               │
└────────────────────────────────────────────────────────────────┘
```

---

## 7. The Long-Running Process Alternative

### 7.1 Concept

Instead of subprocess + files, what if we had a single long-running process managing everything?

```rust
struct OxurReplEngine {
    // JIT compilation via cranelift
    jit: JitCompiler,

    // Memory arena for compiled code
    code_arena: ExecutableMemory,

    // State management
    variables: TypedVariableStore,

    // Currently executing code (for interruption)
    execution_handle: Option<ExecutionHandle>,
}

impl OxurReplEngine {
    fn eval(&mut self, code: &str) -> Result<Value> {
        // 1. Parse Oxur → Core Forms
        let core = parse_and_expand(code)?;

        // 2. Lower to Cranelift IR
        let ir = lower_to_cranelift(&core, &self.variables)?;

        // 3. JIT compile to native code
        let func_ptr = self.jit.compile(ir)?;

        // 4. Allocate in executable memory
        let code_slot = self.code_arena.allocate(func_ptr)?;

        // 5. Execute with panic catching
        let result = std::panic::catch_unwind(|| {
            unsafe { code_slot.call(&mut self.variables) }
        });

        result.map_err(|e| Error::Panic(e))
    }

    fn interrupt(&mut self) {
        // Problem: Can't interrupt native code mid-execution!
        // This is WHY we use subprocess in ODD-0038
    }
}
```

### 7.2 Why We Don't Do This (Yet)

1. **Interruption is fundamentally hard:** Native code can't be safely interrupted mid-execution. The BEAM solves this with reduction counting (cooperative scheduling). We'd need to inject yield points into generated code.

2. **JIT complexity:** Cranelift exists and is great, but managing executable memory, relocation, and symbol resolution is non-trivial. Cargo handles all of this.

3. **Stability:** We'd be responsible for all crash recovery. Subprocess isolation gives us free restart capability.

4. **Debugging:** Generated dylibs can be inspected with standard tools (nm, objdump, gdb). JIT'd memory blobs are harder to debug.

5. **Incremental compilation:** Cargo's incremental compilation is very sophisticated. We'd lose this.

### 7.3 When It Makes Sense

A long-running process approach becomes attractive when:

- Compilation latency is the primary bottleneck
- We have a stable bytecode format for interpretation
- We've implemented cooperative yield points
- The user base is large enough to justify the engineering investment

---

## 8. Comparisons and Trade-offs

### 8.1 Architecture Comparison Matrix

| Aspect | BEAM | ODD-0038 | Cranelift JIT | Custom Bytecode VM |
|--------|------|----------|---------------|-------------------|
| **Cold start latency** | ~1ms | 50-300ms | 10-50ms | <1ms |
| **Warm latency** | ~1ms | 1-5ms | 1-5ms | <1ms |
| **Interruptibility** | ✅ Native | ✅ Process kill | ❌ Must finish | ✅ Yield points |
| **Crash isolation** | ✅ Process | ✅ Subprocess | ❌ Same process | ⚠️ Needs work |
| **Hot reload** | ✅ Native | ⚠️ Recompile | ⚠️ Recompile | ✅ Possible |
| **Debugging** | ✅ Excellent | ✅ Standard tools | ⚠️ Harder | ⚠️ Custom tools |
| **Implementation effort** | N/A (exists) | Low | Medium | High |
| **Optimization quality** | Good | Excellent (LLVM) | Good | Variable |

### 8.2 When to Use What

**Use ODD-0038 (Filesystem VM) when:**

- Starting a new Lisp-on-Rust project
- Reliability is more important than latency
- Standard debugging tools are important
- Team is small

**Use Cranelift JIT when:**

- Compilation latency is the bottleneck
- You have resources to maintain JIT infrastructure
- Sub-50ms response is critical

**Use Custom Bytecode VM when:**

- Interruptibility is critical (long-running computations)
- You need hot code reload
- You're willing to invest in custom tooling
- Project has significant traction

### 8.3 Robert Virding's Perspective

From the viewpoint of an Erlang/LFE creator, our approach might seem like:

> "Ah yes, you're reinventing all the things the BEAM gives you for free."

And that's true! But it's also:

> "You're getting Rust's memory safety and native performance, which the BEAM doesn't provide."

The BEAM was designed for telecoms (fault tolerance, hot reload, distribution). Rust was designed for systems programming (memory safety, zero-cost abstractions). We're grafting interactive development onto a language that wasn't designed for it—which is exactly what makes it interesting.

---

## 9. Recommendations

### 9.1 Short-term (v1.0)

**Stick with ODD-0038.** The filesystem VM approach is:

- Proven (evcxr, 6+ years)
- Debuggable
- Sufficient for initial release
- Low risk

Focus engineering effort on:

- Getting the compilation pipeline right
- Excellent error messages (source maps)
- Good caching (ArtifactCache)

### 9.2 Medium-term (v1.x)

**Add rich state management (ODD-0043):**

- Tiered storage for performance
- Queryable history
- Better introspection

**Consider Cranelift** if users complain about latency:

- Evaluate compilation time vs. code quality trade-off
- Could be optional backend

### 9.3 Long-term (v2.0+)

**Evaluate bytecode VM** based on:

- User demand for interruptibility
- Hot reload requirements
- Project resources

**Do NOT pursue full runtime** unless:

- Oxur has significant adoption
- There's clear demand for BEAM-like features
- Resources exist for multi-year investment

---

## 10. Open Questions

### 10.1 Technical

1. **Cranelift integration:** How hard is it to go from Rust AST to Cranelift IR? Could we skip the Rust source generation entirely?

2. **Bytecode design:** If we designed an Oxur bytecode, what would it look like? Stack-based like JVM? Register-based like Lua?

3. **Yield point injection:** Could we automatically inject yield points during compilation without user annotation?

4. **Memory management:** For a bytecode VM, what memory management strategy? Tracing GC? Reference counting? Arena allocation?

### 10.2 Strategic

1. **Target audience:** Are Oxur users willing to accept REPL latency for Rust's benefits? Or do they expect Clojure/Racket responsiveness?

2. **Differentiation:** Should Oxur compete on "Lisp with Rust performance" or "Rust with Lisp ergonomics"? This affects VM investment.

3. **Ecosystem:** Should we prioritize Rust crate interop (favors current approach) or Lisp library ecosystem (favors custom VM)?

### 10.3 For Discussion

1. What's the acceptable cold-start latency for the REPL? 300ms? 100ms? 50ms?

2. How important is interruptibility? Is Ctrl-C via process kill sufficient?

3. Would users value hot code reload? Or is restart acceptable?

4. Should we expose bytecode compilation as a user-facing feature (like Clojure's AOT)?

---

## Version History

### Version 1.0 (2026-01-06)

Initial brainstorm document synthesizing discussion about Rust VM concepts, BEAM comparisons, and staged approach to VM-like capabilities for Oxur.

---

**Document Status:** Brainstorm - Open for discussion and refinement
