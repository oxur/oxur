use oxur_ast::ast::*;
use oxur_ast::builder::AstBuilder;
use oxur_ast::sexp::Parser;

// ===== Empty Statement Tests =====

#[test]
fn test_build_stmt_empty_with_id() {
    let input = r#"(Stmt
      :id 5
      :kind (Empty)
      :span (Span :lo 0 :hi 1))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let stmt = builder.build_stmt(&sexp).unwrap();

    assert!(matches!(stmt.kind, StmtKind::Empty));
    assert_eq!(stmt.id, NodeId(5));
    assert_eq!(stmt.span.lo, 0);
    assert_eq!(stmt.span.hi, 1);
}

#[test]
fn test_build_stmt_empty_generates_id() {
    let input = r#"(Stmt
      :kind (Empty)
      :span (Span))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let stmt = builder.build_stmt(&sexp).unwrap();

    assert!(matches!(stmt.kind, StmtKind::Empty));
    assert_eq!(stmt.id, NodeId(0)); // First generated ID
}

// ===== Semi Statement Tests =====

#[test]
fn test_build_stmt_semi_with_keyword_syntax() {
    let input = r#"(Stmt
      :id 10
      :kind (Semi
              :expr (Expr
                      :id 11
                      :kind (MacCall
                              :path (Path :segments ((PathSegment :ident (Ident :name "test")))))))
      :span (Span))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Stmt
      :id 20
      :kind (Semi
              :expr (Expr
                      :id 21
                      :kind (MacCall
                              :path (Path :segments ((PathSegment :ident (Ident :name "println")))))))
      :span (Span))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Stmt
      :id 30
      :kind (Semi)
      :span (Span))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_stmt(&sexp);

    assert!(result.is_err());
}

// ===== Expr Statement Tests =====

#[test]
fn test_build_stmt_expr_with_keyword_syntax() {
    let input = r#"(Stmt
      :id 40
      :kind (Expr
              :expr (Expr
                      :id 41
                      :kind (MacCall
                              :path (Path :segments ((PathSegment :ident (Ident :name "test")))))))
      :span (Span))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Stmt
      :id 50
      :kind (Expr
              :expr (Expr
                      :id 51
                      :kind (MacCall
                              :path (Path :segments ((PathSegment :ident (Ident :name "value")))))))
      :span (Span))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Stmt
      :id 60
      :kind (Expr)
      :span (Span))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_stmt(&sexp);

    assert!(result.is_err());
}

// ===== Error Handling Tests =====

#[test]
fn test_build_stmt_wrong_node_type() {
    let input = r#"(NotStmt
      :kind (Empty))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_stmt(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_stmt_missing_kind() {
    let input = r#"(Stmt
      :id 70
      :span (Span))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_stmt(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_stmt_unsupported_kind() {
    let input = r#"(Stmt
      :id 80
      :kind (UnsupportedKind)
      :span (Span))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_stmt(&sexp);

    assert!(result.is_err());
}

// ===== Multiple Statements Tests =====

#[test]
fn test_build_multiple_statements_in_block() {
    let input = r#"(Block
      :stmts ((Stmt :id 1 :kind (Empty) :span (Span))
              (Stmt :id 2 :kind (Semi
                                  :expr (Expr
                                          :id 3
                                          :kind (MacCall
                                                  :path (Path :segments ((PathSegment :ident (Ident :name "test")))))))
                    :span (Span))
              (Stmt :id 4 :kind (Expr
                                  :expr (Expr
                                          :id 5
                                          :kind (MacCall
                                                  :path (Path :segments ((PathSegment :ident (Ident :name "final")))))))
                    :span (Span)))
      :id 10
      :span (Span :lo 0 :hi 100))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Stmt
      :id 100
      :kind (Semi
              :expr (Expr
                      :id 101
                      :kind (MacCall
                              :path (Path
                                      :segments ((PathSegment :ident (Ident :name "println") :id 102))
                                      :span (Span :lo 0 :hi 7))
                              :args (Delimited
                                      :delim Paren
                                      :tokens (TokenStream :source "\"Hello\"")))
                      :span (Span :lo 0 :hi 20)))
      :span (Span :lo 0 :hi 21))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
    let input = r#"(Block
      :stmts ()
      :id 200
      :span (Span :lo 0 :hi 2))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let block = builder.build_block(&sexp).unwrap();

    assert_eq!(block.stmts.len(), 0);
    assert_eq!(block.id, NodeId(200));
}

#[test]
fn test_build_block_without_explicit_stmts() {
    let input = r#"(Block
      :id 300
      :span (Span))"#;

    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let block = builder.build_block(&sexp).unwrap();

    assert_eq!(block.stmts.len(), 0);
    assert_eq!(block.id, NodeId(300));
}

// ===== Span Tests =====

#[test]
fn test_build_stmt_without_span() {
    // Test that missing :span field uses Span::DUMMY (line 35)
    let input = r#"(Stmt
      :id 10
      :kind (Empty))"#;

    let sexp = Parser::parse_str(input).unwrap();
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

    let input1 = r#"(Stmt :kind (Empty) :span (Span))"#;
    let sexp1 = Parser::parse_str(input1).unwrap();
    let stmt1 = builder.build_stmt(&sexp1).unwrap();
    assert_eq!(stmt1.id, NodeId(0));

    let input2 = r#"(Stmt :kind (Empty) :span (Span))"#;
    let sexp2 = Parser::parse_str(input2).unwrap();
    let stmt2 = builder.build_stmt(&sexp2).unwrap();
    assert_eq!(stmt2.id, NodeId(1));

    let input3 = r#"(Stmt :kind (Empty) :span (Span))"#;
    let sexp3 = Parser::parse_str(input3).unwrap();
    let stmt3 = builder.build_stmt(&sexp3).unwrap();
    assert_eq!(stmt3.id, NodeId(2));
}

#[test]
fn test_nested_expr_in_stmt_id_generation() {
    let input = r#"(Stmt
      :kind (Semi
              :expr (Expr
                      :kind (MacCall
                              :path (Path :segments ((PathSegment :ident (Ident :name "test")))))))
      :span (Span))"#;

    let sexp = Parser::parse_str(input).unwrap();
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
