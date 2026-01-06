---
number: 1
title: "Oxur: A Letter of Intent"
author: "Duncan McGreggor"
component: All
tags: [vision, architecture]
created: 2025-12-25
updated: 2026-01-05
state: Draft
supersedes: null
superseded-by: null
version: 1.2
---


# Oxur: A Letter of Intent

**Status**: Vision & Design Exploration
**Version**: 1.2
**Date**: January 2026
**Mission**: To create a Lisp that compiles to Rust with 100% interop, drawing inspiration from Zetalisp, LFE, and Clojure's thoughtful design

---

## The Vision

We're creating Oxur - a Lisp dialect that treats Rust as its compilation target and runtime, with complete bidirectional interoperability. This isn't a Lisp implemented *in* Rust; it's a Lisp that *becomes* Rust, leveraging Rust's type system, ownership model, and entire ecosystem while providing Lisp's expressiveness and metaprogramming power.

Unlike Zylisp (our Lisp-on-Go project), which must work around Go's plugin memory leaks and limited type system, Oxur benefits from Rust's superior design:

- No plugin memory leak issues
- Richer type system with traits, lifetimes, and const generics
- First-class pattern matching in the AST
- Cleaner AST structure with consistent `Foo`/`FooKind` patterns
- Stronger safety guarantees we can expose to Lisp programmers

## The Name

**Oxur** /ˈɒk.sər/ combines:

- **Ox** - strength, reliability (like Rust's mascot Ferris)
- **ur** - primordial, foundational (as in Ur-Lisp)
- Phonetic echo of "oxidize" (Rust's theme)

The name suggests both power and ancient wisdom - a modern Lisp with timeless principles, forged in Rust.

## Core Philosophy

### Rust Semantics, Lisp Syntax

Go is a Lisp-1 (functions and variables share a namespace). Rust is also effectively Lisp-1. This semantic alignment is fundamental - we're not imposing Lisp conventions on Rust, we're revealing Rust's inner Lisp nature.

Key principles:

1. **100% Rust Interop from Day One** - Not bolted on, built in
2. **Ownership as a Feature** - Express borrowing and lifetimes naturally
3. **Traits over Objects** - Embrace Rust's trait system fully
4. **Pattern Matching Everywhere** - It's first-class in both languages
5. **Safety by Default** - Leverage Rust's guarantees
6. **Zero-Cost Abstractions** - Compile to idiomatic Rust
7. **Explicit over Implicit** - Make lifetimes and types visible when needed

### Design Inspirations

**Zetalisp** - Our aesthetic guide:

- Keyword arguments (`:type`, `:lifetime`)
- Flavors system maps naturally to traits
- Clean, orthogonal design
- Keyword-based syntax for rich metadata

**LFE** (Lisp Flavored Erlang):

- Pattern matching as core feature
- Respect for the host language's semantics
- Robert Virding's wisdom on namespace choices
- Syntax that feels natural to both communities

**Clojure**:

- Thoughtful API design
- Rich data literals
- Pragmatic approach to host interop
- But we'll forge our own naming conventions, not copy Clojure's

**Rust's Own Philosophy**:

- Fearless concurrency
- Zero-cost abstractions
- Move semantics and ownership
- Explicit lifetimes
- Trait-based polymorphism

## The Big Architectural Decisions

### Two-Stage Compilation (Like Zylisp)

This worked brilliantly for Go and will work even better for Rust:

```
Oxur Syntax → Core Forms (IR) → Rust AST → Rust Code → Binary
  (Stage 1)         (IR)          (Stage 2)
```

**Stage 1**: The Oxur Compiler (`oxur-comp`)

- Parses Oxur syntax
- Expands macros
- Type checking and inference (optional)
- Compiles to canonical S-expressions (Core Forms)

**Stage 2**: The Rust AST Layer (`oxur-ast`)

- Bidirectional Rust AST ↔ S-expression conversion
- Stable "assembly language" for Rust
- 1:1 mapping - explicit and complete
- Rarely changes, rock solid

**Why this separation matters**:

- Experiment with Oxur syntax without touching Rust interop
- Core Forms are the stable contract between stages
- Debug by inspecting the IR
- Other tools can target Core Forms
- Stage 2 can be used independently (useful for Rust tooling!)

### Canonical S-Expressions as IR

Following Zylisp's success, we'll use S-expressions as our intermediate representation:

- Every field of Rust's AST represented
- All position information preserved (for error messages)
- Keyword arguments for clarity
- Bidirectional transformation guaranteed

Example:

```lisp
(Item
  :attrs ()
  :id 12
  :span (Span :lo 0 :hi 0)
  :vis (Inherited)
  :ident (Ident :name "add" :span (Span :lo 0 :hi 0))
  :kind (Fn
    (Fn
      :defaultness Final
      :sig (FnSig
        :header (FnHeader :safety Default :constness NotConst :ext None :coroutine-kind nil)
        :decl (FnDecl
          :inputs ((Param
              :attrs ()
              :ty (Ty
                :id 1
                :kind (Path
                  nil
                  (Path
                    :span (Span :lo 0 :hi 0)
                    :segments ((PathSegment :ident (Ident :name "i32" :span (Span :lo 0 :hi 0)) :id 4294967295 :args nil))))
                :span (Span :lo 0 :hi 0))
              :pat (Pat
                :id 0
                :kind (Ident
                  :binding-mode (ByValue Not)
                  :ident (Ident :name "a" :span (Span :lo 0 :hi 0))
                  :sub nil)
                :span (Span :lo 0 :hi 0))
              :id 2
              :span (Span :lo 0 :hi 0)
              :is-placeholder false)
            (Param
              :attrs ()
              :ty (Ty
                :id 4
                :kind (Path
                  nil
                  (Path
                    :span (Span :lo 0 :hi 0)
                    :segments ((PathSegment :ident (Ident :name "i32" :span (Span :lo 0 :hi 0)) :id 4294967295 :args nil))))
                :span (Span :lo 0 :hi 0))
              :pat (Pat
                :id 3
                :kind (Ident
                  :binding-mode (ByValue Not)
                  :ident (Ident :name "b" :span (Span :lo 0 :hi 0))
                  :sub nil)
                :span (Span :lo 0 :hi 0))
              :id 5
              :span (Span :lo 0 :hi 0)
              :is-placeholder false))
          :output (Ty
            (Ty
              :id 6
              :kind (Path
                nil
                (Path
                  :span (Span :lo 0 :hi 0)
                  :segments ((PathSegment :ident (Ident :name "i32" :span (Span :lo 0 :hi 0)) :id 4294967295 :args nil))))
              :span (Span :lo 0 :hi 0))))
        :span (Span :lo 0 :hi 0))
      :generics (Generics
        :params ()
        :where-clause (WhereClause :has-where-token false :predicates () :span (Span :lo 0 :hi 0))
        :span (Span :lo 0 :hi 0))
      :body (Block
        :stmts ((Stmt
            :id 10
            :kind (Expr
              (Expr
                :id 9
                :kind (Binary
                  :left (Expr
                    :id 7
                    :kind (Path
                      nil
                      (Path
                        :span (Span :lo 0 :hi 0)
                        :segments ((PathSegment :ident (Ident :name "a" :span (Span :lo 0 :hi 0)) :id 4294967295 :args nil))))
                    :span (Span :lo 0 :hi 0)
                    :attrs ())
                  :op Add
                  :right (Expr
                    :id 8
                    :kind (Path
                      nil
                      (Path
                        :span (Span :lo 0 :hi 0)
                        :segments ((PathSegment :ident (Ident :name "b" :span (Span :lo 0 :hi 0)) :id 4294967295 :args nil))))
                    :span (Span :lo 0 :hi 0)
                    :attrs ()))
                :span (Span :lo 0 :hi 0)
                :attrs ()))
            :span (Span :lo 0 :hi 0)))
        :id 11
        :rules Default
        :span (Span :lo 0 :hi 0)
        :could-be-bare-literal false))))
```

This becomes Rust:

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Wire Format: Postcard for Rust, MessagePack for Polyglot

**Initial choice (v0.1)**: **postcard**

- 3.4x faster serialization than MessagePack
- Smaller wire format (724KB vs 784KB in benchmarks)
- Rust-native ecosystem integration with `postcard-rpc`
- Stable wire format since v1.0.0 (June 2022)
- Acceptable trade-off: Rust-only clients initially

**Migration path (v0.2+)**: **MessagePack** support

- Multi-protocol server (different ports: 7888 for postcard, 7889 for msgpack)
- 50+ language implementations available
- ~50 lines of code to add (trait-based abstraction)
- Zero breaking changes for existing clients
- Enables Python, JavaScript, Clojure clients

The architecture separates protocol semantics from wire serialization via a `Protocol` trait, allowing both formats to coexist when cross-language demand emerges. No premature optimization - start with optimal Rust performance, add polyglot support when needed.

### No Plugin Memory Leak Problem

Unlike Go, Rust doesn't have the plugin memory leak that forced Zylisp into a complex supervised worker architecture. This simplifies our REPL design significantly:

- No need for disposable worker processes that accumulate leaked memory
- No memory monitoring complexity
- Simpler supervision model (optional, not required)
- Cached artifacts persist across sessions without memory pressure

We still use subprocess execution for other reasons (see REPL Architecture below), but the subprocess is stable and long-lived rather than disposable.

## The Rust Challenge: Ownership and Lifetimes

This is the big one - the feature that makes Rust unique and powerful, but also the feature that needs the most careful Lisp representation.

### Ownership in Oxur

We need natural syntax for Rust's ownership operations:

```lisp
;; Borrow (immutable reference)
(borrow x)              ; &x

;; Borrow mutable
(borrow-mut x)          ; &mut x

;; Dereference
(deref x)               ; *x

;; Move (explicit, though default in Rust)
(move x)                ; Clarifies intent

;; Clone
(clone x)               ; x.clone()
```

These are **fundamental operations**, not library functions. They compile directly to Rust's ownership primitives.

### Lifetimes in Oxur

Lifetimes need to be visible but not overwhelming. Inspired by Zetalisp's keyword arguments:

```lisp
;; Function with lifetime parameters
(deffn foo ('a) ((x (& 'a str))) (& 'a str)
  x)

;; Struct with lifetime
(defstruct Holder ('a)
  :fields ((data (& 'a str))))

;; Lifetime bounds in trait implementations
(impl (Display) for (Holder ('a))
  (deffn fmt ((self) (f (& mut Formatter))) Result
    ...))
```

The `'a` notation mirrors Rust directly - familiar to Rust programmers, not too foreign to Lispers.

### The Creative Naming Challenge

For each Rust feature, we ask: "If this was in Zetalisp or LFE, what would it be called?"

Some ideas to explore:

- `deffn` vs `deffunc` vs `fn` for function definition
- `defstruct` for structs (like Lisp tradition)
- `deftrait` or `protocol` for traits?
- `impl` stays `impl`? Or `implement`?
- `match` stays `match`? (Already Lispy!)
- `let` vs `bind` vs `var`?

We'll make these decisions iteratively, favoring:

1. Zetalisp/LFE aesthetic
2. Clarity for newcomers
3. Brevity without obscurity
4. Rust familiarity where it helps

## Pattern Matching: A Gift from Both Sides

Pattern matching is first-class in both Rust and Lisp traditions. Rust's `PatKind` enum in the AST gives us exhaustive pattern support out of the box.

```lisp
;; Simple match
(match value
  (Some x) (print x)
  (None) (print "nothing"))

;; Destructuring
(match point
  ((Point x y)) (+ x y))

;; Guards
(match number
  (x :when (> x 0)) "positive"
  (x :when (< x 0)) "negative"
  (_) "zero")

;; Nested patterns
(match nested
  ((Ok (Some (value))) ...)
  ((Ok (None)) ...)
  ((Err e) ...))
```

This is natural in both languages and will be a joy to use.

## Traits: Rust's Polymorphism Model

Traits are more powerful than Go's interfaces. We need syntax that captures:

- Trait definitions with associated types
- Trait bounds and where clauses
- Trait implementations (both inherent and for traits)
- Generic bounds
- Lifetime bounds on traits

```lisp
;; Trait definition
(deftrait Display
  (deffn fmt ((self) (f (& mut Formatter))) Result))

;; Trait with associated type
(deftrait Iterator
  :associated ((Item type))
  (deffn next ((self (& mut Self))) (Option Item)))

;; Generic function with trait bounds
(deffn print-all (T) ((items (Vec T)))
  :where ((T Display))
  (for item items
    (println "{}" item)))

;; Trait implementation
(impl (Display) for Point
  (deffn fmt ((self) (f (& mut Formatter))) Result
    (write f "({}, {})" self.x self.y)))
```

The `:where` keyword for bounds feels very Zetalisp. The `(T)` notation for generics is borrowed from Clojure but adapted for Rust's conventions.

## Type System Integration

Rust's type system is rich. Oxur needs to expose it without overwhelming:

### Explicit Type Annotations (Optional)

```lisp
;; Simple types
(let ((x i32 42)
      (name String "Alice")))

;; Function signatures
(deffn add ((a i32) (b i32)) i32
  (+ a b))

;; Generic types
(deffn first (T) ((vec (Vec T))) (Option T)
  (get vec 0))

;; Complex types
(let ((callback (Fn (i32) -> i32) ...)))
```

### Type Inference

Where Rust can infer, Oxur can too:

```lisp
;; Inferred types
(let ((x 42)           ; i32 inferred
      (v (vec 1 2 3))  ; Vec<i32> inferred
      (s "hello")))    ; &str inferred
```

### Const Generics

Rust supports const generics (type-level integers):

```lisp
;; Array with const generic size
(deffn sum-array (const N) ((arr (i32; N))) i32
  (fold arr 0 +))
```

This is advanced but important for full Rust compatibility.

## Macros: The Lisp Superpower

Oxur macros compile to Rust code, not to Rust macros. This is important:

```lisp
;; Oxur macro
(defmacro when (condition & body)
  `(if ,condition
     (do ,@body)))

;; Expands during Stage 1 compilation to Core Forms
;; Then Core Forms compile to Rust code
```

This means Oxur macros have full Lisp power at compile time, generating arbitrary Core Forms, which then compile to efficient Rust.

We might also provide a way to invoke Rust macros from Oxur:

```lisp
;; Call Rust's println! macro
(rust-macro! println "Hello {}" name)

;; Or perhaps
(println! "Hello {}" name)
```

This needs design work, but the Rust AST's `MacCall` nodes suggest it's possible.

### Phased Macro System

**Phase 1 (v1.0)**: Core macros only

- Pre-compiled by Oxur team
- Shipped as `core-macros.so` dynamic library
- Examples: `deffn`, `when`, `->`, `when-let`, `unless`, `cond`
- Loaded at compiler startup
- 10-15 essential macros cover common patterns

**Phase 2 (v2.0)**: User-definable macros

- Multi-pass compilation with dependency graph
- Layer-by-layer compilation (topological sort)
- Cycle detection
- Produces `user-macros.so` alongside `core-macros.so`

This phased approach ships a complete, usable language in v1.0 while deferring the complexity of user macro compilation to v2.0.

## Module System

Rust's module system is explicit and hierarchical:

```lisp
;; Module declaration
(mod geometry
  (defstruct Point
    :fields ((x i32) (y i32)))

  (deffn distance ((p1 Point) (p2 Point)) f64
    ...))

;; Using items
(use geometry::Point)
(use geometry::distance)

;; Or with aliases
(use (geometry::Point :as Pt))
```

Rust's visibility rules map naturally:

```lisp
;; Public items
(pub defstruct Point ...)
(pub deffn create-point ...)

;; Crate-public
(pub-crate deffn internal-helper ...)

;; Module-private (default)
(deffn private-impl ...)
```

## Error Handling: Result and Option

Rust's error handling is explicit and built around `Result` and `Option`:

```lisp
;; Result type
(deffn divide ((a i32) (b i32)) (Result i32 String)
  (if (= b 0)
    (Err "division by zero")
    (Ok (/ a b))))

;; Question mark operator
(deffn compute () (Result i32 Error)
  (let ((x (foo?))        ; equivalent to foo()?
        (y (bar?)))
    (Ok (+ x y))))

;; Option type
(deffn find ((items (Vec String)) (target String)) (Option usize)
  (for-indexed item items
    (when (= item target)
      (return (Some index))))
  None)
```

The `?` operator is central to Rust ergonomics. We need good syntax for it.

## Concurrency: Fearless by Default

Rust's ownership makes concurrency safe. Oxur inherits this:

```lisp
;; Spawn a thread
(use std::thread)

(deffn main ()
  (let ((handle (thread::spawn
                  (fn ()
                    (println "Hello from thread!")))))
    (join handle)))

;; Channels
(use std::sync::mpsc)

(deffn main ()
  (let (((tx rx) (mpsc::channel)))
    (thread::spawn
      (move (fn ()
        (send tx "Hello!"))))
    (println "Received: {}" (recv rx))))
```

The `move` keyword is crucial for closures that capture their environment.

## The REPL: Production Network Protocol from Day One

Rust doesn't have Go's plugin memory leak problem, which means our REPL architecture avoids the complex disposable-worker model that Zylisp required. However, comprehensive research into evcxr (Rust's existing REPL/Jupyter kernel, 6+ years of production use) revealed a critical constraint: **Rust threads cannot be interrupted**. This means subprocess execution is mandatory for Ctrl-C support.

### REPL Architecture: Subprocess-Based Execution

```
┌────────────────────────────────────────────────────────────┐
│                      REPL Server                            │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  Tier 1: Calculator Mode                              │ │
│  │  Pure arithmetic: <1ms (no compilation)               │ │
│  └───────────────────────────────────────────────────────┘ │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  Tier 2: Artifact Cache                               │ │
│  │  Previously compiled: 1-5ms (load + execute)          │ │
│  └───────────────────────────────────────────────────────┘ │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  Tier 3: JIT Compilation                              │ │
│  │  New code: 50-300ms (compile + execute)               │ │
│  └───────────────────────────────────────────────────────┘ │
│                            │                                │
│                            ↓ stdin/stdout                   │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  Subprocess (isolated process)                        │ │
│  │  - Loads compiled dynamic libraries                   │ │
│  │  - Executes user code in isolation                    │ │
│  │  - Can be killed (Ctrl-C support!)                    │ │
│  │  - Crash doesn't affect REPL server                   │ │
│  └───────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────┘
```

**Why subprocess execution is mandatory:**

1. **Ctrl-C support** - Rust threads cannot be forcibly stopped (by design, for memory safety). To interrupt an infinite loop, you must kill the process.
2. **Crash isolation** - If user code panics, only the subprocess dies. The REPL server continues running with all session state preserved.
3. **Clean recovery** - Subprocess can be restarted transparently after errors.
4. **Memory isolation** - Separate address space prevents corruption.

**What's simpler than Zylisp:**

- No disposable workers accumulating leaked memory
- No complex memory monitoring
- Long-lived subprocess (not constantly recreated)
- Lightweight IPC via stdin/stdout (~100-200μs overhead, negligible vs compilation time)

**Artifact caching is mandatory from day one:**

- Cache location: `~/.cache/oxur/artifacts/`
- Cache key: SHA256(source + deps + opt_level + source_map)
- Cache hit: 1-5ms (vs 50-300ms compilation)
- This was evcxr's "biggest regret" - they waited 5 years to add it

### Network Protocol: nREPL-Inspired

**Core Protocol Operations** (v0.1):

- `clone` - Create new session (returns session UUID)
- `eval` - Evaluate code (with mode: lisp/sexpr)
- `load-file` - Load and evaluate file
- `interrupt` - Cancel running evaluation (kills subprocess, restarts)
- `close` - Close session
- `describe` - Server capabilities/version
- `ls-sessions` - Active session enumeration

The `mode` parameter enables **dual-mode evaluation** (Oxur syntax vs s-expression AST) within a single protocol, allowing users to switch modes mid-session or run different clients in different modes against the same server.

**Message Structure** (postcard-encoded):

```rust
#[derive(Serialize, Deserialize)]
struct Request {
    id: String,           // UUID correlation ID
    session: String,      // Session UUID
    op: Operation,        // Clone, Eval, Interrupt, etc.
    mode: ReplMode,       // Lisp or Sexpr
    params: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
struct Response {
    id: String,           // Echoed from request
    session: String,
    value: Option<String>,
    out: Option<String>,  // Streaming stdout
    err: Option<String>,  // Streaming stderr
    status: Vec<Status>,  // ["done"], ["error"], ["interrupted"]
}
```

**Subprocess Protocol** (internal, stdin/stdout text):

```
Server → Subprocess:
  "LOAD_AND_RUN /path/to/libeval_005.so run_user_code_5\n"

Subprocess → Server:
  "OXUR_EXECUTION_COMPLETE\n"        (success)
  "OXUR_RUNTIME_ERROR: message\n"    (panic/error)
  "OXUR_PANIC_LOCATION: file:line\n" (optional stack info)
```

### Transport Abstraction

Unified API across all connection types via Tokio's `AsyncRead + AsyncWrite`:

**Available transports:**

- **TCP**: Primary for remote connections (port 7888)
- **Unix sockets**: Low-latency local IPC (Linux/macOS)
- **Named pipes**: Windows-native local IPC
- **In-process channels**: Zero-overhead testing/embedding

The `TransportListener` trait enables zero-cost monomorphization - no runtime overhead for the abstraction. All transport implementations satisfy the same bounds:

```rust
async fn handle_stream<S>(stream: S) -> io::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    // Protocol handling identical regardless of transport
}
```

### Multi-Protocol Future (v0.2+)

When cross-language client demand emerges, adding MessagePack support is trivial:

```
┌─────────────────────────────────────────┐
│  Postcard Server (port 7888)            │
│  ↓ Rust clients (optimal performance)   │
└─────────────────────────────────────────┘

┌─────────────────────────────────────────┐
│  MessagePack Server (port 7889)         │
│  ↓ Python/JS/Clojure clients            │
└─────────────────────────────────────────┘

Both share: EvalEngine, SessionManager, protocol semantics
```

The trait-based architecture means adding MessagePack is ~50 lines of code with zero breaking changes for existing postcard clients.

### Supervision: OTP-Style (Optional)

We still want reliability-first design, but adapted for Rust:

```rust
// Supervisor in Rust (using tokio's supervision patterns)
let supervisor = Supervisor::new(OneForOne);

supervisor.add_child(ChildSpec {
    id: "repl-server",
    start: start_repl_server,
    restart: Permanent,
});

supervisor.start();
```

But this is **optional** - Rust's safety means crashes are rarer. The REPL can run perfectly well as a simple server with subprocess isolation providing the critical reliability guarantees.

## Repository Structure

Following Zylisp's successful pattern, adapted for the current codebase:

```
oxur/crates/
├── oxur-smap/          # Source mapping foundation (no dependencies)
├── oxur-ast/           # Rust AST ↔ S-expr (Stage 2) ✅ IMPLEMENTED
├── oxur-comp/          # Oxur compiler (Stages 1-2)
├── oxur-lang/          # Language definition, core macros
├── oxur-repl/          # REPL server/client/protocol + subprocess
├── oxur-cli/           # CLI tool (oxur command)
├── oxur-table/         # Table formatting utility ✅ IMPLEMENTED
└── design/             # Design documents & CLI ✅ IMPLEMENTED
```

**Dependency graph** (no circles!):

```
                    oxur-smap (foundation, no deps)
                         ↑
        ┌────────────────┼────────────────┐
        ↓                ↓                ↓
    oxur-ast         oxur-lang        oxur-comp
        ↑                ↓                ↑
        └────────────────┼────────────────┘
                         ↓
                     oxur-repl
                         ↓
                     oxur-cli

(core-macros.so compiled from oxur-lang/core-macros/)
```

Clean, testable, maintainable. The `oxur-smap` crate is the foundation - it provides source position tracking across all compilation stages, enabling rustc-quality error messages that point back to original Oxur source.

## Development Timeline (18 Weeks to v1.0)

### Phase 0: Foundation (Weeks 1-2) ✅ COMPLETE

**Goal:** Set up project structure and basic tooling

- [x] Create repositories (`oxur-ast`, `design`)
- [x] Set up CI/CD
- [x] Define Core Forms specification (Document 0003)
- [x] Implement Node ID generator and source map types
- [x] Write project README and contribution guidelines

### Phase 1: Parse & Source Maps (Weeks 3-4) - IN PROGRESS

**Goal:** Implement Stage 1 (Parse) with source map tracking

- [x] S-expression lexer with position tracking (`oxur-ast`) ✅
- [x] S-expression parser (tokens → S-expressions) ✅
- [ ] Reader (S-expressions → Surface Forms)
- [ ] Node ID assignment for all forms
- [ ] Input layer source map creation
- [ ] Parse error reporting with context

**Current status:** S-expression infrastructure complete, need to add Surface Forms layer

### Phase 2: Core Forms & Lowering (Weeks 5-6)

**Goal:** Define Core Forms and implement Stage 3 (Lower)

**Deliverables:**

- [ ] Complete Core Form types in `oxur-comp`
- [ ] Core Form → Rust AST lowering
- [ ] Rust AST → Core Form lifting (for testing)
- [ ] Round-trip tests (Rust → Core Forms → Rust)
- [ ] Source map for lowering stage

### Phase 3: Expansion (Weeks 7-8)

**Goal:** Implement Stage 2 (Expand) with core macros

**Deliverables:**

- [ ] Expander implementation
- [ ] Desugaring for common syntax sugar
- [ ] Core macro framework
- [ ] Implement 3-5 essential core macros
- [ ] Macro expansion tests
- [ ] Source map for expansion stage

### Phase 4: Generation & End-to-End (Weeks 9-10)

**Goal:** Complete the pipeline, compile Hello World

**Deliverables:**

- [ ] Stage 4 (Generate) implementation
- [ ] Stage 5 (Compile) integration
- [ ] End-to-end compilation test
- [ ] Compile and run "Hello, World!"
- [ ] Error translation from rustc errors
- [ ] Full pipeline integration tests

### Phase 5: Core Macros Library (Weeks 11-12)

**Goal:** Build comprehensive core macro library

**Deliverables:**

- [ ] Control flow macros (`when`, `unless`, `cond`)
- [ ] Threading macros (`->`, `->>`)
- [ ] Let variants (`when-let`, `if-let`)
- [ ] Loop helpers (`dotimes`, `doseq`)
- [ ] Core macro compilation infrastructure
- [ ] Core macro tests
- [ ] Documentation for core macros

### Phase 6: REPL (Weeks 13-14)

**Goal:** Build working REPL with subprocess execution and artifact caching

**Deliverables:**

- [ ] Subprocess executor with stdin/stdout protocol
- [ ] Artifact cache (mandatory, day-one requirement)
- [ ] Tier 1 calculator for simple arithmetic
- [ ] Tier 2 cache lookup
- [ ] Tier 3 JIT compilation
- [ ] REPL server implementation
- [ ] REPL client (CLI)
- [ ] Network protocol (postcard)
- [ ] Multi-transport support (TCP, Unix, named pipes)
- [ ] Ctrl-C interrupt handling
- [ ] REPL tests
- [ ] REPL user documentation

### Phase 7: CLI & Tooling (Weeks 15-16)

**Goal:** Polish CLI and add essential tools

**Deliverables:**

- [ ] `oxur build` command
- [ ] `oxur repl` command (default when no command given)
- [ ] `oxur check` command
- [ ] `oxur format` command
- [ ] Project configuration (`Oxur.toml`)
- [ ] Build caching
- [ ] Error message improvements
- [ ] CLI documentation

### Phase 8: v1.0 Release (Weeks 17-18)

**Goal:** Polish, documentation, release

**Deliverables:**

- [ ] Complete documentation
- [ ] Tutorial and examples
- [ ] Performance benchmarks
- [ ] Release notes
- [ ] Blog post announcing v1.0
- [ ] Package for distribution

## Key Architectural Improvements Over Zylisp

### Simpler REPL (No Memory Leak Problem)

- **No plugin memory leak** → subprocess is stable and long-lived
- **No complex memory monitoring** → no disposable workers
- **Subprocess execution** for Ctrl-C support and crash isolation
- **Lightweight IPC** via stdin/stdout (~100-200μs, negligible)
- **Mandatory artifact caching** from day one (not an afterthought)

### Better Protocol

- **postcard/MessagePack** instead of bencode
  - 3.4x faster serialization
  - ~50% smaller wire format
  - Full type support (floats, booleans)
- **Multi-transport from day one** (not just TCP)
  - Unix sockets, named pipes, in-process channels
- **nREPL-inspired but Rust-native**
  - Streaming output
  - Session isolation
  - Correlation IDs for multiplexing

### More Capable Type System

- **Expose Rust's lifetimes, traits, const generics**
- **No runtime library needed** (Rust's type system is powerful enough)
- **First-class pattern matching in AST**
- **Ownership operations as primitives** (borrow, borrow-mut, move, clone)

### Cleaner AST

- **Rust's `Foo`/`FooKind` pattern** more systematic than Go's variety
- **Smaller surface area** despite being more expressive
- **Better position tracking** (`Span` vs `token.Pos`)
- **S-expression format uses keyword arguments** for clarity

### Phased Macro System

- **Core macros pre-compiled** (v1.0) - shipped as `core-macros.so`
- **User macros deferred to v2.0** - ships complete language without complexity
- **Native compilation** instead of interpretation
- **Dependency graph resolution** for layer-by-layer compilation

### Source Mapping Foundation

- **oxur-smap crate** - dedicated source mapping (no dependencies)
- **Multi-stage tracking** - Surface → Core → Rust → Error translation
- **Unique differentiator** - no other Lisp has this level of error fidelity
- **rustc-quality error messages** pointing to original Oxur source

## Success Criteria

We'll know Oxur is succeeding when:

### Core Compilation

1. **Round-trip works**: Rust → S-expr → Rust produces equivalent code
2. **Hello World compiles**: Basic Oxur programs generate working binaries
3. **FFI is seamless**: Calling Rust from Oxur feels natural
4. **Ownership feels right**: Borrowing and lifetimes aren't fighting the language
5. **Patterns are beautiful**: Match expressions are clean and powerful
6. **Traits are accessible**: The trait system is approachable
7. **Macros are powerful**: Compile-time metaprogramming works as expected

### REPL & Protocol

1. **Sub-millisecond calculator**: Pure arithmetic in <1ms (Tier 1)
2. **Fast cache hits**: Previously compiled code in 1-5ms (Tier 2)
3. **Responsive JIT**: New code compiles in 50-300ms (Tier 3)
4. **Ctrl-C works**: Interrupt infinite loops cleanly
5. **Multi-transport works**: Same code runs over TCP, Unix sockets, named pipes
6. **Cross-language clients**: Python/JavaScript clients can connect via MessagePack (v0.2+)
7. **Streaming output**: Incremental stdout/stderr during evaluation
8. **Mode switching**: Seamlessly switch between Lisp syntax and s-expression modes

### Community & Ecosystem

1. **Rust community approves**: Rustaceans see Oxur as idiomatic
2. **Lisp community approves**: Lispers feel at home
3. **Production deployments**: People build real things with Oxur

## Why This Matters

**For Rust Developers:**

- Powerful metaprogramming capabilities
- Alternative syntax for Rust's semantics
- Rapid prototyping with REPL
- Code generation and analysis tools
- Network REPL protocol for tooling integration

**For Lisp Enthusiasts:**

- Modern, safe, fast language with Lisp manipulation
- Access to Rust's amazing ecosystem
- Zero-cost abstractions with Lisp expressiveness
- Pattern matching as a first-class citizen
- Real ownership and lifetime control

**For Everyone:**

- Exploring language design boundaries
- Bridging paradigms thoughtfully
- Learning through alternative representations
- Having fun with powerful tools
- Building a production-grade network REPL protocol for Rust

## Closing Thoughts

Oxur sits at the intersection of three powerful traditions:

- **Lisp's elegance**: Code as data, metaprogramming, REPL-driven development
- **Rust's safety**: Ownership, lifetimes, fearless concurrency
- **Zetalisp's beauty**: Clean design, keyword arguments, orthogonality

We're not making Rust into Lisp or Lisp into Rust. We're revealing that Rust *already has* a Lisp hiding inside it - we're just giving it S-expression syntax and the full power of homoiconicity.

**Version 1.2 updates** incorporate architectural clarity from comprehensive evcxr research and detailed REPL design work:

- **Subprocess execution**: Mandatory for Ctrl-C support (Rust threads can't be interrupted)
- **Artifact caching**: Day-one requirement, not a future optimization
- **Source mapping foundation**: oxur-smap crate for rustc-quality error messages
- **Simplified IPC**: stdin/stdout protocol (proven 6+ years in evcxr)
- **Updated architecture diagrams**: Reflect actual subprocess-based design

This is going to be a phenomenal journey. We have the benefit of Zylisp's lessons, Rust's superior design, evcxr's 6+ years of battle-tested patterns, and a clear architectural vision. The hard problems are obvious (lifetimes, ownership, traits), but they're *interesting* hard problems, not insurmountable ones.

Let's build something beautiful.

**Onward! 🦀✨**

---

*"In Lisp, code is data. In Rust, safety is fearless. In Oxur, we get both."*

---

## Appendix: Changes from v1.1

**Version 1.2 (January 2026):**

- **Updated Syntax**: The latest thinking is that Oxur will have `deffn` instead of `defun` or `defn` and that there will be no brackets, only parentheses.
- **Updated S-expression Example**: The example S-expression in this document has now been generated from Rust using the new `aster` and `oxurfmt` tools.
- **REPL Architecture Updated**: Subprocess execution now correctly documented as mandatory (not "single-process, no workers"). This is required because Rust threads cannot be interrupted - Ctrl-C support requires a killable subprocess.
- **IPC Model Clarified**: Uses stdin/stdout text protocol for subprocess communication. Removed incorrect "no IPC" claims. IPC overhead is minimal (~100-200μs) but does exist.
- **Artifact Caching Emphasized**: Now documented as mandatory day-one requirement, not optional optimization. Cache location and key generation specified.
- **Repository Structure Updated**: Added `oxur-smap` foundation crate for source mapping.
- **Dependency Graph Updated**: Shows `oxur-smap` as foundation that all other crates depend on.
- **Phase 6 Deliverables Updated**: Explicitly includes subprocess executor, artifact cache, and Ctrl-C handling.
- **Success Criteria Refined**: Tier-specific latency targets (Tier 1 <1ms, Tier 2 1-5ms, Tier 3 50-300ms).
- **Source Mapping Section Added**: Documents oxur-smap as unique differentiator for error message quality.

**Version 1.1 (December 2025):**

- Wire format strategy (postcard → MessagePack migration path)
- Network protocol details (nREPL-inspired, multi-transport)
- Transport abstraction architecture
- Phased macro system (core v1.0, user v2.0)
- 18-week development timeline with current progress
- Architectural improvements over Zylisp section
- Enhanced success criteria (REPL & protocol)

**Version 1.0 (December 2025):**

- Initial vision document