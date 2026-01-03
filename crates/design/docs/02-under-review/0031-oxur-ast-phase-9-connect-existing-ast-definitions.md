---
number: 31
title: "oxur-ast Phase 9: Connect Existing AST Definitions"
author: "Duncan McGreggor"
created: 2026-01-03
updated: 2026-01-03
state: Under Review
supersedes: null
superseded-by: null
version: 1.0
---

# oxur-ast Phase 9: Connect Existing AST Definitions

**Status**: Planned
**Estimated Effort**: 1-2 days
**Expected Coverage Gain**: +40-50% (from ~50% to ~70%)
**Complexity**: Low - Most work is straightforward pattern matching

## Overview

This phase focuses on connecting AST types that are **already defined** in `crates/oxur-ast/src/ast/` but not yet implemented in the `from_syn.rs` conversion layer. This is "low-hanging fruit" that provides massive coverage gains with minimal new code.

**Key Insight**: The AST structures exist, the generators (gen_rs, gen_sexp) already handle them. We just need to add conversion logic from syn types.

## Goals

1. Connect 7-9 expression types already defined in ExprKind
2. Connect 12 pattern types already defined in PatKind
3. Achieve ~70% coverage of common Rust syntax
4. Enable parsing of most basic Rust programs

## Current State

### Expressions Already in AST but Not Connected

From `crates/oxur-ast/src/ast/expr.rs`:

- ✅ `Array(Vec<Expr>)` - Array literals `[1, 2, 3]`
- ✅ `Tuple(Vec<Expr>)` - Tuple literals `(a, b, c)`
- ✅ `Struct { path, fields }` - Struct literals `Point { x: 1, y: 2 }`
- ✅ `Assign { left, right }` - Assignment `x = 5`
- ✅ `Paren(Box<Expr>)` - Parenthesized expressions `(expr)`
- ✅ `Try(Box<Expr>)` - Try operator `expr?`
- ✅ `Cast { expr, ty }` - Type casts `x as i32`
- ✅ `Break { label, value }` - Break with optional value
- ✅ `Continue { label }` - Continue with optional label
- ✅ `Return { value }` - Return with optional value
- ✅ `Range { start, end, inclusive }` - Range expressions

### Patterns Already in AST but Not Connected

From `crates/oxur-ast/src/ast/pat.rs`:

- ✅ `Wild` - Wildcard pattern `_`
- ✅ `Struct { path, fields }` - Struct patterns `Point { x, y }`
- ✅ `TupleStruct { path, elems }` - Tuple struct patterns `Some(x)`
- ✅ `Tuple(Vec<Pat>)` - Tuple patterns `(a, b, c)`
- ✅ `Slice(Vec<Pat>)` - Slice patterns `[a, b, ..]`
- ✅ `Ref { pat, mutability }` - Reference patterns `&x`, `&mut x`
- ✅ `Lit(Expr)` - Literal patterns `42`, `"hello"`
- ✅ `Path { qself, path }` - Path patterns `None`, `Some`
- ✅ `Range { start, end, limits }` - Range patterns `1..=5`
- ✅ `Rest` - Rest pattern `..`
- ✅ `Paren(Box<Pat>)` - Parenthesized patterns `(pat)`
- ✅ `Or(Vec<Pat>)` - Or-patterns `A | B` (Phase 8 addition)

## Detailed Tasks

### Task 1: Expression Conversions (Priority Order)

**File**: `crates/oxur-ast/src/integration/from_syn.rs`
**Function**: `convert_expr()` (starting around line 609)

#### 1.1: Struct Literals (HIGH PRIORITY)

**Why**: Blocks most real code, very common pattern

```rust
syn::Expr::Struct(expr_struct) => {
    let path = self.convert_path(&expr_struct.path)?;

    // Convert fields
    let fields = expr_struct.fields
        .iter()
        .map(|field| {
            let ident = self.convert_ident(&field.member);
            let expr = self.convert_expr(&field.expr)?;
            Ok(ExprField {
                attrs: vec![],
                ident,
                expr,
                is_shorthand: false, // TODO: detect shorthand
                span: Span::DUMMY,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    ExprKind::Struct { path, fields }
}
```

**Test Case**:

```rust
let point = Point { x: 1, y: 2 };
let origin = Point { x: 0, y: 0 };
```

#### 1.2: Array Literals

```rust
syn::Expr::Array(expr_array) => {
    let elems = expr_array.elems
        .iter()
        .map(|e| self.convert_expr(e))
        .collect::<Result<Vec<_>>>()?;
    ExprKind::Array(elems)
}
```

**Test Case**:

```rust
let nums = [1, 2, 3, 4, 5];
let empty: Vec<i32> = [];
```

#### 1.3: Tuple Expressions

```rust
syn::Expr::Tuple(expr_tuple) => {
    let elems = expr_tuple.elems
        .iter()
        .map(|e| self.convert_expr(e))
        .collect::<Result<Vec<_>>>()?;
    ExprKind::Tuple(elems)
}
```

**Test Case**:

```rust
let coords = (x, y, z);
let pair = (1, "hello");
```

#### 1.4: Range Expressions

```rust
syn::Expr::Range(expr_range) => {
    let start = expr_range.start
        .as_ref()
        .map(|e| self.convert_expr(e))
        .transpose()?
        .map(Box::new);

    let end = expr_range.end
        .as_ref()
        .map(|e| self.convert_expr(e))
        .transpose()?
        .map(Box::new);

    let inclusive = matches!(expr_range.limits, syn::RangeLimits::Closed(_));

    ExprKind::Range { start, end, inclusive }
}
```

**Test Cases**:

```rust
for i in 0..10 { }      // Exclusive
for i in 0..=10 { }     // Inclusive
let range = ..5;        // RangeTo
let range = 5..;        // RangeFrom
```

#### 1.5: Assignment

```rust
syn::Expr::Assign(expr_assign) => {
    let left = Box::new(self.convert_expr(&expr_assign.left)?);
    let right = Box::new(self.convert_expr(&expr_assign.right)?);
    ExprKind::Assign { left, right }
}
```

**Test Case**:

```rust
x = 5;
point.x = 10;
arr[0] = 42;
```

#### 1.6: Parenthesized Expressions

```rust
syn::Expr::Paren(expr_paren) => {
    let inner = Box::new(self.convert_expr(&expr_paren.expr)?);
    ExprKind::Paren(inner)
}
```

**Test Case**:

```rust
let result = (a + b) * c;
```

#### 1.7: Try Operator

```rust
syn::Expr::Try(expr_try) => {
    let inner = Box::new(self.convert_expr(&expr_try.expr)?);
    ExprKind::Try(inner)
}
```

**Test Case**:

```rust
fn read_file() -> Result<String, Error> {
    let content = fs::read_to_string("file.txt")?;
    Ok(content)
}
```

#### 1.8: Type Cast

```rust
syn::Expr::Cast(expr_cast) => {
    let expr = Box::new(self.convert_expr(&expr_cast.expr)?);
    let ty = Box::new(self.convert_type(&expr_cast.ty)?);
    ExprKind::Cast { expr, ty }
}
```

**Test Case**:

```rust
let x = 5 as f64;
let ptr = addr as *const u8;
```

#### 1.9: Break, Continue, Return

```rust
syn::Expr::Break(expr_break) => {
    let label = expr_break.label
        .as_ref()
        .map(|l| self.convert_label(l));

    let value = expr_break.expr
        .as_ref()
        .map(|e| self.convert_expr(e))
        .transpose()?
        .map(Box::new);

    ExprKind::Break { label, value }
}

syn::Expr::Continue(expr_continue) => {
    let label = expr_continue.label
        .as_ref()
        .map(|l| self.convert_label(l));

    ExprKind::Continue { label }
}

syn::Expr::Return(expr_return) => {
    let value = expr_return.expr
        .as_ref()
        .map(|e| self.convert_expr(e))
        .transpose()?
        .map(Box::new);

    ExprKind::Return { value }
}
```

**Test Cases**:

```rust
fn example() -> i32 {
    return 42;
}

loop {
    if done { break; }
    if skip { continue; }
}

'outer: loop {
    loop {
        break 'outer;
    }
}
```

#### 1.10: Add Helper for Label Conversion

```rust
fn convert_label(&mut self, label: &syn::Label) -> Label {
    Label {
        name: self.convert_ident(&label.name),
    }
}
```

### Task 2: Pattern Conversions (Priority Order)

**File**: `crates/oxur-ast/src/integration/from_syn.rs`
**Function**: `convert_pat()` (starting around line 240)

#### 2.1: Wildcard Pattern (CRITICAL)

**Why**: Required for every match expression

```rust
syn::Pat::Wild(_) => {
    Ok(Pat {
        id: self.next_id(),
        kind: PatKind::Wild,
        span: Span::DUMMY,
        tokens: None,
    })
}
```

**Test Case**:

```rust
match value {
    Some(x) => x,
    _ => 0,  // Wildcard pattern
}
```

#### 2.2: Tuple Patterns

```rust
syn::Pat::Tuple(pat_tuple) => {
    let elems = pat_tuple.elems
        .iter()
        .map(|p| self.convert_pat(p))
        .collect::<Result<Vec<_>>>()?;

    Ok(Pat {
        id: self.next_id(),
        kind: PatKind::Tuple(elems),
        span: Span::DUMMY,
        tokens: None,
    })
}
```

**Test Case**:

```rust
let (x, y, z) = coords;
match pair {
    (0, y) => y,
    (x, 0) => x,
    (x, y) => x + y,
}
```

#### 2.3: Struct Patterns

```rust
syn::Pat::Struct(pat_struct) => {
    let path = self.convert_path(&pat_struct.path)?;

    let fields = pat_struct.fields
        .iter()
        .map(|field| {
            let ident = self.convert_ident(&field.member);
            let pat = self.convert_pat(&field.pat)?;
            Ok(PatField {
                attrs: vec![],
                ident,
                pat,
                is_shorthand: false, // TODO: detect shorthand
                span: Span::DUMMY,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(Pat {
        id: self.next_id(),
        kind: PatKind::Struct { path, fields, rest: false },
        span: Span::DUMMY,
        tokens: None,
    })
}
```

**Test Case**:

```rust
let Point { x, y } = point;
match point {
    Point { x: 0, y } => y,
    Point { x, y: 0 } => x,
}
```

#### 2.4: Tuple Struct Patterns

```rust
syn::Pat::TupleStruct(pat_tuple_struct) => {
    let path = self.convert_path(&pat_tuple_struct.path)?;

    let elems = pat_tuple_struct.elems
        .iter()
        .map(|p| self.convert_pat(p))
        .collect::<Result<Vec<_>>>()?;

    Ok(Pat {
        id: self.next_id(),
        kind: PatKind::TupleStruct { path, elems },
        span: Span::DUMMY,
        tokens: None,
    })
}
```

**Test Case**:

```rust
match option {
    Some(x) => x,
    None => 0,
}
```

#### 2.5: Path Patterns

```rust
syn::Pat::Path(pat_path) => {
    let qself = pat_path.qself
        .as_ref()
        .map(|qs| self.convert_qself(qs))
        .transpose()?;

    let path = self.convert_path(&pat_path.path)?;

    Ok(Pat {
        id: self.next_id(),
        kind: PatKind::Path { qself, path },
        span: Span::DUMMY,
        tokens: None,
    })
}
```

**Test Case**:

```rust
match value {
    None => (),
    Some(_) => (),
}
```

#### 2.6: Literal Patterns

```rust
syn::Pat::Lit(pat_lit) => {
    let expr = self.convert_expr_lit(&pat_lit.lit)?;

    Ok(Pat {
        id: self.next_id(),
        kind: PatKind::Lit(Box::new(expr)),
        span: Span::DUMMY,
        tokens: None,
    })
}
```

**Test Case**:

```rust
match x {
    0 => "zero",
    1 => "one",
    42 => "answer",
    _ => "other",
}
```

#### 2.7: Range Patterns

```rust
syn::Pat::Range(pat_range) => {
    let start = pat_range.start
        .as_ref()
        .map(|e| self.convert_expr(e))
        .transpose()?
        .map(Box::new);

    let end = pat_range.end
        .as_ref()
        .map(|e| self.convert_expr(e))
        .transpose()?
        .map(Box::new);

    let limits = match &pat_range.limits {
        syn::RangeLimits::HalfOpen(_) => RangeLimits::HalfOpen,
        syn::RangeLimits::Closed(_) => RangeLimits::Closed,
    };

    Ok(Pat {
        id: self.next_id(),
        kind: PatKind::Range { start, end, limits },
        span: Span::DUMMY,
        tokens: None,
    })
}
```

**Test Case**:

```rust
match age {
    0..=17 => "minor",
    18..=64 => "adult",
    65.. => "senior",
}
```

#### 2.8: Reference Patterns

```rust
syn::Pat::Reference(pat_ref) => {
    let mutability = if pat_ref.mutability.is_some() {
        Mutability::Mut
    } else {
        Mutability::Not
    };

    let pat = Box::new(self.convert_pat(&pat_ref.pat)?);

    Ok(Pat {
        id: self.next_id(),
        kind: PatKind::Ref { pat, mutability },
        span: Span::DUMMY,
        tokens: None,
    })
}
```

**Test Case**:

```rust
match &value {
    &x => println!("{}", x),
}

match &mut value {
    &mut ref mut x => *x += 1,
}
```

#### 2.9: Slice Patterns

```rust
syn::Pat::Slice(pat_slice) => {
    let elems = pat_slice.elems
        .iter()
        .map(|p| self.convert_pat(p))
        .collect::<Result<Vec<_>>>()?;

    Ok(Pat {
        id: self.next_id(),
        kind: PatKind::Slice(elems),
        span: Span::DUMMY,
        tokens: None,
    })
}
```

**Test Case**:

```rust
match slice {
    [] => "empty",
    [x] => "one element",
    [first, .., last] => "many",
}
```

#### 2.10: Or Patterns

```rust
syn::Pat::Or(pat_or) => {
    let cases = pat_or.cases
        .iter()
        .map(|p| self.convert_pat(p))
        .collect::<Result<Vec<_>>>()?;

    Ok(Pat {
        id: self.next_id(),
        kind: PatKind::Or(cases),
        span: Span::DUMMY,
        tokens: None,
    })
}
```

**Test Case**:

```rust
match value {
    Some(1) | Some(2) | Some(3) => "small",
    Some(x) if x > 100 => "large",
    _ => "other",
}
```

#### 2.11: Parenthesized Patterns

```rust
syn::Pat::Paren(pat_paren) => {
    let inner = Box::new(self.convert_pat(&pat_paren.pat)?);

    Ok(Pat {
        id: self.next_id(),
        kind: PatKind::Paren(inner),
        span: Span::DUMMY,
        tokens: None,
    })
}
```

#### 2.12: Rest Pattern

```rust
syn::Pat::Rest(_) => {
    Ok(Pat {
        id: self.next_id(),
        kind: PatKind::Rest,
        span: Span::DUMMY,
        tokens: None,
    })
}
```

**Test Case**:

```rust
let [first, .., last] = array;
```

### Task 3: Add Missing Helper Types

Some conversions require helper types that may not exist yet.

#### 3.1: Check for ExprField Definition

**File**: `crates/oxur-ast/src/ast/expr.rs`

Verify `ExprField` exists (it should from Phase 5):

```rust
pub struct ExprField {
    pub attrs: AttrVec,
    pub ident: Ident,
    pub expr: Expr,
    pub is_shorthand: bool,
    pub span: Span,
}
```

#### 3.2: Check for PatField Definition

**File**: `crates/oxur-ast/src/ast/pat.rs`

Verify `PatField` exists:

```rust
pub struct PatField {
    pub attrs: AttrVec,
    pub ident: Ident,
    pub pat: Pat,
    pub is_shorthand: bool,
    pub span: Span,
}
```

If missing, add it.

#### 3.3: Check for Label Type

**File**: `crates/oxur-ast/src/ast/expr.rs`

Verify `Label` exists:

```rust
pub struct Label {
    pub name: Ident,
}
```

If missing, add it.

#### 3.4: Check for RangeLimits Enum

**File**: `crates/oxur-ast/src/ast/pat.rs`

```rust
pub enum RangeLimits {
    HalfOpen,  // ..
    Closed,    // ..=
}
```

## Testing Strategy

### Test File Structure

Create comprehensive test files for each feature category:

**File**: `crates/oxur-ast/tests/phase9_expressions_tests.rs`

```rust
use oxur_ast::*;

#[test]
fn test_array_literal() {
    let code = r#"
        fn main() {
            let nums = [1, 2, 3, 4, 5];
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
    // Verify array expression exists
}

#[test]
fn test_tuple_expression() {
    let code = r#"
        fn main() {
            let pair = (1, "hello");
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
    // Verify tuple expression
}

#[test]
fn test_struct_literal() {
    let code = r#"
        struct Point { x: i32, y: i32 }

        fn main() {
            let p = Point { x: 1, y: 2 };
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
    // Verify struct literal
}

// Add tests for all 11 expression types
```

**File**: `crates/oxur-ast/tests/phase9_patterns_tests.rs`

```rust
#[test]
fn test_wildcard_pattern() {
    let code = r#"
        fn main() {
            match x {
                Some(v) => v,
                _ => 0,
            }
        }
    "#;

    // Note: This requires Match which is Phase 10
    // For now, test in function parameters
    let code = r#"
        fn ignore_value(_: i32) {}
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_tuple_pattern() {
    let code = r#"
        fn main() {
            let (x, y, z) = coords;
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

// Add tests for all 12 pattern types
```

### Real-World Testing

Test with actual Rust files:

```bash
# Should now parse successfully
./bin/aster to-ast crates/oxur-lang/src/lib.rs
./bin/aster to-ast crates/oxur-comp/src/lib.rs
```

## Success Criteria

- ✅ All 11 expression types parse without errors
- ✅ All 12 pattern types parse without errors
- ✅ Round-trip tests pass (Rust → AST → Rust → AST produces same result)
- ✅ Can parse 70%+ of simple Rust programs
- ✅ All Phase 9 tests pass (minimum 40 new tests)

## Files to Modify

1. **Primary**:
   - `crates/oxur-ast/src/integration/from_syn.rs` - All conversion logic

2. **Verification** (should already exist, check completeness):
   - `crates/oxur-ast/src/ast/expr.rs` - Verify all ExprKind variants exist
   - `crates/oxur-ast/src/ast/pat.rs` - Verify all PatKind variants exist
   - `crates/oxur-ast/src/gen_rs/expr.rs` - Verify generators handle all types
   - `crates/oxur-ast/src/gen_sexp/expr.rs` - Verify generators handle all types

3. **Testing**:
   - `crates/oxur-ast/tests/phase9_expressions_tests.rs` - New file
   - `crates/oxur-ast/tests/phase9_patterns_tests.rs` - New file
   - `crates/oxur-ast/tests/phase9_integration_tests.rs` - Real-world tests

## Common Pitfalls

1. **Shorthand Detection**: Detecting `Point { x }` vs `Point { x: x }` requires checking if field names match variable names

2. **Range Limits**: Be careful with `..` (HalfOpen) vs `..=` (Closed)

3. **Labels**: Remember to handle optional labels in break/continue/loops

4. **QSelf**: Path patterns may have qualified self (`<T as Trait>::Item`)

5. **Rest Patterns**: Only valid in slice/array contexts

## Dependencies

- ✅ None - All AST types already exist
- ✅ Generators already support these types
- ✅ Only need to add conversion logic

## Next Phase

After Phase 9, proceed to **Phase 10: Critical Expressions** which adds:

- Closures
- For loops
- Match expressions
- Loop/While

---

**Estimated Timeline**:

- Day 1: Expression conversions (6-8 hours)
- Day 2: Pattern conversions (6-8 hours)
- Testing: 2-4 hours

**Total**: 1-2 days for experienced developer
