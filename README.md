# Oxur

[![Build Status][gh-actions-badge]][gh-actions]
[![Tags][github-tags-badge]][github-tags]

<a href="https://raw.githubusercontent.com/oxur/oxur/main/assets/images/logo/v2.3-1000x.png">
  <img src="https://raw.githubusercontent.com/oxur/oxur/main/assets/images/logo/v2.3-250x.png"
       alt="Our mascot, Orux! ('Ruxxy' to his friends)"
       title="Our mascot, Orux! ('Ruxxy' to his friends)">
</a>

*A Rust Lisp dialect with 100% interoperability*

## Overview

Oxur is a Lisp that treats Rust as its compilation target and runtime. Drawing inspiration from Zetalisp, LFE, and Clojure, Oxur provides Lisp's expressiveness and metaprogramming power while leveraging Rust's type system, ownership model, and ecosystem. Simply put, Oxur lets you write your Rust as Lisp.

## Project Status

**Early Development** - Currently in the design phase.

## Repository Structure

This is a Cargo workspace containing multiple related crates:

- **design/** - Design documentation and CLI tool for managing docs
- **oxur-ast/** - Rust AST ↔ S-expression representation
- **oxur-cli/** - CLI infrastructure and unified command-line tool
- **oxur-lang/** *(in progress)* - The Oxur Lisp dialect
- **oxur-repl/** *(in progress)* - REPL server/client

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Cargo (comes with Rust)

### Building

```bash
# Build all crates
cargo build

# or
make build

# Build specific crate
cargo build -p design

# Build with optimizations
cargo build --release
```

## CLI Tools

Oxur includes several command-line tools:

- **aster** - AST manipulation (Rust ↔ S-expression conversion)
- **oxd** - Design documentation manager
- **oxur** - Unified CLI tool (in progress)

### Building CLI Tools

```bash
# Build all CLIs
cargo build --release --bins

# Build specific CLI
cargo build --release --bin aster
cargo build --release --bin oxd
cargo build --release --bin oxur --features binary
```

### CLI Infrastructure

All CLI tools use the `oxur-cli` library for common utilities:

- File I/O helpers (stdin/stdout/file handling)
- Colored terminal output
- Progress tracking

See `crates/oxur-cli/docs/USAGE.md` for development guide.

## Design Documents

ODDs ("Oxur Design Documents"), like [Erlang EEPs](https://github.com/erlang/eep) and [Rust RFCs](https://github.com/rust-lang/rfcs), document all architectural decisions, specifications, and design discussions in the `crates/design/docs/` directory.

To explore Oxur's design decisions, you probably want to [start here](crates/design/docs/index.md).

### Design Documentation CLI

Current `oxd` help text:

```bash
./bin/oxd --help

Oxur Design Documentation Manager

Usage: oxd [OPTIONS] <COMMAND>

Commands:
  add            Add a new document with full processing
  add-batch      Add multiple documents (supports glob patterns)
  add-headers    Add or update YAML frontmatter headers [aliases: headers]
  debug          Debug and introspection commands
  help           Print this message or the help of the given subcommand(s)
  index          Generate the index file [aliases: gen-index]
  info           Show tool information and documentation
  list           List all design documents [aliases: ls]
  new            Create a new design document
  remove         Remove a document (moves to dustbin) [aliases: rm]
  rename         Rename a document file (preserves number)
  replace        Replace a document while preserving its ID
  scan           Scan filesystem and update document state [aliases: rescan]
  search         Search documents [aliases: grep]
  show           Show a specific document
  sync-location  Move document to directory matching its state header [aliases: sync]
  transition     Transition document to a new state [aliases: mv]
  update-index   Synchronize the index with documents on filesystem [aliases: sync-index]
  validate       Validate all documents [aliases: check]

Options:
  -d, --docs-dir <DOCS_DIR>  Path to docs directory (defaults to ./docs) [default: docs]
  -h, --help                 Print help

Use 'oxd <command> --help' for more information about a command.
```

List all design documents:

```bash
./bin/oxd list
```

[![oxd cli tool screenshot of list command][oxd-list-screenshot]][oxd-list-screenshot]

## Contributing

*(To be added)*

## License

Apache License, Version 2.0

Copyright © 2020-2026, Oxur Group

[//]: ---Named-Links---

[gh-actions-badge]: https://github.com/oxur/oxur/workflows/CI/badge.svg
[gh-actions]: https://github.com/oxur/oxur/actions
[github-tags]: https://github.com/oxur/oxur/tags
[github-tags-badge]: https://img.shields.io/github/tag/oxur/oxur.svg
[oxd-list-screenshot]: assets/images/screenshots/oxd-list.png
