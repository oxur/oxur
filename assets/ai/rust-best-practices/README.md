# Rust Best Practices Guidelines

> Comprehensive collection of Rust idioms, patterns, and best practices for writing idiomatic, safe, and performant code.

**Last Updated**: 2025-12-31
**Total Patterns**: 435 across 13 documents

---

## Document Index

This collection is organized into 13 focused areas covering all aspects of Rust development:

| Document | Pattern Count | Pattern ID Range | Description |
|----------|--------------|------------------|-------------|
| [01. Core Idioms](./01-core-idioms.md) | 42 | ID-01 to ID-42 | Fundamental Rust patterns and idioms |
| [02. API Design](./02-api-design.md) | 59 | API-01 to API-59 | Designing ergonomic and idiomatic APIs |
| [03. Error Handling](./03-error-handling.md) | 32 | EH-01 to EH-32 | Robust error handling strategies |
| [04. Ownership & Borrowing](./04-ownership-borrowing.md) | 19 | OB-01 to OB-19 | Mastering Rust's ownership system |
| [05. Type Design](./05-type-design.md) | 27 | TD-01 to TD-27 | Effective type system usage |
| [06. Traits](./06-traits.md) | 26 | TR-01 to TR-26 | Trait design and implementation |
| [07. Concurrency & Async](./07-concurrency-async.md) | 20 | CA-01 to CA-20 | Safe concurrent and asynchronous code |
| [08. Performance](./08-performance.md) | 22 | PF-01 to PF-22 | Writing high-performance Rust |
| [09. Unsafe & FFI](./09-unsafe-ffi.md) | 22 | US-01 to US-22 | Working with unsafe code and FFI |
| [10. Macros](./10-macros.md) | 20 | MC-01 to MC-20 | Effective macro design and usage |
| [11. Anti-patterns](./11-anti-patterns.md) | 80 | AP-01 to AP-80 | Common pitfalls and how to avoid them |
| [12. Project Structure](./12-project-structure.md) | 31 | PS-01 to PS-31 | Organizing Rust projects |
| [13. Documentation](./13-documentation.md) | 35 | DC-01 to DC-35 | Writing excellent documentation |

---

## Pattern Strength Indicators

Each pattern includes a strength indicator that guides how strictly it should be followed:

- **MUST**: Critical patterns that should always be followed (safety, correctness, or strong conventions)
- **SHOULD**: Strong recommendations for idiomatic code (best practices)
- **CONSIDER**: Useful patterns to consider based on context (trade-offs exist)
- **AVOID**: Anti-patterns that should be avoided (common mistakes)

---

## Pattern Format

Each pattern follows a consistent structure:

```markdown
## XX-NN: Pattern Name

**Strength**: MUST | SHOULD | CONSIDER | AVOID

**Summary**: One-line description of the pattern.

[Code examples demonstrating the pattern]

**Rationale**: Explanation of why this pattern matters.

**See also**: Cross-references to related patterns
```

---

## How to Use This Guide

### For New Rustaceans

Start with these foundational documents:

1. **01-core-idioms.md** - Learn fundamental Rust patterns
2. **04-ownership-borrowing.md** - Master Rust's unique ownership system
3. **03-error-handling.md** - Handle errors the Rust way
4. **11-anti-patterns.md** - Avoid common mistakes

### For Intermediate Developers

Focus on these areas to level up:

1. **02-api-design.md** - Design better libraries and interfaces
2. **05-type-design.md** - Leverage the type system effectively
3. **06-traits.md** - Master trait-based design
4. **08-performance.md** - Optimize your code

### For Advanced Practitioners

Explore specialized topics:

1. **07-concurrency-async.md** - Advanced async patterns
2. **09-unsafe-ffi.md** - Safe unsafe code and FFI bindings
3. **10-macros.md** - Metaprogramming with macros
4. **12-project-structure.md** - Architect large projects

### For All Developers

Essential reference materials:

1. **13-documentation.md** - Document your code effectively
2. **11-anti-patterns.md** - Comprehensive anti-pattern catalog (80 patterns!)

---

## Quick Stats

- **Total Patterns**: 435
- **Documents**: 13
- **Code Examples**: 479+ `rust` code blocks
- **Pattern Categories**:
  - Core Idioms & Patterns: 42
  - API Design: 59
  - Error Handling: 32
  - Type System: 46 (Type Design + Traits)
  - Concurrency: 20
  - Performance: 22
  - Advanced Topics: 42 (Unsafe/FFI + Macros)
  - Anti-patterns: 80
  - Project Organization: 66 (Structure + Documentation)

---

## Sources

This collection merges and consolidates patterns from multiple authoritative Rust resources:

- **Rust API Guidelines**: Official API design guidelines
- **Rust Design Patterns**: Community-maintained pattern catalog
- **Rust Performance Book**: Performance best practices
- **Clippy Lints**: Static analysis recommendations
- **Rust RFC discussions**: Language evolution insights
- **Production Rust codebases**: Real-world patterns

All patterns have been deduplicated, merged, and organized for maximum clarity and utility.

---

## Contributing

These guidelines represent current best practices as of 2025-12-31. The Rust ecosystem evolves rapidly:

- Patterns marked **MUST** are stable and unlikely to change
- Patterns marked **SHOULD** represent current consensus
- Patterns marked **CONSIDER** may have evolving trade-offs
- Patterns marked **AVOID** identify well-established anti-patterns

When Rust evolves (new language features, edition changes, ecosystem shifts), these patterns should be reviewed and updated accordingly.

---

## Cross-References

Patterns frequently reference each other across documents. Look for **See also** sections to explore related concepts. Common cross-cutting themes include:

- **Zero-cost abstractions**: TR-04, PF-02, PF-06
- **Memory safety**: OB-*, US-*, EH-*
- **API ergonomics**: API-*, TD-*, TR-*
- **Compile-time guarantees**: TD-*, TR-*, MC-*

---

## License

This is a compilation and synthesis of publicly available Rust best practices. Individual patterns may reference specific sources. Use freely for learning and reference.

---

*For the latest Rust language features and edition-specific guidance, always consult the official [Rust documentation](https://doc.rust-lang.org/).*
