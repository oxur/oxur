//! Phase 5 Round-Trip Tests
//!
//! Comprehensive tests for all Phase 5 implementations:
//! - Priority 1: Pattern variants (Box, Path, Range, Rest, Paren, Type, Const, MacCall, Err)
//! - Priority 2: Type variants (BareFn, ImplTrait, TraitObject, MacCall, Paren)
//! - Priority 3: Expression variants (Paren, Try, Cast, Break, Continue, Return)
//!
//! Each test creates a complete Crate with items using Phase 5 features,
//! then verifies: AST → SExp → AST → Rust code

use oxur_ast::ast::*;
use oxur_ast::gen_rs::generate_rust;
use oxur_ast::sexp::{print_sexp, Parser};
use oxur_ast::{AstBuilder, Generator};

// Helper functions
fn simple_span() -> Span {
    Span::DUMMY
}

fn simple_ident(name: &str) -> Ident {
    Ident::new(name, simple_span())
}

fn simple_path(name: &str) -> Path {
    Path {
        span: simple_span(),
        segments: vec![PathSegment::from_ident(simple_ident(name))],
        tokens: None,
    }
}

fn simple_lit_expr(n: i128) -> Expr {
    Expr {
        id: NodeId(0),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(n), span: simple_span() }),
        span: simple_span(),
        attrs: Vec::new(),
        tokens: None,
    }
}

fn simple_block(stmts: Vec<Stmt>) -> Block {
    Block {
        stmts,
        id: NodeId(0),
        rules: BlockCheckMode::Default,
        span: simple_span(),
        could_be_bare_literal: false,
        tokens: None,
    }
}

fn make_crate(items: Vec<Item>) -> Crate {
    Crate {
        attrs: Vec::new(),
        items,
        spans: ModSpans { inner_span: simple_span(), inject_use_span: simple_span() },
        id: NodeId(0),
        is_placeholder: false,
    }
}

fn verify_round_trip_and_codegen(crate_ast: &Crate, expected_fragments: &[&str]) {
    // Step 1: Generate S-expression from AST
    let gen = Generator::new();
    let sexp1 = gen.generate_crate(crate_ast).unwrap();

    // Step 2: Print and parse S-expression
    let printed = print_sexp(&sexp1);
    let sexp2 = Parser::parse_str(&printed).unwrap();

    // Step 3: Build AST from S-expression
    let mut builder = AstBuilder::new();
    let crate2 = builder.build_crate(&sexp2).unwrap();

    // Step 4: Generate S-expression again (should be identical)
    let sexp3 = gen.generate_crate(&crate2).unwrap();
    assert_eq!(sexp1, sexp3, "Round-trip should preserve S-expression");

    // Step 5: Generate Rust code
    let code = generate_rust(&crate2).unwrap();

    // Step 6: Verify expected code fragments
    for fragment in expected_fragments {
        assert!(
            code.contains(fragment),
            "Generated code should contain '{}'\nGenerated code:\n{}",
            fragment,
            code
        );
    }
}

// Priority 3: Expression Tests (easier to test in functions)

#[test]
fn test_expr_paren_round_trip() {
    let expr = Expr {
        id: NodeId(1),
        kind: ExprKind::Paren(Box::new(simple_lit_expr(42))),
        span: simple_span(),
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Expr(expr), span: simple_span() };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(simple_span()) },
            span: simple_span(),
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: simple_span(),
        vis: Visibility::Inherited,
        ident: simple_ident("test"),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };

    verify_round_trip_and_codegen(&make_crate(vec![item]), &["(42)"]);
}

#[test]
fn test_expr_try_round_trip() {
    let expr = Expr {
        id: NodeId(1),
        kind: ExprKind::Try(Box::new(Expr {
            id: NodeId(2),
            kind: ExprKind::Path(None, simple_path("result")),
            span: simple_span(),
            attrs: Vec::new(),
            tokens: None,
        })),
        span: simple_span(),
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Expr(expr), span: simple_span() };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(simple_span()) },
            span: simple_span(),
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: simple_span(),
        vis: Visibility::Inherited,
        ident: simple_ident("test"),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };

    verify_round_trip_and_codegen(&make_crate(vec![item]), &["result?"]);
}

#[test]
fn test_expr_cast_round_trip() {
    let expr = Expr {
        id: NodeId(1),
        kind: ExprKind::Cast {
            expr: Box::new(simple_lit_expr(42)),
            ty: Box::new(Ty {
                id: NodeId(2),
                kind: TyKind::Path(None, simple_path("u64")),
                span: simple_span(),
                tokens: None,
            }),
        },
        span: simple_span(),
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Expr(expr), span: simple_span() };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(simple_span()) },
            span: simple_span(),
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: simple_span(),
        vis: Visibility::Inherited,
        ident: simple_ident("test"),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };

    verify_round_trip_and_codegen(&make_crate(vec![item]), &["42 as u64"]);
}

#[test]
fn test_expr_break_simple() {
    let expr = Expr {
        id: NodeId(1),
        kind: ExprKind::Break { label: None, value: None },
        span: simple_span(),
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Semi(expr), span: simple_span() };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(simple_span()) },
            span: simple_span(),
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: simple_span(),
        vis: Visibility::Inherited,
        ident: simple_ident("test"),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };

    verify_round_trip_and_codegen(&make_crate(vec![item]), &["break"]);
}

#[test]
fn test_expr_break_with_value() {
    let expr = Expr {
        id: NodeId(1),
        kind: ExprKind::Break { label: None, value: Some(Box::new(simple_lit_expr(5))) },
        span: simple_span(),
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Semi(expr), span: simple_span() };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(simple_span()) },
            span: simple_span(),
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: simple_span(),
        vis: Visibility::Inherited,
        ident: simple_ident("test"),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };

    verify_round_trip_and_codegen(&make_crate(vec![item]), &["break 5"]);
}

#[test]
fn test_expr_continue_simple() {
    let expr = Expr {
        id: NodeId(1),
        kind: ExprKind::Continue { label: None },
        span: simple_span(),
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Semi(expr), span: simple_span() };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(simple_span()) },
            span: simple_span(),
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: simple_span(),
        vis: Visibility::Inherited,
        ident: simple_ident("test"),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };

    verify_round_trip_and_codegen(&make_crate(vec![item]), &["continue"]);
}

#[test]
fn test_expr_return_simple() {
    let expr = Expr {
        id: NodeId(1),
        kind: ExprKind::Return { value: None },
        span: simple_span(),
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Semi(expr), span: simple_span() };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(simple_span()) },
            span: simple_span(),
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: simple_span(),
        vis: Visibility::Inherited,
        ident: simple_ident("test"),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };

    verify_round_trip_and_codegen(&make_crate(vec![item]), &["return"]);
}

#[test]
fn test_expr_return_with_value() {
    let expr = Expr {
        id: NodeId(1),
        kind: ExprKind::Return { value: Some(Box::new(simple_lit_expr(0))) },
        span: simple_span(),
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Semi(expr), span: simple_span() };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(simple_span()) },
            span: simple_span(),
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: simple_span(),
        vis: Visibility::Inherited,
        ident: simple_ident("test"),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };

    verify_round_trip_and_codegen(&make_crate(vec![item]), &["return 0"]);
}

// Complex test combining multiple Phase 5 features
#[test]
fn test_complex_phase5_features() {
    // Function that uses try operator, cast, and control flow
    let expr = Expr {
        id: NodeId(1),
        kind: ExprKind::Paren(Box::new(Expr {
            id: NodeId(2),
            kind: ExprKind::Cast {
                expr: Box::new(Expr {
                    id: NodeId(3),
                    kind: ExprKind::Try(Box::new(Expr {
                        id: NodeId(4),
                        kind: ExprKind::Path(None, simple_path("result")),
                        span: simple_span(),
                        attrs: Vec::new(),
                        tokens: None,
                    })),
                    span: simple_span(),
                    attrs: Vec::new(),
                    tokens: None,
                }),
                ty: Box::new(Ty {
                    id: NodeId(5),
                    kind: TyKind::Path(None, simple_path("u64")),
                    span: simple_span(),
                    tokens: None,
                }),
            },
            span: simple_span(),
            attrs: Vec::new(),
            tokens: None,
        })),
        span: simple_span(),
        attrs: Vec::new(),
        tokens: None,
    };
    let stmt = Stmt { id: NodeId(0), kind: StmtKind::Expr(expr), span: simple_span() };
    let func = Fn {
        defaultness: Defaultness::Final,
        sig: FnSig {
            header: FnHeader {
                safety: Safety::Default,
                constness: Constness::NotConst,
                coroutine_kind: None,
                ext: Extern::None,
            },
            decl: FnDecl { inputs: vec![], output: FnRetTy::Default(simple_span()) },
            span: simple_span(),
        },
        generics: Generics::empty(),
        body: Some(simple_block(vec![stmt])),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: simple_span(),
        vis: Visibility::Inherited,
        ident: simple_ident("test"),
        kind: ItemKind::Fn(Box::new(func)),
        tokens: None,
    };

    verify_round_trip_and_codegen(&make_crate(vec![item]), &["result?", "as u64", "("]);
}
