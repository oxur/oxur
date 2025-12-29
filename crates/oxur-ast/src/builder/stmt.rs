use super::build::AstBuilder;
use super::helpers::*;
use crate::ast::*;
use crate::error::{ParseError, Result};
use crate::sexp::SExp;

impl AstBuilder {
    pub fn build_stmt(&mut self, sexp: &SExp) -> Result<Stmt> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "Stmt" {
            return Err(ParseError::Expected {
                expected: "Stmt".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let kind = if let Some(kind_sexp) = kwargs.get("kind") {
            self.build_stmt_kind(kind_sexp)?
        } else {
            return Err(ParseError::Expected {
                expected: ":kind field".to_string(),
                found: "missing".to_string(),
                pos: list.pos,
            });
        };

        let span = if let Some(span_sexp) = kwargs.get("span") {
            self.build_span(span_sexp)?
        } else {
            Span::DUMMY
        };

        let id = if let Some(id_sexp) = kwargs.get("id") {
            NodeId(expect_number(id_sexp)? as u32)
        } else {
            self.next_id()
        };

        Ok(Stmt { id, kind, span })
    }

    fn build_stmt_kind(&mut self, sexp: &SExp) -> Result<StmtKind> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        match node_type.value.as_str() {
            "Semi" => {
                // Extract the inner expression from element 1
                if list.elements.len() < 2 {
                    return Err(ParseError::Expected {
                        expected: "expression".to_string(),
                        found: "nothing".to_string(),
                        pos: list.pos,
                    });
                }
                let expr_sexp = &list.elements[1];
                let expr = self.build_expr(expr_sexp)?;
                Ok(StmtKind::Semi(expr))
            }
            "Expr" => {
                // Extract the inner expression from element 1
                if list.elements.len() < 2 {
                    return Err(ParseError::Expected {
                        expected: "expression".to_string(),
                        found: "nothing".to_string(),
                        pos: list.pos,
                    });
                }
                let expr_sexp = &list.elements[1];
                let expr = self.build_expr(expr_sexp)?;
                Ok(StmtKind::Expr(expr))
            }
            "MacCall" => {
                // Extract the inner MacCallStmt from element 1
                let mac_call_stmt_sexp = &list.elements[1];
                let mac_call_stmt = self.build_mac_call_stmt(mac_call_stmt_sexp)?;
                Ok(StmtKind::MacCall(mac_call_stmt))
            }
            "Empty" => Ok(StmtKind::Empty),
            _ => Err(ParseError::Expected {
                expected: "Semi, Expr, MacCall, or Empty".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            }),
        }
    }

    fn build_mac_call_stmt(&mut self, sexp: &SExp) -> Result<MacCallStmt> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "MacCallStmt" {
            return Err(ParseError::Expected {
                expected: "MacCallStmt".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let mac = if let Some(mac_sexp) = kwargs.get("mac") {
            let mac_list = expect_list(mac_sexp)?;
            self.build_mac_call_inner(mac_list)?
        } else {
            return Err(ParseError::Expected {
                expected: ":mac field".to_string(),
                found: "missing".to_string(),
                pos: list.pos,
            });
        };

        let style = if let Some(style_sexp) = kwargs.get("style") {
            let sym = expect_symbol(style_sexp)?;
            match sym.value.as_str() {
                "Semicolon" => MacStmtStyle::Semicolon,
                "Braces" => MacStmtStyle::Braces,
                "NoBraces" => MacStmtStyle::NoBraces,
                _ => MacStmtStyle::Semicolon,
            }
        } else {
            MacStmtStyle::Semicolon
        };

        Ok(MacCallStmt { mac, style, attrs: Vec::new(), tokens: None })
    }
}
