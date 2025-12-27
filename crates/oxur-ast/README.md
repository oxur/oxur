# oxur-ast

Rust AST representation and manipulation library using S-expressions.

## Overview

`oxur-ast` provides a comprehensive toolkit for working with Rust Abstract Syntax Trees (AST) using S-expression syntax. It includes:

- **S-expression parsing** - Convert S-expression strings and files into structured data
- **S-expression printing** - Format S-expressions with customizable indentation
- **File I/O** - Read and write S-expressions from/to files
- **AST building** - Transform S-expressions into Rust AST structures
- **Position tracking** - Maintain source location information throughout parsing

## Features

### S-expression Support

The library supports the following S-expression types:

- **Symbols**: `foo`, `bar`, `MyStruct`
- **Keywords**: `:name`, `:type`, `:value`
- **Strings**: `"hello"`, `"world\n"`
- **Numbers**: `42`, `-17`, `0`
- **Nil**: `nil`
- **Lists**: `(foo bar baz)`, `(:key value)`

### File I/O

Read and write S-expressions directly from/to files:

```rust
use oxur_ast::sexp::{Parser, write_sexp_file};

// Read from file
let sexp = Parser::parse_file("my-ast.sexp")?;

// Write to file
write_sexp_file(&sexp, "output.sexp")?;

// Round-trip
let original = Parser::parse_file("input.sexp")?;
write_sexp_file(&original, "output.sexp")?;
let reparsed = Parser::parse_file("output.sexp")?;
```

### String Parsing

Parse S-expressions from strings:

```rust
use oxur_ast::sexp::Parser;

let input = r#"(Crate :items ())"#;
let sexp = Parser::parse_str(input)?;
```

### Printing

Format S-expressions with customizable indentation:

```rust
use oxur_ast::sexp::{Printer, print_sexp};

// Default printer (2-space indentation)
let output = print_sexp(&sexp);

// Custom indentation
let printer = Printer::with_indent(4);
let output = printer.print(&sexp);
```

### AST Building

Convert S-expressions into Rust AST structures:

```rust
use oxur_ast::builder::AstBuilder;
use oxur_ast::sexp::Parser;

let input = r#"
(Crate
  :attrs ()
  :items ()
  :spans (ModSpans :inner-span (Span :lo 0 :hi 0))
  :id 0)
"#;

let sexp = Parser::parse_str(input)?;
let mut builder = AstBuilder::new();
let crate_ast = builder.build_crate(&sexp)?;
```

## Examples

The crate includes several examples demonstrating different features:

### Parse Example

Basic S-expression parsing from files and strings:

```bash
cargo run --example parse_example
```

### Build Simple Crate

Building Rust AST structures from S-expression files:

```bash
cargo run --example build_simple_crate
```

### File I/O

Comprehensive file I/O operations including reading, writing, and round-trip:

```bash
cargo run --example file_io
```

## Test Data Organization

The crate includes a comprehensive test data directory (`test-data/`) with:

### Examples (by complexity)

- **simple/**: Basic S-expressions (nil, numbers, symbols, keywords, strings, lists)
- **intermediate/**: Moderate complexity (functions, macro calls, paths, blocks)
- **complex/**: Advanced structures (full crates, deeply nested blocks)

### Fixtures (by AST node type)

- **crate/**: Crate structures
- **item/**: Item definitions (functions, etc.)
- **expr/**: Expression nodes
- **stmt/**: Statement nodes
- **block/**: Block expressions

### Error Cases

Test files that should fail to parse:

- **unterminated-list.sexp**: Missing closing parenthesis
- **unexpected-close.sexp**: Unexpected closing parenthesis
- **unterminated-string.sexp**: Missing closing quote
- **invalid-escape.sexp**: Invalid escape sequence

See [test-data/README.md](test-data/README.md) for detailed documentation.

## API Documentation

### Parser

```rust
use oxur_ast::sexp::Parser;

// Parse from string
let sexp = Parser::parse_str("(foo bar)")?;

// Parse from file
let sexp = Parser::parse_file("example.sexp")?;
```

### Printer

```rust
use oxur_ast::sexp::{Printer, print_sexp};

// Convenience function (2-space indentation)
let output = print_sexp(&sexp);

// Custom printer
let printer = Printer::with_indent(4);
let output = printer.print(&sexp);

// Write to file
printer.write_file(&sexp, "output.sexp")?;

// Convenience function for writing
write_sexp_file(&sexp, "output.sexp")?;
```

### AstBuilder

```rust
use oxur_ast::builder::AstBuilder;

let mut builder = AstBuilder::new();

// Build different AST nodes
let crate_ast = builder.build_crate(&sexp)?;
let item_ast = builder.build_item(&sexp)?;
let expr_ast = builder.build_expr(&sexp)?;
let stmt_ast = builder.build_stmt(&sexp)?;
let block_ast = builder.build_block(&sexp)?;
```

## Error Handling

The library provides detailed error types:

- `ParseError::EmptyInput`: Empty input provided
- `ParseError::UnterminatedList`: Missing closing parenthesis
- `ParseError::UnexpectedCloseParen`: Unexpected closing parenthesis
- `ParseError::LexError`: Lexical analysis errors (invalid escape, unterminated string)
- `ParseError::FileReadError`: Failed to read file
- `BuildError`: AST building errors with position information

## Testing

Run all tests:

```bash
cargo test
```

Run specific test suite:

```bash
cargo test --test parser_tests
cargo test --test builder_tests
cargo test --test test_data_validation
```

## License

See the [main repository](../../) for license information.

## Contributing

Contributions are welcome! Please see the [main repository](../../) for guidelines.
