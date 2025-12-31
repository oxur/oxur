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
                // Extract the inner MacCall node from element 1
                let mac_call_sexp = &list.elements[1];
                let mac_call_list = expect_list(mac_call_sexp)?;
                let mac_call = self.build_mac_call_inner(mac_call_list)?;
                Ok(ExprKind::MacCall(mac_call))
            }
            "If" => {
                let kwargs = parse_kwargs(list)?;
                let cond = Box::new(self.build_expr(kwargs.get("cond").ok_or_else(|| {
                    ParseError::Expected {
                        expected: ":cond field".to_string(),
                        found: "missing".to_string(),
                        pos: list.pos,
                    }
                })?)?);
                let then_branch =
                    self.build_block(kwargs.get("then").ok_or_else(|| ParseError::Expected {
                        expected: ":then field".to_string(),
                        found: "missing".to_string(),
                        pos: list.pos,
                    })?)?;
                let else_branch = if let Some(else_sexp) = kwargs.get("else") {
                    if !is_nil(else_sexp) {
                        Some(Box::new(self.build_expr(else_sexp)?))
                    } else {
                        None
                    }
                } else {
                    None
                };
                Ok(ExprKind::If { cond, then_branch, else_branch })
            }
            "Match" => {
                let kwargs = parse_kwargs(list)?;
                let expr = Box::new(self.build_expr(kwargs.get("expr").ok_or_else(|| {
                    ParseError::Expected {
                        expected: ":expr field".to_string(),
                        found: "missing".to_string(),
                        pos: list.pos,
                    }
                })?)?);
                let arms_sexp = kwargs.get("arms").ok_or_else(|| ParseError::Expected {
                    expected: ":arms field".to_string(),
                    found: "missing".to_string(),
                    pos: list.pos,
                })?;
                let arms_list = expect_list(arms_sexp)?;
                let mut arms = Vec::new();
                for arm_sexp in &arms_list.elements {
                    arms.push(self.build_arm(arm_sexp)?);
                }
                Ok(ExprKind::Match { expr, arms })
            }
            "While" => {
                let kwargs = parse_kwargs(list)?;
                let label = if let Some(label_sexp) = kwargs.get("label") {
                    if !is_nil(label_sexp) {
                        Some(self.build_label(label_sexp)?)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let cond = Box::new(self.build_expr(kwargs.get("cond").ok_or_else(|| {
                    ParseError::Expected {
                        expected: ":cond field".to_string(),
                        found: "missing".to_string(),
                        pos: list.pos,
                    }
                })?)?);
                let body =
                    self.build_block(kwargs.get("body").ok_or_else(|| ParseError::Expected {
                        expected: ":body field".to_string(),
                        found: "missing".to_string(),
                        pos: list.pos,
                    })?)?;
                Ok(ExprKind::While { label, cond, body })
            }
            "ForLoop" => {
                let kwargs = parse_kwargs(list)?;
                let label = if let Some(label_sexp) = kwargs.get("label") {
                    if !is_nil(label_sexp) {
                        Some(self.build_label(label_sexp)?)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let pat =
                    self.build_pat(kwargs.get("pat").ok_or_else(|| ParseError::Expected {
                        expected: ":pat field".to_string(),
                        found: "missing".to_string(),
                        pos: list.pos,
                    })?)?;
                let iter = Box::new(self.build_expr(kwargs.get("iter").ok_or_else(|| {
                    ParseError::Expected {
                        expected: ":iter field".to_string(),
                        found: "missing".to_string(),
                        pos: list.pos,
                    }
                })?)?);
                let body =
                    self.build_block(kwargs.get("body").ok_or_else(|| ParseError::Expected {
                        expected: ":body field".to_string(),
                        found: "missing".to_string(),
                        pos: list.pos,
                    })?)?;
                Ok(ExprKind::ForLoop { label, pat, iter, body })
            }
            "Loop" => {
                let kwargs = parse_kwargs(list)?;
                let label = if let Some(label_sexp) = kwargs.get("label") {
                    if !is_nil(label_sexp) {
                        Some(self.build_label(label_sexp)?)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let body =
                    self.build_block(kwargs.get("body").ok_or_else(|| ParseError::Expected {
                        expected: ":body field".to_string(),
                        found: "missing".to_string(),
                        pos: list.pos,
                    })?)?;
                Ok(ExprKind::Loop { label, body })
            }
            "Binary" => {
                let kwargs = parse_kwargs(list)?;
                let left = Box::new(self.build_expr(kwargs.get("left").ok_or_else(|| {
                    ParseError::Expected {
                        expected: ":left field".to_string(),
                        found: "missing".to_string(),
                        pos: list.pos,
                    }
                })?)?);
                let op =
                    self.build_binop(kwargs.get("op").ok_or_else(|| ParseError::Expected {
                        expected: ":op field".to_string(),
                        found: "missing".to_string(),
                        pos: list.pos,
                    })?)?;
                let right = Box::new(self.build_expr(kwargs.get("right").ok_or_else(|| {
                    ParseError::Expected {
                        expected: ":right field".to_string(),
                        found: "missing".to_string(),
                        pos: list.pos,
                    }
                })?)?);
                Ok(ExprKind::Binary { left, op, right })
            }
            "Unary" => {
                let kwargs = parse_kwargs(list)?;
                let op =
                    self.build_unop(kwargs.get("op").ok_or_else(|| ParseError::Expected {
                        expected: ":op field".to_string(),
                        found: "missing".to_string(),
                        pos: list.pos,
                    })?)?;
                let expr = Box::new(self.build_expr(kwargs.get("expr").ok_or_else(|| {
                    ParseError::Expected {
                        expected: ":expr field".to_string(),
                        found: "missing".to_string(),
                        pos: list.pos,
                    }
                })?)?);
                Ok(ExprKind::Unary { op, expr })
            }
            "Call" => {
                let kwargs = parse_kwargs(list)?;
                let func = Box::new(self.build_expr(kwargs.get("func").ok_or_else(|| {
                    ParseError::Expected {
                        expected: ":func field".to_string(),
                        found: "missing".to_string(),
                        pos: list.pos,
                    }
                })?)?);
                let args_sexp = kwargs.get("args").ok_or_else(|| ParseError::Expected {
                    expected: ":args field".to_string(),
                    found: "missing".to_string(),
                    pos: list.pos,
                })?;
                let args = self.build_expr_list(args_sexp)?;
                Ok(ExprKind::Call { func, args })
            }
            "MethodCall" => {
                let kwargs = parse_kwargs(list)?;
                let receiver =
                    Box::new(self.build_expr(kwargs.get("receiver").ok_or_else(|| {
                        ParseError::Expected {
                            expected: ":receiver field".to_string(),
                            found: "missing".to_string(),
                            pos: list.pos,
                        }
                    })?)?);
                let method = self.build_ident(kwargs.get("method").ok_or_else(|| {
                    ParseError::Expected {
                        expected: ":method field".to_string(),
                        found: "missing".to_string(),
                        pos: list.pos,
                    }
                })?)?;
                let args_sexp = kwargs.get("args").ok_or_else(|| ParseError::Expected {
                    expected: ":args field".to_string(),
                    found: "missing".to_string(),
                    pos: list.pos,
                })?;
                let args = self.build_expr_list(args_sexp)?;
                Ok(ExprKind::MethodCall { receiver, method, args })
            }
            _ => Err(ParseError::Expected {
                expected: "Supported ExprKind variant".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            }),
        }
    }

    pub(super) fn build_mac_call_inner(&mut self, list: &crate::sexp::List) -> Result<MacCall> {
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
        // Handle both `Empty` as a bare symbol and `(Empty)` as a list
        if let Ok(sym) = expect_symbol(sexp) {
            if sym.value == "Empty" {
                return Ok(MacArgs::Empty);
            }
        }

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
        // Handle both `Empty` as a bare symbol and `(TokenStream ...)` as a list
        if let Ok(sym) = expect_symbol(sexp) {
            if sym.value == "Empty" {
                return Ok(TokenStream::Empty);
            }
        }

        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        match node_type.value.as_str() {
            "Empty" => Ok(TokenStream::Empty),
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

    pub fn build_pat(&mut self, sexp: &SExp) -> Result<Pat> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "Pat" {
            return Err(ParseError::Expected {
                expected: "Pat".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let kind = if let Some(kind_sexp) = kwargs.get("kind") {
            self.build_pat_kind(kind_sexp)?
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

        Ok(Pat { id, kind, span, tokens: None })
    }

    fn build_pat_kind(&mut self, sexp: &SExp) -> Result<PatKind> {
        // Handle symbol patterns (Wild)
        if let SExp::Symbol(sym) = sexp {
            return match sym.value.as_str() {
                "Wild" => Ok(PatKind::Wild),
                _ => Err(ParseError::Expected {
                    expected: "Wild or pattern list".to_string(),
                    found: sym.value.clone(),
                    pos: sym.pos,
                }),
            };
        }

        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        match node_type.value.as_str() {
            "Ident" => {
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
                let binding_mode = if let Some(bm_sexp) = kwargs.get("binding-mode") {
                    self.build_binding_mode(bm_sexp)?
                } else {
                    BindingMode::ByValue(Mutability::Not)
                };
                let sub = if let Some(sub_sexp) = kwargs.get("sub") {
                    if !is_nil(sub_sexp) {
                        Some(Box::new(self.build_pat(sub_sexp)?))
                    } else {
                        None
                    }
                } else {
                    None
                };
                Ok(PatKind::Ident { binding_mode, ident, sub })
            }
            "Struct" => {
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
                let fields = if let Some(fields_sexp) = kwargs.get("fields") {
                    self.build_pat_field_list(fields_sexp)?
                } else {
                    Vec::new()
                };
                Ok(PatKind::Struct { path, fields })
            }
            "TupleStruct" => {
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
                let elems = if let Some(elems_sexp) = kwargs.get("elems") {
                    self.build_pat_list(elems_sexp)?
                } else {
                    Vec::new()
                };
                Ok(PatKind::TupleStruct { path, elems })
            }
            "Tuple" => {
                let pats = self.build_pat_list(&list.elements[1])?;
                Ok(PatKind::Tuple(pats))
            }
            "Slice" => {
                let pats = self.build_pat_list(&list.elements[1])?;
                Ok(PatKind::Slice(pats))
            }
            "Or" => {
                let pats = self.build_pat_list(&list.elements[1])?;
                Ok(PatKind::Or(pats))
            }
            "Ref" => {
                let kwargs = parse_kwargs(list)?;
                let pat = if let Some(pat_sexp) = kwargs.get("pat") {
                    Box::new(self.build_pat(pat_sexp)?)
                } else {
                    return Err(ParseError::Expected {
                        expected: ":pat field".to_string(),
                        found: "missing".to_string(),
                        pos: list.pos,
                    });
                };
                let mutability = if let Some(mut_sexp) = kwargs.get("mutability") {
                    let sym = expect_symbol(mut_sexp)?;
                    match sym.value.as_str() {
                        "Mut" => Mutability::Mut,
                        "Not" => Mutability::Not,
                        _ => Mutability::Not,
                    }
                } else {
                    Mutability::Not
                };
                Ok(PatKind::Ref { pat, mutability })
            }
            "Lit" => {
                let expr = Box::new(self.build_expr(&list.elements[1])?);
                Ok(PatKind::Lit(expr))
            }
            _ => Err(ParseError::Expected {
                expected: "Supported PatKind variant".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            }),
        }
    }

    fn build_arm(&mut self, sexp: &SExp) -> Result<Arm> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "Arm" {
            return Err(ParseError::Expected {
                expected: "Arm".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let pat = self.build_pat(kwargs.get("pat").ok_or_else(|| ParseError::Expected {
            expected: ":pat field".to_string(),
            found: "missing".to_string(),
            pos: list.pos,
        })?)?;

        let body = Box::new(self.build_expr(kwargs.get("body").ok_or_else(|| {
            ParseError::Expected {
                expected: ":body field".to_string(),
                found: "missing".to_string(),
                pos: list.pos,
            }
        })?)?);

        let guard = if let Some(guard_sexp) = kwargs.get("guard") {
            if !is_nil(guard_sexp) {
                Some(Box::new(self.build_expr(guard_sexp)?))
            } else {
                None
            }
        } else {
            None
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

        Ok(Arm { attrs: Vec::new(), pat, guard, body, span, id })
    }

    fn build_label(&mut self, sexp: &SExp) -> Result<Label> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "Label" {
            return Err(ParseError::Expected {
                expected: "Label".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let ident =
            self.build_ident(kwargs.get("ident").ok_or_else(|| ParseError::Expected {
                expected: ":ident field".to_string(),
                found: "missing".to_string(),
                pos: list.pos,
            })?)?;

        Ok(Label { ident })
    }

    fn build_binop(&mut self, sexp: &SExp) -> Result<BinOp> {
        let sym = expect_symbol(sexp)?;
        match sym.value.as_str() {
            "Add" => Ok(BinOp::Add),
            "Sub" => Ok(BinOp::Sub),
            "Mul" => Ok(BinOp::Mul),
            "Div" => Ok(BinOp::Div),
            "Rem" => Ok(BinOp::Rem),
            "And" => Ok(BinOp::And),
            "Or" => Ok(BinOp::Or),
            "BitAnd" => Ok(BinOp::BitAnd),
            "BitOr" => Ok(BinOp::BitOr),
            "BitXor" => Ok(BinOp::BitXor),
            "Shl" => Ok(BinOp::Shl),
            "Shr" => Ok(BinOp::Shr),
            "Eq" => Ok(BinOp::Eq),
            "Ne" => Ok(BinOp::Ne),
            "Lt" => Ok(BinOp::Lt),
            "Le" => Ok(BinOp::Le),
            "Gt" => Ok(BinOp::Gt),
            "Ge" => Ok(BinOp::Ge),
            _ => Err(ParseError::Expected {
                expected: "BinOp variant".to_string(),
                found: sym.value.clone(),
                pos: sym.pos,
            }),
        }
    }

    fn build_unop(&mut self, sexp: &SExp) -> Result<UnOp> {
        let sym = expect_symbol(sexp)?;
        match sym.value.as_str() {
            "Not" => Ok(UnOp::Not),
            "Neg" => Ok(UnOp::Neg),
            "Deref" => Ok(UnOp::Deref),
            _ => Err(ParseError::Expected {
                expected: "UnOp variant".to_string(),
                found: sym.value.clone(),
                pos: sym.pos,
            }),
        }
    }

    fn build_expr_list(&mut self, sexp: &SExp) -> Result<Vec<Expr>> {
        let list = expect_list(sexp)?;
        list.elements.iter().map(|elem| self.build_expr(elem)).collect()
    }

    fn build_pat_list(&mut self, sexp: &SExp) -> Result<Vec<Pat>> {
        let list = expect_list(sexp)?;
        list.elements.iter().map(|elem| self.build_pat(elem)).collect()
    }

    fn build_pat_field_list(&mut self, sexp: &SExp) -> Result<Vec<PatField>> {
        let list = expect_list(sexp)?;
        list.elements.iter().map(|elem| self.build_pat_field(elem)).collect()
    }

    fn build_pat_field(&mut self, sexp: &SExp) -> Result<PatField> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "PatField" {
            return Err(ParseError::Expected {
                expected: "PatField".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        // TODO: Implement build_attr_vec
        let attrs = Vec::new();

        let ident = if let Some(ident_sexp) = kwargs.get("ident") {
            self.build_ident(ident_sexp)?
        } else {
            return Err(ParseError::Expected {
                expected: ":ident field".to_string(),
                found: "missing".to_string(),
                pos: list.pos,
            });
        };

        let pat = if let Some(pat_sexp) = kwargs.get("pat") {
            self.build_pat(pat_sexp)?
        } else {
            return Err(ParseError::Expected {
                expected: ":pat field".to_string(),
                found: "missing".to_string(),
                pos: list.pos,
            });
        };

        let is_shorthand = if let Some(sh_sexp) = kwargs.get("is-shorthand") {
            if let SExp::Symbol(sym) = sh_sexp {
                sym.value == "true"
            } else {
                false
            }
        } else {
            false
        };

        let span = if let Some(span_sexp) = kwargs.get("span") {
            self.build_span(span_sexp)?
        } else {
            Span::DUMMY
        };

        Ok(PatField { attrs, ident, pat, is_shorthand, span })
    }

    fn build_binding_mode(&mut self, sexp: &SExp) -> Result<BindingMode> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        match node_type.value.as_str() {
            "ByRef" => {
                // For now, simplified - just get the mutability from element 1
                let mutability = if list.elements.len() > 1 {
                    let sym = expect_symbol(&list.elements[1])?;
                    match sym.value.as_str() {
                        "Mut" => Mutability::Mut,
                        "Not" => Mutability::Not,
                        _ => Mutability::Not,
                    }
                } else {
                    Mutability::Not
                };
                Ok(BindingMode::ByRef(mutability))
            }
            "ByValue" => {
                let mutability = if list.elements.len() > 1 {
                    let sym = expect_symbol(&list.elements[1])?;
                    match sym.value.as_str() {
                        "Mut" => Mutability::Mut,
                        "Not" => Mutability::Not,
                        _ => Mutability::Not,
                    }
                } else {
                    Mutability::Not
                };
                Ok(BindingMode::ByValue(mutability))
            }
            _ => Err(ParseError::Expected {
                expected: "ByRef or ByValue".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            }),
        }
    }
}
