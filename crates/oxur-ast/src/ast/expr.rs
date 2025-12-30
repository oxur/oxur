use super::*;

/// Expression
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
    pub span: Span,
    pub attrs: AttrVec,
    pub tokens: Option<TokenStream>,
}

/// Expression kind
#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    // Phase 1: Macro calls, literals, paths
    MacCall(MacCall),
    Lit(Lit),
    Path(Option<QSelf>, Path),

    // Stage 2: Control flow
    If { cond: Box<Expr>, then_branch: Block, else_branch: Option<Box<Expr>> },
    Match { expr: Box<Expr>, arms: Vec<Arm> },
    While { label: Option<Label>, cond: Box<Expr>, body: Block },
    ForLoop { label: Option<Label>, pat: Pat, iter: Box<Expr>, body: Block },
    Loop { label: Option<Label>, body: Block },

    // Stage 3: Operations and calls
    Binary { left: Box<Expr>, op: BinOp, right: Box<Expr> },
    Unary { op: UnOp, expr: Box<Expr> },
    Call { func: Box<Expr>, args: Vec<Expr> },
    MethodCall { receiver: Box<Expr>, method: Ident, args: Vec<Expr> },
    // Future: Array, Field, Index, Assign, Closure, etc.
}

/// Macro call
#[derive(Debug, Clone, PartialEq)]
pub struct MacCall {
    pub path: Path,
    pub args: MacArgs,
    pub prior_type_ascription: Option<(usize, bool)>,
}

impl MacCall {
    pub fn new(path: Path, args: MacArgs) -> Self {
        Self { path, args, prior_type_ascription: None }
    }
}

/// Macro arguments
#[derive(Debug, Clone, PartialEq)]
pub enum MacArgs {
    Empty,
    Delimited { dspan: DelSpan, delim: Delimiter, tokens: TokenStream },
    Eq { eq_span: Span, tokens: TokenStream },
}

/// Delimiter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    Paren,
    Brace,
    Bracket,
    Invisible,
}

/// Literal
#[derive(Debug, Clone, PartialEq)]
pub struct Lit {
    pub kind: LitKind,
    pub span: Span,
}

/// Literal kind
#[derive(Debug, Clone, PartialEq)]
pub enum LitKind {
    Str(String),
    Int(i128),
    // Future: Float, Char, Bool, Byte, ByteStr, etc.
}

/// Match arm
#[derive(Debug, Clone, PartialEq)]
pub struct Arm {
    pub attrs: AttrVec,
    pub pat: Pat,
    pub guard: Option<Box<Expr>>,
    pub body: Box<Expr>,
    pub span: Span,
    pub id: NodeId,
}

/// Loop label (e.g., 'outer in 'outer: loop { ... })
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label {
    pub ident: Ident,
}

/// Binary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,    // +
    Sub,    // -
    Mul,    // *
    Div,    // /
    Rem,    // %
    And,    // &&
    Or,     // ||
    BitAnd, // &
    BitOr,  // |
    BitXor, // ^
    Shl,    // <<
    Shr,    // >>
    Eq,     // ==
    Ne,     // !=
    Lt,     // <
    Le,     // <=
    Gt,     // >
    Ge,     // >=
}

/// Unary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Not,   // !
    Neg,   // -
    Deref, // *
}
