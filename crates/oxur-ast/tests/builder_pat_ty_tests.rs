use oxur_ast::ast::*;
use oxur_ast::builder::AstBuilder;
use oxur_ast::sexp::Parser;

// ===== Pattern Building Tests =====

#[test]
fn test_build_pat_wild() {
    let input = r#"
        (Pat :kind Wild)
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    assert!(matches!(pat.kind, PatKind::Wild));
}

#[test]
fn test_build_pat_wild_symbol() {
    let input = r#"
        (Pat :kind Wild)
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    assert!(matches!(pat.kind, PatKind::Wild));
}

#[test]
fn test_build_pat_ident_simple() {
    let input = r#"
        (Pat :kind (Ident
                     :ident (Ident :name "x")))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Ident { binding_mode, ident, sub } => {
            assert_eq!(ident.name, "x");
            assert!(matches!(binding_mode, BindingMode::ByValue(Mutability::Not)));
            assert!(sub.is_none());
        }
        _ => panic!("Expected Ident pattern"),
    }
}

#[test]
fn test_build_pat_ident_with_binding_mode_by_ref() {
    let input = r#"
        (Pat :kind (Ident
                     :ident (Ident :name "x")
                     :binding-mode (ByRef Not)))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Ident { binding_mode, .. } => {
            assert!(matches!(binding_mode, BindingMode::ByRef(Mutability::Not)));
        }
        _ => panic!("Expected Ident pattern"),
    }
}

#[test]
fn test_build_pat_ident_with_binding_mode_by_ref_mut() {
    let input = r#"
        (Pat :kind (Ident
                     :ident (Ident :name "x")
                     :binding-mode (ByRef Mut)))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Ident { binding_mode, .. } => {
            assert!(matches!(binding_mode, BindingMode::ByRef(Mutability::Mut)));
        }
        _ => panic!("Expected Ident pattern"),
    }
}

#[test]
fn test_build_pat_ident_with_binding_mode_by_value_mut() {
    let input = r#"
        (Pat :kind (Ident
                     :ident (Ident :name "x")
                     :binding-mode (ByValue Mut)))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Ident { binding_mode, .. } => {
            assert!(matches!(binding_mode, BindingMode::ByValue(Mutability::Mut)));
        }
        _ => panic!("Expected Ident pattern"),
    }
}

#[test]
fn test_build_pat_ident_with_sub_pattern() {
    let input = r#"
        (Pat :kind (Ident
                     :ident (Ident :name "x")
                     :sub (Pat :kind Wild)))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Ident { sub, .. } => {
            assert!(sub.is_some());
            assert!(matches!(sub.unwrap().kind, PatKind::Wild));
        }
        _ => panic!("Expected Ident pattern"),
    }
}

#[test]
fn test_build_pat_struct() {
    let input = r#"
        (Pat :kind (Struct
                     :path (Path :segments ((PathSegment :ident (Ident :name "Point"))))
                     :fields ((PatField
                                :ident (Ident :name "x")
                                :pat (Pat :kind (Ident :ident (Ident :name "a")))
                                :is-shorthand false)
                              (PatField
                                :ident (Ident :name "y")
                                :pat (Pat :kind (Ident :ident (Ident :name "b")))
                                :is-shorthand false))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Struct { path, fields } => {
            assert_eq!(path.segments.len(), 1);
            assert_eq!(path.segments[0].ident.name, "Point");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].ident.name, "x");
            assert_eq!(fields[1].ident.name, "y");
            assert!(!fields[0].is_shorthand);
            assert!(!fields[1].is_shorthand);
        }
        _ => panic!("Expected Struct pattern"),
    }
}

#[test]
fn test_build_pat_struct_shorthand() {
    let input = r#"
        (Pat :kind (Struct
                     :path (Path :segments ((PathSegment :ident (Ident :name "Point"))))
                     :fields ((PatField
                                :ident (Ident :name "x")
                                :pat (Pat :kind (Ident :ident (Ident :name "x")))
                                :is-shorthand true))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Struct { fields, .. } => {
            assert!(fields[0].is_shorthand);
        }
        _ => panic!("Expected Struct pattern"),
    }
}

#[test]
fn test_build_pat_struct_no_fields() {
    let input = r#"
        (Pat :kind (Struct
                     :path (Path :segments ((PathSegment :ident (Ident :name "Unit"))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Struct { fields, .. } => {
            assert_eq!(fields.len(), 0);
        }
        _ => panic!("Expected Struct pattern"),
    }
}

#[test]
fn test_build_pat_tuple_struct() {
    let input = r#"
        (Pat :kind (TupleStruct
                     :path (Path :segments ((PathSegment :ident (Ident :name "Some"))))
                     :elems ((Pat :kind (Ident :ident (Ident :name "x"))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::TupleStruct { path, elems } => {
            assert_eq!(path.segments.len(), 1);
            assert_eq!(path.segments[0].ident.name, "Some");
            assert_eq!(elems.len(), 1);
        }
        _ => panic!("Expected TupleStruct pattern"),
    }
}

#[test]
fn test_build_pat_tuple_struct_no_elems() {
    let input = r#"
        (Pat :kind (TupleStruct
                     :path (Path :segments ((PathSegment :ident (Ident :name "None"))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::TupleStruct { elems, .. } => {
            assert_eq!(elems.len(), 0);
        }
        _ => panic!("Expected TupleStruct pattern"),
    }
}

#[test]
fn test_build_pat_tuple() {
    let input = r#"
        (Pat :kind (Tuple ((Pat :kind (Ident :ident (Ident :name "x")))
                           (Pat :kind (Ident :ident (Ident :name "y"))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Tuple(pats) => {
            assert_eq!(pats.len(), 2);
        }
        _ => panic!("Expected Tuple pattern"),
    }
}

#[test]
fn test_build_pat_tuple_empty() {
    let input = r#"
        (Pat :kind (Tuple ()))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Tuple(pats) => {
            assert_eq!(pats.len(), 0);
        }
        _ => panic!("Expected Tuple pattern"),
    }
}

#[test]
fn test_build_pat_slice() {
    let input = r#"
        (Pat :kind (Slice ((Pat :kind (Ident :ident (Ident :name "first")))
                           (Pat :kind (Ident :ident (Ident :name "second"))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Slice(pats) => {
            assert_eq!(pats.len(), 2);
        }
        _ => panic!("Expected Slice pattern"),
    }
}

#[test]
fn test_build_pat_or() {
    let input = r#"
        (Pat :kind (Or ((Pat :kind (Ident :ident (Ident :name "a")))
                        (Pat :kind (Ident :ident (Ident :name "b")))
                        (Pat :kind (Ident :ident (Ident :name "c"))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Or(pats) => {
            assert_eq!(pats.len(), 3);
        }
        _ => panic!("Expected Or pattern"),
    }
}

#[test]
fn test_build_pat_ref_immutable() {
    let input = r#"
        (Pat :kind (Ref
                     :pat (Pat :kind (Ident :ident (Ident :name "x")))
                     :mutability Not))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Ref { pat: _, mutability } => {
            assert!(matches!(mutability, Mutability::Not));
        }
        _ => panic!("Expected Ref pattern"),
    }
}

#[test]
fn test_build_pat_ref_mutable() {
    let input = r#"
        (Pat :kind (Ref
                     :pat (Pat :kind (Ident :ident (Ident :name "x")))
                     :mutability Mut))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Ref { mutability, .. } => {
            assert!(matches!(mutability, Mutability::Mut));
        }
        _ => panic!("Expected Ref pattern"),
    }
}

#[test]
fn test_build_pat_ref_default_mutability() {
    let input = r#"
        (Pat :kind (Ref
                     :pat (Pat :kind (Ident :ident (Ident :name "x")))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Ref { mutability, .. } => {
            assert!(matches!(mutability, Mutability::Not));
        }
        _ => panic!("Expected Ref pattern"),
    }
}

#[test]
fn test_build_pat_lit() {
    let input = r#"
        (Pat :kind (Lit (Expr :kind (MacCall (MacCall :path (Path :segments ((PathSegment :ident (Ident :name "value")))) :args Empty)))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Lit(_) => {
            // Success
        }
        _ => panic!("Expected Lit pattern"),
    }
}

#[test]
fn test_build_pat_missing_kind() {
    let input = r#"
        (Pat)
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_pat(&sexp);

    assert!(result.is_err());
}

// ===== Type Building Tests (via Item building since build_ty is private) =====

#[test]
fn test_build_ty_never_via_static() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "VAR")
          :kind (Static
                  :mutability Not
                  :ty (Ty :kind Never)))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Static { ty, .. } => {
            assert!(matches!(ty.kind, TyKind::Never));
        }
        _ => panic!("Expected Static item"),
    }
}

#[test]
fn test_build_ty_infer_via_static() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "VAR")
          :kind (Static
                  :mutability Not
                  :ty (Ty :kind Infer)))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Static { ty, .. } => {
            assert!(matches!(ty.kind, TyKind::Infer));
        }
        _ => panic!("Expected Static item"),
    }
}

#[test]
fn test_build_ty_path_via_static() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "VAR")
          :kind (Static
                  :mutability Not
                  :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Static { ty, .. } => match ty.kind {
            TyKind::Path(qself, path) => {
                assert!(qself.is_none());
                assert_eq!(path.segments.len(), 1);
                assert_eq!(path.segments[0].ident.name, "i32");
            }
            _ => panic!("Expected Path type"),
        },
        _ => panic!("Expected Static item"),
    }
}

#[test]
fn test_build_ty_path_multiple_segments_via_static() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "VAR")
          :kind (Static
                  :mutability Not
                  :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "std"))
                                                            (PathSegment :ident (Ident :name "vec"))
                                                            (PathSegment :ident (Ident :name "Vec"))))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Static { ty, .. } => match ty.kind {
            TyKind::Path(_, path) => {
                assert_eq!(path.segments.len(), 3);
                assert_eq!(path.segments[0].ident.name, "std");
                assert_eq!(path.segments[1].ident.name, "vec");
                assert_eq!(path.segments[2].ident.name, "Vec");
            }
            _ => panic!("Expected Path type"),
        },
        _ => panic!("Expected Static item"),
    }
}

#[test]
fn test_build_ty_ref_immutable_via_static() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "VAR")
          :kind (Static
                  :mutability Not
                  :ty (Ty :kind (Ref
                                  :mutability Not
                                  :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "str"))))))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Static { ty, .. } => match ty.kind {
            TyKind::Ref { lifetime, mutability, ty: _ } => {
                assert!(lifetime.is_none());
                assert!(matches!(mutability, Mutability::Not));
            }
            _ => panic!("Expected Ref type"),
        },
        _ => panic!("Expected Static item"),
    }
}

#[test]
fn test_build_ty_ref_mutable_via_static() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "VAR")
          :kind (Static
                  :mutability Not
                  :ty (Ty :kind (Ref
                                  :mutability Mut
                                  :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "str"))))))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Static { ty, .. } => match ty.kind {
            TyKind::Ref { mutability, .. } => {
                assert!(matches!(mutability, Mutability::Mut));
            }
            _ => panic!("Expected Ref type"),
        },
        _ => panic!("Expected Static item"),
    }
}

#[test]
fn test_build_ty_ref_with_lifetime_via_static() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "VAR")
          :kind (Static
                  :mutability Not
                  :ty (Ty :kind (Ref
                                  :lifetime (Lifetime :ident (Ident :name "a"))
                                  :mutability Not
                                  :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "str"))))))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Static { ty, .. } => match ty.kind {
            TyKind::Ref { lifetime, .. } => {
                assert!(lifetime.is_some());
                assert_eq!(lifetime.unwrap().ident.name, "a");
            }
            _ => panic!("Expected Ref type"),
        },
        _ => panic!("Expected Static item"),
    }
}

#[test]
fn test_build_ty_ptr_immutable_via_static() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "VAR")
          :kind (Static
                  :mutability Not
                  :ty (Ty :kind (Ptr
                                  :mutability Not
                                  :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Static { ty, .. } => match ty.kind {
            TyKind::Ptr { mutability, ty: _ } => {
                assert!(matches!(mutability, Mutability::Not));
            }
            _ => panic!("Expected Ptr type"),
        },
        _ => panic!("Expected Static item"),
    }
}

#[test]
fn test_build_ty_ptr_mutable_via_static() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "VAR")
          :kind (Static
                  :mutability Not
                  :ty (Ty :kind (Ptr
                                  :mutability Mut
                                  :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Static { ty, .. } => match ty.kind {
            TyKind::Ptr { mutability, .. } => {
                assert!(matches!(mutability, Mutability::Mut));
            }
            _ => panic!("Expected Ptr type"),
        },
        _ => panic!("Expected Static item"),
    }
}

#[test]
fn test_build_ty_array_via_static() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "VAR")
          :kind (Static
                  :mutability Not
                  :ty (Ty :kind (Array
                                  :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))
                                  :len (Expr :kind (MacCall (MacCall :path (Path :segments ((PathSegment :ident (Ident :name "n")))) :args Empty)))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Static { ty, .. } => match ty.kind {
            TyKind::Array { ty: _, len: _ } => {
                // Success
            }
            _ => panic!("Expected Array type"),
        },
        _ => panic!("Expected Static item"),
    }
}

#[test]
fn test_build_ty_slice_via_static() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "VAR")
          :kind (Static
                  :mutability Not
                  :ty (Ty :kind (Slice (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "u8"))))))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Static { ty, .. } => match ty.kind {
            TyKind::Slice(_) => {
                // Success
            }
            _ => panic!("Expected Slice type"),
        },
        _ => panic!("Expected Static item"),
    }
}

#[test]
fn test_build_ty_tuple_empty_via_static() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "VAR")
          :kind (Static
                  :mutability Not
                  :ty (Ty :kind (Tuple))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Static { ty, .. } => match ty.kind {
            TyKind::Tuple(tys) => {
                assert_eq!(tys.len(), 0);
            }
            _ => panic!("Expected Tuple type"),
        },
        _ => panic!("Expected Static item"),
    }
}

#[test]
fn test_build_ty_tuple_single_via_static() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "VAR")
          :kind (Static
                  :mutability Not
                  :ty (Ty :kind (Tuple (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Static { ty, .. } => match ty.kind {
            TyKind::Tuple(tys) => {
                assert_eq!(tys.len(), 1);
            }
            _ => panic!("Expected Tuple type"),
        },
        _ => panic!("Expected Static item"),
    }
}

#[test]
fn test_build_ty_tuple_multiple_via_static() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "VAR")
          :kind (Static
                  :mutability Not
                  :ty (Ty :kind (Tuple (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))
                                       (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "String"))))))
                                       (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "bool"))))))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Static { ty, .. } => match ty.kind {
            TyKind::Tuple(tys) => {
                assert_eq!(tys.len(), 3);
            }
            _ => panic!("Expected Tuple type"),
        },
        _ => panic!("Expected Static item"),
    }
}
