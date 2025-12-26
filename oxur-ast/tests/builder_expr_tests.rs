use oxur_ast::ast::*;
use oxur_ast::builder::AstBuilder;
use oxur_ast::sexp::Parser;

// ===== Block Building Tests =====

#[test]
fn test_build_block_with_statements() {
    let input = r#"(Block
      :stmts ((Stmt :id 1 :kind (Empty) :span (Span))
              (Stmt :id 2 :kind (Empty) :span (Span)))
      :id 3
      :span (Span :lo 0 :hi 10))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let block = builder.build_block(&sexp).unwrap();

    assert_eq!(block.stmts.len(), 2);
    assert_eq!(block.id, NodeId(3));
}

#[test]
fn test_build_block_wrong_node_type() {
    let input = "(NotBlock :stmts ())";
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_block(&sexp);

    assert!(result.is_err());
}

// ===== Expression Building Tests =====

#[test]
fn test_build_expr_macro_call() {
    let input = r#"(Expr
      :kind (MacCall
              :path (Path :segments ((PathSegment :ident (Ident :name "println")))))
      :id 1)"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    assert!(matches!(expr.kind, ExprKind::MacCall(_)));
    assert_eq!(expr.id, NodeId(1));
}

#[test]
fn test_build_expr_missing_kind() {
    let input = "(Expr :id 1)";
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_expr(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_expr_wrong_node_type() {
    let input = "(NotExpr :kind (MacCall))";
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_expr(&sexp);

    assert!(result.is_err());
}

// ===== Path Building Tests =====

#[test]
fn test_build_path_single_segment() {
    let input = r#"(Path
      :segments ((PathSegment :ident (Ident :name "std"))))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let path = builder.build_path(&sexp).unwrap();

    assert_eq!(path.segments.len(), 1);
    assert_eq!(path.segments[0].ident.name, "std");
}

#[test]
fn test_build_path_multiple_segments() {
    let input = r#"(Path
      :segments ((PathSegment :ident (Ident :name "std"))
                 (PathSegment :ident (Ident :name "collections"))
                 (PathSegment :ident (Ident :name "HashMap"))))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let path = builder.build_path(&sexp).unwrap();

    assert_eq!(path.segments.len(), 3);
    assert_eq!(path.segments[0].ident.name, "std");
    assert_eq!(path.segments[1].ident.name, "collections");
    assert_eq!(path.segments[2].ident.name, "HashMap");
}

#[test]
fn test_build_path_empty_segments() {
    let input = "(Path :segments ())";
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let path = builder.build_path(&sexp).unwrap();

    assert_eq!(path.segments.len(), 0);
}

#[test]
fn test_build_path_wrong_node_type() {
    let input = "(NotPath :segments ())";
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_path(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_path_with_span() {
    let input = r#"(Path
      :segments ()
      :span (Span :lo 0 :hi 5))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let path = builder.build_path(&sexp).unwrap();

    assert_eq!(path.span.lo, 0);
    assert_eq!(path.span.hi, 5);
}

// ===== MacCall Building Tests =====

#[test]
fn test_build_expr_mac_call_with_empty_args() {
    let input = r#"(Expr
      :kind (MacCall
              :path (Path :segments ((PathSegment :ident (Ident :name "test"))))
              :args (Empty)))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::MacCall(mac_call) => {
            assert!(matches!(mac_call.args, MacArgs::Empty));
        }
        _ => panic!("Expected MacCall"),
    }
}

#[test]
fn test_build_expr_mac_call_with_delimited_args() {
    let input = r#"(Expr
      :kind (MacCall
              :path (Path :segments ((PathSegment :ident (Ident :name "vec"))))
              :args (Delimited
                      :delim Bracket
                      :tokens (TokenStream :source "1, 2, 3"))))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::MacCall(mac_call) => match mac_call.args {
            MacArgs::Delimited { delim, tokens, .. } => {
                assert!(matches!(delim, Delimiter::Bracket));
                match tokens {
                    TokenStream::Source(ref s) => assert_eq!(s, "1, 2, 3"),
                    _ => panic!("Expected Source token stream"),
                }
            }
            _ => panic!("Expected Delimited args"),
        },
        _ => panic!("Expected MacCall"),
    }
}

#[test]
fn test_build_expr_mac_call_missing_path() {
    let input = "(Expr :kind (MacCall :args (Empty)))";
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_expr(&sexp);

    assert!(result.is_err());
}

// ===== Delimiter Building Tests =====

#[test]
fn test_build_mac_args_with_all_delimiters() {
    let test_cases = vec![
        ("Paren", Delimiter::Paren),
        ("Brace", Delimiter::Brace),
        ("Bracket", Delimiter::Bracket),
        ("Invisible", Delimiter::Invisible),
    ];

    for (delim_str, expected_delim) in test_cases {
        let input = format!(
            r#"(Expr
              :kind (MacCall
                      :path (Path :segments ((PathSegment :ident (Ident :name "test"))))
                      :args (Delimited :delim {})))"#,
            delim_str
        );

        let sexp = Parser::parse_str(&input).unwrap();
        let mut builder = AstBuilder::new();
        let expr = builder.build_expr(&sexp).unwrap();

        match expr.kind {
            ExprKind::MacCall(mac_call) => match mac_call.args {
                MacArgs::Delimited { delim, .. } => {
                    assert_eq!(delim, expected_delim);
                }
                _ => panic!("Expected Delimited args"),
            },
            _ => panic!("Expected MacCall"),
        }
    }
}

// ===== TokenStream Building Tests =====

#[test]
fn test_build_token_stream_empty() {
    let input = r#"(Expr
      :kind (MacCall
              :path (Path :segments ((PathSegment :ident (Ident :name "test"))))
              :args (Delimited :tokens (TokenStream))))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::MacCall(mac_call) => match mac_call.args {
            MacArgs::Delimited { tokens, .. } => {
                assert!(matches!(tokens, TokenStream::Empty));
            }
            _ => panic!("Expected Delimited args"),
        },
        _ => panic!("Expected MacCall"),
    }
}

#[test]
fn test_build_token_stream_with_source() {
    let input = r#"(Expr
      :kind (MacCall
              :path (Path :segments ((PathSegment :ident (Ident :name "test"))))
              :args (Delimited :tokens (TokenStream :source "hello world"))))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::MacCall(mac_call) => match mac_call.args {
            MacArgs::Delimited { tokens, .. } => match tokens {
                TokenStream::Source(ref s) => assert_eq!(s, "hello world"),
                _ => panic!("Expected Source token stream"),
            },
            _ => panic!("Expected Delimited args"),
        },
        _ => panic!("Expected MacCall"),
    }
}

// ===== PathSegment Building Tests =====

#[test]
fn test_build_path_segment_with_explicit_id() {
    let input = r#"(Path
      :segments ((PathSegment :ident (Ident :name "test") :id 42)))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let path = builder.build_path(&sexp).unwrap();

    assert_eq!(path.segments[0].id, NodeId(42));
}

#[test]
fn test_build_path_segment_generates_id() {
    let input = r#"(Path
      :segments ((PathSegment :ident (Ident :name "test"))))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let path = builder.build_path(&sexp).unwrap();

    // Should have auto-generated an ID
    assert_eq!(path.segments[0].id, NodeId(0));
}

#[test]
fn test_build_path_segment_missing_ident() {
    let input = "(Path :segments ((PathSegment :id 1)))";
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_path(&sexp);

    assert!(result.is_err());
}

// ===== Error Handling Tests =====

#[test]
fn test_build_expr_kind_unsupported() {
    let input = "(Expr :kind (UnsupportedKind))";
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_expr(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_mac_args_unsupported_kind() {
    let input = r#"(Expr
      :kind (MacCall
              :path (Path :segments ((PathSegment :ident (Ident :name "test"))))
              :args (Eq)))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_expr(&sexp);

    assert!(result.is_err());
}

// ===== Complex Integration Tests =====

#[test]
fn test_build_complex_macro_call_expression() {
    let input = r#"(Expr
      :id 100
      :kind (MacCall
              :path (Path
                      :segments ((PathSegment :ident (Ident :name "println") :id 101))
                      :span (Span :lo 0 :hi 7))
              :args (Delimited
                      :delim Paren
                      :dspan (DelSpan :open (Span :lo 7 :hi 8) :close (Span :lo 20 :hi 21))
                      :tokens (TokenStream :source "\"Hello, world!\"")))
      :span (Span :lo 0 :hi 21))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    assert_eq!(expr.id, NodeId(100));
    assert_eq!(expr.span.lo, 0);
    assert_eq!(expr.span.hi, 21);

    match expr.kind {
        ExprKind::MacCall(mac_call) => {
            assert_eq!(mac_call.path.segments.len(), 1);
            assert_eq!(mac_call.path.segments[0].ident.name, "println");
            assert_eq!(mac_call.path.segments[0].id, NodeId(101));

            match mac_call.args {
                MacArgs::Delimited { dspan, delim, tokens } => {
                    assert_eq!(dspan.open.lo, 7);
                    assert_eq!(dspan.close.hi, 21);
                    assert!(matches!(delim, Delimiter::Paren));
                    match tokens {
                        TokenStream::Source(ref s) => assert_eq!(s, "\"Hello, world!\""),
                        _ => panic!("Expected Source"),
                    }
                }
                _ => panic!("Expected Delimited"),
            }
        }
        _ => panic!("Expected MacCall"),
    }
}
