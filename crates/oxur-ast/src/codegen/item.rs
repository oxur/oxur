//! Item code generation
//!
//! Generates Rust code for top-level items (functions, structs, etc.)

use crate::ast::*;
use crate::codegen::RustCodegen;
use anyhow::Result;

impl RustCodegen {
    /// Generate code for a top-level item
    pub(crate) fn generate_item(&mut self, item: &Item) -> Result<()> {
        // Generate visibility
        self.generate_visibility(&item.vis)?;

        // Dispatch on item kind
        match &item.kind {
            ItemKind::Fn(func) => self.generate_fn_item(&item.ident, func)?,
            ItemKind::Struct(data) => self.generate_struct_item(&item.ident, data)?,
            ItemKind::Enum(enum_def) => self.generate_enum_item(&item.ident, enum_def)?,
            ItemKind::Trait(trait_def) => self.generate_trait_item(&item.ident, trait_def)?,
            ItemKind::Impl(impl_def) => self.generate_impl_item(impl_def)?,
            // Stage 8: Remaining items
            ItemKind::Use(use_tree) => self.generate_use_item(use_tree)?,
            ItemKind::Static { mutability, ty, expr } => {
                self.generate_static_item(&item.ident, *mutability, ty, expr)?
            }
            ItemKind::Const { ty, expr } => self.generate_const_item(&item.ident, ty, expr)?,
            ItemKind::TyAlias { generics, ty } => {
                self.generate_type_alias_item(&item.ident, generics, ty)?
            }
            ItemKind::Mod { items } => self.generate_mod_item(&item.ident, items)?,
        }

        Ok(())
    }

    /// Generate a function item
    fn generate_fn_item(&mut self, ident: &Ident, func: &Fn) -> Result<()> {
        // Generate function header (const, async, unsafe, extern)
        self.generate_fn_header(&func.sig.header)?;

        // Function keyword
        self.write("fn ");

        // Function name
        self.write(&ident.name);

        // Generics
        self.generate_generics(&func.generics)?;

        // Parameters
        self.write("(");
        for (i, param) in func.sig.decl.inputs.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.generate_param(param)?;
        }
        self.write(")");

        // Return type
        self.generate_fn_ret_ty(&func.sig.decl.output)?;

        // Where clause
        self.generate_where_clause(&func.generics.where_clause)?;

        // Body
        if let Some(body) = &func.body {
            self.write(" ");
            self.generate_block(body)?;
        } else {
            // Function declaration without body (e.g., in trait)
            self.write(";");
        }

        self.writeln();

        Ok(())
    }

    /// Generate function header modifiers (const, async, unsafe, extern)
    fn generate_fn_header(&mut self, header: &FnHeader) -> Result<()> {
        // Const
        if matches!(header.constness, Constness::Const) {
            self.write("const ");
        }

        // Async/gen
        if let Some(coroutine) = &header.coroutine_kind {
            match coroutine {
                CoroutineKind::Async => self.write("async "),
                CoroutineKind::Gen => self.write("gen "),
            }
        }

        // Unsafe
        if matches!(header.safety, Safety::Unsafe) {
            self.write("unsafe ");
        }

        // Extern
        match &header.ext {
            Extern::None => {}
            Extern::Explicit(abi) => {
                self.write("extern ");
                self.write("\"");
                self.write(abi);
                self.write("\" ");
            }
        }

        Ok(())
    }

    /// Generate a function parameter
    fn generate_param(&mut self, param: &Param) -> Result<()> {
        // Pattern (parameter name)
        self.generate_pat(&param.pat)?;

        // Type
        self.write(": ");
        self.generate_ty(&param.ty)?;

        Ok(())
    }

    /// Generate function return type
    fn generate_fn_ret_ty(&mut self, ret_ty: &FnRetTy) -> Result<()> {
        match ret_ty {
            FnRetTy::Default(_) => {
                // No return type (unit)
            }
            FnRetTy::Ty(ty) => {
                self.write(" -> ");
                self.generate_ty(ty)?;
            }
        }
        Ok(())
    }

    /// Generate visibility modifier
    fn generate_visibility(&mut self, vis: &Visibility) -> Result<()> {
        match vis {
            Visibility::Public => {
                self.write("pub ");
            }
            Visibility::Restricted { path, shorthand, .. } => {
                self.write("pub(");
                match shorthand {
                    VisRestrictionKind::Crate => self.write("crate"),
                    VisRestrictionKind::Super => self.write("super"),
                    VisRestrictionKind::In => {
                        self.write("in ");
                        self.generate_path(path)?;
                    }
                }
                self.write(") ");
            }
            Visibility::Inherited => {
                // No visibility modifier
            }
        }
        Ok(())
    }

    /// Generate a struct item
    fn generate_struct_item(&mut self, ident: &Ident, data: &VariantData) -> Result<()> {
        self.write("struct ");
        self.write(&ident.name);

        match data {
            VariantData::Struct { fields, .. } => {
                // Named fields: struct Point { x: i32, y: i32 }
                self.write(" {");
                if !fields.is_empty() {
                    self.writeln();
                    self.indent();
                    for field in fields {
                        self.write_indent();
                        self.generate_field(field)?;
                        self.write(",");
                        self.writeln();
                    }
                    self.dedent();
                    self.write_indent();
                }
                self.write("}");
            }
            VariantData::Tuple(fields) => {
                // Tuple struct: struct Color(u8, u8, u8)
                self.write("(");
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_visibility(&field.vis)?;
                    self.generate_ty(&field.ty)?;
                }
                self.write(");");
            }
            VariantData::Unit => {
                // Unit struct: struct Marker;
                self.write(";");
            }
        }

        self.writeln();
        Ok(())
    }

    /// Generate an enum item
    fn generate_enum_item(&mut self, ident: &Ident, enum_def: &EnumDef) -> Result<()> {
        self.write("enum ");
        self.write(&ident.name);
        self.write(" {");
        self.writeln();
        self.indent();

        for variant in &enum_def.variants {
            self.write_indent();
            self.generate_variant(variant)?;
            self.write(",");
            self.writeln();
        }

        self.dedent();
        self.write_indent();
        self.write("}");
        self.writeln();
        Ok(())
    }

    /// Generate a field definition
    fn generate_field(&mut self, field: &FieldDef) -> Result<()> {
        self.generate_visibility(&field.vis)?;
        if let Some(ident) = &field.ident {
            self.write(&ident.name);
            self.write(": ");
        }
        self.generate_ty(&field.ty)?;
        Ok(())
    }

    /// Generate an enum variant
    fn generate_variant(&mut self, variant: &Variant) -> Result<()> {
        self.generate_visibility(&variant.vis)?;
        self.write(&variant.ident.name);

        match &variant.data {
            VariantData::Struct { fields, .. } => {
                // Struct variant: Point { x: i32, y: i32 }
                self.write(" { ");
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    if let Some(ident) = &field.ident {
                        self.write(&ident.name);
                        self.write(": ");
                    }
                    self.generate_ty(&field.ty)?;
                }
                self.write(" }");
            }
            VariantData::Tuple(fields) => {
                // Tuple variant: Some(T)
                self.write("(");
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_ty(&field.ty)?;
                }
                self.write(")");
            }
            VariantData::Unit => {
                // Unit variant: None
                // No additional output needed
            }
        }

        // Explicit discriminant: Foo = 1
        if let Some(ref expr) = variant.disr_expr {
            self.write(" = ");
            self.generate_expr(expr)?;
        }

        Ok(())
    }

    /// Generate a trait item
    fn generate_trait_item(&mut self, ident: &Ident, trait_def: &TraitDef) -> Result<()> {
        // Safety keyword
        if matches!(trait_def.safety, Safety::Unsafe) {
            self.write("unsafe ");
        }

        self.write("trait ");
        self.write(&ident.name);

        // Generic parameters (simplified for now)
        if !trait_def.generics.params.is_empty() {
            self.write("</* generics */>");
        }

        // Trait bounds
        if !trait_def.bounds.is_empty() {
            self.write(": ");
            for (i, bound) in trait_def.bounds.iter().enumerate() {
                if i > 0 {
                    self.write(" + ");
                }
                self.generate_generic_bound(bound)?;
            }
        }

        self.write(" {");
        self.writeln();
        self.indent();

        // Associated items
        for item in &trait_def.items {
            self.generate_assoc_item(item)?;
        }

        self.dedent();
        self.write_indent();
        self.write("}");
        self.writeln();
        Ok(())
    }

    /// Generate an impl item
    fn generate_impl_item(&mut self, impl_def: &ImplDef) -> Result<()> {
        // Safety keyword
        if matches!(impl_def.safety, Safety::Unsafe) {
            self.write("unsafe ");
        }

        self.write("impl");

        // Generic parameters (simplified for now)
        if !impl_def.generics.params.is_empty() {
            self.write("</* generics */>");
        }

        self.write(" ");

        // Trait reference (for trait impls)
        if let Some(trait_ref) = &impl_def.of_trait {
            self.generate_trait_ref(trait_ref)?;
            self.write(" for ");
        }

        // Self type
        self.generate_ty(&impl_def.self_ty)?;

        self.write(" {");
        self.writeln();
        self.indent();

        // Associated items
        for item in &impl_def.items {
            self.generate_assoc_item(item)?;
        }

        self.dedent();
        self.write_indent();
        self.write("}");
        self.writeln();
        Ok(())
    }

    /// Generate an associated item
    fn generate_assoc_item(&mut self, item: &AssocItem) -> Result<()> {
        self.write_indent();
        self.generate_visibility(&item.vis)?;

        match &item.kind {
            AssocItemKind::Fn(func) => {
                self.generate_fn_item(&item.ident, func)?;
            }
            AssocItemKind::Type(ty_opt) => {
                self.write("type ");
                self.write(&item.ident.name);
                if let Some(ty) = &**ty_opt {
                    self.write(" = ");
                    self.generate_ty(ty)?;
                }
                self.write(";");
                self.writeln();
            }
        }

        Ok(())
    }

    /// Generate a trait reference
    fn generate_trait_ref(&mut self, trait_ref: &TraitRef) -> Result<()> {
        self.generate_path(&trait_ref.path)?;
        Ok(())
    }

    /// Generate a generic bound
    pub(crate) fn generate_generic_bound(&mut self, bound: &GenericBound) -> Result<()> {
        match bound {
            GenericBound::Trait(poly_trait_ref, modifier) => {
                // Generate modifier prefix
                match modifier {
                    TraitBoundModifier::Maybe => self.write("?"),
                    TraitBoundModifier::Negative => self.write("!"),
                    TraitBoundModifier::None => {},
                }
                // Generate the trait reference
                self.generate_trait_ref(&poly_trait_ref.trait_ref)?;
                // TODO: Handle bound_lifetimes (HRTB) when we implement full lifetime support
            }
            GenericBound::Outlives(lifetime) => {
                self.generate_lifetime(lifetime)?;
            }
        }
        Ok(())
    }

    pub(crate) fn generate_lifetime(&mut self, lifetime: &Lifetime) -> Result<()> {
        self.write("'");
        self.write(&lifetime.ident.name);
        Ok(())
    }

    /// Stage 8: Generate a use item
    fn generate_use_item(&mut self, use_tree: &UseTree) -> Result<()> {
        self.write("use ");
        self.generate_use_tree(use_tree)?;
        self.write(";");
        self.writeln();
        Ok(())
    }

    /// Generate a use tree
    fn generate_use_tree(&mut self, use_tree: &UseTree) -> Result<()> {
        // Generate prefix path
        self.generate_path(&use_tree.prefix)?;

        // Generate kind
        match &use_tree.kind {
            UseTreeKind::Simple(rename) => {
                if let Some(ident) = rename {
                    self.write(" as ");
                    self.write(&ident.name);
                }
            }
            UseTreeKind::Glob => {
                self.write("::*");
            }
            UseTreeKind::Nested(trees) => {
                self.write("::{");
                for (i, tree) in trees.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_use_tree(tree)?;
                }
                self.write("}");
            }
        }

        Ok(())
    }

    /// Generate a static item
    fn generate_static_item(
        &mut self,
        ident: &Ident,
        mutability: Mutability,
        ty: &Ty,
        expr: &Option<Expr>,
    ) -> Result<()> {
        self.write("static ");
        if matches!(mutability, Mutability::Mut) {
            self.write("mut ");
        }
        self.write(&ident.name);
        self.write(": ");
        self.generate_ty(ty)?;
        if let Some(init_expr) = expr {
            self.write(" = ");
            self.generate_expr(init_expr)?;
        }
        self.write(";");
        self.writeln();
        Ok(())
    }

    /// Generate a const item
    fn generate_const_item(&mut self, ident: &Ident, ty: &Ty, expr: &Option<Expr>) -> Result<()> {
        self.write("const ");
        self.write(&ident.name);
        self.write(": ");
        self.generate_ty(ty)?;
        if let Some(init_expr) = expr {
            self.write(" = ");
            self.generate_expr(init_expr)?;
        }
        self.write(";");
        self.writeln();
        Ok(())
    }

    /// Generate a type alias item
    fn generate_type_alias_item(
        &mut self,
        ident: &Ident,
        generics: &Generics,
        ty: &Option<Ty>,
    ) -> Result<()> {
        self.write("type ");
        self.write(&ident.name);

        // Generic parameters (simplified for now)
        if !generics.params.is_empty() {
            self.write("</* generics */>");
        }

        if let Some(alias_ty) = ty {
            self.write(" = ");
            self.generate_ty(alias_ty)?;
        }
        self.write(";");
        self.writeln();
        Ok(())
    }

    /// Generate a module item
    fn generate_mod_item(&mut self, ident: &Ident, items: &Option<Vec<Item>>) -> Result<()> {
        self.write("mod ");
        self.write(&ident.name);

        match items {
            None => {
                // External module: mod foo;
                self.write(";");
                self.writeln();
            }
            Some(item_list) => {
                // Inline module: mod foo { ... }
                self.write(" {");
                self.writeln();
                self.indent();

                for item in item_list {
                    self.write_indent();
                    self.generate_item(item)?;
                }

                self.dedent();
                self.write_indent();
                self.write("}");
                self.writeln();
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fn_item(name: &str, params: Vec<Param>, ret_ty: FnRetTy, body: Option<Block>) -> Item {
        Item {
            attrs: Vec::new(),
            id: NodeId(0),
            span: Span::DUMMY,
            vis: Visibility::Inherited,
            ident: Ident::new(name, Span::DUMMY),
            kind: ItemKind::Fn(Box::new(Fn {
                defaultness: Defaultness::Final,
                sig: FnSig {
                    header: FnHeader {
                        safety: Safety::Safe,
                        coroutine_kind: None,
                        constness: Constness::NotConst,
                        ext: Extern::None,
                    },
                    decl: FnDecl { inputs: params, output: ret_ty },
                    span: Span::DUMMY,
                },
                generics: Generics::empty(),
                body,
            })),
            tokens: None,
        }
    }

    #[test]
    fn test_generate_simple_function() {
        let item = make_fn_item("foo", vec![], FnRetTy::Default(Span::DUMMY), None);

        let mut codegen = RustCodegen::new();
        codegen.generate_item(&item).unwrap();

        let output = codegen.output();
        assert_eq!(output.trim(), "fn foo();");
    }

    #[test]
    fn test_generate_pub_function() {
        let mut item = make_fn_item("bar", vec![], FnRetTy::Default(Span::DUMMY), None);
        item.vis = Visibility::Public;

        let mut codegen = RustCodegen::new();
        codegen.generate_item(&item).unwrap();

        let output = codegen.output();
        assert_eq!(output.trim(), "pub fn bar();");
    }

    #[test]
    fn test_generate_unsafe_function() {
        let mut item = make_fn_item("baz", vec![], FnRetTy::Default(Span::DUMMY), None);
        if let ItemKind::Fn(ref mut func) = item.kind {
            func.sig.header.safety = Safety::Unsafe;
        }

        let mut codegen = RustCodegen::new();
        codegen.generate_item(&item).unwrap();

        let output = codegen.output();
        assert_eq!(output.trim(), "unsafe fn baz();");
    }
}
