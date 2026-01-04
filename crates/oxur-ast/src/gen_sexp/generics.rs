use crate::ast::*;
use crate::error::Result;
use crate::gen_sexp::gen::Generator;
use crate::gen_sexp::helpers::*;
use crate::sexp::SExp;

impl Generator {
    /// Generate a Generics structure
    pub fn generate_generics(&self, generics: &Generics) -> Result<SExp> {
        let params: Result<Vec<_>> =
            generics.params.iter().map(|p| self.generate_generic_param(p)).collect();
        let params = params?;

        Ok(typed_node(
            "Generics",
            kwargs(vec![
                kwarg("params", list(params)),
                kwarg("where-clause", self.generate_where_clause(&generics.where_clause)?),
                kwarg("span", self.generate_span(generics.span)),
            ]),
        ))
    }

    /// Generate a GenericParam
    fn generate_generic_param(&self, param: &GenericParam) -> Result<SExp> {
        Ok(typed_node(
            "GenericParam",
            kwargs(vec![
                kwarg("attrs", empty_list()), // TODO: implement attribute generation
                kwarg("id", self.generate_node_id(param.id)),
                kwarg("span", self.generate_span(param.span)),
                kwarg("kind", self.generate_generic_param_kind(&param.kind)?),
            ]),
        ))
    }

    /// Generate a GenericParamKind
    fn generate_generic_param_kind(&self, kind: &GenericParamKind) -> Result<SExp> {
        match kind {
            GenericParamKind::Lifetime(lifetime_param) => {
                Ok(list(vec![sym("Lifetime"), self.generate_lifetime_param(lifetime_param)?]))
            }
            GenericParamKind::Type(type_param) => {
                Ok(list(vec![sym("Type"), self.generate_type_param(type_param)?]))
            }
            GenericParamKind::Const(const_param) => {
                Ok(list(vec![sym("Const"), self.generate_const_param(const_param)?]))
            }
        }
    }

    /// Generate a LifetimeParam
    fn generate_lifetime_param(&self, param: &LifetimeParam) -> Result<SExp> {
        let bounds: Vec<_> = param.bounds.iter().map(|b| self.generate_lifetime(b)).collect();

        Ok(typed_node(
            "LifetimeParam",
            kwargs(vec![
                kwarg("ident", self.generate_ident(&param.ident)),
                kwarg("bounds", list(bounds)),
                kwarg(
                    "colon-span",
                    match &param.colon_span {
                        Some(span) => self.generate_span(*span),
                        None => sym("nil"),
                    },
                ),
            ]),
        ))
    }

    /// Generate a TypeParam
    fn generate_type_param(&self, param: &TypeParam) -> Result<SExp> {
        let bounds: Result<Vec<_>> =
            param.bounds.iter().map(|b| self.generate_generic_bound(b)).collect();
        let bounds = bounds?;

        Ok(typed_node(
            "TypeParam",
            kwargs(vec![
                kwarg("ident", self.generate_ident(&param.ident)),
                kwarg("bounds", list(bounds)),
                kwarg(
                    "default",
                    match &param.default {
                        Some(ty) => self.generate_ty(ty)?,
                        None => sym("nil"),
                    },
                ),
            ]),
        ))
    }

    /// Generate a ConstParam
    fn generate_const_param(&self, param: &ConstParam) -> Result<SExp> {
        Ok(typed_node(
            "ConstParam",
            kwargs(vec![
                kwarg("ident", self.generate_ident(&param.ident)),
                kwarg("ty", self.generate_ty(&param.ty)?),
                kwarg(
                    "default",
                    match &param.default {
                        Some(expr) => self.generate_expr(expr)?,
                        None => sym("nil"),
                    },
                ),
            ]),
        ))
    }

    /// Generate a WhereClause
    pub fn generate_where_clause(&self, where_clause: &WhereClause) -> Result<SExp> {
        let predicates: Result<Vec<_>> =
            where_clause.predicates.iter().map(|p| self.generate_where_predicate(p)).collect();
        let predicates = predicates?;

        Ok(typed_node(
            "WhereClause",
            kwargs(vec![
                kwarg(
                    "has-where-token",
                    if where_clause.has_where_token { sym("true") } else { sym("false") },
                ),
                kwarg("predicates", list(predicates)),
                kwarg("span", self.generate_span(where_clause.span)),
            ]),
        ))
    }

    /// Generate a WherePredicate
    fn generate_where_predicate(&self, predicate: &WherePredicate) -> Result<SExp> {
        match predicate {
            WherePredicate::BoundPredicate(bound) => {
                Ok(list(vec![sym("BoundPredicate"), self.generate_where_bound_predicate(bound)?]))
            }
            WherePredicate::RegionPredicate(region) => Ok(list(vec![
                sym("RegionPredicate"),
                self.generate_where_region_predicate(region)?,
            ])),
            WherePredicate::EqPredicate(eq) => {
                Ok(list(vec![sym("EqPredicate"), self.generate_where_eq_predicate(eq)?]))
            }
        }
    }

    /// Generate a WhereBoundPredicate
    fn generate_where_bound_predicate(&self, predicate: &WhereBoundPredicate) -> Result<SExp> {
        let bounds: Result<Vec<_>> =
            predicate.bounds.iter().map(|b| self.generate_generic_bound(b)).collect();
        let bounds = bounds?;

        let bound_lifetimes: Result<Vec<_>> =
            predicate.bound_lifetimes.iter().map(|l| self.generate_lifetime_param(l)).collect();
        let bound_lifetimes = bound_lifetimes?;

        Ok(typed_node(
            "WhereBoundPredicate",
            kwargs(vec![
                kwarg("span", self.generate_span(predicate.span)),
                kwarg("bounded-ty", self.generate_ty(&predicate.bounded_ty)?),
                kwarg("bounds", list(bounds)),
                kwarg("bound-lifetimes", list(bound_lifetimes)),
            ]),
        ))
    }

    /// Generate a WhereRegionPredicate
    fn generate_where_region_predicate(&self, predicate: &WhereRegionPredicate) -> Result<SExp> {
        let bounds: Vec<_> = predicate.bounds.iter().map(|b| self.generate_lifetime(b)).collect();

        Ok(typed_node(
            "WhereRegionPredicate",
            kwargs(vec![
                kwarg("span", self.generate_span(predicate.span)),
                kwarg("lifetime", self.generate_lifetime(&predicate.lifetime)),
                kwarg("bounds", list(bounds)),
            ]),
        ))
    }

    /// Generate a WhereEqPredicate
    fn generate_where_eq_predicate(&self, predicate: &WhereEqPredicate) -> Result<SExp> {
        Ok(typed_node(
            "WhereEqPredicate",
            kwargs(vec![
                kwarg("span", self.generate_span(predicate.span)),
                kwarg("lhs-ty", self.generate_ty(&predicate.lhs_ty)?),
                kwarg("rhs-ty", self.generate_ty(&predicate.rhs_ty)?),
            ]),
        ))
    }

    // Note: generate_generic_bound is already in item.rs
    // Note: generate_lifetime is already in item.rs
}
