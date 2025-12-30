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

    // Future: ExternCrate, Use, Static, Const, Mod, etc.
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
    Ty(Ty),        // Explicit return type
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

/// Generic parameter (placeholder for Phase 1)
#[derive(Debug, Clone, PartialEq)]
pub struct GenericParam {
    // Simplified for Phase 1
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

/// Where predicate (placeholder for Phase 1)
#[derive(Debug, Clone, PartialEq)]
pub struct WherePredicate {
    // Simplified for Phase 1
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
    // Phase 1: Only path types (e.g., i32, String)
    Path(Option<QSelf>, Path),
    // Future: Ptr, Ref, Array, Tup, etc.
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
    // Phase 1: Only identifier patterns
    Ident { binding_mode: BindingMode, ident: Ident, sub: Option<Box<Pat>> },
    // Future: Struct, TupleStruct, Tuple, Slice, etc.
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
    Type(Option<Ty>), // None for type declarations without default
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
    Trait(TraitRef),
    // Future: Lifetime bounds
}
