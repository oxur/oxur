use oxur_ast::ast::*;
use oxur_ast::builder::AstBuilder;
use oxur_ast::sexp::{Parser, SExp};
use std::path::PathBuf;

/// Helper function to parse a fixture file from test-data/fixtures/
fn parse_fixture(path: &str) -> SExp {
    let full_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/fixtures").join(path);
    Parser::parse_file(&full_path)
        .unwrap_or_else(|e| panic!("Failed to parse fixture {}: {}", path, e))
}

/// Helper to extract type from a Const item
fn extract_const_type(item: &Item) -> &Ty {
    match &item.kind {
        ItemKind::Const { ty, .. } => ty,
        _ => panic!("Expected Const item"),
    }
}

// ===== Type Item Tests (build_type_item) =====

#[test]
fn test_build_struct_item_unit() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "MyStruct")
          :kind (Struct Unit))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Struct(variant_data) => {
            assert!(matches!(variant_data, VariantData::Unit));
        }
        _ => panic!("Expected Struct item"),
    }
}

#[test]
fn test_build_struct_item_tuple() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "MyStruct")
          :kind (Struct (Tuple ((FieldDef
                                  :vis (Public)
                                  :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32")))))))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Struct(variant_data) => match variant_data {
            VariantData::Tuple(fields) => {
                assert_eq!(fields.len(), 1);
            }
            _ => panic!("Expected Tuple struct"),
        },
        _ => panic!("Expected Struct item"),
    }
}

#[test]
fn test_build_struct_item_named_fields() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "Point")
          :kind (Struct (Struct
                          :fields ((FieldDef
                                     :vis (Public)
                                     :ident (Ident :name "x")
                                     :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32")))))))
                                   (FieldDef
                                     :vis (Public)
                                     :ident (Ident :name "y")
                                     :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32")))))))))
                          :recovered false)))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Struct(variant_data) => match variant_data {
            VariantData::Struct { fields, recovered } => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].ident.as_ref().unwrap().name, "x");
                assert_eq!(fields[1].ident.as_ref().unwrap().name, "y");
                assert!(!recovered);
            }
            _ => panic!("Expected Struct with fields"),
        },
        _ => panic!("Expected Struct item"),
    }
}

#[test]
fn test_build_struct_item_recovered() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "BadStruct")
          :kind (Struct (Struct
                          :fields ()
                          :recovered true)))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Struct(variant_data) => match variant_data {
            VariantData::Struct { recovered, .. } => {
                assert!(recovered);
            }
            _ => panic!("Expected Struct with fields"),
        },
        _ => panic!("Expected Struct item"),
    }
}

#[test]
fn test_build_enum_item() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "Color")
          :kind (Enum (EnumDef
                        :variants ((Variant
                                     :vis (Public)
                                     :ident (Ident :name "Red")
                                     :data Unit)
                                   (Variant
                                     :vis (Public)
                                     :ident (Ident :name "Green")
                                     :data Unit)
                                   (Variant
                                     :vis (Public)
                                     :ident (Ident :name "Blue")
                                     :data Unit)))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Enum(enum_def) => {
            assert_eq!(enum_def.variants.len(), 3);
            assert_eq!(enum_def.variants[0].ident.name, "Red");
            assert_eq!(enum_def.variants[1].ident.name, "Green");
            assert_eq!(enum_def.variants[2].ident.name, "Blue");
        }
        _ => panic!("Expected Enum item"),
    }
}

#[test]
fn test_build_enum_item_with_tuple_variant() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "Option")
          :kind (Enum (EnumDef
                        :variants ((Variant
                                     :vis (Public)
                                     :ident (Ident :name "Some")
                                     :data (Tuple ((FieldDef
                                                     :vis (Public)
                                                     :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "T"))))))))))
                                   (Variant
                                     :vis (Public)
                                     :ident (Ident :name "None")
                                     :data Unit)))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Enum(enum_def) => {
            assert_eq!(enum_def.variants.len(), 2);
            assert!(matches!(enum_def.variants[0].data, VariantData::Tuple(_)));
            assert!(matches!(enum_def.variants[1].data, VariantData::Unit));
        }
        _ => panic!("Expected Enum item"),
    }
}

#[test]
fn test_build_enum_item_with_struct_variant() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "Message")
          :kind (Enum (EnumDef
                        :variants ((Variant
                                     :vis (Public)
                                     :ident (Ident :name "Point")
                                     :data (Struct
                                             :fields ((FieldDef
                                                        :vis (Public)
                                                        :ident (Ident :name "x")
                                                        :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32")))))))
                                                      (FieldDef
                                                        :vis (Public)
                                                        :ident (Ident :name "y")
                                                        :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32")))))))))
                                             :recovered false))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Enum(enum_def) => {
            assert_eq!(enum_def.variants.len(), 1);
            match &enum_def.variants[0].data {
                VariantData::Struct { fields, .. } => {
                    assert_eq!(fields.len(), 2);
                }
                _ => panic!("Expected Struct variant"),
            }
        }
        _ => panic!("Expected Enum item"),
    }
}

#[test]
fn test_build_enum_item_with_discriminant() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "Number")
          :kind (Enum (EnumDef
                        :variants ((Variant
                                     :vis (Public)
                                     :ident (Ident :name "Five")
                                     :data Unit
                                     :disr-expr (Expr :kind (MacCall (MacCall :path (Path :segments ((PathSegment :ident (Ident :name "five")))) :args Empty))))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Enum(enum_def) => {
            assert!(enum_def.variants[0].disr_expr.is_some());
        }
        _ => panic!("Expected Enum item"),
    }
}

// ===== Trait Item Tests (build_trait_item) =====

#[test]
fn test_build_trait_item_minimal() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "MyTrait")
          :kind (Trait (TraitDef
                         :safety Default
                         :generics (Generics)
                         :bounds ()
                         :items ())))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Trait(trait_def) => {
            assert!(matches!(trait_def.safety, Safety::Default));
            assert_eq!(trait_def.items.len(), 0);
            assert_eq!(trait_def.bounds.len(), 0);
        }
        _ => panic!("Expected Trait item"),
    }
}

#[test]
fn test_build_trait_item_safe() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "SafeTrait")
          :kind (Trait (TraitDef
                         :safety Safe
                         :generics (Generics)
                         :bounds ()
                         :items ())))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Trait(trait_def) => {
            assert!(matches!(trait_def.safety, Safety::Safe));
        }
        _ => panic!("Expected Trait item"),
    }
}

#[test]
fn test_build_trait_item_unsafe() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "UnsafeTrait")
          :kind (Trait (TraitDef
                         :safety Unsafe
                         :generics (Generics)
                         :bounds ()
                         :items ())))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Trait(trait_def) => {
            assert!(matches!(trait_def.safety, Safety::Unsafe));
        }
        _ => panic!("Expected Trait item"),
    }
}

#[test]
fn test_build_trait_item_with_bounds() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "BoundedTrait")
          :kind (Trait (TraitDef
                         :safety Default
                         :generics (Generics)
                         :bounds ((Trait
                                   (PolyTraitRef
                                     :trait-ref (TraitRef :path (Path :segments ((PathSegment :ident (Ident :name "Send")))))
                                     :bound-lifetimes ())
                                   None))
                         :items ())))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Trait(trait_def) => {
            assert_eq!(trait_def.bounds.len(), 1);
            match &trait_def.bounds[0] {
                GenericBound::Trait(poly_trait_ref, _modifier) => {
                    assert_eq!(poly_trait_ref.trait_ref.path.segments[0].ident.name, "Send");
                }
                _ => panic!("Expected Trait bound"),
            }
        }
        _ => panic!("Expected Trait item"),
    }
}

#[test]
fn test_build_trait_item_with_method() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "Drawable")
          :kind (Trait (TraitDef
                         :safety Default
                         :generics (Generics)
                         :bounds ()
                         :items ((AssocItem
                                   :vis (Public)
                                   :ident (Ident :name "draw")
                                   :kind (Fn (Fn
                                               :defaultness Final
                                               :sig (FnSig
                                                      :header (FnHeader :safety Default :constness NotConst)
                                                      :decl (FnDecl :inputs () :output (Default))
                                                      :span (Span :lo 0 :hi 0))
                                               :generics (Generics))))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Trait(trait_def) => {
            assert_eq!(trait_def.items.len(), 1);
            assert_eq!(trait_def.items[0].ident.name, "draw");
            assert!(matches!(trait_def.items[0].kind, AssocItemKind::Fn(_)));
        }
        _ => panic!("Expected Trait item"),
    }
}

#[test]
fn test_build_trait_item_with_type() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "Iterator")
          :kind (Trait (TraitDef
                         :safety Default
                         :generics (Generics)
                         :bounds ()
                         :items ((AssocItem
                                   :vis (Public)
                                   :ident (Ident :name "Item")
                                   :kind (Type nil))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Trait(trait_def) => {
            assert_eq!(trait_def.items.len(), 1);
            assert_eq!(trait_def.items[0].ident.name, "Item");
            match &trait_def.items[0].kind {
                AssocItemKind::Type(ty) => {
                    assert!(ty.is_none());
                }
                _ => panic!("Expected Type assoc item"),
            }
        }
        _ => panic!("Expected Trait item"),
    }
}

#[test]
fn test_build_trait_item_with_default_type() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "Container")
          :kind (Trait (TraitDef
                         :safety Default
                         :generics (Generics)
                         :bounds ()
                         :items ((AssocItem
                                   :vis (Public)
                                   :ident (Ident :name "Item")
                                   :kind (Type (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Trait(trait_def) => match &trait_def.items[0].kind {
            AssocItemKind::Type(ty) => {
                assert!(ty.is_some());
            }
            _ => panic!("Expected Type assoc item"),
        },
        _ => panic!("Expected Trait item"),
    }
}

#[test]
fn test_build_impl_item_minimal() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "impl_block")
          :kind (Impl (ImplDef
                        :safety Default
                        :generics (Generics)
                        :self-ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "MyType"))))))
                        :items ())))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Impl(impl_def) => {
            assert!(matches!(impl_def.safety, Safety::Default));
            assert!(impl_def.of_trait.is_none());
            assert_eq!(impl_def.items.len(), 0);
        }
        _ => panic!("Expected Impl item"),
    }
}

#[test]
fn test_build_impl_item_unsafe() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "impl_block")
          :kind (Impl (ImplDef
                        :safety Unsafe
                        :generics (Generics)
                        :self-ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "MyType"))))))
                        :items ())))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Impl(impl_def) => {
            assert!(matches!(impl_def.safety, Safety::Unsafe));
        }
        _ => panic!("Expected Impl item"),
    }
}

#[test]
fn test_build_impl_item_for_trait() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "impl_block")
          :kind (Impl (ImplDef
                        :safety Default
                        :generics (Generics)
                        :of-trait (TraitRef :path (Path :segments ((PathSegment :ident (Ident :name "Display")))))
                        :self-ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "MyType"))))))
                        :items ())))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Impl(impl_def) => {
            assert!(impl_def.of_trait.is_some());
            assert_eq!(impl_def.of_trait.unwrap().path.segments[0].ident.name, "Display");
        }
        _ => panic!("Expected Impl item"),
    }
}

#[test]
fn test_build_impl_item_with_methods() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "impl_block")
          :kind (Impl (ImplDef
                        :safety Default
                        :generics (Generics)
                        :self-ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "MyType"))))))
                        :items ((AssocItem
                                  :vis (Public)
                                  :ident (Ident :name "new")
                                  :kind (Fn (Fn
                                              :defaultness Final
                                              :sig (FnSig
                                                     :header (FnHeader :safety Default :constness NotConst)
                                                     :decl (FnDecl :inputs () :output (Default))
                                                     :span (Span :lo 0 :hi 0))
                                              :generics (Generics))))
                                (AssocItem
                                  :vis (Public)
                                  :ident (Ident :name "method")
                                  :kind (Fn (Fn
                                              :defaultness Final
                                              :sig (FnSig
                                                     :header (FnHeader :safety Default :constness NotConst)
                                                     :decl (FnDecl :inputs () :output (Default))
                                                     :span (Span :lo 0 :hi 0))
                                              :generics (Generics))))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Impl(impl_def) => {
            assert_eq!(impl_def.items.len(), 2);
            assert_eq!(impl_def.items[0].ident.name, "new");
            assert_eq!(impl_def.items[1].ident.name, "method");
        }
        _ => panic!("Expected Impl item"),
    }
}

#[test]
fn test_build_impl_item_missing_self_ty() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "impl_block")
          :kind (Impl (ImplDef
                        :safety Default
                        :generics (Generics)
                        :items ())))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_item(&sexp);

    assert!(result.is_err());
}

// ===== Declaration Item Tests (build_declaration_item) =====

#[test]
fn test_build_use_item_simple() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "use_item")
          :kind (Use (UseTree
                       :prefix (Path :segments ((PathSegment :ident (Ident :name "std"))
                                                (PathSegment :ident (Ident :name "io"))))
                       :kind (Simple nil))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Use(use_tree) => {
            assert_eq!(use_tree.prefix.segments.len(), 2);
            assert!(matches!(use_tree.kind, UseTreeKind::Simple(None)));
        }
        _ => panic!("Expected Use item"),
    }
}

#[test]
fn test_build_use_item_with_rename() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "use_item")
          :kind (Use (UseTree
                       :prefix (Path :segments ((PathSegment :ident (Ident :name "std"))
                                                (PathSegment :ident (Ident :name "io"))))
                       :kind (Simple (Ident :name "stdio")))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Use(use_tree) => match use_tree.kind {
            UseTreeKind::Simple(Some(ident)) => {
                assert_eq!(ident.name, "stdio");
            }
            _ => panic!("Expected Simple with rename"),
        },
        _ => panic!("Expected Use item"),
    }
}

#[test]
fn test_build_use_item_glob() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "use_item")
          :kind (Use (UseTree
                       :prefix (Path :segments ((PathSegment :ident (Ident :name "std"))
                                                (PathSegment :ident (Ident :name "io"))))
                       :kind Glob)))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Use(use_tree) => {
            assert!(matches!(use_tree.kind, UseTreeKind::Glob));
        }
        _ => panic!("Expected Use item"),
    }
}

#[test]
fn test_build_use_item_nested() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "use_item")
          :kind (Use (UseTree
                       :prefix (Path :segments ((PathSegment :ident (Ident :name "std"))))
                       :kind (Nested ((UseTree
                                        :prefix (Path :segments ((PathSegment :ident (Ident :name "io"))))
                                        :kind (Simple nil))
                                      (UseTree
                                        :prefix (Path :segments ((PathSegment :ident (Ident :name "fs"))))
                                        :kind (Simple nil)))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Use(use_tree) => match use_tree.kind {
            UseTreeKind::Nested(trees) => {
                assert_eq!(trees.len(), 2);
                assert_eq!(trees[0].prefix.segments[0].ident.name, "io");
                assert_eq!(trees[1].prefix.segments[0].ident.name, "fs");
            }
            _ => panic!("Expected Nested"),
        },
        _ => panic!("Expected Use item"),
    }
}

#[test]
fn test_build_static_item_immutable() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "STATIC_VAR")
          :kind (Static
                  :mutability Not
                  :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Static { mutability, ty: _, expr } => {
            assert!(matches!(mutability, Mutability::Not));
            assert!(expr.is_none());
        }
        _ => panic!("Expected Static item"),
    }
}

#[test]
fn test_build_static_item_mutable() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "STATIC_VAR")
          :kind (Static
                  :mutability Mut
                  :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Static { mutability, .. } => {
            assert!(matches!(mutability, Mutability::Mut));
        }
        _ => panic!("Expected Static item"),
    }
}

#[test]
fn test_build_static_item_with_expr() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "STATIC_VAR")
          :kind (Static
                  :mutability Not
                  :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))
                  :expr (Expr :kind (MacCall (MacCall :path (Path :segments ((PathSegment :ident (Ident :name "value")))) :args Empty)))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Static { expr, .. } => {
            assert!(expr.is_some());
        }
        _ => panic!("Expected Static item"),
    }
}

#[test]
fn test_build_static_item_missing_ty() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "STATIC_VAR")
          :kind (Static
                  :mutability Not))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_item(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_const_item() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "CONST_VAR")
          :kind (Const
                  :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Const { ty: _, expr } => {
            assert!(expr.is_none());
        }
        _ => panic!("Expected Const item"),
    }
}

#[test]
fn test_build_const_item_with_expr() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "CONST_VAR")
          :kind (Const
                  :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))
                  :expr (Expr :kind (MacCall (MacCall :path (Path :segments ((PathSegment :ident (Ident :name "value")))) :args Empty)))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Const { expr, .. } => {
            assert!(expr.is_some());
        }
        _ => panic!("Expected Const item"),
    }
}

#[test]
fn test_build_const_item_missing_ty() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "CONST_VAR")
          :kind (Const))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_item(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_ty_alias_item() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "MyType")
          :kind (TyAlias
                  :generics (Generics)))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::TyAlias { generics: _, ty } => {
            assert!(ty.is_none());
        }
        _ => panic!("Expected TyAlias item"),
    }
}

#[test]
fn test_build_ty_alias_item_with_ty() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "MyType")
          :kind (TyAlias
                  :generics (Generics)
                  :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::TyAlias { ty, .. } => {
            assert!(ty.is_some());
        }
        _ => panic!("Expected TyAlias item"),
    }
}

#[test]
fn test_build_mod_item_empty() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "my_mod")
          :kind (Mod
                  :items ()))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Mod { items } => {
            assert!(items.is_some());
            assert_eq!(items.unwrap().len(), 0);
        }
        _ => panic!("Expected Mod item"),
    }
}

#[test]
fn test_build_mod_item_with_items() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "my_mod")
          :kind (Mod
                  :items ((Item
                            :vis (Public)
                            :ident (Ident :name "inner_fn")
                            :kind (Fn (Fn
                                        :defaultness Final
                                        :sig (FnSig
                                               :header (FnHeader :safety Default :constness NotConst)
                                               :decl (FnDecl :inputs () :output (Default))
                                               :span (Span :lo 0 :hi 0))
                                        :generics (Generics)))))))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Mod { items } => {
            assert!(items.is_some());
            let items = items.unwrap();
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].ident.name, "inner_fn");
        }
        _ => panic!("Expected Mod item"),
    }
}

#[test]
fn test_build_mod_item_external() {
    let input = r#"
        (Item
          :vis (Public)
          :ident (Ident :name "my_mod")
          :kind (Mod))
    "#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Mod { items } => {
            assert!(items.is_none());
        }
        _ => panic!("Expected Mod item"),
    }
}

// ===== Type Kind Tests (using fixtures) =====

#[test]
fn test_build_ty_kind_never() {
    let sexp = parse_fixture("ty/never.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    assert!(matches!(ty.kind, TyKind::Never));
}

#[test]
fn test_build_ty_kind_infer() {
    let sexp = parse_fixture("ty/infer.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    assert!(matches!(ty.kind, TyKind::Infer));
}

#[test]
fn test_build_ty_kind_ref_immut() {
    let sexp = parse_fixture("ty/ref-immut.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    match &ty.kind {
        TyKind::Ref { lifetime, mutability, .. } => {
            assert!(lifetime.is_none());
            assert!(matches!(mutability, Mutability::Not));
        }
        _ => panic!("Expected Ref type"),
    }
}

#[test]
fn test_build_ty_kind_ref_mut() {
    let sexp = parse_fixture("ty/ref-mut.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    match &ty.kind {
        TyKind::Ref { mutability, .. } => {
            assert!(matches!(mutability, Mutability::Mut));
        }
        _ => panic!("Expected Ref type"),
    }
}

#[test]
fn test_build_ty_kind_ref_with_lifetime() {
    let sexp = parse_fixture("ty/ref-with-lifetime.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    match &ty.kind {
        TyKind::Ref { lifetime, .. } => {
            assert!(lifetime.is_some());
            assert_eq!(lifetime.as_ref().unwrap().ident.name, "a");
        }
        _ => panic!("Expected Ref type"),
    }
}

#[test]
fn test_build_ty_kind_ptr_const() {
    let sexp = parse_fixture("ty/ptr-const.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    match &ty.kind {
        TyKind::Ptr { mutability, .. } => {
            assert!(matches!(mutability, Mutability::Not));
        }
        _ => panic!("Expected Ptr type"),
    }
}

#[test]
fn test_build_ty_kind_ptr_mut() {
    let sexp = parse_fixture("ty/ptr-mut.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    match &ty.kind {
        TyKind::Ptr { mutability, .. } => {
            assert!(matches!(mutability, Mutability::Mut));
        }
        _ => panic!("Expected Ptr type"),
    }
}

#[test]
fn test_build_ty_kind_array() {
    let sexp = parse_fixture("ty/array.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    assert!(matches!(ty.kind, TyKind::Array { .. }));
}

#[test]
fn test_build_ty_kind_slice() {
    let sexp = parse_fixture("ty/slice.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    assert!(matches!(ty.kind, TyKind::Slice(_)));
}

#[test]
fn test_build_ty_kind_tuple() {
    let sexp = parse_fixture("ty/tuple.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    match &ty.kind {
        TyKind::Tuple(tys) => {
            assert_eq!(tys.len(), 2);
        }
        _ => panic!("Expected Tuple type"),
    }
}

#[test]
fn test_build_ty_kind_bare_fn() {
    let sexp = parse_fixture("ty/bare-fn.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    match &ty.kind {
        TyKind::BareFn { safety, abi, inputs, .. } => {
            assert!(matches!(safety, Safety::Default));
            assert!(abi.is_none());
            assert!(inputs.is_empty());
        }
        _ => panic!("Expected BareFn type"),
    }
}

#[test]
fn test_build_ty_kind_bare_fn_unsafe() {
    let sexp = parse_fixture("ty/bare-fn-unsafe.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    match &ty.kind {
        TyKind::BareFn { safety, .. } => {
            assert!(matches!(safety, Safety::Unsafe));
        }
        _ => panic!("Expected BareFn type"),
    }
}

#[test]
fn test_build_ty_kind_bare_fn_abi() {
    let sexp = parse_fixture("ty/bare-fn-abi.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    match &ty.kind {
        TyKind::BareFn { abi, .. } => {
            assert_eq!(abi.as_ref().unwrap(), "C");
        }
        _ => panic!("Expected BareFn type"),
    }
}

#[test]
fn test_build_ty_kind_bare_fn_with_params() {
    let sexp = parse_fixture("ty/bare-fn-with-params.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    match &ty.kind {
        TyKind::BareFn { inputs, .. } => {
            assert_eq!(inputs.len(), 1);
            assert!(inputs[0].name.is_none());
        }
        _ => panic!("Expected BareFn type"),
    }
}

#[test]
fn test_build_ty_kind_bare_fn_named_param() {
    let sexp = parse_fixture("ty/bare-fn-named-param.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    match &ty.kind {
        TyKind::BareFn { inputs, .. } => {
            assert!(inputs[0].name.is_some());
            assert_eq!(inputs[0].name.as_ref().unwrap().name, "x");
        }
        _ => panic!("Expected BareFn type"),
    }
}

#[test]
fn test_build_ty_kind_impl_trait() {
    let sexp = parse_fixture("ty/impl-trait.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    match &ty.kind {
        TyKind::ImplTrait(bounds) => {
            assert_eq!(bounds.len(), 1);
        }
        _ => panic!("Expected ImplTrait type"),
    }
}

#[test]
fn test_build_ty_kind_trait_object() {
    let sexp = parse_fixture("ty/trait-object.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    match &ty.kind {
        TyKind::TraitObject { bounds, syntax } => {
            assert_eq!(bounds.len(), 1);
            assert!(matches!(syntax, TraitObjectSyntax::Dyn));
        }
        _ => panic!("Expected TraitObject type"),
    }
}

#[test]
fn test_build_ty_kind_trait_object_no_dyn() {
    let sexp = parse_fixture("ty/trait-object-no-dyn.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    match &ty.kind {
        TyKind::TraitObject { syntax, .. } => {
            assert!(matches!(syntax, TraitObjectSyntax::None));
        }
        _ => panic!("Expected TraitObject type"),
    }
}

#[test]
fn test_build_ty_kind_mac_call() {
    let sexp = parse_fixture("ty/mac-call.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    assert!(matches!(ty.kind, TyKind::MacCall(_)));
}

#[test]
fn test_build_ty_kind_paren() {
    let sexp = parse_fixture("ty/paren.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    assert!(matches!(ty.kind, TyKind::Paren(_)));
}

#[test]
fn test_build_ty_with_id_and_span() {
    let sexp = parse_fixture("ty/with-id-and-span.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();
    let ty = extract_const_type(&item);

    assert_eq!(ty.id, NodeId(99));
    assert_eq!(ty.span.lo, 5);
    assert_eq!(ty.span.hi, 10);
}

// ===== Type Kind Error Tests =====

#[test]
fn test_build_ty_kind_invalid() {
    let sexp = parse_fixture("ty/invalid-kind.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_item(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_ty_kind_invalid_symbol() {
    let sexp = parse_fixture("ty/invalid-symbol.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_item(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_ty_trait_object_invalid_syntax() {
    let sexp = parse_fixture("ty/trait-object-invalid-syntax.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_item(&sexp);

    assert!(result.is_err());
}
