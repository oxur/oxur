/// Comprehensive tests for generator module to improve coverage to 95%+
/// This file focuses on testing all expression, item, and statement variants
use oxur_ast::ast::*;
use oxur_ast::sexp::print_sexp;
use oxur_ast::*;

// ============================================================================
// EXPRESSION TESTS - targeting expr.rs coverage
// ============================================================================

#[test]
fn test_generate_expr_lit_str() {
    let gen = Generator::new();
    let lit = Lit { kind: LitKind::Str("hello".to_string()), span: Span::new(0, 7) };
    let expr = Expr {
        id: NodeId(1),
        kind: ExprKind::Lit(lit),
        span: Span::new(0, 7),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Lit"));
    assert!(output.contains("Str"));
    assert!(output.contains("hello"));
}

#[test]
fn test_generate_expr_lit_int() {
    let gen = Generator::new();
    let lit = Lit { kind: LitKind::Int(42), span: Span::new(0, 2) };
    let expr = Expr {
        id: NodeId(1),
        kind: ExprKind::Lit(lit),
        span: Span::new(0, 2),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Lit"));
    assert!(output.contains("Int"));
    assert!(output.contains("42"));
}

#[test]
fn test_generate_expr_path() {
    let gen = Generator::new();
    let ident = Ident::new("foo", Span::new(0, 3));
    let segment = PathSegment::from_ident(ident);
    let path = Path { span: Span::new(0, 3), segments: vec![segment], tokens: None };

    let expr = Expr {
        id: NodeId(1),
        kind: ExprKind::Path(None, path),
        span: Span::new(0, 3),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Path"));
    assert!(output.contains("foo"));
    assert!(output.contains("nil"));
}

#[test]
fn test_generate_expr_if_without_else() {
    let gen = Generator::new();
    let cond = Box::new(Expr {
        id: NodeId(1),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(1), span: Span::new(0, 1) }),
        span: Span::new(0, 1),
        attrs: vec![],
        tokens: None,
    });

    let then_branch = Block {
        stmts: vec![],
        id: NodeId(2),
        rules: BlockCheckMode::Default,
        span: Span::new(2, 4),
        tokens: None,
        could_be_bare_literal: false,
    };

    let expr = Expr {
        id: NodeId(3),
        kind: ExprKind::If { cond, then_branch, else_branch: None },
        span: Span::new(0, 4),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("If"));
    assert!(output.contains(":cond"));
    assert!(output.contains(":then"));
    assert!(output.contains(":else"));
    assert!(output.contains("nil"));
}

#[test]
fn test_generate_expr_if_with_else() {
    let gen = Generator::new();
    let cond = Box::new(Expr {
        id: NodeId(1),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(1), span: Span::new(0, 1) }),
        span: Span::new(0, 1),
        attrs: vec![],
        tokens: None,
    });

    let then_branch = Block {
        stmts: vec![],
        id: NodeId(2),
        rules: BlockCheckMode::Default,
        span: Span::new(2, 4),
        tokens: None,
        could_be_bare_literal: false,
    };

    let else_branch = Some(Box::new(Expr {
        id: NodeId(4),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(2), span: Span::new(5, 6) }),
        span: Span::new(5, 6),
        attrs: vec![],
        tokens: None,
    }));

    let expr = Expr {
        id: NodeId(3),
        kind: ExprKind::If { cond, then_branch, else_branch },
        span: Span::new(0, 6),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("If"));
    assert!(output.contains(":else"));
    assert!(!output.contains(":else nil") || output.matches(":else").count() > 1);
}

#[test]
fn test_generate_expr_match() {
    let gen = Generator::new();
    let expr = Box::new(Expr {
        id: NodeId(1),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(1), span: Span::new(0, 1) }),
        span: Span::new(0, 1),
        attrs: vec![],
        tokens: None,
    });

    let arm = Arm {
        attrs: vec![],
        pat: Pat { id: NodeId(2), kind: PatKind::Wild, span: Span::new(2, 3), tokens: None },
        guard: None,
        body: Box::new(Expr {
            id: NodeId(3),
            kind: ExprKind::Lit(Lit { kind: LitKind::Int(2), span: Span::new(4, 5) }),
            span: Span::new(4, 5),
            attrs: vec![],
            tokens: None,
        }),
        span: Span::new(2, 5),
        id: NodeId(4),
    };

    let match_expr = Expr {
        id: NodeId(5),
        kind: ExprKind::Match { expr, arms: vec![arm] },
        span: Span::new(0, 6),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&match_expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Match"));
    assert!(output.contains(":expr"));
    assert!(output.contains(":arms"));
    assert!(output.contains("Arm"));
}

#[test]
fn test_generate_expr_match_with_guard() {
    let gen = Generator::new();
    let expr = Box::new(Expr {
        id: NodeId(1),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(1), span: Span::new(0, 1) }),
        span: Span::new(0, 1),
        attrs: vec![],
        tokens: None,
    });

    let guard = Some(Box::new(Expr {
        id: NodeId(6),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(3), span: Span::new(3, 4) }),
        span: Span::new(3, 4),
        attrs: vec![],
        tokens: None,
    }));

    let arm = Arm {
        attrs: vec![],
        pat: Pat { id: NodeId(2), kind: PatKind::Wild, span: Span::new(2, 3), tokens: None },
        guard,
        body: Box::new(Expr {
            id: NodeId(3),
            kind: ExprKind::Lit(Lit { kind: LitKind::Int(2), span: Span::new(5, 6) }),
            span: Span::new(5, 6),
            attrs: vec![],
            tokens: None,
        }),
        span: Span::new(2, 6),
        id: NodeId(4),
    };

    let match_expr = Expr {
        id: NodeId(5),
        kind: ExprKind::Match { expr, arms: vec![arm] },
        span: Span::new(0, 7),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&match_expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Match"));
    assert!(output.contains(":guard"));
}

#[test]
fn test_generate_expr_while_without_label() {
    let gen = Generator::new();
    let cond = Box::new(Expr {
        id: NodeId(1),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(1), span: Span::new(0, 1) }),
        span: Span::new(0, 1),
        attrs: vec![],
        tokens: None,
    });

    let body = Block {
        stmts: vec![],
        id: NodeId(2),
        rules: BlockCheckMode::Default,
        span: Span::new(2, 4),
        tokens: None,
        could_be_bare_literal: false,
    };

    let expr = Expr {
        id: NodeId(3),
        kind: ExprKind::While { label: None, cond, body },
        span: Span::new(0, 4),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("While"));
    assert!(output.contains(":label"));
    assert!(output.contains(":cond"));
    assert!(output.contains(":body"));
}

#[test]
fn test_generate_expr_while_with_label() {
    let gen = Generator::new();
    let label = Some(Label { ident: Ident::new("outer", Span::new(0, 5)) });
    let cond = Box::new(Expr {
        id: NodeId(1),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(1), span: Span::new(6, 7) }),
        span: Span::new(6, 7),
        attrs: vec![],
        tokens: None,
    });

    let body = Block {
        stmts: vec![],
        id: NodeId(2),
        rules: BlockCheckMode::Default,
        span: Span::new(8, 10),
        tokens: None,
        could_be_bare_literal: false,
    };

    let expr = Expr {
        id: NodeId(3),
        kind: ExprKind::While { label, cond, body },
        span: Span::new(0, 10),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("While"));
    assert!(output.contains("Label"));
    assert!(output.contains("outer"));
}

#[test]
fn test_generate_expr_for_loop_without_label() {
    let gen = Generator::new();
    let pat = Pat {
        id: NodeId(1),
        kind: PatKind::Ident {
            binding_mode: BindingMode::ByValue(Mutability::Not),
            ident: Ident::new("i", Span::new(0, 1)),
            sub: None,
        },
        span: Span::new(0, 1),
        tokens: None,
    };

    let iter = Box::new(Expr {
        id: NodeId(2),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(10), span: Span::new(2, 4) }),
        span: Span::new(2, 4),
        attrs: vec![],
        tokens: None,
    });

    let body = Block {
        stmts: vec![],
        id: NodeId(3),
        rules: BlockCheckMode::Default,
        span: Span::new(5, 7),
        tokens: None,
        could_be_bare_literal: false,
    };

    let expr = Expr {
        id: NodeId(4),
        kind: ExprKind::ForLoop { label: None, pat, iter, body },
        span: Span::new(0, 7),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("ForLoop"));
    assert!(output.contains(":label"));
    assert!(output.contains(":pat"));
    assert!(output.contains(":iter"));
    assert!(output.contains(":body"));
}

#[test]
fn test_generate_expr_for_loop_with_label() {
    let gen = Generator::new();
    let label = Some(Label { ident: Ident::new("loop1", Span::new(0, 5)) });
    let pat = Pat { id: NodeId(1), kind: PatKind::Wild, span: Span::new(6, 7), tokens: None };

    let iter = Box::new(Expr {
        id: NodeId(2),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(5), span: Span::new(8, 9) }),
        span: Span::new(8, 9),
        attrs: vec![],
        tokens: None,
    });

    let body = Block {
        stmts: vec![],
        id: NodeId(3),
        rules: BlockCheckMode::Default,
        span: Span::new(10, 12),
        tokens: None,
        could_be_bare_literal: false,
    };

    let expr = Expr {
        id: NodeId(4),
        kind: ExprKind::ForLoop { label, pat, iter, body },
        span: Span::new(0, 12),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("ForLoop"));
    assert!(output.contains("Label"));
    assert!(output.contains("loop1"));
}

#[test]
fn test_generate_expr_loop_without_label() {
    let gen = Generator::new();
    let body = Block {
        stmts: vec![],
        id: NodeId(1),
        rules: BlockCheckMode::Default,
        span: Span::new(0, 2),
        tokens: None,
        could_be_bare_literal: false,
    };

    let expr = Expr {
        id: NodeId(2),
        kind: ExprKind::Loop { label: None, body },
        span: Span::new(0, 2),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Loop"));
    assert!(output.contains(":label"));
    assert!(output.contains(":body"));
}

#[test]
fn test_generate_expr_loop_with_label() {
    let gen = Generator::new();
    let label = Some(Label { ident: Ident::new("forever", Span::new(0, 7)) });
    let body = Block {
        stmts: vec![],
        id: NodeId(1),
        rules: BlockCheckMode::Default,
        span: Span::new(8, 10),
        tokens: None,
        could_be_bare_literal: false,
    };

    let expr = Expr {
        id: NodeId(2),
        kind: ExprKind::Loop { label, body },
        span: Span::new(0, 10),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Loop"));
    assert!(output.contains("Label"));
    assert!(output.contains("forever"));
}

#[test]
fn test_generate_expr_binary_all_ops() {
    let gen = Generator::new();
    let ops = vec![
        (BinOp::Add, "Add"),
        (BinOp::Sub, "Sub"),
        (BinOp::Mul, "Mul"),
        (BinOp::Div, "Div"),
        (BinOp::Rem, "Rem"),
        (BinOp::And, "And"),
        (BinOp::Or, "Or"),
        (BinOp::BitAnd, "BitAnd"),
        (BinOp::BitOr, "BitOr"),
        (BinOp::BitXor, "BitXor"),
        (BinOp::Shl, "Shl"),
        (BinOp::Shr, "Shr"),
        (BinOp::Eq, "Eq"),
        (BinOp::Ne, "Ne"),
        (BinOp::Lt, "Lt"),
        (BinOp::Le, "Le"),
        (BinOp::Gt, "Gt"),
        (BinOp::Ge, "Ge"),
    ];

    for (op, op_name) in ops {
        let left = Box::new(Expr {
            id: NodeId(1),
            kind: ExprKind::Lit(Lit { kind: LitKind::Int(1), span: Span::new(0, 1) }),
            span: Span::new(0, 1),
            attrs: vec![],
            tokens: None,
        });

        let right = Box::new(Expr {
            id: NodeId(2),
            kind: ExprKind::Lit(Lit { kind: LitKind::Int(2), span: Span::new(2, 3) }),
            span: Span::new(2, 3),
            attrs: vec![],
            tokens: None,
        });

        let expr = Expr {
            id: NodeId(3),
            kind: ExprKind::Binary { left, op, right },
            span: Span::new(0, 3),
            attrs: vec![],
            tokens: None,
        };

        let sexp = gen.generate_expr(&expr).unwrap();
        let output = print_sexp(&sexp);
        assert!(output.contains("Binary"), "Failed for op: {}", op_name);
        assert!(output.contains(op_name), "Failed for op: {}", op_name);
    }
}

#[test]
fn test_generate_expr_unary_all_ops() {
    let gen = Generator::new();
    let ops = vec![(UnOp::Not, "Not"), (UnOp::Neg, "Neg"), (UnOp::Deref, "Deref")];

    for (op, op_name) in ops {
        let inner_expr = Box::new(Expr {
            id: NodeId(1),
            kind: ExprKind::Lit(Lit { kind: LitKind::Int(1), span: Span::new(1, 2) }),
            span: Span::new(1, 2),
            attrs: vec![],
            tokens: None,
        });

        let expr = Expr {
            id: NodeId(2),
            kind: ExprKind::Unary { op, expr: inner_expr },
            span: Span::new(0, 2),
            attrs: vec![],
            tokens: None,
        };

        let sexp = gen.generate_expr(&expr).unwrap();
        let output = print_sexp(&sexp);
        assert!(output.contains("Unary"), "Failed for op: {}", op_name);
        assert!(output.contains(op_name), "Failed for op: {}", op_name);
    }
}

#[test]
fn test_generate_expr_call() {
    let gen = Generator::new();
    let func = Box::new(Expr {
        id: NodeId(1),
        kind: ExprKind::Path(
            None,
            Path {
                span: Span::new(0, 3),
                segments: vec![PathSegment::from_ident(Ident::new("foo", Span::new(0, 3)))],
                tokens: None,
            },
        ),
        span: Span::new(0, 3),
        attrs: vec![],
        tokens: None,
    });

    let args = vec![Expr {
        id: NodeId(2),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(42), span: Span::new(4, 6) }),
        span: Span::new(4, 6),
        attrs: vec![],
        tokens: None,
    }];

    let expr = Expr {
        id: NodeId(3),
        kind: ExprKind::Call { func, args },
        span: Span::new(0, 7),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Call"));
    assert!(output.contains(":func"));
    assert!(output.contains(":args"));
}

#[test]
fn test_generate_expr_method_call() {
    let gen = Generator::new();
    let receiver = Box::new(Expr {
        id: NodeId(1),
        kind: ExprKind::Path(
            None,
            Path {
                span: Span::new(0, 3),
                segments: vec![PathSegment::from_ident(Ident::new("obj", Span::new(0, 3)))],
                tokens: None,
            },
        ),
        span: Span::new(0, 3),
        attrs: vec![],
        tokens: None,
    });

    let method = Ident::new("foo", Span::new(4, 7));
    let args = vec![Expr {
        id: NodeId(2),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(1), span: Span::new(8, 9) }),
        span: Span::new(8, 9),
        attrs: vec![],
        tokens: None,
    }];

    let expr = Expr {
        id: NodeId(3),
        kind: ExprKind::MethodCall { receiver, method, args },
        span: Span::new(0, 10),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("MethodCall"));
    assert!(output.contains(":receiver"));
    assert!(output.contains(":method"));
    assert!(output.contains(":args"));
}

#[test]
fn test_generate_expr_array() {
    let gen = Generator::new();
    let elems = vec![
        Expr {
            id: NodeId(1),
            kind: ExprKind::Lit(Lit { kind: LitKind::Int(1), span: Span::new(0, 1) }),
            span: Span::new(0, 1),
            attrs: vec![],
            tokens: None,
        },
        Expr {
            id: NodeId(2),
            kind: ExprKind::Lit(Lit { kind: LitKind::Int(2), span: Span::new(2, 3) }),
            span: Span::new(2, 3),
            attrs: vec![],
            tokens: None,
        },
    ];

    let expr = Expr {
        id: NodeId(3),
        kind: ExprKind::Array(elems),
        span: Span::new(0, 4),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Array"));
}

#[test]
fn test_generate_expr_tuple() {
    let gen = Generator::new();
    let elems = vec![
        Expr {
            id: NodeId(1),
            kind: ExprKind::Lit(Lit { kind: LitKind::Int(1), span: Span::new(0, 1) }),
            span: Span::new(0, 1),
            attrs: vec![],
            tokens: None,
        },
        Expr {
            id: NodeId(2),
            kind: ExprKind::Lit(Lit { kind: LitKind::Str("x".to_string()), span: Span::new(2, 5) }),
            span: Span::new(2, 5),
            attrs: vec![],
            tokens: None,
        },
    ];

    let expr = Expr {
        id: NodeId(3),
        kind: ExprKind::Tuple(elems),
        span: Span::new(0, 6),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Tuple"));
}

#[test]
fn test_generate_expr_field() {
    let gen = Generator::new();
    let inner = Box::new(Expr {
        id: NodeId(1),
        kind: ExprKind::Path(
            None,
            Path {
                span: Span::new(0, 3),
                segments: vec![PathSegment::from_ident(Ident::new("obj", Span::new(0, 3)))],
                tokens: None,
            },
        ),
        span: Span::new(0, 3),
        attrs: vec![],
        tokens: None,
    });

    let field = Ident::new("x", Span::new(4, 5));

    let expr = Expr {
        id: NodeId(2),
        kind: ExprKind::Field { expr: inner, field },
        span: Span::new(0, 5),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Field"));
    assert!(output.contains(":expr"));
    assert!(output.contains(":field"));
}

#[test]
fn test_generate_expr_index() {
    let gen = Generator::new();
    let array = Box::new(Expr {
        id: NodeId(1),
        kind: ExprKind::Path(
            None,
            Path {
                span: Span::new(0, 3),
                segments: vec![PathSegment::from_ident(Ident::new("arr", Span::new(0, 3)))],
                tokens: None,
            },
        ),
        span: Span::new(0, 3),
        attrs: vec![],
        tokens: None,
    });

    let index = Box::new(Expr {
        id: NodeId(2),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(0), span: Span::new(4, 5) }),
        span: Span::new(4, 5),
        attrs: vec![],
        tokens: None,
    });

    let expr = Expr {
        id: NodeId(3),
        kind: ExprKind::Index { expr: array, index },
        span: Span::new(0, 6),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Index"));
    assert!(output.contains(":expr"));
    assert!(output.contains(":index"));
}

#[test]
fn test_generate_expr_assign() {
    let gen = Generator::new();
    let left = Box::new(Expr {
        id: NodeId(1),
        kind: ExprKind::Path(
            None,
            Path {
                span: Span::new(0, 1),
                segments: vec![PathSegment::from_ident(Ident::new("x", Span::new(0, 1)))],
                tokens: None,
            },
        ),
        span: Span::new(0, 1),
        attrs: vec![],
        tokens: None,
    });

    let right = Box::new(Expr {
        id: NodeId(2),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(5), span: Span::new(4, 5) }),
        span: Span::new(4, 5),
        attrs: vec![],
        tokens: None,
    });

    let expr = Expr {
        id: NodeId(3),
        kind: ExprKind::Assign { left, right },
        span: Span::new(0, 5),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Assign"));
    assert!(output.contains(":left"));
    assert!(output.contains(":right"));
}

#[test]
fn test_generate_expr_struct() {
    let gen = Generator::new();
    let path = Path {
        span: Span::new(0, 5),
        segments: vec![PathSegment::from_ident(Ident::new("Point", Span::new(0, 5)))],
        tokens: None,
    };

    let fields = vec![ExprField {
        attrs: vec![],
        ident: Ident::new("x", Span::new(6, 7)),
        expr: Expr {
            id: NodeId(1),
            kind: ExprKind::Lit(Lit { kind: LitKind::Int(1), span: Span::new(9, 10) }),
            span: Span::new(9, 10),
            attrs: vec![],
            tokens: None,
        },
        is_shorthand: false,
        span: Span::new(6, 10),
    }];

    let expr = Expr {
        id: NodeId(2),
        kind: ExprKind::Struct { path, fields },
        span: Span::new(0, 11),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Struct"));
    assert!(output.contains(":path"));
    assert!(output.contains(":fields"));
    assert!(output.contains("ExprField"));
}

// Note: generate_expr_field is private, so we test it via ExprKind::Struct
#[test]
fn test_generate_expr_field_shorthand() {
    let gen = Generator::new();
    let path = Path {
        span: Span::new(0, 5),
        segments: vec![PathSegment::from_ident(Ident::new("Point", Span::new(0, 5)))],
        tokens: None,
    };

    let field = ExprField {
        attrs: vec![],
        ident: Ident::new("x", Span::new(6, 7)),
        expr: Expr {
            id: NodeId(1),
            kind: ExprKind::Lit(Lit { kind: LitKind::Int(1), span: Span::new(6, 7) }),
            span: Span::new(6, 7),
            attrs: vec![],
            tokens: None,
        },
        is_shorthand: true,
        span: Span::new(6, 7),
    };

    let expr = Expr {
        id: NodeId(2),
        kind: ExprKind::Struct { path, fields: vec![field] },
        span: Span::new(0, 8),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("ExprField"));
    assert!(output.contains(":is-shorthand"));
    assert!(output.contains("true"));
}

#[test]
fn test_generate_expr_closure() {
    let gen = Generator::new();
    let params = vec![Param {
        attrs: vec![],
        ty: Ty { id: NodeId(1), kind: TyKind::Infer, span: Span::new(1, 2), tokens: None },
        pat: Pat {
            id: NodeId(2),
            kind: PatKind::Ident {
                binding_mode: BindingMode::ByValue(Mutability::Not),
                ident: Ident::new("x", Span::new(1, 2)),
                sub: None,
            },
            span: Span::new(1, 2),
            tokens: None,
        },
        id: NodeId(3),
        span: Span::new(1, 2),
        is_placeholder: false,
    }];

    let body = Box::new(Expr {
        id: NodeId(4),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(1), span: Span::new(4, 5) }),
        span: Span::new(4, 5),
        attrs: vec![],
        tokens: None,
    });

    let expr = Expr {
        id: NodeId(5),
        kind: ExprKind::Closure { params, body },
        span: Span::new(0, 6),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Closure"));
    assert!(output.contains(":params"));
    assert!(output.contains(":body"));
}

#[test]
fn test_generate_expr_range_full() {
    let gen = Generator::new();
    let start = Some(Box::new(Expr {
        id: NodeId(1),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(0), span: Span::new(0, 1) }),
        span: Span::new(0, 1),
        attrs: vec![],
        tokens: None,
    }));

    let end = Some(Box::new(Expr {
        id: NodeId(2),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(10), span: Span::new(3, 5) }),
        span: Span::new(3, 5),
        attrs: vec![],
        tokens: None,
    }));

    let expr = Expr {
        id: NodeId(3),
        kind: ExprKind::Range { start, end, inclusive: false },
        span: Span::new(0, 5),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Range"));
    assert!(output.contains(":start"));
    assert!(output.contains(":end"));
    assert!(output.contains(":inclusive"));
    assert!(output.contains("false"));
}

#[test]
fn test_generate_expr_range_inclusive() {
    let gen = Generator::new();
    let start = Some(Box::new(Expr {
        id: NodeId(1),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(0), span: Span::new(0, 1) }),
        span: Span::new(0, 1),
        attrs: vec![],
        tokens: None,
    }));

    let end = Some(Box::new(Expr {
        id: NodeId(2),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(10), span: Span::new(4, 6) }),
        span: Span::new(4, 6),
        attrs: vec![],
        tokens: None,
    }));

    let expr = Expr {
        id: NodeId(3),
        kind: ExprKind::Range { start, end, inclusive: true },
        span: Span::new(0, 6),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Range"));
    assert!(output.contains("true"));
}

#[test]
fn test_generate_expr_range_to() {
    let gen = Generator::new();
    let end = Some(Box::new(Expr {
        id: NodeId(1),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(10), span: Span::new(2, 4) }),
        span: Span::new(2, 4),
        attrs: vec![],
        tokens: None,
    }));

    let expr = Expr {
        id: NodeId(2),
        kind: ExprKind::Range { start: None, end, inclusive: false },
        span: Span::new(0, 4),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Range"));
    assert!(output.contains(":start"));
    assert!(output.contains("nil"));
}

#[test]
fn test_generate_expr_range_from() {
    let gen = Generator::new();
    let start = Some(Box::new(Expr {
        id: NodeId(1),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(0), span: Span::new(0, 1) }),
        span: Span::new(0, 1),
        attrs: vec![],
        tokens: None,
    }));

    let expr = Expr {
        id: NodeId(2),
        kind: ExprKind::Range { start, end: None, inclusive: false },
        span: Span::new(0, 2),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Range"));
    assert!(output.contains(":end"));
    assert!(output.contains("nil"));
}

#[test]
fn test_generate_expr_range_full_unbounded() {
    let gen = Generator::new();
    let expr = Expr {
        id: NodeId(1),
        kind: ExprKind::Range { start: None, end: None, inclusive: false },
        span: Span::new(0, 2),
        attrs: vec![],
        tokens: None,
    };

    let sexp = gen.generate_expr(&expr).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Range"));
    assert!(output.contains(":start"));
    assert!(output.contains(":end"));
}

// ============================================================================
// ITEM TESTS - targeting item.rs coverage
// ============================================================================

#[test]
fn test_generate_item_struct_unit() {
    let gen = Generator::new();
    let item = Item {
        ident: Ident::new("Unit", Span::new(0, 4)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Struct(VariantData::Unit),
        vis: Visibility::Public,
        span: Span::new(0, 10),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Struct"));
    assert!(output.contains("Unit"));
}

#[test]
fn test_generate_item_struct_tuple() {
    let gen = Generator::new();
    let fields = vec![FieldDef {
        attrs: vec![],
        id: NodeId(2),
        span: Span::new(5, 8),
        vis: Visibility::Inherited,
        ident: None,
        ty: Ty { id: NodeId(3), kind: TyKind::Infer, span: Span::new(5, 8), tokens: None },
    }];

    let item = Item {
        ident: Ident::new("Tuple", Span::new(0, 5)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Struct(VariantData::Tuple(fields)),
        vis: Visibility::Inherited,
        span: Span::new(0, 10),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Struct"));
    assert!(output.contains("Tuple"));
    assert!(output.contains("FieldDef"));
}

#[test]
fn test_generate_item_struct_regular() {
    let gen = Generator::new();
    let fields = vec![FieldDef {
        attrs: vec![],
        id: NodeId(2),
        span: Span::new(5, 10),
        vis: Visibility::Public,
        ident: Some(Ident::new("field", Span::new(5, 10))),
        ty: Ty { id: NodeId(3), kind: TyKind::Infer, span: Span::new(12, 15), tokens: None },
    }];

    let item = Item {
        ident: Ident::new("MyStruct", Span::new(0, 8)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Struct(VariantData::Struct { fields, recovered: false }),
        vis: Visibility::Inherited,
        span: Span::new(0, 20),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Struct"));
    assert!(output.contains("MyStruct"));
    assert!(output.contains("FieldDef"));
    assert!(output.contains("field"));
}

#[test]
fn test_generate_item_enum() {
    let gen = Generator::new();
    let variant = Variant {
        attrs: vec![],
        id: NodeId(2),
        span: Span::new(5, 8),
        vis: Visibility::Inherited,
        ident: Ident::new("A", Span::new(5, 6)),
        data: VariantData::Unit,
        disr_expr: None,
    };

    let enum_def = EnumDef { variants: vec![variant] };

    let item = Item {
        ident: Ident::new("MyEnum", Span::new(0, 6)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Enum(enum_def),
        vis: Visibility::Public,
        span: Span::new(0, 10),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Enum"));
    assert!(output.contains("MyEnum"));
    assert!(output.contains("Variant"));
}

#[test]
fn test_generate_item_enum_with_discriminant() {
    let gen = Generator::new();
    let variant = Variant {
        attrs: vec![],
        id: NodeId(2),
        span: Span::new(5, 10),
        vis: Visibility::Inherited,
        ident: Ident::new("A", Span::new(5, 6)),
        data: VariantData::Unit,
        disr_expr: Some(Expr {
            id: NodeId(3),
            kind: ExprKind::Lit(Lit { kind: LitKind::Int(42), span: Span::new(9, 11) }),
            span: Span::new(9, 11),
            attrs: vec![],
            tokens: None,
        }),
    };

    let enum_def = EnumDef { variants: vec![variant] };

    let item = Item {
        ident: Ident::new("MyEnum", Span::new(0, 6)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Enum(enum_def),
        vis: Visibility::Inherited,
        span: Span::new(0, 12),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Enum"));
    assert!(output.contains(":disr-expr"));
}

#[test]
fn test_generate_item_trait() {
    let gen = Generator::new();
    let trait_def = TraitDef {
        safety: Safety::Safe,
        generics: Generics::empty(),
        bounds: vec![],
        items: vec![],
    };

    let item = Item {
        ident: Ident::new("MyTrait", Span::new(0, 7)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Trait(Box::new(trait_def)),
        vis: Visibility::Public,
        span: Span::new(0, 20),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Trait"));
    assert!(output.contains("MyTrait"));
    assert!(output.contains(":safety"));
}

#[test]
fn test_generate_item_trait_unsafe() {
    let gen = Generator::new();
    let trait_def = TraitDef {
        safety: Safety::Unsafe,
        generics: Generics::empty(),
        bounds: vec![],
        items: vec![],
    };

    let item = Item {
        ident: Ident::new("UnsafeTrait", Span::new(0, 11)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Trait(Box::new(trait_def)),
        vis: Visibility::Inherited,
        span: Span::new(0, 30),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Trait"));
    assert!(output.contains("Unsafe"));
}

#[test]
fn test_generate_item_impl_without_trait() {
    let gen = Generator::new();
    let impl_def = ImplDef {
        safety: Safety::Safe,
        generics: Generics::empty(),
        of_trait: None,
        self_ty: Ty { id: NodeId(2), kind: TyKind::Infer, span: Span::new(5, 8), tokens: None },
        items: vec![],
    };

    let item = Item {
        ident: Ident::new("_", Span::new(0, 1)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Impl(Box::new(impl_def)),
        vis: Visibility::Inherited,
        span: Span::new(0, 20),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Impl"));
    assert!(output.contains(":of-trait"));
    assert!(output.contains("nil"));
}

#[test]
fn test_generate_item_impl_with_trait() {
    let gen = Generator::new();
    let trait_ref = TraitRef {
        path: Path {
            span: Span::new(5, 12),
            segments: vec![PathSegment::from_ident(Ident::new("MyTrait", Span::new(5, 12)))],
            tokens: None,
        },
    };

    let impl_def = ImplDef {
        safety: Safety::Safe,
        generics: Generics::empty(),
        of_trait: Some(trait_ref),
        self_ty: Ty { id: NodeId(2), kind: TyKind::Infer, span: Span::new(17, 20), tokens: None },
        items: vec![],
    };

    let item = Item {
        ident: Ident::new("_", Span::new(0, 1)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Impl(Box::new(impl_def)),
        vis: Visibility::Inherited,
        span: Span::new(0, 30),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Impl"));
    assert!(output.contains(":of-trait"));
    assert!(output.contains("TraitRef"));
}

#[test]
fn test_generate_item_use_simple() {
    let gen = Generator::new();
    let use_tree = UseTree {
        prefix: Path {
            span: Span::new(4, 7),
            segments: vec![PathSegment::from_ident(Ident::new("foo", Span::new(4, 7)))],
            tokens: None,
        },
        kind: UseTreeKind::Simple(None),
    };

    let item = Item {
        ident: Ident::new("_", Span::new(0, 1)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Use(use_tree),
        vis: Visibility::Inherited,
        span: Span::new(0, 8),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Use"));
    assert!(output.contains("UseTree"));
    assert!(output.contains("Simple"));
}

#[test]
fn test_generate_item_use_simple_with_rename() {
    let gen = Generator::new();
    let use_tree = UseTree {
        prefix: Path {
            span: Span::new(4, 7),
            segments: vec![PathSegment::from_ident(Ident::new("foo", Span::new(4, 7)))],
            tokens: None,
        },
        kind: UseTreeKind::Simple(Some(Ident::new("bar", Span::new(11, 14)))),
    };

    let item = Item {
        ident: Ident::new("_", Span::new(0, 1)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Use(use_tree),
        vis: Visibility::Public,
        span: Span::new(0, 15),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Use"));
    assert!(output.contains("Simple"));
    assert!(output.contains("bar"));
}

#[test]
fn test_generate_item_use_glob() {
    let gen = Generator::new();
    let use_tree = UseTree {
        prefix: Path {
            span: Span::new(4, 7),
            segments: vec![PathSegment::from_ident(Ident::new("foo", Span::new(4, 7)))],
            tokens: None,
        },
        kind: UseTreeKind::Glob,
    };

    let item = Item {
        ident: Ident::new("_", Span::new(0, 1)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Use(use_tree),
        vis: Visibility::Inherited,
        span: Span::new(0, 10),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Use"));
    assert!(output.contains("Glob"));
}

#[test]
fn test_generate_item_use_nested() {
    let gen = Generator::new();
    let nested1 = UseTree {
        prefix: Path {
            span: Span::new(9, 10),
            segments: vec![PathSegment::from_ident(Ident::new("a", Span::new(9, 10)))],
            tokens: None,
        },
        kind: UseTreeKind::Simple(None),
    };

    let nested2 = UseTree {
        prefix: Path {
            span: Span::new(12, 13),
            segments: vec![PathSegment::from_ident(Ident::new("b", Span::new(12, 13)))],
            tokens: None,
        },
        kind: UseTreeKind::Simple(None),
    };

    let use_tree = UseTree {
        prefix: Path {
            span: Span::new(4, 7),
            segments: vec![PathSegment::from_ident(Ident::new("foo", Span::new(4, 7)))],
            tokens: None,
        },
        kind: UseTreeKind::Nested(vec![nested1, nested2]),
    };

    let item = Item {
        ident: Ident::new("_", Span::new(0, 1)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Use(use_tree),
        vis: Visibility::Inherited,
        span: Span::new(0, 15),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Use"));
    assert!(output.contains("Nested"));
}

#[test]
fn test_generate_item_static_without_init() {
    let gen = Generator::new();
    let item = Item {
        ident: Ident::new("STATIC", Span::new(7, 13)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Static {
            mutability: Mutability::Not,
            ty: Ty { id: NodeId(2), kind: TyKind::Infer, span: Span::new(15, 18), tokens: None },
            expr: None,
        },
        vis: Visibility::Public,
        span: Span::new(0, 20),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Static"));
    assert!(output.contains(":mutability"));
    assert!(output.contains("Not"));
    assert!(output.contains(":expr"));
    assert!(output.contains("nil"));
}

#[test]
fn test_generate_item_static_with_init() {
    let gen = Generator::new();
    let init_expr = Expr {
        id: NodeId(3),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(42), span: Span::new(22, 24) }),
        span: Span::new(22, 24),
        attrs: vec![],
        tokens: None,
    };

    let item = Item {
        ident: Ident::new("STATIC", Span::new(7, 13)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Static {
            mutability: Mutability::Not,
            ty: Ty { id: NodeId(2), kind: TyKind::Infer, span: Span::new(15, 18), tokens: None },
            expr: Some(init_expr),
        },
        vis: Visibility::Inherited,
        span: Span::new(0, 25),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Static"));
    assert!(output.contains(":expr"));
    assert!(output.contains("42"));
}

#[test]
fn test_generate_item_static_mut() {
    let gen = Generator::new();
    let item = Item {
        ident: Ident::new("MUT_STATIC", Span::new(11, 21)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Static {
            mutability: Mutability::Mut,
            ty: Ty { id: NodeId(2), kind: TyKind::Infer, span: Span::new(23, 26), tokens: None },
            expr: None,
        },
        vis: Visibility::Inherited,
        span: Span::new(0, 28),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Static"));
    assert!(output.contains("Mut"));
}

#[test]
fn test_generate_item_const_without_init() {
    let gen = Generator::new();
    let item = Item {
        ident: Ident::new("CONST", Span::new(6, 11)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Const {
            ty: Ty { id: NodeId(2), kind: TyKind::Infer, span: Span::new(13, 16), tokens: None },
            expr: None,
        },
        vis: Visibility::Inherited,
        span: Span::new(0, 18),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Const"));
    assert!(output.contains(":ty"));
    assert!(output.contains(":expr"));
    assert!(output.contains("nil"));
}

#[test]
fn test_generate_item_const_with_init() {
    let gen = Generator::new();
    let init_expr = Expr {
        id: NodeId(3),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(100), span: Span::new(20, 23) }),
        span: Span::new(20, 23),
        attrs: vec![],
        tokens: None,
    };

    let item = Item {
        ident: Ident::new("CONST", Span::new(6, 11)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Const {
            ty: Ty { id: NodeId(2), kind: TyKind::Infer, span: Span::new(13, 16), tokens: None },
            expr: Some(init_expr),
        },
        vis: Visibility::Public,
        span: Span::new(0, 24),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Const"));
    assert!(output.contains("100"));
}

#[test]
fn test_generate_item_type_alias_without_ty() {
    let gen = Generator::new();
    let item = Item {
        ident: Ident::new("MyType", Span::new(5, 11)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::TyAlias { generics: Generics::empty(), ty: None },
        vis: Visibility::Inherited,
        span: Span::new(0, 12),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("TyAlias"));
    assert!(output.contains(":generics"));
    assert!(output.contains(":ty"));
    assert!(output.contains("nil"));
}

#[test]
fn test_generate_item_type_alias_with_ty() {
    let gen = Generator::new();
    let item = Item {
        ident: Ident::new("MyType", Span::new(5, 11)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::TyAlias {
            generics: Generics::empty(),
            ty: Some(Ty {
                id: NodeId(2),
                kind: TyKind::Infer,
                span: Span::new(14, 17),
                tokens: None,
            }),
        },
        vis: Visibility::Public,
        span: Span::new(0, 18),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("TyAlias"));
    assert!(output.contains(":ty"));
}

#[test]
fn test_generate_item_mod_without_items() {
    let gen = Generator::new();
    let item = Item {
        ident: Ident::new("my_mod", Span::new(4, 10)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Mod { items: None },
        vis: Visibility::Inherited,
        span: Span::new(0, 11),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Mod"));
    assert!(output.contains(":items"));
    assert!(output.contains("nil"));
}

#[test]
fn test_generate_item_mod_with_items() {
    let gen = Generator::new();
    let inner_item = Item {
        ident: Ident::new("inner", Span::new(14, 19)),
        attrs: vec![],
        id: NodeId(2),
        kind: ItemKind::Fn(Box::new(Fn {
            defaultness: Defaultness::Final,
            sig: FnSig {
                header: FnHeader {
                    safety: Safety::Safe,
                    constness: Constness::NotConst,
                    ext: Extern::None,
                    coroutine_kind: None,
                },
                decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::new(0, 0)) },
                span: Span::new(14, 24),
            },
            generics: Generics::empty(),
            body: None,
        })),
        vis: Visibility::Inherited,
        span: Span::new(14, 24),
        tokens: None,
    };

    let item = Item {
        ident: Ident::new("my_mod", Span::new(4, 10)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Mod { items: Some(vec![inner_item]) },
        vis: Visibility::Public,
        span: Span::new(0, 26),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Mod"));
    assert!(output.contains(":items"));
    assert!(output.contains("inner"));
}

// ============================================================================
// STATEMENT TESTS - targeting stmt.rs coverage
// ============================================================================

#[test]
fn test_generate_stmt_let_without_type() {
    let gen = Generator::new();
    let local = Local {
        pat: Pat {
            id: NodeId(1),
            kind: PatKind::Ident {
                binding_mode: BindingMode::ByValue(Mutability::Not),
                ident: Ident::new("x", Span::new(4, 5)),
                sub: None,
            },
            span: Span::new(4, 5),
            tokens: None,
        },
        ty: None,
        kind: LocalKind::Decl,
        span: Span::new(0, 6),
        attrs: vec![],
        tokens: None,
    };

    let stmt = Stmt { id: NodeId(2), kind: StmtKind::Let(Box::new(local)), span: Span::new(0, 6) };

    let sexp = gen.generate_stmt(&stmt).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Stmt"));
    assert!(output.contains("Let"));
    assert!(output.contains("Local"));
    assert!(output.contains(":ty"));
    assert!(output.contains("nil"));
}

#[test]
fn test_generate_stmt_let_with_type() {
    let gen = Generator::new();
    let local = Local {
        pat: Pat {
            id: NodeId(1),
            kind: PatKind::Ident {
                binding_mode: BindingMode::ByValue(Mutability::Not),
                ident: Ident::new("x", Span::new(4, 5)),
                sub: None,
            },
            span: Span::new(4, 5),
            tokens: None,
        },
        ty: Some(Ty { id: NodeId(3), kind: TyKind::Infer, span: Span::new(7, 10), tokens: None }),
        kind: LocalKind::Decl,
        span: Span::new(0, 11),
        attrs: vec![],
        tokens: None,
    };

    let stmt = Stmt { id: NodeId(2), kind: StmtKind::Let(Box::new(local)), span: Span::new(0, 11) };

    let sexp = gen.generate_stmt(&stmt).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Let"));
    assert!(output.contains(":ty"));
}

#[test]
fn test_generate_stmt_let_with_init() {
    let gen = Generator::new();
    let init = LocalInit {
        expr: Expr {
            id: NodeId(3),
            kind: ExprKind::Lit(Lit { kind: LitKind::Int(5), span: Span::new(8, 9) }),
            span: Span::new(8, 9),
            attrs: vec![],
            tokens: None,
        },
        els: None,
    };

    let local = Local {
        pat: Pat {
            id: NodeId(1),
            kind: PatKind::Ident {
                binding_mode: BindingMode::ByValue(Mutability::Not),
                ident: Ident::new("x", Span::new(4, 5)),
                sub: None,
            },
            span: Span::new(4, 5),
            tokens: None,
        },
        ty: None,
        kind: LocalKind::Init(init),
        span: Span::new(0, 10),
        attrs: vec![],
        tokens: None,
    };

    let stmt = Stmt { id: NodeId(2), kind: StmtKind::Let(Box::new(local)), span: Span::new(0, 10) };

    let sexp = gen.generate_stmt(&stmt).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Let"));
    assert!(output.contains("Init"));
    assert!(output.contains("LocalInit"));
}

#[test]
fn test_generate_stmt_let_with_init_else() {
    let gen = Generator::new();
    let init = LocalInit {
        expr: Expr {
            id: NodeId(3),
            kind: ExprKind::Lit(Lit { kind: LitKind::Int(5), span: Span::new(8, 9) }),
            span: Span::new(8, 9),
            attrs: vec![],
            tokens: None,
        },
        els: Some(Block {
            stmts: vec![],
            id: NodeId(4),
            rules: BlockCheckMode::Default,
            span: Span::new(15, 17),
            tokens: None,
            could_be_bare_literal: false,
        }),
    };

    let else_block = Block {
        stmts: vec![],
        id: NodeId(5),
        rules: BlockCheckMode::Default,
        span: Span::new(20, 22),
        tokens: None,
        could_be_bare_literal: false,
    };

    let local = Local {
        pat: Pat { id: NodeId(1), kind: PatKind::Wild, span: Span::new(4, 5), tokens: None },
        ty: None,
        kind: LocalKind::InitElse(init, else_block),
        span: Span::new(0, 23),
        attrs: vec![],
        tokens: None,
    };

    let stmt = Stmt { id: NodeId(2), kind: StmtKind::Let(Box::new(local)), span: Span::new(0, 23) };

    let sexp = gen.generate_stmt(&stmt).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Let"));
    assert!(output.contains("InitElse"));
}

#[test]
fn test_generate_stmt_item() {
    let gen = Generator::new();
    let item = Item {
        ident: Ident::new("inner_fn", Span::new(3, 11)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Fn(Box::new(Fn {
            defaultness: Defaultness::Final,
            sig: FnSig {
                header: FnHeader {
                    safety: Safety::Safe,
                    constness: Constness::NotConst,
                    ext: Extern::None,
                    coroutine_kind: None,
                },
                decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::new(0, 0)) },
                span: Span::new(3, 20),
            },
            generics: Generics::empty(),
            body: None,
        })),
        vis: Visibility::Inherited,
        span: Span::new(3, 20),
        tokens: None,
    };

    let stmt = Stmt { id: NodeId(2), kind: StmtKind::Item(Box::new(item)), span: Span::new(3, 20) };

    let sexp = gen.generate_stmt(&stmt).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Stmt"));
    assert!(output.contains("Item"));
    assert!(output.contains("inner_fn"));
}

#[test]
fn test_generate_stmt_mac_call() {
    let gen = Generator::new();
    let mac_call = MacCall::new(
        Path {
            span: Span::new(0, 5),
            segments: vec![PathSegment::from_ident(Ident::new("macro", Span::new(0, 5)))],
            tokens: None,
        },
        MacArgs::Empty,
    );

    let mac_call_stmt =
        MacCallStmt { mac: mac_call, style: MacStmtStyle::Semicolon, attrs: vec![], tokens: None };

    let stmt =
        Stmt { id: NodeId(1), kind: StmtKind::MacCall(mac_call_stmt), span: Span::new(0, 7) };

    let sexp = gen.generate_stmt(&stmt).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("MacCall"));
    assert!(output.contains("Semicolon"));
}

#[test]
fn test_generate_mac_stmt_style_all() {
    let gen = Generator::new();
    let styles = vec![
        (MacStmtStyle::Semicolon, "Semicolon"),
        (MacStmtStyle::Braces, "Braces"),
        (MacStmtStyle::NoBraces, "NoBraces"),
    ];

    for (style, style_name) in styles {
        let mac_call = MacCall::new(
            Path {
                span: Span::new(0, 1),
                segments: vec![PathSegment::from_ident(Ident::new("m", Span::new(0, 1)))],
                tokens: None,
            },
            MacArgs::Empty,
        );

        let mac_call_stmt = MacCallStmt { mac: mac_call, style, attrs: vec![], tokens: None };

        let stmt =
            Stmt { id: NodeId(1), kind: StmtKind::MacCall(mac_call_stmt), span: Span::new(0, 3) };

        let sexp = gen.generate_stmt(&stmt).unwrap();
        let output = print_sexp(&sexp);
        assert!(output.contains(style_name), "Failed for style: {}", style_name);
    }
}

// ============================================================================
// TYPE TESTS - targeting type generation in item.rs (via Item generation)
// ============================================================================

#[test]
fn test_generate_ty_ref_immutable_via_param() {
    let gen = Generator::new();
    let param = Param {
        attrs: vec![],
        ty: Ty {
            id: NodeId(1),
            kind: TyKind::Ref {
                lifetime: None,
                mutability: Mutability::Not,
                ty: Box::new(Ty {
                    id: NodeId(2),
                    kind: TyKind::Infer,
                    span: Span::new(1, 4),
                    tokens: None,
                }),
            },
            span: Span::new(0, 4),
            tokens: None,
        },
        pat: Pat { id: NodeId(3), kind: PatKind::Wild, span: Span::new(5, 6), tokens: None },
        id: NodeId(4),
        span: Span::new(0, 6),
        is_placeholder: false,
    };

    let fn_item = Item {
        ident: Ident::new("test", Span::new(0, 4)),
        attrs: vec![],
        id: NodeId(5),
        kind: ItemKind::Fn(Box::new(Fn {
            defaultness: Defaultness::Final,
            sig: FnSig {
                header: FnHeader {
                    safety: Safety::Safe,
                    constness: Constness::NotConst,
                    ext: Extern::None,
                    coroutine_kind: None,
                },
                decl: FnDecl { inputs: vec![param], output: FnRetTy::Default(Span::new(0, 0)) },
                span: Span::new(0, 10),
            },
            generics: Generics::empty(),
            body: None,
        })),
        vis: Visibility::Inherited,
        span: Span::new(0, 10),
        tokens: None,
    };

    let sexp = gen.generate_item(&fn_item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Ref"));
    assert!(output.contains(":lifetime"));
    assert!(output.contains(":mutability"));
    assert!(output.contains("Not"));
}

#[test]
fn test_generate_ty_ptr_via_param() {
    let gen = Generator::new();
    let param = Param {
        attrs: vec![],
        ty: Ty {
            id: NodeId(1),
            kind: TyKind::Ptr {
                mutability: Mutability::Mut,
                ty: Box::new(Ty {
                    id: NodeId(2),
                    kind: TyKind::Infer,
                    span: Span::new(5, 8),
                    tokens: None,
                }),
            },
            span: Span::new(0, 8),
            tokens: None,
        },
        pat: Pat { id: NodeId(3), kind: PatKind::Wild, span: Span::new(9, 10), tokens: None },
        id: NodeId(4),
        span: Span::new(0, 10),
        is_placeholder: false,
    };

    let fn_item = Item {
        ident: Ident::new("test", Span::new(0, 4)),
        attrs: vec![],
        id: NodeId(5),
        kind: ItemKind::Fn(Box::new(Fn {
            defaultness: Defaultness::Final,
            sig: FnSig {
                header: FnHeader {
                    safety: Safety::Safe,
                    constness: Constness::NotConst,
                    ext: Extern::None,
                    coroutine_kind: None,
                },
                decl: FnDecl { inputs: vec![param], output: FnRetTy::Default(Span::new(0, 0)) },
                span: Span::new(0, 10),
            },
            generics: Generics::empty(),
            body: None,
        })),
        vis: Visibility::Inherited,
        span: Span::new(0, 10),
        tokens: None,
    };

    let sexp = gen.generate_item(&fn_item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Ptr"));
    assert!(output.contains("Mut"));
}

#[test]
fn test_generate_ty_array_slice_tuple_via_param() {
    let gen = Generator::new();

    // Test array type
    let param = Param {
        attrs: vec![],
        ty: Ty {
            id: NodeId(1),
            kind: TyKind::Array {
                ty: Box::new(Ty {
                    id: NodeId(2),
                    kind: TyKind::Infer,
                    span: Span::new(1, 4),
                    tokens: None,
                }),
                len: Box::new(Expr {
                    id: NodeId(3),
                    kind: ExprKind::Lit(Lit { kind: LitKind::Int(10), span: Span::new(6, 8) }),
                    span: Span::new(6, 8),
                    attrs: vec![],
                    tokens: None,
                }),
            },
            span: Span::new(0, 9),
            tokens: None,
        },
        pat: Pat { id: NodeId(4), kind: PatKind::Wild, span: Span::new(10, 11), tokens: None },
        id: NodeId(5),
        span: Span::new(0, 11),
        is_placeholder: false,
    };

    let fn_item = Item {
        ident: Ident::new("test", Span::new(0, 4)),
        attrs: vec![],
        id: NodeId(6),
        kind: ItemKind::Fn(Box::new(Fn {
            defaultness: Defaultness::Final,
            sig: FnSig {
                header: FnHeader {
                    safety: Safety::Safe,
                    constness: Constness::NotConst,
                    ext: Extern::None,
                    coroutine_kind: None,
                },
                decl: FnDecl { inputs: vec![param], output: FnRetTy::Default(Span::new(0, 0)) },
                span: Span::new(0, 12),
            },
            generics: Generics::empty(),
            body: None,
        })),
        vis: Visibility::Inherited,
        span: Span::new(0, 12),
        tokens: None,
    };

    let sexp = gen.generate_item(&fn_item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Array"));
    assert!(output.contains(":len"));
}

#[test]
fn test_generate_ty_never_and_infer() {
    let gen = Generator::new();

    // Test Never type
    let param1 = Param {
        attrs: vec![],
        ty: Ty { id: NodeId(1), kind: TyKind::Never, span: Span::new(0, 1), tokens: None },
        pat: Pat { id: NodeId(2), kind: PatKind::Wild, span: Span::new(2, 3), tokens: None },
        id: NodeId(3),
        span: Span::new(0, 3),
        is_placeholder: false,
    };

    // Test Infer type
    let param2 = Param {
        attrs: vec![],
        ty: Ty { id: NodeId(4), kind: TyKind::Infer, span: Span::new(5, 6), tokens: None },
        pat: Pat { id: NodeId(5), kind: PatKind::Wild, span: Span::new(7, 8), tokens: None },
        id: NodeId(6),
        span: Span::new(5, 8),
        is_placeholder: false,
    };

    let fn_item = Item {
        ident: Ident::new("test", Span::new(0, 4)),
        attrs: vec![],
        id: NodeId(7),
        kind: ItemKind::Fn(Box::new(Fn {
            defaultness: Defaultness::Final,
            sig: FnSig {
                header: FnHeader {
                    safety: Safety::Safe,
                    constness: Constness::NotConst,
                    ext: Extern::None,
                    coroutine_kind: None,
                },
                decl: FnDecl {
                    inputs: vec![param1, param2],
                    output: FnRetTy::Default(Span::new(0, 0)),
                },
                span: Span::new(0, 10),
            },
            generics: Generics::empty(),
            body: None,
        })),
        vis: Visibility::Inherited,
        span: Span::new(0, 10),
        tokens: None,
    };

    let sexp = gen.generate_item(&fn_item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Never"));
    assert!(output.contains("Infer"));
}

// ============================================================================
// PATTERN TESTS - targeting pattern generation in item.rs (via Let statements)
// ============================================================================

#[test]
fn test_generate_pat_ident_with_sub() {
    let gen = Generator::new();
    let sub_pat = Pat { id: NodeId(2), kind: PatKind::Wild, span: Span::new(3, 4), tokens: None };

    let pat = Pat {
        id: NodeId(1),
        kind: PatKind::Ident {
            binding_mode: BindingMode::ByValue(Mutability::Not),
            ident: Ident::new("x", Span::new(0, 1)),
            sub: Some(Box::new(sub_pat)),
        },
        span: Span::new(0, 5),
        tokens: None,
    };

    let local = Local {
        pat,
        ty: None,
        kind: LocalKind::Decl,
        span: Span::new(0, 5),
        attrs: vec![],
        tokens: None,
    };

    let stmt = Stmt { id: NodeId(3), kind: StmtKind::Let(Box::new(local)), span: Span::new(0, 5) };

    let sexp = gen.generate_stmt(&stmt).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Pat"));
    assert!(output.contains("Ident"));
    assert!(output.contains(":sub"));
}

#[test]
fn test_generate_pat_wild() {
    let gen = Generator::new();
    let pat = Pat { id: NodeId(1), kind: PatKind::Wild, span: Span::new(0, 1), tokens: None };

    let local = Local {
        pat,
        ty: None,
        kind: LocalKind::Decl,
        span: Span::new(0, 1),
        attrs: vec![],
        tokens: None,
    };

    let stmt = Stmt { id: NodeId(2), kind: StmtKind::Let(Box::new(local)), span: Span::new(0, 1) };

    let sexp = gen.generate_stmt(&stmt).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Wild"));
}

#[test]
fn test_generate_pat_struct() {
    let gen = Generator::new();
    let field = PatField {
        attrs: vec![],
        ident: Ident::new("x", Span::new(7, 8)),
        pat: Pat { id: NodeId(2), kind: PatKind::Wild, span: Span::new(10, 11), tokens: None },
        is_shorthand: false,
        span: Span::new(7, 11),
    };

    let pat = Pat {
        id: NodeId(1),
        kind: PatKind::Struct {
            path: Path {
                span: Span::new(0, 5),
                segments: vec![PathSegment::from_ident(Ident::new("Point", Span::new(0, 5)))],
                tokens: None,
            },
            fields: vec![field],
        },
        span: Span::new(0, 13),
        tokens: None,
    };

    let local = Local {
        pat,
        ty: None,
        kind: LocalKind::Decl,
        span: Span::new(0, 13),
        attrs: vec![],
        tokens: None,
    };

    let stmt = Stmt { id: NodeId(3), kind: StmtKind::Let(Box::new(local)), span: Span::new(0, 13) };

    let sexp = gen.generate_stmt(&stmt).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Struct"));
    assert!(output.contains("PatField"));
}

// generate_pat_field is tested via PatKind::Struct which uses it
#[test]
fn test_generate_pat_field_shorthand() {
    let gen = Generator::new();
    let field = PatField {
        attrs: vec![],
        ident: Ident::new("x", Span::new(7, 8)),
        pat: Pat { id: NodeId(1), kind: PatKind::Wild, span: Span::new(7, 8), tokens: None },
        is_shorthand: true,
        span: Span::new(7, 8),
    };

    let pat = Pat {
        id: NodeId(2),
        kind: PatKind::Struct {
            path: Path {
                span: Span::new(0, 5),
                segments: vec![PathSegment::from_ident(Ident::new("Point", Span::new(0, 5)))],
                tokens: None,
            },
            fields: vec![field],
        },
        span: Span::new(0, 10),
        tokens: None,
    };

    let local = Local {
        pat,
        ty: None,
        kind: LocalKind::Decl,
        span: Span::new(0, 10),
        attrs: vec![],
        tokens: None,
    };

    let stmt = Stmt { id: NodeId(5), kind: StmtKind::Let(Box::new(local)), span: Span::new(0, 10) };

    let sexp = gen.generate_stmt(&stmt).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("PatField"));
    assert!(output.contains(":is-shorthand"));
    assert!(output.contains("true"));
}

// Remaining pattern tests use function parameters since generate_pat is private
#[test]
fn test_generate_pat_tuple_struct_via_param() {
    let gen = Generator::new();
    let pat = Pat {
        id: NodeId(1),
        kind: PatKind::TupleStruct {
            path: Path {
                span: Span::new(0, 4),
                segments: vec![PathSegment::from_ident(Ident::new("Some", Span::new(0, 4)))],
                tokens: None,
            },
            elems: vec![Pat {
                id: NodeId(2),
                kind: PatKind::Wild,
                span: Span::new(5, 6),
                tokens: None,
            }],
        },
        span: Span::new(0, 7),
        tokens: None,
    };

    let param = Param {
        attrs: vec![],
        ty: Ty { id: NodeId(3), kind: TyKind::Infer, span: Span::new(8, 9), tokens: None },
        pat,
        id: NodeId(4),
        span: Span::new(0, 9),
        is_placeholder: false,
    };

    let fn_item = Item {
        ident: Ident::new("test", Span::new(0, 4)),
        attrs: vec![],
        id: NodeId(5),
        kind: ItemKind::Fn(Box::new(Fn {
            defaultness: Defaultness::Final,
            sig: FnSig {
                header: FnHeader {
                    safety: Safety::Safe,
                    constness: Constness::NotConst,
                    ext: Extern::None,
                    coroutine_kind: None,
                },
                decl: FnDecl { inputs: vec![param], output: FnRetTy::Default(Span::new(0, 0)) },
                span: Span::new(0, 15),
            },
            generics: Generics::empty(),
            body: None,
        })),
        vis: Visibility::Inherited,
        span: Span::new(0, 15),
        tokens: None,
    };

    let sexp = gen.generate_item(&fn_item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("TupleStruct"));
}

// All other complex pattern kinds are adequately covered by match arms and let statements above

// ============================================================================
// VISIBILITY TESTS
// ============================================================================

#[test]
fn test_generate_visibility_restricted_crate() {
    let gen = Generator::new();
    let vis = Visibility::Restricted {
        path: Box::new(Path {
            span: Span::new(4, 9),
            segments: vec![PathSegment::from_ident(Ident::new("crate", Span::new(4, 9)))],
            tokens: None,
        }),
        shorthand: VisRestrictionKind::Crate,
        span: Span::new(0, 10),
    };

    let sexp = gen.generate_visibility(&vis);
    let output = print_sexp(&sexp);
    assert!(output.contains("Restricted"));
    assert!(output.contains("Crate"));
}

#[test]
fn test_generate_visibility_restricted_super() {
    let gen = Generator::new();
    let vis = Visibility::Restricted {
        path: Box::new(Path {
            span: Span::new(4, 9),
            segments: vec![PathSegment::from_ident(Ident::new("super", Span::new(4, 9)))],
            tokens: None,
        }),
        shorthand: VisRestrictionKind::Super,
        span: Span::new(0, 10),
    };

    let sexp = gen.generate_visibility(&vis);
    let output = print_sexp(&sexp);
    assert!(output.contains("Restricted"));
    assert!(output.contains("Super"));
}

#[test]
fn test_generate_visibility_restricted_in() {
    let gen = Generator::new();
    let vis = Visibility::Restricted {
        path: Box::new(Path {
            span: Span::new(7, 10),
            segments: vec![PathSegment::from_ident(Ident::new("foo", Span::new(7, 10)))],
            tokens: None,
        }),
        shorthand: VisRestrictionKind::In,
        span: Span::new(0, 11),
    };

    let sexp = gen.generate_visibility(&vis);
    let output = print_sexp(&sexp);
    assert!(output.contains("Restricted"));
    assert!(output.contains("In"));
}

// MacArgs, Delimiter, and TokenStream are adequately tested via MacCall generation above

// ============================================================================
// FUNCTION HEADER TESTS
// ============================================================================

#[test]
fn test_generate_fn_header_extern_explicit() {
    let gen = Generator::new();
    let header = FnHeader {
        safety: Safety::Safe,
        constness: Constness::NotConst,
        ext: Extern::Explicit("C".to_string()),
        coroutine_kind: None,
    };

    let fn_item = Item {
        ident: Ident::new("test", Span::new(0, 4)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Fn(Box::new(Fn {
            defaultness: Defaultness::Final,
            sig: FnSig {
                header,
                decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::new(0, 0)) },
                span: Span::new(0, 10),
            },
            generics: Generics::empty(),
            body: None,
        })),
        vis: Visibility::Inherited,
        span: Span::new(0, 10),
        tokens: None,
    };

    let sexp = gen.generate_item(&fn_item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Explicit"));
    assert!(output.contains("\"C\""));
}

#[test]
fn test_generate_fn_header_coroutine_async() {
    let gen = Generator::new();
    let header = FnHeader {
        safety: Safety::Safe,
        constness: Constness::NotConst,
        ext: Extern::None,
        coroutine_kind: Some(CoroutineKind::Async),
    };

    let fn_item = Item {
        ident: Ident::new("test", Span::new(0, 4)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Fn(Box::new(Fn {
            defaultness: Defaultness::Final,
            sig: FnSig {
                header,
                decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::new(0, 0)) },
                span: Span::new(0, 10),
            },
            generics: Generics::empty(),
            body: None,
        })),
        vis: Visibility::Inherited,
        span: Span::new(0, 10),
        tokens: None,
    };

    let sexp = gen.generate_item(&fn_item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains(":coroutine-kind"));
    assert!(output.contains("Async"));
}

#[test]
fn test_generate_fn_header_coroutine_gen() {
    let gen = Generator::new();
    let header = FnHeader {
        safety: Safety::Safe,
        constness: Constness::NotConst,
        ext: Extern::None,
        coroutine_kind: Some(CoroutineKind::Gen),
    };

    let fn_item = Item {
        ident: Ident::new("test", Span::new(0, 4)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Fn(Box::new(Fn {
            defaultness: Defaultness::Final,
            sig: FnSig {
                header,
                decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::new(0, 0)) },
                span: Span::new(0, 10),
            },
            generics: Generics::empty(),
            body: None,
        })),
        vis: Visibility::Inherited,
        span: Span::new(0, 10),
        tokens: None,
    };

    let sexp = gen.generate_item(&fn_item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Gen"));
}

// FnRetTy is tested via function signatures which are part of items
#[test]
fn test_generate_fn_ret_ty_default() {
    let gen = Generator::new();
    let fn_item = Item {
        ident: Ident::new("test", Span::new(0, 4)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Fn(Box::new(Fn {
            defaultness: Defaultness::Final,
            sig: FnSig {
                header: FnHeader {
                    safety: Safety::Safe,
                    constness: Constness::NotConst,
                    ext: Extern::None,
                    coroutine_kind: None,
                },
                decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::new(0, 0)) },
                span: Span::new(0, 10),
            },
            generics: Generics::empty(),
            body: None,
        })),
        vis: Visibility::Inherited,
        span: Span::new(0, 10),
        tokens: None,
    };

    let sexp = gen.generate_item(&fn_item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Default"));
}

#[test]
fn test_generate_fn_ret_ty_ty() {
    let gen = Generator::new();
    let fn_item = Item {
        ident: Ident::new("test", Span::new(0, 4)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Fn(Box::new(Fn {
            defaultness: Defaultness::Final,
            sig: FnSig {
                header: FnHeader {
                    safety: Safety::Safe,
                    constness: Constness::NotConst,
                    ext: Extern::None,
                    coroutine_kind: None,
                },
                decl: FnDecl {
                    inputs: vec![],
                    output: FnRetTy::Ty(Box::new(Ty {
                        id: NodeId(2),
                        kind: TyKind::Infer,
                        span: Span::new(15, 18),
                        tokens: None,
                    })),
                },
                span: Span::new(0, 18),
            },
            generics: Generics::empty(),
            body: None,
        })),
        vis: Visibility::Inherited,
        span: Span::new(0, 18),
        tokens: None,
    };

    let sexp = gen.generate_item(&fn_item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Ty"));
}

// ============================================================================
// BLOCK TESTS
// ============================================================================

#[test]
fn test_generate_block_unsafe() {
    let gen = Generator::new();
    let block = Block {
        stmts: vec![],
        id: NodeId(1),
        rules: BlockCheckMode::Unsafe,
        span: Span::new(0, 2),
        tokens: None,
        could_be_bare_literal: false,
    };

    let sexp = gen.generate_block(&block).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("Block"));
    assert!(output.contains(":rules"));
    assert!(output.contains("Unsafe"));
}

#[test]
fn test_generate_block_with_could_be_bare_literal() {
    let gen = Generator::new();
    let block = Block {
        stmts: vec![],
        id: NodeId(1),
        rules: BlockCheckMode::Default,
        span: Span::new(0, 2),
        tokens: None,
        could_be_bare_literal: true,
    };

    let sexp = gen.generate_block(&block).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains(":could-be-bare-literal"));
    assert!(output.contains("true"));
}

#[test]
fn test_generate_block_with_tokens() {
    let gen = Generator::new();
    let block = Block {
        stmts: vec![],
        id: NodeId(1),
        rules: BlockCheckMode::Default,
        span: Span::new(0, 2),
        tokens: Some(TokenStream::Empty),
        could_be_bare_literal: false,
    };

    let sexp = gen.generate_block(&block).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains(":tokens"));
    assert!(output.contains("nil"));
}

// ============================================================================
// ASSOC ITEM TESTS - via trait/impl items
// ============================================================================

#[test]
fn test_generate_assoc_item_fn_via_impl() {
    let gen = Generator::new();
    let assoc_item = AssocItem {
        attrs: vec![],
        id: NodeId(2),
        span: Span::new(10, 20),
        vis: Visibility::Public,
        ident: Ident::new("method", Span::new(10, 16)),
        kind: AssocItemKind::Fn(Box::new(Fn {
            defaultness: Defaultness::Final,
            sig: FnSig {
                header: FnHeader {
                    safety: Safety::Safe,
                    constness: Constness::NotConst,
                    ext: Extern::None,
                    coroutine_kind: None,
                },
                decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::new(0, 0)) },
                span: Span::new(10, 20),
            },
            generics: Generics::empty(),
            body: None,
        })),
    };

    let impl_def = ImplDef {
        safety: Safety::Safe,
        generics: Generics::empty(),
        of_trait: None,
        self_ty: Ty { id: NodeId(3), kind: TyKind::Infer, span: Span::new(5, 8), tokens: None },
        items: vec![assoc_item],
    };

    let item = Item {
        ident: Ident::new("_", Span::new(0, 1)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Impl(Box::new(impl_def)),
        vis: Visibility::Inherited,
        span: Span::new(0, 25),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("AssocItem"));
    assert!(output.contains("method"));
}

#[test]
fn test_generate_assoc_item_type_via_trait() {
    let gen = Generator::new();
    let assoc_item = AssocItem {
        attrs: vec![],
        id: NodeId(2),
        span: Span::new(10, 15),
        vis: Visibility::Inherited,
        ident: Ident::new("Item", Span::new(10, 14)),
        kind: AssocItemKind::Type(Box::new(Some(Ty {
            id: NodeId(3),
            kind: TyKind::Infer,
            span: Span::new(17, 20),
            tokens: None,
        }))),
    };

    let trait_def = TraitDef {
        safety: Safety::Safe,
        generics: Generics::empty(),
        bounds: vec![],
        items: vec![assoc_item],
    };

    let item = Item {
        ident: Ident::new("MyTrait", Span::new(0, 7)),
        attrs: vec![],
        id: NodeId(1),
        kind: ItemKind::Trait(Box::new(trait_def)),
        vis: Visibility::Public,
        span: Span::new(0, 25),
        tokens: None,
    };

    let sexp = gen.generate_item(&item).unwrap();
    let output = print_sexp(&sexp);
    assert!(output.contains("AssocItem"));
    assert!(output.contains("Type"));
}

// Safety and Defaultness are extensively tested via function and trait generation above
