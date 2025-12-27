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
    // Future: Array, Call, MethodCall, Binary, If, Match, etc.
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
