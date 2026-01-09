# Oxur

[![Crates.io](https://img.shields.io/crates/v/oxur.svg)](https://crates.io/crates/oxur)
[![Documentation](https://docs.rs/oxur/badge.svg)](https://docs.rs/oxur)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/oxur/oxur#license)

**Oxur: A Lisp dialect that treats Rust as its compilation target and runtime**

This is the umbrella crate for the Oxur project, providing convenient access to all major components of the Oxur ecosystem.

## Overview

Oxur is a Lisp dialect designed to leverage Rust's type system, ownership model, and ecosystem while providing Lisp's expressiveness and metaprogramming capabilities.

**Key Philosophy:** Write Rust code using Lisp syntax with 100% bidirectional interoperability.

## Architecture

The Oxur compilation pipeline consists of five stages:

```
┌─────────────────────────────────────────────────────────────────┐
│                    OXUR COMPILATION PIPELINE                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Oxur Source (Lisp)                                             │
│         ↓                                                       │
│  ┌──────────────────┐                                           │
│  │  oxur-lang       │  Stage 1: Parse → Surface Forms           │
│  │  (Lisp Compiler) │  Stage 2: Expand → Core Forms (IR)        │
│  └──────────────────┘                                           │
│         ↓                                                       │
│  Core Forms (Canonical S-expressions)                           │
│         ↓                                                       │
│  ┌──────────────────┐                                           │
│  │  oxur-comp       │  Stage 3: Lower → Rust AST                │
│  │  (Backend)       │  Stage 4: Codegen → Rust Source           │
│  └──────────────────┘  Stage 5: Compile → Binary (via rustc)    │
│         ↓                                                       │
│  Rust Binary                                                    │
│                                                                 │
│  ┌──────────────────┐                                           │
│  │  oxur-ast        │  Supporting: Bidirectional Rust AST ↔     │
│  │  (AST Library)   │  S-expression conversion                  │
│  └──────────────────┘                                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Components

This umbrella crate re-exports the following components:

- **[oxur-lang](https://crates.io/crates/oxur-lang)**: The Oxur Lisp compiler (stages 1-2)
- **[oxur-comp](https://crates.io/crates/oxur-comp)**: The backend compiler (stages 3-5)
- **[oxur-ast](https://crates.io/crates/oxur-ast)**: Bidirectional Rust AST ↔ S-expression conversion
- **[oxur-repl](https://crates.io/crates/oxur-repl)**: Interactive REPL server/client
- **[oxur-cli](https://crates.io/crates/oxur-cli)**: Common CLI utilities and infrastructure
- **[oxur-pretty](https://crates.io/crates/oxur-pretty)**: Pretty printing for code and data structures

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oxur = "0.1"
```

## Usage

```rust
use oxur::*;

// API examples will be added as the project matures
```

## Design Principles

1. **100% Rust Interoperability** - Can call any Rust code, Rust can call any Oxur code
2. **Rust Semantics, Lisp Syntax** - Not Lisp semantics adapted to Rust
3. **Canonical S-expressions** - Single authoritative format for AST representation
4. **Round-trip Preservation** - X → transform → X must preserve meaning
5. **Type-First Design** - Leverage Rust's type system fully

## Project Status

Oxur is in active development. Different components are at various stages:

- **oxur-ast**: ~80% complete, functional conversion
- **oxur-lang**: Planning stage
- **oxur-comp**: Planning stage
- **oxur-repl**: Planning stage
- **oxur-cli**: Early stage, utilities available

## Resources

- **Repository**: [github.com/oxur/oxur](https://github.com/oxur/oxur)
- **Documentation**: [docs.rs/oxur](https://docs.rs/oxur)
- **Design Documents**: [Design Docs](https://github.com/oxur/oxur/tree/main/crates/oxur-odm/docs)

## Contributing

Contributions are welcome! Please read our [Contributing Guide](../../CONTRIBUTING.md) for details.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
