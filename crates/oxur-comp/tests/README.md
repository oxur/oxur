# Error Translation Integration Tests

These tests verify the complete error reporting pipeline from Oxur source code
to formatted rustc error messages.

## Test Coverage

- **Undefined variable errors**: Tests that rustc errors for undefined variables
  are correctly parsed and formatted
- **Error message format**: Verifies structured error output with positions
- **Valid code compilation**: Ensures valid Oxur code compiles successfully
- **Error codes**: Checks that rustc error codes (e.g., E0425) are included
- **SourceMap integration**: Verifies SourceMap flows through entire pipeline
- **Multiple errors**: Tests handling of multiple compilation errors
- **Error translation infrastructure**: Verifies ErrorTranslator formatting

## Current Behavior (Phase 1)

Error messages currently show:
- rustc error message
- Generated Rust file position (e.g., `generated.rs:5:10`)
- rustc error code (e.g., `E0425`)
- Note that position translation is not yet implemented

Example:
```
rustc failed with exit code: Some(1)

error: cannot find value `x` in this scope
  --> generated.rs:5:10
  (Note: Error position translation not yet implemented)
  code: E0425
```

## Future Enhancement (Phase 2)

When reverse index is implemented, errors will show:
- Oxur source file position (e.g., `example.oxur:2:8`)
- No note about translation (it will be working)

Example future output:
```
rustc failed with exit code: Some(1)

error: cannot find value `x` in this scope
  --> example.oxur:2:8
  code: E0425
```

## Running Tests

```bash
# Run all error translation tests
cargo test --test error_translation

# Run with output to see error messages
cargo test --test error_translation -- --nocapture

# Run specific test
cargo test --test error_translation test_undefined_variable_error
```

## Test Organization

**Helper functions:**
- `compile_and_get_error(source)`: Compiles Oxur source and returns error message

**Test categories:**
1. Error detection (undefined variables, type errors)
2. Error formatting (structure, positions, codes)
3. Valid code (no errors)
4. SourceMap integration (mappings preserved)
5. ErrorTranslator infrastructure (formatting consistency)

## Adding New Tests

When adding new error translation tests:

1. Use the `compile_and_get_error` helper for consistency
2. Test specific error types (undefined vars, type errors, etc.)
3. Verify error message structure (error:, -->, code:)
4. Check for translation note (until Phase 2)
5. Add clear assertions with descriptive messages

Example:
```rust
#[test]
fn test_new_error_type() {
    let source = r#"(deffn main () ...)"#;
    let error = compile_and_get_error(source).expect("Should produce error");

    assert!(error.contains("expected pattern"), "Should describe error");
    assert!(error.contains("-->"), "Should have position");
}
```

## Phase 1 Completion

These tests complete Phase 1 (Source Mapping Infrastructure) by verifying:
- ✅ Complete compilation pipeline works end-to-end
- ✅ Errors are parsed from rustc JSON output
- ✅ ErrorTranslator formats messages consistently
- ✅ SourceMap is passed through all stages
- ✅ Error reporting infrastructure is robust

Phase 2 will add:
- Reverse index for Rust position → NodeId lookup
- Full error translation (Rust position → Oxur position)
- No more "translation not yet implemented" notes
