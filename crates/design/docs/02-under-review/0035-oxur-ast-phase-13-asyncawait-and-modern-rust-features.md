---
number: 35
title: "oxur-ast Phase 13: Async/Await and Modern Rust Features"
author: "Duncan McGreggor"
created: 2026-01-03
updated: 2026-01-03
state: Under Review
supersedes: null
superseded-by: null
version: 1.0
---

# oxur-ast Phase 13: Async/Await and Modern Rust Features

**Status**: Planned
**Estimated Effort**: 2-3 days
**Expected Coverage Gain**: +2-3% (from ~95% to ~97-99%)
**Complexity**: Medium-High - Async requires careful handling
**Prerequisite**: Phases 9-12 must be completed

## Overview

This phase adds async/await support and completes the remaining modern Rust features. After this phase, oxur-ast will support ~99% of Rust syntax found in modern codebases, including async functions, await expressions, and edge-case patterns.

## Goals

1. Implement async blocks (`async { }`)
2. Implement await expressions (`.await`)
3. Implement async functions
4. Add remaining rarely-used expression types
5. Achieve 97-99% coverage

## Current State

Some types are defined, some need to be added.

### Async Support in AST

Check if these exist in `crates/oxur-ast/src/ast/`:

- `async` modifier on functions (should exist in FnHeader)
- Async blocks as expressions (may need to add)
- Await as expression operator

## Detailed Tasks

### Task 1: Async Blocks (HIGH PRIORITY)

**Why Important**: Core async Rust feature, increasingly common.

#### 1.1: Verify/Add Async Block to ExprKind

**File**: `crates/oxur-ast/src/ast/expr.rs`

Check if exists, if not add:

```rust
/// Async block: `async { ... }` or `async move { ... }`
Async {
    capture: CaptureBy,  // move or not
    body: Block,
}
```

Also need `CaptureBy` enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureBy {
    Value,  // move
    Ref,    // borrow
}
```

#### 1.2: Implement Conversion

**File**: `crates/oxur-ast/src/integration/from_syn.rs`

```rust
syn::Expr::Async(expr_async) => {
    let capture = if expr_async.capture.is_some() {
        CaptureBy::Value  // async move
    } else {
        CaptureBy::Ref    // async
    };

    let body = self.convert_block(&expr_async.block)?;

    ExprKind::Async { capture, body }
}
```

**Test Cases**:

```rust
// Basic async block
async {
    let result = fetch_data().await;
    process(result)
}

// Async move (captures by value)
let data = vec![1, 2, 3];
let future = async move {
    process_data(data).await
};

// Common in async functions
async fn example() -> Result<String, Error> {
    let data = async {
        fetch().await
    }.await;

    Ok(data)
}
```

### Task 2: Await Expressions (HIGH PRIORITY)

**Why Important**: Essential for async code, very common.

#### 2.1: Verify/Add Await to ExprKind

**File**: `crates/oxur-ast/src/ast/expr.rs`

Check if exists, if not add:

```rust
/// Await: `expr.await`
Await {
    expr: Box<Expr>,
}
```

#### 2.2: Implement Conversion

**File**: `crates/oxur-ast/src/integration/from_syn.rs`

```rust
syn::Expr::Await(expr_await) => {
    let expr = Box::new(self.convert_expr(&expr_await.base)?);
    ExprKind::Await { expr }
}
```

**Test Cases**:

```rust
// Basic await
let result = future.await;

// Chained awaits
let final_result = fetch()
    .await
    .process()
    .await
    .finalize()
    .await;

// With error handling
let data = fetch_data().await?;

// In match
match fetch().await {
    Ok(data) => process(data),
    Err(e) => handle_error(e),
}
```

### Task 3: Async Functions

**Note**: Async functions are just regular functions with an `async` modifier. This should already be supported if FnHeader has an asyncness field.

#### 3.1: Verify FnHeader Support

**File**: `crates/oxur-ast/src/ast/item.rs`

Check `FnHeader`:

```rust
pub struct FnHeader {
    pub safety: Safety,
    pub constness: Constness,
    pub asyncness: Async,      // Should exist
    pub ext: Option<Abi>,
    pub coroutine_kind: Option<CoroutineKind>,
}

pub enum Async {
    Yes(Span),
    No,
}
```

#### 3.2: Verify Conversion

**File**: `crates/oxur-ast/src/integration/from_syn.rs`

In `convert_fn_header()`, check:

```rust
fn convert_fn_header(&mut self, sig: &syn::Signature) -> Result<FnHeader> {
    // ... existing code ...

    let asyncness = if sig.asyncness.is_some() {
        Async::Yes(Span::DUMMY)
    } else {
        Async::No
    };

    Ok(FnHeader {
        safety,
        constness,
        asyncness,
        ext,
        coroutine_kind: None,
    })
}
```

**Test Cases**:

```rust
// Async function
async fn fetch_data() -> Result<String, Error> {
    let response = client.get("url").await?;
    Ok(response.text().await?)
}

// Async method
impl MyStruct {
    async fn process(&self) -> Result<(), Error> {
        self.data.process().await
    }
}

// Async trait methods (if supported)
trait AsyncTrait {
    async fn do_work(&self) -> Result<(), Error>;
}
```

### Task 4: Remaining Expression Types

These are rarely used but should be added for completeness.

#### 4.1: Let Expressions (if-let, while-let contexts)

**File**: `crates/oxur-ast/src/ast/expr.rs`

```rust
/// Let expression (in if-let, while-let)
Let {
    pat: Pat,
    expr: Box<Expr>,
}
```

**Conversion**:

```rust
syn::Expr::Let(expr_let) => {
    let pat = self.convert_pat(&expr_let.pat)?;
    let expr = Box::new(self.convert_expr(&expr_let.expr)?);
    ExprKind::Let { pat, expr }
}
```

**Test Cases**:

```rust
// if-let
if let Some(x) = option {
    process(x);
}

// while-let
while let Some(item) = iterator.next() {
    process(item);
}

// Match guard with let (Rust 1.65+)
match value {
    x if let Some(y) = complex(x) => y,
    _ => 0,
}
```

#### 4.2: Repeat Expressions

Array repeat syntax: `[0; 100]`

```rust
/// Array repeat: `[expr; count]`
Repeat {
    expr: Box<Expr>,
    count: Box<Expr>,
}
```

**Conversion**:

```rust
syn::Expr::Repeat(expr_repeat) => {
    let expr = Box::new(self.convert_expr(&expr_repeat.expr)?);
    let count = Box::new(self.convert_expr(&expr_repeat.len)?);
    ExprKind::Repeat { expr, count }
}
```

**Test Cases**:

```rust
let zeros = [0; 100];
let buffer = [0u8; 1024];
let grid = [[0; 10]; 10];  // 2D array
```

#### 4.3: Unsafe Blocks

```rust
/// Unsafe block: `unsafe { ... }`
Unsafe {
    body: Block,
}
```

**Conversion**:

```rust
syn::Expr::Unsafe(expr_unsafe) => {
    let body = self.convert_block(&expr_unsafe.block)?;
    ExprKind::Unsafe { body }
}
```

**Test Cases**:

```rust
unsafe {
    *ptr = 42;
}

let value = unsafe { *raw_pointer };
```

#### 4.4: Yield Expressions (Generators - unstable)

**Note**: Generators are unstable. Low priority.

```rust
/// Yield: `yield expr`
Yield {
    value: Option<Box<Expr>>,
}
```

**Conversion**:

```rust
syn::Expr::Yield(expr_yield) => {
    let value = expr_yield.expr
        .as_ref()
        .map(|e| self.convert_expr(e))
        .transpose()?
        .map(Box::new);

    ExprKind::Yield { value }
}
```

#### 4.5: Const Blocks

```rust
/// Const block: `const { ... }`
Const {
    body: Block,
}
```

**Conversion**:

```rust
syn::Expr::Const(expr_const) => {
    let body = self.convert_block(&expr_const.block)?;
    ExprKind::Const { body }
}
```

**Test Case**:

```rust
const VALUE: i32 = const { 2 + 2 };
```

### Task 5: Macro Items and Attributes (Low Priority)

If not yet supported, add macro definitions and attributes.

**File**: `crates/oxur-ast/src/ast/item.rs`

```rust
/// Macro definition: `macro_rules! name { ... }`
Macro {
    name: Ident,
    rules: MacroDef,
}
```

This is complex and may be deferred to a later phase.

### Task 6: Update Generators

**File**: `crates/oxur-ast/src/gen_rs/expr.rs`

```rust
fn generate_expr(&mut self, expr: &Expr) -> String {
    match &expr.kind {
        // ... existing cases ...

        ExprKind::Async { capture, body } => {
            let capture_str = match capture {
                CaptureBy::Value => " move",
                CaptureBy::Ref => "",
            };
            let body_str = self.generate_block(body);
            format!("async{} {}", capture_str, body_str)
        }

        ExprKind::Await { expr } => {
            format!("{}.await", self.generate_expr(expr))
        }

        ExprKind::Let { pat, expr } => {
            format!("let {} = {}", self.generate_pat(pat), self.generate_expr(expr))
        }

        ExprKind::Repeat { expr, count } => {
            format!("[{}; {}]", self.generate_expr(expr), self.generate_expr(count))
        }

        ExprKind::Unsafe { body } => {
            format!("unsafe {}", self.generate_block(body))
        }

        ExprKind::Yield { value } => {
            if let Some(v) = value {
                format!("yield {}", self.generate_expr(v))
            } else {
                "yield".to_string()
            }
        }

        ExprKind::Const { body } => {
            format!("const {}", self.generate_block(body))
        }
    }
}
```

## Testing Strategy

### Test Files

**File**: `crates/oxur-ast/tests/phase13_async_tests.rs`

```rust
#[test]
fn test_async_block() {
    let code = r#"
        fn main() {
            let future = async {
                42
            };
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_async_move_block() {
    let code = r#"
        fn main() {
            let data = vec![1, 2, 3];
            let future = async move {
                process(data)
            };
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_await_expression() {
    let code = r#"
        async fn example() -> Result<String, Error> {
            let result = fetch_data().await?;
            Ok(result)
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_async_function() {
    let code = r#"
        async fn fetch() -> Result<String, Error> {
            let response = client.get("url").await?;
            Ok(response)
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_complex_async_chain() {
    let code = r#"
        async fn complex() -> Result<Data, Error> {
            fetch()
                .await?
                .process()
                .await?
                .finalize()
                .await
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}
```

**File**: `crates/oxur-ast/tests/phase13_remaining_exprs_tests.rs`

```rust
#[test]
fn test_let_expression() {
    let code = r#"
        fn main() {
            if let Some(x) = option {
                println!("{}", x);
            }
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_array_repeat() {
    let code = r#"
        fn main() {
            let zeros = [0; 100];
            let buffer = [0u8; 1024];
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_unsafe_block() {
    let code = r#"
        fn main() {
            unsafe {
                *ptr = 42;
            }
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}
```

### Real-World Integration

Test with actual async Rust code:

```rust
#[test]
fn test_tokio_style_code() {
    let code = r#"
        #[tokio::main]
        async fn main() -> Result<(), Box<dyn std::error::Error>> {
            let response = reqwest::get("https://example.com")
                .await?
                .text()
                .await?;

            println!("{}", response);
            Ok(())
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}
```

## Success Criteria

- ✅ Async blocks parse correctly
- ✅ Await expressions work in all contexts
- ✅ Async functions parse with correct signature
- ✅ Let expressions in if-let/while-let work
- ✅ Array repeat syntax works
- ✅ Unsafe blocks parse
- ✅ Can parse real async/await code from popular crates
- ✅ 97-99% coverage achieved

## Files to Modify

1. **Primary**:
   - `crates/oxur-ast/src/integration/from_syn.rs` - All conversions

2. **AST** (add if missing):
   - `crates/oxur-ast/src/ast/expr.rs` - Add new ExprKind variants
   - `crates/oxur-ast/src/ast/item.rs` - Verify async support

3. **Generators**:
   - `crates/oxur-ast/src/gen_rs/expr.rs` - Expression generation
   - `crates/oxur-ast/src/gen_sexp/expr.rs` - S-expression generation

4. **Testing**:
   - `crates/oxur-ast/tests/phase13_async_tests.rs` - New
   - `crates/oxur-ast/tests/phase13_remaining_exprs_tests.rs` - New
   - `crates/oxur-ast/tests/phase13_real_world_async_tests.rs` - New

## Common Pitfalls

1. **Async Transform**: Async functions desugar to regular functions returning `impl Future`. Our AST keeps them as async.
2. **Await Precedence**: `.await` has high precedence but can be chained.
3. **Move Capture**: `async move` vs `async` changes capture semantics.
4. **Let in Guard**: `if let` in match guards is relatively new (Rust 1.65).
5. **Generators**: Yield is unstable, may not work in stable Rust.

## Dependencies

- ✅ All prior phases (9-12) must be complete
- ⚠️ May need to add ExprKind variants
- ⚠️ Async support in FnHeader should already exist

## Final Phase

This is the final phase for basic Rust support. After Phase 13:

- **Coverage**: 97-99% of Rust syntax
- **Can parse**: Virtually all Rust code in the wild
- **Remaining gaps**: Niche features, unstable syntax, macros 2.0

### Optional Future Work

- Full macro_rules! definition parsing
- Procedural macros (proc-macro)
- Inline assembly (`asm!`)
- Advanced pattern guards
- Exotic type system features

---

**Estimated Timeline**:

- Day 1: Async blocks + await (6-8 hours)
- Day 2: Async functions + remaining expressions (6-8 hours)
- Day 3: Generator updates + comprehensive testing (4-6 hours)

**Total**: 2-3 days for experienced developer

## Verification Checklist

Before marking Phase 13 complete:

- [ ] Async blocks: basic, move, nested
- [ ] Await: simple, chained, with error handling
- [ ] Async functions: standalone, methods, with complex bodies
- [ ] Let expressions: if-let, while-let
- [ ] Array repeat: various sizes and types
- [ ] Unsafe blocks: basic usage
- [ ] Real tokio/async-std code parses
- [ ] All round-trip tests pass
- [ ] Final coverage measurement >= 97%
- [ ] Can parse oxur's own source code completely

## Post-Phase 13: Validation

After completing Phase 13, validate against real codebases:

1. Parse all of `rust-lang/rust` stdlib
2. Parse popular crates: tokio, serde, axum, etc.
3. Measure exact coverage percentage
4. Document remaining gaps (if any)
5. Celebrate! 🎉

---

**End of Implementation Roadmap**

After Phase 13, oxur-ast should be able to parse nearly all real-world Rust code!
