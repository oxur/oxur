# Rust AI Guidelines

A modular collection of Rust best practices optimized for AI/LLM consumption when coding or reviewing Rust.

## Purpose

This repository provides comprehensive, example-driven Rust guidelines extracted from authoritative sources including the Rust API Guidelines, with a focus on making them easily consumable by AI coding assistants and developers.

## Guidelines Index

### Core Patterns
- [01 - Core Idioms](01-core-idioms.md) - Essential Rust idioms and conventions
- [02 - API Design](02-api-design.md) - Public API design guidelines
- [03 - Error Handling](03-error-handling.md) - Result, Option, error types
- [05 - Type Design](05-type-design.md) - Structs, enums, newtypes, generics
- [06 - Traits](06-traits.md) - Trait design, impl patterns, trait objects

### Code Organization
- [12 - Project Structure](12-project-structure.md) - Crate organization, modules, visibility
- [13 - Documentation](13-documentation.md) - Doc comments, examples, rustdoc

### Advanced Topics
- [11 - Anti-patterns](11-anti-patterns.md) - What NOT to do (critical for AI)

## Guideline Strength Indicators

Throughout these guidelines, we use strength indicators:

- **MUST** - Required for idiomatic Rust
- **SHOULD** - Strongly recommended unless there's a clear reason not to
- **CONSIDER** - Worth evaluating for your use case
- **AVOID** - Generally problematic, use only with clear justification

## Source Documents

These guidelines are synthesized from:
- Rust API Guidelines (primary source)
- Additional sources to be integrated: Rust Design Patterns, Clippy Documentation, The Rust Style Guide, and others

## Usage with AI

These documents are optimized for:
- Quick reference during code review
- Prompting AI coding assistants
- Learning idiomatic Rust patterns
- Avoiding common pitfalls

Each pattern includes concrete code examples and clear rationale.

## Contributing

See the original prompt document for contribution guidelines and synthesis methodology.
