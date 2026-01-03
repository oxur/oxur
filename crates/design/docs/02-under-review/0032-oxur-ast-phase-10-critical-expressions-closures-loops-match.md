---
number: 32
title: "oxur-ast Phase 10: Critical Expressions (Closures, Loops, Match)"
author: "testing round"
created: 2026-01-03
updated: 2026-01-03
state: Under Review
supersedes: null
superseded-by: null
version: 1.0
---

# oxur-ast Phase 10: Critical Expressions (Closures, Loops, Match)

**Status**: Planned
**Estimated Effort**: 3-4 days
**Expected Coverage Gain**: +20% (from ~70% to ~90%)
**Complexity**: Medium-High - Complex AST structures, need careful handling
**Prerequisite**: Phase 9 must be completed first

## Overview

This phase implements the most critical missing expression types that appear in the majority of real Rust code. These features are essential for parsing iterator chains, control flow, and pattern matching - the backbone of idiomatic Rust.

**Impact**: After this phase, oxur-ast will be able to parse ~90% of typical Rust programs, including most code from the Rust standard library and popular crates.

## Goals

1. Implement Closure expressions - **Critical** (blocks 95% of iterator code)
2. Implement Match expressions - **Critical** (core Rust pattern)
3. Implement ForLoop expressions - **Critical** (most common iteration)
4. Implement Loop expressions - **Important** (infinite loops)
5. Implement While expressions - **Important** (conditional loops)

## Current State

All these types are already defined in `crates/oxur-ast/src/ast/expr.rs`:

```rust
// Already defined:
Closure {
    params: Vec<Param>,
    body: Box<Expr>,
}

Match {
    expr: Box<Expr>,
    arms: Vec<Arm>,
}

ForLoop {
    label: Option<Label>,
    pat: Pat,
    iter: Box<Expr>,
    body: Block,
}

Loop {
    label: Option<Label>,
    body: Block,
}

While {
    label: Option<Label>,
    cond: Box<Expr>,
    body: Block,
}
```

## Detailed Tasks

### Task 1: Closure Expressions (HIGHEST PRIORITY)

**Why Critical**: Closures appear in 95% of modern Rust code, especially with iterators.

**File**: `crates/oxur-ast/src/integration/from_syn.rs`

#### 1.1: Understand syn::ExprClosure Structure

```rust
// syn definition (for reference):
pub struct ExprClosure {
    pub attrs: Vec<Attribute>,
    pub lifetimes: Option<BoundLifetimes>,  // for<'a>
    pub constness: Option<Const>,
    pub movability: Option<Static>,         // static ||
    pub asyncness: Option<Async>,           // async ||
    pub capture: Option<Move>,              // move ||
    pub or1_token: Token![|],
    pub inputs: Punctuated<Pat, Token![,]>,
    pub or2_token: Token![|],
    pub output: ReturnType,
    pub body: Box<Expr>,
}
```

#### 1.2: Implement Conversion

```rust
syn::Expr::Closure(expr_closure) => {
    // Convert parameters (closure arguments)
    let params = expr_closure.inputs
        .iter()
        .map(|input| {
            // Closures use patterns as parameters
            let pat = self.convert_pat(input)?;

            // For closures, we create a Param with the pattern
            // If there's a type annotation in the pattern (Pat::Type),
            // it will already be handled by convert_pat
            Ok(Param {
                attrs: vec![],
                ty: Ty {
                    id: self.next_id(),
                    kind: TyKind::Infer,  // Closures often infer types
                    span: Span::DUMMY,
                    tokens: None,
                },
                pat,
                id: self.next_id(),
                span: Span::DUMMY,
                is_placeholder: false,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // Convert the body
    let body = Box::new(self.convert_expr(&expr_closure.body)?);

    ExprKind::Closure { params, body }
}
```

**Test Cases**:

```rust
// Simple closure
let double = |x| x * 2;

// Multiple parameters
let add = |a, b| a + b;

// With type annotations
let parse = |s: &str| s.parse::<i32>().ok();

// Block body
let process = |x| {
    let doubled = x * 2;
    doubled + 1
};

// Iterator chains (MOST COMMON USE CASE)
let result = items
    .iter()
    .filter(|x| x > &0)
    .map(|x| x * 2)
    .collect::<Vec<_>>();

// With move
let captured = String::from("hello");
let closure = move || println!("{}", captured);
```

**Note**: We're ignoring `movability`, `asyncness`, `capture`, and `lifetimes` for now. These can be added in later phases if needed.

### Task 2: Match Expressions (HIGH PRIORITY)

**Why Critical**: Core Rust pattern matching, used in ~80% of Rust code.

#### 2.1: Understand syn::ExprMatch and Arm Structure

```rust
// syn definitions (for reference):
pub struct ExprMatch {
    pub attrs: Vec<Attribute>,
    pub match_token: Token![match],
    pub expr: Box<Expr>,
    pub brace_token: token::Brace,
    pub arms: Vec<Arm>,
}

pub struct Arm {
    pub attrs: Vec<Attribute>,
    pub pat: Pat,
    pub guard: Option<(Token![if], Box<Expr>)>,
    pub fat_arrow_token: Token![=>],
    pub body: Box<Expr>,
    pub comma: Option<Token![,]>,
}
```

#### 2.2: Verify Arm Type in AST

**File**: `crates/oxur-ast/src/ast/expr.rs`

Check if `Arm` is defined. If not, add:

```rust
/// Match arm
#[derive(Debug, Clone, PartialEq)]
pub struct Arm {
    pub attrs: AttrVec,
    pub pat: Pat,
    pub guard: Option<Box<Expr>>,
    pub body: Box<Expr>,
    pub span: Span,
}
```

#### 2.3: Implement Conversion

```rust
syn::Expr::Match(expr_match) => {
    // Convert the expression being matched
    let expr = Box::new(self.convert_expr(&expr_match.expr)?);

    // Convert all arms
    let arms = expr_match.arms
        .iter()
        .map(|arm| {
            let pat = self.convert_pat(&arm.pat)?;

            let guard = arm.guard
                .as_ref()
                .map(|(_, expr)| self.convert_expr(expr))
                .transpose()?
                .map(Box::new);

            let body = Box::new(self.convert_expr(&arm.body)?);

            Ok(Arm {
                attrs: vec![],
                pat,
                guard,
                body,
                span: Span::DUMMY,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    ExprKind::Match { expr, arms }
}
```

**Test Cases**:

```rust
// Basic match
match value {
    0 => "zero",
    1 => "one",
    _ => "other",
}

// With patterns from Phase 9
match option {
    Some(x) => x,
    None => 0,
}

// With guards
match number {
    x if x < 0 => "negative",
    x if x > 0 => "positive",
    _ => "zero",
}

// Complex patterns
match point {
    Point { x: 0, y: 0 } => "origin",
    Point { x: 0, y } => format!("on y-axis at {}", y),
    Point { x, y: 0 } => format!("on x-axis at {}", x),
    Point { x, y } => format!("at ({}, {})", x, y),
}

// Nested match
match outer {
    Some(inner) => match inner {
        Ok(value) => value,
        Err(_) => 0,
    },
    None => 0,
}
```

### Task 3: ForLoop Expressions (HIGH PRIORITY)

**Why Critical**: The most common iteration pattern in Rust.

#### 3.1: Implement Conversion

```rust
syn::Expr::ForLoop(expr_for) => {
    let label = expr_for.label
        .as_ref()
        .map(|l| self.convert_label(l));

    let pat = self.convert_pat(&expr_for.pat)?;
    let iter = Box::new(self.convert_expr(&expr_for.expr)?);
    let body = self.convert_block(&expr_for.body)?;

    ExprKind::ForLoop { label, pat, iter, body }
}
```

**Test Cases**:

```rust
// Basic iteration
for i in 0..10 {
    println!("{}", i);
}

// Iterator
for item in items.iter() {
    process(item);
}

// With pattern destructuring
for (key, value) in map.iter() {
    println!("{}: {}", key, value);
}

// With label
'outer: for x in xs {
    for y in ys {
        if condition {
            break 'outer;
        }
    }
}

// Mutable iteration
for item in items.iter_mut() {
    *item += 1;
}

// Consuming iteration
for item in items.into_iter() {
    consume(item);
}
```

### Task 4: Loop Expressions

#### 4.1: Implement Conversion

```rust
syn::Expr::Loop(expr_loop) => {
    let label = expr_loop.label
        .as_ref()
        .map(|l| self.convert_label(l));

    let body = self.convert_block(&expr_loop.body)?;

    ExprKind::Loop { label, body }
}
```

**Test Cases**:

```rust
// Infinite loop with break
loop {
    if done {
        break;
    }
    work();
}

// Loop with value
let result = loop {
    counter += 1;
    if counter == 10 {
        break counter * 2;
    }
};

// Nested loops with labels
'outer: loop {
    'inner: loop {
        if condition1 {
            break 'outer;
        }
        if condition2 {
            break 'inner;
        }
    }
}
```

### Task 5: While Expressions

#### 5.1: Implement Conversion

```rust
syn::Expr::While(expr_while) => {
    let label = expr_while.label
        .as_ref()
        .map(|l| self.convert_label(l));

    let cond = Box::new(self.convert_expr(&expr_while.cond)?);
    let body = self.convert_block(&expr_while.body)?;

    ExprKind::While { label, cond, body }
}
```

**Test Cases**:

```rust
// Basic while
while count < 10 {
    count += 1;
}

// With complex condition
while !done && attempts < max_attempts {
    try_operation();
    attempts += 1;
}

// With label
'waiting: while !ready {
    if timeout() {
        break 'waiting;
    }
    sleep(100);
}
```

### Task 6: Update Code Generators

The generators should already handle these types, but verify and update if needed.

#### 6.1: Verify Rust Code Generator

**File**: `crates/oxur-ast/src/gen_rs/expr.rs`

Check if there are implementations for:

- `ExprKind::Closure`
- `ExprKind::Match`
- `ExprKind::ForLoop`
- `ExprKind::Loop`
- `ExprKind::While`

If missing, add them:

```rust
// In impl RustCodegen
fn generate_expr(&mut self, expr: &Expr) -> String {
    match &expr.kind {
        // ... existing cases ...

        ExprKind::Closure { params, body } => {
            let params_str = params
                .iter()
                .map(|p| self.generate_param(p))
                .collect::<Vec<_>>()
                .join(", ");

            let body_str = self.generate_expr(body);

            format!("|{}| {}", params_str, body_str)
        }

        ExprKind::Match { expr, arms } => {
            let expr_str = self.generate_expr(expr);
            let arms_str = arms
                .iter()
                .map(|arm| {
                    let pat = self.generate_pat(&arm.pat);
                    let guard = arm.guard
                        .as_ref()
                        .map(|g| format!(" if {}", self.generate_expr(g)))
                        .unwrap_or_default();
                    let body = self.generate_expr(&arm.body);
                    format!("{}{} => {}", pat, guard, body)
                })
                .collect::<Vec<_>>()
                .join(",\n");

            format!("match {} {{\n{}\n}}", expr_str, arms_str)
        }

        ExprKind::ForLoop { label, pat, iter, body } => {
            let label_str = label
                .as_ref()
                .map(|l| format!("{}: ", self.generate_label(l)))
                .unwrap_or_default();

            let pat_str = self.generate_pat(pat);
            let iter_str = self.generate_expr(iter);
            let body_str = self.generate_block(body);

            format!("{}for {} in {} {}", label_str, pat_str, iter_str, body_str)
        }

        ExprKind::Loop { label, body } => {
            let label_str = label
                .as_ref()
                .map(|l| format!("{}: ", self.generate_label(l)))
                .unwrap_or_default();

            let body_str = self.generate_block(body);

            format!("{}loop {}", label_str, body_str)
        }

        ExprKind::While { label, cond, body } => {
            let label_str = label
                .as_ref()
                .map(|l| format!("{}: ", self.generate_label(l)))
                .unwrap_or_default();

            let cond_str = self.generate_expr(cond);
            let body_str = self.generate_block(body);

            format!("{}while {} {}", label_str, cond_str, body_str)
        }

        // ... rest ...
    }
}

fn generate_label(&mut self, label: &Label) -> String {
    format!("'{}", label.name.name)
}
```

#### 6.2: Verify S-Expression Generator

**File**: `crates/oxur-ast/src/gen_sexp/expr.rs`

Similar additions for S-expression generation.

## Testing Strategy

### Test File Structure

**File**: `crates/oxur-ast/tests/phase10_closures_tests.rs`

```rust
use oxur_ast::*;

#[test]
fn test_simple_closure() {
    let code = r#"
        fn main() {
            let double = |x| x * 2;
            let result = double(5);
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
    // Verify closure structure
}

#[test]
fn test_closure_in_iterator() {
    let code = r#"
        fn main() {
            let numbers = vec![1, 2, 3, 4, 5];
            let doubled: Vec<_> = numbers
                .iter()
                .map(|x| x * 2)
                .collect();
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
    // This is the CRITICAL test - most common pattern
}

#[test]
fn test_closure_with_multiple_params() {
    let code = r#"
        fn main() {
            let add = |a, b| a + b;
            let sum = add(3, 4);
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_closure_with_block_body() {
    let code = r#"
        fn main() {
            let process = |x| {
                let doubled = x * 2;
                doubled + 1
            };
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}
```

**File**: `crates/oxur-ast/tests/phase10_match_tests.rs`

```rust
#[test]
fn test_basic_match() {
    let code = r#"
        fn main() {
            let x = 5;
            let desc = match x {
                0 => "zero",
                1 => "one",
                _ => "other",
            };
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_match_with_option() {
    let code = r#"
        fn main() {
            let opt = Some(42);
            let value = match opt {
                Some(x) => x,
                None => 0,
            };
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_match_with_guards() {
    let code = r#"
        fn main() {
            match number {
                x if x < 0 => println!("negative"),
                x if x > 0 => println!("positive"),
                _ => println!("zero"),
            }
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}
```

**File**: `crates/oxur-ast/tests/phase10_loops_tests.rs`

```rust
#[test]
fn test_for_loop() {
    let code = r#"
        fn main() {
            for i in 0..10 {
                println!("{}", i);
            }
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_for_loop_with_iterator() {
    let code = r#"
        fn main() {
            let items = vec![1, 2, 3];
            for item in items.iter() {
                println!("{}", item);
            }
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_infinite_loop() {
    let code = r#"
        fn main() {
            let mut count = 0;
            loop {
                count += 1;
                if count > 10 {
                    break;
                }
            }
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_while_loop() {
    let code = r#"
        fn main() {
            let mut count = 0;
            while count < 10 {
                count += 1;
            }
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_labeled_loops() {
    let code = r#"
        fn main() {
            'outer: for x in 0..10 {
                'inner: for y in 0..10 {
                    if x == y {
                        break 'outer;
                    }
                }
            }
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}
```

### Real-World Integration Tests

**File**: `crates/oxur-ast/tests/phase10_real_world_tests.rs`

```rust
/// Test parsing real iterator chains
#[test]
fn test_iterator_chain() {
    let code = r#"
        fn process_data(items: Vec<i32>) -> Vec<i32> {
            items
                .iter()
                .filter(|x| **x > 0)
                .map(|x| x * 2)
                .collect()
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
    // Verify the entire chain parses
}

/// Test parsing Option handling (very common pattern)
#[test]
fn test_option_pattern() {
    let code = r#"
        fn get_value(opt: Option<i32>) -> i32 {
            match opt {
                Some(x) => x,
                None => 0,
            }
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

/// Test parsing Result handling
#[test]
fn test_result_pattern() {
    let code = r#"
        fn handle_result(res: Result<i32, String>) -> i32 {
            match res {
                Ok(value) => value,
                Err(msg) => {
                    eprintln!("Error: {}", msg);
                    0
                }
            }
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}
```

### Round-Trip Tests

Ensure parsing is accurate by testing round-trips:

```rust
#[test]
fn test_closure_round_trip() {
    let code = "|x| x * 2";
    let ast = parse_expression(code).unwrap();
    let generated = generate_rust(&ast);
    let ast2 = parse_expression(&generated).unwrap();

    assert_eq!(ast, ast2);
}
```

## Success Criteria

- ✅ All 5 expression types parse without errors
- ✅ Iterator chains with closures work
- ✅ Match expressions with all pattern types from Phase 9 work
- ✅ Labeled loops work correctly
- ✅ Round-trip tests pass for all new features
- ✅ Can parse 90%+ of typical Rust code
- ✅ All Phase 10 tests pass (minimum 30 new tests)

## Files to Modify

1. **Primary**:
   - `crates/oxur-ast/src/integration/from_syn.rs` - All conversion logic (~200 lines added)

2. **Verification** (check if already exist):
   - `crates/oxur-ast/src/ast/expr.rs` - Verify Arm type exists
   - `crates/oxur-ast/src/gen_rs/expr.rs` - Add/verify generators
   - `crates/oxur-ast/src/gen_sexp/expr.rs` - Add/verify generators

3. **Testing**:
   - `crates/oxur-ast/tests/phase10_closures_tests.rs` - New
   - `crates/oxur-ast/tests/phase10_match_tests.rs` - New
   - `crates/oxur-ast/tests/phase10_loops_tests.rs` - New
   - `crates/oxur-ast/tests/phase10_real_world_tests.rs` - New

## Common Pitfalls

1. **Closure Parameters**: Closures use patterns, not typed parameters. Make sure to convert correctly.

2. **Match Arms**: Each arm has an optional guard (`if` condition). Don't forget to handle it.

3. **Label Lifetime Syntax**: Labels use lifetime syntax (`'label`) but aren't actually lifetimes. Make sure the parser doesn't confuse them.

4. **Loop Break Values**: `break` can have a value (`break 42`). Handle both cases.

5. **Iterator Chains**: Nested closures in method calls are very common. Test thoroughly.

## Performance Considerations

- Closures in iterator chains create deeply nested ASTs
- Match expressions with many arms can be large
- Consider adding position tracking for better error messages

## Dependencies

- ✅ Phase 9 must be completed (patterns are essential for match)
- ✅ All AST types already exist
- ⚠️ Generators may need updates for new types

## Next Phase

After Phase 10, proceed to **Phase 11: Literals and Basic Patterns** which adds:

- Complete literal support (Bool, Float, Char, Byte, ByteStr)
- Additional pattern types
- Improved pattern matching completeness

---

**Estimated Timeline**:

- Day 1: Closures (6-8 hours) - Most complex
- Day 2: Match expressions (6-8 hours) - Many edge cases
- Day 3: Loop expressions (4-6 hours) - Simpler
- Day 4: Generator updates + Testing (4-6 hours)

**Total**: 3-4 days for experienced developer

## Verification Checklist

Before marking Phase 10 complete:

- [ ] Closures: Simple, multi-param, with types, with blocks
- [ ] Iterator chains: map, filter, collect patterns work
- [ ] Match: Basic patterns, Option, Result, guards, nested
- [ ] For loops: Range, iterator, pattern destructuring, labeled
- [ ] Loop: Infinite, with break value, labeled
- [ ] While: Condition, labeled
- [ ] All round-trip tests pass
- [ ] Can parse real Rust stdlib functions
- [ ] Test coverage > 95% for new code
