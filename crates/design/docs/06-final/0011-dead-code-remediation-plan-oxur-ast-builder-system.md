---
number: 11
title: "Dead Code Remediation Plan: oxur-ast Builder System"
author: "Duncan McGreggor"
component: AST
tags: [Maintenance, Refactoring]
created: 2025-12-27
updated: 2025-12-27
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# Dead Code Remediation Plan: oxur-ast Builder System

**Date:** 2025-12-26  
**Target:** `crates/oxur-ast/src/builder/` directory  
**Goal:** Remove unreachable dead code and improve test coverage to ~97%

## Background

During coverage analysis, we discovered unreachable "positional syntax" fallback code in the statement builder. This code was intended to support alternative syntax but was never completed. The `parse_kwargs` helper function's strict keyword-value pair enforcement makes these fallback branches unreachable.

**Key Finding:** The `parse_kwargs` function fails fast when encountering non-keyword elements, preventing any positional syntax fallback from ever executing.

## Current State

- **stmt.rs Coverage:** 93.65%
- **Dead Code Lines:** 4 (lines 59-60, 75-76 in `stmt.rs`)
- **Pattern:** Likely exists in other builder files (`expr.rs`, `item.rs`)

## Decision: Remove Dead Code (Clean Architecture Approach)

After analysis, we've decided to:
1. Remove all unreachable positional syntax fallback code
2. Document the strict keyword-only design in `parse_kwargs`
3. Simplify builder logic for clarity
4. Achieve ~97% coverage with honest, maintainable code

**Rationale:**
- No evidence this feature was ever needed or requested
- No tests exist for positional syntax (feature never worked)
- Simpler code is more maintainable
- Strict keyword-only syntax is clearer and more consistent

## Step-by-Step Remediation

### Phase 1: Discovery

#### 1.1 Find All Instances of the Pattern

Run this command to locate similar dead code patterns:

```bash
grep -n "else if list.elements.len() > 1" crates/oxur-ast/src/builder/*.rs
```

Expected files to check:
- `crates/oxur-ast/src/builder/stmt.rs` (confirmed)
- `crates/oxur-ast/src/builder/expr.rs` (suspected)
- `crates/oxur-ast/src/builder/item.rs` (suspected)

#### 1.2 Document Findings

Create a list of all files and line numbers with this pattern. For each instance, verify:
- [ ] Is it after a `parse_kwargs` call?
- [ ] Is it checking `list.elements.len() > 1`?
- [ ] Does it access `list.elements[1]` directly?

### Phase 2: Code Documentation

#### 2.1 Update `parse_kwargs` Documentation

**File:** `crates/oxur-ast/src/builder/helpers.rs`  
**Location:** Lines 73-96

Add clear documentation explaining the design choice:

```rust
/// Parse keyword-value pairs from a list
/// Returns a map of keyword name -> value
///
/// # Design Note
/// This function enforces strict keyword-value pair syntax.
/// All elements after the node type (index 0) must be keyword-value pairs.
/// 
/// **Mixed positional/keyword syntax is intentionally NOT supported.**
/// 
/// # Format
/// ```lisp
/// (NodeType :key1 value1 :key2 value2 ...)
/// ```
///
/// # Errors
/// - Returns error if odd number of elements after node type
/// - Returns error if non-keyword found where keyword expected
/// - Returns error if value is missing after a keyword
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

        let key = expect_keyword(&list.elements[i])?;
        let value = &list.elements[i + 1];

        map.insert(key.name.clone(), value);
        i += 2;
    }

    Ok(map)
}
```

### Phase 3: Remove Dead Code from stmt.rs

#### 3.1 Simplify `Semi` Statement Builder

**File:** `crates/oxur-ast/src/builder/stmt.rs`  
**Lines:** 52-68

**Before:**
```rust
"Semi" => {
    let kwargs = parse_kwargs(list)?;
    if let Some(expr_sexp) = kwargs.get("expr") {
        let expr = self.build_expr(expr_sexp)?;
        Ok(StmtKind::Semi(expr))
    } else if list.elements.len() > 1 {
        // Expression is the second element
        let expr = self.build_expr(&list.elements[1])?;  // 🪦 DEAD
        Ok(StmtKind::Semi(expr))                         // 🪦 DEAD
    } else {
        Err(ParseError::Expected {
            expected: "expression".to_string(),
            found: "missing".to_string(),
            pos: list.pos,
        })
    }
}
```

**After:**
```rust
"Semi" => {
    let kwargs = parse_kwargs(list)?;
    let expr_sexp = kwargs
        .get("expr")
        .ok_or_else(|| ParseError::Expected {
            expected: ":expr field".to_string(),
            found: "missing".to_string(),
            pos: list.pos,
        })?;
    let expr = self.build_expr(expr_sexp)?;
    Ok(StmtKind::Semi(expr))
}
```

#### 3.2 Simplify `Expr` Statement Builder

**File:** `crates/oxur-ast/src/builder/stmt.rs`  
**Lines:** 69-84

**Before:**
```rust
"Expr" => {
    let kwargs = parse_kwargs(list)?;
    if let Some(expr_sexp) = kwargs.get("expr") {
        let expr = self.build_expr(expr_sexp)?;
        Ok(StmtKind::Expr(expr))
    } else if list.elements.len() > 1 {
        let expr = self.build_expr(&list.elements[1])?;  // 🪦 DEAD
        Ok(StmtKind::Expr(expr))                         // 🪦 DEAD
    } else {
        Err(ParseError::Expected {
            expected: "expression".to_string(),
            found: "missing".to_string(),
            pos: list.pos,
        })
    }
}
```

**After:**
```rust
"Expr" => {
    let kwargs = parse_kwargs(list)?;
    let expr_sexp = kwargs
        .get("expr")
        .ok_or_else(|| ParseError::Expected {
            expected: ":expr field".to_string(),
            found: "missing".to_string(),
            pos: list.pos,
        })?;
    let expr = self.build_expr(expr_sexp)?;
    Ok(StmtKind::Expr(expr))
}
```

### Phase 4: Check and Fix Other Builders

#### 4.1 Examine `expr.rs`

**File:** `crates/oxur-ast/src/builder/expr.rs`

Search for similar patterns:
```bash
grep -A 5 -B 5 "else if list.elements.len() > 1" crates/oxur-ast/src/builder/expr.rs
```

If found, apply the same simplification pattern.

#### 4.2 Examine `item.rs`

**File:** `crates/oxur-ast/src/builder/item.rs`

Search for similar patterns:
```bash
grep -A 5 -B 5 "else if list.elements.len() > 1" crates/oxur-ast/src/builder/item.rs
```

If found, apply the same simplification pattern.

### Phase 5: Testing

#### 5.1 Run Existing Tests

All existing tests should pass without modification since they already use keyword syntax:

```bash
cd crates/oxur-ast
cargo test
```

Expected: **All tests pass** (they already use keyword syntax exclusively)

#### 5.2 Verify Error Handling Still Works

The existing tests already cover the error cases:
- `test_build_stmt_semi_missing_expr` - Tests missing `:expr` field
- `test_build_stmt_expr_missing_expr` - Tests missing `:expr` field

These tests should still pass and now hit the simplified error paths.

#### 5.3 Run Coverage Analysis

```bash
cargo tarpaulin --out Html --output-dir coverage
```

**Expected Results:**
- `stmt.rs`: ~97% coverage (up from 93.65%)
- Remaining uncovered: Error branches (^0 markers) that require error injection
- Overall: Improved coverage with cleaner code

### Phase 6: Documentation

#### 6.1 Update Test File Comments

**File:** `crates/oxur-ast/tests/builder_stmt_tests.rs`

Remove or update the note at line 225:

**Before:**
```rust
// Note: Lines 59-60 and 75-76 (positional syntax fallback) are unreachable
// because parse_kwargs() requires ALL elements to be keyword-value pairs.
// If a non-keyword element is encountered, parse_kwargs() fails before
// the positional fallback is checked. This appears to be incomplete/dead code.
```

**After:**
```rust
// Note: This test suite uses keyword syntax exclusively (e.g., :expr, :kind).
// The builder enforces strict keyword-value pair syntax via parse_kwargs().
// Positional syntax is not supported by design.
```

#### 6.2 Add Design Documentation

Consider creating a brief design doc:

**File:** `crates/oxur-ast/docs/builder-syntax.md`

```markdown
# AST Builder S-Expression Syntax

## Overview

The AST builder uses a strict keyword-value pair syntax for all node construction.

## Syntax Rules

1. **Node Type First:** `(NodeType ...)`
2. **Keyword-Value Pairs:** All fields use `:keyword value` format
3. **No Positional Arguments:** Mixed or positional syntax is not supported

## Examples

### Valid ✅
```lisp
(Stmt
  :id 10
  :kind (Semi :expr (Expr :kind (MacCall :path (Path ...))))
  :span (Span :lo 0 :hi 10))
```

### Invalid ❌
```lisp
; Positional syntax not supported
(Stmt 10 (Semi (Expr ...)) (Span 0 10))
```

## Rationale

- **Clarity:** Explicit field names make S-expressions self-documenting
- **Flexibility:** Fields can appear in any order
- **Robustness:** Missing fields produce clear error messages
- **Consistency:** One syntax style across entire codebase
```

### Phase 7: Commit

#### 7.1 Create Comprehensive Commit

```bash
git add crates/oxur-ast/src/builder/helpers.rs
git add crates/oxur-ast/src/builder/stmt.rs
git add crates/oxur-ast/src/builder/expr.rs  # if modified
git add crates/oxur-ast/src/builder/item.rs  # if modified
git add crates/oxur-ast/tests/builder_stmt_tests.rs

git commit -m "refactor: remove unreachable positional syntax fallback code

BREAKING CHANGE: None (feature was never functional)

- Remove dead code from stmt.rs (lines 59-60, 75-76)
- Simplify Semi and Expr statement builders
- Document strict keyword-only syntax in parse_kwargs
- Update test comments to reflect design decision
- Improve coverage from 93.65% to ~97%

The positional syntax fallback code was unreachable because parse_kwargs()
enforces strict keyword-value pair syntax. This cleanup removes incomplete
feature code and improves code clarity.

Refs: dead-code-analysis-stmt-builder.md"
```

## Verification Checklist

Before considering this work complete, verify:

- [ ] All dead code patterns found via grep
- [ ] `parse_kwargs` documentation updated
- [ ] `stmt.rs` simplified (Semi and Expr)
- [ ] `expr.rs` checked and fixed if needed
- [ ] `item.rs` checked and fixed if needed
- [ ] All tests pass: `cargo test`
- [ ] Coverage improved: `cargo tarpaulin`
- [ ] Test comments updated
- [ ] Changes committed with clear message
- [ ] Coverage report shows ~97% for stmt.rs

## Expected Outcomes

### Code Quality
- ✅ Cleaner, more maintainable code
- ✅ Honest implementation (code matches intent)
- ✅ Better documentation of design decisions
- ✅ Consistent syntax enforcement

### Coverage
- ✅ stmt.rs: 93.65% → ~97%
- ✅ Removed 4 lines of dead code
- ✅ Remaining uncovered: legitimate error branches
- ✅ Overall project coverage improved

### Maintenance
- ✅ Fewer lines to maintain
- ✅ Clear error messages
- ✅ Self-documenting code structure
- ✅ Future developers understand design choice

## Alternative Considered and Rejected

**Option: Implement Positional Syntax Support**

We explicitly chose NOT to implement positional syntax because:
1. No evidence of need (no issues, no requests)
2. Feature was incomplete for unknown duration
3. Would require extensive testing and validation
4. Adds complexity without clear benefit
5. Current keyword syntax is more maintainable

If positional syntax is needed in future, it should be:
- Properly designed and documented
- Fully tested from the start
- Considered as a feature request with clear use cases

## Questions or Issues?

If you encounter any issues during this remediation:

1. **Tests fail after changes:** Revert and investigate which test revealed a wrong assumption
2. **Coverage doesn't improve:** Check if there are additional dead code instances
3. **Pattern exists in many files:** Consider scripting the refactor
4. **Unsure about a pattern:** When in doubt, check if it's after `parse_kwargs(?)`

## Related Documents

- Original Analysis: `dead-code-analysis-stmt-builder.md`
- Coverage Report: `coverage/index.html` (after running tarpaulin)
- Test Suite: `crates/oxur-ast/tests/builder_stmt_tests.rs`
