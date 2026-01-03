//! Comprehensive tests for Stage 2-7: Control Flow, Operations, and Advanced Expressions

use oxur_ast::ast::*;
use oxur_ast::gen_rs::RustCodegen;
use oxur_ast::gen_sexp::Generator;

fn simple_int(n: i128) -> Expr {
    Expr {
        id: NodeId(0),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(n), span: Span::DUMMY }),
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    }
}

fn simple_ident_expr(name: &str) -> Expr {
    Expr {
        id: NodeId(0),
        kind: ExprKind::Path(
            None,
            Path {
                span: Span::DUMMY,
                segments: vec![PathSegment {
                    ident: Ident::new(name, Span::DUMMY),
                    id: NodeId(0),
                    args: None,
                }],
                tokens: None,
            },
        ),
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    }
}

fn simple_block(stmts: Vec<Stmt>) -> Block {
    Block {
        stmts,
        id: NodeId(0),
        rules: BlockCheckMode::Default,
        span: Span::DUMMY,
        could_be_bare_literal: false,
        tokens: None,
    }
}

fn make_crate(items: Vec<Item>) -> Crate {
    Crate {
        attrs: Vec::new(),
        items,
        spans: ModSpans { inner_span: Span::DUMMY, inject_use_span: Span::DUMMY },
        id: NodeId(0),
        is_placeholder: false,
    }
}

// Stage 2: Control Flow Tests

#[test]
fn test_codegen_if_simple() {
    let cond = Box::new(simple_ident_expr("x"));
    let then_branch = simple_block(vec![]);
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::If { cond, then_branch, else_branch: None },
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Expr(expr), span: Span::DUMMY };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::DUMMY) },
            span: Span::DUMMY,
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("test", Span::DUMMY),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("if x"));
    assert!(output.contains("{"));
}

#[test]
fn test_codegen_if_else() {
    let cond = Box::new(simple_ident_expr("x"));
    let then_branch = simple_block(vec![]);
    // For else branch, just use another if-like structure or simple expression
    let else_body = simple_block(vec![]);
    let else_branch = Some(Box::new(Expr {
        id: NodeId(0),
        kind: ExprKind::If {
            cond: Box::new(simple_ident_expr("false")),
            then_branch: else_body,
            else_branch: None,
        },
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    }));
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::If { cond, then_branch, else_branch },
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Expr(expr), span: Span::DUMMY };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::DUMMY) },
            span: Span::DUMMY,
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("test", Span::DUMMY),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("if x"));
    assert!(output.contains("else"));
}

#[test]
fn test_codegen_loop_basic() {
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::Loop { label: None, body: simple_block(vec![]) },
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Expr(expr), span: Span::DUMMY };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::DUMMY) },
            span: Span::DUMMY,
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("test", Span::DUMMY),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("loop {"));
}

#[test]
fn test_codegen_while_basic() {
    let cond = Box::new(simple_ident_expr("condition"));
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::While { label: None, cond, body: simple_block(vec![]) },
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Expr(expr), span: Span::DUMMY };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::DUMMY) },
            span: Span::DUMMY,
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("test", Span::DUMMY),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("while condition"));
}

// Stage 3: Binary/Unary Operations Tests

#[test]
fn test_codegen_binary_add() {
    let left = Box::new(simple_int(1));
    let right = Box::new(simple_int(2));
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::Binary { left, op: BinOp::Add, right },
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Semi(expr), span: Span::DUMMY };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::DUMMY) },
            span: Span::DUMMY,
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("test", Span::DUMMY),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("1 + 2"));
}

#[test]
fn test_generator_binary_ops() {
    let left = Box::new(simple_int(5));
    let right = Box::new(simple_int(3));
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::Binary { left, op: BinOp::Sub, right },
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let generator = Generator::new();
    let result = generator.generate_expr(&expr);
    assert!(result.is_ok(), "Should generate binary expression");
}

#[test]
fn test_codegen_unary_neg() {
    let expr_inner = Box::new(simple_int(5));
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::Unary { op: UnOp::Neg, expr: expr_inner },
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Semi(expr), span: Span::DUMMY };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::DUMMY) },
            span: Span::DUMMY,
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("test", Span::DUMMY),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("-5"));
}

#[test]
fn test_generator_unary_ops() {
    let expr_inner = Box::new(simple_ident_expr("x"));
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::Unary { op: UnOp::Not, expr: expr_inner },
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let generator = Generator::new();
    let result = generator.generate_expr(&expr);
    assert!(result.is_ok(), "Should generate unary expression");
}

#[test]
fn test_codegen_call_with_args() {
    let func = Box::new(simple_ident_expr("foo"));
    let args = vec![simple_int(1), simple_int(2)];
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::Call { func, args },
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Semi(expr), span: Span::DUMMY };
    let func_item = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::DUMMY) },
            span: Span::DUMMY,
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("test", Span::DUMMY),
        kind: ItemKind::Fn(Box::new(func_item)),
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("foo(1, 2)"));
}

#[test]
fn test_generator_call() {
    let func = Box::new(simple_ident_expr("bar"));
    let args = vec![simple_int(42)];
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::Call { func, args },
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let generator = Generator::new();
    let result = generator.generate_expr(&expr);
    assert!(result.is_ok(), "Should generate call expression");
}

#[test]
fn test_codegen_method_call() {
    let receiver = Box::new(simple_ident_expr("obj"));
    let method = Ident::new("method", Span::DUMMY);
    let args = vec![simple_int(10)];
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::MethodCall { receiver, method, args },
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Semi(expr), span: Span::DUMMY };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::DUMMY) },
            span: Span::DUMMY,
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("test", Span::DUMMY),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("obj.method(10)"));
}

// Stage 7: Advanced Expressions Tests

#[test]
fn test_codegen_array_literal() {
    let elements = vec![simple_int(1), simple_int(2), simple_int(3)];
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::Array(elements),
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Semi(expr), span: Span::DUMMY };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::DUMMY) },
            span: Span::DUMMY,
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("test", Span::DUMMY),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("[1, 2, 3]"));
}

#[test]
fn test_codegen_tuple_literal() {
    let elements = vec![simple_int(1), simple_int(2)];
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::Tuple(elements),
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Semi(expr), span: Span::DUMMY };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::DUMMY) },
            span: Span::DUMMY,
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("test", Span::DUMMY),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("(1, 2)"));
}

#[test]
fn test_codegen_field_access() {
    let expr_inner = Box::new(simple_ident_expr("obj"));
    let field = Ident::new("field", Span::DUMMY);
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::Field { expr: expr_inner, field },
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Semi(expr), span: Span::DUMMY };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::DUMMY) },
            span: Span::DUMMY,
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("test", Span::DUMMY),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("obj.field"));
}

#[test]
fn test_codegen_index() {
    let expr_inner = Box::new(simple_ident_expr("arr"));
    let index = Box::new(simple_int(0));
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::Index { expr: expr_inner, index },
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Semi(expr), span: Span::DUMMY };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::DUMMY) },
            span: Span::DUMMY,
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("test", Span::DUMMY),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("arr[0]"));
}

#[test]
fn test_codegen_assignment() {
    let left = Box::new(simple_ident_expr("x"));
    let right = Box::new(simple_int(42));
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::Assign { left, right },
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Semi(expr), span: Span::DUMMY };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::DUMMY) },
            span: Span::DUMMY,
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("test", Span::DUMMY),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("x = 42"));
}

#[test]
fn test_generator_array() {
    let elements = vec![simple_int(1), simple_int(2)];
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::Array(elements),
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let generator = Generator::new();
    let result = generator.generate_expr(&expr);
    assert!(result.is_ok(), "Should generate array expression");
}

#[test]
fn test_generator_tuple() {
    let elements = vec![simple_int(1), simple_int(2)];
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::Tuple(elements),
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let generator = Generator::new();
    let result = generator.generate_expr(&expr);
    assert!(result.is_ok(), "Should generate tuple expression");
}

#[test]
fn test_generator_field_access() {
    let expr_inner = Box::new(simple_ident_expr("obj"));
    let field = Ident::new("field", Span::DUMMY);
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::Field { expr: expr_inner, field },
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let generator = Generator::new();
    let result = generator.generate_expr(&expr);
    assert!(result.is_ok(), "Should generate field access");
}

#[test]
fn test_generator_index() {
    let expr_inner = Box::new(simple_ident_expr("arr"));
    let index = Box::new(simple_int(0));
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::Index { expr: expr_inner, index },
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let generator = Generator::new();
    let result = generator.generate_expr(&expr);
    assert!(result.is_ok(), "Should generate index expression");
}

#[test]
fn test_generator_assignment() {
    let left = Box::new(simple_ident_expr("x"));
    let right = Box::new(simple_int(10));
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::Assign { left, right },
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let generator = Generator::new();
    let result = generator.generate_expr(&expr);
    assert!(result.is_ok(), "Should generate assignment");
}
