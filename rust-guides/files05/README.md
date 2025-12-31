# Rust AI Coding Guidelines

> A comprehensive reference for AI assistants and agents writing, refactoring, or auditing Rust code.

## Purpose

This document collection provides **idiomatic Rust patterns and anti-patterns** optimized for AI consumption. Each guideline includes:

- **Strength indicator**: MUST, SHOULD, CONSIDER, AVOID
- **Concrete code examples** (good and bad)
- **Brief rationale** explaining *why*
- **Cross-references** to related patterns and Clippy lints

## Document Index

| File | Description | Priority |
|------|-------------|----------|
| [01-core-idioms.md](01-core-idioms.md) | Essential Rust idioms every file should follow | 🔴 Critical |
| [02-api-design.md](02-api-design.md) | Public API design for libraries | 🔴 Critical |
| [03-error-handling.md](03-error-handling.md) | Result, Option, error types, propagation | 🔴 Critical |
| [04-ownership-borrowing.md](04-ownership-borrowing.md) | Lifetimes, borrowing strategies, borrow checker | 🔴 Critical |
| [05-type-design.md](05-type-design.md) | Structs, enums, newtypes, generics | 🟡 Important |
| [06-traits.md](06-traits.md) | Trait design, implementations, objects | 🟡 Important |
| [07-concurrency-async.md](07-concurrency-async.md) | Async patterns, Send/Sync, threading | 🟡 Important |
| [08-performance.md](08-performance.md) | Allocation, cloning, iterators, zero-cost | 🟡 Important |
| [09-unsafe-ffi.md](09-unsafe-ffi.md) | Unsafe guidelines, FFI patterns | 🟠 Specialized |
| [10-macros.md](10-macros.md) | Declarative and procedural macros | 🟠 Specialized |
| [11-anti-patterns.md](11-anti-patterns.md) | What NOT to do — critical for AI | 🔴 Critical |
| [12-project-structure.md](12-project-structure.md) | Crates, modules, visibility | 🟢 Reference |
| [13-documentation.md](13-documentation.md) | Doc comments, examples, rustdoc | 🟢 Reference |

## Quick Reference: Strength Indicators

| Indicator | Meaning | Action |
|-----------|---------|--------|
| **MUST** | Required for correctness or safety | Always follow |
| **SHOULD** | Strong recommendation, exceptions rare | Follow unless specific reason not to |
| **CONSIDER** | Good practice, context-dependent | Evaluate for your situation |
| **AVOID** | Anti-pattern, causes problems | Do not use unless exceptional circumstances |

## How to Use These Guidelines

### For Code Generation
1. Load relevant section(s) before generating code
2. Check `11-anti-patterns.md` to avoid common mistakes
3. Follow MUST/SHOULD guidelines strictly
4. Apply CONSIDER guidelines based on context

### For Code Review/Refactoring
1. Check generated code against anti-patterns first
2. Verify API design matches `02-api-design.md`
3. Ensure error handling follows `03-error-handling.md`

### For Planning
1. Use `12-project-structure.md` for crate organization
2. Reference `05-type-design.md` for data modeling decisions

## Sources

This reference synthesizes guidance from:

- [cheats.rs](https://cheats.rs/) — Comprehensive Rust cheat sheet
- *Rust Design Patterns* (rust-unofficial)
- *Rust API Guidelines* (rust-lang)
- *Clippy Documentation*
- *The Rust Style Guide*
- *Pragmatic Rust Guidelines*
- *Asynchronous Programming in Rust*
- *The Little Book of Rust Macros*

## Version

- **Generated**: 2024-12-30
- **Rust Edition**: 2021 (patterns compatible with 2024)
- **MSRV**: 1.70+ (most patterns), noted where newer features required

---

*This is a living document. Guidelines may be updated as Rust evolves.*
