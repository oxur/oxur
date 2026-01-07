# Oxur Language Test Data

This directory contains test data for the Oxur language, organized by complexity and category.

## Directory Structure

```
test-data/
├── README.md              # This file
├── examples/              # Example programs by complexity
│   ├── simple/           # Basic features
│   ├── intermediate/     # Moderate complexity
│   └── complex/          # Advanced features
├── edge-cases/           # Edge cases and boundary conditions
└── error-cases/          # Invalid code that should error
```

## Examples

### Simple Examples (`examples/simple/`)

Basic language features suitable for introduction and smoke testing:

- **arithmetic.oxur** - Basic arithmetic operations (Tier 1 calculator)
- **repl-variable.oxur** - Variable definition and usage
- **repl-variable-mutable.oxur** - Mutable variable mutation
- **function.oxur** - Function definition and calling

### Intermediate Examples (`examples/intermediate/`)

Moderate complexity features:

- **recursion.oxur** - Recursive functions (factorial, fibonacci, tail recursion)
- **closures.oxur** - Closures and higher-order functions

### Complex Examples (`examples/complex/`)

Advanced language features:

- **macros.oxur** - Macro definitions and expansions

## Edge Cases (`edge-cases/`)

Boundary conditions and special inputs:

- **empty.oxur** - Empty file handling
- **unicode.oxur** - Unicode in identifiers and strings

## Error Cases (`error-cases/`)

Invalid code that should produce specific errors:

- **syntax_error.oxur** - Syntax errors (unclosed parens, invalid literals, etc.)
- **type_error.oxur** - Type mismatches and type checking errors
- **runtime_error.oxur** - Runtime errors (division by zero, stack overflow, etc.)

## Usage

### In Tests

```rust
use std::path::PathBuf;

// Load test data
let test_data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("test-data");

// Read a simple example
let arithmetic_path = test_data_dir
    .join("examples/simple/arithmetic.oxur");
let code = std::fs::read_to_string(&arithmetic_path)?;

// Parse and evaluate
let result = parser.parse(&code)?;
```

### Expected Behavior

Each .oxur file contains comments indicating expected behavior:

```lisp
(+ 1 2)
;; Expected: 3
```

Error cases include expected error messages:

```lisp
(/ 10 0)
;; Expected: Runtime error: division by zero
```

## Test Coverage

These examples are designed to cover:

- ✅ **Tier 1:** Calculator mode (simple arithmetic)
- ✅ **Tier 2:** Cached compilation (variables, functions)
- ✅ **Tier 3:** JIT compilation (full features)
- ✅ **Error handling:** Syntax, type, and runtime errors
- ✅ **Edge cases:** Empty input, Unicode, boundary conditions

## Adding New Test Data

When adding new test data:

1. **Choose appropriate directory:**
   - Simple: Basic single-feature examples
   - Intermediate: Multiple features combined
   - Complex: Advanced language features
   - Edge cases: Boundary conditions
   - Error cases: Invalid code

2. **Include expected output:**
   - Use `;;  Expected: <result>` comments
   - For errors, include expected error messages

3. **Follow naming conventions:**
   - Use lowercase with hyphens: `my-test.oxur`
   - Descriptive names: `recursion.oxur`, not `test1.oxur`

4. **Document purpose:**
   - Add a header comment explaining what is tested
   - Include usage examples if non-obvious

## Integration with Test Framework

This test data is used by:

- **Unit tests:** Individual parser/evaluator tests
- **Integration tests:** End-to-end REPL tests
- **Benchmarks:** Performance measurement across tiers
- **Documentation:** Example code in guides

## See Also

- **ODD-0040:** Oxur REPL Implementation Plan
- **crates/oxur-repl/tests/e2e_tests.rs:** End-to-end tests using this data
- **crates/oxur-repl/benches/repl_benchmarks.rs:** Benchmarks using this data
