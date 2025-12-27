# Oxur

A Lisp dialect that compiles to Rust with 100% interoperability.

<a href="https://raw.githubusercontent.com/oxur/oxur/main/assets/images/logo/v2.3-1000x.png">
  <img src="https://raw.githubusercontent.com/oxur/oxur/main/assets/images/logo/v2.3-250x.png"
       alt="Our mascot, Orux! ('Ruxxy' to his friends)"
       title="Our mascot, Orux! ('Ruxxy' to his friends)">
</a>

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

Apache License, Version 2.0

Copyright © 2020, Oxur Group
