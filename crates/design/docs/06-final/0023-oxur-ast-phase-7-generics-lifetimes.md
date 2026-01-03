---
number: 23
title: "oxur-ast Phase 7: Generics & Lifetimes"
author: "Duncan McGreggor"
created: 2025-12-31
updated: 2025-12-31
state: Accepted
supersedes: null
superseded-by: null
version: 1.0
---

# oxur-ast Phase 7: Generics & Lifetimes

**Phase**: 7 - Type System Completeness
**Goal**: Implement full generic type system with lifetimes, bounds, and constraints
**Estimated Time**: 3-4 days (20-30 hours)
**Complexity**: HIGH (Core Rust feature, affects all item types)

## Executive Summary

Phase 7 addresses the **generic type system** - one of Rust's most powerful and complex features. While our AST has basic `Generics` structures, the implementation is incomplete:

- `GenericParam` variants are only partially supported (type parameters work, lifetimes/consts are stubbed)
- `WherePredicate` variants are unimplemented (critical for trait bounds)
- Lifetime system is incomplete (affects references, borrows, function signatures)
- Associated types and trait bounds need full support

This phase completes the generic type system, enabling oxur-ast to handle real-world Rust code with generic functions, structs, traits, and implementations.

**Impact**: Increases capability from ~70% → ~90% of real Rust code

## 1. Scope

### 1.1 Current State

**What Works:**

- Basic generic type parameters: `fn foo<T>() {}`
- Simple generic structs: `struct Wrapper<T> { value: T }`
- Empty generics and where clauses

**What Doesn't Work:**

```rust
// Lifetime parameters
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { x }

// Where clauses with trait bounds
fn process<T>(item: T)
where
    T: Clone + Debug,
    T::Item: Display
{ }

// Const generic parameters
struct Array<T, const N: usize> {
    data: [T; N]
}

// Complex trait bounds
fn compare<T: PartialOrd + Debug>(a: T, b: T) -> bool { }

// Associated types in bounds
trait Container {
    type Item;
    fn get(&self) -> Self::Item;
}
```

### 1.2 Goals

1. **Lifetime System** (8-10 hours)
   - Full `Lifetime` AST support (named, static, anonymous)
   - Lifetime bounds and outlives relations
   - Lifetime elision understanding
   - Reference types with lifetimes

2. **Generic Parameters** (6-8 hours)
   - Complete `GenericParam::Type` with full bounds
   - Implement `GenericParam::Lifetime` with constraints
   - Implement `GenericParam::Const` for const generics
   - Default type parameters

3. **Where Clauses** (4-6 hours)
   - `WherePredicate::BoundPredicate` (trait bounds)
   - `WherePredicate::RegionPredicate` (lifetime outlives)
   - `WherePredicate::EqPredicate` (associated type equality)
   - Complex multi-constraint where clauses

4. **Trait Bounds** (2-4 hours)
   - `GenericBound::Trait` with paths and modifiers
   - `GenericBound::Outlives` for lifetime bounds
   - Higher-ranked trait bounds (HRTB): `for<'a>`
   - Negative trait bounds: `T: !Send`

### 1.3 Non-Goals

- Const generics evaluation (runtime concern)
- Type inference algorithm (compiler concern)
- Lifetime variance analysis (advanced semantic analysis)
- Generic specialization (unstable Rust feature)

## 2. Architecture

### 2.1 AST Structures (Existing)

The core structures already exist in `ast.rs`, but need complete implementation:

```rust
// Generics container
pub struct Generics {
    pub params: Vec<GenericParam>,
    pub where_clause: WhereClause,
    pub span: Span,
}

// Generic parameter variants
pub enum GenericParam {
    Lifetime(LifetimeParam),  // Needs full implementation
    Type(TypeParam),          // Needs bounds completion
    Const(ConstParam),        // Needs full implementation
}

// Where clause predicates
pub enum WherePredicate {
    BoundPredicate(WhereBoundPredicate),   // Needs implementation
    RegionPredicate(WhereRegionPredicate), // Needs implementation
    EqPredicate(WhereEqPredicate),         // Needs implementation
}

// Trait bounds
pub enum GenericBound {
    Trait(PolyTraitRef, TraitBoundModifier),  // Needs completion
    Outlives(Lifetime),                        // Needs implementation
}
```

### 2.2 Module Structure

**Affected modules:**

- `src/ast.rs` - Core structures (already defined)
- `src/generator/generics.rs` - AST → S-expr (NEW, needs creation)
- `src/builder/generics.rs` - S-expr → AST (NEW, needs creation)
- `src/codegen/generics.rs` - AST → Rust code (NEW, needs creation)
- `src/integration/from_syn.rs` - syn → AST (needs enhancement)
- `src/integration/to_syn.rs` - AST → syn (needs enhancement)

### 2.3 S-expression Representation

```scheme
;; Type parameter with bounds
(GenericParam
  :kind (Type
    :ident (Ident :name "T" :span ...)
    :bounds ((Trait
               :trait-ref (PolyTraitRef :path (Path ...) :bound-lifetimes ())
               :modifier None)
             (Trait
               :trait-ref (PolyTraitRef :path (Path ...) :bound-lifetimes ())
               :modifier None))
    :default nil))

;; Lifetime parameter with bound
(GenericParam
  :kind (Lifetime
    :ident (Ident :name "'a" :span ...)
    :bounds ((Lifetime :ident (Ident :name "'b" :span ...)))))

;; Const parameter
(GenericParam
  :kind (Const
    :ident (Ident :name "N" :span ...)
    :ty (Ty :kind (Path nil (Path ...)) ...)
    :default nil))

;; Where clause with multiple predicates
(WhereClause
  :has-where-token true
  :predicates ((BoundPredicate
                 :bounded-ty (Ty ...)
                 :bounds (...)
                 :bound-lifetimes ())
               (RegionPredicate
                 :lifetime (Lifetime ...)
                 :bounds (...))))
```

## 3. Implementation Plan

### 3.1 Priority 1: Lifetime System (8-10 hours)

**Goal**: Full lifetime support in AST, S-expr, and codegen

**Tasks:**

1. **AST Enhancement** (2 hours)
   - Verify `Lifetime` structure completeness
   - Enhance `LifetimeParam` with bounds
   - Add lifetime bound helpers

2. **Generator** (2 hours)
   - Create `src/generator/generics.rs`
   - Implement `generate_lifetime()`
   - Implement `generate_lifetime_param()`
   - Implement `generate_lifetime_bounds()`

3. **Builder** (2 hours)
   - Create `src/builder/generics.rs`
   - Implement `build_lifetime()`
   - Implement `build_lifetime_param()`
   - Handle lifetime constraints

4. **Code Generation** (2 hours)
   - Create `src/codegen/generics.rs`
   - Implement lifetime formatting: `'a`, `'static`, `'_`
   - Implement lifetime bounds: `'a: 'b + 'c`
   - Test reference types: `&'a str`, `&'a mut T`

**Test Coverage:**

```rust
// Basic lifetime
fn example<'a>(x: &'a str) -> &'a str { x }

// Multiple lifetimes
fn combine<'a, 'b>(x: &'a str, y: &'b str) -> &'a str { x }

// Lifetime bounds
fn outlives<'a, 'b: 'a>(x: &'a str, y: &'b str) -> &'a str { x }

// Static lifetime
fn static_ref() -> &'static str { "hello" }
```

### 3.2 Priority 2: Generic Type Parameters (6-8 hours)

**Goal**: Complete type parameter implementation with bounds

**Tasks:**

1. **Type Parameter Bounds** (3 hours)
   - Enhance `TypeParam` builder/generator
   - Implement trait bound lists
   - Support default type parameters
   - Handle bound modifiers (?, +)

2. **Const Parameters** (3 hours)
   - Implement `ConstParam` builder/generator
   - Support const generic types (usize, bool, char)
   - Handle const default values
   - Test array sizes, const generics

**Test Coverage:**

```rust
// Type parameter with single bound
fn process<T: Clone>(item: T) {}

// Multiple bounds
fn compare<T: PartialOrd + Debug>(a: T, b: T) {}

// Default type parameter
fn wrapper<T = i32>(value: T) {}

// Const generic
struct Array<T, const N: usize> { data: [T; N] }

// Mixed generics
fn complex<'a, T: Clone, const N: usize>(items: &'a [T; N]) {}
```

### 3.3 Priority 3: Where Clauses (4-6 hours)

**Goal**: Full where clause support for complex constraints

**Tasks:**

1. **BoundPredicate** (2 hours)
   - Implement bounded type predicates
   - Support bound generic lifetimes (HRTB)
   - Handle multiple trait bounds
   - Test complex scenarios

2. **RegionPredicate** (1 hour)
   - Implement lifetime outlives predicates
   - Support multiple lifetime bounds
   - Test lifetime constraints

3. **EqPredicate** (1 hour)
   - Implement associated type equality
   - Support complex type expressions
   - Test trait associated types

**Test Coverage:**

```rust
// Basic where clause
fn process<T>(item: T)
where
    T: Clone + Debug
{}

// Associated type constraints
fn container<C>(c: C)
where
    C: Container,
    C::Item: Display
{}

// Higher-ranked trait bounds
fn callback<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> &'a str
{}

// Lifetime outlives
fn outlives<'a, 'b, T>(x: &'a T, y: &'b T)
where
    'b: 'a,
    T: 'a + Clone
{}
```

### 3.4 Priority 4: Trait Bounds (2-4 hours)

**Goal**: Complete trait bound system

**Tasks:**

1. **Trait References** (1 hour)
   - Implement `PolyTraitRef` builder/generator
   - Support trait paths with generics
   - Handle bound lifetimes

2. **Bound Modifiers** (1 hour)
   - Implement `?Sized` (maybe bounds)
   - Support negative bounds: `T: !Send`
   - Test modifier combinations

3. **Outlives Bounds** (1 hour)
   - Implement `GenericBound::Outlives`
   - Support lifetime bounds in type parameters
   - Test combined trait + outlives bounds

**Test Coverage:**

```rust
// Maybe bound (relaxed Sized)
fn unsized<T: ?Sized>(x: &T) {}

// Negative bound
fn not_send<T: !Send>(x: T) {}

// Outlives in type parameter
fn bounded<'a, T: 'a>(x: &'a T) {}

// Complex poly trait ref
fn iterator<I>(iter: I)
where
    I: Iterator<Item = String>
{}
```

### 3.5 Priority 5: Integration & Testing (4-6 hours)

**Goal**: syn integration and comprehensive testing

**Tasks:**

1. **from_syn Enhancement** (2 hours)
   - Convert syn `Generics` → oxur_ast `Generics`
   - Handle all generic param variants
   - Convert where clause predicates
   - Test with Phase 6 item types

2. **to_syn Enhancement** (1 hour)
   - Convert oxur_ast `Generics` → syn `Generics`
   - Verify round-trip preservation
   - Test edge cases

3. **Comprehensive Testing** (3 hours)
   - Write 80+ new tests covering all combinations
   - Test real-world patterns (Option, Result, Vec, etc.)
   - Verify round-trip for complex generics
   - Integration tests with functions, structs, traits, impls

**Test Structure:**

```
tests/
  phase7_lifetime_tests.rs        (20 tests)
  phase7_type_param_tests.rs      (20 tests)
  phase7_where_clause_tests.rs    (20 tests)
  phase7_trait_bound_tests.rs     (15 tests)
  phase7_integration_tests.rs     (25 tests - real-world patterns)
```

## 4. Code Examples

### 4.1 Example: Type Parameter with Bounds

**Rust Input:**

```rust
fn process<T: Clone + Debug>(item: T) -> T {
    println!("{:?}", item);
    item.clone()
}
```

**Generated S-expression:**

```scheme
(Item
  :kind (Fn
    (Fn
      :sig (FnSig
        :decl (FnDecl
          :inputs ((Param
            :ty (Ty
              :kind (Path nil (Path
                :segments ((PathSegment :ident (Ident :name "T" ...) ...))))
              ...)
            :pat (Pat :kind (Ident BindByValue Immutable (Ident :name "item" ...)) ...)
            ...))
          :output (Ty
            :kind (Path nil (Path
              :segments ((PathSegment :ident (Ident :name "T" ...) ...))))
            ...))
        ...)
      :generics (Generics
        :params ((GenericParam
          :kind (Type
            :ident (Ident :name "T" ...)
            :bounds ((Trait
                       :trait-ref (PolyTraitRef
                         :trait-ref (TraitRef
                           :path (Path :segments ((PathSegment :ident (Ident :name "Clone" ...) ...))))
                         :bound-lifetimes ())
                       :modifier None)
                     (Trait
                       :trait-ref (PolyTraitRef
                         :trait-ref (TraitRef
                           :path (Path :segments ((PathSegment :ident (Ident :name "Debug" ...) ...))))
                         :bound-lifetimes ())
                       :modifier None))
            :default nil)
          :attrs ()
          :id 0
          :span ...))
        :where-clause (WhereClause :has-where-token false :predicates () ...)
        ...)
      :body (Block ...)))
  :ident (Ident :name "process" ...)
  ...)
```

**Generated Rust:**

```rust
fn process<T: Clone + Debug>(item: T) -> T {
    println!("{:?}", item);
    item.clone()
}
```

### 4.2 Example: Where Clause with Associated Types

**Rust Input:**

```rust
fn display_items<C>(container: C)
where
    C: Container,
    C::Item: Display
{
    println!("{}", container.get());
}
```

**Generated S-expression:**

```scheme
(Item
  :kind (Fn
    (Fn
      :generics (Generics
        :params ((GenericParam
          :kind (Type
            :ident (Ident :name "C" ...)
            :bounds ()
            :default nil)
          ...))
        :where-clause (WhereClause
          :has-where-token true
          :predicates ((BoundPredicate
                         :bounded-ty (Ty
                           :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "C" ...) ...))))
                           ...)
                         :bounds ((Trait
                           :trait-ref (PolyTraitRef
                             :trait-ref (TraitRef :path (Path :segments ((PathSegment :ident (Ident :name "Container" ...) ...))))
                             :bound-lifetimes ())
                           :modifier None))
                         :bound-lifetimes ())
                       (BoundPredicate
                         :bounded-ty (Ty
                           :kind (Path nil (Path
                             :segments ((PathSegment :ident (Ident :name "C" ...) ...)
                                       (PathSegment :ident (Ident :name "Item" ...) ...))))
                           ...)
                         :bounds ((Trait
                           :trait-ref (PolyTraitRef
                             :trait-ref (TraitRef :path (Path :segments ((PathSegment :ident (Ident :name "Display" ...) ...))))
                             :bound-lifetimes ())
                           :modifier None))
                         :bound-lifetimes ()))
          ...))
      ...))
  ...)
```

### 4.3 Example: Higher-Ranked Trait Bounds

**Rust Input:**

```rust
fn apply<F>(f: F) -> i32
where
    F: for<'a> Fn(&'a str) -> i32
{
    f("hello")
}
```

**Key S-expression fragment:**

```scheme
(WherePredicate
  :kind (BoundPredicate
    :bounded-ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "F" ...) ...)))) ...)
    :bounds ((Trait
               :trait-ref (PolyTraitRef
                 :trait-ref (TraitRef
                   :path (Path :segments ((PathSegment :ident (Ident :name "Fn" ...)
                                            :args (AngleBracketed
                                              :args ((Type (Ty :kind (Ref
                                                :lifetime (Lifetime :ident (Ident :name "'a" ...))
                                                :mutability Immutable
                                                :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "str" ...) ...))))
                                                     ...)) ...)))
                                              ...)))))
                 :bound-lifetimes ((LifetimeParam
                   :ident (Ident :name "'a" ...)
                   :bounds ()
                   :colon-span nil)))
               :modifier None))
    :bound-lifetimes ()))
```

## 5. Testing Strategy

### 5.1 Unit Tests

**Lifetime Tests** (`tests/phase7_lifetime_tests.rs`):

- Basic lifetime parameters: `'a`, `'static`, `'_`
- Multiple lifetimes: `'a, 'b, 'c`
- Lifetime bounds: `'a: 'b`
- Lifetime in references: `&'a T`, `&'a mut T`
- Lifetime elision patterns

**Type Parameter Tests** (`tests/phase7_type_param_tests.rs`):

- Simple type parameters: `<T>`
- Multiple type parameters: `<T, U, V>`
- Type bounds: `T: Clone`, `T: Clone + Debug`
- Default type parameters: `T = i32`
- Mixed with lifetimes: `<'a, T>`

**Where Clause Tests** (`tests/phase7_where_clause_tests.rs`):

- Basic where clauses
- Multiple predicates
- Associated type constraints
- Lifetime outlives predicates
- HRTB patterns

**Trait Bound Tests** (`tests/phase7_trait_bound_tests.rs`):

- Single trait bounds
- Multiple trait bounds with `+`
- Maybe bounds: `?Sized`
- Negative bounds: `!Send`
- Outlives bounds

### 5.2 Integration Tests

**Real-World Patterns** (`tests/phase7_integration_tests.rs`):

```rust
// Test standard library patterns
fn test_option_like() {
    // Option<T> pattern
}

fn test_result_like() {
    // Result<T, E> pattern
}

fn test_iterator_like() {
    // Iterator trait with associated types
}

fn test_from_into() {
    // From/Into conversion traits
}

fn test_generic_struct_impl() {
    // Generic struct with impl blocks
}
```

### 5.3 Round-Trip Tests

Every test verifies: Rust → AST → S-expr → AST → Rust

```rust
fn verify_generic_round_trip(source: &str) {
    // Parse Rust to syn AST
    let syn_file = syn::parse_file(source).unwrap();

    // Convert to oxur_ast
    let crate1 = from_syn_file(&syn_file).unwrap();

    // Generate S-expression
    let gen = Generator::new();
    let sexp1 = gen.generate_crate(&crate1).unwrap();

    // Round-trip through S-expression
    let printed = print_sexp(&sexp1);
    let sexp2 = Parser::parse_str(&printed).unwrap();

    // Build AST from S-expression
    let mut builder = AstBuilder::new();
    let crate2 = builder.build_crate(&sexp2).unwrap();

    // Verify identical
    let sexp3 = gen.generate_crate(&crate2).unwrap();
    assert_eq!(sexp1, sexp3);

    // Generate Rust code
    let code = generate_rust(&crate2).unwrap();

    // Verify semantically equivalent (may differ in whitespace)
    verify_semantic_equivalence(source, &code);
}
```

### 5.4 Coverage Goals

- **Target**: 95%+ line coverage for generic-related code
- **Minimum**: 90% coverage for new modules
- **Integration**: 100% of standard Rust generic patterns tested

## 6. Roadmap

### Day 1: Lifetimes (8-10 hours)

- Morning: Create `generator/generics.rs`, implement lifetime generation
- Midday: Create `builder/generics.rs`, implement lifetime building
- Afternoon: Create `codegen/generics.rs`, implement lifetime code generation
- Evening: Write lifetime tests, verify round-trips

### Day 2: Type Parameters & Const Generics (6-8 hours)

- Morning: Implement type parameter bounds (builder/generator/codegen)
- Afternoon: Implement const generic parameters
- Evening: Write comprehensive type parameter tests

### Day 3: Where Clauses (4-6 hours)

- Morning: Implement BoundPredicate and RegionPredicate
- Afternoon: Implement EqPredicate for associated types
- Evening: Write where clause tests

### Day 4: Trait Bounds & Integration (6-8 hours)

- Morning: Complete trait bound system (modifiers, outlives)
- Midday: Enhance from_syn/to_syn for generics
- Afternoon: Write integration tests
- Evening: Verify all tests pass, update documentation

## 7. Success Criteria

1. **Completeness**
   - ✅ All `GenericParam` variants fully implemented
   - ✅ All `WherePredicate` variants fully implemented
   - ✅ All `GenericBound` variants fully implemented
   - ✅ Lifetime system complete with bounds

2. **Quality**
   - ✅ 95%+ test coverage for generic-related code
   - ✅ 80+ new tests passing
   - ✅ Round-trip preservation for all generic patterns
   - ✅ from_syn/to_syn handle all generic constructs

3. **Real-World Readiness**
   - ✅ Can parse standard library generic patterns
   - ✅ Handles Option, Result, Vec, Iterator patterns
   - ✅ Supports real Rust codebases with generics
   - ✅ Generated code compiles and is semantically equivalent

## 8. Dependencies

**Depends on:**

- Phase 6 (Integration Layer) - Provides item type parsing foundation

**Blocks:**

- Phase 8 (Advanced Features) - Traits, impls need complete generic system
- Production readiness - Most real Rust code uses generics extensively

## 9. Risks & Mitigations

### Risk 1: HRTB Complexity

**Risk**: Higher-ranked trait bounds are notoriously complex
**Mitigation**: Start with simple cases, build up incrementally. Focus on common patterns (`for<'a> Fn(&'a T)`) first.

### Risk 2: Associated Type Constraints

**Risk**: Associated types in where clauses can be deeply nested
**Mitigation**: Use recursive descent for type expressions. Test incrementally.

### Risk 3: Const Generic Evaluation

**Risk**: Const generics can involve complex constant expressions
**Mitigation**: Only parse/preserve const generics, don't evaluate. Defer to rustc for evaluation.

### Risk 4: Integration Complexity

**Risk**: syn's generic representation is complex
**Mitigation**: Study syn source code, write extensive from_syn/to_syn tests.

## 10. Impact Analysis

### Before Phase 7

- **Generic Functions**: Basic `<T>` only
- **Generic Structs**: Simple type parameters
- **Lifetimes**: Not supported
- **Where Clauses**: Empty only
- **Real-World Code**: ~70% parseable

### After Phase 7

- **Generic Functions**: Full support with bounds, lifetimes, where clauses
- **Generic Structs**: Complete with const generics
- **Lifetimes**: Full reference lifetime system
- **Where Clauses**: All predicate types supported
- **Real-World Code**: ~90% parseable

**Capability increase**: 70% → 90% (+20%)

### Example Real-World Impact

**Before**: Cannot parse

```rust
fn iter_map<'a, I, F, B>(iter: I, f: F) -> impl Iterator<Item = B> + 'a
where
    I: Iterator + 'a,
    I::Item: Clone,
    F: Fn(I::Item) -> B + 'a,
    B: 'a
{
    iter.map(f)
}
```

**After**: Full parsing, round-trip, code generation support

## 11. Future Work

After Phase 7, the following advanced features remain:

- **Phase 8**: Trait definitions, impl blocks, macros, attributes
- **Semantic Analysis**: Type checking, lifetime checking (may never be in scope)
- **Optimization**: Generic monomorphization insights (compiler concern)
- **Error Recovery**: Better error messages for malformed generics

---

**Phase Status**: PLANNED
**Dependencies Met**: Requires Phase 6 completion
**Next Phase**: Phase 8 (Advanced Features & Polish)
