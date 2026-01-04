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
- **oxur-ast/** - *(in progress)* - Rust AST ↔ S-expression representation and CLI tool for generating S-expressions and Rust code
- **oxur-cli/** - *(early stages)* - CLI infrastructure and unified command-line tool
- **oxur-pretty/** - S-expression formatter with rustfmt-style CLI
- **oxur-lang/** - *(planning)* - The Oxur Lisp dialect
- **oxur-repl/** - *(planning)* - REPL server/client
- **oxur-comp/** - *(planning)* - The Oxur compiler

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

- **`aster`** - AST manipulation (Rust ↔ S-expression conversions)
- **`oxd`** - Design documentation manager
- **`oxur-fmt`** - S-expression formatter

Which you can also build individually:

```bash
cargo build --release --bin aster
cargo build --release --bin oxd
cargo build --release --bin oxur-fmt
```

## Oxur Syntax

The Oxur syntax is currently in the "actively researching, exploring, and experimenting" and  stage. As part of that, we are examining the following:
- [Coalton](https://github.com/coalton-lang/coalton) - Hindley-Milner type inference for Common Lisp
- [LFE](https://lfe.io) - LFE's syntax for Erlang type specs (these are type specifications that can by used by Erlang's static analyser `dialyzer`; they are not static types)
- [Typed Racket](https://docs.racket-lang.org/ts-guide/)
- [Shen](https://shen-language.github.io/)

Given the Zetalisp inspiration for Oxur, we are leaning heavily toward a Coalton-influenced syntax for types in a Rust Lisp.

Now, that being said, we're going to go out on a limb and show some of what we're thinking, even though we can't make any promise that any of this will survive the process of experimentation :-D Here's an example of the sort of sytnax we're exploring:

```clj
(use std::collections::hashmap)
(use std::io::error)

(deffn greet-user
  (name:string count:(option u32)) -> (result string error)
  "Greets a user with optional repetition"
  (let (base (string::from "Hello, ")            ; static String::from
        greeting (string:push-str base name)     ; instance method
        final (match count
                ((some n) (string:repeat greeting n))
                (none) greeting))
    (ok final)))
```

If you're curious about the development of Oxur's syntax, you're going to want to keep your eye on [this doc]() as it moves through the design process.

## The Oxur AST

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

## S-Expression Formatting

The `oxur-pretty` crate provides the `oxur-fmt` tool for formatting S-expression files with human-readable output. It follows rustfmt conventions for a familiar workflow:

```bash
$ ./bin/oxur-fmt --help
Format S-expression files

Usage: oxur-fmt [OPTIONS] <FILE>...

Arguments:
  <FILE>...  Input files to format (use '-' for stdin)

Options:
      --check                      Check if files are formatted correctly
      --emit <MODE>                What data to emit and how [possible values: files, stdout]
      --backup                     Backup any modified files
      --config <key1=val1,key2=val2>  Set options from command line
      --color <MODE>               Use colored output [possible values: always, never, auto]
  -l, --files-with-diff            Prints names of mismatched files
  -v, --verbose                    Print verbose output
  -q, --quiet                      Print less output
  -h, --help                       Print help
```

Format files in-place, check formatting for CI/CD, or pipe through stdin/stdout:

```bash
# Format file in-place
./bin/oxur-fmt my-ast.sexp

# Check if formatted (exits 1 if not)
./bin/oxur-fmt --check src/*.sexp

# Stdin to stdout
echo "(Span :lo 0 :hi 10)" | ./bin/oxur-fmt
```

See [the crate README](./crates/oxur-pretty/README.md) for detailed usage and configuration options.

## The Oxur Compiler

Parts of the compile change have been explored in the AST work above (particularly with the `aster` tool). Other parts will be explored in early REPL work. For the latest thinking on our approach, see:

- [Oxur Compilation Chain Architecture](crates/design/docs/05-active/0013-oxur-compilation-chain-architecture.md).

## The Oxur REPL

The following design docs show our current thinking with regard to separation of concerns, extensibility (protocol, clients, servers, etc.):

- [Research: Building a transport-agnostic REPL protocol in Rust](crates/design/docs/06-final/0016-building-a-transport-agnostic-repl-protocol-in-rust.md)
- [Oxur Remote REPL Protocol Design](crates/design/docs/02-under-review/0018-oxur-remote-repl-protocol-design.md)
- [Recommendations for Future-proofing Multiple REPL Protocols](crates/design/docs/02-under-review/0017-recommendations-for-future-proofing-multiple-repl-protocols.md)

Exact mechanics have yet to be ironed out.

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
