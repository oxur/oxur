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

// ===== Block Building Tests =====

#[test]
fn test_build_block_with_statements() {
    let sexp = parse_fixture("block/with-statements.sexp");
    let mut builder = AstBuilder::new();
    let block = builder.build_block(&sexp).unwrap();

    assert_eq!(block.stmts.len(), 2);
    assert_eq!(block.id, NodeId(3));
}

#[test]
fn test_build_block_wrong_node_type() {
    let sexp = parse_fixture("block/wrong-node-type.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_block(&sexp);

    assert!(result.is_err());
}

// ===== Expression Building Tests =====

#[test]
fn test_build_expr_macro_call() {
    let sexp = parse_fixture("expr/macro-call.sexp");
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    assert!(matches!(expr.kind, ExprKind::MacCall(_)));
    assert_eq!(expr.id, NodeId(1));
}

#[test]
fn test_build_expr_missing_kind() {
    let sexp = parse_fixture("expr/missing-kind.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_expr(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_expr_wrong_node_type() {
    let sexp = parse_fixture("expr/wrong-node-type.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_expr(&sexp);

    assert!(result.is_err());
}

// ===== Path Building Tests =====

#[test]
fn test_build_path_single_segment() {
    let sexp = parse_fixture("path/single-segment.sexp");
    let mut builder = AstBuilder::new();
    let path = builder.build_path(&sexp).unwrap();

    assert_eq!(path.segments.len(), 1);
    assert_eq!(path.segments[0].ident.name, "std");
}

#[test]
fn test_build_path_multiple_segments() {
    let sexp = parse_fixture("path/multiple-segments.sexp");
    let mut builder = AstBuilder::new();
    let path = builder.build_path(&sexp).unwrap();

    assert_eq!(path.segments.len(), 3);
    assert_eq!(path.segments[0].ident.name, "std");
    assert_eq!(path.segments[1].ident.name, "collections");
    assert_eq!(path.segments[2].ident.name, "HashMap");
}

#[test]
fn test_build_path_empty_segments() {
    let sexp = parse_fixture("path/empty-segments.sexp");
    let mut builder = AstBuilder::new();
    let path = builder.build_path(&sexp).unwrap();

    assert_eq!(path.segments.len(), 0);
}

#[test]
fn test_build_path_wrong_node_type() {
    let sexp = parse_fixture("path/wrong-node-type.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_path(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_path_with_span() {
    let sexp = parse_fixture("path/with-span.sexp");
    let mut builder = AstBuilder::new();
    let path = builder.build_path(&sexp).unwrap();

    assert_eq!(path.span.lo, 0);
    assert_eq!(path.span.hi, 5);
}

// ===== MacCall Building Tests =====

#[test]
fn test_build_expr_mac_call_with_empty_args() {
    let sexp = parse_fixture("expr/macro-call-empty.sexp");
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
    let sexp = parse_fixture("expr/mac-call-with-delimited-args.sexp");
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
    let sexp = parse_fixture("expr/mac-call-missing-path.sexp");
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
              :id 1
              :kind (MacCall
                      (MacCall
                        :path (Path :segments ((PathSegment :ident (Ident :name "test"))))
                        :args (Delimited
                                :dspan (DelSpan :open (Span :lo 0 :hi 0) :close (Span :lo 0 :hi 0))
                                :delim {})
                        :prior-type-ascription nil))
              :span (Span :lo 0 :hi 0)
              :attrs ())"#,
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
    let sexp = parse_fixture("expr/token-stream-empty.sexp");
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
    let sexp = parse_fixture("expr/token-stream-with-source.sexp");
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
    let sexp = parse_fixture("path/segment-with-explicit-id.sexp");
    let mut builder = AstBuilder::new();
    let path = builder.build_path(&sexp).unwrap();

    assert_eq!(path.segments[0].id, NodeId(42));
}

#[test]
fn test_build_path_segment_generates_id() {
    let sexp = parse_fixture("path/segment-generates-id.sexp");
    let mut builder = AstBuilder::new();
    let path = builder.build_path(&sexp).unwrap();

    // Should have auto-generated an ID
    assert_eq!(path.segments[0].id, NodeId(0));
}

#[test]
fn test_build_path_segment_missing_ident() {
    let sexp = parse_fixture("path/segment-missing-ident.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_path(&sexp);

    assert!(result.is_err());
}

// ===== Error Handling Tests =====

#[test]
fn test_build_expr_kind_unsupported() {
    let sexp = parse_fixture("expr/kind-unsupported.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_expr(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_mac_args_unsupported_kind() {
    let sexp = parse_fixture("expr/mac-args-unsupported-kind.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_expr(&sexp);

    assert!(result.is_err());
}

// ===== Complex Integration Tests =====

#[test]
fn test_build_complex_macro_call_expression() {
    let sexp = parse_fixture("expr/complex-macro-call.sexp");
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
