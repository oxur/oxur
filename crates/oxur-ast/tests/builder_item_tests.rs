use oxur_ast::ast::*;
use oxur_ast::builder::AstBuilder;
use oxur_ast::sexp::{Parser, SExp};
use std::path::PathBuf;

/// Helper function to parse a fixture file from test-data/fixtures/
#[allow(dead_code)]
fn parse_fixture(path: &str) -> SExp {
    let full_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/fixtures").join(path);
    Parser::parse_file(&full_path)
        .unwrap_or_else(|e| panic!("Failed to parse fixture {}: {}", path, e))
}

// ===== Visibility Building Tests =====

#[test]
fn test_build_visibility_public() {
    let sexp = parse_fixture("item/visibility-public.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    assert!(matches!(item.vis, Visibility::Public));
}

#[test]
fn test_build_visibility_inherited() {
    let sexp = parse_fixture("item/visibility-inherited.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    assert!(matches!(item.vis, Visibility::Inherited));
}

#[test]
fn test_build_visibility_default_inherited() {
    let sexp = parse_fixture("item/visibility-default-inherited.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    assert!(matches!(item.vis, Visibility::Inherited));
}

// ===== Ident Building Tests =====

#[test]
fn test_build_ident_with_span() {
    let sexp = parse_fixture("item/ident-with-span.sexp");
    let mut builder = AstBuilder::new();
    let ident = builder.build_ident(&sexp).unwrap();

    assert_eq!(ident.name, "my_function");
    assert_eq!(ident.span.lo, 0);
    assert_eq!(ident.span.hi, 11);
}

#[test]
fn test_build_ident_without_span() {
    let sexp = parse_fixture("item/ident-without-span.sexp");
    let mut builder = AstBuilder::new();
    let ident = builder.build_ident(&sexp).unwrap();

    assert_eq!(ident.name, "test");
    assert_eq!(ident.span, Span::DUMMY);
}

#[test]
fn test_build_ident_missing_name() {
    let sexp = parse_fixture("item/ident-missing-name.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_ident(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_ident_wrong_node_type() {
    let sexp = parse_fixture("item/ident-wrong-node-type.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_ident(&sexp);

    assert!(result.is_err());
}

// ===== Item Building Tests =====

#[test]
fn test_build_item_missing_ident() {
    let sexp = parse_fixture("item/missing-ident.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_item(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_item_missing_kind() {
    let sexp = parse_fixture("item/missing-kind.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_item(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_item_wrong_node_type() {
    let sexp = parse_fixture("item/wrong-node-type.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_item(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_item_with_explicit_id() {
    let sexp = parse_fixture("item/with-explicit-id.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    assert_eq!(item.id, NodeId(42));
}

// ===== Function Building Tests =====

#[test]
fn test_build_fn_minimal() {
    let sexp = parse_fixture("fn/minimal.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Fn(f) => {
            assert!(matches!(f.defaultness, Defaultness::Final));
            assert!(f.body.is_none());
        }
    }
}

#[test]
fn test_build_fn_with_defaultness_final() {
    let sexp = parse_fixture("fn/defaultness-final.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Fn(f) => {
            assert!(matches!(f.defaultness, Defaultness::Final));
        }
    }
}

#[test]
fn test_build_fn_with_defaultness_default() {
    let sexp = parse_fixture("fn/defaultness-default.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Fn(f) => {
            assert!(matches!(f.defaultness, Defaultness::Default));
        }
    }
}

#[test]
fn test_build_fn_with_body() {
    let sexp = parse_fixture("fn/with-body.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Fn(f) => {
            assert!(f.body.is_some());
            if let Some(block) = f.body {
                assert_eq!(block.id, NodeId(1));
            }
        }
    }
}

#[test]
fn test_build_fn_with_nil_body() {
    let sexp = parse_fixture("fn/with-nil-body.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Fn(f) => {
            assert!(f.body.is_none());
        }
    }
}

#[test]
fn test_build_fn_missing_sig() {
    let sexp = parse_fixture("fn/missing-sig.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_item(&sexp);

    assert!(result.is_err());
}

// ===== FnSig Building Tests =====

#[test]
fn test_build_fn_sig_minimal() {
    let sexp = parse_fixture("fn/sig-minimal.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Fn(f) => {
            assert!(matches!(f.sig.header.safety, Safety::Default));
            assert!(matches!(f.sig.header.constness, Constness::NotConst));
            assert_eq!(f.sig.decl.inputs.len(), 0);
        }
    }
}

#[test]
fn test_build_fn_sig_with_header() {
    let sexp = parse_fixture("fn/sig-with-header.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Fn(f) => {
            assert!(matches!(f.sig.header.safety, Safety::Unsafe));
            assert!(matches!(f.sig.header.constness, Constness::Const));
        }
    }
}

#[test]
fn test_build_fn_sig_with_decl() {
    let sexp = parse_fixture("fn/sig-with-decl.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Fn(f) => {
            assert_eq!(f.sig.decl.inputs.len(), 0);
            assert!(matches!(f.sig.decl.output, FnRetTy::Default(_)));
        }
    }
}

// ===== FnHeader Building Tests =====

#[test]
fn test_build_fn_header_default_safety() {
    let sexp = parse_fixture("fn/header-default-safety.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Fn(f) => {
            assert!(matches!(f.sig.header.safety, Safety::Default));
        }
    }
}

#[test]
fn test_build_fn_header_safe() {
    let sexp = parse_fixture("fn/header-safe.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Fn(f) => {
            assert!(matches!(f.sig.header.safety, Safety::Safe));
        }
    }
}

#[test]
fn test_build_fn_header_unsafe() {
    let sexp = parse_fixture("fn/header-unsafe.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Fn(f) => {
            assert!(matches!(f.sig.header.safety, Safety::Unsafe));
        }
    }
}

#[test]
fn test_build_fn_header_const() {
    let sexp = parse_fixture("fn/header-const.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Fn(f) => {
            assert!(matches!(f.sig.header.constness, Constness::Const));
        }
    }
}

#[test]
fn test_build_fn_header_not_const() {
    let sexp = parse_fixture("fn/header-not-const.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Fn(f) => {
            assert!(matches!(f.sig.header.constness, Constness::NotConst));
        }
    }
}

// ===== FnDecl Building Tests =====

#[test]
fn test_build_fn_decl_no_inputs_default_output() {
    let sexp = parse_fixture("fn/decl-no-inputs-default-output.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Fn(f) => {
            assert_eq!(f.sig.decl.inputs.len(), 0);
            assert!(matches!(f.sig.decl.output, FnRetTy::Default(_)));
        }
    }
}

#[test]
fn test_build_fn_decl_with_inputs() {
    let sexp = parse_fixture("fn/decl-with-inputs.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Fn(f) => {
            assert_eq!(f.sig.decl.inputs.len(), 1);
        }
    }
}

#[test]
fn test_build_fn_decl_with_multiple_inputs() {
    let sexp = parse_fixture("fn/decl-with-multiple-inputs.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Fn(f) => {
            assert_eq!(f.sig.decl.inputs.len(), 3);
        }
    }
}

// ===== FnRetTy Building Tests =====

#[test]
fn test_build_fn_ret_ty_default() {
    let sexp = parse_fixture("fn/ret-ty-default.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Fn(f) => {
            assert!(matches!(f.sig.decl.output, FnRetTy::Default(_)));
        }
    }
}

#[test]
fn test_build_fn_ret_ty_ty_variant() {
    let sexp = parse_fixture("fn/ret-ty-ty-variant.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Fn(f) => {
            // Currently simplified to return Default - this is Phase 1 limitation
            assert!(matches!(f.sig.decl.output, FnRetTy::Default(_)));
        }
    }
}

// ===== Generics Building Tests =====

#[test]
fn test_build_fn_with_generics() {
    let sexp = parse_fixture("fn/with-generics.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Fn(f) => {
            assert_eq!(f.generics.params.len(), 0);
        }
    }
}

// ===== Complex Integration Tests =====

#[test]
fn test_build_complete_function_item() {
    let sexp = parse_fixture("fn/complete-function-item.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    assert_eq!(item.id, NodeId(100));
    assert!(matches!(item.vis, Visibility::Public));
    assert_eq!(item.ident.name, "calculate");
    assert_eq!(item.span.lo, 0);
    assert_eq!(item.span.hi, 50);

    match item.kind {
        ItemKind::Fn(f) => {
            assert!(matches!(f.defaultness, Defaultness::Final));
            assert!(matches!(f.sig.header.safety, Safety::Default));
            assert!(matches!(f.sig.header.constness, Constness::NotConst));
            assert_eq!(f.sig.decl.inputs.len(), 2);
            assert!(f.body.is_some());

            if let Some(block) = f.body {
                assert_eq!(block.id, NodeId(102));
                assert_eq!(block.stmts.len(), 1);
            }
        }
    }
}

#[test]
fn test_build_unsafe_const_function() {
    let sexp = parse_fixture("fn/unsafe-const-function.sexp");
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    match item.kind {
        ItemKind::Fn(f) => {
            assert!(matches!(f.sig.header.safety, Safety::Unsafe));
            assert!(matches!(f.sig.header.constness, Constness::Const));
        }
    }
}

#[test]
fn test_build_item_kind_unsupported() {
    let sexp = parse_fixture("item/kind-unsupported.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_item(&sexp);

    assert!(result.is_err());
}
