---
number: 14
title: "Rust AST Inventory for Oxur Lisp"
author: "Duncan McGreggor"
component: AST
tags: [rust, reference]
created: 2025-12-27
updated: 2025-12-27
state: Accepted
supersedes: null
superseded-by: null
version: 1.0
---

# Rust AST Inventory for Oxur Lisp

*Complete Categorized Reference from syn 2.0*

**Purpose**: Reference for designing Oxur's Core Forms that will map to/from Rust AST
**Based on**: syn crate 2.0.111 (with `full` feature enabled)
**Date**: December 2025

---

## Table of Contents

1. [Items (Top-Level Declarations)](#items-top-level-declarations)
2. [Expressions](#expressions)
3. [Patterns](#patterns)
4. [Types](#types)
5. [Statements](#statements)
6. [Generics & Bounds](#generics--bounds)
7. [Attributes & Metadata](#attributes--metadata)
8. [Literals](#literals)
9. [Foreign Items (FFI)](#foreign-items-ffi)
10. [Trait Items](#trait-items)
11. [Impl Items](#impl-items)
12. [Use Declarations](#use-declarations)
13. [Miscellaneous](#miscellaneous)

---

## Items (Top-Level Declarations)

Items are things that can appear at the top level of a module or inside braces.

### Item

The main enum containing all item types:

- **ItemConst** - A constant item: `const MAX: u16 = 65535`
- **ItemEnum** - An enum definition: `enum Foo<A, B> { A(A), B(B) }`
- **ItemExternCrate** - An extern crate item: `extern crate serde`
- **ItemFn** - A free-standing function: `fn process(n: usize) -> Result<()> { ... }`
- **ItemForeignMod** - A block of foreign items: `extern "C" { ... }`
- **ItemImpl** - An impl block: `impl<A> Trait for Data<A> { ... }`
- **ItemMacro** - A macro invocation, including `macro_rules!` definitions
- **ItemMod** - A module or module declaration: `mod m` or `mod m { ... }`
- **ItemStatic** - A static item: `static BIKE: Shed = Shed(42)`
- **ItemStruct** - A struct definition: `struct Foo<A> { x: A }`
- **ItemTrait** - A trait definition: `pub trait Iterator { ... }`
- **ItemTraitAlias** - A trait alias: `pub trait SharableIterator = Iterator + Sync`
- **ItemType** - A type alias: `type Result<T> = std::result::Result<T, MyError>`
- **ItemUnion** - A union definition: `union Foo<A, B> { x: A, y: B }`
- **ItemUse** - A use declaration: `use std::collections::HashMap`

### Supporting Structures

- **File** - A complete file of Rust source code (contains `Vec<Item>` plus attributes and shebang)
- **Signature** - A function signature in a trait or implementation: `unsafe fn initialize(&self)`
- **Variant** - An enum variant
- **Field** - A field of a struct or enum variant
- **DataEnum** - Enum input to derive macro (contains variants)
- **DataStruct** - Struct input to derive macro (contains fields)
- **DataUnion** - Union input to derive macro (contains fields)

---

## Expressions

Expressions are the heart of Rust code - things that produce values.

### Expr

The main enum containing all expression types:

#### Literal & Path Expressions

- **ExprLit** - A literal in place of an expression: `1`, `"foo"`
- **ExprPath** - A path like `std::mem::replace` possibly with generic parameters and qualified self-type

#### Operators

- **ExprBinary** - Binary operation: `a + b`, `a += b`
- **ExprUnary** - Unary operation: `!x`, `*x`
- **ExprCast** - Cast expression: `foo as f64`

#### Function Calls & Method Calls

- **ExprCall** - Function call: `invoke(a, b)`
- **ExprMethodCall** - Method call: `x.foo::<T>(a, b)`

#### Field & Index Access

- **ExprField** - Named or unnamed field access: `obj.k` or `obj.0`
- **ExprIndex** - Square bracket indexing: `vector[2]`

#### Struct & Tuple Construction

- **ExprStruct** - Struct literal: `Point { x: 1, y: 1 }`
- **ExprTuple** - Tuple expression: `(a, b, c, d)`
- **ExprArray** - Slice literal: `[a, b, c, d]`
- **ExprRepeat** - Array literal from one repeated element: `[0u8; N]`

#### Control Flow

- **ExprIf** - If expression with optional else: `if expr { ... } else { ... }`
- **ExprMatch** - Match expression: `match n { Some(n) => {}, None => {} }`
- **ExprWhile** - While loop: `while expr { ... }`
- **ExprForLoop** - For loop: `for pat in expr { ... }`
- **ExprLoop** - Conditionless loop: `loop { ... }`
- **ExprBreak** - Break, with optional label and expression
- **ExprContinue** - Continue, with optional label
- **ExprReturn** - Return with optional value
- **ExprLet** - Let guard: `let Some(x) = opt`

#### Block Expressions

- **ExprBlock** - Blocked scope: `{ ... }`
- **ExprUnsafe** - Unsafe block: `unsafe { ... }`
- **ExprAsync** - Async block: `async { ... }`
- **ExprConst** - Const block: `const { ... }`
- **ExprTryBlock** - Try block: `try { ... }`

#### Closures & Async

- **ExprClosure** - Closure expression: `|a, b| a + b`
- **ExprAwait** - Await expression: `fut.await`

#### Error Handling

- **ExprTry** - Try expression: `expr?`

#### Range Expressions

- **ExprRange** - Range: `1..2`, `1..`, `..2`, `1..=2`, `..=2`

#### Reference & Pointer

- **ExprReference** - Referencing operation: `&a` or `&mut a`
- **ExprRawAddr** - Address-of operation: `&raw const place` or `&raw mut place`

#### Generators & Special

- **ExprYield** - Yield expression: `yield expr`
- **ExprMacro** - Macro invocation: `format!("{}", q)`
- **ExprGroup** - Expression in invisible delimiters
- **ExprParen** - Parenthesized expression: `(a + b)`
- **ExprAssign** - Assignment: `a = compute()`
- **ExprInfer** - Type inference placeholder for const generics: `_`

### Supporting Structures

- **FieldValue** - Field-value pair in struct literal
- **Arm** - One arm of a match: `0..=10 => { return true; }`
- **Block** - Braced block containing statements
- **Label** - Lifetime labeling a loop: `'outer`

---

## Patterns

Patterns match values and bind variables.

### Pat

The main enum for pattern types:

- **PatConst** - Const block: `const { ... }`
- **PatIdent** - Pattern that binds a new variable: `ref mut binding @ SUBPATTERN`
- **PatLit** - Literal pattern: `1`, `"foo"`
- **PatMacro** - Macro invocation in pattern position
- **PatOr** - Pattern matching any one of a set: `A | B | C`
- **PatParen** - Parenthesized pattern: `(A | B)`
- **PatPath** - Path pattern: `std::mem::replace`
- **PatRange** - Range pattern: `1..2`, `1..`, `..2`, `1..=2`, `..=2`
- **PatReference** - Reference pattern: `&mut var`
- **PatRest** - Dots in tuple/slice pattern: `[0, 1, ..]`
- **PatSlice** - Dynamically sized slice pattern: `[a, b, ref i @ .., y, z]`
- **PatStruct** - Struct or variant pattern: `Variant { x, y, .. }`
- **PatTuple** - Tuple pattern: `(a, b)`
- **PatTupleStruct** - Tuple struct/variant pattern: `Variant(x, y, .., z)`
- **PatType** - Type ascription pattern: `foo: f64`
- **PatWild** - Wildcard pattern: `_`

### Supporting Structures

- **FieldPat** - Single field in a struct pattern

---

## Types

Type annotations and type expressions.

### Type

The main enum for type expressions:

#### Primitive & Path Types

- **TypePath** - Path type: `std::slice::Iter`, optionally qualified with self-type
- **TypeNever** - Never type: `!`
- **TypeInfer** - Type inference placeholder: `_`

#### Compound Types

- **TypeArray** - Fixed size array: `[T; n]`
- **TypeSlice** - Dynamically sized slice: `[T]`
- **TypeTuple** - Tuple type: `(A, B, C, String)`
- **TypeReference** - Reference type: `&'a T` or `&'a mut T`
- **TypePtr** - Raw pointer type: `*const T` or `*mut T`

#### Function Types

- **TypeBareFn** - Bare function type: `fn(usize) -> bool`

#### Trait Types

- **TypeImplTrait** - Impl trait type: `impl Bound1 + Bound2 + Bound3`
- **TypeTraitObject** - Trait object type: `dyn Bound1 + Bound2 + Bound3`

#### Special

- **TypeMacro** - Macro in type position
- **TypeGroup** - Type in invisible delimiters
- **TypeParen** - Parenthesized type (equivalent to inner type)

### Supporting Structures

- **BareFnArg** - Argument in function type: the `usize` in `fn(usize) -> bool`
- **BareVariadic** - Variadic argument of function pointer: `fn(usize, ...)`
- **ReturnType** - Return type of function signature (enum: Default or Type)

---

## Statements

Statements are executable actions that don't return values (usually).

### Stmt

The main enum for statement types:

- **Stmt::Local** - Local let binding: `let x: u64 = s.parse()?;`
- **Stmt::Item** - Item definition (any Item)
- **Stmt::Expr** - Expression without semicolon
- **Stmt::Macro** - Macro invocation in statement position

### Supporting Structures

- **Local** - Local let binding structure
- **LocalInit** - Expression assigned in let binding, including optional diverging else
- **StmtMacro** - Macro invocation statement structure

---

## Generics & Bounds

Generic parameters, lifetime bounds, trait bounds, where clauses.

### Generic Parameters

- **Generics** - Lifetimes and type parameters attached to a declaration
- **GenericParam** - Enum of Lifetime, Type, or Const parameter
- **LifetimeParam** - Lifetime definition: `'a: 'b + 'c + 'd`
- **TypeParam** - Generic type parameter: `T: Into<String>`
- **ConstParam** - Const generic parameter: `const LENGTH: usize`

### Generic Arguments

- **GenericArgument** - Enum: Lifetime, Type, Const, AssocType, AssocConst, Constraint
- **AngleBracketedGenericArguments** - Angle bracketed args: `<K, V>` in `HashMap<K, V>`
- **ParenthesizedGenericArguments** - Function args: `(A, B) -> C` in `Fn(A,B) -> C`

### Bounds & Constraints

- **TypeParamBound** - Enum: Trait bound or Lifetime
- **TraitBound** - Trait used as bound on type parameter
- **TraitBoundModifier** - Modifier on trait bound (currently only `?` in `?Sized`)
- **Constraint** - Associated type bound: `Iterator<Item: Display>`
- **AssocType** - Equality constraint on associated type: `Item = u8` in `Iterator<Item = u8>`
- **AssocConst** - Equality constraint on associated constant: `PANIC = false` in `Trait<PANIC = false>`

### Where Clauses

- **WhereClause** - Where clause in a definition: `where T: Deserialize<'de>, D: 'static`
- **WherePredicate** - Enum: Lifetime, Type, or Eq (equality predicate - unsupported)
- **PredicateLifetime** - Lifetime predicate: `'a: 'b + 'c`
- **PredicateType** - Type predicate: `for<'c> Foo<'c>: Trait<'c>`

### Lifetime & Bounds

- **Lifetime** - A Rust lifetime: `'a`
- **BoundLifetimes** - Set of bound lifetimes: `for<'a, 'b, 'c>`
- **PreciseCapture** - Precise capturing bound: `use<'a, T>` in `impl Trait + use<'a, T>`
- **CapturedParam** - Single parameter in precise capturing bound (enum: Lifetime or Type)

### Path-Related

- **Path** - Path at which named item is exported: `std::collections::HashMap`
- **PathSegment** - Segment of path with arguments: `HashMap` in `std::collections::HashMap`
- **PathArguments** - Enum: None, AngleBracketed, or Parenthesized
- **QSelf** - Explicit Self type in qualified path: `T` in `<T as Display>::fmt`

### Helper Types for Code Generation

- **ImplGenerics** - Returned by `Generics::split_for_impl` (for printing)
- **TypeGenerics** - Returned by `Generics::split_for_impl` (for printing)
- **Turbofish** - Returned by `TypeGenerics::as_turbofish` (for printing `::<>`)

---

## Attributes & Metadata

Attributes decorate or provide metadata about items.

### Attribute

- **Attribute** - An attribute like `#[repr(transparent)]`
- **AttrStyle** - Enum: Outer (`#[...]`) or Inner (`#![...]`)

### Meta (Attribute Content)

- **Meta** - Enum containing attribute content types:
  - **Path** - Just an identifier: `#[test]`
  - **List** - List structure: `#[derive(Copy, Clone)]`
  - **NameValue** - Name-value pair: `#[feature = "nightly"]`
- **MetaList** - Structured list within attribute
- **MetaNameValue** - Name-value pair within attribute

### Macro-Related

- **Macro** - Macro invocation: `println!("{}", mac)`
- **MacroDelimiter** - Grouping token around macro body: `(...)`, `{...}`, `[...]`

---

## Literals

Literal values in Rust code.

### Lit

Main enum for literal types:

- **LitStr** - UTF-8 string literal: `"foo"`
- **LitByteStr** - Byte string literal: `b"foo"`
- **LitCStr** - Nul-terminated C-string literal: `c"foo"`
- **LitByte** - Byte literal: `b'f'`
- **LitChar** - Character literal: `'a'`
- **LitInt** - Integer literal: `1` or `1u16`
- **LitFloat** - Floating point literal: `1f64` or `1.0e10f64`
- **LitBool** - Boolean literal: `true` or `false`

---

## Foreign Items (FFI)

Items within `extern` blocks for FFI.

### ForeignItem

Enum for foreign item types:

- **ForeignItemFn** - Foreign function in extern block
- **ForeignItemStatic** - Foreign static in extern block: `static ext: u8`
- **ForeignItemType** - Foreign type in extern block: `type void`
- **ForeignItemMacro** - Macro invocation within extern block

### Supporting Structures

- **ItemForeignMod** - The `extern "C" { ... }` block itself
- **Abi** - Binary interface of a function: `extern "C"`
- **Variadic** - Variadic argument of foreign function

---

## Trait Items

Items within trait definitions.

### TraitItem

Enum for trait item types:

- **TraitItemConst** - Associated constant within trait definition
- **TraitItemFn** - Associated function within trait definition
- **TraitItemType** - Associated type within trait definition
- **TraitItemMacro** - Macro invocation within trait definition

### Supporting Structure

- **ItemTrait** - The trait definition itself: `pub trait Iterator { ... }`

---

## Impl Items

Items within impl blocks.

### ImplItem

Enum for impl item types:

- **ImplItemConst** - Associated constant within impl block
- **ImplItemFn** - Associated function within impl block
- **ImplItemType** - Associated type within impl block
- **ImplItemMacro** - Macro invocation within impl block

### Supporting Structures

- **ItemImpl** - The impl block itself: `impl<A> Trait for Data<A> { ... }`
- **Receiver** - The `self` argument of an associated method
- **ImplRestriction** - Unused, reserved for RFC 3323 restrictions (enum)

---

## Use Declarations

Use statements for importing items.

### UseTree

Enum for use tree types:

- **UsePath** - Path prefix of imports: `std::...`
- **UseName** - Identifier imported: `HashMap`
- **UseRename** - Renamed identifier: `HashMap as Map`
- **UseGlob** - Glob import: `*`
- **UseGroup** - Braced group of imports: `{A, B, C}`

### Supporting Structure

- **ItemUse** - The use declaration itself: `use std::collections::HashMap`

---

## Miscellaneous

Additional types that support the AST.

### Visibility

- **Visibility** - Enum: Public, Restricted, or Inherited
- **VisRestricted** - Restricted visibility: `pub(self)`, `pub(super)`, `pub(crate)`, `pub(in some::module)`

### Function & Method Components

- **FnArg** - Enum: Receiver (self) or Typed (regular parameter)
- **Signature** - Complete function signature
- **Receiver** - The self parameter

### Field-Related

- **Fields** - Enum: Named, Unnamed, or Unit
- **FieldsNamed** - Named fields: `Point { x: f64, y: f64 }`
- **FieldsUnnamed** - Unnamed fields: `Some(T)`
- **FieldMutability** - Unused, reserved for RFC 3323 (enum)

### Member Access

- **Member** - Enum: Named (identifier) or Unnamed (index)
- **Index** - Index of unnamed tuple struct field

### Binary & Unary Operators

- **BinOp** - Enum for binary operators: `+`, `+=`, `&`, etc.
- **UnOp** - Enum for unary operators: `*`, `!`, `-`

### Range

- **RangeLimits** - Enum: HalfOpen (`..`) or Closed (`..=`)

### Mutability

- **StaticMutability** - Enum: Mut or None (for static items)
- **PointerMutability** - Enum: Mut or Const (for raw pointers, where const isn't implicit default)

### Identifiers & Tokens

- **Ident** - A word of Rust code (keyword or variable name)

### Derive Input

- **DeriveInput** - Data structure sent to derive macro
- **Data** - Enum: Struct, Enum, or Union (storage of data structure)

### Error Handling

- **Error** - Error returned when parser fails
- **Result** - Type alias for parser result

---

## Notes on Feature Gates

Many of these types require specific `syn` features:

- **`derive`** (default) - Types for derive macros (structs, enums, basic types)
- **`full`** - All valid Rust syntax (items, expressions, statements)
- **`parsing`** (default) - Ability to parse tokens into syntax tree
- **`printing`** (default) - Ability to print syntax tree as tokens
- **`visit`** - Traversal trait (immutable)
- **`visit-mut`** - Traversal trait (mutable)
- **`fold`** - Transformation trait

Most Oxur work will use the `full` feature to get complete coverage.

---

## Mapping Strategy for Oxur

When designing Oxur's Core Forms, consider:

1. **One-to-One Mappings**: Most items/expressions/types should map directly
2. **S-expression Representation**: Every AST node becomes an s-expression
3. **Homoiconicity**: The s-expression form should be inspectable and transformable
4. **Bidirectional**: Must be able to go from Oxur → Rust AST → Oxur cleanly
5. **Preserve Semantics**: No information loss in round-tripping

### Example Mapping Patterns

```lisp
;; Item::Fn
(define-func process
  ([n usize])
  (-> Result<()>)
  (body ...))

;; Expr::Match
(match-expr n
  [(Some n) (block ...)]
  [None (block ...)])

;; Type::Reference
(ref-type 'a mut T)

;; Pattern::Struct
(struct-pattern Variant
  [x _]
  [y _]
  [.. true])
```

---

## References

- [syn crate documentation](https://docs.rs/syn/2.0.111/syn/)
- [syn GitHub repository](https://github.com/dtolnay/syn)
- [Rust AST Explorer](https://rust-analyzer.github.io/manual.html) (for visualization)
- Oxur Compilation Chain Architecture (Design Doc #13)

---

*End of AST Inventory*
