# Implementation Plan: S-Expression File I/O and Test Data Organization

## Overview

Add file I/O capabilities to the oxur-ast S-expression implementation and organize test data into external `.sexp` files organized by complexity level.

## User Decisions

Based on discussion:
- **Location**: `crates/oxur-ast/test-data/` (co-located with crate)
- **Organization**:
  - Examples: By complexity (simple/, intermediate/, complex/) - for learning
  - Fixtures: By AST node type (crate/, item/, expr/, stmt/, block/) - for systematic testing
  - error-cases/: Separate directory for invalid S-expressions
- **Dual Purpose**: Examples can be used in tests, but fixtures are separate from examples
- **Directory Name**: `test-data/`

## Current State

**What Exists:**
- ✅ Complete S-expression parser (`Parser::parse_str()`)
- ✅ Complete S-expression printer (`Printer::print()`)
- ✅ Full lexer with position tracking
- ✅ All S-expression types (Symbol, Keyword, String, Number, Nil, List)
- ✅ 33 passing unit tests using inline strings
- ✅ Two basic examples using inline strings

**What's Missing (ALL of file I/O):**
- ❌ No `parse_file()` method
- ❌ No `write_file()` method
- ❌ No `FileReadError` variant in error types
- ❌ No `test-data/` directory
- ❌ No `.sexp` fixture files (all tests use inline `r#"..."#` strings)

## Directory Structure (Final Design)

```
crates/oxur-ast/
├── src/
│   ├── error.rs              # Will add FileReadError
│   ├── sexp/
│   │   ├── parser.rs         # Will add parse_file()
│   │   ├── printer.rs        # Will add write_file()
│   │   ├── lexer.rs          # (no changes)
│   │   ├── types.rs          # (no changes)
│   │   └── mod.rs            # (no changes)
│   └── ...
├── test-data/                # NEW - hybrid organization
│   ├── README.md            # Explains organization, usage
│   ├── examples/            # Learning/documentation (by complexity)
│   │   ├── simple/          # Basic examples for beginners
│   │   │   ├── empty-crate.sexp
│   │   │   ├── simple-fn.sexp
│   │   │   ├── nil-value.sexp
│   │   │   └── ...
│   │   ├── intermediate/    # Moderate complexity
│   │   │   ├── fn-with-params.sexp
│   │   │   ├── nested-blocks.sexp
│   │   │   ├── macro-call.sexp
│   │   │   └── ...
│   │   └── complex/         # Real-world complexity
│   │       ├── full-crate.sexp
│   │       ├── deeply-nested.sexp
│   │       └── ...
│   ├── fixtures/            # Systematic testing (by AST type)
│   │   ├── crate/
│   │   │   ├── empty.sexp
│   │   │   ├── with-one-item.sexp
│   │   │   └── with-multiple-items.sexp
│   │   ├── item/
│   │   │   ├── public-function.sexp
│   │   │   ├── inherited-visibility.sexp
│   │   │   └── unsafe-function.sexp
│   │   ├── expr/
│   │   │   ├── macro-call.sexp
│   │   │   ├── path-single.sexp
│   │   │   └── path-multiple.sexp
│   │   ├── stmt/
│   │   │   ├── empty.sexp
│   │   │   ├── semi.sexp
│   │   │   └── expr.sexp
│   │   └── block/
│   │       ├── empty.sexp
│   │       └── with-stmts.sexp
│   └── error-cases/         # Invalid S-expressions for error testing
│       ├── unterminated-list.sexp
│       ├── unexpected-close.sexp
│       └── invalid-escape.sexp
├── tests/
│   ├── parser_tests.rs      # Will update to use fixtures
│   ├── printer_tests.rs     # Will update to use fixtures
│   └── ...
├── examples/
│   ├── parse_example.rs     # Will update to use examples/
│   ├── build_simple_crate.rs # Will update to use examples/
│   └── file_io.rs           # NEW - demonstrates file I/O
└── Cargo.toml               # Will add tempfile to dev-deps
```

## Implementation Phases

### Phase 1: Core File I/O (2-3 hours)

**Goal**: Add ability to read/write S-expressions from/to files

**Critical Files:**
- `crates/oxur-ast/src/error.rs`
- `crates/oxur-ast/src/sexp/parser.rs`
- `crates/oxur-ast/src/sexp/printer.rs`
- `crates/oxur-ast/Cargo.toml`

**Tasks:**

1. **Update error.rs** - Add FileReadError variant
   ```rust
   #[error("Failed to read file {path}: {source}")]
   FileReadError {
       path: PathBuf,
       #[source]
       source: std::io::Error,
   }
   ```

2. **Extend parser.rs** - Add parse_file() method
   ```rust
   pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<SExp> {
       let content = std::fs::read_to_string(path.as_ref())
           .map_err(|e| Error::FileReadError {
               path: path.as_ref().to_path_buf(),
               source: e,
           })?;
       Self::parse_str(&content)
   }
   ```

3. **Extend printer.rs** - Add write_file() and convenience function
   ```rust
   pub fn write_file<P: AsRef<Path>>(&self, sexp: &SExp, path: P) -> std::io::Result<()>
   pub fn write_sexp_file<P: AsRef<Path>>(sexp: &SExp, path: P) -> std::io::Result<()>
   ```

4. **Update Cargo.toml** - Add tempfile for testing
   ```toml
   [dev-dependencies]
   tempfile = "3.8"
   ```

5. **Add unit tests** for file I/O operations

**Success Criteria:**
- [ ] `Parser::parse_file()` works and has tests
- [ ] `Printer::write_file()` works and has tests
- [ ] Error handling for missing files, bad paths
- [ ] No breaking changes to existing API
- [ ] All existing tests still pass

### Phase 2: Directory Structure & Examples (2-3 hours)

**Goal**: Create test-data structure with initial example files

**Tasks:**

1. **Create directory structure**
   ```bash
   mkdir -p crates/oxur-ast/test-data/examples/{simple,intermediate,complex}
   mkdir -p crates/oxur-ast/test-data/fixtures/{crate,item,expr,stmt,block}
   mkdir -p crates/oxur-ast/test-data/error-cases
   ```

2. **Create README.md** in test-data/
   - Explain two-tier organization (examples vs fixtures)
   - Document when to use examples/ vs fixtures/
   - Document file naming conventions
   - Show usage examples in tests
   - List what each complexity level should contain
   - Explain what each fixture type contains

3. **Create examples/simple/** (6-8 files for beginners)
   - `empty-crate.sexp` - Minimal crate
   - `simple-fn.sexp` - Basic function
   - `path-expr.sexp` - Simple path
   - `nil-value.sexp` - Nil literal
   - `number.sexp` - Number literal
   - `symbol.sexp` - Symbol
   - `keyword.sexp` - Keyword

4. **Create examples/intermediate/** (6-8 files)
   - `fn-with-params.sexp` - Function with parameters
   - `nested-blocks.sexp` - Nested block structures
   - `macro-call.sexp` - Macro invocation
   - `multi-item-crate.sexp` - Crate with several items
   - `visibility-variants.sexp` - Different visibility levels

5. **Create examples/complex/** (4-6 files)
   - `full-crate.sexp` - Complete realistic crate
   - `deeply-nested.sexp` - Deep nesting stress test
   - `all-node-types.sexp` - Exercises all AST nodes

6. **Create fixtures/** organized by AST type (20-25 files total)
   - **fixtures/crate/**: `empty.sexp`, `with-one-item.sexp`, `with-multiple-items.sexp`
   - **fixtures/item/**: `public-function.sexp`, `inherited-visibility.sexp`, `unsafe-function.sexp`, `const-function.sexp`, `function-with-body.sexp`
   - **fixtures/expr/**: `macro-call-empty.sexp`, `macro-call-delimited.sexp`, `path-single-segment.sexp`, `path-multiple-segments.sexp`
   - **fixtures/stmt/**: `empty.sexp`, `semi.sexp`, `expr.sexp`
   - **fixtures/block/**: `empty.sexp`, `with-stmts.sexp`

7. **Create error-cases/** (3-5 files)
   - `unterminated-list.sexp` - Missing closing paren
   - `unexpected-close.sexp` - Extra closing paren
   - `invalid-escape.sexp` - Bad escape sequence in string

8. **Document each file** with comments explaining what it represents

**Success Criteria:**
- [ ] Directory structure created (examples/, fixtures/, error-cases/)
- [ ] README.md explains dual organization (complexity vs type)
- [ ] 16-22 example files in examples/ (organized by complexity)
- [ ] 20-25 fixture files in fixtures/ (organized by AST type)
- [ ] 3-5 error-case files in error-cases/
- [ ] All files parse successfully (except error-cases/)
- [ ] Each file has explanatory comments

### Phase 3: Test Migration (3-4 hours)

**Goal**: Update tests to use external .sexp files instead of inline strings

**Files to Update:**
- `tests/parser_tests.rs`
- `tests/printer_tests.rs`
- `tests/builder_tests.rs`
- `tests/builder_expr_tests.rs`
- `tests/builder_item_tests.rs`
- `tests/builder_stmt_tests.rs`
- `tests/integration_tests.rs`

**Tasks:**

1. **Add helper functions** to tests (in common module or each test file)
   ```rust
   fn parse_example(path: &str) -> SExp {
       Parser::parse_file(format!("test-data/examples/{}", path))
           .unwrap_or_else(|e| panic!("Failed to parse example {}: {}", path, e))
   }

   fn parse_fixture(path: &str) -> SExp {
       Parser::parse_file(format!("test-data/fixtures/{}", path))
           .unwrap_or_else(|e| panic!("Failed to parse fixture {}: {}", path, e))
   }
   ```

2. **Migrate parser_tests.rs** (317 lines currently)
   - Replace inline `r#"..."#` with `parse_fixture("crate/empty.sexp")` etc.
   - Keep test logic identical
   - Verify round-trip tests still work

3. **Migrate printer_tests.rs** (232 lines currently)
   - Use fixtures for printer input
   - Test output formatting

4. **Migrate builder tests** (4 files, ~45KB total)
   - Use fixtures from fixtures/crate/, fixtures/item/, etc. based on what's being tested
   - builder_tests.rs → uses fixtures/crate/
   - builder_item_tests.rs → uses fixtures/item/
   - builder_expr_tests.rs → uses fixtures/expr/
   - builder_stmt_tests.rs → uses fixtures/stmt/
   - Ensure all test scenarios still covered

5. **Add validation tests**
   ```rust
   #[test]
   fn all_examples_are_valid() {
       // Parse every .sexp file in test-data/examples/
       // Ensure they all parse successfully
   }

   #[test]
   fn all_fixtures_are_valid() {
       // Parse every .sexp file in test-data/fixtures/
       // Ensure they all parse successfully
   }

   #[test]
   fn error_cases_fail_as_expected() {
       // Verify that files in error-cases/ properly fail to parse
       // Ensure error messages are helpful
   }
   ```

**Success Criteria:**
- [ ] All tests updated to use fixtures
- [ ] All tests still passing
- [ ] Test code is more readable
- [ ] Validation test catches corrupted fixtures
- [ ] Test files are 20-30% shorter

### Phase 4: Update Examples & Documentation (1-2 hours)

**Goal**: Update example programs and add file I/O documentation

**Tasks:**

1. **Update examples/parse_example.rs**
   - Change from inline strings to `parse_file()`
   - Use files from test-data/examples/simple/
   - Show both string and file parsing

2. **Update examples/build_simple_crate.rs**
   - Read from test-data/examples/simple/empty-crate.sexp
   - Show file path in output

3. **Create examples/file_io.rs** (NEW)
   - Demonstrate parse_file()
   - Demonstrate write_file()
   - Show round-trip: read → modify → write

4. **Update crates/oxur-ast/README.md**
   - Add section on file I/O
   - Link to test-data organization
   - Show usage examples

5. **Update API documentation**
   - Add rustdoc for parse_file()
   - Add rustdoc for write_file()
   - Include examples in docs

**Success Criteria:**
- [ ] All examples run successfully
- [ ] Examples demonstrate file I/O
- [ ] README documents new features
- [ ] API docs include examples

## File Naming Conventions

**For examples/simple/ directory:**
- Descriptive, hyphenated names
- Include what AST node it represents
- Examples: `empty-crate.sexp`, `simple-fn.sexp`, `nil-value.sexp`

**For examples/intermediate/ directory:**
- More specific about features
- Examples: `fn-with-params.sexp`, `nested-blocks.sexp`, `visibility-public.sexp`

**For examples/complex/ directory:**
- Describe the scenario or stress test
- Examples: `full-crate.sexp`, `deeply-nested.sexp`, `all-node-types.sexp`

**For fixtures/ directories:**
- Short, descriptive names focused on what's being tested
- Examples in fixtures/crate/: `empty.sexp`, `with-one-item.sexp`
- Examples in fixtures/item/: `public-function.sexp`, `inherited-visibility.sexp`
- Examples in fixtures/expr/: `macro-call.sexp`, `path-single.sexp`

**For error-cases/ directory:**
- Describe what kind of error it should trigger
- Examples: `unterminated-list.sexp`, `unexpected-close.sexp`

**File format:**
- Use 2-space indentation
- Add comment header explaining what it represents
- Include position info for completeness
- Format consistently with `Printer::print()`

## Testing Strategy

1. **Unit tests** for file I/O (Phase 1)
   - Test parse_file() with valid files
   - Test parse_file() with missing files
   - Test write_file() creates correct content
   - Test write_file() with bad paths

2. **Validation test** (Phase 3)
   - Parse all .sexp files in test-data/
   - Fail if any fixture is invalid
   - Run in CI to catch corruption

3. **Round-trip tests** (Phase 3)
   - Parse fixture → print → reparse → verify structure
   - Ensures fixtures are canonical

## Success Metrics

After implementation:
- [ ] All 33+ existing tests passing
- [ ] 16-22 example files in examples/ (organized by complexity)
- [ ] 20-25 fixture files in fixtures/ (organized by AST type)
- [ ] 3-5 error-case files in error-cases/
- [ ] Test files 20-30% shorter
- [ ] New file_io.rs example demonstrates capabilities
- [ ] README documents file I/O and dual organization
- [ ] All examples and fixtures parse in CI validation tests
- [ ] Error cases properly fail with expected errors

## Non-Goals

- NOT switching to external sexpr crate (keeping custom implementation)
- NOT changing S-expression syntax or semantics
- NOT rewriting parser/printer (just extending)
- NOT adding CLI tool (that's a separate phase from original planning docs)
- NOT adding syn integration yet (deferred)

## Estimated Time

- Phase 1: 2-3 hours
- Phase 2: 2-3 hours
- Phase 3: 3-4 hours
- Phase 4: 1-2 hours
- **Total: 8-12 hours**

## Key Differences from Original Plan

1. **Hybrid organization** instead of single approach
   - Original: Either all by complexity OR all by type
   - New: examples/ by complexity, fixtures/ by type, plus error-cases/
   - Reason: Examples are for learning (complexity progression), fixtures are for testing (type coverage)

2. **Three-tier structure** with clear separation
   - examples/: For documentation, learning, can be used in tests
   - fixtures/: For systematic testing organized by AST node type
   - error-cases/: For error handling tests
   - Reason: Clear separation of concerns, easy to navigate

3. **Fixtures NOT dual-purpose**
   - Original: Suggested examples = fixtures
   - New: Examples and fixtures are separate
   - Reason: Examples can stay simple for teaching, fixtures can be comprehensive for testing

## Risk Mitigation

**Risk**: Breaking existing tests
- **Mitigation**: Keep all test logic identical, only change data source

**Risk**: File path issues across platforms
- **Mitigation**: Use Path::new() consistently, test relative paths

**Risk**: Fixtures becoming outdated
- **Mitigation**: Validation test in CI ensures all fixtures parse

**Risk**: Unclear whether to add to examples/ or fixtures/
- **Mitigation**: README.md documents clear guidelines: examples/ for learning/docs, fixtures/ for systematic testing

**Risk**: Unclear which complexity level for new examples
- **Mitigation**: README.md documents guidelines for each level

**Risk**: Unclear which fixture type directory for new tests
- **Mitigation**: Match directory to AST node type being tested

## Next Steps After This Plan

1. Implement Phase 1 (file I/O)
2. Run all tests to verify no breakage
3. Implement Phase 2 (create structure and examples)
4. Implement Phase 3 (migrate tests)
5. Implement Phase 4 (examples and docs)
6. Final testing and cleanup
7. Commit with detailed message explaining the changes
