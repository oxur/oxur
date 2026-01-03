# Feature Plan: Continue After Error for aster to-ast Command

**Date:** 2025-12-31
**Feature:** `--continue-after-error` flag for partial AST generation
**Effort Estimate:** 2-3 hours
**Priority:** High (user-requested for syntax exploration)

---

## Overview

Add a `--continue-after-error` flag to the `aster to-ast` command that allows processing to continue when unsupported Rust constructs are encountered. Instead of failing, the tool will generate S-expression comments documenting the unsupported code.

## Use Case

The user wants to explore Rust code for syntax inspiration, getting S-expressions for the parts that ARE supported while seeing what's missing in context. This enables:
- Partial AST generation from real-world Rust files
- Clear documentation of what's not yet supported
- Syntax exploration without manual file editing

## Current Behavior

```bash
$ ./bin/aster to-ast -o output.sexp input.rs
Error: Expected supported item type (currently only: `fn`), found `use` statement at line 1, column 1
```

Command fails immediately on first unsupported item.

## Desired Behavior

```bash
$ ./bin/aster to-ast --continue-after-error -o output.sexp input.rs
```

Generates output like:
```clojure
;; Oxur AST does not support the following Rust code
;; Error: Expected supported item type (currently only: `fn`), found `use` statement
;;
;; use std::env;

;; Oxur AST does not support the following Rust code
;; Error: Expected supported item type (currently only: `fn`), found `use` statement
;;
;; use oxur_syntax_exploration::{
;;     create_personalized_greeting,
;;     format_greeting,
;; };

(Crate
  :attrs ()
  :items (
    (Item
      :ident (Ident :name "main" ...)
      :kind (Fn ...)
      ...)
    (Item
      :ident (Ident :name "print_usage_info" ...)
      :kind (Fn ...)
      ...)
  )
  ...)
```

---

## Technical Approach

### Key Insight

Instead of converting the entire `syn::File` at once, we'll:
1. Process each top-level item individually
2. Collect successful conversions into the AST
3. Collect failures as comment blocks
4. Merge both into the final output

### Dependencies

Already available:
- ✅ `prettyplease` - For converting syn AST back to pretty Rust code
- ✅ `syn` - Already gives us individual items
- ✅ S-expression comment syntax (just strings starting with `;`)

### Implementation Strategy

#### Phase 1: CLI Changes
Add flag to `ToAst` command in `src/cli.rs`

#### Phase 2: Error Handling Infrastructure
Create types and functions for collecting partial results:
- `PartialConversionResult` - Success or error with context
- `generate_error_comment()` - Create S-expression comment block

#### Phase 3: Modified Conversion Logic
Update `from_syn.rs` to support partial conversion:
- `convert_file_partial()` - Item-by-item conversion
- Collect both successes and failures

#### Phase 4: Output Generation
Generate mixed output combining:
- S-expression comments for errors
- Generated S-expressions for successful items

#### Phase 5: Testing
Test with files containing mix of supported/unsupported items

---

## Detailed Implementation Plan

### 1. CLI Changes (`src/cli.rs`)

**File:** `crates/oxur-ast/src/cli.rs`

Add flag to `Commands::ToAst`:
```rust
#[command(name = "to-ast", about = "Convert Rust source to AST S-expression")]
ToAst {
    /// Input Rust source file
    #[arg(short, long)]
    input: PathBuf,

    /// Output S-expression file
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Use compact output format
    #[arg(long)]
    compact: bool,

    /// Continue processing after errors, generating comments for unsupported items
    #[arg(long)]
    continue_after_error: bool,
},
```

### 2. Commands Changes (`src/commands/to_ast.rs` or `mod.rs`)

**Current signature:**
```rust
pub fn to_ast(input: PathBuf, output: Option<PathBuf>, compact: bool) -> Result<()>
```

**New signature:**
```rust
pub fn to_ast(
    input: PathBuf,
    output: Option<PathBuf>,
    compact: bool,
    continue_after_error: bool,
) -> Result<()>
```

**Modified logic:**
```rust
pub fn to_ast(input: PathBuf, output: Option<PathBuf>, compact: bool, continue_after_error: bool) -> Result<()> {
    // Read file
    let content = fs::read_to_string(&input)?;

    if continue_after_error {
        // Use partial conversion
        let (crate_ast, comments) = parse_rust_file_partial(&content)?;

        // Generate output with comments interspersed
        let output_content = generate_output_with_comments(&crate_ast, &comments, compact)?;

        // Write output
        write_output(output, output_content)?;
    } else {
        // Existing behavior - fail on first error
        let crate_ast = parse_rust_file(&content)?;
        let sexp = generate_crate(&crate_ast);
        let output_content = print_sexp(&sexp, compact);
        write_output(output, output_content)?;
    }

    Ok(())
}
```

### 3. Integration Changes (`src/integration/from_syn.rs`)

**New types:**
```rust
/// Result of attempting to convert a single item
pub enum ItemConversionResult {
    Success(Item),
    Error {
        error: ParseError,
        syn_item: syn::Item,  // Keep original for pretty-printing
    },
}

/// Result of partial file conversion
pub struct PartialConversionResult {
    pub items: Vec<Item>,
    pub errors: Vec<ErrorComment>,
}

/// An error comment to be inserted in output
pub struct ErrorComment {
    pub error_message: String,
    pub rust_code: String,
}
```

**New functions:**
```rust
/// Convert a syn::File to AST, collecting errors instead of failing
pub fn parse_rust_file_partial(source: &str) -> Result<(Crate, Vec<ErrorComment>)> {
    let file = syn::parse_file(source)?;

    let mut converter = SynConverter::new();
    let mut successful_items = Vec::new();
    let mut error_comments = Vec::new();

    for item in &file.items {
        match converter.convert_item(item) {
            Ok(ast_item) => {
                successful_items.push(ast_item);
            }
            Err(e) => {
                // Generate pretty Rust code for the failed item
                let rust_code = prettyprint_item(item);

                error_comments.push(ErrorComment {
                    error_message: e.to_string(),
                    rust_code,
                });
            }
        }
    }

    // Create Crate with successful items
    let inner_span = Span::new(0, 0);
    let spans = ModSpans::new(inner_span);
    let crate_ast = Crate::new(successful_items, spans, converter.next_id());

    Ok((crate_ast, error_comments))
}

/// Pretty-print a syn::Item back to Rust code
fn prettyprint_item(item: &syn::Item) -> String {
    // prettyplease requires a File, so wrap the item
    let file = syn::File {
        shebang: None,
        attrs: vec![],
        items: vec![item.clone()],
    };

    prettyplease::unparse(&file)
}

/// Generate S-expression comment block for an error
fn generate_error_comment(error: &ErrorComment) -> String {
    let mut lines = vec![
        ";; Oxur AST does not support the following Rust code".to_string(),
        format!(";; Error: {}", error.error_message),
        ";;".to_string(),
    ];

    // Comment out each line of Rust code
    for line in error.rust_code.lines() {
        lines.push(format!(";; {}", line));
    }

    lines.join("\n")
}
```

### 4. Output Generation

**New function in `commands/to_ast.rs` or helper module:**
```rust
fn generate_output_with_comments(
    crate_ast: &Crate,
    error_comments: &[ErrorComment],
    compact: bool,
) -> Result<String> {
    let mut output = String::new();

    // Add all error comments at the top
    for error in error_comments {
        output.push_str(&generate_error_comment(error));
        output.push_str("\n\n");
    }

    // Add the generated S-expression
    let sexp = generate_crate(crate_ast);
    let sexp_string = print_sexp(&sexp, compact);
    output.push_str(&sexp_string);

    Ok(output)
}
```

**Alternative approach** (comments interspersed at original positions):
Would require tracking item positions and interleaving comments, which is more complex but produces cleaner output. For V1, we'll use the simpler "comments at top" approach.

### 5. Main Entry Point (`src/main.rs`)

Update the command dispatch:
```rust
fn execute_command(command: Commands) -> Result<()> {
    match command {
        Commands::ToAst { input, output, compact, continue_after_error } => {
            commands::to_ast(input, output, compact, continue_after_error)
        }
        // ... other commands
    }
}
```

---

## Testing Strategy

### Test Case 1: File with Mix of Supported/Unsupported Items

**Input file:** `test_partial.rs`
```rust
use std::io;

fn hello() {
    println!("Hello");
}

struct Point {
    x: i32,
    y: i32,
}

fn world() {
    println!("World");
}
```

**Expected behavior:**
- Without flag: Fails on `use std::io;`
- With flag: Generates comments for `use`, S-expressions for both functions

### Test Case 2: File with Only Unsupported Items

**Input:** File with only `use` statements

**Expected behavior:**
- Without flag: Fails immediately
- With flag: Generates only comments, with minimal/empty Crate S-expression

### Test Case 3: File with Only Supported Items

**Input:** File with only functions

**Expected behavior:**
- With or without flag: Same output (no errors to comment)

### Test Case 4: Complex Unsupported Item

**Input:** Large struct with many fields

**Expected behavior:**
- Error comment includes full pretty-printed struct

---

## Files to Modify

1. ✅ `crates/oxur-ast/src/cli.rs` - Add flag
2. ✅ `crates/oxur-ast/src/commands/mod.rs` or `to_ast.rs` - Add parameter
3. ✅ `crates/oxur-ast/src/integration/from_syn.rs` - Add partial conversion
4. ✅ `crates/oxur-ast/src/main.rs` - Update dispatch
5. ✅ Create test files in `tests/` or `fixtures/`

## Dependencies Check

**Already in Cargo.toml:**
```toml
[dependencies]
prettyplease = "0.2"  # ✅ Already present
syn = { version = "2.0", features = ["full", "extra-traits"] }  # ✅ Already present
```

No new dependencies needed!

---

## Edge Cases to Handle

1. **Empty file** - Should produce minimal Crate with no items
2. **All items fail** - Should produce all comments + minimal Crate
3. **Nested errors** - Currently only handles top-level items
   - Future: Could extend to handle errors in expressions/statements within functions
4. **Very large failed items** - Pretty-printing might produce huge comments
   - Acceptable for V1, could add truncation later

---

## Success Criteria

- ✅ CLI accepts `--continue-after-error` flag
- ✅ Without flag: Existing behavior unchanged (fail on error)
- ✅ With flag: Generates comments for unsupported items
- ✅ With flag: Successfully converts supported items
- ✅ Output is valid S-expression format (comments don't break parsing)
- ✅ Error messages are informative
- ✅ Rust code in comments is readable (pretty-printed)
- ✅ Tests pass for mixed files

---

## Future Enhancements (Out of Scope for V1)

1. **Interleaved comments** - Place error comments at original item positions
2. **Nested error handling** - Handle errors within function bodies
3. **Error summary** - Print summary of errors to stderr
4. **Partial item support** - Try to convert parts of unsupported items
5. **Custom error messages** - More helpful guidance for common errors
6. **Color-coded output** - Highlight errors in terminal
7. **Statistics** - Report "X of Y items converted successfully"

---

## Implementation Checklist

- [ ] Add `continue_after_error` flag to CLI
- [ ] Update `to_ast` command signature and dispatch
- [ ] Add `ItemConversionResult` and `ErrorComment` types
- [ ] Implement `parse_rust_file_partial()` function
- [ ] Implement `prettyprint_item()` helper
- [ ] Implement `generate_error_comment()` helper
- [ ] Implement `generate_output_with_comments()` function
- [ ] Update command logic to use partial conversion when flag is set
- [ ] Create test fixtures (mixed supported/unsupported items)
- [ ] Add integration test for the feature
- [ ] Rebuild and test manually with real files
- [ ] Update documentation/help text

---

## Estimated Timeline

- **CLI changes:** 15 minutes
- **Type definitions:** 15 minutes
- **Partial conversion logic:** 45 minutes
- **Output generation:** 30 minutes
- **Integration/testing:** 45 minutes
- **Manual testing & fixes:** 30 minutes

**Total:** ~3 hours

---

## Notes

- This is a clean additive feature - no risk to existing functionality
- Aligns perfectly with user's syntax exploration use case
- Uses existing dependencies (prettyplease)
- Can be extended in future for more sophisticated error handling
- Documentation in comments will help users understand AST gaps

---

*Ready for implementation!*
