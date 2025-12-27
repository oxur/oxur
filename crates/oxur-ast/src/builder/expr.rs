use super::build::AstBuilder;
use super::helpers::*;
use crate::ast::*;
use crate::error::{ParseError, Result};
use crate::sexp::SExp;

impl AstBuilder {
    pub fn build_block(&mut self, sexp: &SExp) -> Result<Block> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "Block" {
            return Err(ParseError::Expected {
                expected: "Block".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let stmts = if let Some(stmts_sexp) = kwargs.get("stmts") {
            let stmts_list = expect_list(stmts_sexp)?;
            let mut statements = Vec::new();
            for stmt_sexp in &stmts_list.elements {
                statements.push(self.build_stmt(stmt_sexp)?);
            }
            statements
        } else {
            Vec::new()
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

        Ok(Block::new(stmts, id, span))
    }

    pub fn build_expr(&mut self, sexp: &SExp) -> Result<Expr> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "Expr" {
            return Err(ParseError::Expected {
                expected: "Expr".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let kind = if let Some(kind_sexp) = kwargs.get("kind") {
            self.build_expr_kind(kind_sexp)?
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

        Ok(Expr { id, kind, span, attrs: Vec::new(), tokens: None })
    }

    fn build_expr_kind(&mut self, sexp: &SExp) -> Result<ExprKind> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        match node_type.value.as_str() {
            "MacCall" => {
                let mac_call = self.build_mac_call_inner(list)?;
                Ok(ExprKind::MacCall(mac_call))
            }
            _ => Err(ParseError::Expected {
                expected: "MacCall (only supported in Phase 1)".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            }),
        }
    }

    fn build_mac_call_inner(&mut self, list: &crate::sexp::List) -> Result<MacCall> {
        let kwargs = parse_kwargs(list)?;

        let path = if let Some(path_sexp) = kwargs.get("path") {
            self.build_path(path_sexp)?
        } else {
            return Err(ParseError::Expected {
                expected: ":path field".to_string(),
                found: "missing".to_string(),
                pos: list.pos,
            });
        };

        let args = if let Some(args_sexp) = kwargs.get("args") {
            self.build_mac_args(args_sexp)?
        } else {
            MacArgs::Empty
        };

        Ok(MacCall::new(path, args))
    }

    pub fn build_path(&mut self, sexp: &SExp) -> Result<Path> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "Path" {
            return Err(ParseError::Expected {
                expected: "Path".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let segments = if let Some(segments_sexp) = kwargs.get("segments") {
            let segments_list = expect_list(segments_sexp)?;
            let mut segs = Vec::new();
            for seg_sexp in &segments_list.elements {
                segs.push(self.build_path_segment(seg_sexp)?);
            }
            segs
        } else {
            Vec::new()
        };

        let span = if let Some(span_sexp) = kwargs.get("span") {
            self.build_span(span_sexp)?
        } else {
            Span::DUMMY
        };

        Ok(Path { span, segments, tokens: None })
    }

    fn build_path_segment(&mut self, sexp: &SExp) -> Result<PathSegment> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "PathSegment" {
            return Err(ParseError::Expected {
                expected: "PathSegment".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let ident = if let Some(ident_sexp) = kwargs.get("ident") {
            self.build_ident(ident_sexp)?
        } else {
            return Err(ParseError::Expected {
                expected: ":ident field".to_string(),
                found: "missing".to_string(),
                pos: list.pos,
            });
        };

        let id = if let Some(id_sexp) = kwargs.get("id") {
            NodeId(expect_number(id_sexp)? as u32)
        } else {
            self.next_id()
        };

        Ok(PathSegment { ident, id, args: None })
    }

    fn build_mac_args(&mut self, sexp: &SExp) -> Result<MacArgs> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        match node_type.value.as_str() {
            "Empty" => Ok(MacArgs::Empty),
            "Delimited" => {
                let kwargs = parse_kwargs(list)?;

                let dspan = if let Some(dspan_sexp) = kwargs.get("dspan") {
                    self.build_del_span(dspan_sexp)?
                } else {
                    DelSpan::new(Span::DUMMY, Span::DUMMY)
                };

                let delim = if let Some(delim_sexp) = kwargs.get("delim") {
                    self.build_delimiter(delim_sexp)?
                } else {
                    Delimiter::Paren
                };

                let tokens = if let Some(tokens_sexp) = kwargs.get("tokens") {
                    self.build_token_stream(tokens_sexp)?
                } else {
                    TokenStream::Empty
                };

                Ok(MacArgs::Delimited { dspan, delim, tokens })
            }
            _ => Err(ParseError::Expected {
                expected: "Empty or Delimited".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            }),
        }
    }

    fn build_del_span(&mut self, sexp: &SExp) -> Result<DelSpan> {
        let list = expect_list(sexp)?;
        let kwargs = parse_kwargs(list)?;

        let open = if let Some(open_sexp) = kwargs.get("open") {
            self.build_span(open_sexp)?
        } else {
            Span::DUMMY
        };

        let close = if let Some(close_sexp) = kwargs.get("close") {
            self.build_span(close_sexp)?
        } else {
            Span::DUMMY
        };

        Ok(DelSpan::new(open, close))
    }

    fn build_delimiter(&mut self, sexp: &SExp) -> Result<Delimiter> {
        let sym = expect_symbol(sexp)?;
        match sym.value.as_str() {
            "Paren" => Ok(Delimiter::Paren),
            "Brace" => Ok(Delimiter::Brace),
            "Bracket" => Ok(Delimiter::Bracket),
            "Invisible" => Ok(Delimiter::Invisible),
            _ => Err(ParseError::Expected {
                expected: "Paren, Brace, Bracket, or Invisible".to_string(),
                found: sym.value.clone(),
                pos: sym.pos,
            }),
        }
    }

    fn build_token_stream(&mut self, sexp: &SExp) -> Result<TokenStream> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        match node_type.value.as_str() {
            "TokenStream" => {
                let kwargs = parse_kwargs(list)?;
                if let Some(source_sexp) = kwargs.get("source") {
                    let source = expect_string(source_sexp)?;
                    Ok(TokenStream::Source(source))
                } else {
                    Ok(TokenStream::Empty)
                }
            }
            _ => Err(ParseError::Expected {
                expected: "TokenStream".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            }),
        }
    }
}
