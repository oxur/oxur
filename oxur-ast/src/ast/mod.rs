pub mod expr;
pub mod item;
pub mod path;
pub mod span;
pub mod stmt;
pub mod types;

pub use expr::*;
pub use item::*;
pub use path::*;
pub use span::*;
pub use stmt::*;
pub use types::*;

/// The root of the AST
#[derive(Debug, Clone, PartialEq)]
pub struct Crate {
    pub attrs: AttrVec,
    pub items: Vec<Item>,
    pub spans: ModSpans,
    pub id: NodeId,
    pub is_placeholder: bool,
}

impl Crate {
    pub fn new(items: Vec<Item>, spans: ModSpans, id: NodeId) -> Self {
        Self { attrs: Vec::new(), items, spans, id, is_placeholder: false }
    }
}
