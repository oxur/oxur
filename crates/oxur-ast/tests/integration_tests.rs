use oxur_ast::ast;
use oxur_ast::builder::AstBuilder;
use oxur_ast::integration::parse_rust_file;
use oxur_ast::sexp::{print_sexp, Parser};
use oxur_ast::Generator;

#[test]
fn test_parse_and_build_simple_crate() {
    let input = r#"(Crate
      :attrs ()
      :items ()
      :spans (ModSpans :inner-span (Span :lo 0 :hi 50))
      :id 0)"#;

    // Parse S-expression
    let sexp = Parser::parse_str(input).unwrap();

    // Build AST
    let mut builder = AstBuilder::new();
    let crate_ast = builder.build_crate(&sexp).unwrap();

    // Verify
    assert_eq!(crate_ast.items.len(), 0);
    assert_eq!(crate_ast.id.0, 0);
}

#[test]
fn test_round_trip_crate() {
    let input =
        r#"(Crate :attrs () :items () :spans (ModSpans :inner-span (Span :lo 0 :hi 10)) :id 0)"#;

    // Parse
    let sexp = Parser::parse_str(input).unwrap();

    // Print and re-parse
    let printed = print_sexp(&sexp);
    let reparsed = Parser::parse_str(&printed).unwrap();

    // Both should build to AST
    let mut builder1 = AstBuilder::new();
    let mut builder2 = AstBuilder::new();

    let ast1 = builder1.build_crate(&sexp).unwrap();
    let ast2 = builder2.build_crate(&reparsed).unwrap();

    assert_eq!(ast1.items.len(), ast2.items.len());
}

#[test]
fn test_ast_type_constructors() {
    use oxur_ast::ast::*;

    // Test Span
    let span = Span::new(0, 10);
    assert_eq!(span.lo, 0);
    assert_eq!(span.hi, 10);

    // Test NodeId
    let id = NodeId(42);
    assert_eq!(id.0, 42);

    // Test ModSpans
    let mod_spans = ModSpans::new(span);
    assert_eq!(mod_spans.inner_span, span);

    // Test Generics::empty
    let generics = Generics::empty();
    assert_eq!(generics.params.len(), 0);

    // Test WhereClause::empty
    let where_clause = WhereClause::empty();
    assert!(!where_clause.has_where_token);
}

#[test]
fn test_ast_path_construction() {
    use oxur_ast::ast::*;

    let span = Span::new(0, 10);
    let ident = Ident::new("test", span);
    let path = Path::from_ident(ident.clone());

    assert_eq!(path.segments.len(), 1);
    assert_eq!(path.segments[0].ident.name, "test");
}

#[test]
fn test_ast_block_construction() {
    use oxur_ast::ast::*;

    let span = Span::new(0, 10);
    let id = NodeId(1);
    let block = Block::new(vec![], id, span);

    assert_eq!(block.stmts.len(), 0);
    assert_eq!(block.id, id);
}

#[test]
fn test_builder_id_generation() {
    let mut builder = AstBuilder::new();

    let id1 = builder.next_id();
    let id2 = builder.next_id();
    let id3 = builder.next_id();

    assert_eq!(id1.0, 0);
    assert_eq!(id2.0, 1);
    assert_eq!(id3.0, 2);
}

#[test]
fn test_simple_crate_build() {
    use oxur_ast::ast::*;

    let span = Span::new(0, 50);
    let mod_spans = ModSpans::new(span);
    let id = NodeId(0);

    let crate_ast = Crate::new(vec![], mod_spans, id);

    assert_eq!(crate_ast.items.len(), 0);
    assert_eq!(crate_ast.attrs.len(), 0);
    assert!(!crate_ast.is_placeholder);
}

// Phase 3: Integration tests with syn

#[test]
fn test_parse_hello_world() {
    let source = r#"
fn main() {
    println!("Hello, world!");
}
    "#;

    let crate_node = parse_rust_file(source).expect("Failed to parse");

    assert_eq!(crate_node.items.len(), 1);
    assert_eq!(crate_node.items[0].ident.name, "main");
}

#[test]
fn test_round_trip_hello_world() {
    let source = r#"
fn main() {
    println!("Hello, world!");
}
    "#;

    // Parse Rust
    let crate1 = parse_rust_file(source).expect("Failed to parse");

    // Generate S-expression
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate1).expect("Failed to generate");

    // Parse S-expression
    let sexp_text = print_sexp(&sexp);
    let sexp2 = Parser::parse_str(&sexp_text).expect("Failed to parse S-expr");

    // Build AST
    let mut builder = AstBuilder::new();
    let crate2 = builder.build_crate(&sexp2).expect("Failed to build");

    // Verify
    assert_eq!(crate1.items.len(), crate2.items.len());
    assert_eq!(crate1.items[0].ident.name, crate2.items[0].ident.name);
}

#[test]
fn test_parse_simple_function() {
    let source = r#"
fn add(a: i32, b: i32) -> i32 {
    42
}
    "#;

    let crate_node = parse_rust_file(source).expect("Failed to parse");

    assert_eq!(crate_node.items.len(), 1);

    let item = &crate_node.items[0];
    assert_eq!(item.ident.name, "add");

    // Verify it has parameters
    let ast::ItemKind::Fn(fn_item) = &item.kind else { panic!("Expected Fn"); };
    assert_eq!(fn_item.sig.decl.inputs.len(), 2);
}

#[test]
fn test_parse_with_let_binding() {
    let source = r#"
fn test() {
    let x = 42;
    let y = "hello";
}
    "#;

    let crate_node = parse_rust_file(source).expect("Failed to parse");
    assert_eq!(crate_node.items.len(), 1);
}

#[test]
fn test_parse_unsafe_function() {
    let source = r#"
unsafe fn dangerous() {
    println!("danger!");
}
    "#;

    let crate_node = parse_rust_file(source).expect("Failed to parse");

    let item = &crate_node.items[0];
    let ast::ItemKind::Fn(fn_item) = &item.kind else { panic!("Expected Fn"); };
    assert_eq!(fn_item.sig.header.safety, ast::Safety::Unsafe);
}

#[test]
fn test_parse_const_function() {
    let source = r#"
const fn compile_time() -> i32 {
    42
}
    "#;

    let crate_node = parse_rust_file(source).expect("Failed to parse");

    let item = &crate_node.items[0];
    let ast::ItemKind::Fn(fn_item) = &item.kind else { panic!("Expected Fn"); };
    assert_eq!(fn_item.sig.header.constness, ast::Constness::Const);
}

#[test]
fn test_round_trip_multiple_empty_functions() {
    let rust_code = r#"
fn a() {}
fn b() {}
fn c() {}
fn d() {}
fn e() {}
    "#;

    let crate1 = parse_rust_file(rust_code).unwrap();
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate1).unwrap();
    let sexp_text = print_sexp(&sexp);
    let sexp2 = Parser::parse_str(&sexp_text).unwrap();
    let mut builder = AstBuilder::new();
    let crate2 = builder.build_crate(&sexp2).unwrap();

    assert_eq!(crate1.items.len(), crate2.items.len());
    assert_eq!(crate1.items.len(), 5);
}

#[test]
fn test_parse_public_unsafe_const() {
    let rust_code = "pub unsafe fn danger() {}";

    let crate_node = parse_rust_file(rust_code).unwrap();

    assert_eq!(crate_node.items.len(), 1);
}

#[test]
fn test_parse_const_pub() {
    let rust_code = "pub const fn const_pub() {}";

    let crate_node = parse_rust_file(rust_code).unwrap();

    assert_eq!(crate_node.items.len(), 1);
}

#[test]
fn test_parse_many_functions_mixed_visibility() {
    let rust_code = r#"
fn private1() {}
pub fn public1() {}
fn private2() {}
pub fn public2() {}
unsafe fn unsafe_priv() {}
pub unsafe fn unsafe_pub() {}
const fn const_priv() {}
pub const fn const_pub() {}
    "#;

    let crate_node = parse_rust_file(rust_code).unwrap();

    assert_eq!(crate_node.items.len(), 8);
}

#[test]
fn test_builder_generates_sequential_ids() {
    let rust_code = "fn foo() {} fn bar() {} fn baz() {}";

    let crate1 = parse_rust_file(rust_code).unwrap();
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate1).unwrap();
    let sexp_text = print_sexp(&sexp);
    let sexp2 = Parser::parse_str(&sexp_text).unwrap();
    let mut builder = AstBuilder::new();
    let crate2 = builder.build_crate(&sexp2).unwrap();

    // Builder should generate sequential IDs
    assert!(crate2.items[0].id.0 > 0);
    assert!(crate2.items[1].id.0 > crate2.items[0].id.0);
}
