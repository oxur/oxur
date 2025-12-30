/// Tests to improve generator coverage using existing test-data fixtures
use oxur_ast::sexp::{print_sexp, Parser};
use oxur_ast::{AstBuilder, Generator};
use std::fs;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data").join("fixtures").join(name)
}

/// Test generator/stmt.rs coverage using statement fixtures
#[test]
fn test_generate_stmt_empty() {
    let path = fixture_path("stmt/empty.sexp");
    let content = fs::read_to_string(&path).unwrap();
    let sexp = Parser::parse_str(&content).unwrap();

    let mut builder = AstBuilder::new();
    let stmt = builder.build_stmt(&sexp).unwrap();

    let gen = Generator::new();
    let result = gen.generate_stmt(&stmt).unwrap();

    let output = print_sexp(&result);
    assert!(output.contains("Stmt"));
    assert!(output.contains("Empty"));
}

#[test]
fn test_generate_stmt_expr_with_macro() {
    let path = fixture_path("stmt/expr-with-macro-call.sexp");
    let content = fs::read_to_string(&path).unwrap();
    let sexp = Parser::parse_str(&content).unwrap();

    let mut builder = AstBuilder::new();
    let stmt = builder.build_stmt(&sexp).unwrap();

    let gen = Generator::new();
    let result = gen.generate_stmt(&stmt).unwrap();

    let output = print_sexp(&result);
    assert!(output.contains("Stmt"));
    assert!(output.contains("MacCall"));
}

#[test]
fn test_generate_stmt_semi_with_macro() {
    let path = fixture_path("stmt/semi-with-macro-call.sexp");
    let content = fs::read_to_string(&path).unwrap();
    let sexp = Parser::parse_str(&content).unwrap();

    let mut builder = AstBuilder::new();
    let stmt = builder.build_stmt(&sexp).unwrap();

    let gen = Generator::new();
    let result = gen.generate_stmt(&stmt).unwrap();

    let output = print_sexp(&result);
    assert!(output.contains("Stmt"));
    assert!(output.contains("Semi"));
    assert!(output.contains("MacCall"));
}

/// Test generator/expr.rs coverage using expression fixtures
#[test]
fn test_generate_expr_macro_call() {
    let path = fixture_path("expr/macro-call.sexp");
    let content = fs::read_to_string(&path).unwrap();
    let sexp = Parser::parse_str(&content).unwrap();

    let mut builder = AstBuilder::new();
    let expr = builder.build_expr(&sexp).unwrap();

    let gen = Generator::new();
    let result = gen.generate_expr(&expr).unwrap();

    let output = print_sexp(&result);
    assert!(output.contains("Expr"));
    assert!(output.contains("MacCall"));
}

/// Test generator with blocks
#[test]
fn test_generate_block_empty() {
    let path = fixture_path("block/empty.sexp");
    let content = fs::read_to_string(&path).unwrap();
    let sexp = Parser::parse_str(&content).unwrap();

    let mut builder = AstBuilder::new();
    let block = builder.build_block(&sexp).unwrap();

    let gen = Generator::new();
    let result = gen.generate_block(&block).unwrap();

    let output = print_sexp(&result);
    assert!(output.contains("Block"));
}

/// Test generator with full crates
#[test]
fn test_generate_crate_empty() {
    let path = fixture_path("crate/empty.sexp");
    let content = fs::read_to_string(&path).unwrap();
    let sexp = Parser::parse_str(&content).unwrap();

    let mut builder = AstBuilder::new();
    let crate_node = builder.build_crate(&sexp).unwrap();

    let gen = Generator::new();
    let result = gen.generate_crate(&crate_node).unwrap();

    let output = print_sexp(&result);
    assert!(output.contains("Crate"));
}
