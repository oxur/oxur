//! Expression and statement code generation
//!
//! Generates Rust code for expressions, statements, and blocks

use crate::ast::*;
use crate::codegen::RustCodegen;
use anyhow::Result;

impl RustCodegen {
    /// Generate a block of statements
    pub(crate) fn generate_block(&mut self, block: &Block) -> Result<()> {
        self.write("{");
        self.writeln();
        self.indent();

        for stmt in &block.stmts {
            self.generate_stmt(stmt)?;
        }

        self.dedent();
        self.write_indent();
        self.write("}");

        Ok(())
    }

    /// Generate a statement
    pub(crate) fn generate_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        self.write_indent();

        match &stmt.kind {
            StmtKind::Expr(expr) => {
                // Expression without semicolon (typically last statement in block)
                self.generate_expr(expr)?;
                self.writeln();
            }
            StmtKind::Semi(expr) => {
                // Expression with semicolon
                self.generate_expr(expr)?;
                self.write(";");
                self.writeln();
            }
            StmtKind::Let(local) => {
                // Let binding
                self.generate_let(local)?;
                self.write(";");
                self.writeln();
            }
            StmtKind::Item(item) => {
                // Nested item (rare)
                self.generate_item(item)?;
            }
            StmtKind::MacCall(mac_stmt) => {
                // Macro call statement
                self.generate_mac_call(&mac_stmt.mac)?;
                match mac_stmt.style {
                    MacStmtStyle::Semicolon => self.write(";"),
                    MacStmtStyle::Braces | MacStmtStyle::NoBraces => {}
                }
                self.writeln();
            }
            StmtKind::Empty => {
                // Empty statement (just a semicolon)
                self.write(";");
                self.writeln();
            }
        }

        Ok(())
    }

    /// Generate a let binding
    fn generate_let(&mut self, local: &Local) -> Result<()> {
        self.write("let ");
        self.generate_pat(&local.pat)?;

        // Type annotation
        if let Some(ty) = &local.ty {
            self.write(": ");
            self.generate_ty(ty)?;
        }

        // Initializer
        match &local.kind {
            LocalKind::Decl => {
                // No initializer
            }
            LocalKind::Init(init) => {
                self.write(" = ");
                self.generate_expr(&init.expr)?;
            }
            LocalKind::InitElse(init, else_block) => {
                self.write(" = ");
                self.generate_expr(&init.expr)?;
                self.write(" else ");
                self.generate_block(else_block)?;
            }
        }

        Ok(())
    }

    /// Generate an expression
    pub(crate) fn generate_expr(&mut self, expr: &Expr) -> Result<()> {
        match &expr.kind {
            ExprKind::MacCall(mac) => {
                self.generate_mac_call(mac)?;
            }
            ExprKind::Lit(lit) => {
                self.generate_lit(lit)?;
            }
            ExprKind::Path(qself, path) => {
                if qself.is_some() {
                    // TODO: Phase 2+ will handle qualified paths like <T as Trait>::Assoc
                    self.write("/* qualified path */");
                }
                self.generate_path(path)?;
            }
            ExprKind::If { cond, then_branch, else_branch } => {
                self.generate_if(cond, then_branch, else_branch.as_deref())?;
            }
            ExprKind::Match { expr, arms } => {
                self.generate_match(expr, arms)?;
            }
            ExprKind::While { label, cond, body } => {
                self.generate_while(label.as_ref(), cond, body)?;
            }
            ExprKind::ForLoop { label, pat, iter, body } => {
                self.generate_for_loop(label.as_ref(), pat, iter, body)?;
            }
            ExprKind::Loop { label, body } => {
                self.generate_loop(label.as_ref(), body)?;
            }
        }
        Ok(())
    }

    /// Generate a literal
    fn generate_lit(&mut self, lit: &Lit) -> Result<()> {
        match &lit.kind {
            LitKind::Str(s) => {
                self.write("\"");
                // Escape special characters
                for ch in s.chars() {
                    match ch {
                        '"' => self.write("\\\""),
                        '\\' => self.write("\\\\"),
                        '\n' => self.write("\\n"),
                        '\r' => self.write("\\r"),
                        '\t' => self.write("\\t"),
                        _ => {
                            let mut buf = [0; 4];
                            self.write(ch.encode_utf8(&mut buf));
                        }
                    }
                }
                self.write("\"");
            }
            LitKind::Int(n) => {
                self.write(&n.to_string());
            }
        }
        Ok(())
    }

    /// Generate a macro call
    pub(crate) fn generate_mac_call(&mut self, mac: &MacCall) -> Result<()> {
        // Macro path
        self.generate_path(&mac.path)?;
        self.write("!");

        // Macro arguments
        match &mac.args {
            MacArgs::Empty => {
                self.write("()");
            }
            MacArgs::Delimited { delim, tokens, .. } => {
                let (open, close) = match delim {
                    Delimiter::Paren => ("(", ")"),
                    Delimiter::Brace => ("{", "}"),
                    Delimiter::Bracket => ("[", "]"),
                    Delimiter::Invisible => ("", ""),
                };
                self.write(open);
                self.generate_token_stream(tokens)?;
                self.write(close);
            }
            MacArgs::Eq { tokens, .. } => {
                self.write(" = ");
                self.generate_token_stream(tokens)?;
            }
        }

        Ok(())
    }

    /// Generate a token stream (simplified for Phase 1)
    fn generate_token_stream(&mut self, tokens: &TokenStream) -> Result<()> {
        match tokens {
            TokenStream::Source(s) => {
                self.write(s);
            }
            TokenStream::Empty => {}
        }
        Ok(())
    }

    /// Generate an if expression
    fn generate_if(
        &mut self,
        cond: &Expr,
        then_branch: &Block,
        else_branch: Option<&Expr>,
    ) -> Result<()> {
        self.write("if ");
        self.generate_expr(cond)?;
        self.write(" ");
        self.generate_block(then_branch)?;

        if let Some(else_expr) = else_branch {
            self.write(" else ");
            // Check if it's another if (else if chain) or a block
            match &else_expr.kind {
                ExprKind::If { .. } => {
                    self.generate_expr(else_expr)?;
                }
                _ => {
                    // Assume it's a block expression for else
                    self.generate_expr(else_expr)?;
                }
            }
        }

        Ok(())
    }

    /// Generate a match expression
    fn generate_match(&mut self, expr: &Expr, arms: &[Arm]) -> Result<()> {
        self.write("match ");
        self.generate_expr(expr)?;
        self.write(" {");
        self.writeln();
        self.indent();

        for arm in arms {
            self.write_indent();
            self.generate_pat(&arm.pat)?;

            if let Some(guard) = &arm.guard {
                self.write(" if ");
                self.generate_expr(guard)?;
            }

            self.write(" => ");
            self.generate_expr(&arm.body)?;
            self.write(",");
            self.writeln();
        }

        self.dedent();
        self.write_indent();
        self.write("}");

        Ok(())
    }

    /// Generate a while loop
    fn generate_while(&mut self, label: Option<&Label>, cond: &Expr, body: &Block) -> Result<()> {
        if let Some(lbl) = label {
            self.write("'");
            self.write(&lbl.ident.name);
            self.write(": ");
        }

        self.write("while ");
        self.generate_expr(cond)?;
        self.write(" ");
        self.generate_block(body)?;

        Ok(())
    }

    /// Generate a for loop
    fn generate_for_loop(
        &mut self,
        label: Option<&Label>,
        pat: &Pat,
        iter: &Expr,
        body: &Block,
    ) -> Result<()> {
        if let Some(lbl) = label {
            self.write("'");
            self.write(&lbl.ident.name);
            self.write(": ");
        }

        self.write("for ");
        self.generate_pat(pat)?;
        self.write(" in ");
        self.generate_expr(iter)?;
        self.write(" ");
        self.generate_block(body)?;

        Ok(())
    }

    /// Generate an infinite loop
    fn generate_loop(&mut self, label: Option<&Label>, body: &Block) -> Result<()> {
        if let Some(lbl) = label {
            self.write("'");
            self.write(&lbl.ident.name);
            self.write(": ");
        }

        self.write("loop ");
        self.generate_block(body)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_string_literal() {
        let expr = Expr {
            id: NodeId(0),
            kind: ExprKind::Lit(Lit {
                kind: LitKind::Str("Hello, world!".to_string()),
                span: Span::DUMMY,
            }),
            span: Span::DUMMY,
            attrs: Vec::new(),
            tokens: None,
        };

        let mut codegen = RustCodegen::new();
        codegen.generate_expr(&expr).unwrap();

        assert_eq!(codegen.output(), "\"Hello, world!\"");
    }

    #[test]
    fn test_generate_int_literal() {
        let expr = Expr {
            id: NodeId(0),
            kind: ExprKind::Lit(Lit { kind: LitKind::Int(42), span: Span::DUMMY }),
            span: Span::DUMMY,
            attrs: Vec::new(),
            tokens: None,
        };

        let mut codegen = RustCodegen::new();
        codegen.generate_expr(&expr).unwrap();

        assert_eq!(codegen.output(), "42");
    }

    #[test]
    fn test_generate_string_with_escapes() {
        let expr = Expr {
            id: NodeId(0),
            kind: ExprKind::Lit(Lit {
                kind: LitKind::Str("Line 1\nLine 2\t\"quoted\"".to_string()),
                span: Span::DUMMY,
            }),
            span: Span::DUMMY,
            attrs: Vec::new(),
            tokens: None,
        };

        let mut codegen = RustCodegen::new();
        codegen.generate_expr(&expr).unwrap();

        assert_eq!(codegen.output(), r#""Line 1\nLine 2\t\"quoted\"""#);
    }

    #[test]
    fn test_generate_macro_call() {
        let mac = MacCall {
            path: Path::from_ident(Ident::new("println", Span::DUMMY)),
            args: MacArgs::Delimited {
                dspan: DelSpan::new(Span::DUMMY, Span::DUMMY),
                delim: Delimiter::Paren,
                tokens: TokenStream::Source("\"test\"".to_string()),
            },
            prior_type_ascription: None,
        };

        let mut codegen = RustCodegen::new();
        codegen.generate_mac_call(&mac).unwrap();

        assert_eq!(codegen.output(), "println!(\"test\")");
    }

    #[test]
    fn test_generate_simple_block() {
        let block = Block::new(vec![], NodeId(0), Span::DUMMY);

        let mut codegen = RustCodegen::new();
        codegen.generate_block(&block).unwrap();

        assert_eq!(codegen.output(), "{\n}");
    }
}
