---
number: 12
title: "Implementation Plan: File-Based S-Expression Test Data"
author: "AST node"
created: 2025-12-27
updated: 2025-12-27
state: Accepted
supersedes: null
superseded-by: null
---

# Implementation Plan: File-Based S-Expression Test Data

## Executive Summary

This plan outlines how to enhance the `oxur-ast` crate with file I/O capabilities for S-expressions, organize test data into files, and migrate existing tests. Total estimated time: 8-12 hours.

## Goals

1. **Enable file-based S-expression handling**: Read and write .sexp files
2. **Organize test data**: Create logical directory structure for test fixtures
3. **Improve maintainability**: Make tests easier to read, write, and debug
4. **Enhance documentation**: Use S-expression files as examples

## Non-Goals

- We are NOT switching to an external S-expression library
- We are NOT changing the S-expression syntax or semantics
- We are NOT rewriting the parser or printer (just extending)

## Why Keep Our Custom Implementation?

The external `sexpr` crate (https://github.com/zv/sexpr) exists, but we should keep our custom implementation because:

1. **Full control**: We can extend exactly as needed for Rust AST
2. **No dependencies**: Keeps the crate lightweight
3. **Tailored errors**: Custom error messages for our use case  
4. **Integrated**: Works seamlessly with our position tracking
5. **Learning**: Understanding S-expressions helps with Oxur design
6. **Specialized**: Our printer is optimized for AST output

The `sexpr` crate is general-purpose; ours is specialized for AST serialization.

## Phases

### Phase 1: Core File I/O (2-3 hours)

**Goal**: Add ability to read/write S-expressions from/to files

**Tasks**:

1. **Update error types** (`src/error.rs`)
   - Add `FileReadError` variant with `PathBuf` and `std::io::Error`
   - Implement `Display` for new variant
   - Update `Result` type alias

2. **Extend Parser** (`src/sexp/parser.rs`)
   - Add `parse_file()` method
   - Handle file reading errors properly
   - Add basic tests

3. **Extend Printer** (`src/sexp/printer.rs`)
   - Add `write_file()` method  
   - Add `write_sexp_file()` convenience function
   - Add basic tests (use `tempfile` crate)

4. **Add dependencies** (if needed)
   - Add `tempfile` to `dev-dependencies` for tests

**Deliverables**:
- [ ] Updated `error.rs` with `FileReadError`
- [ ] `Parser::parse_file()` implemented and tested
- [ ] `Printer::write_file()` implemented and tested
- [ ] Unit tests passing
- [ ] No breaking changes to existing API

**Files to modify**:
- `oxur-ast/src/error.rs`
- `oxur-ast/src/sexp/parser.rs`
- `oxur-ast/src/sexp/printer.rs`
- `oxur-ast/Cargo.toml` (dev-dependencies)

### Phase 2: Directory Structure (1-2 hours)

**Goal**: Create organized structure for test data

**Tasks**:

1. **Create directory hierarchy**
   ```
   oxur-ast/
   └── test-data/
       ├── examples/          # Documented examples
       ├── fixtures/          # Test fixtures
       │   ├── crate/
       │   ├── item/
       │   ├── expr/
       │   ├── stmt/
       │   └── block/
       └── error-cases/       # Invalid S-expressions
   ```

2. **Create README files**
   - Main `test-data/README.md`
   - `examples/README.md` with usage guide
   - `fixtures/README.md` with organization notes

**Deliverables**:
- [ ] Directory structure created
- [ ] README files written
- [ ] `.gitignore` updated if needed

**Commands**:
```bash
cd oxur-ast
mkdir -p test-data/{examples,fixtures/{crate,item,expr,stmt,block},error-cases}
```

### Phase 3: Create Example Files (2-3 hours)

**Goal**: Create well-documented example S-expressions

**Tasks**:

1. **Core examples** (in `test-data/examples/`)
   - `simple-crate.sexp` - Empty crate
   - `simple-function.sexp` - Basic function
   - `macro-call-expr.sexp` - Macro invocation
   - `path-expr.sexp` - Path expression
   - `empty-block.sexp` - Empty block
   - `stmt-semi.sexp` - Statement with semicolon

2. **Add documentation**
   - Comment at top of each file explaining what it represents
   - Show equivalent Rust code where applicable
   - Use consistent formatting

3. **Validate examples**
   - Ensure all examples parse correctly
   - Test building AST from each example

**Deliverables**:
- [ ] 6-10 well-documented example files
- [ ] All examples parse without errors
- [ ] README explains each example

**File naming convention**:
- Use descriptive hyphenated names
- Include AST node type in name
- Example: `empty-crate.sexp`, `public-function.sexp`

### Phase 4: Extract Test Fixtures (3-4 hours)

**Goal**: Move inline S-expressions from tests to files

**Tasks**:

1. **Identify inline S-expressions**
   - Scan all test files
   - Find `r#"(...)"#` patterns
   - Document current test coverage

2. **Extract to fixture files** (by directory)
   
   **`fixtures/crate/`**:
   - `empty-crate.sexp`
   - `crate-with-one-item.sexp`  
   - `crate-with-multiple-items.sexp`
   - `complex-nested-crate.sexp`
   
   **`fixtures/item/`**:
   - `public-function.sexp`
   - `inherited-visibility.sexp`
   - `unsafe-function.sexp`
   - `const-function.sexp`
   - `function-with-body.sexp`
   - `function-with-params.sexp`
   
   **`fixtures/expr/`**:
   - `macro-call-empty.sexp`
   - `macro-call-delimited.sexp`
   - `path-single-segment.sexp`
   - `path-multiple-segments.sexp`
   
   **`fixtures/stmt/`**:
   - `empty-stmt.sexp`
   - `semi-stmt.sexp`
   - `expr-stmt.sexp`
   
   **`fixtures/block/`**:
   - `empty-block.sexp`
   - `block-with-stmts.sexp`

3. **Add comments to each fixture**
   - Describe what it tests
   - Reference related Rust AST node

**Deliverables**:
- [ ] 20-30 fixture files created
- [ ] All fixtures parse correctly
- [ ] Organized by AST node type

### Phase 5: Migrate Tests (3-4 hours)

**Goal**: Update all tests to use file-based fixtures

**Tasks**:

1. **Update `builder_tests.rs`**
   - Replace inline strings with file reads
   - Group related tests
   - Add helper functions if needed

2. **Update `builder_expr_tests.rs`**
   - Migrate expression tests to use fixtures
   - Ensure all scenarios still covered

3. **Update `builder_item_tests.rs`**  
   - Migrate item tests to use fixtures
   - Test error cases

4. **Update `builder_stmt_tests.rs`**
   - Migrate statement tests to use fixtures

5. **Update `integration_tests.rs`**
   - Use example files where appropriate
   - Test round-trip with files

6. **Add validation test**
   - Test that all .sexp files parse
   - Run in CI to catch corruption

**Helper function pattern**:
```rust
fn parse_fixture(path: &str) -> SExp {
    Parser::parse_file(format!("test-data/fixtures/{}", path))
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path, e))
}
```

**Deliverables**:
- [ ] All test files updated
- [ ] All tests passing
- [ ] Validation test added
- [ ] Code is cleaner and more readable

### Phase 6: Update Examples (1-2 hours)

**Goal**: Update example programs to use files

**Tasks**:

1. **Update `examples/build_simple_crate.rs`**
   - Read from example file instead of inline string
   - Show file path in output

2. **Update `examples/parse_example.rs`**
   - Demonstrate file parsing
   - Show both string and file parsing

3. **Create new example** (`examples/file_io.rs`)
   - Demonstrate reading S-expression files
   - Demonstrate writing S-expression files
   - Show round-trip: file → parse → print → file

**Deliverables**:
- [ ] All examples updated
- [ ] New file I/O example added
- [ ] Examples run successfully

### Phase 7: Documentation (1-2 hours)

**Goal**: Document new capabilities and organization

**Tasks**:

1. **Update main README**
   - Add section on file I/O
   - Link to test-data organization
   - Show example usage

2. **Create `CONTRIBUTING.md`** (if not exists)
   - Explain test data organization
   - Show how to add new test cases
   - Document file naming conventions

3. **API documentation**
   - Document `parse_file()` method
   - Document `write_file()` method
   - Add examples in rustdoc

4. **Update CHANGELOG**
   - Document new features
   - Note improved test organization

**Deliverables**:
- [ ] README updated
- [ ] CONTRIBUTING.md created/updated
- [ ] API docs complete
- [ ] CHANGELOG updated

## Success Criteria

We'll know this is successful when:

- [ ] All existing tests pass without modification to test logic
- [ ] Test files are significantly shorter and more readable
- [ ] New contributors can easily find and understand test examples
- [ ] S-expression files have syntax highlighting in editors
- [ ] Git diffs show test data changes clearly
- [ ] CI validates all .sexp files parse correctly
- [ ] Documentation clearly explains file organization

## Testing Strategy

### Unit Tests
- Test `parse_file()` with valid files
- Test `parse_file()` with nonexistent files
- Test `write_file()` creates correct content
- Test `write_file()` with invalid paths

### Integration Tests  
- Test all fixture files parse correctly
- Test round-trip: file → parse → build → serialize → file
- Test examples work with file I/O

### CI Validation
- Add step to parse all .sexp files
- Fail build if any .sexp file is invalid
- Check formatting consistency

## Rollout Plan

### Week 1: Implementation
- Days 1-2: Phase 1 (File I/O)
- Day 3: Phase 2 (Directory structure)
- Days 4-5: Phase 3 (Examples)

### Week 2: Migration
- Days 1-3: Phase 4 (Extract fixtures)
- Days 4-5: Phase 5 (Migrate tests)

### Week 3: Polish
- Days 1-2: Phase 6 (Update examples)
- Days 3-5: Phase 7 (Documentation)

## Risk Mitigation

### Risk: Breaking existing tests
**Mitigation**: Run tests after each phase; keep inline strings until files are proven

### Risk: File path issues across platforms
**Mitigation**: Use `Path::new()` and test on multiple platforms

### Risk: Large diffs make review difficult
**Mitigation**: Break into small PRs, one phase at a time

### Risk: Lost test coverage
**Mitigation**: Checklist of all current tests; verify each is migrated

## Dependencies

### New Crate Dependencies
```toml
[dev-dependencies]
tempfile = "3.8"  # For testing file I/O
```

### External Tools (Optional)
- Lisp formatter for .sexp files (if available)
- Git hooks for .sexp validation

## Metrics

Track these metrics to measure success:

- **Lines of test code**: Should decrease by ~30-40%
- **Number of test fixtures**: Should increase to ~30+
- **Test readability**: Subjective but should improve
- **Time to add new test**: Should decrease
- **Test failures**: Should remain constant or decrease

## Future Enhancements

After this plan is complete, consider:

1. **Fuzzing**: Generate random .sexp files for testing
2. **Schema validation**: Define schema for valid S-expressions  
3. **Pretty printer**: Format all .sexp files consistently
4. **AST→S-expression generator**: Auto-generate test files from Rust AST
5. **Documentation generator**: Generate docs from example files

## Questions to Resolve

Before starting, answer these:

- [ ] Should we use absolute or relative paths in tests?
- [ ] Do we need a helper crate for test utilities?
- [ ] Should example files include full position info or use simplified spans?
- [ ] How do we handle test data that changes frequently?
- [ ] Should we commit debug output files or gitignore them?

## Conclusion

This plan provides a structured approach to enhancing the oxur-ast crate with file-based S-expression handling. The benefits are clear:

- **Better maintainability**: Tests are easier to read and modify
- **Better documentation**: Examples serve dual purpose
- **Better collaboration**: Easy to share and discuss specific test cases
- **Better tooling**: Can leverage existing tools for .sexp files

The custom S-expression implementation should be kept rather than adopting an external library, as it's specifically tailored to our AST serialization needs.

Estimated total time: 8-12 hours spread across 2-3 weeks for thorough implementation and testing.
