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

The build includes several command-line tools:

- **`aster`** - AST manipulation (Rust ↔ S-expression conversion)
- **`oxd`** - Design documentation manager

Which you can also build individually:

```bash
cargo build --release --bin aster
cargo build --release --bin oxd
```

## Oxur's AST

The Oxur AST is really just the Rust AST in S-expression format. Here's a simple example taken from `crates/oxur-ast/test-data/examples/intermediate/`:

```lisp
(Item
  :vis Public
  :kind (Fn
    :sig (FnSig
      :name "main"
      :params ()
      :return-type nil)
    :body (Block
      :stmts ()
      :span (Span :lo 0 :hi 0)))
  :span (Span :lo 0 :hi 0))
```

The `oxur-ast` crate includes the `aster` binary which may be used to convert between Oxur and Rust ASTs:

```shell
$ ./bin/aster --help
AST manipulation and conversion tool

Usage: aster <COMMAND>

Commands:
  to-ast   Convert Rust source to S-expression AST [aliases: ast]
  to-rust  Convert S-expression to Rust source
  verify   Verify round-trip conversion
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

Use 'aster <command> --help' for more information.
```

A full round-trip example that you can run yourself is provided in [the crate README](./crates/oxur-ast/README.md#end-to-end-examples).

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

## License

Apache License, Version 2.0

Copyright © 2020-2026, Oxur Group

[//]: ---Named-Links---

[gh-actions-badge]: https://github.com/oxur/oxur/workflows/CI/badge.svg
[gh-actions]: https://github.com/oxur/oxur/actions
[github-tags]: https://github.com/oxur/oxur/tags
[github-tags-badge]: https://img.shields.io/github/tag/oxur/oxur.svg
[oxd-list-screenshot]: assets/images/screenshots/oxd-list.png
