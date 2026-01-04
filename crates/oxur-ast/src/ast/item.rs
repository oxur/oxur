use super::*;

/// Item (top-level declaration)
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub attrs: AttrVec,
    pub id: NodeId,
    pub span: Span,
    pub vis: Visibility,
    pub ident: Ident,
    pub kind: ItemKind,
    pub tokens: Option<TokenStream>,
}

/// Item kind
#[derive(Debug, Clone, PartialEq)]
pub enum ItemKind {
    // Phase 1: Only function items
    Fn(Box<Fn>),

    // Stage 4: Struct and Enum items
    Struct(VariantData),
    Enum(EnumDef),

    // Stage 5: Trait and Impl items
    Trait(Box<TraitDef>),
    Impl(Box<ImplDef>),

    // Stage 8: Remaining items
    /// Use declaration: `use std::io;`, `use foo::bar as baz;`
    Use(UseTree),

    /// Static item: `static X: i32 = 5;`
    Static {
        mutability: Mutability,
        ty: Ty,
        expr: Option<Expr>,
    },

    /// Const item: `const X: i32 = 5;`
    Const {
        ty: Ty,
        expr: Option<Expr>,
    },

    /// Type alias: `type Foo = Bar;`
    TyAlias {
        generics: Generics,
        ty: Option<Ty>,
    },

    /// Module: `mod foo;` or `mod foo { ... }`
    Mod {
        items: Option<Vec<Item>>,
    }, // None for `mod foo;`, Some for `mod foo { ... }`

    /// Declarative macro definition: `macro_rules! name { ... }` (Phase 8.5)
    MacroDef(Box<MacroDef>),
    // Future: ExternCrate, ForeignMod, etc.
}

/// Declarative macro definition (Phase 8.5)
#[derive(Debug, Clone, PartialEq)]
pub struct MacroDef {
    pub macro_rules: bool, // true for macro_rules!, false for macro
    pub body: MacArgs,     // Token stream of macro body
}

/// Function item
#[derive(Debug, Clone, PartialEq)]
pub struct Fn {
    pub defaultness: Defaultness,
    pub sig: FnSig,
    pub generics: Generics,
    pub body: Option<Block>,
}

/// Function signature
#[derive(Debug, Clone, PartialEq)]
pub struct FnSig {
    pub header: FnHeader,
    pub decl: FnDecl,
    pub span: Span,
}

/// Function header
#[derive(Debug, Clone, PartialEq)]
pub struct FnHeader {
    pub safety: Safety,
    pub coroutine_kind: Option<CoroutineKind>,
    pub constness: Constness,
    pub ext: Extern,
}

/// Function declaration (parameters and return type)
#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    pub inputs: Vec<Param>,
    pub output: FnRetTy,
}

/// Function parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub attrs: AttrVec,
    pub ty: Ty,
    pub pat: Pat,
    pub id: NodeId,
    pub span: Span,
    pub is_placeholder: bool,
}

/// Function return type
#[derive(Debug, Clone, PartialEq)]
pub enum FnRetTy {
    Default(Span), // No return type (unit)
    Ty(Box<Ty>),   // Explicit return type
}

/// Generics
#[derive(Debug, Clone, PartialEq)]
pub struct Generics {
    pub params: Vec<GenericParam>,
    pub where_clause: WhereClause,
    pub span: Span,
}

impl Generics {
    pub fn empty() -> Self {
        Self { params: Vec::new(), where_clause: WhereClause::empty(), span: Span::DUMMY }
    }
}

/// Generic parameter
#[derive(Debug, Clone, PartialEq)]
pub struct GenericParam {
    pub attrs: AttrVec,
    pub id: NodeId,
    pub span: Span,
    pub kind: GenericParamKind,
}

/// Generic parameter kind
#[derive(Debug, Clone, PartialEq)]
pub enum GenericParamKind {
    /// Lifetime parameter: `'a`, `'a: 'b`
    Lifetime(LifetimeParam),
    /// Type parameter: `T`, `T: Clone`, `T = i32`
    Type(TypeParam),
    /// Const parameter: `const N: usize`, `const N: usize = 5`
    Const(Box<ConstParam>),
}

/// Lifetime parameter
#[derive(Debug, Clone, PartialEq)]
pub struct LifetimeParam {
    pub ident: Ident,
    /// Lifetime bounds: `'a: 'b + 'c`
    pub bounds: Vec<Lifetime>,
    pub colon_span: Option<Span>,
}

/// Type parameter
#[derive(Debug, Clone, PartialEq)]
pub struct TypeParam {
    pub ident: Ident,
    /// Trait bounds: `T: Clone + Debug`
    pub bounds: Vec<GenericBound>,
    /// Default type: `T = i32`
    pub default: Option<Ty>,
}

/// Const parameter
#[derive(Debug, Clone, PartialEq)]
pub struct ConstParam {
    pub ident: Ident,
    pub ty: Ty,
    /// Default value: `const N: usize = 5`
    pub default: Option<Expr>,
}

/// Where clause
#[derive(Debug, Clone, PartialEq)]
pub struct WhereClause {
    pub has_where_token: bool,
    pub predicates: Vec<WherePredicate>,
    pub span: Span,
}

impl WhereClause {
    pub fn empty() -> Self {
        Self { has_where_token: false, predicates: Vec::new(), span: Span::DUMMY }
    }
}

/// Where predicate
#[derive(Debug, Clone, PartialEq)]
pub enum WherePredicate {
    /// Bound predicate: `T: Clone + Debug`, `C::Item: Display`
    BoundPredicate(WhereBoundPredicate),
    /// Region (lifetime) predicate: `'a: 'b + 'c`
    RegionPredicate(WhereRegionPredicate),
    /// Equality predicate: `T = int` (associated type equality)
    EqPredicate(WhereEqPredicate),
}

/// Bound predicate: `T: Clone`
#[derive(Debug, Clone, PartialEq)]
pub struct WhereBoundPredicate {
    pub span: Span,
    /// The type being bounded: `T`, `C::Item`
    pub bounded_ty: Ty,
    /// Trait bounds: `Clone + Debug`
    pub bounds: Vec<GenericBound>,
    /// Higher-ranked trait bounds: `for<'a>`
    pub bound_lifetimes: Vec<LifetimeParam>,
}

/// Region (lifetime) predicate: `'a: 'b + 'c`
#[derive(Debug, Clone, PartialEq)]
pub struct WhereRegionPredicate {
    pub span: Span,
    /// The lifetime being bounded: `'a`
    pub lifetime: Lifetime,
    /// Lifetime bounds: `'b`, `'c`
    pub bounds: Vec<Lifetime>,
}

/// Equality predicate: `T = int` (associated type equality)
#[derive(Debug, Clone, PartialEq)]
pub struct WhereEqPredicate {
    pub span: Span,
    /// Left side: `T`
    pub lhs_ty: Ty,
    /// Right side: `int`
    pub rhs_ty: Ty,
}

/// Type
#[derive(Debug, Clone, PartialEq)]
pub struct Ty {
    pub id: NodeId,
    pub kind: TyKind,
    pub span: Span,
    pub tokens: Option<TokenStream>,
}

/// Type kind
#[derive(Debug, Clone, PartialEq)]
pub enum TyKind {
    // Phase 1: Path types (e.g., i32, String)
    Path(Option<QSelf>, Path),

    // Stage 6: Advanced types
    /// Reference type: `&T`, `&mut T`, `&'a T`, `&'a mut T`
    Ref {
        lifetime: Option<Lifetime>,
        mutability: Mutability,
        ty: Box<Ty>,
    },

    /// Raw pointer type: `*const T`, `*mut T`
    Ptr {
        mutability: Mutability,
        ty: Box<Ty>,
    },

    /// Array type: `[T; N]`
    Array {
        ty: Box<Ty>,
        len: Box<Expr>,
    },

    /// Slice type: `[T]`
    Slice(Box<Ty>),

    /// Tuple type: `(A, B, C)`
    Tuple(Vec<Ty>),

    /// Never type: `!`
    Never,

    /// Infer type: `_`
    Infer,

    // Phase 5: Complete type coverage
    /// Bare function type: `fn(i32) -> String`, `unsafe fn() -> !`
    BareFn {
        safety: Safety,
        abi: Option<String>,
        inputs: Vec<BareFnParam>,
        output: FnRetTy,
    },

    /// Impl trait type: `impl Iterator<Item = u8>`
    ImplTrait(Vec<GenericBound>),

    /// Trait object type: `dyn Display + Send`
    TraitObject {
        bounds: Vec<GenericBound>,
        syntax: TraitObjectSyntax,
    },

    /// Macro invocation in type position: `vec![T]`
    MacCall(MacCall),

    /// Parenthesized type: `(T)`
    Paren(Box<Ty>),
}

/// Lifetime (e.g., `'a`, `'static`)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lifetime {
    pub ident: Ident,
}

/// Qualified self (for associated types)
#[derive(Debug, Clone, PartialEq)]
pub struct QSelf {
    // Simplified for Phase 1
}

/// Pattern
#[derive(Debug, Clone, PartialEq)]
pub struct Pat {
    pub id: NodeId,
    pub kind: PatKind,
    pub span: Span,
    pub tokens: Option<TokenStream>,
}

/// Pattern kind
#[derive(Debug, Clone, PartialEq)]
pub enum PatKind {
    // Phase 1: Identifier patterns
    Ident {
        binding_mode: BindingMode,
        ident: Ident,
        sub: Option<Box<Pat>>,
    },

    // Stage 6: Advanced patterns
    /// Wildcard pattern: `_`
    Wild,

    /// Struct pattern: `Point { x, y }`
    Struct {
        path: Path,
        fields: Vec<PatField>,
    },

    /// Tuple struct pattern: `Some(x)`
    TupleStruct {
        path: Path,
        elems: Vec<Pat>,
    },

    /// Tuple pattern: `(a, b, c)`
    Tuple(Vec<Pat>),

    /// Slice pattern: `[a, b, .., c]`
    Slice(Vec<Pat>),

    /// Or pattern: `Some(x) | None`
    Or(Vec<Pat>),

    /// Reference pattern: `&x`, `&mut x`
    Ref {
        pat: Box<Pat>,
        mutability: Mutability,
    },

    /// Literal pattern: `42`, `"hello"`
    Lit(Box<Expr>),

    // Phase 5: Complete pattern coverage
    /// Box pattern: `box pat`
    Box(Box<Pat>),

    /// Path pattern: `None`, `Some`, `std::option::Option::None`
    Path {
        qself: Option<QSelf>,
        path: Path,
    },

    /// Range pattern: `1..=5`, `..=10`, `5..`
    Range {
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        limits: RangeEnd,
    },

    /// Rest pattern: `..` in a tuple or slice
    Rest,

    /// Parenthesized pattern: `(pat)`
    Paren(Box<Pat>),

    /// Type ascription: `x: i32`
    Type {
        pat: Box<Pat>,
        ty: Box<Ty>,
    },

    /// Const block: `const { expr }`
    Const(ConstBlock),

    /// Macro invocation
    MacCall(MacCall),

    /// Error recovery
    Err,
}

/// Pattern field (for struct patterns)
#[derive(Debug, Clone, PartialEq)]
pub struct PatField {
    pub attrs: AttrVec,
    pub ident: Ident,
    pub pat: Pat,
    pub is_shorthand: bool, // true for `Point { x, y }` vs `Point { x: x, y: y }`
    pub span: Span,
}

/// Binding mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingMode {
    ByRef(Mutability),
    ByValue(Mutability),
}

/// Mutability
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Mut,
    Not,
}

/// Range end style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeEnd {
    /// `..=` (inclusive)
    Included,
    /// `..` (exclusive)
    Excluded,
}

/// Const block in pattern position
#[derive(Debug, Clone, PartialEq)]
pub struct ConstBlock {
    pub id: NodeId,
    pub value: Box<Expr>,
}

/// Block (forward declaration - defined in expr.rs)
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub id: NodeId,
    pub rules: BlockCheckMode,
    pub span: Span,
    pub tokens: Option<TokenStream>,
    pub could_be_bare_literal: bool,
}

/// Block check mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockCheckMode {
    Default,
    Unsafe,
}

impl Block {
    pub fn new(stmts: Vec<Stmt>, id: NodeId, span: Span) -> Self {
        Self {
            stmts,
            id,
            rules: BlockCheckMode::Default,
            span,
            tokens: None,
            could_be_bare_literal: false,
        }
    }
}

// Stmt and StmtKind are defined in stmt.rs

/// Stage 4: Struct and Enum types
/// Variant data (for structs and enum variants)
#[derive(Debug, Clone, PartialEq)]
pub enum VariantData {
    /// Struct with named fields: `struct Point { x: i32, y: i32 }`
    Struct { fields: Vec<FieldDef>, recovered: bool },

    /// Tuple struct: `struct Color(u8, u8, u8)`
    Tuple(Vec<FieldDef>),

    /// Unit struct: `struct Marker;`
    Unit,
}

/// Enum definition
#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub variants: Vec<Variant>,
}

/// Enum variant
#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub attrs: AttrVec,
    pub id: NodeId,
    pub span: Span,
    pub vis: Visibility,
    pub ident: Ident,
    pub data: VariantData,
    /// Explicit discriminant: `Foo = 1`
    pub disr_expr: Option<Expr>,
}

/// Field definition (for structs and enum variants)
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDef {
    pub attrs: AttrVec,
    pub id: NodeId,
    pub span: Span,
    pub vis: Visibility,
    pub ident: Option<Ident>, // None for tuple fields
    pub ty: Ty,
}

/// Stage 5: Trait and Impl types
/// Trait definition
#[derive(Debug, Clone, PartialEq)]
pub struct TraitDef {
    pub safety: Safety,
    pub generics: Generics,
    pub bounds: Vec<GenericBound>,
    pub items: Vec<AssocItem>,
}

/// Implementation definition
#[derive(Debug, Clone, PartialEq)]
pub struct ImplDef {
    pub safety: Safety,
    pub generics: Generics,
    pub of_trait: Option<TraitRef>, // None for inherent impls
    pub self_ty: Ty,
    pub items: Vec<AssocItem>,
}

/// Associated item (in traits and impls)
#[derive(Debug, Clone, PartialEq)]
pub struct AssocItem {
    pub attrs: AttrVec,
    pub id: NodeId,
    pub span: Span,
    pub vis: Visibility,
    pub ident: Ident,
    pub kind: AssocItemKind,
}

/// Associated item kind
#[derive(Debug, Clone, PartialEq)]
pub enum AssocItemKind {
    /// Associated function: `fn foo() { ... }`
    Fn(Box<Fn>),
    /// Associated type: `type Foo = Bar;`
    Type(Box<Option<Ty>>), // None for type declarations without default
                           // Future: Const, MacCall
}

/// Trait reference (for trait bounds and impl blocks)
#[derive(Debug, Clone, PartialEq)]
pub struct TraitRef {
    pub path: Path,
}

/// Generic bound (for trait bounds)
#[derive(Debug, Clone, PartialEq)]
pub enum GenericBound {
    /// Trait bound: `Clone`, `?Sized`, `!Send`
    Trait(PolyTraitRef, TraitBoundModifier),
    /// Lifetime bound: `'a`
    Outlives(Lifetime),
}

/// Polymorphic trait reference (with higher-ranked trait bounds)
#[derive(Debug, Clone, PartialEq)]
pub struct PolyTraitRef {
    /// Trait reference: `Clone`, `Iterator<Item = i32>`
    pub trait_ref: TraitRef,
    /// Higher-ranked lifetimes: `for<'a>`
    pub bound_lifetimes: Vec<LifetimeParam>,
}

/// Trait bound modifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraitBoundModifier {
    /// No modifier
    None,
    /// Maybe bound: `?Sized`
    Maybe,
    /// Negative bound: `!Send` (unstable)
    Negative,
}

// Phase 5: Type system supporting types
/// Bare function parameter: `x: i32` in `fn(x: i32) -> bool`
#[derive(Debug, Clone, PartialEq)]
pub struct BareFnParam {
    pub attrs: AttrVec,
    pub name: Option<Ident>,
    pub ty: Ty,
}

/// Trait object syntax style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraitObjectSyntax {
    /// `dyn Trait`
    Dyn,
    /// No `dyn` keyword (deprecated but still valid)
    None,
}

/// Stage 8: Use tree types
/// Use tree: represents the structure of a use declaration
#[derive(Debug, Clone, PartialEq)]
pub struct UseTree {
    pub prefix: Path,
    pub kind: UseTreeKind,
}

/// Use tree kind
#[derive(Debug, Clone, PartialEq)]
pub enum UseTreeKind {
    /// Simple: `use foo::bar;`
    Simple(Option<Ident>), // None for no rename, Some(ident) for `as ident`

    /// Glob: `use foo::*;`
    Glob,

    /// Nested: `use foo::{bar, baz};`
    Nested(Vec<UseTree>),
}
