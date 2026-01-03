use crate::ast::*;
use crate::error::Result;
use crate::gen_sexp::gen::Generator;
use crate::gen_sexp::helpers::*;
use crate::sexp::SExp;

impl Generator {
    pub fn generate_stmt(&self, stmt: &Stmt) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("id", self.generate_node_id(stmt.id)),
            kwarg("kind", self.generate_stmt_kind(&stmt.kind)?),
            kwarg("span", self.generate_span(stmt.span)),
        ]);

        Ok(typed_node("Stmt", fields))
    }

    fn generate_stmt_kind(&self, kind: &StmtKind) -> Result<SExp> {
        match kind {
            StmtKind::Expr(expr) => Ok(list(vec![sym("Expr"), self.generate_expr(expr)?])),
            StmtKind::Semi(expr) => Ok(list(vec![sym("Semi"), self.generate_expr(expr)?])),
            StmtKind::Let(local) => Ok(list(vec![sym("Let"), self.generate_local(local)?])),
            StmtKind::Item(item) => Ok(list(vec![sym("Item"), self.generate_item(item)?])),
            StmtKind::MacCall(mac_call_stmt) => {
                Ok(list(vec![sym("MacCall"), self.generate_mac_call_stmt(mac_call_stmt)?]))
            }
            StmtKind::Empty => Ok(sym("Empty")),
        }
    }

    fn generate_local(&self, local: &Local) -> Result<SExp> {
        let mut fields = kwargs(vec![
            kwarg("pat", self.generate_pat(&local.pat)?),
            kwarg("kind", self.generate_local_kind(&local.kind)?),
            kwarg("span", self.generate_span(local.span)),
            kwarg("attrs", self.generate_attr_vec(&local.attrs)?),
        ]);

        // ty is optional
        if let Some(ty) = &local.ty {
            fields.extend(kwarg("ty", self.generate_ty(ty)?));
        } else {
            fields.extend(kwarg("ty", sym("nil")));
        }

        // Only include tokens if present
        if local.tokens.is_some() {
            fields.extend(kwarg("tokens", sym("nil")));
        }

        Ok(typed_node("Local", fields))
    }

    fn generate_local_kind(&self, kind: &LocalKind) -> Result<SExp> {
        match kind {
            LocalKind::Decl => Ok(sym("Decl")),
            LocalKind::Init(init) => Ok(list(vec![sym("Init"), self.generate_local_init(init)?])),
            LocalKind::InitElse(init, block) => Ok(list(vec![
                sym("InitElse"),
                self.generate_local_init(init)?,
                self.generate_block(block)?,
            ])),
        }
    }

    fn generate_local_init(&self, init: &LocalInit) -> Result<SExp> {
        let mut fields = kwargs(vec![kwarg("expr", self.generate_expr(&init.expr)?)]);

        // els is optional
        if let Some(els) = &init.els {
            fields.extend(kwarg("els", self.generate_block(els)?));
        } else {
            fields.extend(kwarg("els", sym("nil")));
        }

        Ok(typed_node("LocalInit", fields))
    }

    fn generate_mac_call_stmt(&self, stmt: &MacCallStmt) -> Result<SExp> {
        let mut fields = kwargs(vec![
            kwarg("mac", self.generate_mac_call(&stmt.mac)?),
            kwarg("style", self.generate_mac_stmt_style(stmt.style)),
            kwarg("attrs", self.generate_attr_vec(&stmt.attrs)?),
        ]);

        // Only include tokens if present
        if stmt.tokens.is_some() {
            fields.extend(kwarg("tokens", sym("nil")));
        }

        Ok(typed_node("MacCallStmt", fields))
    }

    fn generate_mac_stmt_style(&self, style: MacStmtStyle) -> SExp {
        match style {
            MacStmtStyle::Semicolon => sym("Semicolon"),
            MacStmtStyle::Braces => sym("Braces"),
            MacStmtStyle::NoBraces => sym("NoBraces"),
        }
    }
}
