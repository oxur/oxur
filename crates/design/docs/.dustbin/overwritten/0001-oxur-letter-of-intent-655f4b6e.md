---
number: 1
title: "Oxur: A Letter of Intent"
author: "Duncan McGreggor"
created: 2025-12-25
updated: 2025-12-26
state: Overwritten
supersedes: null
superseded-by: null
---

# Oxur: A Letter of Intent

## Overview

Oxur is a Lisp dialect designed to compile to Rust with 100% interoperability. It treats Rust as its compilation target and runtime, providing Lisp's expressiveness and metaprogramming power while leveraging Rust's type system, ownership model, and ecosystem.

## Background

The Lisp family of languages offers unparalleled expressiveness through homoiconicity, powerful macro systems, and a philosophy of minimal syntax. Meanwhile, Rust provides modern systems programming capabilities with memory safety guarantees, zero-cost abstractions, and excellent performance.

Oxur aims to bridge these two worlds, drawing inspiration from:

- **Zetalisp** - The original Lisp Machine dialect, known for its rich development environment
- **LFE (Lisp Flavoured Erlang)** - A Lisp that targets the BEAM VM, demonstrating how to build a Lisp on an existing platform
- **Clojure** - A modern Lisp that emphasizes immutability and functional programming

## Proposal

### Core Philosophy

1. **Rust is the Foundation** - Oxur is not an interpreted Lisp that happens to be written in Rust. It compiles to Rust source code that then compiles normally.

2. **100% Interop** - Any Rust code should be callable from Oxur, and any Oxur code should be callable from Rust.

3. **Gradual Adoption** - Developers can mix Oxur and Rust in the same project, adopting Oxur incrementally.

4. **Macro Power** - Full access to Rust's type system and capabilities through Lisp's macro system.

### Project Structure

The project is organized as a Cargo workspace with multiple crates:

- **design** - Design documentation and management tools
- **rast** - Rust AST ↔ S-expression conversion (bidirectional)
- **lang** - The Oxur compiler (Stage 1)
- **repl** - REPL server and client
- **cli** - User-facing command line tool

### Development Phases

**Phase 1: Foundation**
- Implement rast for Rust AST manipulation
- Build basic S-expression parser
- Create initial compiler pipeline

**Phase 2: Core Language**
- Define Oxur syntax and semantics
- Implement standard forms (def, fn, let, if, etc.)
- Add Rust type annotations

**Phase 3: Interop**
- Full Rust FFI
- Crate dependency handling
- Mixed Rust/Oxur compilation

**Phase 4: Productivity**
- REPL development
- IDE integration
- Documentation and tooling

## Alternatives Considered

### Transpile to Rust AST Only

Using `syn` to generate Rust code directly rather than going through an S-expression intermediate representation.

**Rejected because:** Having a canonical S-expression representation enables powerful metaprogramming and makes the compiler easier to understand and extend.

### Target LLVM Directly

Compile Oxur directly to LLVM IR, bypassing Rust entirely.

**Rejected because:** This loses all benefits of Rust's type system, borrow checker, and ecosystem. The goal is integration, not replacement.

### Build on MIR

Work at Rust's Mid-level Intermediate Representation instead of source code.

**Rejected because:** MIR is an internal implementation detail and not stable. Source-to-source compilation is more maintainable long-term.

## Implementation Plan

1. Set up workspace structure (this document)
2. Design and implement rast crate
3. Define Oxur core syntax
4. Build initial compiler (def, fn, basic types)
5. Add REPL capabilities
6. Iterate on language features

## Open Questions

1. **Ownership Syntax** - How should Oxur express Rust's ownership and borrowing concepts?
2. **Lifetime Annotations** - Should lifetimes be explicit, inferred, or use a different notation?
3. **Trait System** - How to integrate Rust traits with Lisp-style polymorphism?
4. **Error Handling** - Result types vs exceptions vs a hybrid approach?

## Success Criteria

- Oxur code can define and call Rust functions
- Rust code can call Oxur-defined functions
- A simple but complete program can be written in Oxur
- The REPL provides interactive development
- Documentation is comprehensive and accessible