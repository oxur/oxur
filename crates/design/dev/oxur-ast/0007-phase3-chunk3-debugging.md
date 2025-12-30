# Chunk 3 Debugging Note

**Date:** 2025-12-29
**Status:** In Progress - Bug Found

## Current State

**Completed:**
- ✅ Implemented all 3 commands:
  - `to-ast` (Rust → S-expression) - WORKING
  - `to-rust` (S-expression → Rust) - implemented
  - `verify` (round-trip verification) - implemented
- ✅ Created test fixtures (hello_world.rs, simple_fn.rs, let_bindings.rs)
- ✅ Written integration tests (13 tests in integration_tests.rs)
- ✅ Written regression tests (regression_tests.rs)

**NOT COMMITTED YET** - Has bug that needs fixing first

## Bug Description

**Issue:** Round-trip parsing failure

**Symptoms:**
```bash
$ aster to-ast crates/oxur-ast/tests/fixtures/hello_world.rs -o /tmp/hello.sexp
# SUCCESS - generates 179 lines of S-expression

$ aster to-rust /tmp/hello.sexp
# ERROR: Expected value after keyword, found end of list at line 29, column 7

$ aster verify crates/oxur-ast/tests/fixtures/hello_world.rs
# ERROR: Same parsing error
```

**Test Results:**
- 12 out of 13 integration tests passing
- `test_round_trip_hello_world` FAILING with same error
- Error: `Expected { expected: "value after keyword", found: "end of list", pos: Position { offset: 332, line: 29, column 7 } }`

**Debug Info:**
- Generated S-expression appears well-formed visually
- 179 lines in /tmp/hello.sexp
- Error occurs when trying to parse the generated S-expression back
- Line 29 contains a keyword without its corresponding value
- Likely culprit: One of the generator methods is adding a keyword without a value

**Suspected Areas:**
- Generator methods in src/generator/gen.rs, item.rs, expr.rs, or stmt.rs
- Possibly in `generate_mod_spans`, `generate_fn_header`, or similar functions
- Look for `kwarg()` calls that might be malformed
- Check for missing values in typed_node constructions

**Files Modified (not yet committed):**
```
M  crates/oxur-ast/src/commands/to_ast.rs
M  crates/oxur-ast/src/commands/to_rust.rs
M  crates/oxur-ast/src/commands/verify.rs
A  crates/oxur-ast/tests/fixtures/hello_world.rs
A  crates/oxur-ast/tests/fixtures/simple_fn.rs
A  crates/oxur-ast/tests/fixtures/let_bindings.rs
M  crates/oxur-ast/tests/integration_tests.rs (added 6 new tests)
A  crates/oxur-ast/tests/regression_tests.rs
```

## Next Steps After Compaction

1. **Debug the parsing error:**
   - Read /tmp/hello.sexp line 29 specifically
   - Find which generator method is creating the malformed keyword
   - Fix the generator to provide proper keyword-value pairs

2. **Once fixed:**
   - Run all tests: `cargo test -p oxur-ast`
   - Test all commands manually
   - Verify clippy clean: `cargo clippy -p oxur-ast -- -D warnings`
   - Commit Chunk 3

3. **Then proceed to Chunk 4:**
   - Create benchmark suite
   - Write examples (parse_rust_file.rs, convert_file.rs)
   - Update README and docs
   - Final verification
   - Create code reuse deduplication report

## Quick Test Commands

```bash
# Generate S-expression (working)
cargo run -p oxur-ast --bin aster -- to-ast crates/oxur-ast/tests/fixtures/hello_world.rs

# Parse back (failing)
cargo run -p oxur-ast --bin aster -- to-ast crates/oxur-ast/tests/fixtures/hello_world.rs -o /tmp/test.sexp
cargo run -p oxur-ast --bin aster -- to-rust /tmp/test.sexp

# Verify round-trip (failing)
cargo run -p oxur-ast --bin aster -- verify crates/oxur-ast/tests/fixtures/hello_world.rs

# Run tests
cargo test -p oxur-ast --test integration_tests
```

## Context Before Compaction

- Token usage: ~148k/200k (74%)
- Chunks 1 & 2 committed successfully
- Chunk 3 nearly complete, just needs bug fix
- All implementation done, just needs debugging and testing
