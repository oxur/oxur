# Treebeard VM Research Prompt

## Context

I'm designing **Treebeard**, a tree-walking interpreter with compilation escape hatches for **Oxur**, a Lisp that compiles to Rust. This is a research task to inform the VM's architecture.

### What Oxur Is

Oxur is a Lisp syntax for Rust semantics. Key characteristics:

- **S-expression syntax** that maps to Rust's semantic model
- **Lisp-1** (single namespace for functions and values, like Rust)
- **Rust's type system** expressed in Lisp forms
- **Rust's pattern matching** (not Erlang's)
- **Rust's ownership/borrowing** (the Lisp forms must express this)
- **Lisp macros** (quote, quasiquote, defmacro) operating on S-expressions
- **AST is essentially `syn` (Rust's AST library) in S-expression form**

Oxur is inspired by LFE (Lisp Flavored Erlang) in spirit - bringing Lisp's joy to a systems language - but the semantics are Rust's, not Erlang's.

### What Treebeard Needs To Do

Treebeard is a **REPL-focused interpreter** that:

1. **Accepts Oxur S-expressions** (the source format)
2. **Expands Lisp macros** (defmacro, quasiquote, etc.)
3. **Evaluates expressions directly** (tree-walking interpretation)
4. **Can invoke the Rust compile chain on demand** (when evaluation alone isn't sufficient)
5. **Supports incremental, interactive development** (define functions, test them, iterate)

Performance expectations: **Interpreted performance is acceptable** (10-100x slower than native). The goal is responsiveness for interactive use, not production performance.

### The Key Architectural Insight

Treebeard is NOT a bytecode VM like BEAM or the JVM. It's a **tree-walking interpreter** that operates directly on S-expression AST nodes. However, it needs **"compilation escape hatches"** - the ability to:

- Recognize when pure interpretation is insufficient
- Generate Rust AST from Oxur AST
- Invoke cargo/rustc
- Load and call compiled code
- Resume interpreted execution

### Constraints

- **Budget**: ~50k lines of Rust maximum for the entire VM
- **Concurrency**: Not initially required, but architecture must not preclude future Rust-like concurrency (async, channels, etc.)
- **Error handling**: Should mirror Rust's Result/panic model, with accommodations for REPL interruption
- **TCO**: Desired but not required (Clojure's `loop/recur` approach is acceptable fallback)

---

## Research Questions

### 1. Tree-Walking Interpreter Design Patterns

Research tree-walking interpreters, particularly those that:
- Operate on AST directly (not bytecode)
- Support Lisp-like macro expansion
- Need to handle a rich type system (not just dynamic types)
- Have been built in Rust

Specific questions:
- What are the tradeoffs of tree-walking vs bytecode compilation for a REPL use case?
- How do existing tree-walking interpreters handle **closures and environment capture**?
- What representation is used for the "environment" (bindings in scope)?
- How is **tail call optimization** achieved in tree-walking interpreters (if at all)?

Look at: Boa (JavaScript), Rune, any Lisp interpreters in Rust, Scheme implementations.

### 2. Macro Expansion in an Interpreter

Research how Lisp interpreters handle macros:
- When does expansion happen? (read-time, compile-time, lazily?)
- How is the macro environment maintained separately from the runtime environment?
- How do interpreters handle **hygiene** (or choose not to)?
- What's the interaction between macros and the REPL? (redefining macros, dependencies)

Look at: How Racket handles macros, how Clojure handles macros (especially given JVM constraints), how traditional Scheme implementations work.

### 3. The BEAM VM (For Contrast and Inspiration)

Research BEAM's architecture, but with a specific lens:

**What we can learn:**
- How does BEAM handle **hot code loading**? (We need something similar for REPL iteration)
- How does BEAM's **module system** work? (We need to understand this for Oxur's module design)
- How does LFE specifically layer on top of BEAM? (Compilation phases, what LFE handles vs what BEAM handles)

**What we explicitly DON'T want:**
- BEAM's actor model and scheduler (Oxur uses Rust's concurrency)
- BEAM's dynamic typing (Oxur uses Rust's type system)
- BEAM's bytecode format (Treebeard is a tree-walker)

Specific question: What aspects of BEAM/LFE's design are **essential to a good Lisp REPL experience** vs which are incidental to Erlang's specific goals?

### 4. Hybrid Interpretation/Compilation Architectures

Research systems that mix interpretation and compilation:

- **JIT compilation strategies**: When do systems decide to compile? How do they handle the transition?
- **CLISP and other Common Lisp implementations**: How do they handle the interpreted/compiled duality?
- **Julia's approach**: Tree-walking for simple cases, LLVM compilation for performance
- **LuaJIT's trace compilation**: Not directly applicable, but the "escape to compiled code" pattern is relevant

Specific questions:
- How do hybrid systems handle **shared state** between interpreted and compiled code?
- How do they handle **function redefinition** when some callers are compiled and some interpreted?
- What's the **calling convention** between interpreted and compiled code?

### 5. Ownership and Borrowing in an Interpreter

This is Oxur's most novel challenge. Research:

- Are there any interpreters that try to enforce ownership semantics at interpretation time?
- How might a tree-walking interpreter **simulate** Rust's borrow checker?
- What's the minimum viable ownership model for REPL exploration? (Can we be more permissive during interpretation, then catch errors at compile time?)

Consider: We may need to accept that full ownership checking only happens at compile time, and the interpreter operates in a more dynamic/GC'd mode. **Research the implications of this hybrid approach.**

### 6. Environment and Binding Representation

Research how interpreters represent:
- **Lexical environments** (for closures)
- **Top-level/global bindings** (for REPL definitions)
- **Type information** (if maintained at runtime)

Specific questions:
- Persistent data structures vs mutable hashmaps for environments?
- How to handle **shadowing** efficiently?
- How to handle **mutual recursion** (functions defined in terms of each other)?
- How to represent **Rust's different binding modes** (let, let mut, static, const)?

### 7. REPL State Management

Research how REPLs maintain state:

- How do Lisp REPLs handle **incremental definition**? (define a function, use it, redefine it)
- How is the **"current module/namespace"** tracked?
- How are **errors handled** without losing state? (evaluation fails, but definitions persist)
- How is **interruption** handled? (user hits Ctrl+C during long computation)

Look at: Clojure's REPL (nREPL), Racket's REPL, Common Lisp's SLIME/SLY interaction model.

### 8. Calling External Code (Crate Loading)

Research how interpreters handle external dependencies:

- How do dynamic language REPLs handle **importing libraries**?
- What's involved in **dynamically loading Rust code**? (dylib loading, ABI concerns)
- How do other Rust-hosted languages (Rhai, Rune, Gluon) handle calling Rust functions?

Specific to Oxur: We need to call out to cargo to build dependencies, load them somehow, and make them callable. What are the options?

---

## Requested Output Format

For each research area, provide:

1. **Summary of findings** (what the research revealed)
2. **Key design decisions** (choices Treebeard needs to make, with tradeoffs)
3. **Recommended approach** (what you'd suggest given Oxur's constraints)
4. **Open questions** (things that need prototyping or further investigation)

At the end, provide:

5. **Architectural sketch** (how the pieces fit together)
6. **Risk assessment** (what's hardest, what might not work)
7. **Suggested prototyping order** (what to build first to validate the design)

---

## Additional Context

The Oxur project has an existing compilation chain:
```
Oxur Source (.oxur) 
  → Reader (text → S-expressions)
  → Macro Expansion (S-expr → S-expr)  
  → Oxur AST (structured S-expressions)
  → Rust AST (syn-based)
  → Rust Source (.rs)
  → rustc → Binary
```

Treebeard inserts into this chain at the "Oxur AST" level. It can:
- Evaluate Oxur AST directly (tree-walking)
- Hand off to the existing Rust AST conversion when compilation is needed

The key insight is that **Oxur AST ≈ S-expression form of syn**, so there's a clean bidirectional mapping.

---

## What Success Looks Like

After this research, I should be able to:

1. **Start implementing Treebeard** with confidence in the core architecture
2. **Know what to defer** (features that can come later)
3. **Understand the risks** (what might require redesign)
4. **Have specific patterns to follow** (not just vague principles)

The goal is a **minimal but sound foundation** that can grow, not a complete design for everything Treebeard might eventually do.
