use super::build::AstBuilder;
use super::helpers::*;
use crate::ast::*;
use crate::error::{ParseError, Result};
use crate::sexp::SExp;

impl AstBuilder {
    pub fn build_item(&mut self, sexp: &SExp) -> Result<Item> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "Item" {
            return Err(ParseError::Expected {
                expected: "Item".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let vis = if let Some(vis_sexp) = kwargs.get("vis") {
            self.build_visibility(vis_sexp)?
        } else {
            Visibility::Inherited
        };

        let ident = self.build_ident(require_field(&kwargs, "ident", list.pos)?)?;

        let kind = self.build_item_kind(require_field(&kwargs, "kind", list.pos)?)?;

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

        Ok(Item { attrs: Vec::new(), id, span, vis, ident, kind, tokens: None })
    }

    pub fn build_visibility(&mut self, sexp: &SExp) -> Result<Visibility> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        match node_type.value.as_str() {
            "Public" => Ok(Visibility::Public),
            "Inherited" => Ok(Visibility::Inherited),
            _ => Err(ParseError::Expected {
                expected: "Public or Inherited".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            }),
        }
    }

    pub fn build_ident(&mut self, sexp: &SExp) -> Result<Ident> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "Ident" {
            return Err(ParseError::Expected {
                expected: "Ident".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let name = expect_string(require_field(&kwargs, "name", list.pos)?)?;

        let span = if let Some(span_sexp) = kwargs.get("span") {
            self.build_span(span_sexp)?
        } else {
            Span::DUMMY
        };

        Ok(Ident::new(name, span))
    }

    pub fn build_item_kind(&mut self, sexp: &SExp) -> Result<ItemKind> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        match node_type.value.as_str() {
            "Fn" => {
                // Extract the inner Fn node from element 1
                let fn_sexp = &list.elements[1];
                let fn_list = expect_list(fn_sexp)?;
                let fn_item = self.build_fn(fn_list)?;
                Ok(ItemKind::Fn(Box::new(fn_item)))
            }
            "Struct" => {
                // Extract the VariantData from element 1
                if list.elements.len() < 2 {
                    return Err(ParseError::Expected {
                        expected: "VariantData after Struct".to_string(),
                        found: "nothing".to_string(),
                        pos: list.pos,
                    });
                }
                let data_sexp = &list.elements[1];
                let variant_data = self.build_variant_data(data_sexp)?;
                Ok(ItemKind::Struct(variant_data))
            }
            "Enum" => {
                // Extract the EnumDef from element 1
                if list.elements.len() < 2 {
                    return Err(ParseError::Expected {
                        expected: "EnumDef after Enum".to_string(),
                        found: "nothing".to_string(),
                        pos: list.pos,
                    });
                }
                let enum_sexp = &list.elements[1];
                let enum_def = self.build_enum_def(enum_sexp)?;
                Ok(ItemKind::Enum(enum_def))
            }
            "Trait" => {
                // Extract the TraitDef from element 1
                if list.elements.len() < 2 {
                    return Err(ParseError::Expected {
                        expected: "TraitDef after Trait".to_string(),
                        found: "nothing".to_string(),
                        pos: list.pos,
                    });
                }
                let trait_sexp = &list.elements[1];
                let trait_def = self.build_trait_def(trait_sexp)?;
                Ok(ItemKind::Trait(Box::new(trait_def)))
            }
            "Impl" => {
                // Extract the ImplDef from element 1
                if list.elements.len() < 2 {
                    return Err(ParseError::Expected {
                        expected: "ImplDef after Impl".to_string(),
                        found: "nothing".to_string(),
                        pos: list.pos,
                    });
                }
                let impl_sexp = &list.elements[1];
                let impl_def = self.build_impl_def(impl_sexp)?;
                Ok(ItemKind::Impl(Box::new(impl_def)))
            }
            // Stage 8: Remaining items
            "Use" => {
                // Extract the UseTree from element 1
                if list.elements.len() < 2 {
                    return Err(ParseError::Expected {
                        expected: "UseTree after Use".to_string(),
                        found: "nothing".to_string(),
                        pos: list.pos,
                    });
                }
                let use_tree_sexp = &list.elements[1];
                let use_tree = self.build_use_tree(use_tree_sexp)?;
                Ok(ItemKind::Use(use_tree))
            }
            "Static" => {
                let kwargs = parse_kwargs(list)?;
                let mutability = if let Some(mut_sexp) = kwargs.get("mutability") {
                    self.build_mutability(mut_sexp)?
                } else {
                    Mutability::Not
                };
                let ty = self.build_ty(require_field(&kwargs, "ty", list.pos)?)?;
                let expr = if let Some(expr_sexp) = optional_field(&kwargs, "expr") {
                    Some(self.build_expr(expr_sexp)?)
                } else {
                    None
                };
                Ok(ItemKind::Static { mutability, ty, expr })
            }
            "Const" => {
                let kwargs = parse_kwargs(list)?;
                let ty = self.build_ty(require_field(&kwargs, "ty", list.pos)?)?;
                let expr = if let Some(expr_sexp) = optional_field(&kwargs, "expr") {
                    Some(self.build_expr(expr_sexp)?)
                } else {
                    None
                };
                Ok(ItemKind::Const { ty, expr })
            }
            "TyAlias" => {
                let kwargs = parse_kwargs(list)?;
                let generics = if let Some(gen_sexp) = kwargs.get("generics") {
                    self.build_generics(gen_sexp)?
                } else {
                    Generics::empty()
                };
                let ty = if let Some(ty_sexp) = optional_field(&kwargs, "ty") {
                    Some(self.build_ty(ty_sexp)?)
                } else {
                    None
                };
                Ok(ItemKind::TyAlias { generics, ty })
            }
            "Mod" => {
                let kwargs = parse_kwargs(list)?;
                let items = if let Some(items_sexp) = optional_field(&kwargs, "items") {
                    let items_list = expect_list(items_sexp)?;
                    let item_vec: Result<Vec<Item>> =
                        items_list.elements.iter().map(|sexp| self.build_item(sexp)).collect();
                    Some(item_vec?)
                } else {
                    None
                };
                Ok(ItemKind::Mod { items })
            }
            _ => Err(ParseError::Expected {
                expected: "Fn, Struct, Enum, Trait, Impl, Use, Static, Const, TyAlias, or Mod"
                    .to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            }),
        }
    }

    fn build_fn(&mut self, list: &crate::sexp::List) -> Result<Fn> {
        let kwargs = parse_kwargs(list)?;

        let defaultness = if let Some(default_sexp) = kwargs.get("defaultness") {
            let sym = expect_symbol(default_sexp)?;
            match sym.value.as_str() {
                "Final" => Defaultness::Final,
                "Default" => Defaultness::Default,
                _ => Defaultness::Final,
            }
        } else {
            Defaultness::Final
        };

        let sig = self.build_fn_sig(require_field(&kwargs, "sig", list.pos)?)?;

        let generics = if let Some(gen_sexp) = kwargs.get("generics") {
            self.build_generics(gen_sexp)?
        } else {
            Generics::empty()
        };

        let body = if let Some(body_sexp) = optional_field(&kwargs, "body") {
            Some(self.build_block(body_sexp)?)
        } else {
            None
        };

        Ok(Fn { defaultness, sig, generics, body })
    }

    fn build_fn_sig(&mut self, sexp: &SExp) -> Result<FnSig> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "FnSig" {
            return Err(ParseError::Expected {
                expected: "FnSig".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let header = if let Some(header_sexp) = kwargs.get("header") {
            self.build_fn_header(header_sexp)?
        } else {
            FnHeader {
                safety: Safety::Default,
                coroutine_kind: None,
                constness: Constness::NotConst,
                ext: Extern::None,
            }
        };

        let decl = if let Some(decl_sexp) = kwargs.get("decl") {
            self.build_fn_decl(decl_sexp)?
        } else {
            FnDecl { inputs: Vec::new(), output: FnRetTy::Default(Span::DUMMY) }
        };

        let span = if let Some(span_sexp) = kwargs.get("span") {
            self.build_span(span_sexp)?
        } else {
            Span::DUMMY
        };

        Ok(FnSig { header, decl, span })
    }

    fn build_fn_header(&mut self, sexp: &SExp) -> Result<FnHeader> {
        let list = expect_list(sexp)?;
        let kwargs = parse_kwargs(list)?;

        let safety = if let Some(safety_sexp) = kwargs.get("safety") {
            let sym = expect_symbol(safety_sexp)?;
            match sym.value.as_str() {
                "Safe" => Safety::Safe,
                "Unsafe" => Safety::Unsafe,
                "Default" => Safety::Default,
                _ => Safety::Default,
            }
        } else {
            Safety::Default
        };

        let constness = if let Some(const_sexp) = kwargs.get("constness") {
            let sym = expect_symbol(const_sexp)?;
            match sym.value.as_str() {
                "Const" => Constness::Const,
                "NotConst" => Constness::NotConst,
                _ => Constness::NotConst,
            }
        } else {
            Constness::NotConst
        };

        Ok(FnHeader { safety, coroutine_kind: None, constness, ext: Extern::None })
    }

    fn build_fn_decl(&mut self, sexp: &SExp) -> Result<FnDecl> {
        let list = expect_list(sexp)?;
        let kwargs = parse_kwargs(list)?;

        let inputs = if let Some(inputs_sexp) = kwargs.get("inputs") {
            let inputs_list = expect_list(inputs_sexp)?;
            let mut params = Vec::new();
            for param_sexp in &inputs_list.elements {
                params.push(self.build_param(param_sexp)?);
            }
            params
        } else {
            Vec::new()
        };

        let output = if let Some(output_sexp) = kwargs.get("output") {
            self.build_fn_ret_ty(output_sexp)?
        } else {
            FnRetTy::Default(Span::DUMMY)
        };

        Ok(FnDecl { inputs, output })
    }

    pub(crate) fn build_param(&mut self, sexp: &SExp) -> Result<Param> {
        let list = expect_list(sexp)?;
        let _kwargs = parse_kwargs(list)?; // For future use

        // For Phase 1, simplified parameter parsing
        let ty = Ty {
            id: self.next_id(),
            kind: TyKind::Path(None, Path::from_ident(Ident::new("i32", Span::DUMMY))),
            span: Span::DUMMY,
            tokens: None,
        };

        let pat = Pat {
            id: self.next_id(),
            kind: PatKind::Ident {
                binding_mode: BindingMode::ByValue(Mutability::Not),
                ident: Ident::new("param", Span::DUMMY),
                sub: None,
            },
            span: Span::DUMMY,
            tokens: None,
        };

        Ok(Param {
            attrs: Vec::new(),
            ty,
            pat,
            id: self.next_id(),
            span: Span::DUMMY,
            is_placeholder: false,
        })
    }

    fn build_fn_ret_ty(&mut self, sexp: &SExp) -> Result<FnRetTy> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        match node_type.value.as_str() {
            "Default" => Ok(FnRetTy::Default(Span::DUMMY)),
            "Ty" => {
                // Parse type - simplified for Phase 1
                Ok(FnRetTy::Default(Span::DUMMY))
            }
            _ => Err(ParseError::Expected {
                expected: "Default or Ty".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            }),
        }
    }

    fn build_generics(&mut self, sexp: &SExp) -> Result<Generics> {
        // Simplified for Phase 1 - no generics support
        let _list = expect_list(sexp)?; // Validate it's a list
        Ok(Generics::empty())
    }

    fn build_ty(&mut self, sexp: &SExp) -> Result<Ty> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "Ty" {
            return Err(ParseError::Expected {
                expected: "Ty".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let kind = self.build_ty_kind(require_field(&kwargs, "kind", list.pos)?)?;

        let id = if let Some(id_sexp) = kwargs.get("id") {
            NodeId(expect_number(id_sexp)? as u32)
        } else {
            self.next_id()
        };

        let span = if let Some(span_sexp) = kwargs.get("span") {
            self.build_span(span_sexp)?
        } else {
            Span::DUMMY
        };

        Ok(Ty { id, kind, span, tokens: None })
    }

    fn build_ty_kind(&mut self, sexp: &SExp) -> Result<TyKind> {
        // Handle symbol types (Never, Infer)
        if let SExp::Symbol(sym) = sexp {
            return match sym.value.as_str() {
                "Never" => Ok(TyKind::Never),
                "Infer" => Ok(TyKind::Infer),
                _ => Err(ParseError::Expected {
                    expected: "Never or Infer".to_string(),
                    found: sym.value.clone(),
                    pos: sym.pos,
                }),
            };
        }

        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        match node_type.value.as_str() {
            "Path" => {
                // For now, simplified - just get the path (element 2)
                let path_sexp = &list.elements[2];
                let path = self.build_path(path_sexp)?;
                Ok(TyKind::Path(None, path))
            }
            "Ref" => {
                let kwargs = parse_kwargs(list)?;
                let lifetime = if let Some(lt_sexp) = optional_field(&kwargs, "lifetime") {
                    Some(self.build_lifetime(lt_sexp)?)
                } else {
                    None
                };
                let mutability = if let Some(mut_sexp) = kwargs.get("mutability") {
                    self.build_mutability(mut_sexp)?
                } else {
                    Mutability::Not
                };
                let ty = Box::new(self.build_ty(require_field(&kwargs, "ty", list.pos)?)?);
                Ok(TyKind::Ref { lifetime, mutability, ty })
            }
            "Ptr" => {
                let kwargs = parse_kwargs(list)?;
                let mutability = if let Some(mut_sexp) = kwargs.get("mutability") {
                    self.build_mutability(mut_sexp)?
                } else {
                    Mutability::Not
                };
                let ty = Box::new(self.build_ty(require_field(&kwargs, "ty", list.pos)?)?);
                Ok(TyKind::Ptr { mutability, ty })
            }
            "Array" => {
                let kwargs = parse_kwargs(list)?;
                let ty = Box::new(self.build_ty(require_field(&kwargs, "ty", list.pos)?)?);
                let len = self.build_expr(require_field(&kwargs, "len", list.pos)?)?;
                Ok(TyKind::Array { ty, len: Box::new(len) })
            }
            "Slice" => {
                let ty = Box::new(self.build_ty(&list.elements[1])?);
                Ok(TyKind::Slice(ty))
            }
            "Tuple" => {
                let tys = list.elements[1..]
                    .iter()
                    .map(|ty_sexp| self.build_ty(ty_sexp))
                    .collect::<Result<Vec<_>>>()?;
                Ok(TyKind::Tuple(tys))
            }
            _ => Err(ParseError::Expected {
                expected: "Path, Ref, Ptr, Array, Slice, Tuple, Never, or Infer".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            }),
        }
    }

    fn build_variant_data(&mut self, sexp: &SExp) -> Result<VariantData> {
        // Check if it's a symbol (Unit) or a list (Struct/Tuple)
        if let SExp::Symbol(sym) = sexp {
            match sym.value.as_str() {
                "Unit" => Ok(VariantData::Unit),
                _ => Err(ParseError::Expected {
                    expected: "Unit, Struct, or Tuple".to_string(),
                    found: sym.value.clone(),
                    pos: sym.pos,
                }),
            }
        } else {
            let list = expect_list(sexp)?;
            let node_type = expect_symbol(&list.elements[0])?;

            match node_type.value.as_str() {
                "Struct" => {
                    let kwargs = parse_kwargs(list)?;
                    let fields =
                        self.build_field_list(require_field(&kwargs, "fields", list.pos)?)?;
                    let recovered = if let Some(rec_sexp) = kwargs.get("recovered") {
                        expect_symbol(rec_sexp)?.value == "true"
                    } else {
                        false
                    };
                    Ok(VariantData::Struct { fields, recovered })
                }
                "Tuple" => {
                    // Tuple has fields as second element
                    let fields_sexp = &list.elements[1];
                    let fields = self.build_field_list(fields_sexp)?;
                    Ok(VariantData::Tuple(fields))
                }
                _ => Err(ParseError::Expected {
                    expected: "Struct or Tuple".to_string(),
                    found: node_type.value.clone(),
                    pos: node_type.pos,
                }),
            }
        }
    }

    fn build_enum_def(&mut self, sexp: &SExp) -> Result<EnumDef> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "EnumDef" {
            return Err(ParseError::Expected {
                expected: "EnumDef".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;
        let variants = self.build_variant_list(require_field(&kwargs, "variants", list.pos)?)?;
        Ok(EnumDef { variants })
    }

    fn build_variant(&mut self, sexp: &SExp) -> Result<Variant> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "Variant" {
            return Err(ParseError::Expected {
                expected: "Variant".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let attrs = Vec::new(); // Simplified for now
        let id = if let Some(id_sexp) = kwargs.get("id") {
            NodeId(expect_number(id_sexp)? as u32)
        } else {
            self.next_id()
        };
        let span = if let Some(span_sexp) = kwargs.get("span") {
            self.build_span(span_sexp)?
        } else {
            Span::DUMMY
        };
        let vis = if let Some(vis_sexp) = kwargs.get("vis") {
            self.build_visibility(vis_sexp)?
        } else {
            Visibility::Inherited
        };
        let ident = self.build_ident(require_field(&kwargs, "ident", list.pos)?)?;
        let data = self.build_variant_data(require_field(&kwargs, "data", list.pos)?)?;

        let disr_expr = if let Some(disr_sexp) = optional_field(&kwargs, "disr-expr") {
            Some(self.build_expr(disr_sexp)?)
        } else {
            None
        };

        Ok(Variant { attrs, id, span, vis, ident, data, disr_expr })
    }

    fn build_field_def(&mut self, sexp: &SExp) -> Result<FieldDef> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "FieldDef" {
            return Err(ParseError::Expected {
                expected: "FieldDef".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let attrs = Vec::new(); // Simplified for now
        let id = if let Some(id_sexp) = kwargs.get("id") {
            NodeId(expect_number(id_sexp)? as u32)
        } else {
            self.next_id()
        };
        let span = if let Some(span_sexp) = kwargs.get("span") {
            self.build_span(span_sexp)?
        } else {
            Span::DUMMY
        };
        let vis = if let Some(vis_sexp) = kwargs.get("vis") {
            self.build_visibility(vis_sexp)?
        } else {
            Visibility::Inherited
        };

        let ident = if let Some(ident_sexp) = optional_field(&kwargs, "ident") {
            Some(self.build_ident(ident_sexp)?)
        } else {
            None
        };

        let ty = self.build_ty(require_field(&kwargs, "ty", list.pos)?)?;

        Ok(FieldDef { attrs, id, span, vis, ident, ty })
    }

    fn build_field_list(&mut self, sexp: &SExp) -> Result<Vec<FieldDef>> {
        let list = expect_list(sexp)?;
        list.elements.iter().map(|elem| self.build_field_def(elem)).collect()
    }

    fn build_variant_list(&mut self, sexp: &SExp) -> Result<Vec<Variant>> {
        let list = expect_list(sexp)?;
        list.elements.iter().map(|elem| self.build_variant(elem)).collect()
    }

    fn build_trait_def(&mut self, sexp: &SExp) -> Result<TraitDef> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "TraitDef" {
            return Err(ParseError::Expected {
                expected: "TraitDef".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let safety = if let Some(safety_sexp) = kwargs.get("safety") {
            self.build_safety(safety_sexp)?
        } else {
            Safety::Default
        };

        let generics = if let Some(generics_sexp) = kwargs.get("generics") {
            self.build_generics(generics_sexp)?
        } else {
            Generics::empty()
        };

        let bounds = if let Some(bounds_sexp) = kwargs.get("bounds") {
            self.build_generic_bound_list(bounds_sexp)?
        } else {
            Vec::new()
        };

        let items = if let Some(items_sexp) = kwargs.get("items") {
            self.build_assoc_item_list(items_sexp)?
        } else {
            Vec::new()
        };

        Ok(TraitDef { safety, generics, bounds, items })
    }

    fn build_impl_def(&mut self, sexp: &SExp) -> Result<ImplDef> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "ImplDef" {
            return Err(ParseError::Expected {
                expected: "ImplDef".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let safety = if let Some(safety_sexp) = kwargs.get("safety") {
            self.build_safety(safety_sexp)?
        } else {
            Safety::Default
        };

        let generics = if let Some(generics_sexp) = kwargs.get("generics") {
            self.build_generics(generics_sexp)?
        } else {
            Generics::empty()
        };

        let of_trait = if let Some(trait_sexp) = optional_field(&kwargs, "of-trait") {
            Some(self.build_trait_ref(trait_sexp)?)
        } else {
            None
        };

        let self_ty = self.build_ty(require_field(&kwargs, "self-ty", list.pos)?)?;

        let items = if let Some(items_sexp) = kwargs.get("items") {
            self.build_assoc_item_list(items_sexp)?
        } else {
            Vec::new()
        };

        Ok(ImplDef { safety, generics, of_trait, self_ty, items })
    }

    fn build_assoc_item(&mut self, sexp: &SExp) -> Result<AssocItem> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "AssocItem" {
            return Err(ParseError::Expected {
                expected: "AssocItem".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let attrs = Vec::new(); // Simplified for now
        let id = if let Some(id_sexp) = kwargs.get("id") {
            NodeId(expect_number(id_sexp)? as u32)
        } else {
            self.next_id()
        };
        let span = if let Some(span_sexp) = kwargs.get("span") {
            self.build_span(span_sexp)?
        } else {
            Span::DUMMY
        };
        let vis = if let Some(vis_sexp) = kwargs.get("vis") {
            self.build_visibility(vis_sexp)?
        } else {
            Visibility::Inherited
        };
        let ident = self.build_ident(require_field(&kwargs, "ident", list.pos)?)?;

        let kind = self.build_assoc_item_kind(require_field(&kwargs, "kind", list.pos)?)?;

        Ok(AssocItem { attrs, id, span, vis, ident, kind })
    }

    fn build_assoc_item_kind(&mut self, sexp: &SExp) -> Result<AssocItemKind> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        match node_type.value.as_str() {
            "Fn" => {
                let fn_sexp = &list.elements[1];
                let fn_list = expect_list(fn_sexp)?;
                let fn_item = self.build_fn(fn_list)?;
                Ok(AssocItemKind::Fn(Box::new(fn_item)))
            }
            "Type" => {
                if list.elements.len() > 1 {
                    let ty_sexp = &list.elements[1];
                    if is_nil(ty_sexp) {
                        Ok(AssocItemKind::Type(Box::new(None)))
                    } else {
                        Ok(AssocItemKind::Type(Box::new(Some(self.build_ty(ty_sexp)?))))
                    }
                } else {
                    Ok(AssocItemKind::Type(Box::new(None)))
                }
            }
            _ => Err(ParseError::Expected {
                expected: "Fn or Type".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            }),
        }
    }

    fn build_trait_ref(&mut self, sexp: &SExp) -> Result<TraitRef> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "TraitRef" {
            return Err(ParseError::Expected {
                expected: "TraitRef".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;
        let path = self.build_path(require_field(&kwargs, "path", list.pos)?)?;

        Ok(TraitRef { path })
    }

    fn build_generic_bound(&mut self, sexp: &SExp) -> Result<GenericBound> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        match node_type.value.as_str() {
            "Trait" => {
                let trait_ref_sexp = &list.elements[1];
                let trait_ref = self.build_trait_ref(trait_ref_sexp)?;
                Ok(GenericBound::Trait(trait_ref))
            }
            _ => Err(ParseError::Expected {
                expected: "Trait".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            }),
        }
    }

    fn build_generic_bound_list(&mut self, sexp: &SExp) -> Result<Vec<GenericBound>> {
        let list = expect_list(sexp)?;
        list.elements.iter().map(|elem| self.build_generic_bound(elem)).collect()
    }

    fn build_assoc_item_list(&mut self, sexp: &SExp) -> Result<Vec<AssocItem>> {
        let list = expect_list(sexp)?;
        list.elements.iter().map(|elem| self.build_assoc_item(elem)).collect()
    }

    fn build_safety(&mut self, sexp: &SExp) -> Result<Safety> {
        let sym = expect_symbol(sexp)?;
        match sym.value.as_str() {
            "Safe" => Ok(Safety::Safe),
            "Unsafe" => Ok(Safety::Unsafe),
            "Default" => Ok(Safety::Default),
            _ => Err(ParseError::Expected {
                expected: "Safe, Unsafe, or Default".to_string(),
                found: sym.value.clone(),
                pos: sym.pos,
            }),
        }
    }

    fn build_mutability(&mut self, sexp: &SExp) -> Result<Mutability> {
        let sym = expect_symbol(sexp)?;
        match sym.value.as_str() {
            "Mut" => Ok(Mutability::Mut),
            "Not" => Ok(Mutability::Not),
            _ => Err(ParseError::Expected {
                expected: "Mut or Not".to_string(),
                found: sym.value.clone(),
                pos: sym.pos,
            }),
        }
    }

    fn build_lifetime(&mut self, sexp: &SExp) -> Result<Lifetime> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "Lifetime" {
            return Err(ParseError::Expected {
                expected: "Lifetime".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let ident = self.build_ident(require_field(&kwargs, "ident", list.pos)?)?;

        Ok(Lifetime { ident })
    }

    /// Stage 8: Build use tree
    fn build_use_tree(&mut self, sexp: &SExp) -> Result<UseTree> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        if node_type.value != "UseTree" {
            return Err(ParseError::Expected {
                expected: "UseTree".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let prefix = self.build_path(require_field(&kwargs, "prefix", list.pos)?)?;

        let kind = self.build_use_tree_kind(require_field(&kwargs, "kind", list.pos)?)?;

        Ok(UseTree { prefix, kind })
    }

    fn build_use_tree_kind(&mut self, sexp: &SExp) -> Result<UseTreeKind> {
        // Check if it's the symbol "Glob"
        if let SExp::Symbol(sym) = sexp {
            if sym.value == "Glob" {
                return Ok(UseTreeKind::Glob);
            }
        }

        // Otherwise it should be a list
        let list = expect_list(sexp)?;
        if list.elements.is_empty() {
            return Err(ParseError::Expected {
                expected: "UseTreeKind variant".to_string(),
                found: "empty list".to_string(),
                pos: list.pos,
            });
        }

        let node_type = expect_symbol(&list.elements[0])?;

        match node_type.value.as_str() {
            "Simple" => {
                let rename = if list.elements.len() > 1 {
                    if is_nil(&list.elements[1]) {
                        None
                    } else {
                        Some(self.build_ident(&list.elements[1])?)
                    }
                } else {
                    None
                };
                Ok(UseTreeKind::Simple(rename))
            }
            "Nested" => {
                let trees = if list.elements.len() > 1 {
                    let trees_list = expect_list(&list.elements[1])?;
                    trees_list
                        .elements
                        .iter()
                        .map(|sexp| self.build_use_tree(sexp))
                        .collect::<Result<Vec<_>>>()?
                } else {
                    Vec::new()
                };
                Ok(UseTreeKind::Nested(trees))
            }
            _ => Err(ParseError::Expected {
                expected: "Simple, Glob, or Nested".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            }),
        }
    }
}
