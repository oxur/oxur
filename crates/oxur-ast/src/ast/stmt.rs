use super::*;

/// Statement
#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub id: NodeId,
    pub kind: StmtKind,
    pub span: Span,
}

/// Statement kind
#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Expr(Expr),
    Semi(Expr),
    Let(Box<Local>),
    Item(Item),
    MacCall(MacCallStmt),
    Empty,
}

/// Local variable declaration (let binding)
#[derive(Debug, Clone, PartialEq)]
pub struct Local {
    pub pat: Pat,
    pub ty: Option<Ty>,
    pub kind: LocalKind,
    pub span: Span,
    pub attrs: AttrVec,
    pub tokens: Option<TokenStream>,
}

/// Local kind
#[derive(Debug, Clone, PartialEq)]
pub enum LocalKind {
    Decl,
    Init(LocalInit),
    InitElse(LocalInit, Block),
}

/// Local initialization
#[derive(Debug, Clone, PartialEq)]
pub struct LocalInit {
    pub expr: Expr,
    pub els: Option<Block>,
}

/// Macro call statement
#[derive(Debug, Clone, PartialEq)]
pub struct MacCallStmt {
    pub mac: MacCall,
    pub style: MacStmtStyle,
    pub attrs: AttrVec,
    pub tokens: Option<TokenStream>,
}

/// Macro statement style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacStmtStyle {
    Semicolon,
    Braces,
    NoBraces,
}
