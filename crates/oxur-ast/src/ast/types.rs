/// Node identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

impl NodeId {
    pub const DUMMY: NodeId = NodeId(u32::MAX);
}

/// Attribute vector
pub type AttrVec = Vec<Attribute>;

/// Simplified attribute (Phase 1)
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    // Placeholder for Phase 1
    // In full implementation, this would include:
    // - kind: AttrKind
    // - id: AttrId
    // - span: Span
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
