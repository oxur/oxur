use oxur_ast::ast::*;
use oxur_ast::builder::AstBuilder;
use oxur_ast::sexp::Parser;

// ===== Visibility Building Tests =====

#[test]
fn test_build_visibility_public() {
    let input = r#"(Item
      :vis (Public)
      :ident (Ident :name "foo")
      :kind (Fn :sig (FnSig)))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    assert!(matches!(item.vis, Visibility::Public));
}

#[test]
fn test_build_visibility_inherited() {
    let input = r#"(Item
      :vis (Inherited)
      :ident (Ident :name "foo")
      :kind (Fn :sig (FnSig)))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    assert!(matches!(item.vis, Visibility::Inherited));
}

#[test]
fn test_build_visibility_default_inherited() {
    let input = r#"(Item
      :ident (Ident :name "foo")
      :kind (Fn :sig (FnSig)))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    assert!(matches!(item.vis, Visibility::Inherited));
}

// ===== Ident Building Tests =====

#[test]
fn test_build_ident_with_span() {
    let input = r#"(Ident :name "my_function" :span (Span :lo 0 :hi 11))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let ident = builder.build_ident(&sexp).unwrap();

    assert_eq!(ident.name, "my_function");
    assert_eq!(ident.span.lo, 0);
    assert_eq!(ident.span.hi, 11);
}

#[test]
fn test_build_ident_without_span() {
    let input = r#"(Ident :name "test")"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let ident = builder.build_ident(&sexp).unwrap();

    assert_eq!(ident.name, "test");
    assert_eq!(ident.span, Span::DUMMY);
}

#[test]
fn test_build_ident_missing_name() {
    let input = r#"(Ident)"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_ident(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_ident_wrong_node_type() {
    let input = r#"(NotIdent :name "test")"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_ident(&sexp);

    assert!(result.is_err());
}

// ===== Item Building Tests =====

#[test]
fn test_build_item_missing_ident() {
    let input = r#"(Item :kind (Fn :sig (FnSig)))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_item(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_item_missing_kind() {
    let input = r#"(Item :ident (Ident :name "foo"))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_item(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_item_wrong_node_type() {
    let input = r#"(NotItem :ident (Ident :name "foo") :kind (Fn :sig (FnSig)))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_item(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_item_with_explicit_id() {
    let input = r#"(Item
      :id 42
      :ident (Ident :name "foo")
      :kind (Fn :sig (FnSig)))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let item = builder.build_item(&sexp).unwrap();

    assert_eq!(item.id, NodeId(42));
}

// ===== Function Building Tests =====

#[test]
fn test_build_fn_minimal() {
    let input = r#"(Item
      :ident (Ident :name "main")
      :kind (Fn :sig (FnSig)))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :ident (Ident :name "foo")
      :kind (Fn :defaultness Final :sig (FnSig)))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :ident (Ident :name "foo")
      :kind (Fn :defaultness Default :sig (FnSig)))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :ident (Ident :name "main")
      :kind (Fn
              :sig (FnSig)
              :body (Block :stmts () :id 1)))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :ident (Ident :name "foo")
      :kind (Fn :sig (FnSig) :body nil))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :ident (Ident :name "foo")
      :kind (Fn :defaultness Final))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_item(&sexp);

    assert!(result.is_err());
}

// ===== FnSig Building Tests =====

#[test]
fn test_build_fn_sig_minimal() {
    let input = r#"(Item
      :ident (Ident :name "foo")
      :kind (Fn :sig (FnSig)))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :ident (Ident :name "foo")
      :kind (Fn :sig (FnSig :header (FnHeader :safety Unsafe :constness Const))))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :ident (Ident :name "foo")
      :kind (Fn :sig (FnSig :decl (FnDecl :inputs () :output (Default)))))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :ident (Ident :name "foo")
      :kind (Fn :sig (FnSig :header (FnHeader :safety Default))))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :ident (Ident :name "foo")
      :kind (Fn :sig (FnSig :header (FnHeader :safety Safe))))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :ident (Ident :name "foo")
      :kind (Fn :sig (FnSig :header (FnHeader :safety Unsafe))))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :ident (Ident :name "foo")
      :kind (Fn :sig (FnSig :header (FnHeader :constness Const))))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :ident (Ident :name "foo")
      :kind (Fn :sig (FnSig :header (FnHeader :constness NotConst))))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :ident (Ident :name "foo")
      :kind (Fn :sig (FnSig :decl (FnDecl :inputs () :output (Default)))))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :ident (Ident :name "foo")
      :kind (Fn :sig (FnSig :decl (FnDecl
                                    :inputs ((Param))
                                    :output (Default)))))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :ident (Ident :name "foo")
      :kind (Fn :sig (FnSig :decl (FnDecl
                                    :inputs ((Param) (Param) (Param))
                                    :output (Default)))))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :ident (Ident :name "foo")
      :kind (Fn :sig (FnSig :decl (FnDecl :output (Default)))))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :ident (Ident :name "foo")
      :kind (Fn :sig (FnSig :decl (FnDecl :output (Ty)))))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :ident (Ident :name "foo")
      :kind (Fn :sig (FnSig) :generics (Generics :params ())))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :id 100
      :vis (Public)
      :ident (Ident :name "calculate" :span (Span :lo 0 :hi 9))
      :kind (Fn
              :defaultness Final
              :sig (FnSig
                     :header (FnHeader :safety Default :constness NotConst)
                     :decl (FnDecl
                             :inputs ((Param) (Param))
                             :output (Default))
                     :span (Span :lo 10 :hi 40))
              :generics (Generics :params ())
              :body (Block
                      :stmts ((Stmt :id 101 :kind (Empty) :span (Span)))
                      :id 102
                      :span (Span :lo 40 :hi 50)))
      :span (Span :lo 0 :hi 50))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :ident (Ident :name "unsafe_const_fn")
      :kind (Fn :sig (FnSig :header (FnHeader :safety Unsafe :constness Const))))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Item
      :ident (Ident :name "foo")
      :kind (Struct))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_item(&sexp);

    assert!(result.is_err());
}
