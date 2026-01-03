use crate::ast::*;
use crate::builder::build::AstBuilder;
use crate::builder::helpers::*;
use crate::error::Result;
use crate::sexp::SExp;

impl AstBuilder {
    /// Build a Generics structure
    pub fn build_generics(&mut self, sexp: &SExp) -> Result<Generics> {
        let list = expect_node_type(expect_list(sexp)?, "Generics")?;
        let kwargs = parse_kwargs(list)?;

        let params_sexp = require_field(&kwargs, "params", list.pos)?;
        let params = self.build_generic_param_list(params_sexp)?;

        let where_clause_sexp = require_field(&kwargs, "where-clause", list.pos)?;
        let where_clause = self.build_where_clause(where_clause_sexp)?;

        let span_sexp = require_field(&kwargs, "span", list.pos)?;
        let span = self.build_span(span_sexp)?;

        Ok(Generics {
            params,
            where_clause,
            span,
        })
    }

    /// Build a list of GenericParam
    fn build_generic_param_list(&mut self, sexp: &SExp) -> Result<Vec<GenericParam>> {
        let list = expect_list(sexp)?;
        list.elements.iter().map(|elem| self.build_generic_param(elem)).collect()
    }

    /// Build a GenericParam
    fn build_generic_param(&mut self, sexp: &SExp) -> Result<GenericParam> {
        let list = expect_node_type(expect_list(sexp)?, "GenericParam")?;
        let kwargs = parse_kwargs(list)?;

        let attrs_sexp = require_field(&kwargs, "attrs", list.pos)?;
        let attrs = self.build_attr_vec(attrs_sexp)?;

        let id_sexp = require_field(&kwargs, "id", list.pos)?;
        let id = self.build_node_id(id_sexp)?;

        let span_sexp = require_field(&kwargs, "span", list.pos)?;
        let span = self.build_span(span_sexp)?;

        let kind_sexp = require_field(&kwargs, "kind", list.pos)?;
        let kind = self.build_generic_param_kind(kind_sexp)?;

        Ok(GenericParam {
            attrs,
            id,
            span,
            kind,
        })
    }

    /// Build a GenericParamKind
    fn build_generic_param_kind(&mut self, sexp: &SExp) -> Result<GenericParamKind> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        match node_type.value.as_str() {
            "Lifetime" => {
                let lifetime_param_sexp = &list.elements[1];
                let lifetime_param = self.build_lifetime_param(lifetime_param_sexp)?;
                Ok(GenericParamKind::Lifetime(lifetime_param))
            }
            "Type" => {
                let type_param_sexp = &list.elements[1];
                let type_param = self.build_type_param(type_param_sexp)?;
                Ok(GenericParamKind::Type(type_param))
            }
            "Const" => {
                let const_param_sexp = &list.elements[1];
                let const_param = self.build_const_param(const_param_sexp)?;
                Ok(GenericParamKind::Const(const_param))
            }
            _ => Err(ParseError::Expected {
                expected: "Lifetime, Type, or Const".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            }),
        }
    }

    /// Build a LifetimeParam
    fn build_lifetime_param(&mut self, sexp: &SExp) -> Result<LifetimeParam> {
        let list = expect_node_type(expect_list(sexp)?, "LifetimeParam")?;
        let kwargs = parse_kwargs(list)?;

        let ident_sexp = require_field(&kwargs, "ident", list.pos)?;
        let ident = self.build_ident(ident_sexp)?;

        let bounds_sexp = require_field(&kwargs, "bounds", list.pos)?;
        let bounds = self.build_lifetime_list(bounds_sexp)?;

        let colon_span_sexp = require_field(&kwargs, "colon-span", list.pos)?;
        let colon_span = if is_nil(colon_span_sexp)? {
            None
        } else {
            Some(self.build_span(colon_span_sexp)?)
        };

        Ok(LifetimeParam {
            ident,
            bounds,
            colon_span,
        })
    }

    /// Build a list of Lifetime
    fn build_lifetime_list(&mut self, sexp: &SExp) -> Result<Vec<Lifetime>> {
        let list = expect_list(sexp)?;
        list.elements.iter().map(|elem| self.build_lifetime(elem)).collect()
    }

    /// Build a TypeParam
    fn build_type_param(&mut self, sexp: &SExp) -> Result<TypeParam> {
        let list = expect_node_type(expect_list(sexp)?, "TypeParam")?;
        let kwargs = parse_kwargs(list)?;

        let ident_sexp = require_field(&kwargs, "ident", list.pos)?;
        let ident = self.build_ident(ident_sexp)?;

        let bounds_sexp = require_field(&kwargs, "bounds", list.pos)?;
        let bounds = self.build_generic_bound_list(bounds_sexp)?;

        let default_sexp = require_field(&kwargs, "default", list.pos)?;
        let default = if is_nil(default_sexp)? {
            None
        } else {
            Some(self.build_ty(default_sexp)?)
        };

        Ok(TypeParam {
            ident,
            bounds,
            default,
        })
    }

    /// Build a ConstParam
    fn build_const_param(&mut self, sexp: &SExp) -> Result<ConstParam> {
        let list = expect_node_type(expect_list(sexp)?, "ConstParam")?;
        let kwargs = parse_kwargs(list)?;

        let ident_sexp = require_field(&kwargs, "ident", list.pos)?;
        let ident = self.build_ident(ident_sexp)?;

        let ty_sexp = require_field(&kwargs, "ty", list.pos)?;
        let ty = self.build_ty(ty_sexp)?;

        let default_sexp = require_field(&kwargs, "default", list.pos)?;
        let default = if is_nil(default_sexp)? {
            None
        } else {
            Some(self.build_expr(default_sexp)?)
        };

        Ok(ConstParam {
            ident,
            ty,
            default,
        })
    }

    /// Build a WhereClause
    pub fn build_where_clause(&mut self, sexp: &SExp) -> Result<WhereClause> {
        let list = expect_node_type(expect_list(sexp)?, "WhereClause")?;
        let kwargs = parse_kwargs(list)?;

        let has_where_token_sexp = require_field(&kwargs, "has-where-token", list.pos)?;
        let has_where_token = parse_bool(has_where_token_sexp)?;

        let predicates_sexp = require_field(&kwargs, "predicates", list.pos)?;
        let predicates = self.build_where_predicate_list(predicates_sexp)?;

        let span_sexp = require_field(&kwargs, "span", list.pos)?;
        let span = self.build_span(span_sexp)?;

        Ok(WhereClause {
            has_where_token,
            predicates,
            span,
        })
    }

    /// Build a list of WherePredicate
    fn build_where_predicate_list(&mut self, sexp: &SExp) -> Result<Vec<WherePredicate>> {
        let list = expect_list(sexp)?;
        list.elements.iter().map(|elem| self.build_where_predicate(elem)).collect()
    }

    /// Build a WherePredicate
    fn build_where_predicate(&mut self, sexp: &SExp) -> Result<WherePredicate> {
        let list = expect_list(sexp)?;
        let node_type = expect_symbol(&list.elements[0])?;

        match node_type.value.as_str() {
            "BoundPredicate" => {
                let bound_sexp = &list.elements[1];
                let bound = self.build_where_bound_predicate(bound_sexp)?;
                Ok(WherePredicate::BoundPredicate(bound))
            }
            "RegionPredicate" => {
                let region_sexp = &list.elements[1];
                let region = self.build_where_region_predicate(region_sexp)?;
                Ok(WherePredicate::RegionPredicate(region))
            }
            "EqPredicate" => {
                let eq_sexp = &list.elements[1];
                let eq = self.build_where_eq_predicate(eq_sexp)?;
                Ok(WherePredicate::EqPredicate(eq))
            }
            _ => Err(ParseError::Expected {
                expected: "BoundPredicate, RegionPredicate, or EqPredicate".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            }),
        }
    }

    /// Build a WhereBoundPredicate
    fn build_where_bound_predicate(&mut self, sexp: &SExp) -> Result<WhereBoundPredicate> {
        let list = expect_node_type(expect_list(sexp)?, "WhereBoundPredicate")?;
        let kwargs = parse_kwargs(list)?;

        let span_sexp = require_field(&kwargs, "span", list.pos)?;
        let span = self.build_span(span_sexp)?;

        let bounded_ty_sexp = require_field(&kwargs, "bounded-ty", list.pos)?;
        let bounded_ty = self.build_ty(bounded_ty_sexp)?;

        let bounds_sexp = require_field(&kwargs, "bounds", list.pos)?;
        let bounds = self.build_generic_bound_list(bounds_sexp)?;

        let bound_lifetimes_sexp = require_field(&kwargs, "bound-lifetimes", list.pos)?;
        let bound_lifetimes = self.build_lifetime_param_list(bound_lifetimes_sexp)?;

        Ok(WhereBoundPredicate {
            span,
            bounded_ty,
            bounds,
            bound_lifetimes,
        })
    }

    /// Build a list of LifetimeParam
    fn build_lifetime_param_list(&mut self, sexp: &SExp) -> Result<Vec<LifetimeParam>> {
        let list = expect_list(sexp)?;
        list.elements.iter().map(|elem| self.build_lifetime_param(elem)).collect()
    }

    /// Build a WhereRegionPredicate
    fn build_where_region_predicate(&mut self, sexp: &SExp) -> Result<WhereRegionPredicate> {
        let list = expect_node_type(expect_list(sexp)?, "WhereRegionPredicate")?;
        let kwargs = parse_kwargs(list)?;

        let span_sexp = require_field(&kwargs, "span", list.pos)?;
        let span = self.build_span(span_sexp)?;

        let lifetime_sexp = require_field(&kwargs, "lifetime", list.pos)?;
        let lifetime = self.build_lifetime(lifetime_sexp)?;

        let bounds_sexp = require_field(&kwargs, "bounds", list.pos)?;
        let bounds = self.build_lifetime_list(bounds_sexp)?;

        Ok(WhereRegionPredicate {
            span,
            lifetime,
            bounds,
        })
    }

    /// Build a WhereEqPredicate
    fn build_where_eq_predicate(&mut self, sexp: &SExp) -> Result<WhereEqPredicate> {
        let list = expect_node_type(expect_list(sexp)?, "WhereEqPredicate")?;
        let kwargs = parse_kwargs(list)?;

        let span_sexp = require_field(&kwargs, "span", list.pos)?;
        let span = self.build_span(span_sexp)?;

        let lhs_ty_sexp = require_field(&kwargs, "lhs-ty", list.pos)?;
        let lhs_ty = self.build_ty(lhs_ty_sexp)?;

        let rhs_ty_sexp = require_field(&kwargs, "rhs-ty", list.pos)?;
        let rhs_ty = self.build_ty(rhs_ty_sexp)?;

        Ok(WhereEqPredicate {
            span,
            lhs_ty,
            rhs_ty,
        })
    }

    // Note: build_generic_bound and build_lifetime are already in item.rs
}

use crate::error::ParseError;

/// Helper to check if an SExp is nil
fn is_nil(sexp: &SExp) -> Result<bool> {
    match sexp {
        SExp::Symbol(sym) if sym.value == "nil" => Ok(true),
        _ => Ok(false),
    }
}

/// Helper to parse a boolean SExp
fn parse_bool(sexp: &SExp) -> Result<bool> {
    match sexp {
        SExp::Symbol(sym) => match sym.value.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(ParseError::Expected {
                expected: "true or false".to_string(),
                found: sym.value.clone(),
                pos: sym.pos,
            }),
        },
        _ => Err(ParseError::Expected {
            expected: "boolean symbol".to_string(),
            found: format!("{:?}", sexp),
            pos: Position::new(0, 1, 1),
        }),
    }
}
