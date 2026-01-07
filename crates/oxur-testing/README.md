# oxur-testing

Shared testing infrastructure and utilities for Oxur crates.

## Purpose

This crate provides common test utilities used across `oxur-ast`, `oxur-lang`, and `oxur-repl` for:

- Loading test data files
- Extracting expected outputs from comments
- Discovering test files by pattern
- Reducing code duplication in test infrastructure

## Usage

Add as a dev-dependency in your crate's `Cargo.toml`:

```toml
[dev-dependencies]
oxur-testing = { version = "0.1.0", path = "../oxur-testing" }
```

### Loading Test Files

```rust
use oxur_testing::load_test_file;

#[test]
fn test_arithmetic() {
    let test_file = load_test_file(
        env!("CARGO_MANIFEST_DIR"),
        "examples/simple/arithmetic.oxur"
    ).expect("Failed to load test file");

    assert!(!test_file.content.is_empty());
    assert_eq!(test_file.expected_outputs.len(), 3);
    assert_eq!(test_file.expected_outputs[0].value, "3");
}
```

### Using Macros (Less Boilerplate)

```rust
use oxur_testing::test_file;

#[test]
fn test_with_macro() {
    let file = test_file!("examples/simple/arithmetic.oxur");
    assert!(!file.content.is_empty());
}
```

### Discovering Test Files

```rust
use oxur_testing::discover_tests;

#[test]
fn test_all_oxur_files() {
    let files = discover_tests!("*.oxur");

    for file in files {
        println!("Testing: {}", file.relative_path);
        // Run tests on file.content
    }
}
```

### Environment Variable Locking

When tests modify environment variables (which are process-global state), they can interfere with each other when running in parallel, causing flaky test failures. Use `with_env_lock` to prevent race conditions:

```rust
use oxur_testing::env_lock::with_env_lock;
use std::env;

#[test]
fn test_with_custom_env() {
    with_env_lock(|| {
        env::set_var("MY_CACHE_DIR", "/tmp/test-cache");

        // Test code that depends on MY_CACHE_DIR
        // ...

        env::remove_var("MY_CACHE_DIR");
    });
}
```

**Why this matters:** Without the lock, parallel tests modifying the same (or different) environment variables can see each other's changes, causing non-deterministic test failures.

### Expected Output Format

Test files can include expected outputs in comments:

**Lisp-style:**
```lisp
(+ 1 2)
;; Expected: 3

(/ 10 0)
;; Expected: Runtime error: division by zero
```

**Rust-style:**
```rust
fn add(a: i32, b: i32) -> i32 { a + b }
// Expected: compiles successfully
```

The library automatically detects error expectations by looking for the word "error" in the expected value.

## API

### Core Functions

- `load_test_file(manifest_dir, relative_path)` - Load a single test file
- `discover_test_files(manifest_dir, pattern)` - Find all test files matching pattern
- `env_lock::with_env_lock(closure)` - Execute closure with exclusive environment variable access

### Macros

- `test_file!(path)` - Load test file with panic on error
- `discover_tests!(pattern)` - Discover test files with panic on error
- `manifest_dir!()` - Get CARGO_MANIFEST_DIR at compile time

### Types

- `TestFile` - Represents a test file with content and expected outputs
- `ExpectedOutput` - Represents an expected output with value, error flag, and line number

### Modules

- `env_lock` - Synchronization primitives for tests that modify environment variables

## Test Data Organization

Test data should be organized in a `test-data/` directory in each crate:

```
crates/your-crate/
├── test-data/
│   ├── examples/
│   │   ├── simple/
│   │   ├── intermediate/
│   │   └── complex/
│   ├── fixtures/      # Optional
│   ├── edge-cases/    # Optional
│   └── error-cases/   # Optional
└── tests/
    └── integration_tests.rs
```

## Testing

```bash
cd crates/oxur-testing
cargo test
```

All 11 unit tests should pass (8 for test file utilities + 3 for env_lock).

## See Also

- `crates/oxur-ast/test-data/` - Example test data for AST crate
- `crates/oxur-lang/test-data/` - Example test data for language crate
