//! Expression and statement code generation
//!
//! Generates Rust code for expressions, statements, and blocks

use crate::ast::*;
use crate::gen_rs::RustCodegen;
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
            ExprKind::Binary { left, op, right } => {
                self.generate_binary(left, *op, right)?;
            }
            ExprKind::Unary { op, expr } => {
                self.generate_unary(*op, expr)?;
            }
            ExprKind::Call { func, args } => {
                self.generate_call(func, args)?;
            }
            ExprKind::MethodCall { receiver, method, args } => {
                self.generate_method_call(receiver, method, args)?;
            }
            // Stage 7: Remaining expressions
            ExprKind::Array(elems) => {
                self.write("[");
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_expr(elem)?;
                }
                self.write("]");
            }
            ExprKind::Tuple(elems) => {
                self.write("(");
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_expr(elem)?;
                }
                if elems.len() == 1 {
                    self.write(","); // Trailing comma for 1-tuples
                }
                self.write(")");
            }
            ExprKind::Field { expr, field } => {
                self.generate_expr(expr)?;
                self.write(".");
                self.write(&field.name);
            }
            ExprKind::Index { expr, index } => {
                self.generate_expr(expr)?;
                self.write("[");
                self.generate_expr(index)?;
                self.write("]");
            }
            ExprKind::Assign { left, right } => {
                self.generate_expr(left)?;
                self.write(" = ");
                self.generate_expr(right)?;
            }
            ExprKind::Struct { path, fields } => {
                self.generate_path(path)?;
                self.write(" {");
                if !fields.is_empty() {
                    self.write(" ");
                    for (i, field) in fields.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.write(&field.ident.name);
                        if !field.is_shorthand {
                            self.write(": ");
                            self.generate_expr(&field.expr)?;
                        }
                    }
                    self.write(" ");
                }
                self.write("}");
            }
            ExprKind::Closure { params, body } => {
                self.write("|");
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_pat(&param.pat)?;
                }
                self.write("| ");
                self.generate_expr(body)?;
            }
            ExprKind::Range { start, end, inclusive } => {
                if let Some(start_expr) = start {
                    self.generate_expr(start_expr)?;
                }
                if *inclusive {
                    self.write("..=");
                } else {
                    self.write("..");
                }
                if let Some(end_expr) = end {
                    self.generate_expr(end_expr)?;
                }
            }
            // Phase 5: Priority 3 - Expression gap filling
            ExprKind::Paren(expr) => {
                self.write("(");
                self.generate_expr(expr)?;
                self.write(")");
            }
            ExprKind::Try(expr) => {
                self.generate_expr(expr)?;
                self.write("?");
            }
            ExprKind::Cast { expr, ty } => {
                self.generate_expr(expr)?;
                self.write(" as ");
                self.generate_ty(ty)?;
            }
            ExprKind::Break { label, value } => {
                self.write("break");
                if let Some(label) = label {
                    self.write(" '");
                    self.write(&label.ident.name);
                }
                if let Some(value) = value {
                    self.write(" ");
                    self.generate_expr(value)?;
                }
            }
            ExprKind::Continue { label } => {
                self.write("continue");
                if let Some(label) = label {
                    self.write(" '");
                    self.write(&label.ident.name);
                }
            }
            ExprKind::Return { value } => {
                self.write("return");
                if let Some(value) = value {
                    self.write(" ");
                    self.generate_expr(value)?;
                }
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
            // Phase 11: New literal types
            LitKind::Bool(b) => {
                self.write(if *b { "true" } else { "false" });
            }
            LitKind::Float(f) => {
                self.write(f);
            }
            LitKind::Char(c) => {
                self.write("'");
                match c {
                    '\'' => self.write("\\'"),
                    '\\' => self.write("\\\\"),
                    '\n' => self.write("\\n"),
                    '\r' => self.write("\\r"),
                    '\t' => self.write("\\t"),
                    _ => {
                        let mut buf = [0; 4];
                        self.write(c.encode_utf8(&mut buf));
                    }
                }
                self.write("'");
            }
            LitKind::Byte(b) => {
                self.write(&format!("b'{}'", *b as char));
            }
            LitKind::ByteStr(bytes) => {
                self.write("b\"");
                for &byte in bytes {
                    if byte.is_ascii_graphic() && byte != b'"' && byte != b'\\' {
                        self.write(&format!("{}", byte as char));
                    } else {
                        self.write(&format!("\\x{:02x}", byte));
                    }
                }
                self.write("\"");
            }
            LitKind::CStr(bytes) => {
                self.write("c\"");
                for &byte in bytes {
                    if byte.is_ascii_graphic() && byte != b'"' && byte != b'\\' {
                        self.write(&format!("{}", byte as char));
                    } else {
                        self.write(&format!("\\x{:02x}", byte));
                    }
                }
                self.write("\"");
            }
            LitKind::Verbatim(s) => {
                // Verbatim literals are already in string form
                self.write(s);
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

    /// Generate a binary operation
    fn generate_binary(&mut self, left: &Expr, op: BinOp, right: &Expr) -> Result<()> {
        // For now, use parentheses for all binary operations
        // TODO Phase 2+: Implement proper operator precedence
        self.write("(");
        self.generate_expr(left)?;
        self.write(" ");
        self.write(self.binop_to_str(op));
        self.write(" ");
        self.generate_expr(right)?;
        self.write(")");
        Ok(())
    }

    /// Generate a unary operation
    fn generate_unary(&mut self, op: UnOp, expr: &Expr) -> Result<()> {
        self.write(self.unop_to_str(op));
        self.generate_expr(expr)?;
        Ok(())
    }

    /// Generate a function call
    fn generate_call(&mut self, func: &Expr, args: &[Expr]) -> Result<()> {
        self.generate_expr(func)?;
        self.write("(");
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.generate_expr(arg)?;
        }
        self.write(")");
        Ok(())
    }

    /// Generate a method call
    fn generate_method_call(
        &mut self,
        receiver: &Expr,
        method: &Ident,
        args: &[Expr],
    ) -> Result<()> {
        self.generate_expr(receiver)?;
        self.write(".");
        self.write(&method.name);
        self.write("(");
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.generate_expr(arg)?;
        }
        self.write(")");
        Ok(())
    }

    /// Convert binary operator to string
    fn binop_to_str(&self, op: BinOp) -> &'static str {
        match op {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::And => "&&",
            BinOp::Or => "||",
            BinOp::BitAnd => "&",
            BinOp::BitOr => "|",
            BinOp::BitXor => "^",
            BinOp::Shl => "<<",
            BinOp::Shr => ">>",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
        }
    }

    /// Convert unary operator to string
    fn unop_to_str(&self, op: UnOp) -> &'static str {
        match op {
            UnOp::Not => "!",
            UnOp::Neg => "-",
            UnOp::Deref => "*",
            UnOp::Ref => "&",
            UnOp::RefMut => "&mut ",
        }
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

    // Stage 3: Binary operators
    #[test]
    fn test_generate_binary_add() {
        let expr = Expr {
            id: NodeId(0),
            kind: ExprKind::Binary {
                left: Box::new(Expr {
                    id: NodeId(1),
                    kind: ExprKind::Lit(Lit { kind: LitKind::Int(1), span: Span::DUMMY }),
                    span: Span::DUMMY,
                    attrs: Vec::new(),
                    tokens: None,
                }),
                op: BinOp::Add,
                right: Box::new(Expr {
                    id: NodeId(2),
                    kind: ExprKind::Lit(Lit { kind: LitKind::Int(2), span: Span::DUMMY }),
                    span: Span::DUMMY,
                    attrs: Vec::new(),
                    tokens: None,
                }),
            },
            span: Span::DUMMY,
            attrs: Vec::new(),
            tokens: None,
        };

        let mut codegen = RustCodegen::new();
        codegen.generate_expr(&expr).unwrap();

        assert_eq!(codegen.output(), "(1 + 2)");
    }

    #[test]
    fn test_generate_binary_comparison() {
        let expr = Expr {
            id: NodeId(0),
            kind: ExprKind::Binary {
                left: Box::new(Expr {
                    id: NodeId(1),
                    kind: ExprKind::Lit(Lit { kind: LitKind::Int(5), span: Span::DUMMY }),
                    span: Span::DUMMY,
                    attrs: Vec::new(),
                    tokens: None,
                }),
                op: BinOp::Lt,
                right: Box::new(Expr {
                    id: NodeId(2),
                    kind: ExprKind::Lit(Lit { kind: LitKind::Int(10), span: Span::DUMMY }),
                    span: Span::DUMMY,
                    attrs: Vec::new(),
                    tokens: None,
                }),
            },
            span: Span::DUMMY,
            attrs: Vec::new(),
            tokens: None,
        };

        let mut codegen = RustCodegen::new();
        codegen.generate_expr(&expr).unwrap();

        assert_eq!(codegen.output(), "(5 < 10)");
    }

    #[test]
    fn test_generate_binary_logical() {
        let expr = Expr {
            id: NodeId(0),
            kind: ExprKind::Binary {
                left: Box::new(Expr {
                    id: NodeId(1),
                    kind: ExprKind::Path(None, Path::from_ident(Ident::new("a", Span::DUMMY))),
                    span: Span::DUMMY,
                    attrs: Vec::new(),
                    tokens: None,
                }),
                op: BinOp::And,
                right: Box::new(Expr {
                    id: NodeId(2),
                    kind: ExprKind::Path(None, Path::from_ident(Ident::new("b", Span::DUMMY))),
                    span: Span::DUMMY,
                    attrs: Vec::new(),
                    tokens: None,
                }),
            },
            span: Span::DUMMY,
            attrs: Vec::new(),
            tokens: None,
        };

        let mut codegen = RustCodegen::new();
        codegen.generate_expr(&expr).unwrap();

        assert_eq!(codegen.output(), "(a && b)");
    }

    // Stage 3: Unary operators
    #[test]
    fn test_generate_unary_not() {
        let expr = Expr {
            id: NodeId(0),
            kind: ExprKind::Unary {
                op: UnOp::Not,
                expr: Box::new(Expr {
                    id: NodeId(1),
                    kind: ExprKind::Path(None, Path::from_ident(Ident::new("flag", Span::DUMMY))),
                    span: Span::DUMMY,
                    attrs: Vec::new(),
                    tokens: None,
                }),
            },
            span: Span::DUMMY,
            attrs: Vec::new(),
            tokens: None,
        };

        let mut codegen = RustCodegen::new();
        codegen.generate_expr(&expr).unwrap();

        assert_eq!(codegen.output(), "!flag");
    }

    #[test]
    fn test_generate_unary_neg() {
        let expr = Expr {
            id: NodeId(0),
            kind: ExprKind::Unary {
                op: UnOp::Neg,
                expr: Box::new(Expr {
                    id: NodeId(1),
                    kind: ExprKind::Lit(Lit { kind: LitKind::Int(42), span: Span::DUMMY }),
                    span: Span::DUMMY,
                    attrs: Vec::new(),
                    tokens: None,
                }),
            },
            span: Span::DUMMY,
            attrs: Vec::new(),
            tokens: None,
        };

        let mut codegen = RustCodegen::new();
        codegen.generate_expr(&expr).unwrap();

        assert_eq!(codegen.output(), "-42");
    }

    #[test]
    fn test_generate_unary_deref() {
        let expr = Expr {
            id: NodeId(0),
            kind: ExprKind::Unary {
                op: UnOp::Deref,
                expr: Box::new(Expr {
                    id: NodeId(1),
                    kind: ExprKind::Path(None, Path::from_ident(Ident::new("ptr", Span::DUMMY))),
                    span: Span::DUMMY,
                    attrs: Vec::new(),
                    tokens: None,
                }),
            },
            span: Span::DUMMY,
            attrs: Vec::new(),
            tokens: None,
        };

        let mut codegen = RustCodegen::new();
        codegen.generate_expr(&expr).unwrap();

        assert_eq!(codegen.output(), "*ptr");
    }

    // Stage 3: Function calls
    #[test]
    fn test_generate_call_no_args() {
        let expr = Expr {
            id: NodeId(0),
            kind: ExprKind::Call {
                func: Box::new(Expr {
                    id: NodeId(1),
                    kind: ExprKind::Path(None, Path::from_ident(Ident::new("foo", Span::DUMMY))),
                    span: Span::DUMMY,
                    attrs: Vec::new(),
                    tokens: None,
                }),
                args: Vec::new(),
            },
            span: Span::DUMMY,
            attrs: Vec::new(),
            tokens: None,
        };

        let mut codegen = RustCodegen::new();
        codegen.generate_expr(&expr).unwrap();

        assert_eq!(codegen.output(), "foo()");
    }

    #[test]
    fn test_generate_call_with_args() {
        let expr = Expr {
            id: NodeId(0),
            kind: ExprKind::Call {
                func: Box::new(Expr {
                    id: NodeId(1),
                    kind: ExprKind::Path(None, Path::from_ident(Ident::new("add", Span::DUMMY))),
                    span: Span::DUMMY,
                    attrs: Vec::new(),
                    tokens: None,
                }),
                args: vec![
                    Expr {
                        id: NodeId(2),
                        kind: ExprKind::Lit(Lit { kind: LitKind::Int(1), span: Span::DUMMY }),
                        span: Span::DUMMY,
                        attrs: Vec::new(),
                        tokens: None,
                    },
                    Expr {
                        id: NodeId(3),
                        kind: ExprKind::Lit(Lit { kind: LitKind::Int(2), span: Span::DUMMY }),
                        span: Span::DUMMY,
                        attrs: Vec::new(),
                        tokens: None,
                    },
                ],
            },
            span: Span::DUMMY,
            attrs: Vec::new(),
            tokens: None,
        };

        let mut codegen = RustCodegen::new();
        codegen.generate_expr(&expr).unwrap();

        assert_eq!(codegen.output(), "add(1, 2)");
    }

    // Stage 3: Method calls
    #[test]
    fn test_generate_method_call_no_args() {
        let expr = Expr {
            id: NodeId(0),
            kind: ExprKind::MethodCall {
                receiver: Box::new(Expr {
                    id: NodeId(1),
                    kind: ExprKind::Path(None, Path::from_ident(Ident::new("obj", Span::DUMMY))),
                    span: Span::DUMMY,
                    attrs: Vec::new(),
                    tokens: None,
                }),
                method: Ident::new("len", Span::DUMMY),
                args: Vec::new(),
            },
            span: Span::DUMMY,
            attrs: Vec::new(),
            tokens: None,
        };

        let mut codegen = RustCodegen::new();
        codegen.generate_expr(&expr).unwrap();

        assert_eq!(codegen.output(), "obj.len()");
    }

    #[test]
    fn test_generate_method_call_with_args() {
        let expr = Expr {
            id: NodeId(0),
            kind: ExprKind::MethodCall {
                receiver: Box::new(Expr {
                    id: NodeId(1),
                    kind: ExprKind::Path(None, Path::from_ident(Ident::new("vec", Span::DUMMY))),
                    span: Span::DUMMY,
                    attrs: Vec::new(),
                    tokens: None,
                }),
                method: Ident::new("push", Span::DUMMY),
                args: vec![Expr {
                    id: NodeId(2),
                    kind: ExprKind::Lit(Lit { kind: LitKind::Int(42), span: Span::DUMMY }),
                    span: Span::DUMMY,
                    attrs: Vec::new(),
                    tokens: None,
                }],
            },
            span: Span::DUMMY,
            attrs: Vec::new(),
            tokens: None,
        };

        let mut codegen = RustCodegen::new();
        codegen.generate_expr(&expr).unwrap();

        assert_eq!(codegen.output(), "vec.push(42)");
    }
}
