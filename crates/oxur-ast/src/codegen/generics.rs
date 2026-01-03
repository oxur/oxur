use crate::ast::*;
use crate::codegen::rust::RustCodegen;
use anyhow::Result;

impl RustCodegen {
    /// Generate Generics
    pub(crate) fn generate_generics(&mut self, generics: &Generics) -> Result<()> {
        // Only generate if there are params or where clauses
        if generics.params.is_empty() && generics.where_clause.predicates.is_empty() {
            return Ok(());
        }

        // Generate <T, U, V> part
        if !generics.params.is_empty() {
            self.write("<");
            for (i, param) in generics.params.iter().enumerate() {
                if i > 0 {
                    self.write(", ");
                }
                self.generate_generic_param(param)?;
            }
            self.write(">");
        }

        Ok(())
    }

    /// Generate a GenericParam
    fn generate_generic_param(&mut self, param: &GenericParam) -> Result<()> {
        match &param.kind {
            GenericParamKind::Lifetime(lifetime_param) => {
                self.generate_lifetime_param(lifetime_param)?;
            }
            GenericParamKind::Type(type_param) => {
                self.generate_type_param(type_param)?;
            }
            GenericParamKind::Const(const_param) => {
                self.generate_const_param(const_param)?;
            }
        }
        Ok(())
    }

    /// Generate a LifetimeParam: `'a`, `'a: 'b`, `'a: 'b + 'c`
    fn generate_lifetime_param(&mut self, param: &LifetimeParam) -> Result<()> {
        self.write("'");
        self.write(&param.ident.name);

        if !param.bounds.is_empty() {
            self.write(": ");
            for (i, bound) in param.bounds.iter().enumerate() {
                if i > 0 {
                    self.write(" + ");
                }
                self.generate_lifetime(bound)?;
            }
        }

        Ok(())
    }

    /// Generate a TypeParam: `T`, `T: Clone`, `T: Clone + Debug`, `T = i32`
    fn generate_type_param(&mut self, param: &TypeParam) -> Result<()> {
        self.write(&param.ident.name);

        if !param.bounds.is_empty() {
            self.write(": ");
            for (i, bound) in param.bounds.iter().enumerate() {
                if i > 0 {
                    self.write(" + ");
                }
                self.generate_generic_bound(bound)?;
            }
        }

        if let Some(default) = &param.default {
            self.write(" = ");
            self.generate_ty(default)?;
        }

        Ok(())
    }

    /// Generate a ConstParam: `const N: usize`, `const N: usize = 5`
    fn generate_const_param(&mut self, param: &ConstParam) -> Result<()> {
        self.write("const ");
        self.write(&param.ident.name);
        self.write(": ");
        self.generate_ty(&param.ty)?;

        if let Some(default) = &param.default {
            self.write(" = ");
            self.generate_expr(default)?;
        }

        Ok(())
    }

    /// Generate a WhereClause
    pub(crate) fn generate_where_clause(&mut self, where_clause: &WhereClause) -> Result<()> {
        if !where_clause.has_where_token || where_clause.predicates.is_empty() {
            return Ok(());
        }

        self.write("\nwhere\n");
        self.indent();

        for (i, predicate) in where_clause.predicates.iter().enumerate() {
            if i > 0 {
                self.write(",\n");
            }
            self.write_indent();
            self.generate_where_predicate(predicate)?;
        }

        self.dedent();

        Ok(())
    }

    /// Generate a WherePredicate
    fn generate_where_predicate(&mut self, predicate: &WherePredicate) -> Result<()> {
        match predicate {
            WherePredicate::BoundPredicate(bound) => {
                self.generate_where_bound_predicate(bound)?;
            }
            WherePredicate::RegionPredicate(region) => {
                self.generate_where_region_predicate(region)?;
            }
            WherePredicate::EqPredicate(eq) => {
                self.generate_where_eq_predicate(eq)?;
            }
        }
        Ok(())
    }

    /// Generate a WhereBoundPredicate: `T: Clone`, `C::Item: Display`
    fn generate_where_bound_predicate(&mut self, predicate: &WhereBoundPredicate) -> Result<()> {
        // Generate HRTB if present: `for<'a>`
        if !predicate.bound_lifetimes.is_empty() {
            self.write("for<");
            for (i, lifetime_param) in predicate.bound_lifetimes.iter().enumerate() {
                if i > 0 {
                    self.write(", ");
                }
                self.write(&lifetime_param.ident.name);
            }
            self.write("> ");
        }

        // Generate the bounded type
        self.generate_ty(&predicate.bounded_ty)?;
        self.write(": ");

        // Generate bounds
        for (i, bound) in predicate.bounds.iter().enumerate() {
            if i > 0 {
                self.write(" + ");
            }
            self.generate_generic_bound(bound)?;
        }

        Ok(())
    }

    /// Generate a WhereRegionPredicate: `'a: 'b`, `'a: 'b + 'c`
    fn generate_where_region_predicate(&mut self, predicate: &WhereRegionPredicate) -> Result<()> {
        self.generate_lifetime(&predicate.lifetime)?;
        self.write(": ");

        for (i, bound) in predicate.bounds.iter().enumerate() {
            if i > 0 {
                self.write(" + ");
            }
            self.generate_lifetime(bound)?;
        }

        Ok(())
    }

    /// Generate a WhereEqPredicate: `T = int` (associated type equality)
    fn generate_where_eq_predicate(&mut self, predicate: &WhereEqPredicate) -> Result<()> {
        self.generate_ty(&predicate.lhs_ty)?;
        self.write(" = ");
        self.generate_ty(&predicate.rhs_ty)?;

        Ok(())
    }

    // Note: generate_generic_bound and generate_lifetime are already in item.rs
}
