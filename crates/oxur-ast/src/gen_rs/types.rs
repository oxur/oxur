//! Type and pattern code generation
//!
//! Generates Rust code for types and patterns

use crate::ast::*;
use crate::gen_rs::RustCodegen;
use anyhow::Result;

impl RustCodegen {
    /// Generate a pattern
    pub(crate) fn generate_pat(&mut self, pat: &Pat) -> Result<()> {
        match &pat.kind {
            PatKind::Ident { binding_mode, ident, sub } => {
                // Binding mode (ref, mut, ref mut)
                match binding_mode {
                    BindingMode::ByRef(Mutability::Not) => self.write("ref "),
                    BindingMode::ByRef(Mutability::Mut) => self.write("ref mut "),
                    BindingMode::ByValue(Mutability::Mut) => self.write("mut "),
                    BindingMode::ByValue(Mutability::Not) => {}
                }

                // Identifier name
                self.write(&ident.name);

                // Sub-pattern (e.g., x @ Some(_))
                if let Some(sub_pat) = sub {
                    self.write(" @ ");
                    self.generate_pat(sub_pat)?;
                }
            }
            // Stage 6: Advanced patterns
            PatKind::Wild => {
                self.write("_");
            }
            PatKind::Struct { path, fields } => {
                self.generate_path(path)?;
                self.write(" { ");
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(&field.ident.name);
                    if !field.is_shorthand {
                        self.write(": ");
                        self.generate_pat(&field.pat)?;
                    }
                }
                self.write(" }");
            }
            PatKind::TupleStruct { path, elems } => {
                self.generate_path(path)?;
                self.write("(");
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_pat(elem)?;
                }
                self.write(")");
            }
            PatKind::Tuple(pats) => {
                self.write("(");
                for (i, pat) in pats.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_pat(pat)?;
                }
                self.write(")");
            }
            PatKind::Slice(pats) => {
                self.write("[");
                for (i, pat) in pats.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_pat(pat)?;
                }
                self.write("]");
            }
            PatKind::Or(pats) => {
                for (i, pat) in pats.iter().enumerate() {
                    if i > 0 {
                        self.write(" | ");
                    }
                    self.generate_pat(pat)?;
                }
            }
            PatKind::Ref { pat, mutability } => {
                self.write("&");
                if matches!(mutability, Mutability::Mut) {
                    self.write("mut ");
                }
                self.generate_pat(pat)?;
            }
            PatKind::Lit(expr) => {
                self.generate_expr(expr)?;
            }
            // Phase 5: Complete pattern coverage
            PatKind::Box(pat) => {
                self.write("box ");
                self.generate_pat(pat)?;
            }
            PatKind::Path { qself, path } => {
                if let Some(_qself) = qself {
                    // TODO: Phase 5+ will generate full qualified paths
                    self.write("/* <qualified> */ ");
                }
                self.generate_path(path)?;
            }
            PatKind::Range { start, end, limits } => {
                if let Some(start_expr) = start {
                    self.generate_expr(start_expr)?;
                }
                match limits {
                    RangeEnd::Included => self.write("..="),
                    RangeEnd::Excluded => self.write(".."),
                }
                if let Some(end_expr) = end {
                    self.generate_expr(end_expr)?;
                }
            }
            PatKind::Rest => {
                self.write("..");
            }
            PatKind::Paren(pat) => {
                self.write("(");
                self.generate_pat(pat)?;
                self.write(")");
            }
            PatKind::Type { pat, ty } => {
                self.generate_pat(pat)?;
                self.write(": ");
                self.generate_ty(ty)?;
            }
            PatKind::Const(const_block) => {
                self.write("const { ");
                self.generate_expr(&const_block.value)?;
                self.write(" }");
            }
            PatKind::MacCall(mac) => {
                self.generate_mac_call(mac)?;
            }
            PatKind::Err => {
                self.write("/* error */");
            }
        }
        Ok(())
    }

    /// Generate a type
    pub(crate) fn generate_ty(&mut self, ty: &Ty) -> Result<()> {
        match &ty.kind {
            TyKind::Path(qself, path) => {
                if qself.is_some() {
                    // TODO: Phase 2+ will handle qualified types like <T as Trait>::Assoc
                    self.write("/* qualified type */");
                }
                self.generate_path(path)?;
            }
            // Stage 6: Advanced types
            TyKind::Ref { lifetime, mutability, ty } => {
                self.write("&");
                if let Some(lt) = lifetime {
                    self.generate_lifetime(lt)?;
                    self.write(" ");
                }
                if matches!(mutability, Mutability::Mut) {
                    self.write("mut ");
                }
                self.generate_ty(ty)?;
            }
            TyKind::Ptr { mutability, ty } => {
                self.write("*");
                if matches!(mutability, Mutability::Mut) {
                    self.write("mut ");
                } else {
                    self.write("const ");
                }
                self.generate_ty(ty)?;
            }
            TyKind::Array { ty, len } => {
                self.write("[");
                self.generate_ty(ty)?;
                self.write("; ");
                self.generate_expr(len)?;
                self.write("]");
            }
            TyKind::Slice(ty) => {
                self.write("[");
                self.generate_ty(ty)?;
                self.write("]");
            }
            TyKind::Tuple(tys) => {
                self.write("(");
                for (i, ty) in tys.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_ty(ty)?;
                }
                self.write(")");
            }
            TyKind::Never => {
                self.write("!");
            }
            TyKind::Infer => {
                self.write("_");
            }
            // Phase 5: Complete type coverage
            TyKind::BareFn { safety, abi, inputs, output } => {
                // Safety keyword
                if safety == &Safety::Unsafe {
                    self.write("unsafe ");
                }

                // ABI
                if let Some(abi_str) = abi {
                    self.write("extern ");
                    self.write("\"");
                    self.write(abi_str);
                    self.write("\" ");
                }

                self.write("fn(");
                for (i, param) in inputs.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    if let Some(name) = &param.name {
                        self.write(&name.name);
                        self.write(": ");
                    }
                    self.generate_ty(&param.ty)?;
                }
                self.write(")");

                // Return type
                match output {
                    FnRetTy::Default(_) => {}
                    FnRetTy::Ty(ty) => {
                        self.write(" -> ");
                        self.generate_ty(ty)?;
                    }
                }
            }
            TyKind::ImplTrait(bounds) => {
                self.write("impl ");
                for (i, bound) in bounds.iter().enumerate() {
                    if i > 0 {
                        self.write(" + ");
                    }
                    self.generate_generic_bound(bound)?;
                }
            }
            TyKind::TraitObject { bounds, syntax } => {
                if matches!(syntax, TraitObjectSyntax::Dyn) {
                    self.write("dyn ");
                }
                for (i, bound) in bounds.iter().enumerate() {
                    if i > 0 {
                        self.write(" + ");
                    }
                    self.generate_generic_bound(bound)?;
                }
            }
            TyKind::MacCall(mac) => {
                self.generate_mac_call(mac)?;
            }
            TyKind::Paren(ty) => {
                self.write("(");
                self.generate_ty(ty)?;
                self.write(")");
            }
        }
        Ok(())
    }

    /// Generate a path
    pub(crate) fn generate_path(&mut self, path: &Path) -> Result<()> {
        for (i, segment) in path.segments.iter().enumerate() {
            if i > 0 {
                self.write("::");
            }
            self.write(&segment.ident.name);

            // Generic arguments (skip for Phase 1)
            if segment.args.is_some() {
                // TODO: Phase 2+ will generate generic arguments
                self.write("</* generics */>");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_simple_ident_pattern() {
        let pat = Pat {
            id: NodeId(0),
            kind: PatKind::Ident {
                binding_mode: BindingMode::ByValue(Mutability::Not),
                ident: Ident::new("x", Span::DUMMY),
                sub: None,
            },
            span: Span::DUMMY,
            tokens: None,
        };

        let mut codegen = RustCodegen::new();
        codegen.generate_pat(&pat).unwrap();

        assert_eq!(codegen.output(), "x");
    }

    #[test]
    fn test_generate_mut_pattern() {
        let pat = Pat {
            id: NodeId(0),
            kind: PatKind::Ident {
                binding_mode: BindingMode::ByValue(Mutability::Mut),
                ident: Ident::new("y", Span::DUMMY),
                sub: None,
            },
            span: Span::DUMMY,
            tokens: None,
        };

        let mut codegen = RustCodegen::new();
        codegen.generate_pat(&pat).unwrap();

        assert_eq!(codegen.output(), "mut y");
    }

    #[test]
    fn test_generate_ref_pattern() {
        let pat = Pat {
            id: NodeId(0),
            kind: PatKind::Ident {
                binding_mode: BindingMode::ByRef(Mutability::Not),
                ident: Ident::new("z", Span::DUMMY),
                sub: None,
            },
            span: Span::DUMMY,
            tokens: None,
        };

        let mut codegen = RustCodegen::new();
        codegen.generate_pat(&pat).unwrap();

        assert_eq!(codegen.output(), "ref z");
    }

    #[test]
    fn test_generate_simple_path_type() {
        let ty = Ty {
            id: NodeId(0),
            kind: TyKind::Path(None, Path::from_ident(Ident::new("i32", Span::DUMMY))),
            span: Span::DUMMY,
            tokens: None,
        };

        let mut codegen = RustCodegen::new();
        codegen.generate_ty(&ty).unwrap();

        assert_eq!(codegen.output(), "i32");
    }

    #[test]
    fn test_generate_multi_segment_path() {
        let path = Path {
            span: Span::DUMMY,
            segments: vec![
                PathSegment::from_ident(Ident::new("std", Span::DUMMY)),
                PathSegment::from_ident(Ident::new("io", Span::DUMMY)),
                PathSegment::from_ident(Ident::new("Result", Span::DUMMY)),
            ],
            tokens: None,
        };

        let mut codegen = RustCodegen::new();
        codegen.generate_path(&path).unwrap();

        assert_eq!(codegen.output(), "std::io::Result");
    }
}
