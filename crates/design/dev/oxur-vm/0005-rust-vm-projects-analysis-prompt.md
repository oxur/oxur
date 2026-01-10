# Claude Code Codebase Analysis Prompt: Rust VM/Interpreter Projects for Treebeard

## Context

You are analyzing open source Rust VM and interpreter implementations to inform the design of **Treebeard**, a tree-walking interpreter for Rust's `syn` AST with compilation escape hatches. Treebeard will serve as the execution engine for languages that compile to Rust (like Oxur, a Lisp-to-Rust language).

**Treebeard's key characteristics:**
- Tree-walking interpreter operating on `syn` AST (not bytecode)
- Runtime ownership/borrowing enforcement (simplified model, not full Miri)
- Compilation escape hatch (compile hot paths to native code via rustc)
- REPL-focused with nREPL-compatible protocol
- ~50k lines of Rust budget
- 10-100x native performance is acceptable

**What we're looking for:**
- Environment/binding representation patterns
- Closure capture mechanisms
- How Rust interpreters handle calling native Rust functions
- Value representation (boxing, tagging, etc.)
- Error handling and recovery patterns
- REPL state management
- Any ownership/borrowing tracking at runtime

---

## Codebases to Analyze (Priority Order)

### Tier 1: High Priority (Deeply Analyze)

#### 1. Rhai (`rhaiscript/rhai`)
**Why:** AST-walking interpreter in Rust, excellent Rust interop, mature codebase, similar performance goals.

**Clone:** `git clone https://github.com/rhaiscript/rhai`

**Focus areas:**
- `src/eval/` - The evaluation engine
- `src/types/scope.rs` - How variables and bindings are managed
- `src/types/dynamic.rs` - The `Dynamic` type (boxed values)
- `src/func/` - Function registration and calling
- `src/engine.rs` - The main `Engine` struct
- `src/ast/` - AST representation

**Questions to answer:**
1. How does Rhai represent its environment/scope? Is it a frame chain or flat?
2. How are closures captured? What's stored in a closure value?
3. How does `register_fn` work to expose Rust functions? What's the type conversion machinery?
4. How does Rhai handle function redefinition?
5. What's the `Dynamic` type's memory layout? How expensive is type checking?
6. How does Rhai implement tail call optimization (if at all)?
7. How does the `Scope` interact with evaluation? Is it passed by reference or cloned?

---

#### 2. Miri (`rust-lang/miri`)
**Why:** The reference implementation for runtime Rust semantics, especially ownership/borrowing.

**Clone:** `git clone https://github.com/rust-lang/miri`

**Focus areas:**
- `src/machine.rs` - The `MiriMachine` struct (interpreter state)
- `src/borrow_tracker/` - Stacked Borrows / Tree Borrows implementation
- `src/eval.rs` - Main evaluation entry point
- `src/shims/` - How external functions are handled

**Questions to answer:**
1. How does Miri track ownership state per-allocation?
2. What's the `BorrowTracker` data structure? How expensive is it?
3. How does Miri represent "moved" vs "borrowed" state?
4. What's the minimum data needed per-value for ownership tracking?
5. How does Miri handle scope exit and borrow invalidation?
6. What aspects of Miri's model could be simplified for a REPL use case?

**Note:** Miri interprets MIR, not syn AST, so focus on the ownership tracking, not the evaluation loop.

---

#### 3. Rune (`rune-rs/rune`)
**Why:** Dynamic language for Rust with good type system, pattern matching, modules.

**Clone:** `git clone https://github.com/rune-rs/rune`

**Focus areas:**
- `crates/rune/src/runtime/` - Runtime value representation
- `crates/rune/src/runtime/vm.rs` - Virtual machine
- `crates/rune/src/compile/` - How compilation works
- `crates/rune/src/module.rs` - Module system for Rust interop
- `crates/rune-macros/` - How Rust types are exposed

**Questions to answer:**
1. How does Rune's `Value` type compare to Rhai's `Dynamic`?
2. How does the module system register Rust types and functions?
3. How does Rune handle pattern matching at runtime?
4. What's Rune's approach to type information at runtime?
5. How does Rune handle async/await? (relevant for future Treebeard)

---

### Tier 2: Medium Priority (Survey for Specific Features)

#### 4. Gluon (`gluon-lang/gluon`)
**Why:** Statically typed, functional, has type inference. Interesting for type representation.

**Clone:** `git clone https://github.com/gluon-lang/gluon`

**Focus areas:**
- `vm/src/` - The virtual machine
- `vm/src/value.rs` - Value representation
- `vm/src/gc.rs` - Garbage collection approach
- `base/src/types/` - Type representation

**Questions to answer:**
1. How does Gluon represent types at runtime with static typing?
2. How does the GC interact with the interpreter?
3. How are closures represented with type information?

---

#### 5. Ketos (`murarth/ketos`)  
**Why:** Lisp dialect in Rust with bytecode VM. Good reference for Lisp-specific patterns.

**Clone:** `git clone https://github.com/murarth/ketos`

**Focus areas:**
- `src/interpreter.rs` - Main interpreter
- `src/scope.rs` - Scope/environment
- `src/value.rs` - Value representation
- `src/compile.rs` - Compilation to bytecode
- `src/exec.rs` - Bytecode execution

**Questions to answer:**
1. How does Ketos handle macro expansion?
2. How is the Lisp environment model implemented?
3. How does Ketos bridge Rust types to Lisp values?

---

#### 6. rust-hosted-langs/book interpreter
**Why:** Educational implementation with excellent documentation, covers memory management.

**Clone:** `git clone https://github.com/rust-hosted-langs/book`

**Focus areas:**
- `interpreter/src/` - The interpreter implementation
- Book chapters at `rust-hosted-langs.github.io/book/`

**Questions to answer:**
1. How is the tagged pointer scheme implemented?
2. How does the allocator interface work?
3. What's the bytecode format and dispatch mechanism?

---

### Tier 3: Reference Only (Skim for Specific Patterns)

#### 7. lisp-rs (`vishpat/lisp-rs`)
**Why:** Minimal Lisp interpreter, good for understanding basic patterns.

#### 8. Boa (`nickel-org/boa`) 
**Why:** JavaScript interpreter, complex language semantics.

---

## Analysis Template

For each codebase, produce a report with:

### 1. Architecture Overview
- Main entry points
- Key data structures
- Module organization
- Lines of code estimate

### 2. Environment/Binding Model
```rust
// Show the actual struct definitions
// Explain the design choices
```
- How are scopes represented?
- How is variable lookup implemented?
- How is shadowing handled?
- How are closures captured?

### 3. Value Representation
```rust
// Show the Value enum or equivalent
```
- What's the memory layout?
- How is type checking done?
- How are references handled?
- What's the cost of value operations?

### 4. Function Calling
- How are user-defined functions called?
- How are native Rust functions registered and called?
- What's the calling convention?
- How are arguments passed?

### 5. Ownership/Borrowing (if applicable)
- Is there any ownership tracking?
- How is it implemented?
- What's the performance cost?

### 6. Patterns to Adopt for Treebeard
- Specific code patterns worth copying/adapting
- Design decisions that align with Treebeard's goals

### 7. Patterns to Avoid
- Complexity that doesn't serve Treebeard's goals
- Overengineering for our use case

### 8. Open Questions
- Things that need experimentation
- Unclear tradeoffs

---

## Specific Code Patterns to Extract

When analyzing, look for and extract (with file paths and line numbers):

### Environment Patterns
```
PATTERN: Environment frame chain
FOUND IN: [codebase] [file:line]
CODE: [relevant snippet]
NOTES: [why this matters for Treebeard]
```

### Value Boxing Patterns
```
PATTERN: Tagged pointer / NaN boxing / enum dispatch
FOUND IN: [codebase] [file:line]  
CODE: [relevant snippet]
NOTES: [tradeoffs]
```

### Rust Interop Patterns
```
PATTERN: Registering Rust functions
FOUND IN: [codebase] [file:line]
CODE: [relevant snippet]
NOTES: [how this could work for calling compiled code]
```

### Closure Patterns
```
PATTERN: Closure capture mechanism
FOUND IN: [codebase] [file:line]
CODE: [relevant snippet]
NOTES: [how this interacts with ownership]
```

---

## Synthesis Questions

After analyzing all codebases, synthesize answers to:

1. **What's the best environment representation for Treebeard?**
   - Given: tree-walking (not bytecode), syn AST, need to track ownership
   - Compare approaches from Rhai, Rune, Ketos

2. **What's the minimum viable ownership tracking model?**
   - Given: REPL use case, 10-100x slowdown acceptable, full checking at compile time
   - Extract from Miri what's essential vs what's for full UB detection

3. **How should Treebeard call compiled Rust code?**
   - Given: need to invoke rustc-compiled functions, pass/return Values
   - Compare Rhai's `register_fn`, Rune's modules, Gluon's FFI

4. **What value representation gives the best tradeoff?**
   - Given: need to track ownership state per-value, support Rust's types
   - Compare Dynamic (Rhai), Value (Rune), tagged pointers (rust-hosted-langs)

5. **How should closures work with ownership?**
   - Given: closures may capture by reference or by move
   - This is novel—synthesize from Miri's tracking + interpreter closure patterns

---

## Deliverables

1. **Per-codebase analysis reports** (using template above)
2. **Comparison matrix** of key design decisions across codebases
3. **Recommended patterns document** for Treebeard
4. **Code snippets collection** of patterns to adapt
5. **Risk assessment** of patterns that might not work for our use case

---

## Commands to Get Started

```bash
# Create analysis workspace
mkdir -p ~/treebeard-research/codebases
cd ~/treebeard-research/codebases

# Clone Tier 1 (priority)
git clone --depth 1 https://github.com/rhaiscript/rhai
git clone --depth 1 https://github.com/rust-lang/miri
git clone --depth 1 https://github.com/rune-rs/rune

# Clone Tier 2
git clone --depth 1 https://github.com/gluon-lang/gluon
git clone --depth 1 https://github.com/murarth/ketos
git clone --depth 1 https://github.com/rust-hosted-langs/book

# Get line counts
find . -name "*.rs" | head -20 | xargs wc -l

# Start with Rhai (most relevant)
cd rhai
find src -name "*.rs" | head -30
```

---

## Notes for Claude Code

- Use `rg` (ripgrep) for searching patterns across codebases
- Use `tokei` or `cloc` for line counts
- Focus on understanding *why* design decisions were made, not just *what* they are
- When you find a relevant pattern, trace it through the codebase to understand its implications
- Don't just read code—run tests to understand behavior where helpful
- The goal is actionable insights for Treebeard, not comprehensive documentation of each project
