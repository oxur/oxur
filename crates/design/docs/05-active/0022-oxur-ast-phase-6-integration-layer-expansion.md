---
number: 22
title: "oxur-ast Phase 6: Integration Layer Expansion"
author: "Duncan McGreggor"
created: 2025-12-31
updated: 2025-12-31
state: Active
supersedes: null
superseded-by: null
version: 1.0
---

# oxur-ast Phase 6: Integration Layer Expansion

**Phase**: 6 - Integration & Parsing
**Goal**: Fix the critical parsing bottleneck - enable parsing all Rust constructs
**Estimated Time**: 2-3 days (15-20 hours)
**Prerequisites**: Phase 5 complete (80% AST coverage achieved)

---

## Executive Summary

Phase 6 addresses the **most critical bottleneck** preventing production use of oxur-ast: the integration layer can currently only parse function items from Rust source files.

**The Problem:**

```rust
// ✅ Currently works
fn hello() { println!("Hi"); }

// ❌ Currently fails with "unsupported `struct` item"
struct Point { x: i32, y: i32 }

// ❌ Currently fails with "unsupported `enum` item"
enum Option<T> { Some(T), None }

// ❌ Currently fails with "unsupported `trait` item"
trait Display { fn display(&self); }
```

Despite having full AST support for structs, enums, traits, impls, and other items (implemented in Phases 1-5), the `from_syn.rs` integration layer rejects them when parsing real Rust files.

**Impact:** This single bottleneck prevents oxur-ast from being useful for real-world Rust code analysis.

**Phase 6 Solution:** Expand `from_syn.rs` to convert ALL supported item types from `syn::Item` to `oxur_ast::Item`, unlocking the ability to parse complete Rust files.

---

## Table of Contents

1. [Current State Assessment](#1-current-state-assessment)
2. [Phase 6 Scope](#2-phase-6-scope)
3. [Integration Layer Architecture](#3-integration-layer-architecture)
4. [Item Type Conversion](#4-item-type-conversion)
5. [Pattern Conversion](#5-pattern-conversion)
6. [Type Conversion](#6-type-conversion)
7. [Testing Strategy](#7-testing-strategy)
8. [Success Criteria](#8-success-criteria)
9. [Implementation Roadmap](#9-implementation-roadmap)

---

## 1. Current State Assessment

### The Integration Bottleneck 🔴

**File**: `src/integration/from_syn.rs`

**Current implementation:**

```rust
pub fn convert_item(item: &syn::Item) -> Result<Item, ConversionError> {
    match item {
        syn::Item::Fn(fn_item) => {
            // ✅ Only this works!
            Ok(convert_fn_item(fn_item))
        }
        syn::Item::Struct(_) => Err(ConversionError::UnsupportedItem("struct".into())),
        syn::Item::Enum(_) => Err(ConversionError::UnsupportedItem("enum".into())),
        syn::Item::Trait(_) => Err(ConversionError::UnsupportedItem("trait".into())),
        syn::Item::Impl(_) => Err(ConversionError::UnsupportedItem("impl".into())),
        // ... all other items rejected!
    }
}
```

**Pattern conversion:**

```rust
fn convert_pat(pat: &syn::Pat) -> Result<Pat, ConversionError> {
    match pat {
        syn::Pat::Ident(ident_pat) => {
            // ✅ Only identifier patterns work
            Ok(convert_ident_pat(ident_pat))
        }
        _ => Err(ConversionError::ComplexPattern),  // ❌ Everything else rejected!
    }
}
```

**Type conversion:**

```rust
fn convert_ty(ty: &syn::Type) -> Result<Ty, ConversionError> {
    match ty {
        syn::Type::Path(path_ty) => {
            // ✅ Only path types work
            Ok(convert_path_ty(path_ty))
        }
        _ => Err(ConversionError::ComplexType),  // ❌ Everything else rejected!
    }
}
```

### What We Have vs What We Can Parse

| AST Feature | Implemented in AST | Parseable from Rust |
|-------------|-------------------|---------------------|
| Functions | ✅ | ✅ |
| Structs | ✅ | ❌ |
| Enums | ✅ | ❌ |
| Traits | ✅ | ❌ |
| Impls | ✅ | ❌ |
| Use statements | ✅ | ❌ |
| Const/Static | ✅ | ❌ |
| Type aliases | ✅ | ❌ |
| Modules | ✅ | ❌ |
| Tuple patterns | ✅ | ❌ |
| Struct patterns | ✅ | ❌ |
| Reference types | ✅ | ❌ |
| Tuple types | ✅ | ❌ |

**Analysis:** We have comprehensive AST support but minimal parsing capability.

---

## 2. Phase 6 Scope

### Priority 1: Item Type Conversion (HIGH) - 8-10 hours

**Goal**: Convert all supported ItemKind variants from syn to oxur_ast

**Items to implement:**

1. ✅ `ItemFn` - Already works
2. 🔴 `ItemStruct` - NEW
3. 🔴 `ItemEnum` - NEW
4. 🔴 `ItemTrait` - NEW
5. 🔴 `ItemImpl` - NEW
6. 🔴 `ItemUse` - NEW
7. 🔴 `ItemStatic` - NEW
8. 🔴 `ItemConst` - NEW
9. 🔴 `ItemType` - NEW
10. 🔴 `ItemMod` - NEW

**Deliverables**:

1. Implement `convert_struct_item()`
2. Implement `convert_enum_item()`
3. Implement `convert_trait_item()`
4. Implement `convert_impl_item()`
5. Implement `convert_use_item()`
6. Implement `convert_static_item()`
7. Implement `convert_const_item()`
8. Implement `convert_type_alias_item()`
9. Implement `convert_mod_item()`
10. Integration tests for each item type

### Priority 2: Pattern Conversion (HIGH) - 4-5 hours

**Goal**: Convert all pattern types we support in our AST

**Patterns to implement:**

1. ✅ `PatIdent` - Already works
2. 🔴 `PatWild` - NEW
3. 🔴 `PatTuple` - NEW
4. 🔴 `PatTupleStruct` - NEW
5. 🔴 `PatStruct` - NEW
6. 🔴 `PatOr` - NEW
7. 🔴 `PatReference` - NEW
8. 🔴 `PatLit` - NEW
9. 🔴 `PatRange` - NEW
10. 🔴 `PatSlice` - NEW

**Deliverables**:

1. Implement pattern conversion for each variant
2. Handle complex nested patterns
3. Pattern conversion tests

### Priority 3: Type Conversion (MEDIUM) - 3-4 hours

**Goal**: Convert all type constructs we support in our AST

**Types to implement:**

1. ✅ `TypePath` - Already works
2. 🔴 `TypeReference` - NEW
3. 🔴 `TypePtr` - NEW
4. 🔴 `TypeSlice` - NEW
5. 🔴 `TypeArray` - NEW
6. 🔴 `TypeTuple` - NEW
7. 🔴 `TypeNever` - NEW
8. 🔴 `TypeInfer` - NEW
9. 🔴 `TypeBareFn` - NEW (if time permits)
10. 🔴 `TypeImplTrait` - NEW (if time permits)

**Deliverables**:

1. Implement type conversion for each variant
2. Handle complex nested types
3. Type conversion tests

### Out of Scope (Future Phases)

- ❌ Generic parameters (Phase 7)
- ❌ Lifetime parameters (Phase 7)
- ❌ Macro definitions (Phase 8)
- ❌ FFI blocks (Phase 8)
- ❌ Advanced attributes (Phase 8)

---

## 3. Integration Layer Architecture

### Current Flow

```
Rust Source Code
      ↓
   syn::parse_file()
      ↓
   syn::File (syn AST)
      ↓
   convert_item()  ← BOTTLENECK: Only converts Fn items!
      ↓
   oxur_ast::Item
      ↓
   oxur_ast::Crate
```

### Target Flow

```
Rust Source Code
      ↓
   syn::parse_file()
      ↓
   syn::File (syn AST)
      ↓
   convert_item()  ← EXPANDED: Converts ALL items!
      ├─ convert_fn_item()
      ├─ convert_struct_item()      ← NEW
      ├─ convert_enum_item()        ← NEW
      ├─ convert_trait_item()       ← NEW
      ├─ convert_impl_item()        ← NEW
      └─ ... (all items)
      ↓
   oxur_ast::Item
      ↓
   oxur_ast::Crate
```

### File Structure

```
src/integration/
├── from_syn.rs           # Main conversion entry point
├── items.rs              # NEW: Item conversions
├── patterns.rs           # NEW: Pattern conversions
├── types.rs              # NEW: Type conversions
├── expressions.rs        # Existing: Expression conversions
└── helpers.rs            # Shared conversion utilities
```

---

## 4. Item Type Conversion

### Part 4.1: Struct Conversion

**syn → oxur_ast mapping:**

```rust
// src/integration/items.rs

use syn;
use crate::ast::*;
use crate::error::ConversionError;

pub fn convert_struct_item(item: &syn::ItemStruct) -> Result<Item, ConversionError> {
    let ident = Ident::new(&item.ident.to_string(), Span::DUMMY);

    // Convert struct variant (unit, tuple, or named)
    let variant = match &item.fields {
        syn::Fields::Named(fields) => {
            let field_list = fields.named.iter()
                .map(|f| convert_struct_field(f))
                .collect::<Result<Vec<_>, _>>()?;
            StructVariant::Struct(field_list)
        }
        syn::Fields::Unnamed(fields) => {
            let field_list = fields.unnamed.iter()
                .map(|f| convert_struct_field(f))
                .collect::<Result<Vec<_>, _>>()?;
            StructVariant::Tuple(field_list)
        }
        syn::Fields::Unit => {
            StructVariant::Unit
        }
    };

    Ok(Item {
        attrs: Vec::new(),  // TODO: Phase 8
        id: NodeId::new(),
        span: Span::DUMMY,
        vis: convert_visibility(&item.vis),
        ident,
        kind: ItemKind::Struct {
            variant,
            generics: Generics::empty(),  // TODO: Phase 7
        },
        tokens: None,
    })
}

fn convert_struct_field(field: &syn::Field) -> Result<StructField, ConversionError> {
    Ok(StructField {
        attrs: Vec::new(),
        vis: convert_visibility(&field.vis),
        ident: field.ident.as_ref()
            .map(|i| Ident::new(&i.to_string(), Span::DUMMY)),
        ty: convert_type(&field.ty)?,
        span: Span::DUMMY,
    })
}
```

**Test case:**

```rust
#[test]
fn test_parse_struct_named() {
    let source = r#"
    struct Point {
        x: i32,
        y: i32,
    }
    "#;

    let crate_ast = parse_rust_file(source).unwrap();
    assert_eq!(crate_ast.items.len(), 1);

    match &crate_ast.items[0].kind {
        ItemKind::Struct { variant: StructVariant::Struct(fields), .. } => {
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].ident.as_ref().unwrap().name, "x");
            assert_eq!(fields[1].ident.as_ref().unwrap().name, "y");
        }
        _ => panic!("Expected struct item"),
    }
}
```

### Part 4.2: Enum Conversion

```rust
pub fn convert_enum_item(item: &syn::ItemEnum) -> Result<Item, ConversionError> {
    let ident = Ident::new(&item.ident.to_string(), Span::DUMMY);

    let variants = item.variants.iter()
        .map(|v| convert_enum_variant(v))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Item {
        attrs: Vec::new(),
        id: NodeId::new(),
        span: Span::DUMMY,
        vis: convert_visibility(&item.vis),
        ident,
        kind: ItemKind::Enum {
            variants,
            generics: Generics::empty(),  // TODO: Phase 7
        },
        tokens: None,
    })
}

fn convert_enum_variant(variant: &syn::Variant) -> Result<EnumVariant, ConversionError> {
    let ident = Ident::new(&variant.ident.to_string(), Span::DUMMY);

    let kind = match &variant.fields {
        syn::Fields::Named(fields) => {
            let field_list = fields.named.iter()
                .map(|f| convert_struct_field(f))
                .collect::<Result<Vec<_>, _>>()?;
            EnumVariantKind::Struct(field_list)
        }
        syn::Fields::Unnamed(fields) => {
            let field_list = fields.unnamed.iter()
                .map(|f| convert_struct_field(f))
                .collect::<Result<Vec<_>, _>>()?;
            EnumVariantKind::Tuple(field_list)
        }
        syn::Fields::Unit => {
            EnumVariantKind::Unit
        }
    };

    let discriminant = variant.discriminant.as_ref()
        .map(|(_, expr)| convert_expr(expr))
        .transpose()?;

    Ok(EnumVariant {
        attrs: Vec::new(),
        id: NodeId::new(),
        ident,
        kind,
        discriminant,
        span: Span::DUMMY,
    })
}
```

### Part 4.3: Trait Conversion

```rust
pub fn convert_trait_item(item: &syn::ItemTrait) -> Result<Item, ConversionError> {
    let ident = Ident::new(&item.ident.to_string(), Span::DUMMY);

    let items = item.items.iter()
        .map(|i| convert_trait_item_member(i))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Item {
        attrs: Vec::new(),
        id: NodeId::new(),
        span: Span::DUMMY,
        vis: convert_visibility(&item.vis),
        ident,
        kind: ItemKind::Trait {
            safety: Safety::Default,  // TODO: handle unsafe traits
            is_auto: false,
            generics: Generics::empty(),  // TODO: Phase 7
            bounds: Vec::new(),  // TODO: Phase 7
            items,
        },
        tokens: None,
    })
}

fn convert_trait_item_member(item: &syn::TraitItem) -> Result<AssocItem, ConversionError> {
    match item {
        syn::TraitItem::Fn(fn_item) => {
            Ok(AssocItem::Fn(convert_trait_fn(fn_item)?))
        }
        syn::TraitItem::Type(type_item) => {
            Ok(AssocItem::Type(convert_trait_type(type_item)?))
        }
        _ => Err(ConversionError::UnsupportedTraitItem),
    }
}
```

### Part 4.4: Impl Conversion

```rust
pub fn convert_impl_item(item: &syn::ItemImpl) -> Result<Item, ConversionError> {
    let self_ty = Box::new(convert_type(&item.self_ty)?);

    let of_trait = item.trait_.as_ref()
        .map(|(_, path, _)| convert_trait_ref(path))
        .transpose()?;

    let items = item.items.iter()
        .map(|i| convert_impl_item_member(i))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Item {
        attrs: Vec::new(),
        id: NodeId::new(),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("impl", Span::DUMMY),  // Impls don't have names
        kind: ItemKind::Impl {
            safety: Safety::Default,
            polarity: ImplPolarity::Positive,
            defaultness: Defaultness::Final,
            constness: Constness::NotConst,
            generics: Generics::empty(),  // TODO: Phase 7
            of_trait,
            self_ty,
            items,
        },
        tokens: None,
    })
}
```

### Part 4.5: Simpler Items

```rust
pub fn convert_use_item(item: &syn::ItemUse) -> Result<Item, ConversionError> {
    let tree = convert_use_tree(&item.tree)?;

    Ok(Item {
        attrs: Vec::new(),
        id: NodeId::new(),
        span: Span::DUMMY,
        vis: convert_visibility(&item.vis),
        ident: Ident::new("use", Span::DUMMY),
        kind: ItemKind::Use(tree),
        tokens: None,
    })
}

pub fn convert_const_item(item: &syn::ItemConst) -> Result<Item, ConversionError> {
    Ok(Item {
        attrs: Vec::new(),
        id: NodeId::new(),
        span: Span::DUMMY,
        vis: convert_visibility(&item.vis),
        ident: Ident::new(&item.ident.to_string(), Span::DUMMY),
        kind: ItemKind::Const {
            defaultness: Defaultness::Final,
            ty: Box::new(convert_type(&item.ty)?),
            expr: Some(Box::new(convert_expr(&item.expr)?)),
        },
        tokens: None,
    })
}

pub fn convert_static_item(item: &syn::ItemStatic) -> Result<Item, ConversionError> {
    Ok(Item {
        attrs: Vec::new(),
        id: NodeId::new(),
        span: Span::DUMMY,
        vis: convert_visibility(&item.vis),
        ident: Ident::new(&item.ident.to_string(), Span::DUMMY),
        kind: ItemKind::Static {
            mutability: if matches!(item.mutability, Some(_)) {
                Mutability::Mut
            } else {
                Mutability::Not
            },
            ty: Box::new(convert_type(&item.ty)?),
            expr: Some(Box::new(convert_expr(&item.expr)?)),
        },
        tokens: None,
    })
}

pub fn convert_type_alias_item(item: &syn::ItemType) -> Result<Item, ConversionError> {
    Ok(Item {
        attrs: Vec::new(),
        id: NodeId::new(),
        span: Span::DUMMY,
        vis: convert_visibility(&item.vis),
        ident: Ident::new(&item.ident.to_string(), Span::DUMMY),
        kind: ItemKind::TyAlias {
            defaultness: Defaultness::Final,
            generics: Generics::empty(),  // TODO: Phase 7
            ty: Some(Box::new(convert_type(&item.ty)?)),
        },
        tokens: None,
    })
}
```

---

## 5. Pattern Conversion

### Implementation Strategy

```rust
// src/integration/patterns.rs

use syn;
use crate::ast::*;
use crate::error::ConversionError;

pub fn convert_pattern(pat: &syn::Pat) -> Result<Pat, ConversionError> {
    let kind = match pat {
        syn::Pat::Ident(ident_pat) => convert_ident_pattern(ident_pat)?,
        syn::Pat::Wild(_) => PatKind::Wild,
        syn::Pat::Tuple(tuple_pat) => convert_tuple_pattern(tuple_pat)?,
        syn::Pat::TupleStruct(ts_pat) => convert_tuple_struct_pattern(ts_pat)?,
        syn::Pat::Struct(struct_pat) => convert_struct_pattern(struct_pat)?,
        syn::Pat::Or(or_pat) => convert_or_pattern(or_pat)?,
        syn::Pat::Reference(ref_pat) => convert_reference_pattern(ref_pat)?,
        syn::Pat::Lit(lit_pat) => convert_lit_pattern(lit_pat)?,
        syn::Pat::Range(range_pat) => convert_range_pattern(range_pat)?,
        syn::Pat::Slice(slice_pat) => convert_slice_pattern(slice_pat)?,
        syn::Pat::Rest(_) => PatKind::Rest,
        syn::Pat::Paren(paren_pat) => {
            PatKind::Paren(Box::new(convert_pattern(&paren_pat.pat)?))
        }
        _ => return Err(ConversionError::UnsupportedPattern),
    };

    Ok(Pat {
        id: NodeId::new(),
        kind,
        span: Span::DUMMY,
        tokens: None,
    })
}

fn convert_tuple_pattern(pat: &syn::PatTuple) -> Result<PatKind, ConversionError> {
    let elems = pat.elems.iter()
        .map(|p| convert_pattern(p))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PatKind::Tuple(elems))
}

fn convert_struct_pattern(pat: &syn::PatStruct) -> Result<PatKind, ConversionError> {
    let path = convert_path(&pat.path)?;

    let fields = pat.fields.iter()
        .map(|f| convert_field_pattern(f))
        .collect::<Result<Vec<_>, _>>()?;

    let rest = if pat.rest.is_some() {
        PatFieldsRest::Rest
    } else {
        PatFieldsRest::None
    };

    Ok(PatKind::Struct {
        qself: None,  // TODO: Phase 7
        path,
        fields,
        rest,
    })
}

fn convert_or_pattern(pat: &syn::PatOr) -> Result<PatKind, ConversionError> {
    let cases = pat.cases.iter()
        .map(|p| convert_pattern(p))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PatKind::Or(cases))
}
```

---

## 6. Type Conversion

### Implementation Strategy

```rust
// src/integration/types.rs

use syn;
use crate::ast::*;
use crate::error::ConversionError;

pub fn convert_type(ty: &syn::Type) -> Result<Ty, ConversionError> {
    let kind = match ty {
        syn::Type::Path(path_ty) => {
            TyKind::Path(None, convert_path(&path_ty.path)?)
        }
        syn::Type::Reference(ref_ty) => {
            let mutability = if ref_ty.mutability.is_some() {
                Mutability::Mut
            } else {
                Mutability::Not
            };
            TyKind::Rptr(
                None,  // TODO: Phase 7 - lifetime
                Box::new(MutTy {
                    ty: convert_type(&ref_ty.elem)?,
                    mutbl: mutability,
                })
            )
        }
        syn::Type::Ptr(ptr_ty) => {
            let mutability = match ptr_ty.const_token {
                Some(_) => Mutability::Not,
                None => Mutability::Mut,
            };
            TyKind::Ptr(Box::new(MutTy {
                ty: convert_type(&ptr_ty.elem)?,
                mutbl: mutability,
            }))
        }
        syn::Type::Slice(slice_ty) => {
            TyKind::Slice(Box::new(convert_type(&slice_ty.elem)?))
        }
        syn::Type::Array(array_ty) => {
            let elem = Box::new(convert_type(&array_ty.elem)?);
            let len = Box::new(convert_array_len(&array_ty.len)?);
            TyKind::Array(elem, len)
        }
        syn::Type::Tuple(tuple_ty) => {
            let elems = tuple_ty.elems.iter()
                .map(|t| convert_type(t))
                .collect::<Result<Vec<_>, _>>()?;
            TyKind::Tup(elems)
        }
        syn::Type::Never(_) => TyKind::Never,
        syn::Type::Infer(_) => TyKind::Infer,
        syn::Type::ImplTrait(impl_trait) => {
            let bounds = impl_trait.bounds.iter()
                .map(|b| convert_type_param_bound(b))
                .collect::<Result<Vec<_>, _>>()?;
            TyKind::ImplTrait(NodeId::new(), bounds)
        }
        syn::Type::TraitObject(trait_obj) => {
            let bounds = trait_obj.bounds.iter()
                .map(|b| convert_type_param_bound(b))
                .collect::<Result<Vec<_>, _>>()?;
            let syntax = if trait_obj.dyn_token.is_some() {
                TraitObjectSyntax::Dyn
            } else {
                TraitObjectSyntax::None
            };
            TyKind::TraitObject(bounds, syntax)
        }
        syn::Type::Paren(paren_ty) => {
            TyKind::Paren(Box::new(convert_type(&paren_ty.elem)?))
        }
        _ => return Err(ConversionError::UnsupportedType),
    };

    Ok(Ty {
        id: NodeId::new(),
        kind,
        span: Span::DUMMY,
        tokens: None,
    })
}
```

---

## 7. Testing Strategy

### Test Organization

```
oxur-ast/tests/
├── integration_items_tests.rs        # NEW: Test all item conversions
├── integration_patterns_tests.rs     # NEW: Test pattern conversions
├── integration_types_tests.rs        # NEW: Test type conversions
└── integration_real_world_tests.rs   # NEW: Real Rust files
```

### Test Coverage

**Item Conversion Tests** (10 tests):

```rust
#[test] fn test_parse_struct_unit() { ... }
#[test] fn test_parse_struct_tuple() { ... }
#[test] fn test_parse_struct_named() { ... }
#[test] fn test_parse_enum_simple() { ... }
#[test] fn test_parse_enum_with_discriminants() { ... }
#[test] fn test_parse_trait_with_methods() { ... }
#[test] fn test_parse_impl_inherent() { ... }
#[test] fn test_parse_impl_trait() { ... }
#[test] fn test_parse_use_simple() { ... }
#[test] fn test_parse_const_static() { ... }
```

**Pattern Tests** (10 tests):

```rust
#[test] fn test_parse_tuple_pattern() { ... }
#[test] fn test_parse_struct_pattern() { ... }
#[test] fn test_parse_or_pattern() { ... }
#[test] fn test_parse_reference_pattern() { ... }
#[test] fn test_parse_slice_pattern() { ... }
// ... etc
```

**Type Tests** (8 tests):

```rust
#[test] fn test_parse_reference_type() { ... }
#[test] fn test_parse_tuple_type() { ... }
#[test] fn test_parse_slice_type() { ... }
#[test] fn test_parse_array_type() { ... }
// ... etc
```

**Real-World Tests**:

```rust
#[test]
fn test_parse_complete_module() {
    let source = r#"
    pub struct Config {
        host: String,
        port: u16,
    }

    pub enum Status {
        Active,
        Inactive,
    }

    pub trait Service {
        fn start(&self) -> Result<(), String>;
    }

    impl Service for Config {
        fn start(&self) -> Result<(), String> {
            Ok(())
        }
    }
    "#;

    let crate_ast = parse_rust_file(source).unwrap();
    assert_eq!(crate_ast.items.len(), 4);
}
```

---

## 8. Success Criteria

Phase 6 is complete when:

### Item Parsing ✅

- [ ] All 10 ItemKind variants can be parsed from Rust
- [ ] Struct parsing works (unit, tuple, named)
- [ ] Enum parsing works (all variants)
- [ ] Trait parsing works (with methods and types)
- [ ] Impl parsing works (trait + inherent)
- [ ] Use/Const/Static/Type/Mod parsing works
- [ ] 50+ item parsing tests passing

### Pattern Parsing ✅

- [ ] All supported PatKind variants can be parsed
- [ ] Complex nested patterns work
- [ ] Pattern tests cover all variants
- [ ] 30+ pattern tests passing

### Type Parsing ✅

- [ ] All supported TyKind variants can be parsed
- [ ] Complex nested types work
- [ ] Reference, tuple, slice, array types work
- [ ] 25+ type tests passing

### Integration ✅

- [ ] Can parse complete Rust modules
- [ ] Real-world Rust files parse successfully
- [ ] Round-trip: Rust → AST → SExp → AST → Rust works
- [ ] No regression in existing functionality
- [ ] All 750+ tests passing (656 current + ~100 new)

### Quality ✅

- [ ] Code coverage >85%
- [ ] All clippy warnings addressed
- [ ] Documentation updated
- [ ] Error messages are helpful

---

## 9. Implementation Roadmap

### Day 1: Item Conversions (8-10 hours)

**Morning (4 hours):**

- Set up new file structure (items.rs, patterns.rs, types.rs)
- Implement struct conversion
- Implement enum conversion
- Basic tests for structs and enums

**Afternoon (4-6 hours):**

- Implement trait conversion
- Implement impl conversion
- Implement use/const/static/type/mod conversions
- Comprehensive item tests

### Day 2: Pattern & Type Conversions (6-8 hours)

**Morning (3-4 hours):**

- Implement all pattern conversions
- Pattern tests
- Fix any issues

**Afternoon (3-4 hours):**

- Implement all type conversions
- Type tests
- Integration testing

### Day 3: Testing & Polish (1-2 hours)

**Morning (1-2 hours):**

- Real-world file tests
- Documentation updates
- Fix any edge cases
- Final validation

### Total Time Estimate

- **Item Conversions**: 8-10 hours
- **Pattern Conversions**: 3-4 hours
- **Type Conversions**: 3-4 hours
- **Testing & Polish**: 1-2 hours
- **TOTAL**: 15-20 hours (2-3 working days)

---

## 10. Impact Analysis

### Before Phase 6

```rust
// Can only parse this:
fn hello() {
    println!("Hi");
}

// Everything else fails!
```

**Usability**: 10% - Can only analyze simple function-only code

### After Phase 6

```rust
// Can parse complete Rust modules:
pub struct Point { x: i32, y: i32 }
pub enum Option<T> { Some(T), None }
pub trait Display { fn display(&self); }
impl Display for Point { ... }
use std::collections::HashMap;
const PI: f64 = 3.14;
static GLOBAL: i32 = 42;
type MyInt = i32;
mod utils { ... }
```

**Usability**: 85% - Can analyze most real-world Rust code!

### Phase 6 Unlocks

1. **Real-world code analysis** - Can now parse actual Rust projects
2. **Oxur language development** - Can use oxur-ast in oxur-comp
3. **Tooling foundation** - Can build LSP, formatters, linters
4. **Phase 7 enablement** - Generics need full parsing first

---

## Conclusion

Phase 6 is the **highest-impact phase** despite being one of the smallest. It removes the critical bottleneck preventing oxur-ast from being useful for real-world Rust code.

**Key Insight**: We've built 80% of the AST infrastructure, but we can only parse 10% of actual Rust code. Phase 6 flips this ratio.

**After Phase 6**:

- ✅ Can parse complete Rust files
- ✅ Can analyze real-world codebases
- ✅ Ready for production use (with Phase 7 for generics)
- ✅ Foundation for advanced tooling

**Next**: Phase 7 will add generics and lifetimes for full type system support.

---

*"The best infrastructure is useless if it can't accept real inputs. Phase 6 opens the gates."*
