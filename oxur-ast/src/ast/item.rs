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
