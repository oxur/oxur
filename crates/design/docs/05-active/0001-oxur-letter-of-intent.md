---
number: 1
title: "Oxur: A Letter of Intent"
author: "Duncan McGreggor"
created: 2025-12-25
updated: 2025-12-26
state: Active
supersedes: null
superseded-by: null
---

# Oxur: A Letter of Intent

**Status**: Vision & Design Exploration  
**Date**: December 2025  
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

**Stage 1**: The Oxur Compiler (`oxur/lang`)
- Parses Oxur syntax
- Expands macros
- Type checking and inference (optional)
- Compiles to canonical S-expressions (Core Forms)

**Stage 2**: The Rust AST Layer (`oxur/rast`)
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
- All `token::Pos` information preserved (for error messages)
- Keyword arguments for clarity
- Bidirectional transformation guaranteed

Example:
```lisp
(FuncDecl
  :doc nil
  :name (Ident :name "add")
  :type (FuncType
          :params (FieldList
                    :list ((Param :name "a" :ty (Path :segments ["i32"]))
                           (Param :name "b" :ty (Path :segments ["i32"]))))
          :return (Path :segments ["i32"]))
  :body (Block ...))
```

This becomes Rust:
```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### No Plugin Memory Leak Problem!

Unlike Go, Rust doesn't have the plugin memory leak that forced Zylisp into a complex supervised worker architecture. This simplifies our REPL design significantly:
- No need for disposable worker processes
- No IPC overhead
- No memory monitoring complexity
- Simpler supervision model

We still want reliability-first design, but we can achieve it more elegantly in Rust.

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
(defn foo ['a] ((x (& 'a str))) (& 'a str)
  x)

;; Struct with lifetime
(defstruct Holder ['a]
  :fields ((data (& 'a str))))

;; Lifetime bounds in trait implementations
(impl (Display) for (Holder ['a])
  (defn fmt ((self) (f (& mut Formatter))) Result
    ...))
```

The `'a` notation mirrors Rust directly - familiar to Rust programmers, not too foreign to Lispers.

### The Creative Naming Challenge

For each Rust feature, we ask: "If this was in Zetalisp or LFE, what would it be called?"

Some ideas to explore:
- `defn` vs `deffunc` vs `fn` for function definition
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
  (defn fmt ((self) (f (& mut Formatter))) Result))

;; Trait with associated type
(deftrait Iterator
  :associated ((Item type))
  (defn next ((self (& mut Self))) (Option Item)))

;; Generic function with trait bounds
(defn print-all [T] ((items (Vec T)))
  :where ((T Display))
  (for item items
    (println "{}" item)))

;; Trait implementation
(impl (Display) for Point
  (defn fmt ((self) (f (& mut Formatter))) Result
    (write f "({}, {})" self.x self.y)))
```

The `:where` keyword for bounds feels very Zetalisp. The `[T]` notation for generics is borrowed from Clojure but adapted for Rust's conventions.

## Type System Integration

Rust's type system is rich. Oxur needs to expose it without overwhelming:

### Explicit Type Annotations (Optional)

```lisp
;; Simple types
(let ((x i32 42)
      (name String "Alice")))

;; Function signatures
(defn add ((a i32) (b i32)) i32
  (+ a b))

;; Generic types
(defn first [T] ((vec (Vec T))) (Option T)
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
(defn sum-array [const N] ((arr [i32; N])) i32
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

## Module System

Rust's module system is explicit and hierarchical:

```lisp
;; Module declaration
(mod geometry
  (defstruct Point
    :fields ((x i32) (y i32)))
  
  (defn distance ((p1 Point) (p2 Point)) f64
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
(pub defn create-point ...)

;; Crate-public
(pub-crate defn internal-helper ...)

;; Module-private (default)
(defn private-impl ...)
```

## Error Handling: Result and Option

Rust's error handling is explicit and built around `Result` and `Option`:

```lisp
;; Result type
(defn divide ((a i32) (b i32)) (Result i32 String)
  (if (= b 0)
    (Err "division by zero")
    (Ok (/ a b))))

;; Question mark operator
(defn compute () (Result i32 Error)
  (let ((x (foo?))        ; equivalent to foo()?
        (y (bar?)))
    (Ok (+ x y))))

;; Option type
(defn find ((items (Vec String)) (target String)) (Option usize)
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

(defn main ()
  (let ((handle (thread::spawn
                  (fn () 
                    (println "Hello from thread!")))))
    (join handle)))

;; Channels
(use std::sync::mpsc)

(defn main ()
  (let (((tx rx) (mpsc::channel)))
    (thread::spawn
      (move (fn ()
        (send tx "Hello!"))))
    (println "Received: {}" (recv rx))))
```

The `move` keyword is crucial for closures that capture their environment.

## The REPL: Simpler than Zylisp

Without Go's plugin memory leak, our REPL can be simpler:

### Tiered Execution (Kept from Zylisp)

The three-tier strategy still makes sense:

**Tier 1: Direct Interpretation** (~1ms)
- Simple expressions: literals, arithmetic, variable lookups
- No compilation needed

**Tier 2: Cached Compilation** (~0ms after first compile)
- Function definitions compile once
- Subsequent calls are instant
- But no worker process complexity!

**Tier 3: JIT Compilation** (slower first time)
- Complex expressions
- Compile to Rust, then to native code
- Cache the result

### REPL Architecture

```
┌─────────────────────────────────────┐
│         REPL Server                  │
│  (Single process, no workers!)      │
│                                      │
│  ┌──────────────────────────────┐  │
│  │  Tier 1: Interpreter         │  │
│  └──────────────────────────────┘  │
│  ┌──────────────────────────────┐  │
│  │  Tier 2: Compiled Cache      │  │
│  └──────────────────────────────┘  │
│  ┌──────────────────────────────┐  │
│  │  Tier 3: JIT Compiler        │  │
│  └──────────────────────────────┘  │
└─────────────────────────────────────┘
```

Much simpler! No IPC, no memory monitoring, no worker restarts.

### Supervision: OTP-Style (Kept from Zylisp)

We still want reliability-first design, but adapted for Rust:

```rust
// Supervisor in Rust (using rely-rs or similar)
let supervisor = Supervisor::new(OneForOne);

supervisor.add_child(ChildSpec {
    id: "repl-server",
    start: start_repl_server,
    restart: Permanent,
});

supervisor.start();
```

But this is optional - Rust's safety means crashes are rarer.

## Repository Structure

Following Zylisp's successful pattern:

```
github.com/oxur/
├── rast/           # Rust AST ↔ S-expr conversion (Stage 2)
├── lang/           # Oxur compiler (Stage 1)
├── repl/           # REPL server/client
├── cli/            # CLI tool
├── rely-rs/        # Supervision library (Rust port of rely)
├── design/         # This document and all design docs
└── rust-ast-coverage/  # Comprehensive Rust test cases
```

**Dependency graph** (no circles!):
```
cli → repl → lang → rast
      ↓
    rely-rs
```

Clean, testable, maintainable.

## Development Priorities

### Phase 0: Foundation

1. **Define Core Forms specification**
   - Canonical S-expression format for Rust AST
   - Document every Rust AST node type
   - Establish conventions

2. **Build `rast`**
   - S-expression parser
   - Rust AST → S-expr generator
   - S-expr → Rust AST builder
   - Comprehensive tests against `rust-ast-coverage`

3. **Validate round-trip**
   - Rust source → AST → S-expr → AST → Rust source
   - Bootstrap demo like Zylisp's

### Phase 1: Minimal Viable Oxur

4. **Basic Oxur syntax**
   - Functions, variables, basic types
   - Simple expressions
   - No macros yet

5. **Stage 1 compiler**
   - Parse Oxur syntax
   - Compile to Core Forms
   - Integrate with `rast`

6. **Hello World**
   - Write Oxur code that compiles to working Rust
   - Prove the pipeline works

### Phase 2: Essential Features

7. **Pattern matching**
   - Full `match` support
   - Destructuring
   - Guards

8. **Ownership primitives**
   - Borrow, borrow-mut, move
   - Basic lifetime annotations

9. **Traits (basic)**
   - Trait definitions
   - Trait implementations
   - Simple bounds

### Phase 3: Power Features

10. **Macros**
    - Macro definition and expansion
    - Hygiene and gensym
    - Macro debugging

11. **Advanced types**
    - Generics with bounds
    - Associated types
    - Const generics

12. **REPL**
    - Three-tier execution
    - Session management
    - Integration testing

### Phase 4: Production Ready

13. **Complete Rust coverage**
    - All control flow
    - All type constructs
    - All trait features
    - Unsafe blocks (carefully!)

14. **Tooling**
    - LSP server
    - Formatter
    - Linter
    - Documentation generator

15. **Standard library**
    - Idiomatic wrappers for Rust std
    - Lisp-friendly APIs
    - Community-driven growth

## Key Design Questions to Resolve

As we work through this, we'll need to decide:

### Naming Conventions

- Function definition: `defn`, `deffunc`, `fn`, `define`?
- Struct definition: `defstruct`, `struct`, `record`?
- Trait definition: `deftrait`, `trait`, `protocol`?
- Variable binding: `let`, `bind`, `var`?
- Match arms: keep `match`? Use `case`?

We'll explore each with the question: "What would Zetalisp or LFE choose?"

### Syntax for Rust-Specific Features

- Lifetime syntax: `'a`, `<'a>`, `#'a`, something else?
- Reference syntax: `(& x)`, `(ref x)`, `#&x`?
- Mutable reference: `(&mut x)`, `(ref-mut x)`, `#&mut x`?
- Generic type parameters: `[T]`, `<T>`, `{T}`?
- Where clauses: `:where`, `:bounds`, `:constraints`?

### Type Annotation Philosophy

- Required in function signatures? (Leaning yes)
- Required in let bindings? (Leaning no, infer when possible)
- Required in struct fields? (Leaning yes)
- How verbose for complex types?

### Macro Integration

- Can Oxur macros generate Rust macros?
- Can Oxur code invoke Rust macros?
- How do we handle procedural macros?
- Derive macros for Oxur structs?

### Error Handling

- Special syntax for `?` operator?
- Syntactic sugar for Result/Option handling?
- Debugging tools for error propagation?

### Module and Visibility

- Follow Rust's module system exactly?
- Any syntactic sugar for common patterns?
- How to handle `pub(crate)`, `pub(super)`, etc.?

## Success Criteria

We'll know Oxur is succeeding when:

1. **Round-trip works**: Rust → S-expr → Rust produces equivalent code
2. **Hello World compiles**: Basic Oxur programs generate working binaries
3. **FFI is seamless**: Calling Rust from Oxur feels natural
4. **Ownership feels right**: Borrowing and lifetimes aren't fighting the language
5. **Patterns are beautiful**: Match expressions are clean and powerful
6. **Traits are accessible**: The trait system is approachable
7. **Macros are powerful**: Compile-time metaprogramming works as expected
8. **REPL is fast**: Sub-millisecond responses for simple expressions
9. **Rust community approves**: Rustaceans see Oxur as idiomatic
10. **Lisp community approves**: Lispers feel at home

## Why This Matters

**For Rust Developers:**
- Powerful metaprogramming capabilities
- Alternative syntax for Rust's semantics
- Rapid prototyping with REPL
- Code generation and analysis tools

**For Lisp Enthusiasts:**
- Modern, safe, fast language with Lisp manipulation
- Access to Rust's amazing ecosystem
- Zero-cost abstractions with Lisp expressiveness
- Pattern matching as a first-class citizen

**For Everyone:**
- Exploring language design boundaries
- Bridging paradigms thoughtfully
- Learning through alternative representations
- Having fun with powerful tools

## Closing Thoughts

Oxur sits at the intersection of three powerful traditions:
- **Lisp's elegance**: Code as data, metaprogramming, REPL-driven development
- **Rust's safety**: Ownership, lifetimes, fearless concurrency
- **Zetalisp's beauty**: Clean design, keyword arguments, orthogonality

We're not making Rust into Lisp or Lisp into Rust. We're revealing that Rust *already has* a Lisp hiding inside it - we're just giving it S-expression syntax and the full power of homoiconicity.

This is going to be a phenomenal journey. We have the benefit of Zylisp's lessons, Rust's superior design, and a clear architectural vision. The hard problems are obvious (lifetimes, ownership, traits), but they're *interesting* hard problems, not insurmountable ones.

Let's build something beautiful.

**Onward! 🦀✨**

---

*"In Lisp, code is data. In Rust, safety is fearless. In Oxur, we get both."*