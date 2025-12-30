use crate::ast::*;
use crate::error::Result;
use crate::generator::gen::Generator;
use crate::generator::helpers::*;
use crate::sexp::SExp;

impl Generator {
    pub fn generate_item(&self, item: &Item) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("attrs", self.generate_attr_vec(&item.attrs)?),
            kwarg("id", self.generate_node_id(item.id)),
            kwarg("span", self.generate_span(item.span)),
            kwarg("vis", self.generate_visibility(&item.vis)),
            kwarg("ident", self.generate_ident(&item.ident)),
            kwarg("kind", self.generate_item_kind(&item.kind)?),
        ]);

        // Only include tokens if present
        let fields = if item.tokens.is_some() {
            let mut f = fields;
            f.extend(kwarg("tokens", sym("nil")));
            f
        } else {
            fields
        };

        Ok(typed_node("Item", fields))
    }

    pub fn generate_visibility(&self, vis: &Visibility) -> SExp {
        match vis {
            Visibility::Public => list(vec![sym("Public")]),
            Visibility::Inherited => list(vec![sym("Inherited")]),
            Visibility::Restricted { path, shorthand, span } => {
                let fields = kwargs(vec![
                    kwarg("path", self.generate_path(path)),
                    kwarg("shorthand", self.generate_vis_restriction_kind(*shorthand)),
                    kwarg("span", self.generate_span(*span)),
                ]);
                typed_node("Restricted", fields)
            }
        }
    }

    pub fn generate_ident(&self, ident: &Ident) -> SExp {
        let fields = kwargs(vec![
            kwarg("name", string(&ident.name)),
            kwarg("span", self.generate_span(ident.span)),
        ]);

        typed_node("Ident", fields)
    }

    fn generate_item_kind(&self, kind: &ItemKind) -> Result<SExp> {
        match kind {
            ItemKind::Fn(func) => Ok(list(vec![sym("Fn"), self.generate_fn(func)?])),
            ItemKind::Struct(data) => {
                Ok(list(vec![sym("Struct"), self.generate_variant_data(data)]))
            }
            ItemKind::Enum(enum_def) => {
                Ok(list(vec![sym("Enum"), self.generate_enum_def(enum_def)]))
            }
        }
    }

    fn generate_fn(&self, func: &Fn) -> Result<SExp> {
        let mut fields = kwargs(vec![
            kwarg("defaultness", self.generate_defaultness(func.defaultness)),
            kwarg("sig", self.generate_fn_sig(&func.sig)?),
            kwarg("generics", self.generate_generics(&func.generics)?),
        ]);

        // Body is optional
        if let Some(body) = &func.body {
            fields.extend(kwarg("body", self.generate_block(body)?));
        } else {
            fields.extend(kwarg("body", sym("nil")));
        }

        Ok(typed_node("Fn", fields))
    }

    fn generate_defaultness(&self, defaultness: Defaultness) -> SExp {
        match defaultness {
            Defaultness::Default => sym("Default"),
            Defaultness::Final => sym("Final"),
        }
    }

    fn generate_fn_sig(&self, sig: &FnSig) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("header", self.generate_fn_header(&sig.header)?),
            kwarg("decl", self.generate_fn_decl(&sig.decl)?),
            kwarg("span", self.generate_span(sig.span)),
        ]);

        Ok(typed_node("FnSig", fields))
    }

    fn generate_fn_header(&self, header: &FnHeader) -> Result<SExp> {
        let mut fields = kwargs(vec![
            kwarg("safety", self.generate_safety(header.safety)),
            kwarg("constness", self.generate_constness(header.constness)),
            kwarg("ext", self.generate_extern(&header.ext)),
        ]);

        // coroutine_kind is optional
        if let Some(kind) = header.coroutine_kind {
            fields.extend(kwarg("coroutine-kind", self.generate_coroutine_kind(kind)));
        } else {
            fields.extend(kwarg("coroutine-kind", sym("nil")));
        }

        Ok(typed_node("FnHeader", fields))
    }

    fn generate_safety(&self, safety: Safety) -> SExp {
        match safety {
            Safety::Safe => sym("Safe"),
            Safety::Unsafe => sym("Unsafe"),
            Safety::Default => sym("Default"),
        }
    }

    fn generate_constness(&self, constness: Constness) -> SExp {
        match constness {
            Constness::Const => sym("Const"),
            Constness::NotConst => sym("NotConst"),
        }
    }

    fn generate_extern(&self, ext: &Extern) -> SExp {
        match ext {
            Extern::None => sym("None"),
            Extern::Explicit(abi) => list(vec![sym("Explicit"), string(abi)]),
        }
    }

    fn generate_coroutine_kind(&self, kind: CoroutineKind) -> SExp {
        match kind {
            CoroutineKind::Async => sym("Async"),
            CoroutineKind::Gen => sym("Gen"),
        }
    }

    fn generate_fn_decl(&self, decl: &FnDecl) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("inputs", self.generate_params(&decl.inputs)?),
            kwarg("output", self.generate_fn_ret_ty(&decl.output)?),
        ]);

        Ok(typed_node("FnDecl", fields))
    }

    fn generate_params(&self, params: &[Param]) -> Result<SExp> {
        let param_sexps: Result<Vec<SExp>> =
            params.iter().map(|param| self.generate_param(param)).collect();
        Ok(list(param_sexps?))
    }

    fn generate_param(&self, param: &Param) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("attrs", self.generate_attr_vec(&param.attrs)?),
            kwarg("ty", self.generate_ty(&param.ty)?),
            kwarg("pat", self.generate_pat(&param.pat)?),
            kwarg("id", self.generate_node_id(param.id)),
            kwarg("span", self.generate_span(param.span)),
            kwarg("is-placeholder", sym(if param.is_placeholder { "true" } else { "false" })),
        ]);

        Ok(typed_node("Param", fields))
    }

    fn generate_fn_ret_ty(&self, ret_ty: &FnRetTy) -> Result<SExp> {
        match ret_ty {
            FnRetTy::Default(span) => Ok(list(vec![sym("Default"), self.generate_span(*span)])),
            FnRetTy::Ty(ty) => Ok(list(vec![sym("Ty"), self.generate_ty(ty)?])),
        }
    }

    fn generate_generics(&self, generics: &Generics) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("params", self.generate_generic_params(&generics.params)?),
            kwarg("where-clause", self.generate_where_clause(&generics.where_clause)?),
            kwarg("span", self.generate_span(generics.span)),
        ]);

        Ok(typed_node("Generics", fields))
    }

    fn generate_generic_params(&self, _params: &[GenericParam]) -> Result<SExp> {
        // Phase 2: Just empty list for now
        Ok(empty_list())
    }

    fn generate_where_clause(&self, clause: &WhereClause) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("has-where-token", sym(if clause.has_where_token { "true" } else { "false" })),
            kwarg("predicates", self.generate_where_predicates(&clause.predicates)?),
            kwarg("span", self.generate_span(clause.span)),
        ]);

        Ok(typed_node("WhereClause", fields))
    }

    fn generate_where_predicates(&self, _predicates: &[WherePredicate]) -> Result<SExp> {
        // Phase 2: Just empty list for now
        Ok(empty_list())
    }

    pub(crate) fn generate_ty(&self, ty: &Ty) -> Result<SExp> {
        let mut fields = kwargs(vec![
            kwarg("id", self.generate_node_id(ty.id)),
            kwarg("kind", self.generate_ty_kind(&ty.kind)?),
            kwarg("span", self.generate_span(ty.span)),
        ]);

        // Only include tokens if present
        if ty.tokens.is_some() {
            fields.extend(kwarg("tokens", sym("nil")));
        }

        Ok(typed_node("Ty", fields))
    }

    fn generate_ty_kind(&self, kind: &TyKind) -> Result<SExp> {
        match kind {
            TyKind::Path(qself, path) => {
                let qself_sexp = if qself.is_some() {
                    sym("TODO") // TODO: implement QSelf
                } else {
                    sym("nil")
                };
                Ok(list(vec![sym("Path"), qself_sexp, self.generate_path(path)]))
            }
        }
    }

    pub(crate) fn generate_pat(&self, pat: &Pat) -> Result<SExp> {
        let mut fields = kwargs(vec![
            kwarg("id", self.generate_node_id(pat.id)),
            kwarg("kind", self.generate_pat_kind(&pat.kind)?),
            kwarg("span", self.generate_span(pat.span)),
        ]);

        // Only include tokens if present
        if pat.tokens.is_some() {
            fields.extend(kwarg("tokens", sym("nil")));
        }

        Ok(typed_node("Pat", fields))
    }

    fn generate_pat_kind(&self, kind: &PatKind) -> Result<SExp> {
        match kind {
            PatKind::Ident { binding_mode, ident, sub } => {
                let mut fields = kwargs(vec![
                    kwarg("binding-mode", self.generate_binding_mode(*binding_mode)),
                    kwarg("ident", self.generate_ident(ident)),
                ]);

                // sub is optional
                if let Some(sub_pat) = sub {
                    fields.extend(kwarg("sub", self.generate_pat(sub_pat)?));
                } else {
                    fields.extend(kwarg("sub", sym("nil")));
                }

                Ok(typed_node("Ident", fields))
            }
        }
    }

    fn generate_binding_mode(&self, mode: BindingMode) -> SExp {
        match mode {
            BindingMode::ByRef(mutability) => {
                list(vec![sym("ByRef"), self.generate_mutability(mutability)])
            }
            BindingMode::ByValue(mutability) => {
                list(vec![sym("ByValue"), self.generate_mutability(mutability)])
            }
        }
    }

    fn generate_mutability(&self, mutability: Mutability) -> SExp {
        match mutability {
            Mutability::Mut => sym("Mut"),
            Mutability::Not => sym("Not"),
        }
    }

    fn generate_vis_restriction_kind(&self, kind: VisRestrictionKind) -> SExp {
        match kind {
            VisRestrictionKind::Crate => sym("Crate"),
            VisRestrictionKind::Super => sym("Super"),
            VisRestrictionKind::In => sym("In"),
        }
    }

    pub fn generate_path(&self, path: &Path) -> SExp {
        let mut fields = kwargs(vec![
            kwarg("span", self.generate_span(path.span)),
            kwarg("segments", self.generate_path_segments(&path.segments)),
        ]);

        // Only include tokens if present
        if path.tokens.is_some() {
            fields.extend(kwarg("tokens", sym("nil")));
        }

        typed_node("Path", fields)
    }

    fn generate_path_segments(&self, segments: &[PathSegment]) -> SExp {
        let segment_sexps: Vec<SExp> =
            segments.iter().map(|seg| self.generate_path_segment(seg)).collect();
        list(segment_sexps)
    }

    fn generate_path_segment(&self, segment: &PathSegment) -> SExp {
        let mut fields = kwargs(vec![
            kwarg("ident", self.generate_ident(&segment.ident)),
            kwarg("id", self.generate_node_id(segment.id)),
        ]);

        // args is optional
        if segment.args.is_some() {
            fields.extend(kwarg("args", sym("TODO"))); // TODO: implement GenericArgs
        } else {
            fields.extend(kwarg("args", sym("nil")));
        }

        typed_node("PathSegment", fields)
    }

    fn generate_variant_data(&self, data: &VariantData) -> SExp {
        match data {
            VariantData::Struct { fields, recovered } => {
                let fields_sexp = list(fields.iter().map(|f| self.generate_field_def(f)).collect());
                typed_node(
                    "Struct",
                    kwargs(vec![
                        kwarg("fields", fields_sexp),
                        kwarg("recovered", sym(if *recovered { "true" } else { "false" })),
                    ]),
                )
            }
            VariantData::Tuple(fields) => {
                let fields_sexp = list(fields.iter().map(|f| self.generate_field_def(f)).collect());
                list(vec![sym("Tuple"), fields_sexp])
            }
            VariantData::Unit => sym("Unit"),
        }
    }

    fn generate_enum_def(&self, enum_def: &EnumDef) -> SExp {
        let variants_sexp =
            list(enum_def.variants.iter().map(|v| self.generate_variant(v)).collect::<Vec<_>>());
        typed_node("EnumDef", kwargs(vec![kwarg("variants", variants_sexp)]))
    }

    fn generate_variant(&self, variant: &Variant) -> SExp {
        let mut fields = kwargs(vec![
            kwarg("attrs", self.generate_attr_vec(&variant.attrs).unwrap_or_else(|_| sym("ERROR"))),
            kwarg("id", self.generate_node_id(variant.id)),
            kwarg("span", self.generate_span(variant.span)),
            kwarg("vis", self.generate_visibility(&variant.vis)),
            kwarg("ident", self.generate_ident(&variant.ident)),
            kwarg("data", self.generate_variant_data(&variant.data)),
        ]);

        // Optional discriminant expression
        if let Some(ref expr) = variant.disr_expr {
            fields.extend(kwarg(
                "disr-expr",
                self.generate_expr(expr).unwrap_or_else(|_| sym("ERROR")),
            ));
        } else {
            fields.extend(kwarg("disr-expr", sym("nil")));
        }

        typed_node("Variant", fields)
    }

    fn generate_field_def(&self, field: &FieldDef) -> SExp {
        let mut fields = kwargs(vec![
            kwarg("attrs", self.generate_attr_vec(&field.attrs).unwrap_or_else(|_| sym("ERROR"))),
            kwarg("id", self.generate_node_id(field.id)),
            kwarg("span", self.generate_span(field.span)),
            kwarg("vis", self.generate_visibility(&field.vis)),
        ]);

        // Optional ident (None for tuple fields)
        if let Some(ref ident) = field.ident {
            fields.extend(kwarg("ident", self.generate_ident(ident)));
        } else {
            fields.extend(kwarg("ident", sym("nil")));
        }

        fields.extend(kwarg("ty", self.generate_ty(&field.ty).unwrap_or_else(|_| sym("ERROR"))));

        typed_node("FieldDef", fields)
    }
}
