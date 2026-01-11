# Claude Code Codebase Analysis Prompt: BEAM/LFE/Elixir for Treebeard

## Context

You are analyzing the Erlang/OTP runtime system (BEAM), LFE (Lisp Flavored Erlang), and Elixir to inform the design of **Treebeard**, a tree-walking interpreter for Rust's `syn` AST. While Treebeard is fundamentally different from BEAM (tree-walking vs bytecode, Rust ownership vs GC), there are critical lessons to learn:

**What we want to learn from BEAM/LFE/Elixir:**
- **Hot code loading**: How BEAM achieves code replacement without stopping the system
- **Module system design**: How code is organized, loaded, and versioned
- **REPL implementation**: How LFE's shell supports interactive development with macro definitions
- **Lisp-on-VM layering**: How LFE maps Lisp syntax onto Erlang/BEAM semantics
- **Macro expansion architecture**: How macros are handled in Elixir's compilation pipeline
- **Core Erlang as IR**: How languages compile to a clean intermediate representation

**What we explicitly DON'T want:**
- BEAM's actor model / process scheduler (Oxur uses Rust concurrency)
- BEAM's garbage collection (Treebeard uses Rust ownership)
- BEAM bytecode format (Treebeard is a tree-walker on `syn` AST)
- BEAM's dynamic typing (Oxur uses Rust's type system)

**The key question:** What makes LFE's REPL experience great, and how much of that is essential to the Lisp-on-VM approach vs incidental to Erlang's specific implementation?

---

## Codebases to Analyze

### Tier 1: Primary Focus

#### 1. LFE (`lfe/lfe`)
**Why:** The direct inspiration for Oxur—a Lisp that compiles to a systems language VM.

**Clone:** `git clone https://github.com/lfe/lfe`

**Focus areas:**
- `src/lfe_scan.erl` - Lexer/tokenizer for S-expressions
- `src/lfe_parse.erl` - Parser for LFE syntax
- `src/lfe_macro.erl` - Macro expansion system
- `src/lfe_codegen.erl` - Code generation to Core Erlang
- `src/lfe_eval.erl` - The interpreter/evaluator
- `src/lfe_shell.erl` - REPL implementation
- `src/lfe_env.erl` - Environment handling

**Questions to answer:**
1. **How does LFE handle macro expansion?**
   - When does expansion happen? (read-time vs compile-time)
   - How is the macro environment separate from runtime environment?
   - How does the REPL handle macro redefinition?

2. **How does LFE's evaluator work?**
   - Is it tree-walking or does it compile everything?
   - How does it differ from the compiler path?
   - What can the evaluator do that the compiler can't (and vice versa)?

3. **How does LFE bridge to Erlang?**
   - How are Erlang functions called from LFE?
   - How are LFE functions exposed to Erlang?
   - What's the cost of crossing this boundary?

4. **How does the LFE shell maintain state?**
   - How are definitions persisted across inputs?
   - How is the "current module" tracked?
   - How does it handle errors without losing state?

5. **What's the compilation pipeline?**
   - LFE source → ??? → Core Erlang → BEAM bytecode
   - What are the intermediate representations?
   - Where do macros fit in?

---

#### 2. Erlang/OTP - Code Loading (`erlang/otp`)
**Why:** The definitive implementation of hot code loading.

**Clone:** `git clone --depth 1 https://github.com/erlang/otp`

**Focus areas:**
- `lib/kernel/src/code.erl` - Code server
- `lib/kernel/src/code_server.erl` - Code server implementation
- `erts/preloaded/src/erlang.erl` - Built-in functions
- `lib/compiler/src/core_erlang.pdf` - Core Erlang specification (if present)
- `erts/emulator/beam/beam_load.c` - BEAM loader (C code, skim for concepts)

**Questions to answer:**
1. **How does the two-version system work?**
   - How are "current" and "old" code distinguished?
   - What triggers the transition from current to old?
   - How are processes in old code handled?

2. **How does `code:load_file/1` work?**
   - What's the sequence of operations?
   - How are dependencies handled?
   - What happens to running code?

3. **How do fully-qualified calls enable hot loading?**
   - Why does `Module:function()` pick up new code?
   - Why do local calls stay in current version?
   - How can Treebeard replicate this pattern?

4. **What's the module loading protocol?**
   - How does the code server find modules?
   - How is caching handled?
   - How are load paths managed?

---

#### 3. Elixir Compiler (`elixir-lang/elixir`)
**Why:** Modern, well-documented macro system with hygiene.

**Clone:** `git clone --depth 1 https://github.com/elixir-lang/elixir`

**Focus areas:**
- `lib/elixir/src/elixir_expand.erl` - Macro expansion
- `lib/elixir/src/elixir_quote.erl` - Quote/unquote implementation
- `lib/elixir/src/elixir_env.erl` - Compilation environment
- `lib/elixir/lib/macro.ex` - Macro module (Elixir side)
- `lib/elixir/lib/macro/env.ex` - Environment struct
- `lib/iex/lib/iex.ex` - REPL implementation

**Questions to answer:**
1. **How does Elixir's macro expansion work?**
   - What's the expansion algorithm?
   - How is hygiene implemented (the `:counter` mechanism)?
   - How are nested macros handled?

2. **How does `quote/unquote` work?**
   - What's the AST representation?
   - How does `unquote` get evaluated at the right time?
   - How does `bind_quoted` work?

3. **What's in the `Macro.Env` struct?**
   - What compile-time information is tracked?
   - How is it passed through expansion?
   - What's available to macros?

4. **How does IEx handle incremental definition?**
   - How are modules defined in the REPL?
   - How is redefinition handled?
   - How does it differ from file compilation?

---

### Tier 2: Secondary Reference

#### 4. The BEAM Book (`happi/theBeamBook`)
**Why:** Best documentation of BEAM internals.

**Clone:** `git clone https://github.com/happi/theBeamBook`

**Focus areas:**
- Chapter on Code Loading
- Chapter on the Compiler
- Chapter on BEAM Instructions (skim)

**Questions to answer:**
1. What's the relationship between BEAM and ERTS?
2. How does the code server interact with the loader?
3. What can we learn about module versioning?

---

#### 5. Core Erlang
**Why:** Clean intermediate representation that LFE targets.

**Reference:** `erlang/otp/lib/compiler/src/` - look for core_* files

**Questions to answer:**
1. What does Core Erlang look like?
2. Why is it a good compilation target?
3. What gets desugared vs preserved?

---

## Analysis Template

For each codebase, produce a report with:

### 1. Architecture Overview
- Compilation/evaluation pipeline
- Key modules and their responsibilities
- How it fits with BEAM

### 2. Macro System (if applicable)
```erlang
% Show key data structures
% Explain expansion algorithm
```
- When does expansion happen?
- What information is available to macros?
- How is hygiene handled (or not)?

### 3. Environment/Binding Model
```erlang
% Show environment representation
```
- How are bindings represented?
- How does it differ compile-time vs runtime?
- How do closures capture environment?

### 4. REPL Implementation
- How is state maintained?
- How are definitions persisted?
- How are errors handled?

### 5. Hot Code Loading (BEAM specific)
- The two-version mechanism
- Triggering code switches
- Handling in-flight calls

### 6. Patterns to Adopt for Treebeard
- Design decisions that translate to tree-walking
- REPL patterns worth copying

### 7. Patterns That Don't Apply
- Things specific to bytecode/BEAM
- Things that require GC

---

## Specific Code Patterns to Extract

### LFE Macro Expansion
```
PATTERN: Macro expansion loop
FOUND IN: lfe_macro.erl [function name/line]
CODE: [relevant snippet]
NOTES: [how Oxur macros could work similarly]
```

### Environment Handling
```
PATTERN: Compile-time vs runtime environment separation
FOUND IN: [file:line]
CODE: [relevant snippet]
NOTES: [implications for Treebeard's Environment type]
```

### Hot Code Loading
```
PATTERN: Two-version code management
FOUND IN: code_server.erl [function/line]
CODE: [relevant snippet]
NOTES: [how Treebeard's compilation escape hatch could use this]
```

### REPL State
```
PATTERN: Persisting definitions across evaluations
FOUND IN: lfe_shell.erl [function/line]
CODE: [relevant snippet]  
NOTES: [how treebeard-repl sessions should work]
```

---

## Synthesis Questions

After analyzing, answer these for Treebeard:

### 1. Macro Expansion Strategy
**Given:** Oxur has Lisp-style macros (defmacro, quasiquote), Treebeard receives expanded `syn` AST

- Should Oxur expand macros eagerly (at parse time) or lazily (on use)?
- How should macro dependencies be tracked for REPL use?
- What environment information should macros have access to?

**Compare:** LFE's approach vs Elixir's approach

### 2. Hot Reloading for Tree-Walking
**Given:** Treebeard interprets `syn` AST, has a compilation escape hatch to rustc

- How can we achieve BEAM-like hot reloading without bytecode?
- Should function lookup be late-bound (always get current version)?
- How to handle compiled functions when source is redefined?

**Extract from:** BEAM's two-version system, code_server behavior

### 3. REPL Definition Persistence
**Given:** treebeard-repl uses nREPL protocol, needs session management

- How should `(defn foo ...)` in the REPL persist the definition?
- How should redefinition work?
- What state needs to survive errors?

**Extract from:** LFE shell, IEx

### 4. Thin Layer Architecture
**Given:** oxur-vm should be thin over treebeard-core

- What does LFE handle vs what does BEAM handle?
- Where's the right boundary for Oxur vs Treebeard?
- What should frontends NOT do?

**Extract from:** LFE's layering on BEAM

### 5. Lisp-to-Target Mapping
**Given:** Oxur maps S-expressions to `syn` AST (Rust semantics)

- How does LFE map Lisp forms to Erlang semantics?
- What Lisp features don't map cleanly? How are they handled?
- What compromises does LFE make?

**Extract from:** LFE design decisions, Robert Virding's talks/papers

---

## Deliverables

1. **Per-codebase analysis reports** focusing on Treebeard-relevant aspects
2. **Macro expansion comparison** (LFE vs Elixir vs recommendations for Oxur)
3. **Hot code loading patterns** adaptable to tree-walking
4. **REPL implementation guide** based on LFE shell and IEx
5. **Thin layer principles** from LFE's architecture

---

## Commands to Get Started

```bash
# Create analysis workspace
mkdir -p ~/treebeard-research/beam-codebases
cd ~/treebeard-research/beam-codebases

# Clone LFE (primary focus)
git clone https://github.com/lfe/lfe

# Clone Elixir (for macro system)
git clone --depth 1 https://github.com/elixir-lang/elixir

# Clone OTP (for code loading - large!)
git clone --depth 1 --filter=blob:none --sparse https://github.com/erlang/otp
cd otp
git sparse-checkout set lib/kernel/src lib/compiler/src erts/preloaded/src
cd ..

# Clone BEAM Book
git clone https://github.com/happi/theBeamBook

# Start with LFE - it's the most directly relevant
cd lfe
find src -name "*.erl" | head -20

# Key files to examine first:
# - src/lfe_macro.erl (macro expansion)
# - src/lfe_shell.erl (REPL)
# - src/lfe_eval.erl (evaluator)
# - src/lfe_codegen.erl (compilation to Core Erlang)
```

---

## Key Insight to Keep in Mind

LFE proves that a Lisp can successfully layer on top of a systems language VM while maintaining full interoperability. The key lessons are:

1. **Thin layer principle**: LFE does syntax + macros, BEAM does everything else
2. **Semantic alignment**: LFE embraces Erlang's semantics rather than fighting them
3. **Interop is paramount**: Zero-cost calls to Erlang functions
4. **REPL flexibility**: Can do things in the REPL that you can't in compiled code

Oxur/Treebeard is doing the same thing for Rust that LFE did for Erlang. The difference is that Rust has ownership semantics that need to be addressed—but the layering principle remains.

---

## Notes for Claude Code

- Erlang code uses different conventions than Rust—atoms start lowercase, variables uppercase
- LFE is written in Erlang, so you're reading `.erl` files
- The BEAM Book is in AsciiDoc format—read the source files directly
- Focus on the *architectural decisions* not implementation details of BEAM
- When you find something relevant, think "how would this work in a tree-walker?"
- The goal is design patterns for Treebeard, not understanding BEAM completely
