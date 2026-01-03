use super::expr::MacArgs;
use super::path::Path;
use super::span::Span;

/// Node identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

impl NodeId {
    pub const DUMMY: NodeId = NodeId(u32::MAX);
}

/// Attribute vector
pub type AttrVec = Vec<Attribute>;

/// Attribute (Phase 8.5)
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub kind: AttrKind,
    pub id: AttrId,
    pub style: AttrStyle,
    pub span: Span,
}

pub type AttrId = usize;

/// Attribute style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrStyle {
    Outer, // #[...]
    Inner, // #![...]
}

/// Attribute kind
#[derive(Debug, Clone, PartialEq)]
pub enum AttrKind {
    Normal(NormalAttr),
    DocComment(CommentKind, String),
}

/// Normal attribute
#[derive(Debug, Clone, PartialEq)]
pub struct NormalAttr {
    pub item: AttrItem,
}

/// Attribute item
#[derive(Debug, Clone, PartialEq)]
pub struct AttrItem {
    pub path: Path,
    pub args: MacArgs, // Reuse MacArgs from expressions
}

/// Comment kind (for doc comments)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentKind {
    Line,  // /// or //!
    Block, // /** */ or /*! */
}

/// Token stream (simplified for Phase 1)
#[derive(Debug, Clone, PartialEq)]
pub enum TokenStream {
    Source(String), // Raw source string
    Empty,
}

/// Defaultness
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Defaultness {
    Default,
    Final,
}

/// Safety
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Safety {
    Safe,
    Unsafe,
    Default,
}

/// Constness
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constness {
    Const,
    NotConst,
}

/// Extern ABI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Extern {
    None,
    Explicit(String), // ABI string like "C"
}

/// Coroutine kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoroutineKind {
    Async,
    Gen,
}
