# Oxur

A Lisp dialect that compiles to Rust with 100% interoperability.

[![][logo]][logo-large]

## Overview

Oxur is a Lisp that treats Rust as its compilation target and runtime. Drawing inspiration from Zetalisp, LFE, and Clojure, Oxur provides Lisp's expressiveness and metaprogramming power while leveraging Rust's type system, ownership model, and ecosystem.

## Project Status

**Early Development** - Currently in the design phase.

## Repository Structure

This is a Cargo workspace containing multiple related crates:

- **design/** - Design documentation and CLI tool for managing docs
- **rast/** *(planned)* - Rust AST ↔ S-expression conversion
- **lang/** *(planned)* - Oxur compiler (Stage 1)
- **repl/** *(planned)* - REPL server/client
- **cli/** *(planned)* - User-facing CLI tool

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Cargo (comes with Rust)

### Building

```bash
# Build all crates
cargo build

# Build specific crate
cargo build -p design

# Build with optimizations
cargo build --release
```

### Design Documentation CLI

```bash
# List all design documents
cargo run -p design -- list

# Show a specific document
cargo run -p design -- show 0001

# Create a new design document
cargo run -p design -- new "Document Title"

# Validate all documents
cargo run -p design -- validate
```

## Design Documents

All architectural decisions, specifications, and design discussions are documented in the `design/docs/` directory. Start with [00-index.md](design/docs/00-index.md).

## Contributing

*(To be added)*

## License

Copyright © 2020, Oxur Group

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

<!-- Named page links below: /-->

[logo]: https://raw.githubusercontent.com/oxur/oxur/main/assets/images/logo/v1-tiny.jpg
[logo-large]: https://raw.githubusercontent.com/oxur/oxur/main/assets/images/logo/v1.jpg
