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
                let kwargs = parse_kwargs(list)?;
                let expr_sexp = kwargs
                    .get("expr")
                    .ok_or_else(|| ParseError::Expected {
                        expected: ":expr field".to_string(),
                        found: "missing".to_string(),
                        pos: list.pos,
                    })?;
                let expr = self.build_expr(expr_sexp)?;
                Ok(StmtKind::Semi(expr))
            }
            "Expr" => {
                let kwargs = parse_kwargs(list)?;
                let expr_sexp = kwargs
                    .get("expr")
                    .ok_or_else(|| ParseError::Expected {
                        expected: ":expr field".to_string(),
                        found: "missing".to_string(),
                        pos: list.pos,
                    })?;
                let expr = self.build_expr(expr_sexp)?;
                Ok(StmtKind::Expr(expr))
            }
            "Empty" => Ok(StmtKind::Empty),
            _ => Err(ParseError::Expected {
                expected: "Semi, Expr, or Empty (Phase 1)".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            }),
        }
    }
}
