# Rust AI Guidelines

AI-optimized Rust best practices synthesized from authoritative sources including the Pragmatic Rust Guidelines, Rust API Guidelines, and community standards.

## Overview

This collection provides modular, example-rich guidelines for writing idiomatic Rust. Each file is optimized for AI/LLM consumption with:

- **Clear strength indicators** (MUST, SHOULD, CONSIDER, AVOID)
- **Concrete code examples** for every pattern
- **Brief rationales** explaining the "why"
- **Cross-references** to related patterns

## Quick Start

For AI agents coding in Rust:
1. Read `01-core-idioms.md` for essential patterns
2. Consult relevant topic files as needed
3. Check `11-anti-patterns.md` for common mistakes

## Files

### Foundation
- **[01-core-idioms.md](01-core-idioms.md)** - Essential Rust idioms and naming conventions
- **[02-api-design.md](02-api-design.md)** - Public API design guidelines
- **[03-error-handling.md](03-error-handling.md)** - Result, Option, custom error types

### Type System
- **[04-ownership-borrowing.md](04-ownership-borrowing.md)** - Lifetime patterns, Send/Sync
- **[05-type-design.md](05-type-design.md)** - Structs, enums, newtypes, generics
- **[06-traits.md](06-traits.md)** - Trait design and implementation patterns

### Advanced Topics
- **[07-concurrency-async.md](07-concurrency-async.md)** - Async patterns, threading
- **[08-performance.md](08-performance.md)** - Allocation, optimization strategies
- **[09-unsafe-ffi.md](09-unsafe-ffi.md)** - Unsafe guidelines, FFI patterns
- **[10-macros.md](10-macros.md)** - Macro patterns (limited coverage)

### Critical Reading
- **[11-anti-patterns.md](11-anti-patterns.md)** - What NOT to do (essential for AI)
- **[12-project-structure.md](12-project-structure.md)** - Crate organization, features
- **[13-documentation.md](13-documentation.md)** - Doc comments, rustdoc

## Source Materials

Primary sources:
- **Pragmatic Rust Guidelines** (Microsoft) - Practical, production-focused
- **Rust API Guidelines** - Authoritative API design patterns
- **Clippy Documentation** - Common lints and fixes
- **Rust Style Guide** - Formatting and conventions

## Usage Notes

### For AI Agents
- All code examples are designed to be directly usable or clearly marked as pseudocode
- Strength indicators help prioritize which guidelines to follow strictly
- Anti-patterns section is critical for avoiding common AI-generated mistakes

### For Humans
- These guidelines build on existing standards (API Guidelines, Clippy)
- "MUST" guidelines should always hold; "SHOULD" allows flexibility
- Follow the spirit, not just the letter of each guideline

## Guideline Strength Levels

| Strength | Meaning | When to Deviate |
|----------|---------|-----------------|
| **MUST** | Always required | Only with strong architectural justification |
| **SHOULD** | Strong recommendation | When you have a good reason documented in code |
| **CONSIDER** | Suggested approach | Situational; evaluate trade-offs |
| **AVOID** | Anti-pattern | Don't do this unless no alternative exists |

## Contributing

Based on the Pragmatic Rust Guidelines model:
- Guidelines should positively affect safety, performance, or maintainability
- Majority of experienced Rust developers should agree
- Must be comprehensible to Rust novices (4+ weeks experience)
- Must be pragmatic and actually followable

## Meta Principles

### The Golden Rule
**Each guideline exists for a reason; it's the spirit that counts, not the letter.**

Before working around a guideline, understand:
- Why it exists
- What it tries to safeguard
- Whether following it would violate its underlying motivation

### Applicability
- Guidelines declared **must** should always hold
- Guidelines declared **should** indicate more flexibility
- Teams are free to adopt these as they see fit
- You occasionally might have good reasons to do things differently

## Version

Last updated: 2025-01-05
Based on Pragmatic Rust Guidelines as of 2025-11-05

## External Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Pragmatic Rust Guidelines](https://microsoft.github.io/rust-guidelines/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/)
- [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
