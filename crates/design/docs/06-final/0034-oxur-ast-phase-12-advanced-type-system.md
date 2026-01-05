---
number: 34
title: "oxur-ast Phase 12: Advanced Type System"
author: "adding advanced"
created: 2026-01-03
updated: 2026-01-05
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# oxur-ast Phase 12: Advanced Type System

**Status**: Planned
**Estimated Effort**: 2-3 days
**Expected Coverage Gain**: +2-3% (from ~93% to ~95-97%)
**Complexity**: Medium - Type system details
**Prerequisite**: Phases 9-11 must be completed

## Overview

This phase completes the type system by adding advanced type features commonly used in modern Rust: impl Trait, trait objects (dyn), and bare function types. These features are essential for APIs, trait-based design, and function pointers.

## Goals

1. Implement `impl Trait` types (`impl Iterator<Item = i32>`)
2. Implement Trait Object types (`dyn Display + Send`)
3. Implement Bare Function types (`fn(i32) -> bool`)
4. Complete remaining type variants
5. Achieve 95-97% coverage

## Current State

From `crates/oxur-ast/src/ast/types.rs`, these are **already defined** but not connected:

```rust
/// Bare function type: `fn(i32) -> bool`
BareFn {
    safety: Safety,
    abi: Option<Abi>,
    generic_params: Vec<GenericParam>,
    inputs: Vec<BareFnArg>,
    output: FnRetTy,
}

/// Impl trait: `impl Iterator`
ImplTrait {
    bounds: Vec<GenericBound>,
}

/// Trait object: `dyn Display`
TraitObject {
    safety: Safety,  // unsafe dyn Trait
    bounds: Vec<GenericBound>,
}

/// Parenthesized type: `(T)`
Paren(Box<Ty>)

/// Macro in type position
Macro(MacCall)
```

## Detailed Tasks

### Task 1: Impl Trait Types (HIGH PRIORITY)

**Why Important**: Very common in modern Rust for return types and function parameters.

#### 1.1: Implementation

**File**: `crates/oxur-ast/src/integration/from_syn.rs`

```rust
syn::Type::ImplTrait(type_impl_trait) => {
    let bounds = type_impl_trait.bounds
        .iter()
        .map(|bound| self.convert_type_param_bound(bound))
        .collect::<Result<Vec<_>>>()?;

    TyKind::ImplTrait { bounds }
}
```

**Test Cases**:

```rust
// Return position
fn numbers() -> impl Iterator<Item = i32> {
    vec![1, 2, 3].into_iter()
}

// Parameter position (since Rust 1.26)
fn process(iter: impl Iterator<Item = i32>) {
    for item in iter {
        println!("{}", item);
    }
}

// Multiple bounds
fn complex() -> impl Display + Debug + Send {
    42
}
```

### Task 2: Trait Object Types (IMPORTANT)

**Why Important**: Essential for dynamic dispatch and trait objects.

#### 2.1: Implementation

```rust
syn::Type::TraitObject(type_trait_object) => {
    let safety = match type_trait_object.dyn_token {
        Some(_) => Safety::Default,
        None => Safety::Default,
    };

    let bounds = type_trait_object.bounds
        .iter()
        .map(|bound| self.convert_type_param_bound(bound))
        .collect::<Result<Vec<_>>>()?;

    TyKind::TraitObject { safety, bounds }
}
```

**Test Cases**:

```rust
// Basic trait object
let display: Box<dyn Display> = Box::new(42);

// Multiple bounds
let complex: Box<dyn Display + Debug + Send> = Box::new("hello");

// Lifetime bounds
let obj: &dyn Display + 'static = &42;

// Unsafe trait object (rare)
let unsafe_trait: Box<dyn UnsafeTrait> = ...;
```

### Task 3: Bare Function Types

**Why Important**: Function pointers, callbacks, FFI.

#### 3.1: Understand BareFnArg

**File**: `crates/oxur-ast/src/ast/types.rs`

Verify `BareFnArg` exists:

```rust
pub struct BareFnArg {
    pub attrs: AttrVec,
    pub name: Option<Ident>,
    pub ty: Ty,
}
```

#### 3.2: Implementation

```rust
syn::Type::BareFn(type_bare_fn) => {
    let safety = match type_bare_fn.unsafety {
        Some(_) => Safety::Unsafe(Span::DUMMY),
        None => Safety::Default,
    };

    let abi = type_bare_fn.abi
        .as_ref()
        .map(|abi_spec| {
            let name = abi_spec.name
                .as_ref()
                .map(|lit_str| lit_str.value())
                .unwrap_or_else(|| "Rust".to_string());
            Abi { name }
        });

    // Generic params (for<'a>)
    let generic_params = if let Some(bound_lifetimes) = &type_bare_fn.lifetimes {
        bound_lifetimes.lifetimes
            .iter()
            .map(|lifetime_def| {
                let lifetime = self.convert_lifetime(&lifetime_def.lifetime)?;
                Ok(GenericParam::Lifetime {
                    attrs: vec![],
                    lifetime,
                    bounds: vec![],
                })
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        vec![]
    };

    // Input parameters
    let inputs = type_bare_fn.inputs
        .iter()
        .map(|bare_fn_arg| {
            let name = bare_fn_arg.name
                .as_ref()
                .map(|(ident, _)| self.convert_ident(ident));

            let ty = self.convert_type(&bare_fn_arg.ty)?;

            Ok(BareFnArg {
                attrs: vec![],
                name,
                ty,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // Return type
    let output = self.convert_return_type(&type_bare_fn.output)?;

    TyKind::BareFn {
        safety,
        abi,
        generic_params,
        inputs,
        output,
    }
}
```

**Test Cases**:

```rust
// Simple function pointer
type Callback = fn(i32) -> bool;

// With named parameters
type Operation = fn(x: i32, y: i32) -> i32;

// Unsafe function pointer
type UnsafeCallback = unsafe fn(*const u8) -> i32;

// FFI function pointer
type CCallback = extern "C" fn(i32) -> i32;

// Higher-rank trait bounds
type HigherRank = for<'a> fn(&'a str) -> &'a str;

// Complex example
static OPERATION: fn(i32, i32) -> i32 = |a, b| a + b;
```

### Task 4: Parenthesized Types

Simple wrapper for clarity in type expressions.

```rust
syn::Type::Paren(type_paren) => {
    let inner = Box::new(self.convert_type(&type_paren.elem)?);
    TyKind::Paren(inner)
}
```

**Test Cases**:

```rust
type Complex = (Box<dyn Display>);
type Nested = ((i32));
```

### Task 5: Macro Types

Types that are the result of macro expansion.

```rust
syn::Type::Macro(type_macro) => {
    let mac = self.convert_macro(&type_macro.mac)?;
    TyKind::Macro(mac)
}
```

**Test Cases**:

```rust
type Generated = vec![i32];  // vec! macro in type position
```

### Task 6: Helper Function for Type Param Bounds

**File**: `crates/oxur-ast/src/integration/from_syn.rs`

Add if not exists:

```rust
fn convert_type_param_bound(&mut self, bound: &syn::TypeParamBound) -> Result<GenericBound> {
    match bound {
        syn::TypeParamBound::Trait(trait_bound) => {
            let modifier = match trait_bound.modifier {
                syn::TraitBoundModifier::None => TraitBoundModifier::None,
                syn::TraitBoundModifier::Maybe(_) => TraitBoundModifier::Maybe,
            };

            let path = self.convert_path(&trait_bound.path)?;

            Ok(GenericBound::Trait {
                modifier,
                path,
                poly_trait: None, // Simplified for now
            })
        }
        syn::TypeParamBound::Lifetime(lifetime) => {
            let lifetime = self.convert_lifetime(lifetime)?;
            Ok(GenericBound::Lifetime(lifetime))
        }
        _ => Err(ParseError::Expected {
            expected: "trait or lifetime bound".to_string(),
            found: "other bound".to_string(),
            pos: Position::new(0, 1, 1),
        }),
    }
}

fn convert_lifetime(&mut self, lifetime: &syn::Lifetime) -> Result<Lifetime> {
    Ok(Lifetime {
        id: self.next_id(),
        ident: Ident::new(&lifetime.ident.to_string(), Span::DUMMY),
    })
}
```

### Task 7: Update Generators

Update generators to handle new type variants.

#### 7.1: Rust Generator

**File**: `crates/oxur-ast/src/gen_rs/types.rs`

```rust
fn generate_type(&mut self, ty: &Ty) -> String {
    match &ty.kind {
        // ... existing cases ...

        TyKind::ImplTrait { bounds } => {
            let bounds_str = bounds
                .iter()
                .map(|b| self.generate_bound(b))
                .collect::<Vec<_>>()
                .join(" + ");
            format!("impl {}", bounds_str)
        }

        TyKind::TraitObject { safety, bounds } => {
            let safety_str = match safety {
                Safety::Unsafe(_) => "unsafe ",
                _ => "",
            };

            let bounds_str = bounds
                .iter()
                .map(|b| self.generate_bound(b))
                .collect::<Vec_>>()
                .join(" + ");

            format!("{}dyn {}", safety_str, bounds_str)
        }

        TyKind::BareFn { safety, abi, generic_params, inputs, output } => {
            let safety_str = match safety {
                Safety::Unsafe(_) => "unsafe ",
                _ => "",
            };

            let abi_str = abi
                .as_ref()
                .map(|a| format!("extern \"{}\" ", a.name))
                .unwrap_or_default();

            let params_str = if !generic_params.is_empty() {
                let params = generic_params
                    .iter()
                    .map(|p| self.generate_generic_param(p))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("for<{}> ", params)
            } else {
                String::new()
            };

            let inputs_str = inputs
                .iter()
                .map(|arg| {
                    let name = arg.name
                        .as_ref()
                        .map(|n| format!("{}: ", n.name))
                        .unwrap_or_default();
                    format!("{}{}", name, self.generate_type(&arg.ty))
                })
                .collect::<Vec<_>>()
                .join(", ");

            let output_str = self.generate_return_type(output);

            format!("{}{}{}fn({}){}", safety_str, abi_str, params_str, inputs_str, output_str)
        }

        TyKind::Paren(inner) => {
            format!("({})", self.generate_type(inner))
        }

        TyKind::Macro(mac) => {
            self.generate_macro_call(mac)
        }
    }
}

fn generate_bound(&mut self, bound: &GenericBound) -> String {
    match bound {
        GenericBound::Trait { modifier, path, .. } => {
            let modifier_str = match modifier {
                TraitBoundModifier::Maybe => "?",
                _ => "",
            };
            format!("{}{}", modifier_str, self.generate_path(path))
        }
        GenericBound::Lifetime(lifetime) => {
            format!("'{}", lifetime.ident.name)
        }
        _ => "".to_string(),
    }
}
```

## Testing Strategy

### Test Files

**File**: `crates/oxur-ast/tests/phase12_impl_trait_tests.rs`

```rust
#[test]
fn test_impl_trait_return() {
    let code = r#"
        fn numbers() -> impl Iterator<Item = i32> {
            vec![1, 2, 3].into_iter()
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_impl_trait_parameter() {
    let code = r#"
        fn process(iter: impl Iterator<Item = i32>) {
            for item in iter {
                println!("{}", item);
            }
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}
```

**File**: `crates/oxur-ast/tests/phase12_trait_object_tests.rs`

```rust
#[test]
fn test_trait_object() {
    let code = r#"
        fn main() {
            let display: Box<dyn Display> = Box::new(42);
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}
```

**File**: `crates/oxur-ast/tests/phase12_bare_fn_tests.rs`

```rust
#[test]
fn test_function_pointer() {
    let code = r#"
        type Callback = fn(i32) -> bool;

        fn main() {
            let cb: Callback = |x| x > 0;
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_ffi_function_pointer() {
    let code = r#"
        type CCallback = extern "C" fn(i32) -> i32;
    "#;

    let ast = parse_rust_source(code).unwrap();
}
```

## Success Criteria

- ✅ Impl Trait types parse in return and parameter positions
- ✅ Trait objects with multiple bounds parse correctly
- ✅ Bare function types with all features parse
- ✅ FFI function pointers work
- ✅ Higher-rank trait bounds parse
- ✅ Round-trip tests preserve type structure
- ✅ 95-97% coverage achieved

## Files to Modify

1. **Primary**:
   - `crates/oxur-ast/src/integration/from_syn.rs` - Type conversions

2. **Generators**:
   - `crates/oxur-ast/src/gen_rs/types.rs` - Type generation
   - `crates/oxur-ast/src/gen_sexp/types.rs` - S-expression generation

3. **Testing**:
   - `crates/oxur-ast/tests/phase12_impl_trait_tests.rs` - New
   - `crates/oxur-ast/tests/phase12_trait_object_tests.rs` - New
   - `crates/oxur-ast/tests/phase12_bare_fn_tests.rs` - New

## Common Pitfalls

1. **Impl Trait Ambiguity**: `impl Trait` can appear in multiple positions with different meanings
2. **Dyn Keyword**: Older code might not have `dyn`, handle gracefully
3. **ABI Strings**: Various ABI names ("C", "system", "rust-call", etc.)
4. **Higher-Rank**: `for<'a>` syntax is tricky, ensure correct parsing
5. **Multiple Bounds**: Order and formatting of `+` separators

## Dependencies

- ✅ All type infrastructure exists
- ✅ GenericBound types should be defined
- ⚠️ May need TraitBoundModifier enum

## Next Phase

After Phase 12, proceed to **Phase 13: Async/Await and Modern Features** which adds:

- Async blocks and functions
- Await expressions
- Remaining modern Rust features

---

**Estimated Timeline**:

- Day 1: Impl Trait + Trait Objects (6-8 hours)
- Day 2: Bare Function types + testing (6-8 hours)
- Day 3: Generator updates + comprehensive testing (4-6 hours)

**Total**: 2-3 days for experienced developer
