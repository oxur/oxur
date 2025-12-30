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

        // Generics (skip for Phase 1 - no params yet)
        if !func.generics.params.is_empty() {
            self.write("<");
            // TODO: Phase 2+ will generate generic params
            self.write(">");
        }

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
        let ItemKind::Fn(ref mut func) = item.kind;
        func.sig.header.safety = Safety::Unsafe;

        let mut codegen = RustCodegen::new();
        codegen.generate_item(&item).unwrap();

        let output = codegen.output();
        assert_eq!(output.trim(), "unsafe fn baz();");
    }
}
