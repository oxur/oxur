//! Comprehensive tests for builder/expr.rs to achieve 95%+ coverage
//!
//! This test file systematically covers all expression building functions
//! and their various code paths.

use oxur_ast::ast::*;
use oxur_ast::builder::AstBuilder;
use oxur_ast::sexp::Parser;

// ===== Control Flow Expression Tests =====

#[test]
fn test_build_expr_if_simple() {
    let input = r#"(Expr
      :id 1
      :kind (If
              :cond (Expr :id 2 :kind (Lit (Lit :kind (Int 1))))
              :then (Block :stmts ()))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::If { cond, then_branch, else_branch } => {
            assert!(matches!(cond.kind, ExprKind::Lit(_)));
            assert!(then_branch.stmts.is_empty());
            assert!(else_branch.is_none());
        }
        _ => panic!("Expected If expression"),
    }
}

#[test]
fn test_build_expr_if_with_else() {
    let input = r#"(Expr
      :id 1
      :kind (If
              :cond (Expr :id 2 :kind (Lit (Lit :kind (Int 1))))
              :then (Block :stmts ())
              :else (Expr :id 3 :kind (Lit (Lit :kind (Int 0)))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::If { else_branch, .. } => {
            assert!(else_branch.is_some());
        }
        _ => panic!("Expected If expression"),
    }
}

#[test]
fn test_build_expr_match() {
    let input = r#"(Expr
      :id 1
      :kind (Match
              :expr (Expr :id 2 :kind (Lit (Lit :kind (Int 1))))
              :arms ((Arm
                       :pat (Pat :kind Wild)
                       :body (Expr :id 3 :kind (Lit (Lit :kind (Int 0)))))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Match { expr, arms } => {
            assert!(matches!(expr.kind, ExprKind::Lit(_)));
            assert_eq!(arms.len(), 1);
        }
        _ => panic!("Expected Match expression"),
    }
}

#[test]
fn test_build_expr_while_simple() {
    let input = r#"(Expr
      :id 1
      :kind (While
              :cond (Expr :id 2 :kind (Lit (Lit :kind (Int 1))))
              :body (Block :stmts ()))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::While { label, cond, body } => {
            assert!(label.is_none());
            assert!(matches!(cond.kind, ExprKind::Lit(_)));
            assert!(body.stmts.is_empty());
        }
        _ => panic!("Expected While expression"),
    }
}

#[test]
fn test_build_expr_while_with_label() {
    let input = r#"(Expr
      :id 1
      :kind (While
              :label (Label :ident (Ident :name "outer"))
              :cond (Expr :id 2 :kind (Lit (Lit :kind (Int 1))))
              :body (Block :stmts ()))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::While { label, .. } => {
            assert!(label.is_some());
            assert_eq!(label.unwrap().ident.name, "outer");
        }
        _ => panic!("Expected While expression"),
    }
}

#[test]
fn test_build_expr_for_loop() {
    let input = r#"(Expr
      :id 1
      :kind (ForLoop
              :pat (Pat :kind (Ident :ident (Ident :name "x")))
              :iter (Expr :id 2 :kind (Lit (Lit :kind (Int 1))))
              :body (Block :stmts ()))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::ForLoop { label, pat, iter, body } => {
            assert!(label.is_none());
            assert!(matches!(pat.kind, PatKind::Ident { .. }));
            assert!(matches!(iter.kind, ExprKind::Lit(_)));
            assert!(body.stmts.is_empty());
        }
        _ => panic!("Expected ForLoop expression"),
    }
}

#[test]
fn test_build_expr_for_loop_with_label() {
    let input = r#"(Expr
      :id 1
      :kind (ForLoop
              :label (Label :ident (Ident :name "outer"))
              :pat (Pat :kind (Ident :ident (Ident :name "x")))
              :iter (Expr :id 2 :kind (Lit (Lit :kind (Int 1))))
              :body (Block :stmts ()))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::ForLoop { label, .. } => {
            assert!(label.is_some());
        }
        _ => panic!("Expected ForLoop expression"),
    }
}

#[test]
fn test_build_expr_loop_simple() {
    let input = r#"(Expr
      :id 1
      :kind (Loop :body (Block :stmts ()))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Loop { label, body } => {
            assert!(label.is_none());
            assert!(body.stmts.is_empty());
        }
        _ => panic!("Expected Loop expression"),
    }
}

#[test]
fn test_build_expr_loop_with_label() {
    let input = r#"(Expr
      :id 1
      :kind (Loop
              :label (Label :ident (Ident :name "forever"))
              :body (Block :stmts ()))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Loop { label, .. } => {
            assert!(label.is_some());
        }
        _ => panic!("Expected Loop expression"),
    }
}

// ===== Operator Expression Tests =====

#[test]
fn test_build_expr_binary_add() {
    let input = r#"(Expr
      :id 1
      :kind (Binary
              :left (Expr :id 2 :kind (Lit (Lit :kind (Int 1))))
              :op Add
              :right (Expr :id 3 :kind (Lit (Lit :kind (Int 2)))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Binary { op, .. } => {
            assert_eq!(op, BinOp::Add);
        }
        _ => panic!("Expected Binary expression"),
    }
}

#[test]
fn test_build_expr_binary_all_operators() {
    let operators = [
        ("Add", BinOp::Add),
        ("Sub", BinOp::Sub),
        ("Mul", BinOp::Mul),
        ("Div", BinOp::Div),
        ("Rem", BinOp::Rem),
        ("And", BinOp::And),
        ("Or", BinOp::Or),
        ("BitAnd", BinOp::BitAnd),
        ("BitOr", BinOp::BitOr),
        ("BitXor", BinOp::BitXor),
        ("Shl", BinOp::Shl),
        ("Shr", BinOp::Shr),
        ("Eq", BinOp::Eq),
        ("Ne", BinOp::Ne),
        ("Lt", BinOp::Lt),
        ("Le", BinOp::Le),
        ("Gt", BinOp::Gt),
        ("Ge", BinOp::Ge),
    ];

    for (op_str, expected_op) in operators {
        let input = format!(
            r#"(Expr
              :id 1
              :kind (Binary
                      :left (Expr :id 2 :kind (Lit (Lit :kind (Int 1))))
                      :op {}
                      :right (Expr :id 3 :kind (Lit (Lit :kind (Int 2)))))
              :span (Span :lo 0 :hi 10))"#,
            op_str
        );
        let sexp = Parser::parse_str(&input).unwrap();
        let mut builder = AstBuilder::new();
        let expr = builder.build_expr(&sexp).unwrap();

        match expr.kind {
            ExprKind::Binary { op, .. } => {
                assert_eq!(op, expected_op, "Failed for operator {}", op_str);
            }
            _ => panic!("Expected Binary expression for {}", op_str),
        }
    }
}

#[test]
fn test_build_expr_unary_all_operators() {
    let operators = [("Not", UnOp::Not), ("Neg", UnOp::Neg), ("Deref", UnOp::Deref)];

    for (op_str, expected_op) in operators {
        let input = format!(
            r#"(Expr
              :id 1
              :kind (Unary
                      :op {}
                      :expr (Expr :id 2 :kind (Lit (Lit :kind (Int 1)))))
              :span (Span :lo 0 :hi 10))"#,
            op_str
        );
        let sexp = Parser::parse_str(&input).unwrap();
        let mut builder = AstBuilder::new();
        let expr = builder.build_expr(&sexp).unwrap();

        match expr.kind {
            ExprKind::Unary { op, .. } => {
                assert_eq!(op, expected_op, "Failed for operator {}", op_str);
            }
            _ => panic!("Expected Unary expression for {}", op_str),
        }
    }
}

// ===== Call Expression Tests =====

#[test]
fn test_build_expr_call() {
    let input = r#"(Expr
      :id 1
      :kind (Call
              :func (Expr :id 2 :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "foo"))))))
              :args ())
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Call { func, args } => {
            assert!(matches!(func.kind, ExprKind::Path(_, _)));
            assert!(args.is_empty());
        }
        _ => panic!("Expected Call expression"),
    }
}

#[test]
fn test_build_expr_call_with_args() {
    let input = r#"(Expr
      :id 1
      :kind (Call
              :func (Expr :id 2 :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "foo"))))))
              :args ((Expr :id 3 :kind (Lit (Lit :kind (Int 1))))
                     (Expr :id 4 :kind (Lit (Lit :kind (Int 2))))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Call { args, .. } => {
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Call expression"),
    }
}

#[test]
fn test_build_expr_method_call() {
    let input = r#"(Expr
      :id 1
      :kind (MethodCall
              :receiver (Expr :id 2 :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "x"))))))
              :method (Ident :name "len")
              :args ())
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::MethodCall { receiver, method, args } => {
            assert!(matches!(receiver.kind, ExprKind::Path(_, _)));
            assert_eq!(method.name, "len");
            assert!(args.is_empty());
        }
        _ => panic!("Expected MethodCall expression"),
    }
}

// ===== Collection Expression Tests =====

#[test]
fn test_build_expr_array() {
    let input = r#"(Expr
      :id 1
      :kind (Array ((Expr :id 2 :kind (Lit (Lit :kind (Int 1))))
                    (Expr :id 3 :kind (Lit (Lit :kind (Int 2))))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Array(elems) => {
            assert_eq!(elems.len(), 2);
        }
        _ => panic!("Expected Array expression"),
    }
}

#[test]
fn test_build_expr_tuple() {
    let input = r#"(Expr
      :id 1
      :kind (Tuple ((Expr :id 2 :kind (Lit (Lit :kind (Int 1))))
                    (Expr :id 3 :kind (Lit (Lit :kind (Int 2))))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Tuple(elems) => {
            assert_eq!(elems.len(), 2);
        }
        _ => panic!("Expected Tuple expression"),
    }
}

#[test]
fn test_build_expr_struct() {
    let input = r#"(Expr
      :id 1
      :kind (Struct
              :path (Path :segments ((PathSegment :ident (Ident :name "Point"))))
              :fields ((ExprField
                         :ident (Ident :name "x")
                         :expr (Expr :id 2 :kind (Lit (Lit :kind (Int 1)))))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Struct { path, fields } => {
            assert_eq!(path.segments.len(), 1);
            assert_eq!(fields.len(), 1);
        }
        _ => panic!("Expected Struct expression"),
    }
}

#[test]
fn test_build_expr_struct_no_fields() {
    let input = r#"(Expr
      :id 1
      :kind (Struct
              :path (Path :segments ((PathSegment :ident (Ident :name "Unit")))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Struct { fields, .. } => {
            assert!(fields.is_empty());
        }
        _ => panic!("Expected Struct expression"),
    }
}

// ===== Access Expression Tests =====

#[test]
fn test_build_expr_field() {
    let input = r#"(Expr
      :id 1
      :kind (Field
              :expr (Expr :id 2 :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "point"))))))
              :field (Ident :name "x"))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Field { expr, field } => {
            assert!(matches!(expr.kind, ExprKind::Path(_, _)));
            assert_eq!(field.name, "x");
        }
        _ => panic!("Expected Field expression"),
    }
}

#[test]
fn test_build_expr_index() {
    let input = r#"(Expr
      :id 1
      :kind (Index
              :expr (Expr :id 2 :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "arr"))))))
              :index (Expr :id 3 :kind (Lit (Lit :kind (Int 0)))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Index { expr, index } => {
            assert!(matches!(expr.kind, ExprKind::Path(_, _)));
            assert!(matches!(index.kind, ExprKind::Lit(_)));
        }
        _ => panic!("Expected Index expression"),
    }
}

// ===== Other Expression Tests =====

#[test]
fn test_build_expr_assign() {
    let input = r#"(Expr
      :id 1
      :kind (Assign
              :left (Expr :id 2 :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "x"))))))
              :right (Expr :id 3 :kind (Lit (Lit :kind (Int 5)))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Assign { left, right } => {
            assert!(matches!(left.kind, ExprKind::Path(_, _)));
            assert!(matches!(right.kind, ExprKind::Lit(_)));
        }
        _ => panic!("Expected Assign expression"),
    }
}

#[test]
fn test_build_expr_closure_simple() {
    let input = r#"(Expr
      :id 1
      :kind (Closure
              :body (Expr :id 2 :kind (Lit (Lit :kind (Int 42)))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Closure { params, body } => {
            assert!(params.is_empty());
            assert!(matches!(body.kind, ExprKind::Lit(_)));
        }
        _ => panic!("Expected Closure expression"),
    }
}

#[test]
fn test_build_expr_closure_with_params() {
    let input = r#"(Expr
      :id 1
      :kind (Closure
              :params ((Param
                         :attrs ()
                         :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))
                         :pat (Pat :kind (Ident :ident (Ident :name "x")))))
              :body (Expr :id 2 :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "x")))))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Closure { params, .. } => {
            assert_eq!(params.len(), 1);
        }
        _ => panic!("Expected Closure expression"),
    }
}

#[test]
fn test_build_expr_range_full() {
    let input = r#"(Expr
      :id 1
      :kind (Range
              :start (Expr :id 2 :kind (Lit (Lit :kind (Int 0))))
              :end (Expr :id 3 :kind (Lit (Lit :kind (Int 10))))
              :inclusive false)
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Range { start, end, inclusive } => {
            assert!(start.is_some());
            assert!(end.is_some());
            assert!(!inclusive);
        }
        _ => panic!("Expected Range expression"),
    }
}

#[test]
fn test_build_expr_range_inclusive() {
    let input = r#"(Expr
      :id 1
      :kind (Range
              :start (Expr :id 2 :kind (Lit (Lit :kind (Int 0))))
              :end (Expr :id 3 :kind (Lit (Lit :kind (Int 10))))
              :inclusive true)
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Range { inclusive, .. } => {
            assert!(inclusive);
        }
        _ => panic!("Expected Range expression"),
    }
}

#[test]
fn test_build_expr_range_from() {
    let input = r#"(Expr
      :id 1
      :kind (Range
              :start (Expr :id 2 :kind (Lit (Lit :kind (Int 5)))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Range { start, end, .. } => {
            assert!(start.is_some());
            assert!(end.is_none());
        }
        _ => panic!("Expected Range expression"),
    }
}

#[test]
fn test_build_expr_range_to() {
    let input = r#"(Expr
      :id 1
      :kind (Range
              :end (Expr :id 2 :kind (Lit (Lit :kind (Int 5)))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Range { start, end, .. } => {
            assert!(start.is_none());
            assert!(end.is_some());
        }
        _ => panic!("Expected Range expression"),
    }
}

#[test]
fn test_build_expr_paren() {
    let input = r#"(Expr
      :id 1
      :kind (Paren (Expr :id 2 :kind (Lit (Lit :kind (Int 42)))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Paren(inner) => {
            assert!(matches!(inner.kind, ExprKind::Lit(_)));
        }
        _ => panic!("Expected Paren expression"),
    }
}

#[test]
fn test_build_expr_try() {
    let input = r#"(Expr
      :id 1
      :kind (Try (Expr :id 2 :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "result")))))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Try(inner) => {
            assert!(matches!(inner.kind, ExprKind::Path(_, _)));
        }
        _ => panic!("Expected Try expression"),
    }
}

#[test]
fn test_build_expr_cast() {
    let input = r#"(Expr
      :id 1
      :kind (Cast
              :expr (Expr :id 2 :kind (Lit (Lit :kind (Int 5))))
              :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "f64")))))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Cast { expr, ty } => {
            assert!(matches!(expr.kind, ExprKind::Lit(_)));
            assert!(matches!(ty.kind, TyKind::Path(_, _)));
        }
        _ => panic!("Expected Cast expression"),
    }
}

#[test]
fn test_build_expr_break_simple() {
    let input = r#"(Expr
      :id 1
      :kind (Break)
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Break { label, value } => {
            assert!(label.is_none());
            assert!(value.is_none());
        }
        _ => panic!("Expected Break expression"),
    }
}

#[test]
fn test_build_expr_break_with_label() {
    let input = r#"(Expr
      :id 1
      :kind (Break
              :label (Label :ident (Ident :name "outer")))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Break { label, value } => {
            assert!(label.is_some());
            assert!(value.is_none());
        }
        _ => panic!("Expected Break expression"),
    }
}

#[test]
fn test_build_expr_break_with_value() {
    let input = r#"(Expr
      :id 1
      :kind (Break
              :value (Expr :id 2 :kind (Lit (Lit :kind (Int 42)))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Break { label, value } => {
            assert!(label.is_none());
            assert!(value.is_some());
        }
        _ => panic!("Expected Break expression"),
    }
}

#[test]
fn test_build_expr_continue_simple() {
    let input = r#"(Expr
      :id 1
      :kind (Continue)
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Continue { label } => {
            assert!(label.is_none());
        }
        _ => panic!("Expected Continue expression"),
    }
}

#[test]
fn test_build_expr_continue_with_label() {
    let input = r#"(Expr
      :id 1
      :kind (Continue
              :label (Label :ident (Ident :name "outer")))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Continue { label } => {
            assert!(label.is_some());
        }
        _ => panic!("Expected Continue expression"),
    }
}

#[test]
fn test_build_expr_return_simple() {
    let input = r#"(Expr
      :id 1
      :kind (Return)
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Return { value } => {
            assert!(value.is_none());
        }
        _ => panic!("Expected Return expression"),
    }
}

#[test]
fn test_build_expr_return_with_value() {
    let input = r#"(Expr
      :id 1
      :kind (Return
              :value (Expr :id 2 :kind (Lit (Lit :kind (Int 42)))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Return { value } => {
            assert!(value.is_some());
        }
        _ => panic!("Expected Return expression"),
    }
}

// ===== Basic Expression Tests =====

#[test]
fn test_build_expr_path_simple() {
    let input = r#"(Expr
      :id 1
      :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "foo")))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Path(qself, path) => {
            assert!(qself.is_none());
            assert_eq!(path.segments.len(), 1);
            assert_eq!(path.segments[0].ident.name, "foo");
        }
        _ => panic!("Expected Path expression"),
    }
}

#[test]
fn test_build_expr_lit_int() {
    let input = r#"(Expr
      :id 1
      :kind (Lit (Lit :kind (Int 42)))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Lit(lit) => match lit.kind {
            LitKind::Int(n) => assert_eq!(n, 42),
            _ => panic!("Expected Int literal"),
        },
        _ => panic!("Expected Lit expression"),
    }
}

#[test]
fn test_build_expr_lit_str() {
    let input = r#"(Expr
      :id 1
      :kind (Lit (Lit :kind (Str "hello")))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Lit(lit) => match &lit.kind {
            LitKind::Str(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected Str literal"),
        },
        _ => panic!("Expected Lit expression"),
    }
}

// ===== Pattern Kind Tests =====

#[test]
fn test_build_pat_wild() {
    let input = r#"(Pat :kind Wild)"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    assert!(matches!(pat.kind, PatKind::Wild));
}

#[test]
fn test_build_pat_ident_simple() {
    let input = r#"(Pat :kind (Ident :ident (Ident :name "x")))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Ident { ident, binding_mode, sub } => {
            assert_eq!(ident.name, "x");
            assert!(matches!(binding_mode, BindingMode::ByValue(Mutability::Not)));
            assert!(sub.is_none());
        }
        _ => panic!("Expected Ident pattern"),
    }
}

#[test]
fn test_build_pat_ident_with_binding_mode_by_ref() {
    let input = r#"(Pat :kind (Ident
                              :ident (Ident :name "x")
                              :binding-mode (ByRef Mut)))"#;
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
fn test_build_pat_ident_with_binding_mode_by_value() {
    let input = r#"(Pat :kind (Ident
                              :ident (Ident :name "x")
                              :binding-mode (ByValue Mut)))"#;
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
fn test_build_pat_ident_with_sub() {
    let input = r#"(Pat :kind (Ident
                              :ident (Ident :name "x")
                              :sub (Pat :kind Wild)))"#;
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
    let input = r#"(Pat :kind (Struct
                              :path (Path :segments ((PathSegment :ident (Ident :name "Point"))))
                              :fields ((PatField
                                         :ident (Ident :name "x")
                                         :pat (Pat :kind Wild)))))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Struct { path, fields } => {
            assert_eq!(path.segments[0].ident.name, "Point");
            assert_eq!(fields.len(), 1);
        }
        _ => panic!("Expected Struct pattern"),
    }
}

#[test]
fn test_build_pat_struct_no_fields() {
    let input = r#"(Pat :kind (Struct
                              :path (Path :segments ((PathSegment :ident (Ident :name "Unit"))))))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Struct { fields, .. } => {
            assert!(fields.is_empty());
        }
        _ => panic!("Expected Struct pattern"),
    }
}

#[test]
fn test_build_pat_tuple_struct() {
    let input = r#"(Pat :kind (TupleStruct
                              :path (Path :segments ((PathSegment :ident (Ident :name "Some"))))
                              :elems ((Pat :kind Wild))))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::TupleStruct { path, elems } => {
            assert_eq!(path.segments[0].ident.name, "Some");
            assert_eq!(elems.len(), 1);
        }
        _ => panic!("Expected TupleStruct pattern"),
    }
}

#[test]
fn test_build_pat_tuple() {
    let input = r#"(Pat :kind (Tuple ((Pat :kind Wild) (Pat :kind Wild))))"#;
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
fn test_build_pat_slice() {
    let input = r#"(Pat :kind (Slice ((Pat :kind Wild) (Pat :kind Wild))))"#;
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
    let input = r#"(Pat :kind (Or ((Pat :kind Wild) (Pat :kind Wild))))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Or(pats) => {
            assert_eq!(pats.len(), 2);
        }
        _ => panic!("Expected Or pattern"),
    }
}

#[test]
fn test_build_pat_ref() {
    let input = r#"(Pat :kind (Ref :pat (Pat :kind Wild) :mutability Mut))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Ref { pat, mutability } => {
            assert!(matches!(pat.kind, PatKind::Wild));
            assert_eq!(mutability, Mutability::Mut);
        }
        _ => panic!("Expected Ref pattern"),
    }
}

#[test]
fn test_build_pat_ref_no_mutability() {
    let input = r#"(Pat :kind (Ref :pat (Pat :kind Wild)))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Ref { mutability, .. } => {
            assert_eq!(mutability, Mutability::Not);
        }
        _ => panic!("Expected Ref pattern"),
    }
}

#[test]
fn test_build_pat_lit() {
    let input = r#"(Pat :kind (Lit (Expr :id 1 :kind (Lit (Lit :kind (Int 42))))))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Lit(expr) => {
            assert!(matches!(expr.kind, ExprKind::Lit(_)));
        }
        _ => panic!("Expected Lit pattern"),
    }
}

#[test]
fn test_build_pat_box() {
    let input = r#"(Pat :kind (Box (Pat :kind Wild)))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Box(inner) => {
            assert!(matches!(inner.kind, PatKind::Wild));
        }
        _ => panic!("Expected Box pattern"),
    }
}

#[test]
fn test_build_pat_path() {
    let input =
        r#"(Pat :kind (Path :path (Path :segments ((PathSegment :ident (Ident :name "None"))))))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Path { qself, path } => {
            assert!(qself.is_none());
            assert_eq!(path.segments[0].ident.name, "None");
        }
        _ => panic!("Expected Path pattern"),
    }
}

#[test]
fn test_build_pat_path_with_qself() {
    let input = r#"(Pat :kind (Path :qself () :path (Path :segments ((PathSegment :ident (Ident :name "None"))))))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Path { qself, .. } => {
            assert!(qself.is_some());
        }
        _ => panic!("Expected Path pattern"),
    }
}

#[test]
fn test_build_pat_range() {
    let input = r#"(Pat :kind (Range
                              :start (Expr :id 1 :kind (Lit (Lit :kind (Int 0))))
                              :end (Expr :id 2 :kind (Lit (Lit :kind (Int 10))))
                              :limits Included))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Range { start, end, limits } => {
            assert!(start.is_some());
            assert!(end.is_some());
            assert_eq!(limits, RangeEnd::Included);
        }
        _ => panic!("Expected Range pattern"),
    }
}

#[test]
fn test_build_pat_range_excluded() {
    let input = r#"(Pat :kind (Range
                              :start (Expr :id 1 :kind (Lit (Lit :kind (Int 0))))
                              :end (Expr :id 2 :kind (Lit (Lit :kind (Int 10))))
                              :limits Excluded))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Range { limits, .. } => {
            assert_eq!(limits, RangeEnd::Excluded);
        }
        _ => panic!("Expected Range pattern"),
    }
}

#[test]
fn test_build_pat_rest() {
    let input = r#"(Pat :kind (Rest))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    assert!(matches!(pat.kind, PatKind::Rest));
}

#[test]
fn test_build_pat_paren() {
    let input = r#"(Pat :kind (Paren (Pat :kind Wild)))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Paren(inner) => {
            assert!(matches!(inner.kind, PatKind::Wild));
        }
        _ => panic!("Expected Paren pattern"),
    }
}

#[test]
fn test_build_pat_type() {
    let input = r#"(Pat :kind (Type
                              :pat (Pat :kind Wild)
                              :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Type { pat, ty } => {
            assert!(matches!(pat.kind, PatKind::Wild));
            assert!(matches!(ty.kind, TyKind::Path(_, _)));
        }
        _ => panic!("Expected Type pattern"),
    }
}

#[test]
fn test_build_pat_const() {
    let input = r#"(Pat :kind (Const
                              :value (Expr :id 1 :kind (Lit (Lit :kind (Int 42))))))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Const(const_block) => {
            assert!(matches!(const_block.value.kind, ExprKind::Lit(_)));
        }
        _ => panic!("Expected Const pattern"),
    }
}

#[test]
fn test_build_pat_mac_call() {
    let input = r#"(Pat :kind (MacCall
                              :mac (MacCall
                                     :path (Path :segments ((PathSegment :ident (Ident :name "pat")))))))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    assert!(matches!(pat.kind, PatKind::MacCall(_)));
}

#[test]
fn test_build_pat_err() {
    let input = r#"(Pat :kind (Err))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    assert!(matches!(pat.kind, PatKind::Err));
}

// ===== Arm Building Tests (via Match expression) =====

#[test]
fn test_build_arm_simple() {
    // Test arm building through Match expression
    let input = r#"(Expr
      :id 1
      :kind (Match
              :expr (Expr :id 2 :kind (Lit (Lit :kind (Int 1))))
              :arms ((Arm
                       :pat (Pat :kind Wild)
                       :body (Expr :id 3 :kind (Lit (Lit :kind (Int 42)))))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Match { arms, .. } => {
            assert_eq!(arms.len(), 1);
            assert!(matches!(arms[0].pat.kind, PatKind::Wild));
            assert!(arms[0].guard.is_none());
        }
        _ => panic!("Expected Match expression"),
    }
}

#[test]
fn test_build_arm_with_guard() {
    let input = r#"(Expr
      :id 1
      :kind (Match
              :expr (Expr :id 2 :kind (Lit (Lit :kind (Int 1))))
              :arms ((Arm
                       :pat (Pat :kind Wild)
                       :guard (Expr :id 3 :kind (Lit (Lit :kind (Int 1))))
                       :body (Expr :id 4 :kind (Lit (Lit :kind (Int 42)))))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Match { arms, .. } => {
            assert!(arms[0].guard.is_some());
        }
        _ => panic!("Expected Match expression"),
    }
}

#[test]
fn test_build_arm_with_span_and_id() {
    let input = r#"(Expr
      :id 1
      :kind (Match
              :expr (Expr :id 2 :kind (Lit (Lit :kind (Int 1))))
              :arms ((Arm
                       :pat (Pat :kind Wild)
                       :body (Expr :id 3 :kind (Lit (Lit :kind (Int 42))))
                       :span (Span :lo 10 :hi 20)
                       :id 99)))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Match { arms, .. } => {
            assert_eq!(arms[0].span.lo, 10);
            assert_eq!(arms[0].span.hi, 20);
            assert_eq!(arms[0].id, NodeId(99));
        }
        _ => panic!("Expected Match expression"),
    }
}

// ===== ExprField Building Tests (via Struct expression) =====

#[test]
fn test_build_expr_field_simple() {
    let input = r#"(Expr
      :id 1
      :kind (Struct
              :path (Path :segments ((PathSegment :ident (Ident :name "Point"))))
              :fields ((ExprField
                         :ident (Ident :name "x")
                         :expr (Expr :id 2 :kind (Lit (Lit :kind (Int 42)))))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Struct { fields, .. } => {
            assert_eq!(fields[0].ident.name, "x");
            assert!(!fields[0].is_shorthand);
        }
        _ => panic!("Expected Struct expression"),
    }
}

#[test]
fn test_build_expr_field_shorthand() {
    let input = r#"(Expr
      :id 1
      :kind (Struct
              :path (Path :segments ((PathSegment :ident (Ident :name "Point"))))
              :fields ((ExprField
                         :ident (Ident :name "x")
                         :expr (Expr :id 2 :kind (Lit (Lit :kind (Int 42))))
                         :is-shorthand true)))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Struct { fields, .. } => {
            assert!(fields[0].is_shorthand);
        }
        _ => panic!("Expected Struct expression"),
    }
}

#[test]
fn test_build_expr_field_with_span() {
    let input = r#"(Expr
      :id 1
      :kind (Struct
              :path (Path :segments ((PathSegment :ident (Ident :name "Point"))))
              :fields ((ExprField
                         :ident (Ident :name "x")
                         :expr (Expr :id 2 :kind (Lit (Lit :kind (Int 42))))
                         :span (Span :lo 5 :hi 15))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Struct { fields, .. } => {
            assert_eq!(fields[0].span.lo, 5);
            assert_eq!(fields[0].span.hi, 15);
        }
        _ => panic!("Expected Struct expression"),
    }
}

// ===== PatField Building Tests (via Struct pattern) =====

#[test]
fn test_build_pat_field_simple() {
    let input = r#"(Pat :kind (Struct
                              :path (Path :segments ((PathSegment :ident (Ident :name "Point"))))
                              :fields ((PatField
                                         :ident (Ident :name "x")
                                         :pat (Pat :kind Wild)))))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Struct { fields, .. } => {
            assert_eq!(fields[0].ident.name, "x");
            assert!(!fields[0].is_shorthand);
        }
        _ => panic!("Expected Struct pattern"),
    }
}

#[test]
fn test_build_pat_field_shorthand() {
    let input = r#"(Pat :kind (Struct
                              :path (Path :segments ((PathSegment :ident (Ident :name "Point"))))
                              :fields ((PatField
                                         :ident (Ident :name "x")
                                         :pat (Pat :kind Wild)
                                         :is-shorthand true))))"#;
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

// ===== Error Path Tests =====

#[test]
fn test_build_expr_kind_unsupported_variant() {
    let input = r#"(Expr
      :id 1
      :kind (UnsupportedKind foo bar)
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_expr(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_pat_kind_unsupported_symbol() {
    let input = r#"(Pat :kind UnsupportedSymbol)"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_pat(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_pat_kind_unsupported_variant() {
    let input = r#"(Pat :kind (UnsupportedPatKind))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_pat(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_lit_kind_unsupported() {
    let input = r#"(Expr
      :id 1
      :kind (Lit (Lit :kind (UnsupportedLit foo)))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_expr(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_range_end_invalid() {
    let input = r#"(Pat :kind (Range
                              :start (Expr :id 1 :kind (Lit (Lit :kind (Int 0))))
                              :limits InvalidRangeEnd))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_pat(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_binding_mode_invalid() {
    let input = r#"(Pat :kind (Ident
                              :ident (Ident :name "x")
                              :binding-mode (InvalidMode)))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_pat(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_delimiter_invalid() {
    let input = r#"(Expr
      :id 1
      :kind (MacCall
              (MacCall
                :path (Path :segments ((PathSegment :ident (Ident :name "test"))))
                :args (Delimited :delim InvalidDelimiter)))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_expr(&sexp);

    assert!(result.is_err());
}

#[test]
fn test_build_token_stream_invalid() {
    let input = r#"(Expr
      :id 1
      :kind (MacCall
              (MacCall
                :path (Path :segments ((PathSegment :ident (Ident :name "test"))))
                :args (Delimited :tokens (InvalidTokenType))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_expr(&sexp);

    assert!(result.is_err());
}

// ===== Block Building Additional Tests =====

#[test]
fn test_build_block_empty() {
    let input = r#"(Block :stmts ())"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let block = builder.build_block(&sexp).unwrap();

    assert!(block.stmts.is_empty());
}

#[test]
fn test_build_block_without_stmts_field() {
    let input = r#"(Block)"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let block = builder.build_block(&sexp).unwrap();

    assert!(block.stmts.is_empty());
}

#[test]
fn test_build_block_with_explicit_id() {
    let input = r#"(Block :stmts () :id 42)"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let block = builder.build_block(&sexp).unwrap();

    assert_eq!(block.id, NodeId(42));
}

#[test]
fn test_build_block_with_span() {
    let input = r#"(Block :stmts () :span (Span :lo 5 :hi 15))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let block = builder.build_block(&sexp).unwrap();

    assert_eq!(block.span.lo, 5);
    assert_eq!(block.span.hi, 15);
}

// ===== Expr Building Additional Tests =====

#[test]
fn test_build_expr_with_explicit_id() {
    let input = r#"(Expr
      :id 99
      :kind (Lit (Lit :kind (Int 42)))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    assert_eq!(expr.id, NodeId(99));
}

#[test]
fn test_build_expr_auto_generated_id() {
    let input = r#"(Expr
      :kind (Lit (Lit :kind (Int 42)))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    // Auto-generated ID starts at 0
    assert_eq!(expr.id, NodeId(0));
}

// ===== Lit Building Tests =====

#[test]
fn test_build_lit_with_span() {
    let input = r#"(Expr
      :id 1
      :kind (Lit (Lit :kind (Int 42) :span (Span :lo 5 :hi 7)))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::Lit(lit) => {
            assert_eq!(lit.span.lo, 5);
            assert_eq!(lit.span.hi, 7);
        }
        _ => panic!("Expected Lit expression"),
    }
}

// ===== Path Building Additional Tests =====

#[test]
fn test_build_path_no_segments_field() {
    let input = r#"(Path)"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let path = builder.build_path(&sexp).unwrap();

    assert!(path.segments.is_empty());
}

// ===== MacArgs Building Additional Tests =====

#[test]
fn test_build_mac_args_empty_as_list() {
    let input = r#"(Expr
      :id 1
      :kind (MacCall
              (MacCall
                :path (Path :segments ((PathSegment :ident (Ident :name "test"))))
                :args (Empty)))
      :span (Span :lo 0 :hi 10))"#;
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
fn test_build_token_stream_empty_as_list() {
    let input = r#"(Expr
      :id 1
      :kind (MacCall
              (MacCall
                :path (Path :segments ((PathSegment :ident (Ident :name "test"))))
                :args (Delimited :tokens (Empty))))
      :span (Span :lo 0 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    match expr.kind {
        ExprKind::MacCall(mac_call) => match mac_call.args {
            MacArgs::Delimited { tokens, .. } => {
                assert!(matches!(tokens, TokenStream::Empty));
            }
            _ => panic!("Expected Delimited"),
        },
        _ => panic!("Expected MacCall"),
    }
}

// ===== BindingMode Additional Tests =====

#[test]
fn test_build_binding_mode_by_ref_not() {
    let input = r#"(Pat :kind (Ident
                              :ident (Ident :name "x")
                              :binding-mode (ByRef Not)))"#;
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
fn test_build_binding_mode_by_value_not() {
    let input = r#"(Pat :kind (Ident
                              :ident (Ident :name "x")
                              :binding-mode (ByValue Not)))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Ident { binding_mode, .. } => {
            assert!(matches!(binding_mode, BindingMode::ByValue(Mutability::Not)));
        }
        _ => panic!("Expected Ident pattern"),
    }
}

#[test]
fn test_build_binding_mode_by_ref_no_mutability_arg() {
    let input = r#"(Pat :kind (Ident
                              :ident (Ident :name "x")
                              :binding-mode (ByRef)))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Ident { binding_mode, .. } => {
            // Defaults to Not when no mutability argument
            assert!(matches!(binding_mode, BindingMode::ByRef(Mutability::Not)));
        }
        _ => panic!("Expected Ident pattern"),
    }
}

#[test]
fn test_build_binding_mode_by_value_no_mutability_arg() {
    let input = r#"(Pat :kind (Ident
                              :ident (Ident :name "x")
                              :binding-mode (ByValue)))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Ident { binding_mode, .. } => {
            // Defaults to Not when no mutability argument
            assert!(matches!(binding_mode, BindingMode::ByValue(Mutability::Not)));
        }
        _ => panic!("Expected Ident pattern"),
    }
}

// Test unknown mutability value defaults to Not
#[test]
fn test_build_binding_mode_unknown_mutability() {
    let input = r#"(Pat :kind (Ident
                              :ident (Ident :name "x")
                              :binding-mode (ByRef SomeOther)))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Ident { binding_mode, .. } => {
            // Unknown mutability defaults to Not
            assert!(matches!(binding_mode, BindingMode::ByRef(Mutability::Not)));
        }
        _ => panic!("Expected Ident pattern"),
    }
}

// ===== Pat mutability tests =====

#[test]
fn test_build_pat_ref_mutability_not() {
    let input = r#"(Pat :kind (Ref :pat (Pat :kind Wild) :mutability Not))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Ref { mutability, .. } => {
            assert_eq!(mutability, Mutability::Not);
        }
        _ => panic!("Expected Ref pattern"),
    }
}

#[test]
fn test_build_pat_ref_unknown_mutability_defaults_to_not() {
    let input = r#"(Pat :kind (Ref :pat (Pat :kind Wild) :mutability Unknown))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Ref { mutability, .. } => {
            // Unknown value defaults to Not
            assert_eq!(mutability, Mutability::Not);
        }
        _ => panic!("Expected Ref pattern"),
    }
}

// ===== Pat with explicit id and span =====

#[test]
fn test_build_pat_with_explicit_id() {
    let input = r#"(Pat :kind Wild :id 42)"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    assert_eq!(pat.id, NodeId(42));
}

#[test]
fn test_build_pat_with_span() {
    let input = r#"(Pat :kind Wild :span (Span :lo 5 :hi 10))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    assert_eq!(pat.span.lo, 5);
    assert_eq!(pat.span.hi, 10);
}

// ===== Missing :kind field error =====

#[test]
fn test_build_pat_missing_kind() {
    let input = r#"(Pat :id 1)"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let result = builder.build_pat(&sexp);

    assert!(result.is_err());
}

// ===== TupleStruct with empty elems =====

#[test]
fn test_build_pat_tuple_struct_empty_elems() {
    let input = r#"(Pat :kind (TupleStruct
                              :path (Path :segments ((PathSegment :ident (Ident :name "Unit"))))))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::TupleStruct { elems, .. } => {
            assert!(elems.is_empty());
        }
        _ => panic!("Expected TupleStruct pattern"),
    }
}

// ===== Range pattern with default limits =====

#[test]
fn test_build_pat_range_default_limits() {
    let input = r#"(Pat :kind (Range
                              :start (Expr :id 1 :kind (Lit (Lit :kind (Int 0))))
                              :end (Expr :id 2 :kind (Lit (Lit :kind (Int 10))))))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Range { limits, .. } => {
            // Default is Excluded
            assert_eq!(limits, RangeEnd::Excluded);
        }
        _ => panic!("Expected Range pattern"),
    }
}

// ===== Const pattern with explicit id =====

#[test]
fn test_build_pat_const_with_id() {
    let input = r#"(Pat :kind (Const
                              :id 99
                              :value (Expr :id 1 :kind (Lit (Lit :kind (Int 42))))))"#;
    let sexp = Parser::parse_str(input).unwrap();
    let mut builder = AstBuilder::new();
    let pat = builder.build_pat(&sexp).unwrap();

    match pat.kind {
        PatKind::Const(const_block) => {
            assert_eq!(const_block.id, NodeId(99));
        }
        _ => panic!("Expected Const pattern"),
    }
}
