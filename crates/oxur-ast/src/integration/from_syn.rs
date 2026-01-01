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

                error_comments.push(ErrorComment { error_message: e.to_string(), rust_code });
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
    let file = syn::File { shebang: None, attrs: vec![], items: vec![item.clone()] };

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
            syn::Item::Struct(item_struct) => self.convert_item_struct(item_struct),
            syn::Item::Enum(item_enum) => self.convert_item_enum(item_enum),
            syn::Item::Trait(item_trait) => self.convert_item_trait(item_trait),
            syn::Item::Impl(item_impl) => self.convert_item_impl(item_impl),
            syn::Item::Use(item_use) => self.convert_item_use(item_use),
            syn::Item::Static(item_static) => self.convert_item_static(item_static),
            syn::Item::Const(item_const) => self.convert_item_const(item_const),
            syn::Item::Type(item_type) => self.convert_item_type(item_type),
            syn::Item::Mod(item_mod) => self.convert_item_mod(item_mod),
            syn::Item::ExternCrate(_) => Err(ParseError::Expected {
                expected: "supported item type (currently: `fn`, `struct`, `enum`, `trait`, `impl`, `use`, `static`, `const`, `type`, `mod`)".to_string(),
                found: "`extern crate` item".to_string(),
                pos: Position::new(0, 1, 1),
            }),
            syn::Item::ForeignMod(_) => Err(ParseError::Expected {
                expected: "supported item type (currently: `fn`, `struct`, `enum`, `trait`, `impl`, `use`, `static`, `const`, `type`, `mod`)".to_string(),
                found: "`extern` block item".to_string(),
                pos: Position::new(0, 1, 1),
            }),
            syn::Item::Macro(_) => Err(ParseError::Expected {
                expected: "supported item type (currently: `fn`, `struct`, `enum`, `trait`, `impl`, `use`, `static`, `const`, `type`, `mod`)".to_string(),
                found: "macro definition".to_string(),
                pos: Position::new(0, 1, 1),
            }),
            syn::Item::TraitAlias(_) => Err(ParseError::Expected {
                expected: "supported item type (currently: `fn`, `struct`, `enum`, `trait`, `impl`, `use`, `static`, `const`, `type`, `mod`)".to_string(),
                found: "`trait` alias".to_string(),
                pos: Position::new(0, 1, 1),
            }),
            syn::Item::Union(_) => Err(ParseError::Expected {
                expected: "supported item type (currently: `fn`, `struct`, `enum`, `trait`, `impl`, `use`, `static`, `const`, `type`, `mod`)".to_string(),
                found: "`union` item".to_string(),
                pos: Position::new(0, 1, 1),
            }),
            _ => Err(ParseError::Expected {
                expected: "supported item type (currently: `fn`, `struct`, `enum`, `trait`, `impl`, `use`, `static`, `const`, `type`, `mod`)".to_string(),
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

    fn convert_item_struct(&mut self, item_struct: &syn::ItemStruct) -> Result<Item> {
        let ident = self.convert_ident(&item_struct.ident);
        let vis = self.convert_visibility(&item_struct.vis);

        // Convert fields based on struct type
        let variant_data = match &item_struct.fields {
            syn::Fields::Named(fields_named) => {
                let fields = fields_named
                    .named
                    .iter()
                    .map(|f| self.convert_field(f))
                    .collect::<Result<Vec<_>>>()?;
                VariantData::Struct { fields, recovered: false }
            }
            syn::Fields::Unnamed(fields_unnamed) => {
                let fields = fields_unnamed
                    .unnamed
                    .iter()
                    .map(|f| self.convert_field(f))
                    .collect::<Result<Vec<_>>>()?;
                VariantData::Tuple(fields)
            }
            syn::Fields::Unit => VariantData::Unit,
        };

        Ok(Item {
            attrs: vec![],
            id: self.next_id(),
            span: Span::DUMMY,
            vis,
            ident,
            kind: ItemKind::Struct(variant_data),
            tokens: None,
        })
    }

    fn convert_field(&mut self, field: &syn::Field) -> Result<FieldDef> {
        let ident = field.ident.as_ref().map(|i| self.convert_ident(i));
        let ty = self.convert_type(&field.ty)?;
        let vis = self.convert_visibility(&field.vis);

        Ok(FieldDef {
            attrs: vec![],
            id: self.next_id(),
            span: Span::DUMMY,
            vis,
            ident,
            ty,
        })
    }

    fn convert_item_enum(&mut self, item_enum: &syn::ItemEnum) -> Result<Item> {
        let ident = self.convert_ident(&item_enum.ident);
        let vis = self.convert_visibility(&item_enum.vis);

        // Convert all variants
        let variants =
            item_enum.variants.iter().map(|v| self.convert_variant(v)).collect::<Result<Vec<_>>>()?;

        let enum_def = EnumDef { variants };

        Ok(Item {
            attrs: vec![],
            id: self.next_id(),
            span: Span::DUMMY,
            vis,
            ident,
            kind: ItemKind::Enum(enum_def),
            tokens: None,
        })
    }

    fn convert_variant(&mut self, variant: &syn::Variant) -> Result<Variant> {
        let ident = self.convert_ident(&variant.ident);
        let vis = Visibility::Inherited; // Variants inherit visibility from enum

        // Convert variant fields (same as struct fields)
        let data = match &variant.fields {
            syn::Fields::Named(fields_named) => {
                let fields = fields_named
                    .named
                    .iter()
                    .map(|f| self.convert_field(f))
                    .collect::<Result<Vec<_>>>()?;
                VariantData::Struct { fields, recovered: false }
            }
            syn::Fields::Unnamed(fields_unnamed) => {
                let fields = fields_unnamed
                    .unnamed
                    .iter()
                    .map(|f| self.convert_field(f))
                    .collect::<Result<Vec<_>>>()?;
                VariantData::Tuple(fields)
            }
            syn::Fields::Unit => VariantData::Unit,
        };

        // Convert discriminant if present (e.g., Foo = 1)
        let disr_expr = if let Some((_, expr)) = &variant.discriminant {
            Some(self.convert_expr(expr)?)
        } else {
            None
        };

        Ok(Variant {
            attrs: vec![],
            id: self.next_id(),
            span: Span::DUMMY,
            vis,
            ident,
            data,
            disr_expr,
        })
    }

    fn convert_item_impl(&mut self, item_impl: &syn::ItemImpl) -> Result<Item> {
        // Impl blocks don't have an ident like other items, so we create a dummy one
        let ident = Ident::new("impl".to_string(), Span::DUMMY);
        let vis = Visibility::Inherited; // Impl blocks don't have visibility

        // Convert safety (unsafe impl)
        let safety = match item_impl.unsafety {
            Some(_) => Safety::Unsafe,
            None => Safety::Safe,
        };

        // Convert generics
        let generics = self.convert_generics(&item_impl.generics)?;

        // Convert optional trait reference (trait impl vs inherent impl)
        let of_trait = if let Some((_, path, _)) = &item_impl.trait_ {
            Some(TraitRef { path: self.convert_path(path)? })
        } else {
            None
        };

        // Convert self type (the type being implemented for)
        let self_ty = self.convert_type(&item_impl.self_ty)?;

        // Convert impl items (methods, associated types, etc.)
        let items = item_impl
            .items
            .iter()
            .map(|item| self.convert_impl_item(item))
            .collect::<Result<Vec<_>>>()?;

        let impl_def = ImplDef { safety, generics, of_trait, self_ty, items };

        Ok(Item {
            attrs: vec![],
            id: self.next_id(),
            span: Span::DUMMY,
            vis,
            ident,
            kind: ItemKind::Impl(Box::new(impl_def)),
            tokens: None,
        })
    }

    fn convert_impl_item(&mut self, item: &syn::ImplItem) -> Result<AssocItem> {
        match item {
            syn::ImplItem::Fn(method) => {
                let ident = self.convert_ident(&method.sig.ident);
                let vis = self.convert_visibility(&method.vis);

                // Convert the function signature
                let sig = self.convert_fn_sig(&method.sig)?;
                let generics = self.convert_generics(&method.sig.generics)?;

                // Impl methods always have a body
                let body = Some(self.convert_block(&method.block)?);

                let fn_def = Fn {
                    defaultness: Defaultness::Final,
                    sig,
                    generics,
                    body,
                };

                Ok(AssocItem {
                    attrs: vec![],
                    id: self.next_id(),
                    span: Span::DUMMY,
                    vis,
                    ident,
                    kind: AssocItemKind::Fn(Box::new(fn_def)),
                })
            }
            syn::ImplItem::Type(ty) => {
                let ident = self.convert_ident(&ty.ident);
                let vis = self.convert_visibility(&ty.vis);

                // Associated type in impl always has a concrete type
                let concrete_ty = self.convert_type(&ty.ty)?;

                Ok(AssocItem {
                    attrs: vec![],
                    id: self.next_id(),
                    span: Span::DUMMY,
                    vis,
                    ident,
                    kind: AssocItemKind::Type(Box::new(Some(concrete_ty))),
                })
            }
            _ => Err(ParseError::Expected {
                expected: "impl method or associated type".to_string(),
                found: "unsupported impl item (const, macro)".to_string(),
                pos: Position::new(0, 1, 1),
            }),
        }
    }

    fn convert_item_use(&mut self, item_use: &syn::ItemUse) -> Result<Item> {
        // Use declarations don't have a meaningful ident, so we create a dummy one
        let ident = Ident::new("use".to_string(), Span::DUMMY);
        let vis = self.convert_visibility(&item_use.vis);

        // Convert the use tree
        let use_tree = self.convert_use_tree(&item_use.tree)?;

        Ok(Item {
            attrs: vec![],
            id: self.next_id(),
            span: Span::DUMMY,
            vis,
            ident,
            kind: ItemKind::Use(use_tree),
            tokens: None,
        })
    }

    fn convert_use_tree(&mut self, tree: &syn::UseTree) -> Result<UseTree> {
        match tree {
            syn::UseTree::Path(use_path) => {
                // Recursive case: `foo::bar::...`
                let segment = PathSegment {
                    ident: self.convert_ident(&use_path.ident),
                    id: NodeId::DUMMY,
                    args: None,
                };

                // Recursively convert the rest of the tree
                let rest = self.convert_use_tree(&use_path.tree)?;

                // Combine the current segment with the rest
                let mut segments = vec![segment];
                segments.extend(rest.prefix.segments);

                Ok(UseTree {
                    prefix: Path {
                        span: Span::DUMMY,
                        segments,
                        tokens: None,
                    },
                    kind: rest.kind,
                })
            }
            syn::UseTree::Name(use_name) => {
                // Simple name: `use foo::bar;`
                let segment = PathSegment {
                    ident: self.convert_ident(&use_name.ident),
                    id: NodeId::DUMMY,
                    args: None,
                };

                Ok(UseTree {
                    prefix: Path {
                        span: Span::DUMMY,
                        segments: vec![segment],
                        tokens: None,
                    },
                    kind: UseTreeKind::Simple(None), // No renaming
                })
            }
            syn::UseTree::Rename(use_rename) => {
                // Rename: `use foo::bar as baz;`
                let segment = PathSegment {
                    ident: self.convert_ident(&use_rename.ident),
                    id: NodeId::DUMMY,
                    args: None,
                };

                let rename = self.convert_ident(&use_rename.rename);

                Ok(UseTree {
                    prefix: Path {
                        span: Span::DUMMY,
                        segments: vec![segment],
                        tokens: None,
                    },
                    kind: UseTreeKind::Simple(Some(rename)),
                })
            }
            syn::UseTree::Glob(_) => {
                // Glob: `use foo::*;`
                Ok(UseTree {
                    prefix: Path {
                        span: Span::DUMMY,
                        segments: vec![],
                        tokens: None,
                    },
                    kind: UseTreeKind::Glob,
                })
            }
            syn::UseTree::Group(use_group) => {
                // Nested: `use foo::{bar, baz};`
                let items = use_group
                    .items
                    .iter()
                    .map(|item| self.convert_use_tree(item))
                    .collect::<Result<Vec<_>>>()?;

                Ok(UseTree {
                    prefix: Path {
                        span: Span::DUMMY,
                        segments: vec![],
                        tokens: None,
                    },
                    kind: UseTreeKind::Nested(items),
                })
            }
        }
    }

    fn convert_item_static(&mut self, item_static: &syn::ItemStatic) -> Result<Item> {
        let ident = self.convert_ident(&item_static.ident);
        let vis = self.convert_visibility(&item_static.vis);

        // Convert mutability
        let mutability = match item_static.mutability {
            syn::StaticMutability::Mut(_) => Mutability::Mut,
            syn::StaticMutability::None => Mutability::Not,
            _ => Mutability::Not, // Future-proof for new variants
        };

        // Convert type
        let ty = self.convert_type(&item_static.ty)?;

        // Convert initializer expression
        let expr = Some(self.convert_expr(&item_static.expr)?);

        Ok(Item {
            attrs: vec![],
            id: self.next_id(),
            span: Span::DUMMY,
            vis,
            ident,
            kind: ItemKind::Static { mutability, ty, expr },
            tokens: None,
        })
    }

    fn convert_item_const(&mut self, item_const: &syn::ItemConst) -> Result<Item> {
        let ident = self.convert_ident(&item_const.ident);
        let vis = self.convert_visibility(&item_const.vis);

        // Convert type
        let ty = self.convert_type(&item_const.ty)?;

        // Convert initializer expression
        let expr = Some(self.convert_expr(&item_const.expr)?);

        Ok(Item {
            attrs: vec![],
            id: self.next_id(),
            span: Span::DUMMY,
            vis,
            ident,
            kind: ItemKind::Const { ty, expr },
            tokens: None,
        })
    }

    fn convert_item_type(&mut self, item_type: &syn::ItemType) -> Result<Item> {
        let ident = self.convert_ident(&item_type.ident);
        let vis = self.convert_visibility(&item_type.vis);

        // Convert generics
        let generics = self.convert_generics(&item_type.generics)?;

        // Convert the aliased type
        let ty = Some(self.convert_type(&item_type.ty)?);

        Ok(Item {
            attrs: vec![],
            id: self.next_id(),
            span: Span::DUMMY,
            vis,
            ident,
            kind: ItemKind::TyAlias { generics, ty },
            tokens: None,
        })
    }

    fn convert_item_mod(&mut self, item_mod: &syn::ItemMod) -> Result<Item> {
        let ident = self.convert_ident(&item_mod.ident);
        let vis = self.convert_visibility(&item_mod.vis);

        // Convert module items if present (inline module vs external module)
        let items = if let Some((_, module_items)) = &item_mod.content {
            // Inline module: `mod foo { ... }`
            let converted_items = module_items
                .iter()
                .map(|item| self.convert_item(item))
                .collect::<Result<Vec<_>>>()?;
            Some(converted_items)
        } else {
            // External module: `mod foo;`
            None
        };

        Ok(Item {
            attrs: vec![],
            id: self.next_id(),
            span: Span::DUMMY,
            vis,
            ident,
            kind: ItemKind::Mod { items },
            tokens: None,
        })
    }

    fn convert_type_param_bound(&mut self, bound: &syn::TypeParamBound) -> Result<GenericBound> {
        match bound {
            syn::TypeParamBound::Trait(trait_bound) => {
                let path = self.convert_path(&trait_bound.path)?;
                Ok(GenericBound::Trait(TraitRef { path }))
            }
            syn::TypeParamBound::Lifetime(_) => Err(ParseError::Expected {
                expected: "trait bound".to_string(),
                found: "lifetime bound (not yet supported)".to_string(),
                pos: Position::new(0, 1, 1),
            }),
            _ => Err(ParseError::Expected {
                expected: "trait bound".to_string(),
                found: "unknown bound type".to_string(),
                pos: Position::new(0, 1, 1),
            }),
        }
    }

    fn convert_item_trait(&mut self, item_trait: &syn::ItemTrait) -> Result<Item> {
        let ident = self.convert_ident(&item_trait.ident);
        let vis = self.convert_visibility(&item_trait.vis);

        // Convert safety (unsafe trait or not)
        let safety = match item_trait.unsafety {
            Some(_) => Safety::Unsafe,
            None => Safety::Safe,
        };

        // Convert generics
        let generics = self.convert_generics(&item_trait.generics)?;

        // Convert supertraits (bounds like `: Clone + Send`)
        let bounds = item_trait
            .supertraits
            .iter()
            .map(|b| self.convert_type_param_bound(b))
            .collect::<Result<Vec<_>>>()?;

        // Convert trait items (methods, types, etc.)
        let items = item_trait
            .items
            .iter()
            .map(|item| self.convert_trait_item(item))
            .collect::<Result<Vec<_>>>()?;

        let trait_def = TraitDef { safety, generics, bounds, items };

        Ok(Item {
            attrs: vec![],
            id: self.next_id(),
            span: Span::DUMMY,
            vis,
            ident,
            kind: ItemKind::Trait(Box::new(trait_def)),
            tokens: None,
        })
    }

    fn convert_trait_item(&mut self, item: &syn::TraitItem) -> Result<AssocItem> {
        match item {
            syn::TraitItem::Fn(method) => {
                let ident = self.convert_ident(&method.sig.ident);
                let vis = Visibility::Inherited; // Trait items are public by default

                // Convert the function signature
                let sig = self.convert_fn_sig(&method.sig)?;
                let generics = self.convert_generics(&method.sig.generics)?;

                // Trait methods may or may not have a body
                let body = if let Some(block) = &method.default {
                    Some(self.convert_block(block)?)
                } else {
                    None
                };

                let fn_def = Fn {
                    defaultness: Defaultness::Final,
                    sig,
                    generics,
                    body,
                };

                Ok(AssocItem {
                    attrs: vec![],
                    id: self.next_id(),
                    span: Span::DUMMY,
                    vis,
                    ident,
                    kind: AssocItemKind::Fn(Box::new(fn_def)),
                })
            }
            syn::TraitItem::Type(ty) => {
                let ident = self.convert_ident(&ty.ident);
                let vis = Visibility::Inherited;

                // Associated type may have a default
                let default_ty = if let Some((_eq, ty)) = &ty.default {
                    Some(self.convert_type(ty)?)
                } else {
                    None
                };

                Ok(AssocItem {
                    attrs: vec![],
                    id: self.next_id(),
                    span: Span::DUMMY,
                    vis,
                    ident,
                    kind: AssocItemKind::Type(Box::new(default_ty)),
                })
            }
            _ => Err(ParseError::Expected {
                expected: "trait method or associated type".to_string(),
                found: "unsupported trait item (const, macro)".to_string(),
                pos: Position::new(0, 1, 1),
            }),
        }
    }
}
