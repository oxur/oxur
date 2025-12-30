//! Type and pattern code generation
//!
//! Generates Rust code for types and patterns

use crate::ast::*;
use crate::codegen::RustCodegen;
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
            // Stage 6: Advanced patterns (TODO - implement)
            PatKind::Wild => {
                self.write("_");
            }
            PatKind::Struct { .. } => {
                self.write("/* struct pattern */");
            }
            PatKind::TupleStruct { .. } => {
                self.write("/* tuple struct pattern */");
            }
            PatKind::Tuple(..) => {
                self.write("/* tuple pattern */");
            }
            PatKind::Slice(..) => {
                self.write("/* slice pattern */");
            }
            PatKind::Or(..) => {
                self.write("/* or pattern */");
            }
            PatKind::Ref { .. } => {
                self.write("/* ref pattern */");
            }
            PatKind::Lit(..) => {
                self.write("/* lit pattern */");
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
            // Stage 6: Advanced types (TODO - implement)
            TyKind::Ref { .. } => {
                self.write("/* ref type */");
            }
            TyKind::Ptr { .. } => {
                self.write("/* ptr type */");
            }
            TyKind::Array { .. } => {
                self.write("/* array type */");
            }
            TyKind::Slice(..) => {
                self.write("/* slice type */");
            }
            TyKind::Tuple(..) => {
                self.write("/* tuple type */");
            }
            TyKind::Never => {
                self.write("!");
            }
            TyKind::Infer => {
                self.write("_");
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
