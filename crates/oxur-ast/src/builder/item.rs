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

        let ident = if let Some(ident_sexp) = kwargs.get("ident") {
            self.build_ident(ident_sexp)?
        } else {
            return Err(ParseError::Expected {
                expected: ":ident field".to_string(),
                found: "missing".to_string(),
                pos: list.pos,
            });
        };

        let kind = if let Some(kind_sexp) = kwargs.get("kind") {
            self.build_item_kind(kind_sexp)?
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

        let name = if let Some(name_sexp) = kwargs.get("name") {
            expect_string(name_sexp)?
        } else {
            return Err(ParseError::Expected {
                expected: ":name field".to_string(),
                found: "missing".to_string(),
                pos: list.pos,
            });
        };

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
                let fn_item = self.build_fn(list)?;
                Ok(ItemKind::Fn(Box::new(fn_item)))
            }
            _ => Err(ParseError::Expected {
                expected: "Fn (only supported in Phase 1)".to_string(),
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

        let sig = if let Some(sig_sexp) = kwargs.get("sig") {
            self.build_fn_sig(sig_sexp)?
        } else {
            return Err(ParseError::Expected {
                expected: ":sig field".to_string(),
                found: "missing".to_string(),
                pos: list.pos,
            });
        };

        let generics = if let Some(gen_sexp) = kwargs.get("generics") {
            self.build_generics(gen_sexp)?
        } else {
            Generics::empty()
        };

        let body = if let Some(body_sexp) = kwargs.get("body") {
            if !is_nil(body_sexp) {
                Some(self.build_block(body_sexp)?)
            } else {
                None
            }
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

    fn build_param(&mut self, sexp: &SExp) -> Result<Param> {
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
}
