use crate::ast::*;
use crate::error::{ParseError, Position, Result};

/// An error comment to be inserted in output
#[derive(Debug, Clone)]
pub struct ErrorComment {
    pub error_message: String,
    pub rust_code: String,
}

/// Convert syn::File to our Crate
pub fn from_syn_file(file: &syn::File) -> Result<Crate> {
    let mut converter = SynConverter::new();
    converter.convert_file(file)
}

/// Convert syn::File to AST, collecting errors instead of failing
pub fn from_syn_file_partial(file: &syn::File) -> (Crate, Vec<ErrorComment>) {
    let mut converter = SynConverter::new();
    let mut successful_items = Vec::new();
    let mut error_comments = Vec::new();

    for item in &file.items {
        match converter.convert_item(item) {
            Ok(ast_item) => {
                successful_items.push(ast_item);
            }
            Err(e) => {
                // Generate pretty Rust code for the failed item
                let rust_code = prettyprint_item(item);

                error_comments.push(ErrorComment {
                    error_message: e.to_string(),
                    rust_code,
                });
            }
        }
    }

    // Create Crate with successful items
    let inner_span = Span::new(0, 0);
    let spans = ModSpans::new(inner_span);
    let crate_ast = Crate::new(successful_items, spans, converter.next_id());

    (crate_ast, error_comments)
}

/// Pretty-print a syn::Item back to Rust code
fn prettyprint_item(item: &syn::Item) -> String {
    // prettyplease requires a File, so wrap the item
    let file = syn::File {
        shebang: None,
        attrs: vec![],
        items: vec![item.clone()],
    };

    prettyplease::unparse(&file)
}

/// Generate S-expression comment block for an error
pub fn generate_error_comment(error: &ErrorComment) -> String {
    let mut lines = vec![
        ";; Oxur AST does not support the following Rust code".to_string(),
        format!(";; Error: {}", error.error_message),
        ";;".to_string(),
    ];

    // Comment out each line of Rust code
    for line in error.rust_code.lines() {
        lines.push(format!(";; {}", line));
    }

    lines.join("\n")
}

struct SynConverter {
    next_node_id: usize,
}

impl SynConverter {
    fn new() -> Self {
        Self { next_node_id: 0 }
    }

    fn next_id(&mut self) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id += 1;
        NodeId(id as u32)
    }

    fn convert_file(&mut self, file: &syn::File) -> Result<Crate> {
        let items =
            file.items.iter().map(|item| self.convert_item(item)).collect::<Result<Vec<_>>>()?;

        // Create spans from syn::File
        // Note: syn doesn't give us exact byte offsets easily, so we approximate
        let inner_span = Span::new(0, 0);
        let spans = ModSpans::new(inner_span);

        Ok(Crate::new(items, spans, self.next_id()))
    }

    fn convert_item(&mut self, item: &syn::Item) -> Result<Item> {
        match item {
            syn::Item::Fn(item_fn) => self.convert_item_fn(item_fn),
            syn::Item::Const(_) => Err(ParseError::Expected {
                expected: "supported item type (currently only: `fn`)".to_string(),
                found: "`const` item".to_string(),
                pos: Position::new(0, 1, 1),
            }),
            syn::Item::Enum(_) => Err(ParseError::Expected {
                expected: "supported item type (currently only: `fn`)".to_string(),
                found: "`enum` item".to_string(),
                pos: Position::new(0, 1, 1),
            }),
            syn::Item::ExternCrate(_) => Err(ParseError::Expected {
                expected: "supported item type (currently only: `fn`)".to_string(),
                found: "`extern crate` item".to_string(),
                pos: Position::new(0, 1, 1),
            }),
            syn::Item::ForeignMod(_) => Err(ParseError::Expected {
                expected: "supported item type (currently only: `fn`)".to_string(),
                found: "`extern` block item".to_string(),
                pos: Position::new(0, 1, 1),
            }),
            syn::Item::Impl(_) => Err(ParseError::Expected {
                expected: "supported item type (currently only: `fn`)".to_string(),
                found: "`impl` block".to_string(),
                pos: Position::new(0, 1, 1),
            }),
            syn::Item::Macro(_) => Err(ParseError::Expected {
                expected: "supported item type (currently only: `fn`)".to_string(),
                found: "macro definition".to_string(),
                pos: Position::new(0, 1, 1),
            }),
            syn::Item::Mod(_) => Err(ParseError::Expected {
                expected: "supported item type (currently only: `fn`)".to_string(),
                found: "`mod` item".to_string(),
                pos: Position::new(0, 1, 1),
            }),
            syn::Item::Static(_) => Err(ParseError::Expected {
                expected: "supported item type (currently only: `fn`)".to_string(),
                found: "`static` item".to_string(),
                pos: Position::new(0, 1, 1),
            }),
            syn::Item::Struct(_) => Err(ParseError::Expected {
                expected: "supported item type (currently only: `fn`)".to_string(),
                found: "`struct` item".to_string(),
                pos: Position::new(0, 1, 1),
            }),
            syn::Item::Trait(_) => Err(ParseError::Expected {
                expected: "supported item type (currently only: `fn`)".to_string(),
                found: "`trait` item".to_string(),
                pos: Position::new(0, 1, 1),
            }),
            syn::Item::TraitAlias(_) => Err(ParseError::Expected {
                expected: "supported item type (currently only: `fn`)".to_string(),
                found: "`trait` alias".to_string(),
                pos: Position::new(0, 1, 1),
            }),
            syn::Item::Type(_) => Err(ParseError::Expected {
                expected: "supported item type (currently only: `fn`)".to_string(),
                found: "`type` alias".to_string(),
                pos: Position::new(0, 1, 1),
            }),
            syn::Item::Union(_) => Err(ParseError::Expected {
                expected: "supported item type (currently only: `fn`)".to_string(),
                found: "`union` item".to_string(),
                pos: Position::new(0, 1, 1),
            }),
            syn::Item::Use(_) => Err(ParseError::Expected {
                expected: "supported item type (currently only: `fn`)".to_string(),
                found: "`use` statement".to_string(),
                pos: Position::new(0, 1, 1),
            }),
            _ => Err(ParseError::Expected {
                expected: "supported item type (currently only: `fn`)".to_string(),
                found: "unknown item".to_string(),
                pos: Position::new(0, 1, 1),
            }),
        }
    }

    fn convert_item_fn(&mut self, item_fn: &syn::ItemFn) -> Result<Item> {
        let ident = self.convert_ident(&item_fn.sig.ident);
        let vis = self.convert_visibility(&item_fn.vis);

        let fn_sig = self.convert_fn_sig(&item_fn.sig)?;
        let generics = self.convert_generics(&item_fn.sig.generics)?;
        let body = Some(self.convert_block(&item_fn.block)?);

        let fn_item = Fn { defaultness: Defaultness::Final, sig: fn_sig, generics, body };

        Ok(Item {
            attrs: vec![], // Phase 3: simplified
            id: self.next_id(),
            span: Span::DUMMY, // Will improve with proc-macro2::Span
            vis,
            ident,
            kind: ItemKind::Fn(Box::new(fn_item)),
            tokens: None,
        })
    }

    fn convert_ident(&mut self, ident: &syn::Ident) -> Ident {
        Ident::new(ident.to_string(), Span::DUMMY)
    }

    fn convert_visibility(&mut self, vis: &syn::Visibility) -> Visibility {
        match vis {
            syn::Visibility::Public(_) => Visibility::Public,
            syn::Visibility::Inherited => Visibility::Inherited,
            syn::Visibility::Restricted(_) => {
                // Simplified for Phase 3
                Visibility::Inherited
            }
        }
    }

    fn convert_fn_sig(&mut self, sig: &syn::Signature) -> Result<FnSig> {
        let header = self.convert_fn_header(sig);
        let decl = self.convert_fn_decl(sig)?;

        Ok(FnSig { header, decl, span: Span::DUMMY })
    }

    fn convert_fn_header(&mut self, sig: &syn::Signature) -> FnHeader {
        let safety = match sig.unsafety {
            Some(_) => Safety::Unsafe,
            None => Safety::Default,
        };

        let constness = match sig.constness {
            Some(_) => Constness::Const,
            None => Constness::NotConst,
        };

        let coroutine_kind = sig.asyncness.map(|_| CoroutineKind::Async);

        let ext = match &sig.abi {
            Some(abi) => {
                if let Some(name) = &abi.name {
                    Extern::Explicit(name.value())
                } else {
                    Extern::Explicit("C".to_string())
                }
            }
            None => Extern::None,
        };

        FnHeader { safety, coroutine_kind, constness, ext }
    }

    fn convert_fn_decl(&mut self, sig: &syn::Signature) -> Result<FnDecl> {
        let inputs = sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Typed(pat_type) => Some(self.convert_fn_arg(pat_type)),
                syn::FnArg::Receiver(_) => None, // Skip self for Phase 3
            })
            .collect::<Result<Vec<_>>>()?;

        let output = self.convert_return_type(&sig.output)?;

        Ok(FnDecl { inputs, output })
    }

    fn convert_fn_arg(&mut self, pat_type: &syn::PatType) -> Result<Param> {
        let pat = self.convert_pat(&pat_type.pat)?;
        let ty = self.convert_type(&pat_type.ty)?;

        Ok(Param {
            attrs: vec![],
            ty,
            pat,
            id: self.next_id(),
            span: Span::DUMMY,
            is_placeholder: false,
        })
    }

    fn convert_pat(&mut self, pat: &syn::Pat) -> Result<Pat> {
        match pat {
            syn::Pat::Ident(pat_ident) => {
                let ident = self.convert_ident(&pat_ident.ident);

                let binding_mode = if pat_ident.by_ref.is_some() {
                    BindingMode::ByRef(if pat_ident.mutability.is_some() {
                        Mutability::Mut
                    } else {
                        Mutability::Not
                    })
                } else {
                    BindingMode::ByValue(if pat_ident.mutability.is_some() {
                        Mutability::Mut
                    } else {
                        Mutability::Not
                    })
                };

                Ok(Pat {
                    id: self.next_id(),
                    kind: PatKind::Ident {
                        binding_mode,
                        ident,
                        sub: None, // Phase 3: simplified
                    },
                    span: Span::DUMMY,
                    tokens: None,
                })
            }
            _ => Err(ParseError::Expected {
                expected: "ident pattern".to_string(),
                found: "complex pattern".to_string(),
                pos: Position::new(0, 1, 1),
            }),
        }
    }

    fn convert_type(&mut self, ty: &syn::Type) -> Result<Ty> {
        match ty {
            syn::Type::Path(type_path) => {
                let path = self.convert_path(&type_path.path)?;
                Ok(Ty {
                    id: self.next_id(),
                    kind: TyKind::Path(None, path),
                    span: Span::DUMMY,
                    tokens: None,
                })
            }
            _ => Err(ParseError::Expected {
                expected: "path type".to_string(),
                found: "complex type".to_string(),
                pos: Position::new(0, 1, 1),
            }),
        }
    }

    fn convert_path(&mut self, path: &syn::Path) -> Result<Path> {
        let segments = path
            .segments
            .iter()
            .map(|seg| {
                let ident = self.convert_ident(&seg.ident);
                PathSegment::from_ident(ident)
            })
            .collect();

        Ok(Path { span: Span::DUMMY, segments, tokens: None })
    }

    fn convert_return_type(&mut self, ret: &syn::ReturnType) -> Result<FnRetTy> {
        match ret {
            syn::ReturnType::Default => Ok(FnRetTy::Default(Span::DUMMY)),
            syn::ReturnType::Type(_, ty) => Ok(FnRetTy::Ty(Box::new(self.convert_type(ty)?))),
        }
    }

    fn convert_generics(&mut self, _generics: &syn::Generics) -> Result<Generics> {
        // Simplified for Phase 3 - just create empty generics
        Ok(Generics {
            params: vec![],
            where_clause: WhereClause {
                has_where_token: false,
                predicates: vec![],
                span: Span::DUMMY,
            },
            span: Span::DUMMY,
        })
    }

    fn convert_block(&mut self, block: &syn::Block) -> Result<Block> {
        let stmts =
            block.stmts.iter().map(|stmt| self.convert_stmt(stmt)).collect::<Result<Vec<_>>>()?;

        Ok(Block::new(stmts, self.next_id(), Span::DUMMY))
    }

    fn convert_stmt(&mut self, stmt: &syn::Stmt) -> Result<Stmt> {
        match stmt {
            syn::Stmt::Expr(expr, semi) => {
                let expr = self.convert_expr(expr)?;
                let kind = if semi.is_some() { StmtKind::Semi(expr) } else { StmtKind::Expr(expr) };

                Ok(Stmt { id: self.next_id(), kind, span: Span::DUMMY })
            }
            syn::Stmt::Local(local) => {
                let local = self.convert_local(local)?;
                Ok(Stmt {
                    id: self.next_id(),
                    kind: StmtKind::Let(Box::new(local)),
                    span: Span::DUMMY,
                })
            }
            syn::Stmt::Item(_) => {
                // Skip items in blocks for Phase 3
                Ok(Stmt { id: self.next_id(), kind: StmtKind::Empty, span: Span::DUMMY })
            }
            syn::Stmt::Macro(mac) => {
                // Convert macro statement
                self.convert_macro_stmt(mac)
            }
        }
    }

    fn convert_local(&mut self, local: &syn::Local) -> Result<Local> {
        let pat = self.convert_pat(&local.pat)?;

        // syn::Local doesn't have direct ty field anymore
        let ty = None; // Phase 3: simplified

        let kind = if let Some(init) = &local.init {
            let expr = self.convert_expr(&init.expr)?;
            let local_init = LocalInit { expr, els: None }; // Phase 3: simplified
            LocalKind::Init(local_init)
        } else {
            LocalKind::Decl
        };

        Ok(Local { pat, kind, span: Span::DUMMY, ty, attrs: vec![], tokens: None })
    }

    fn convert_expr(&mut self, expr: &syn::Expr) -> Result<Expr> {
        let kind = match expr {
            syn::Expr::Macro(expr_macro) => {
                let mac_call = self.convert_macro(&expr_macro.mac)?;
                ExprKind::MacCall(mac_call)
            }
            syn::Expr::Lit(expr_lit) => {
                let lit = self.convert_lit(&expr_lit.lit)?;
                ExprKind::Lit(lit)
            }
            syn::Expr::Path(expr_path) => {
                let path = self.convert_path(&expr_path.path)?;
                ExprKind::Path(None, path)
            }
            _ => {
                return Err(ParseError::Expected {
                    expected: "supported expression".to_string(),
                    found: "complex expression".to_string(),
                    pos: Position::new(0, 1, 1),
                });
            }
        };

        Ok(Expr { id: self.next_id(), kind, span: Span::DUMMY, attrs: vec![], tokens: None })
    }

    fn convert_macro(&mut self, mac: &syn::Macro) -> Result<MacCall> {
        let path = self.convert_path(&mac.path)?;

        // Convert tokens to string representation
        let tokens_str = mac.tokens.to_string();

        let args = MacArgs::Delimited {
            dspan: DelSpan::new(Span::DUMMY, Span::DUMMY),
            delim: self.convert_delimiter(&mac.delimiter),
            tokens: TokenStream::Source(tokens_str),
        };

        Ok(MacCall { path, args, prior_type_ascription: None })
    }

    fn convert_delimiter(&mut self, delim: &syn::MacroDelimiter) -> Delimiter {
        match delim {
            syn::MacroDelimiter::Paren(_) => Delimiter::Paren,
            syn::MacroDelimiter::Brace(_) => Delimiter::Brace,
            syn::MacroDelimiter::Bracket(_) => Delimiter::Bracket,
        }
    }

    fn convert_macro_stmt(&mut self, mac: &syn::StmtMacro) -> Result<Stmt> {
        let mac_call = self.convert_macro(&mac.mac)?;

        let style =
            if mac.semi_token.is_some() { MacStmtStyle::Semicolon } else { MacStmtStyle::Braces };

        let mac_call_stmt = MacCallStmt { mac: mac_call, style, attrs: vec![], tokens: None };

        Ok(Stmt { id: self.next_id(), kind: StmtKind::MacCall(mac_call_stmt), span: Span::DUMMY })
    }

    fn convert_lit(&mut self, lit: &syn::Lit) -> Result<Lit> {
        let kind = match lit {
            syn::Lit::Str(lit_str) => LitKind::Str(lit_str.value()),
            syn::Lit::Int(lit_int) => {
                let value =
                    lit_int.base10_digits().parse::<i128>().map_err(|_| ParseError::Expected {
                        expected: "valid integer".to_string(),
                        found: lit_int.base10_digits().to_string(),
                        pos: Position::new(0, 1, 1),
                    })?;
                LitKind::Int(value)
            }
            _ => {
                return Err(ParseError::Expected {
                    expected: "string or int literal".to_string(),
                    found: "other literal".to_string(),
                    pos: Position::new(0, 1, 1),
                });
            }
        };

        Ok(Lit { kind, span: Span::DUMMY })
    }
}
