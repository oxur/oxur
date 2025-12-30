use crate::ast::*;
use crate::error::Result;
use crate::generator::gen::Generator;
use crate::generator::helpers::*;
use crate::sexp::SExp;

impl Generator {
    pub fn generate_block(&self, block: &Block) -> Result<SExp> {
        let mut fields = kwargs(vec![
            kwarg("stmts", self.generate_stmts(&block.stmts)?),
            kwarg("id", self.generate_node_id(block.id)),
            kwarg("rules", self.generate_block_check_mode(block.rules)),
            kwarg("span", self.generate_span(block.span)),
        ]);

        // Only include optional fields if present
        if block.tokens.is_some() {
            fields.extend(kwarg("tokens", sym("nil")));
        }
        fields.extend(kwarg(
            "could-be-bare-literal",
            sym(if block.could_be_bare_literal { "true" } else { "false" }),
        ));

        Ok(typed_node("Block", fields))
    }

    fn generate_block_check_mode(&self, mode: BlockCheckMode) -> SExp {
        match mode {
            BlockCheckMode::Default => sym("Default"),
            BlockCheckMode::Unsafe => sym("Unsafe"),
        }
    }

    fn generate_stmts(&self, stmts: &[Stmt]) -> Result<SExp> {
        let stmt_sexps: Result<Vec<SExp>> =
            stmts.iter().map(|stmt| self.generate_stmt(stmt)).collect();
        Ok(list(stmt_sexps?))
    }

    pub fn generate_expr(&self, expr: &Expr) -> Result<SExp> {
        let mut fields = kwargs(vec![
            kwarg("id", self.generate_node_id(expr.id)),
            kwarg("kind", self.generate_expr_kind(&expr.kind)?),
            kwarg("span", self.generate_span(expr.span)),
            kwarg("attrs", self.generate_attr_vec(&expr.attrs)?),
        ]);

        // Only include tokens if present
        if expr.tokens.is_some() {
            fields.extend(kwarg("tokens", sym("nil")));
        }

        Ok(typed_node("Expr", fields))
    }

    fn generate_expr_kind(&self, kind: &ExprKind) -> Result<SExp> {
        match kind {
            ExprKind::MacCall(mac_call) => {
                Ok(list(vec![sym("MacCall"), self.generate_mac_call(mac_call)?]))
            }
            ExprKind::Lit(lit) => Ok(list(vec![sym("Lit"), self.generate_lit(lit)])),
            ExprKind::Path(qself, path) => {
                let qself_sexp = if qself.is_some() {
                    sym("TODO") // TODO: implement QSelf
                } else {
                    sym("nil")
                };
                Ok(list(vec![sym("Path"), qself_sexp, self.generate_path(path)]))
            }
            ExprKind::If { cond, then_branch, else_branch } => {
                let mut fields = kwargs(vec![
                    kwarg("cond", self.generate_expr(cond)?),
                    kwarg("then", self.generate_block(then_branch)?),
                ]);
                if let Some(else_expr) = else_branch {
                    fields.extend(kwarg("else", self.generate_expr(else_expr)?));
                } else {
                    fields.extend(kwarg("else", sym("nil")));
                }
                Ok(list(vec![sym("If")].into_iter().chain(fields).collect()))
            }
            ExprKind::Match { expr, arms } => {
                let arms_sexp = list(arms.iter().map(|arm| self.generate_arm(arm)).collect());
                Ok(list(vec![
                    sym("Match"),
                    kw("expr"), self.generate_expr(expr)?,
                    kw("arms"), arms_sexp,
                ]))
            }
            ExprKind::While { label, cond, body } => {
                let label_sexp = label.as_ref()
                    .map(|l| self.generate_label(l))
                    .unwrap_or_else(|| sym("nil"));
                Ok(list(vec![
                    sym("While"),
                    kw("label"), label_sexp,
                    kw("cond"), self.generate_expr(cond)?,
                    kw("body"), self.generate_block(body)?,
                ]))
            }
            ExprKind::ForLoop { label, pat, iter, body } => {
                let label_sexp = label.as_ref()
                    .map(|l| self.generate_label(l))
                    .unwrap_or_else(|| sym("nil"));
                Ok(list(vec![
                    sym("ForLoop"),
                    kw("label"), label_sexp,
                    kw("pat"), self.generate_pat(pat)?,
                    kw("iter"), self.generate_expr(iter)?,
                    kw("body"), self.generate_block(body)?,
                ]))
            }
            ExprKind::Loop { label, body } => {
                let label_sexp = label.as_ref()
                    .map(|l| self.generate_label(l))
                    .unwrap_or_else(|| sym("nil"));
                Ok(list(vec![
                    sym("Loop"),
                    kw("label"), label_sexp,
                    kw("body"), self.generate_block(body)?,
                ]))
            }
        }
    }

    pub fn generate_mac_call(&self, mac_call: &MacCall) -> Result<SExp> {
        let mut fields = kwargs(vec![
            kwarg("path", self.generate_path(&mac_call.path)),
            kwarg("args", self.generate_mac_args(&mac_call.args)?),
        ]);

        // prior_type_ascription is optional
        if mac_call.prior_type_ascription.is_some() {
            fields.extend(kwarg("prior-type-ascription", sym("TODO")));
        } else {
            fields.extend(kwarg("prior-type-ascription", sym("nil")));
        }

        Ok(typed_node("MacCall", fields))
    }

    fn generate_mac_args(&self, args: &MacArgs) -> Result<SExp> {
        match args {
            MacArgs::Empty => Ok(sym("Empty")),
            MacArgs::Delimited { dspan, delim, tokens } => {
                let fields = kwargs(vec![
                    kwarg("dspan", self.generate_del_span(*dspan)),
                    kwarg("delim", self.generate_delimiter(*delim)),
                    kwarg("tokens", self.generate_token_stream(tokens)),
                ]);
                Ok(typed_node("Delimited", fields))
            }
            MacArgs::Eq { eq_span, tokens } => {
                let fields = kwargs(vec![
                    kwarg("eq-span", self.generate_span(*eq_span)),
                    kwarg("tokens", self.generate_token_stream(tokens)),
                ]);
                Ok(typed_node("Eq", fields))
            }
        }
    }

    fn generate_del_span(&self, dspan: DelSpan) -> SExp {
        let fields = kwargs(vec![
            kwarg("open", self.generate_span(dspan.open)),
            kwarg("close", self.generate_span(dspan.close)),
        ]);

        typed_node("DelSpan", fields)
    }

    fn generate_delimiter(&self, delim: Delimiter) -> SExp {
        match delim {
            Delimiter::Paren => sym("Paren"),
            Delimiter::Brace => sym("Brace"),
            Delimiter::Bracket => sym("Bracket"),
            Delimiter::Invisible => sym("Invisible"),
        }
    }

    fn generate_token_stream(&self, tokens: &TokenStream) -> SExp {
        match tokens {
            TokenStream::Source(source) => {
                let fields = kwargs(vec![kwarg("source", string(source))]);
                typed_node("TokenStream", fields)
            }
            TokenStream::Empty => sym("Empty"),
        }
    }

    fn generate_lit(&self, lit: &Lit) -> SExp {
        let fields = kwargs(vec![
            kwarg("kind", self.generate_lit_kind(&lit.kind)),
            kwarg("span", self.generate_span(lit.span)),
        ]);

        typed_node("Lit", fields)
    }

    fn generate_lit_kind(&self, kind: &LitKind) -> SExp {
        match kind {
            LitKind::Str(s) => list(vec![sym("Str"), string(s)]),
            LitKind::Int(i) => list(vec![sym("Int"), num(*i)]),
        }
    }

    fn generate_arm(&self, arm: &Arm) -> SExp {
        let mut fields = kwargs(vec![
            kwarg("pat", self.generate_pat(&arm.pat).unwrap_or_else(|_| sym("ERROR"))),
            kwarg("body", self.generate_expr(&arm.body).unwrap_or_else(|_| sym("ERROR"))),
        ]);

        if let Some(guard) = &arm.guard {
            fields.extend(kwarg("guard", self.generate_expr(guard).unwrap_or_else(|_| sym("ERROR"))));
        } else {
            fields.extend(kwarg("guard", sym("nil")));
        }

        fields.extend(kwargs(vec![
            kwarg("id", num(arm.id.0 as i128)),
            kwarg("span", self.generate_span(arm.span)),
        ]));

        typed_node("Arm", fields)
    }

    fn generate_label(&self, label: &Label) -> SExp {
        typed_node("Label", kwargs(vec![
            kwarg("ident", self.generate_ident(&label.ident)),
        ]))
    }
}
