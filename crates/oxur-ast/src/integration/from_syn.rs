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
            syn::Item::Macro(item_macro) => self.convert_item_macro(item_macro),
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
        let attrs = self.convert_attributes(&item_fn.attrs)?;
        let ident = self.convert_ident(&item_fn.sig.ident);
        let vis = self.convert_visibility(&item_fn.vis);

        let fn_sig = self.convert_fn_sig(&item_fn.sig)?;
        let generics = self.convert_generics(&item_fn.sig.generics)?;
        let body = Some(self.convert_block(&item_fn.block)?);

        let fn_item = Fn { defaultness: Defaultness::Final, sig: fn_sig, generics, body };

        Ok(Item {
            attrs,
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
            syn::Pat::Type(pat_type) => {
                // Type-annotated pattern: `x: T`
                // For now, recursively convert the inner pattern and ignore the type
                // The type will be extracted separately in convert_local
                self.convert_pat(&pat_type.pat)
            }
            // Phase 9: Wildcard pattern (CRITICAL)
            syn::Pat::Wild(_) => Ok(Pat {
                id: self.next_id(),
                kind: PatKind::Wild,
                span: Span::DUMMY,
                tokens: None,
            }),
            // Phase 9: Tuple patterns
            syn::Pat::Tuple(pat_tuple) => {
                let elems =
                    pat_tuple.elems.iter().map(|p| self.convert_pat(p)).collect::<Result<Vec<_>>>()?;

                Ok(Pat { id: self.next_id(), kind: PatKind::Tuple(elems), span: Span::DUMMY, tokens: None })
            }
            // Phase 9: Struct patterns
            syn::Pat::Struct(pat_struct) => {
                let path = self.convert_path(&pat_struct.path)?;

                let fields = pat_struct
                    .fields
                    .iter()
                    .map(|field| {
                        let ident = match &field.member {
                            syn::Member::Named(name) => self.convert_ident(name),
                            syn::Member::Unnamed(index) => {
                                Ident::new(index.index.to_string(), Span::DUMMY)
                            }
                        };
                        let pat = self.convert_pat(&field.pat)?;
                        Ok(PatField {
                            attrs: vec![],
                            ident,
                            pat,
                            is_shorthand: false, // TODO: detect shorthand
                            span: Span::DUMMY,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;

                Ok(Pat {
                    id: self.next_id(),
                    kind: PatKind::Struct { path, fields },
                    span: Span::DUMMY,
                    tokens: None,
                })
            }
            // Phase 9: Tuple struct patterns
            syn::Pat::TupleStruct(pat_tuple_struct) => {
                let path = self.convert_path(&pat_tuple_struct.path)?;

                let elems = pat_tuple_struct
                    .elems
                    .iter()
                    .map(|p| self.convert_pat(p))
                    .collect::<Result<Vec<_>>>()?;

                Ok(Pat {
                    id: self.next_id(),
                    kind: PatKind::TupleStruct { path, elems },
                    span: Span::DUMMY,
                    tokens: None,
                })
            }
            // Phase 9: Path patterns
            syn::Pat::Path(pat_path) => {
                // TODO: Handle qself properly when QSelf conversion is implemented
                let qself = None;

                let path = self.convert_path(&pat_path.path)?;

                Ok(Pat {
                    id: self.next_id(),
                    kind: PatKind::Path { qself, path },
                    span: Span::DUMMY,
                    tokens: None,
                })
            }
            // Phase 9: Literal patterns
            syn::Pat::Lit(pat_lit) => {
                let lit = self.convert_lit(&pat_lit.lit)?;
                let expr = Expr {
                    id: self.next_id(),
                    kind: ExprKind::Lit(lit),
                    span: Span::DUMMY,
                    attrs: vec![],
                    tokens: None,
                };

                Ok(Pat {
                    id: self.next_id(),
                    kind: PatKind::Lit(Box::new(expr)),
                    span: Span::DUMMY,
                    tokens: None,
                })
            }
            // Phase 9: Range patterns
            syn::Pat::Range(pat_range) => {
                let start = pat_range
                    .start
                    .as_ref()
                    .map(|e| self.convert_expr(e))
                    .transpose()?
                    .map(Box::new);

                let end =
                    pat_range.end.as_ref().map(|e| self.convert_expr(e)).transpose()?.map(Box::new);

                let limits = match &pat_range.limits {
                    syn::RangeLimits::HalfOpen(_) => RangeEnd::Excluded,
                    syn::RangeLimits::Closed(_) => RangeEnd::Included,
                };

                Ok(Pat {
                    id: self.next_id(),
                    kind: PatKind::Range { start, end, limits },
                    span: Span::DUMMY,
                    tokens: None,
                })
            }
            // Phase 9: Reference patterns
            syn::Pat::Reference(pat_ref) => {
                let mutability =
                    if pat_ref.mutability.is_some() { Mutability::Mut } else { Mutability::Not };

                let pat = Box::new(self.convert_pat(&pat_ref.pat)?);

                Ok(Pat {
                    id: self.next_id(),
                    kind: PatKind::Ref { pat, mutability },
                    span: Span::DUMMY,
                    tokens: None,
                })
            }
            // Phase 9: Slice patterns
            syn::Pat::Slice(pat_slice) => {
                let elems =
                    pat_slice.elems.iter().map(|p| self.convert_pat(p)).collect::<Result<Vec<_>>>()?;

                Ok(Pat { id: self.next_id(), kind: PatKind::Slice(elems), span: Span::DUMMY, tokens: None })
            }
            // Phase 9: Or patterns
            syn::Pat::Or(pat_or) => {
                let cases =
                    pat_or.cases.iter().map(|p| self.convert_pat(p)).collect::<Result<Vec<_>>>()?;

                Ok(Pat { id: self.next_id(), kind: PatKind::Or(cases), span: Span::DUMMY, tokens: None })
            }
            // Phase 9: Parenthesized patterns
            syn::Pat::Paren(pat_paren) => {
                let inner = Box::new(self.convert_pat(&pat_paren.pat)?);

                Ok(Pat { id: self.next_id(), kind: PatKind::Paren(inner), span: Span::DUMMY, tokens: None })
            }
            // Phase 9: Rest pattern
            syn::Pat::Rest(_) => {
                Ok(Pat { id: self.next_id(), kind: PatKind::Rest, span: Span::DUMMY, tokens: None })
            }
            _ => Err(ParseError::Expected {
                expected: "ident pattern".to_string(),
                found: "complex pattern".to_string(),
                pos: Position::new(0, 1, 1),
            }),
        }
    }

    fn convert_type(&mut self, ty: &syn::Type) -> Result<Ty> {
        let kind = match ty {
            syn::Type::Path(type_path) => {
                let path = self.convert_path(&type_path.path)?;
                TyKind::Path(None, path)
            }
            syn::Type::Reference(type_ref) => {
                // Convert &T or &mut T or &'a T
                let lifetime = type_ref
                    .lifetime
                    .as_ref()
                    .map(|lt| Lifetime { ident: self.convert_ident(&lt.ident) });

                let mutability =
                    if type_ref.mutability.is_some() { Mutability::Mut } else { Mutability::Not };

                let ty = Box::new(self.convert_type(&type_ref.elem)?);
                TyKind::Ref { lifetime, mutability, ty }
            }
            syn::Type::Slice(type_slice) => {
                // Convert [T]
                let inner_ty = Box::new(self.convert_type(&type_slice.elem)?);
                TyKind::Slice(inner_ty)
            }
            syn::Type::Array(type_array) => {
                // Convert [T; N]
                let ty = Box::new(self.convert_type(&type_array.elem)?);
                let len = Box::new(self.convert_expr(&type_array.len)?);
                TyKind::Array { ty, len }
            }
            syn::Type::Tuple(type_tuple) => {
                // Convert (T, U, V) or ()
                let types = type_tuple
                    .elems
                    .iter()
                    .map(|ty| self.convert_type(ty))
                    .collect::<Result<Vec<_>>>()?;
                TyKind::Tuple(types)
            }
            syn::Type::Ptr(type_ptr) => {
                // Convert *const T or *mut T
                let mutability =
                    if type_ptr.mutability.is_some() { Mutability::Mut } else { Mutability::Not };

                let ty = Box::new(self.convert_type(&type_ptr.elem)?);
                TyKind::Ptr { mutability, ty }
            }
            syn::Type::Never(_) => {
                // Convert !
                TyKind::Never
            }
            syn::Type::Infer(_) => {
                // Convert _
                TyKind::Infer
            }
            // Phase 12: ImplTrait - impl Iterator<Item = i32>
            syn::Type::ImplTrait(type_impl_trait) => {
                let bounds = type_impl_trait
                    .bounds
                    .iter()
                    .map(|bound| self.convert_type_param_bound(bound))
                    .collect::<Result<Vec<_>>>()?;

                TyKind::ImplTrait(bounds)
            }
            // Phase 12: TraitObject - dyn Display + Send
            syn::Type::TraitObject(type_trait_object) => {
                let syntax = if type_trait_object.dyn_token.is_some() {
                    TraitObjectSyntax::Dyn
                } else {
                    TraitObjectSyntax::None
                };

                let bounds = type_trait_object
                    .bounds
                    .iter()
                    .map(|bound| self.convert_type_param_bound(bound))
                    .collect::<Result<Vec<_>>>()?;

                TyKind::TraitObject { bounds, syntax }
            }
            // Phase 12: BareFn - fn(i32) -> bool
            syn::Type::BareFn(type_bare_fn) => {
                let safety = match type_bare_fn.unsafety {
                    Some(_) => Safety::Unsafe,
                    None => Safety::Default,
                };

                let abi = type_bare_fn.abi.as_ref().map(|abi_spec| {
                    abi_spec
                        .name
                        .as_ref()
                        .map(|lit_str| lit_str.value())
                        .unwrap_or_else(|| "Rust".to_string())
                });

                // Input parameters
                let inputs = type_bare_fn
                    .inputs
                    .iter()
                    .map(|bare_fn_arg| {
                        let name = bare_fn_arg.name.as_ref().map(|(ident, _)| self.convert_ident(ident));

                        let ty = self.convert_type(&bare_fn_arg.ty)?;

                        Ok(BareFnParam { attrs: vec![], name, ty })
                    })
                    .collect::<Result<Vec<_>>>()?;

                // Return type
                let output = self.convert_return_type(&type_bare_fn.output)?;

                TyKind::BareFn { safety, abi, inputs, output }
            }
            // Phase 12: Paren - (Type)
            syn::Type::Paren(type_paren) => {
                let inner = Box::new(self.convert_type(&type_paren.elem)?);
                TyKind::Paren(inner)
            }
            // Phase 12: Macro - vec![i32]
            syn::Type::Macro(type_macro) => {
                let mac = self.convert_macro(&type_macro.mac)?;
                TyKind::MacCall(mac)
            }
            _ => {
                return Err(ParseError::Expected {
                    expected:
                        "supported type (path, reference, slice, array, tuple, ptr, never, infer, impl, dyn, fn, paren, macro)"
                            .to_string(),
                    found: "unsupported type".to_string(),
                    pos: Position::new(0, 1, 1),
                })
            }
        };

        Ok(Ty { id: self.next_id(), kind, span: Span::DUMMY, tokens: None })
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

    fn convert_generics(&mut self, generics: &syn::Generics) -> Result<Generics> {
        // Convert generic parameters (lifetimes, types, consts)
        let params = generics
            .params
            .iter()
            .map(|param| self.convert_generic_param(param))
            .collect::<Result<Vec<_>>>()?;

        // Convert where clause
        let where_clause = self.convert_where_clause(&generics.where_clause)?;

        Ok(Generics { params, where_clause, span: Span::DUMMY })
    }

    fn convert_generic_param(&mut self, param: &syn::GenericParam) -> Result<GenericParam> {
        let kind = match param {
            syn::GenericParam::Lifetime(lifetime_param) => {
                GenericParamKind::Lifetime(self.convert_lifetime_param(lifetime_param)?)
            }
            syn::GenericParam::Type(type_param) => {
                GenericParamKind::Type(self.convert_type_param(type_param)?)
            }
            syn::GenericParam::Const(const_param) => {
                GenericParamKind::Const(Box::new(self.convert_const_param(const_param)?))
            }
        };

        Ok(GenericParam { attrs: vec![], id: self.next_id(), span: Span::DUMMY, kind })
    }

    fn convert_lifetime_param(
        &mut self,
        lifetime_param: &syn::LifetimeParam,
    ) -> Result<LifetimeParam> {
        let ident = self.convert_ident(&lifetime_param.lifetime.ident);

        // Convert bounds (e.g., 'a: 'b + 'c)
        let bounds = lifetime_param
            .bounds
            .iter()
            .map(|bound| {
                let ident = self.convert_ident(&bound.ident);
                Lifetime { ident }
            })
            .collect();

        // Determine colon_span based on whether there are bounds
        let colon_span = if !lifetime_param.bounds.is_empty() { Some(Span::DUMMY) } else { None };

        Ok(LifetimeParam { ident, bounds, colon_span })
    }

    fn convert_type_param(&mut self, type_param: &syn::TypeParam) -> Result<TypeParam> {
        let ident = self.convert_ident(&type_param.ident);

        // Convert bounds (e.g., T: Clone + Send)
        let bounds = type_param
            .bounds
            .iter()
            .map(|bound| self.convert_type_param_bound(bound))
            .collect::<Result<Vec<_>>>()?;

        // Convert default type if present (e.g., T = i32)
        let default = if let Some(default) = &type_param.default {
            Some(self.convert_type(default)?)
        } else {
            None
        };

        Ok(TypeParam { ident, bounds, default })
    }

    fn convert_const_param(&mut self, const_param: &syn::ConstParam) -> Result<ConstParam> {
        let ident = self.convert_ident(&const_param.ident);
        let ty = self.convert_type(&const_param.ty)?;

        // Convert default expression if present (e.g., const N: usize = 5)
        let default = if let Some(default) = &const_param.default {
            Some(self.convert_expr(default)?)
        } else {
            None
        };

        Ok(ConstParam { ident, ty, default })
    }

    fn convert_where_clause(
        &mut self,
        where_clause: &Option<syn::WhereClause>,
    ) -> Result<WhereClause> {
        if let Some(where_clause) = where_clause {
            let predicates = where_clause
                .predicates
                .iter()
                .map(|pred| self.convert_where_predicate(pred))
                .collect::<Result<Vec<_>>>()?;

            Ok(WhereClause { has_where_token: true, predicates, span: Span::DUMMY })
        } else {
            Ok(WhereClause::empty())
        }
    }

    fn convert_where_predicate(
        &mut self,
        predicate: &syn::WherePredicate,
    ) -> Result<WherePredicate> {
        match predicate {
            syn::WherePredicate::Type(pred_type) => {
                // Convert bounded type (e.g., T in T: Clone)
                let bounded_ty = self.convert_type(&pred_type.bounded_ty)?;

                // Convert bounds (e.g., Clone + Send)
                let bounds = pred_type
                    .bounds
                    .iter()
                    .map(|bound| self.convert_type_param_bound(bound))
                    .collect::<Result<Vec<_>>>()?;

                // Convert bound lifetimes for HRTB (e.g., for<'a> in for<'a> F: Fn(&'a T))
                let bound_lifetimes = pred_type
                    .lifetimes
                    .as_ref()
                    .map(|lt| {
                        lt.lifetimes
                            .iter()
                            .filter_map(|param| match param {
                                syn::GenericParam::Lifetime(lifetime_param) => {
                                    Some(self.convert_lifetime_param(lifetime_param))
                                }
                                _ => None,
                            })
                            .collect::<Result<Vec<_>>>()
                    })
                    .transpose()?
                    .unwrap_or_default();

                Ok(WherePredicate::BoundPredicate(WhereBoundPredicate {
                    span: Span::DUMMY,
                    bounded_ty,
                    bounds,
                    bound_lifetimes,
                }))
            }
            syn::WherePredicate::Lifetime(pred_lifetime) => {
                // Convert lifetime (e.g., 'a in 'a: 'b)
                let lifetime =
                    Lifetime { ident: self.convert_ident(&pred_lifetime.lifetime.ident) };

                // Convert bounds (e.g., 'b + 'c in 'a: 'b + 'c)
                let bounds = pred_lifetime
                    .bounds
                    .iter()
                    .map(|bound| {
                        let ident = self.convert_ident(&bound.ident);
                        Lifetime { ident }
                    })
                    .collect();

                Ok(WherePredicate::RegionPredicate(WhereRegionPredicate {
                    span: Span::DUMMY,
                    lifetime,
                    bounds,
                }))
            }
            _ => Err(ParseError::Expected {
                expected: "type or lifetime predicate".to_string(),
                found: "unknown where predicate".to_string(),
                pos: Position::new(0, 1, 1),
            }),
        }
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

        // Extract type annotation from Pat::Type if present
        let ty = if let syn::Pat::Type(pat_type) = &local.pat {
            Some(self.convert_type(&pat_type.ty)?)
        } else {
            None
        };

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
            syn::Expr::Call(expr_call) => {
                let func = Box::new(self.convert_expr(&expr_call.func)?);
                let args = expr_call
                    .args
                    .iter()
                    .map(|arg| self.convert_expr(arg))
                    .collect::<Result<Vec<_>>>()?;
                ExprKind::Call { func, args }
            }
            syn::Expr::MethodCall(method_call) => {
                let receiver = Box::new(self.convert_expr(&method_call.receiver)?);
                let method = self.convert_ident(&method_call.method);
                let args = method_call
                    .args
                    .iter()
                    .map(|arg| self.convert_expr(arg))
                    .collect::<Result<Vec<_>>>()?;
                ExprKind::MethodCall { receiver, method, args }
            }
            syn::Expr::If(expr_if) => {
                let cond = Box::new(self.convert_expr(&expr_if.cond)?);
                let then_branch = self.convert_block(&expr_if.then_branch)?;
                let else_branch = if let Some((_, else_expr)) = &expr_if.else_branch {
                    Some(Box::new(self.convert_expr(else_expr)?))
                } else {
                    None
                };
                ExprKind::If { cond, then_branch, else_branch }
            }
            syn::Expr::Binary(expr_binary) => {
                let left = Box::new(self.convert_expr(&expr_binary.left)?);
                let op = self.convert_bin_op(&expr_binary.op);
                let right = Box::new(self.convert_expr(&expr_binary.right)?);
                ExprKind::Binary { left, op, right }
            }
            syn::Expr::Index(expr_index) => {
                let expr = Box::new(self.convert_expr(&expr_index.expr)?);
                let index = Box::new(self.convert_expr(&expr_index.index)?);
                ExprKind::Index { expr, index }
            }
            syn::Expr::Block(expr_block) => {
                // Convert block expression by extracting its final expression
                // For now, we only support blocks that end with an expression
                let block = &expr_block.block;

                // Check if the block has statements
                // If the last statement is an expression without semicolon, use it
                if let Some(syn::Stmt::Expr(expr, None)) = block.stmts.last() {
                    // Recursively convert the expression
                    return self.convert_expr(expr);
                }

                // If the block is empty or doesn't end with an expression,
                // this is not supported yet
                return Err(ParseError::Expected {
                    expected: "block ending with expression".to_string(),
                    found: "block with statements or empty block".to_string(),
                    pos: Position::new(0, 1, 1),
                });
            }
            syn::Expr::Unary(expr_unary) => {
                let op = self.convert_un_op(&expr_unary.op);
                let expr = Box::new(self.convert_expr(&expr_unary.expr)?);
                ExprKind::Unary { op, expr }
            }
            syn::Expr::Reference(expr_ref) => {
                let op = if expr_ref.mutability.is_some() { UnOp::RefMut } else { UnOp::Ref };
                let expr = Box::new(self.convert_expr(&expr_ref.expr)?);
                ExprKind::Unary { op, expr }
            }
            syn::Expr::Field(expr_field) => {
                let expr = Box::new(self.convert_expr(&expr_field.base)?);
                let field = match &expr_field.member {
                    syn::Member::Named(ident) => self.convert_ident(ident),
                    syn::Member::Unnamed(index) => Ident::new(index.index.to_string(), Span::DUMMY),
                };
                ExprKind::Field { expr, field }
            }
            // Phase 9: Struct literals (HIGH PRIORITY)
            syn::Expr::Struct(expr_struct) => {
                let path = self.convert_path(&expr_struct.path)?;

                // Convert fields
                let fields = expr_struct
                    .fields
                    .iter()
                    .map(|field| {
                        let ident = match &field.member {
                            syn::Member::Named(name) => self.convert_ident(name),
                            syn::Member::Unnamed(index) => {
                                Ident::new(index.index.to_string(), Span::DUMMY)
                            }
                        };
                        let expr = self.convert_expr(&field.expr)?;
                        Ok(ExprField {
                            attrs: vec![],
                            ident,
                            expr,
                            is_shorthand: false, // TODO: detect shorthand
                            span: Span::DUMMY,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;

                ExprKind::Struct { path, fields }
            }
            // Phase 9: Array literals
            syn::Expr::Array(expr_array) => {
                let elems =
                    expr_array.elems.iter().map(|e| self.convert_expr(e)).collect::<Result<Vec<_>>>()?;
                ExprKind::Array(elems)
            }
            // Phase 9: Tuple expressions
            syn::Expr::Tuple(expr_tuple) => {
                let elems =
                    expr_tuple.elems.iter().map(|e| self.convert_expr(e)).collect::<Result<Vec<_>>>()?;
                ExprKind::Tuple(elems)
            }
            // Phase 9: Range expressions
            syn::Expr::Range(expr_range) => {
                let start = expr_range
                    .start
                    .as_ref()
                    .map(|e| self.convert_expr(e))
                    .transpose()?
                    .map(Box::new);

                let end =
                    expr_range.end.as_ref().map(|e| self.convert_expr(e)).transpose()?.map(Box::new);

                let inclusive = matches!(expr_range.limits, syn::RangeLimits::Closed(_));

                ExprKind::Range { start, end, inclusive }
            }
            // Phase 9: Assignment
            syn::Expr::Assign(expr_assign) => {
                let left = Box::new(self.convert_expr(&expr_assign.left)?);
                let right = Box::new(self.convert_expr(&expr_assign.right)?);
                ExprKind::Assign { left, right }
            }
            // Phase 9: Parenthesized expressions
            syn::Expr::Paren(expr_paren) => {
                let inner = Box::new(self.convert_expr(&expr_paren.expr)?);
                ExprKind::Paren(inner)
            }
            // Phase 9: Try operator
            syn::Expr::Try(expr_try) => {
                let inner = Box::new(self.convert_expr(&expr_try.expr)?);
                ExprKind::Try(inner)
            }
            // Phase 9: Type cast
            syn::Expr::Cast(expr_cast) => {
                let expr = Box::new(self.convert_expr(&expr_cast.expr)?);
                let ty = Box::new(self.convert_type(&expr_cast.ty)?);
                ExprKind::Cast { expr, ty }
            }
            // Phase 9: Break with optional label and value
            syn::Expr::Break(expr_break) => {
                let label = expr_break.label.as_ref().map(|l| self.convert_label(l));

                let value = expr_break
                    .expr
                    .as_ref()
                    .map(|e| self.convert_expr(e))
                    .transpose()?
                    .map(Box::new);

                ExprKind::Break { label, value }
            }
            // Phase 9: Continue with optional label
            syn::Expr::Continue(expr_continue) => {
                let label = expr_continue.label.as_ref().map(|l| self.convert_label(l));

                ExprKind::Continue { label }
            }
            // Phase 9: Return with optional value
            syn::Expr::Return(expr_return) => {
                let value = expr_return
                    .expr
                    .as_ref()
                    .map(|e| self.convert_expr(e))
                    .transpose()?
                    .map(Box::new);

                ExprKind::Return { value }
            }
            // Phase 10: Closure expressions (HIGHEST PRIORITY)
            syn::Expr::Closure(expr_closure) => {
                // Convert parameters (closure arguments use patterns)
                let params = expr_closure
                    .inputs
                    .iter()
                    .map(|input| {
                        let pat = self.convert_pat(input)?;

                        // Closures often infer types
                        Ok(Param {
                            attrs: vec![],
                            ty: Ty {
                                id: self.next_id(),
                                kind: TyKind::Infer,
                                span: Span::DUMMY,
                                tokens: None,
                            },
                            pat,
                            id: self.next_id(),
                            span: Span::DUMMY,
                            is_placeholder: false,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;

                let body = Box::new(self.convert_expr(&expr_closure.body)?);

                ExprKind::Closure { params, body }
            }
            // Phase 10: Match expressions
            syn::Expr::Match(expr_match) => {
                let expr = Box::new(self.convert_expr(&expr_match.expr)?);

                let arms = expr_match
                    .arms
                    .iter()
                    .map(|arm| {
                        let pat = self.convert_pat(&arm.pat)?;

                        let guard = arm
                            .guard
                            .as_ref()
                            .map(|(_, expr)| self.convert_expr(expr))
                            .transpose()?
                            .map(Box::new);

                        let body = Box::new(self.convert_expr(&arm.body)?);

                        Ok(Arm { attrs: vec![], pat, guard, body, span: Span::DUMMY, id: self.next_id() })
                    })
                    .collect::<Result<Vec<_>>>()?;

                ExprKind::Match { expr, arms }
            }
            // Phase 10: ForLoop expressions
            syn::Expr::ForLoop(expr_for) => {
                let label = expr_for.label.as_ref().map(|l| self.convert_label(&l.name));

                let pat = self.convert_pat(&expr_for.pat)?;
                let iter = Box::new(self.convert_expr(&expr_for.expr)?);
                let body = self.convert_block(&expr_for.body)?;

                ExprKind::ForLoop { label, pat, iter, body }
            }
            // Phase 10: Loop expressions
            syn::Expr::Loop(expr_loop) => {
                let label = expr_loop.label.as_ref().map(|l| self.convert_label(&l.name));

                let body = self.convert_block(&expr_loop.body)?;

                ExprKind::Loop { label, body }
            }
            // Phase 10: While expressions
            syn::Expr::While(expr_while) => {
                let label = expr_while.label.as_ref().map(|l| self.convert_label(&l.name));

                let cond = Box::new(self.convert_expr(&expr_while.cond)?);
                let body = self.convert_block(&expr_while.body)?;

                ExprKind::While { label, cond, body }
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

    fn convert_label(&mut self, lifetime: &syn::Lifetime) -> Label {
        Label { ident: self.convert_ident(&lifetime.ident) }
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

    fn convert_attributes(&mut self, attrs: &[syn::Attribute]) -> Result<Vec<Attribute>> {
        attrs.iter().map(|attr| self.convert_attribute(attr)).collect()
    }

    fn convert_attribute(&mut self, attr: &syn::Attribute) -> Result<Attribute> {
        let style = match attr.style {
            syn::AttrStyle::Outer => AttrStyle::Outer,
            syn::AttrStyle::Inner(_) => AttrStyle::Inner,
        };

        let kind = if let Some(doc) = self.extract_doc_comment(attr) {
            AttrKind::DocComment(CommentKind::Line, doc)
        } else {
            let path = self.convert_path(attr.path())?;
            let args = self.convert_attr_args(&attr.meta)?;

            let item = AttrItem { path, args };
            let normal = NormalAttr { item };
            AttrKind::Normal(normal)
        };

        Ok(Attribute { kind, id: 0, style, span: Span::DUMMY })
    }

    fn extract_doc_comment(&self, attr: &syn::Attribute) -> Option<String> {
        // Check if this is a doc comment attribute
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &meta.value {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        return Some(lit_str.value());
                    }
                }
            }
        }
        None
    }

    fn convert_attr_args(&mut self, meta: &syn::Meta) -> Result<MacArgs> {
        match meta {
            syn::Meta::Path(_) => Ok(MacArgs::Empty),

            syn::Meta::List(meta_list) => {
                // Convert tokens to our TokenStream
                let tokens = TokenStream::Source(meta_list.tokens.to_string());
                let delim_span = DelSpan::new(Span::DUMMY, Span::DUMMY);
                Ok(MacArgs::Delimited { dspan: delim_span, delim: Delimiter::Paren, tokens })
            }

            syn::Meta::NameValue(meta_nv) => {
                // For #[key = "value"] style
                let value = match &meta_nv.value {
                    syn::Expr::Lit(expr_lit) => {
                        format!("{:?}", expr_lit.lit)
                    }
                    other => format!("{:?}", other),
                };
                let tokens = TokenStream::Source(value);
                Ok(MacArgs::Eq { eq_span: Span::DUMMY, tokens })
            }
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
            // Phase 1: String and integer literals
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
            // Phase 11: Boolean literals (VERY COMMON)
            syn::Lit::Bool(lit_bool) => LitKind::Bool(lit_bool.value),
            // Phase 11: Floating point literals
            syn::Lit::Float(lit_float) => LitKind::Float(lit_float.base10_digits().to_string()),
            // Phase 11: Character literals
            syn::Lit::Char(lit_char) => LitKind::Char(lit_char.value()),
            // Phase 11: Byte literals
            syn::Lit::Byte(lit_byte) => LitKind::Byte(lit_byte.value()),
            // Phase 11: Byte string literals
            syn::Lit::ByteStr(lit_byte_str) => LitKind::ByteStr(lit_byte_str.value().to_vec()),
            // Phase 11: C string literals
            syn::Lit::CStr(lit_cstr) => LitKind::CStr(lit_cstr.value().into_bytes()),
            // Phase 11: Verbatim (unknown/error literals)
            syn::Lit::Verbatim(lit_verbatim) => LitKind::Verbatim(lit_verbatim.to_string()),
            // Catch-all for future syn::Lit variants (syn::Lit is non-exhaustive)
            _ => LitKind::Verbatim(format!("{:?}", lit)),
        };

        Ok(Lit { kind, span: Span::DUMMY })
    }

    fn convert_bin_op(&mut self, op: &syn::BinOp) -> BinOp {
        match op {
            syn::BinOp::Add(_) => BinOp::Add,
            syn::BinOp::Sub(_) => BinOp::Sub,
            syn::BinOp::Mul(_) => BinOp::Mul,
            syn::BinOp::Div(_) => BinOp::Div,
            syn::BinOp::Rem(_) => BinOp::Rem,
            syn::BinOp::And(_) => BinOp::And,
            syn::BinOp::Or(_) => BinOp::Or,
            syn::BinOp::BitXor(_) => BinOp::BitXor,
            syn::BinOp::BitAnd(_) => BinOp::BitAnd,
            syn::BinOp::BitOr(_) => BinOp::BitOr,
            syn::BinOp::Shl(_) => BinOp::Shl,
            syn::BinOp::Shr(_) => BinOp::Shr,
            syn::BinOp::Eq(_) => BinOp::Eq,
            syn::BinOp::Lt(_) => BinOp::Lt,
            syn::BinOp::Le(_) => BinOp::Le,
            syn::BinOp::Ne(_) => BinOp::Ne,
            syn::BinOp::Ge(_) => BinOp::Ge,
            syn::BinOp::Gt(_) => BinOp::Gt,
            syn::BinOp::AddAssign(_) => BinOp::Add, // Simplification
            syn::BinOp::SubAssign(_) => BinOp::Sub,
            syn::BinOp::MulAssign(_) => BinOp::Mul,
            syn::BinOp::DivAssign(_) => BinOp::Div,
            syn::BinOp::RemAssign(_) => BinOp::Rem,
            syn::BinOp::BitXorAssign(_) => BinOp::BitXor,
            syn::BinOp::BitAndAssign(_) => BinOp::BitAnd,
            syn::BinOp::BitOrAssign(_) => BinOp::BitOr,
            syn::BinOp::ShlAssign(_) => BinOp::Shl,
            syn::BinOp::ShrAssign(_) => BinOp::Shr,
            _ => BinOp::Add, // Default fallback
        }
    }

    fn convert_un_op(&mut self, op: &syn::UnOp) -> UnOp {
        match op {
            syn::UnOp::Deref(_) => UnOp::Deref,
            syn::UnOp::Not(_) => UnOp::Not,
            syn::UnOp::Neg(_) => UnOp::Neg,
            _ => UnOp::Deref, // Default fallback
        }
    }

    fn convert_item_struct(&mut self, item_struct: &syn::ItemStruct) -> Result<Item> {
        let attrs = self.convert_attributes(&item_struct.attrs)?;
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
            attrs,
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

        Ok(FieldDef { attrs: vec![], id: self.next_id(), span: Span::DUMMY, vis, ident, ty })
    }

    fn convert_item_enum(&mut self, item_enum: &syn::ItemEnum) -> Result<Item> {
        let attrs = self.convert_attributes(&item_enum.attrs)?;
        let ident = self.convert_ident(&item_enum.ident);
        let vis = self.convert_visibility(&item_enum.vis);

        // Convert all variants
        let variants = item_enum
            .variants
            .iter()
            .map(|v| self.convert_variant(v))
            .collect::<Result<Vec<_>>>()?;

        let enum_def = EnumDef { variants };

        Ok(Item {
            attrs,
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
        let attrs = self.convert_attributes(&item_impl.attrs)?;
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
            attrs,
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

                let fn_def = Fn { defaultness: Defaultness::Final, sig, generics, body };

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
        let attrs = self.convert_attributes(&item_use.attrs)?;

        // Convert the use tree
        let use_tree = self.convert_use_tree(&item_use.tree)?;

        Ok(Item {
            attrs,
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
                    prefix: Path { span: Span::DUMMY, segments, tokens: None },
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
                    prefix: Path { span: Span::DUMMY, segments: vec![segment], tokens: None },
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
                    prefix: Path { span: Span::DUMMY, segments: vec![segment], tokens: None },
                    kind: UseTreeKind::Simple(Some(rename)),
                })
            }
            syn::UseTree::Glob(_) => {
                // Glob: `use foo::*;`
                Ok(UseTree {
                    prefix: Path { span: Span::DUMMY, segments: vec![], tokens: None },
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
                    prefix: Path { span: Span::DUMMY, segments: vec![], tokens: None },
                    kind: UseTreeKind::Nested(items),
                })
            }
        }
    }

    fn convert_item_static(&mut self, item_static: &syn::ItemStatic) -> Result<Item> {
        let ident = self.convert_ident(&item_static.ident);
        let vis = self.convert_visibility(&item_static.vis);
        let attrs = self.convert_attributes(&item_static.attrs)?;

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
            attrs,
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
        let attrs = self.convert_attributes(&item_const.attrs)?;

        // Convert type
        let ty = self.convert_type(&item_const.ty)?;

        // Convert initializer expression
        let expr = Some(self.convert_expr(&item_const.expr)?);

        Ok(Item {
            attrs,
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
        let attrs = self.convert_attributes(&item_type.attrs)?;

        // Convert generics
        let generics = self.convert_generics(&item_type.generics)?;

        // Convert the aliased type
        let ty = Some(self.convert_type(&item_type.ty)?);

        Ok(Item {
            attrs,
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
        let attrs = self.convert_attributes(&item_mod.attrs)?;

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
            attrs,
            id: self.next_id(),
            span: Span::DUMMY,
            vis,
            ident,
            kind: ItemKind::Mod { items },
            tokens: None,
        })
    }

    fn convert_item_macro(&mut self, item_macro: &syn::ItemMacro) -> Result<Item> {
        let attrs = self.convert_attributes(&item_macro.attrs)?;

        // Get the macro name - for macro_rules!, this comes from the ident field
        // For the newer `macro` syntax, it also uses ident
        let ident = if let Some(ref mac_ident) = item_macro.ident {
            self.convert_ident(mac_ident)
        } else {
            // Fallback: use the path as the name
            let path_str = item_macro
                .mac
                .path
                .segments
                .last()
                .map(|seg| seg.ident.to_string())
                .unwrap_or_else(|| "unnamed_macro".to_string());
            Ident::new(path_str, Span::DUMMY)
        };

        // Convert the macro body - convert tokens to string
        let tokens_str = item_macro.mac.tokens.to_string();
        let body = MacArgs::Delimited {
            dspan: DelSpan::new(Span::DUMMY, Span::DUMMY),
            delim: Delimiter::Brace,
            tokens: TokenStream::Source(tokens_str),
        };

        // Check if this is a macro_rules! definition
        let macro_rules = item_macro.mac.path.is_ident("macro_rules");

        let macro_def = MacroDef { macro_rules, body };

        Ok(Item {
            attrs,
            id: self.next_id(),
            span: Span::DUMMY,
            vis: Visibility::Inherited, // Macros don't have visibility in the same way
            ident,
            kind: ItemKind::MacroDef(Box::new(macro_def)),
            tokens: None,
        })
    }

    fn convert_type_param_bound(&mut self, bound: &syn::TypeParamBound) -> Result<GenericBound> {
        match bound {
            syn::TypeParamBound::Trait(trait_bound) => {
                let path = self.convert_path(&trait_bound.path)?;
                let trait_ref = TraitRef { path };
                let poly_trait_ref = PolyTraitRef {
                    trait_ref,
                    bound_lifetimes: vec![], // TODO: implement bound lifetimes
                };
                // Determine modifier from trait_bound.modifier
                let modifier = match trait_bound.modifier {
                    syn::TraitBoundModifier::None => TraitBoundModifier::None,
                    syn::TraitBoundModifier::Maybe(_) => TraitBoundModifier::Maybe,
                };
                Ok(GenericBound::Trait(poly_trait_ref, modifier))
            }
            syn::TypeParamBound::Lifetime(lifetime) => {
                let ident = self.convert_ident(&lifetime.ident);
                Ok(GenericBound::Outlives(Lifetime { ident }))
            }
            _ => Err(ParseError::Expected {
                expected: "trait bound".to_string(),
                found: "unknown bound type".to_string(),
                pos: Position::new(0, 1, 1),
            }),
        }
    }

    fn convert_item_trait(&mut self, item_trait: &syn::ItemTrait) -> Result<Item> {
        let attrs = self.convert_attributes(&item_trait.attrs)?;
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
            attrs,
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

                let fn_def = Fn { defaultness: Defaultness::Final, sig, generics, body };

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
