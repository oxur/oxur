# S-Expression Test Data

This directory contains S-expression files organized for testing and documentation purposes.

## Directory Organization

### `examples/` - Learning and Documentation (by complexity)

Documented S-expression examples organized by complexity level for learning and reference.

- **`simple/`** - Basic examples for beginners
  - Single values, simple lists
  - Fundamental S-expression types
  - Good starting point for understanding the format

- **`intermediate/`** - Moderate complexity
  - Nested structures
  - Multiple AST node types combined
  - Real-world patterns

- **`complex/`** - Full complexity
  - Complete, realistic AST representations
  - Deeply nested structures
  - Stress tests and comprehensive examples

### `fixtures/` - Systematic Testing (by AST node type)

Test fixtures organized by the AST node type they represent, for systematic testing coverage.

- **`crate/`** - Crate-level structures
- **`item/`** - Item definitions (functions, structs, etc.)
- **`expr/`** - Expressions
- **`stmt/`** - Statements
- **`block/`** - Block structures

### `error-cases/` - Invalid S-expressions

Files that should fail to parse, used for testing error handling and error messages.

## Usage in Tests

### Using Examples

```rust
use oxur_ast::sexp::Parser;

fn parse_example(path: &str) -> SExp {
    Parser::parse_file(format!("test-data/examples/{}", path))
        .unwrap_or_else(|e| panic!("Failed to parse example {}: {}", path, e))
}

#[test]
fn test_with_example() {
    let sexp = parse_example("simple/empty-crate.sexp");
    // Test logic here
}
```

### Using Fixtures

```rust
fn parse_fixture(path: &str) -> SExp {
    Parser::parse_file(format!("test-data/fixtures/{}", path))
        .unwrap_or_else(|e| panic!("Failed to parse fixture {}: {}", path, e))
}

#[test]
fn test_crate_building() {
    let sexp = parse_fixture("crate/empty.sexp");
    let mut builder = AstBuilder::new();
    let crate_ast = builder.build_crate(&sexp).unwrap();
    assert_eq!(crate_ast.items.len(), 0);
}
```

## File Naming Conventions

### Examples (`examples/`)

- **simple/**: `empty-crate.sexp`, `nil-value.sexp`, `simple-fn.sexp`
  - Descriptive names indicating what they demonstrate
  - Focus on clarity for learning

- **intermediate/**: `fn-with-params.sexp`, `nested-blocks.sexp`
  - More specific about features demonstrated
  - May combine multiple concepts

- **complex/**: `full-crate.sexp`, `deeply-nested.sexp`
  - Describe the scenario or stress test
  - Represent real-world usage

### Fixtures (`fixtures/`)

- Short, descriptive names focused on what's being tested
- **crate/**: `empty.sexp`, `with-one-item.sexp`, `with-multiple-items.sexp`
- **item/**: `public-function.sexp`, `inherited-visibility.sexp`
- **expr/**: `macro-call.sexp`, `path-single.sexp`
- **stmt/**: `empty.sexp`, `semi.sexp`, `expr.sexp`
- **block/**: `empty.sexp`, `with-stmts.sexp`

### Error Cases (`error-cases/`)

- Describe what kind of error they should trigger
- Examples: `unterminated-list.sexp`, `unexpected-close.sexp`, `invalid-escape.sexp`

## File Format

All `.sexp` files follow these conventions:

1. **2-space indentation** for nested structures
2. **Comment header** (`;; `) explaining what the file represents
3. **Position information** included for completeness
4. **Consistent formatting** using `Printer::print()` output

Example:
```sexp
;; Empty crate - the minimal AST structure
;; Equivalent to an empty Rust file
(Crate
  :items ()
  :span (Span :lo 0 :hi 0))
```

## When to Use Examples vs Fixtures

**Use `examples/`** when:
- Writing documentation or tutorials
- Need a progression from simple → complex
- Want to demonstrate a concept clearly
- Examples can be referenced in README or docs

**Use `fixtures/`** when:
- Writing systematic unit tests
- Testing specific AST node types
- Need comprehensive coverage by type
- Testing edge cases for a particular node

## Adding New Files

### Adding an Example

1. Choose complexity level (simple/intermediate/complex)
2. Create file with descriptive name
3. Add comment header explaining what it demonstrates
4. Ensure it parses correctly: `cargo test --test validation`

### Adding a Fixture

1. Determine AST node type (crate/item/expr/stmt/block)
2. Create file in appropriate subdirectory
3. Name it based on what variation it tests
4. Add comment explaining the test case
5. Verify it parses: `cargo test --test validation`

### Adding an Error Case

1. Create file that should fail to parse
2. Name it based on the error it should trigger
3. Add comment explaining expected error
4. Verify it fails correctly: `cargo test --test validation`

## Validation

All example and fixture files are validated in CI to ensure they:
- Parse successfully (except error-cases/)
- Are properly formatted
- Have required comment headers

Run validation locally:
```bash
cargo test --test validation
```

## Contributing

When contributing new S-expression files:
1. Follow the naming conventions above
2. Include clear comment headers
3. Format using `Printer::print()` for consistency
4. Add corresponding tests if needed
5. Run validation before committing

## Questions?

- Not sure which directory? Examples = learning, Fixtures = testing
- Not sure which complexity? Start simple, move up if needed
- File format questions? Check existing files or `Printer::print()` output
