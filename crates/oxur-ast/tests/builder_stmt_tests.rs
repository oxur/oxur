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

// ===== Empty Statement Tests =====

#[test]
fn test_build_stmt_empty_with_id() {
    let sexp = parse_fixture("stmt/empty-with-id.sexp");
    let mut builder = AstBuilder::new();
    let stmt = builder.build_stmt(&sexp).unwrap();

    assert!(matches!(stmt.kind, StmtKind::Empty));
    assert_eq!(stmt.id, NodeId(5));
    assert_eq!(stmt.span.lo, 0);
    assert_eq!(stmt.span.hi, 1);
}

#[test]
fn test_build_stmt_empty_generates_id() {
    let sexp = parse_fixture("stmt/empty-generates-id.sexp");
    let mut builder = AstBuilder::new();
    let stmt = builder.build_stmt(&sexp).unwrap();

    assert!(matches!(stmt.kind, StmtKind::Empty));
    assert_eq!(stmt.id, NodeId(0)); // First generated ID
}

// ===== Semi Statement Tests =====

#[test]
fn test_build_stmt_semi_with_keyword_syntax() {
    let sexp = parse_fixture("stmt/semi-with-keyword-syntax.sexp");
    let mut builder = AstBuilder::new();
    let stmt = builder.build_stmt(&sexp).unwrap();

    match stmt.kind {
        StmtKind::Semi(ref expr) => {
            assert_eq!(expr.id, NodeId(11));
        }
        _ => panic!("Expected Semi statement"),
    }
    assert_eq!(stmt.id, NodeId(10));
}

#[test]
fn test_build_stmt_semi_with_macro_call() {
    let sexp = parse_fixture("stmt/semi-with-macro-call.sexp");
    let mut builder = AstBuilder::new();
    let stmt = builder.build_stmt(&sexp).unwrap();

    match stmt.kind {
        StmtKind::Semi(ref expr) => {
            assert_eq!(expr.id, NodeId(21));
        }
        _ => panic!("Expected Semi statement"),
    }
}

#[test]
fn test_build_stmt_semi_missing_expr() {
    let sexp = parse_fixture("stmt/semi-missing-expr.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_stmt(&sexp);

    assert!(result.is_err());
}

// ===== Expr Statement Tests =====

#[test]
fn test_build_stmt_expr_with_keyword_syntax() {
    let sexp = parse_fixture("stmt/expr-with-keyword-syntax.sexp");
    let mut builder = AstBuilder::new();
    let stmt = builder.build_stmt(&sexp).unwrap();

    match stmt.kind {
        StmtKind::Expr(ref expr) => {
            assert_eq!(expr.id, NodeId(41));
        }
        _ => panic!("Expected Expr statement"),
    }
    assert_eq!(stmt.id, NodeId(40));
}

#[test]
fn test_build_stmt_expr_with_macro_call() {
    let sexp = parse_fixture("stmt/expr-with-macro-call.sexp");
    let mut builder = AstBuilder::new();
    let stmt = builder.build_stmt(&sexp).unwrap();

    match stmt.kind {
        StmtKind::Expr(ref expr) => {
            assert_eq!(expr.id, NodeId(51));
        }
        _ => panic!("Expected Expr statement"),
    }
}

#[test]
fn test_build_stmt_expr_missing_expr() {
    let sexp = parse_fixture("stmt/expr-missing-expr.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_stmt(&sexp);

    assert!(result.is_err());
}

// ===== Error Handling Tests =====

#[test]
fn test_build_stmt_wrong_node_type() {
    let sexp = parse_fixture("stmt/wrong-node-type.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_stmt(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_stmt_missing_kind() {
    let sexp = parse_fixture("stmt/missing-kind.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_stmt(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_stmt_unsupported_kind() {
    let sexp = parse_fixture("stmt/unsupported-kind.sexp");
    let mut builder = AstBuilder::new();
    let result = builder.build_stmt(&sexp);

    assert!(result.is_err());
}

// ===== Multiple Statements Tests =====

#[test]
fn test_build_multiple_statements_in_block() {
    let sexp = parse_fixture("block/multiple-statements.sexp");
    let mut builder = AstBuilder::new();
    let block = builder.build_block(&sexp).unwrap();

    assert_eq!(block.stmts.len(), 3);

    // First statement should be Empty
    assert!(matches!(block.stmts[0].kind, StmtKind::Empty));
    assert_eq!(block.stmts[0].id, NodeId(1));

    // Second statement should be Semi
    match &block.stmts[1].kind {
        StmtKind::Semi(expr) => {
            assert_eq!(expr.id, NodeId(3));
        }
        _ => panic!("Expected Semi statement"),
    }
    assert_eq!(block.stmts[1].id, NodeId(2));

    // Third statement should be Expr
    match &block.stmts[2].kind {
        StmtKind::Expr(expr) => {
            assert_eq!(expr.id, NodeId(5));
        }
        _ => panic!("Expected Expr statement"),
    }
    assert_eq!(block.stmts[2].id, NodeId(4));
}

// ===== Complex Integration Tests =====

#[test]
fn test_build_stmt_with_complex_expression() {
    let sexp = parse_fixture("stmt/complex-expression.sexp");
    let mut builder = AstBuilder::new();
    let stmt = builder.build_stmt(&sexp).unwrap();

    assert_eq!(stmt.id, NodeId(100));
    assert_eq!(stmt.span.lo, 0);
    assert_eq!(stmt.span.hi, 21);

    match stmt.kind {
        StmtKind::Semi(expr) => {
            assert_eq!(expr.id, NodeId(101));
            match expr.kind {
                ExprKind::MacCall(ref mac_call) => {
                    assert_eq!(mac_call.path.segments.len(), 1);
                    assert_eq!(mac_call.path.segments[0].ident.name, "println");
                    assert_eq!(mac_call.path.segments[0].id, NodeId(102));
                }
                _ => panic!("Expected MacCall"),
            }
        }
        _ => panic!("Expected Semi statement"),
    }
}

#[test]
fn test_build_empty_block() {
    let sexp = parse_fixture("block/empty-block.sexp");
    let mut builder = AstBuilder::new();
    let block = builder.build_block(&sexp).unwrap();

    assert_eq!(block.stmts.len(), 0);
    assert_eq!(block.id, NodeId(200));
}

#[test]
fn test_build_block_without_explicit_stmts() {
    let sexp = parse_fixture("block/without-explicit-stmts.sexp");
    let mut builder = AstBuilder::new();
    let block = builder.build_block(&sexp).unwrap();

    assert_eq!(block.stmts.len(), 0);
    assert_eq!(block.id, NodeId(300));
}

// ===== Span Tests =====

#[test]
fn test_build_stmt_without_span() {
    // Test that missing :span field uses Span::DUMMY (line 35)
    let sexp = parse_fixture("stmt/without-span.sexp");
    let mut builder = AstBuilder::new();
    let stmt = builder.build_stmt(&sexp).unwrap();

    assert!(matches!(stmt.kind, StmtKind::Empty));
    assert_eq!(stmt.id, NodeId(10));
    // DUMMY span has lo=0, hi=0
    assert_eq!(stmt.span.lo, 0);
    assert_eq!(stmt.span.hi, 0);
}

// Note: This test suite uses keyword syntax exclusively (e.g., :expr, :kind).
// The builder enforces strict keyword-value pair syntax via parse_kwargs().
// Positional syntax is not supported by design.

// ===== ID Generation Tests =====

#[test]
fn test_stmt_id_generation_sequence() {
    let mut builder = AstBuilder::new();

    let sexp1 = parse_fixture("stmt/empty-generates-id.sexp");
    let stmt1 = builder.build_stmt(&sexp1).unwrap();
    assert_eq!(stmt1.id, NodeId(0));

    let sexp2 = parse_fixture("stmt/empty-generates-id.sexp");
    let stmt2 = builder.build_stmt(&sexp2).unwrap();
    assert_eq!(stmt2.id, NodeId(1));

    let sexp3 = parse_fixture("stmt/empty-generates-id.sexp");
    let stmt3 = builder.build_stmt(&sexp3).unwrap();
    assert_eq!(stmt3.id, NodeId(2));
}

#[test]
fn test_nested_expr_in_stmt_id_generation() {
    let sexp = parse_fixture("stmt/semi-with-keyword-syntax.sexp");
    let mut builder = AstBuilder::new();
    let stmt = builder.build_stmt(&sexp).unwrap();

    // IDs are generated for all nested nodes
    // The statement should have an ID
    assert!(stmt.id.0 < 100); // Sanity check

    // Expression inside also gets an ID
    match stmt.kind {
        StmtKind::Semi(ref expr) => {
            assert!(expr.id.0 < 100); // Sanity check
                                      // The expr ID should be different from stmt ID
            assert_ne!(expr.id, stmt.id);
        }
        _ => panic!("Expected Semi statement"),
    }
}
