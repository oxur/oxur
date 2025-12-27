use super::{NodeId, Span, TokenStream};

/// Identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

impl Ident {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self { name: name.into(), span }
    }
}

/// Path to an item
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub span: Span,
    pub segments: Vec<PathSegment>,
    pub tokens: Option<TokenStream>,
}

impl Path {
    pub fn from_ident(ident: Ident) -> Self {
        Self { span: ident.span, segments: vec![PathSegment::from_ident(ident)], tokens: None }
    }
}

/// Path segment
#[derive(Debug, Clone, PartialEq)]
pub struct PathSegment {
    pub ident: Ident,
    pub id: NodeId,
    pub args: Option<GenericArgs>,
}

impl PathSegment {
    pub fn from_ident(ident: Ident) -> Self {
        Self { ident, id: NodeId::DUMMY, args: None }
    }
}

/// Generic arguments (placeholder for Phase 1)
#[derive(Debug, Clone, PartialEq)]
pub struct GenericArgs {
    // Simplified for Phase 1
}

/// Visibility
#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Restricted { path: Box<Path>, shorthand: VisRestrictionKind, span: Span },
    Inherited,
}

/// Visibility restriction kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisRestrictionKind {
    Crate,
    Super,
    In,
}
