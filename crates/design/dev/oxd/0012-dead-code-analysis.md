# Dead Code Analysis: oxur-ast Statement Builder

**Date:** 2025-12-26
**File:** `crates/oxur-ast/src/builder/stmt.rs`
**Lines:** 59-60, 75-76
**Issue:** Unreachable positional syntax fallback code

## Executive Summary

The `build_stmt_kind` function in `stmt.rs` contains dead code on lines 59-60 and 75-76. These lines implement a "positional syntax" fallback for `Semi` and `Expr` statement kinds, but they are unreachable due to the design of the `parse_kwargs` helper function.

**Impact:**
- 4 lines of dead code
- Prevents coverage from reaching 95%+ (currently at 93.65%)
- Suggests incomplete or abandoned feature implementation
- No functional impact (feature was never working)

## The Code in Question

### Semi Statement (lines 52-68)

```rust
"Semi" => {
    let kwargs = parse_kwargs(list)?;                    // Line 53
    if let Some(expr_sexp) = kwargs.get("expr") {
        let expr = self.build_expr(expr_sexp)?;
        Ok(StmtKind::Semi(expr))
    } else if list.elements.len() > 1 {                  // Line 57
        // Expression is the second element
        let expr = self.build_expr(&list.elements[1])?;  // Line 59 🪦 DEAD
        Ok(StmtKind::Semi(expr))                         // Line 60 🪦 DEAD
    } else {
        Err(ParseError::Expected {
            expected: "expression".to_string(),
            found: "missing".to_string(),
            pos: list.pos,
        })
    }
}
```

### Expr Statement (lines 69-84)

```rust
"Expr" => {
    let kwargs = parse_kwargs(list)?;                    // Line 70
    if let Some(expr_sexp) = kwargs.get("expr") {
        let expr = self.build_expr(expr_sexp)?;
        Ok(StmtKind::Expr(expr))
    } else if list.elements.len() > 1 {                  // Line 74
        let expr = self.build_expr(&list.elements[1])?;  // Line 75 🪦 DEAD
        Ok(StmtKind::Expr(expr))                         // Line 76 🪦 DEAD
    } else {
        Err(ParseError::Expected {
            expected: "expression".to_string(),
            found: "missing".to_string(),
            pos: list.pos,
        })
    }
}
```

## Root Cause Analysis

### The `parse_kwargs` Function

Located in `crates/oxur-ast/src/builder/helpers.rs` (lines 75-96):

```rust
pub fn parse_kwargs(list: &List) -> Result<std::collections::HashMap<String, &SExp>> {
    let mut map = std::collections::HashMap::new();
    let mut i = 1; // Skip first element (node type)

    while i < list.elements.len() {
        if i + 1 >= list.elements.len() {
            return Err(ParseError::Expected {
                expected: "value after keyword".to_string(),
                found: "end of list".to_string(),
                pos: list.pos,
            });
        }

        let key = expect_keyword(&list.elements[i])?;  // ⚠️ THE PROBLEM
        let value = &list.elements[i + 1];

        map.insert(key.name.clone(), value);
        i += 2;
    }

    Ok(map)
}
```

**The Problem:** Line 88's `expect_keyword(&list.elements[i])?` requires **every** element after the node type to be a keyword. If it encounters a non-keyword element (like a List for positional syntax), it returns an error immediately.

### Execution Flow

**Scenario: Positional Syntax Attempt**

```lisp
(Semi (Expr :kind (MacCall :path (Path ...))))
      ^^^^^
      Intended as positional argument
```

**What Happens:**

1. `build_stmt_kind` is called with the S-expression
2. Match arm for `"Semi"` is entered (line 52)
3. `parse_kwargs(list)?` is called (line 53)
4. Inside `parse_kwargs`:
   - `i = 1` (skip "Semi")
   - Loop condition: `1 < 2` ✓
   - Check if `i + 1 >= list.elements.len()`: `2 >= 2` ✓ (enters error branch)
   - Returns `Err("value after keyword", "end of list")`
5. The `?` operator propagates the error upward
6. **Line 57's `else if` is NEVER reached**
7. Lines 59-60 are **NEVER executed**

### Why It Fails

The `parse_kwargs` function has an **all-or-nothing design**:

- **All keywords:** `(Semi :expr (Expr ...))` ✅ Works
- **No keywords:** `(Semi)` ✅ Returns empty map
- **Mixed/Positional:** `(Semi (Expr ...))` ❌ Errors

When there's a non-keyword element at an odd position (where a keyword is expected), `parse_kwargs` errors before returning, preventing the fallback check.

## Currently Supported Syntax

### ✅ Working: Keyword Syntax

```lisp
(Stmt
  :id 100
  :kind (Semi
          :expr (Expr
                  :id 101
                  :kind (MacCall :path (Path ...))))
  :span (Span))
```

### ✅ Working: Empty (triggers error, but correctly)

```lisp
(Stmt
  :kind (Semi)
  :span (Span))
```

**Result:** Error - "expected expression, found missing" (correct)

### ❌ Broken: Positional Syntax (intended but never worked)

```lisp
(Stmt
  :kind (Semi (Expr :kind (MacCall :path (Path ...))))
  :span (Span))
```

**Result:** Error - "expected value after keyword, found end of list" (wrong error)

## Evidence from Test Suite

All existing tests in `crates/oxur-ast/tests/builder_stmt_tests.rs` use **keyword syntax only**:

```rust
// Example from test_build_stmt_semi_with_keyword_syntax
let input = r#"(Stmt
  :id 10
  :kind (Semi
          :expr (Expr  // <-- keyword syntax
                  :id 11
                  :kind (MacCall ...)))
  :span (Span))"#;
```

**No tests exist for positional syntax**, confirming this feature was never implemented or tested.

## Attempted Fix During Coverage Work

During coverage improvement attempts, I tried to create tests for the positional syntax:

```rust
#[test]
fn test_build_stmt_semi_with_positional_syntax() {
    let input = r#"(Stmt
      :kind (Semi (Expr :kind (MacCall ...)))
      :span (Span))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let stmt = builder.build_stmt(&sexp).unwrap();  // PANICS HERE
}
```

**Error:**
```
called `Result::unwrap()` on an `Err` value:
Expected {
    expected: "value after keyword",
    found: "end of list",
    pos: Position { offset: 18, line: 2, column: 13 }
}
```

This confirms the dead code is unreachable.

## Proposed Solutions

### Option 1: Remove Dead Code (Simple)

Remove lines 59-60 and 75-76, simplifying the code:

```rust
"Semi" => {
    let kwargs = parse_kwargs(list)?;
    if let Some(expr_sexp) = kwargs.get("expr") {
        let expr = self.build_expr(expr_sexp)?;
        Ok(StmtKind::Semi(expr))
    } else {
        Err(ParseError::Expected {
            expected: "expression".to_string(),
            found: "missing".to_string(),
            pos: list.pos,
        })
    }
}
```

**Pros:**
- Clean, honest code
- Matches actual behavior
- Improves coverage from 93.65% to ~97%

**Cons:**
- Removes future extensibility (if positional syntax was planned)

### Option 2: Fix `parse_kwargs` to Support Mixed Syntax (Complex)

Modify `parse_kwargs` to allow positional arguments:

```rust
pub fn parse_kwargs(list: &List) -> Result<std::collections::HashMap<String, &SExp>> {
    let mut map = std::collections::HashMap::new();
    let mut i = 1;

    while i < list.elements.len() {
        // Try to parse as keyword
        if let Ok(key) = expect_keyword(&list.elements[i]) {
            // It's a keyword - get its value
            if i + 1 >= list.elements.len() {
                return Err(ParseError::Expected {
                    expected: "value after keyword".to_string(),
                    found: "end of list".to_string(),
                    pos: list.pos,
                });
            }
            map.insert(key.name.clone(), &list.elements[i + 1]);
            i += 2;
        } else {
            // Not a keyword - stop parsing (allow positional args to remain)
            break;
        }
    }

    Ok(map)
}
```

**Pros:**
- Enables positional syntax
- Makes dead code reachable
- More flexible parsing

**Cons:**
- Changes fundamental parsing behavior
- May break existing code that relies on strict keyword parsing
- Requires extensive testing across all builders
- More complex logic

### Option 3: Document and Ignore (Current Approach)

Add comments explaining the dead code and accept 93.65% coverage:

```rust
} else if list.elements.len() > 1 {
    // NOTE: This branch is currently unreachable because parse_kwargs()
    // requires ALL elements to be keyword-value pairs. Positional syntax
    // support was planned but never completed. Consider removing or
    // fixing parse_kwargs() to support mixed syntax.
    let expr = self.build_expr(&list.elements[1])?;
    Ok(StmtKind::Semi(expr))
}
```

**Pros:**
- No behavior changes
- Documents the issue for future developers
- Minimal effort

**Cons:**
- Dead code remains
- Coverage stays below 95%

## Impact Assessment

### Current State

| Metric | Value |
|--------|-------|
| stmt.rs Coverage | 93.65% |
| Total Lines | 119 |
| Uncovered Lines | 23 |
| Dead Code Lines | 4 |
| Error Branches | 19 |

### If Dead Code Removed

| Metric | Value |
|--------|-------|
| stmt.rs Coverage | ~97% (estimated) |
| Total Lines | 115 |
| Uncovered Lines | ~19 |
| Dead Code Lines | 0 |
| Error Branches | 19 |

**Note:** Remaining uncovered lines would be error branches (^0 markers), which are hard to trigger without error injection.

## Recommendations

1. **Short-term:** Add documentation comments explaining the dead code (Option 3)
2. **Medium-term:** Evaluate if positional syntax is a desired feature
3. **Long-term:** Either:
   - Remove dead code if feature not needed (Option 1)
   - Implement properly with `parse_kwargs` fix (Option 2)

## Related Files

- `crates/oxur-ast/src/builder/stmt.rs` - Contains dead code
- `crates/oxur-ast/src/builder/helpers.rs` - Contains restrictive `parse_kwargs`
- `crates/oxur-ast/tests/builder_stmt_tests.rs` - Test suite (all keyword syntax)
- `crates/oxur-ast/src/builder/expr.rs` - Similar pattern may exist
- `crates/oxur-ast/src/builder/item.rs` - Similar pattern may exist

## Questions for Investigation

1. Was positional syntax ever intended to work?
2. Are there other builders with similar dead code?
3. Is there documentation specifying the S-expression syntax?
4. Would enabling positional syntax break existing code?
5. Should the AST builder support multiple syntax styles?

## Conclusion

The dead code represents an incomplete or abandoned feature for positional syntax in statement building. The current implementation strictly requires keyword syntax, making the positional fallback unreachable.

The code is well-intentioned but fundamentally blocked by the `parse_kwargs` design. A decision should be made to either remove this code or properly implement the feature with a more flexible keyword parser.

For now, documenting the issue and accepting 93.65% coverage for this file is reasonable, as the remaining uncovered code consists of legitimate dead code and error branches that would require significant refactoring to test.
