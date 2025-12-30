# oxur-ast

Rust AST ↔ S-expression bidirectional conversion library with CLI tool.

## Overview

`oxur-ast` provides a comprehensive toolkit for working with Rust Abstract Syntax Trees (AST) using S-expression syntax. It includes:

- **Rust parsing** - Parse real Rust source code into AST structures via `syn`
- **S-expression parsing** - Convert S-expression strings and files into structured data
- **S-expression printing** - Format S-expressions with customizable indentation
- **File I/O** - Read and write S-expressions from/to files
- **AST building** - Transform S-expressions into Rust AST structures
- **Position tracking** - Maintain source location information throughout parsing
- **CLI tool** - `aster` command-line tool for conversions and verification
- **Round-trip verification** - Ensure conversion integrity

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

### Rust Parsing Integration

Parse real Rust source code into AST structures:

```rust
use oxur_ast::integration::parse_rust_file;
use oxur_ast::Generator;
use oxur_ast::sexp::print_sexp;

// Parse Rust source
let source = r#"
fn main() {
    println!("Hello, world!");
}
"#;

let crate_node = parse_rust_file(source)?;

// Generate S-expression
let gen = Generator::new();
let sexp = gen.generate_crate(&crate_node)?;

// Print formatted output
println!("{}", print_sexp(&sexp));
```

## CLI Tool: `aster`

The `aster` command-line tool provides convenient commands for AST conversions and verification.

### Installation

```bash
cargo install --path .
```

### Commands

**Convert Rust to S-expression:**

```bash
# Output to stdout
aster to-ast hello.rs

# Save to file
aster to-ast hello.rs -o hello.sexp

# Compact format
aster to-ast hello.rs --compact
```

**Convert S-expression to Rust:**

```bash
# Output to stdout
aster to-rust hello.sexp

# Save to file
aster to-rust hello.sexp -o output.rs
```

**Verify round-trip conversion:**

```bash
# Basic verification
aster verify hello.rs

# Verbose output
aster verify hello.rs --verbose
```

### End-to-End Examples

#### Example 1: S-expression → Rust → Compiled Binary

This example demonstrates creating a Rust program from an S-expression representation, compiling it, and running it.

**Step 1:** Create project structure

```bash
mkdir -p /tmp/oxur-hw1/src
```

**Step 2:** Create `Cargo.toml`

```bash
cat > /tmp/oxur-hw1/Cargo.toml <<'EOF'
[package]
name = "oxur-hw1"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "oxur-hw1"
path = "src/main.rs"
EOF
```

**Step 3:** Create `src/main.sexp` (hello world in S-expression format)

```bash
cat > /tmp/oxur-hw1/src/main.sexp <<'EOF'
(Crate
  :attrs ()
  :items ((Item
             :attrs ()
             :id 2
             :span (Span :lo 0 :hi 0)
             :vis (Inherited)
             :ident (Ident :name "main" :span (Span :lo 0 :hi 0))
             :kind (Fn
                     (Fn
                       :defaultness Final
                       :sig (FnSig
                              :header (FnHeader
                                        :safety Default
                                        :constness NotConst
                                        :ext None
                                        :coroutine-kind nil)
                              :decl (FnDecl
                                      :inputs ()
                                      :output (Default (Span :lo 0 :hi 0)))
                              :span (Span :lo 0 :hi 0))
                       :generics (Generics
                                   :params ()
                                   :where-clause (WhereClause
                                                   :has-where-token false
                                                   :predicates ()
                                                   :span (Span :lo 0 :hi 0))
                                   :span (Span :lo 0 :hi 0))
                       :body (Block
                               :stmts ((Stmt
                                         :id 0
                                         :kind (MacCall
                                                 (MacCallStmt
                                                   :mac (MacCall
                                                          :path (Path
                                                                  :span (Span :lo 0 :hi 0)
                                                                  :segments ((PathSegment
                                                                               :ident (Ident :name "println" :span (Span :lo 0 :hi 0))
                                                                               :id 4294967295
                                                                               :args nil)))
                                                          :args (Delimited
                                                                  :dspan (DelSpan
                                                                           :open (Span :lo 0 :hi 0)
                                                                           :close (Span :lo 0 :hi 0))
                                                                  :delim Paren
                                                                  :tokens (TokenStream :source "\"Hello from Oxur (via S-expression)!\""))
                                                          :prior-type-ascription nil)
                                                   :style Semicolon
                                                   :attrs ()))
                                         :span (Span :lo 0 :hi 0)))
                               :id 1
                               :rules Default
                               :span (Span :lo 0 :hi 0)
                               :could-be-bare-literal false)))))
  :spans (ModSpans
           :inner-span (Span :lo 0 :hi 0)
           :inject-use-span (Span :lo 0 :hi 0))
  :id 3
  :is-placeholder false)
EOF
```

**Step 4:** Convert S-expression to Rust

```bash
cd /tmp/oxur-hw1
aster to-rust src/main.sexp -o src/main.rs
```

**Step 5:** Compile the program

```bash
cargo build
```

**Step 6:** Run the compiled binary

```bash
./target/debug/oxur-hw1
```

Output:
```
Hello from Oxur (via S-expression)!
```

---

#### Example 2: Rust → S-expression → Comparison

This example demonstrates converting Rust code to S-expression format and comparing it with the previous example.

**Step 1:** Create project structure

```bash
mkdir -p /tmp/oxur-hw2/src
```

**Step 2:** Create `Cargo.toml`

```bash
cat > /tmp/oxur-hw2/Cargo.toml <<'EOF'
[package]
name = "oxur-hw2"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "oxur-hw2"
path = "src/main.rs"
EOF
```

**Step 3:** Create `src/main.rs` (hello world in Rust)

```bash
cat > /tmp/oxur-hw2/src/main.rs <<'EOF'
fn main() {
    println!("Hello from Oxur (via Rust)!");
}
EOF
```

**Step 4:** Convert Rust to S-expression

```bash
cd /tmp/oxur-hw2
aster to-ast src/main.rs -o src/main.sexp
```

**Step 5:** Compare the two S-expression representations

```bash
diff -u /tmp/oxur-hw1/src/main.sexp /tmp/oxur-hw2/src/main.sexp
```

The diff shows the minimal differences - primarily just the string literal content:

```diff
--- /tmp/oxur-hw1/src/main.sexp
+++ /tmp/oxur-hw2/src/main.sexp
@@ -xx,x +xx,x @@
-                                                                  :tokens (TokenStream :source "\"Hello from Oxur (via S-expression)!\""))
+                                                                  :tokens (TokenStream :source "\"Hello from Oxur (via Rust)!\""))
```

This demonstrates that:
1. **Bidirectional conversion works** - Rust ↔ S-expression conversions are equivalent
2. **Round-trip integrity** - Converting back and forth preserves AST structure
3. **Semantic equivalence** - The only differences are in the actual content (string literals), not the structure

### Architecture

```
Rust Source → syn → oxur AST → Generator → S-expression
                      ↑                          ↓
                      └────── Builder ← Parser ──┘
```

## Examples

The crate includes several examples demonstrating different features:

### Parse Rust File

Parse a Rust source file and display AST information:

```bash
cargo run --example parse_rust_file tests/fixtures/simple_fn.rs
```

### Convert File

Convert a Rust source file to S-expression format:

```bash
cargo run --example convert_file tests/fixtures/hello_world.rs /tmp/hello.sexp
```

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
cargo test --test integration_tests
```

## Performance Benchmarks

Run performance benchmarks:

```bash
cargo bench
```

The benchmark suite includes:

- **parse_rust**: Parsing Rust source code via syn
- **generate_sexp**: Generating S-expressions from AST
- **parse_sexp**: Parsing S-expression text
- **build_ast**: Building AST from S-expressions
- **round_trip**: Full round-trip conversion (Rust → S-expr → AST)

## License

See the [main repository](../../) for license information.

## Contributing

Contributions are welcome! Please see the [main repository](../../) for guidelines.
