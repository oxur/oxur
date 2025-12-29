use oxur_ast::ast::*;
use oxur_ast::sexp::{print_sexp, Parser};
use oxur_ast::*;

#[test]
fn test_round_trip_span() {
    // Start with an AST object
    let span = Span::new(0, 10);

    // Generate to SExp
    let gen = Generator::new();
    let sexp1 = gen.generate_span(span);

    // Parse the printed version
    let printed = print_sexp(&sexp1);
    let sexp2 = Parser::parse_str(&printed).unwrap();

    // Build back to AST
    let mut builder = AstBuilder::new();
    let span2 = builder.build_span(&sexp2).unwrap();

    // Generate again
    let sexp3 = gen.generate_span(span2);

    // The two generated S-expressions should be equivalent
    assert_eq!(sexp1, sexp3);
}

#[test]
fn test_round_trip_ident() {
    // Start with an AST object
    let ident = Ident::new("main", Span::new(3, 7));

    let gen = Generator::new();
    let sexp1 = gen.generate_ident(&ident);

    let printed = print_sexp(&sexp1);
    let sexp2 = Parser::parse_str(&printed).unwrap();

    let mut builder = AstBuilder::new();
    let ident2 = builder.build_ident(&sexp2).unwrap();

    let sexp3 = gen.generate_ident(&ident2);

    assert_eq!(sexp1, sexp3);
}

#[test]
fn test_round_trip_path() {
    // Start with an AST object
    let ident = Ident::new("println", Span::new(17, 24));
    let segment = PathSegment::from_ident(ident);
    let path = Path { span: Span::new(17, 24), segments: vec![segment], tokens: None };

    let gen = Generator::new();
    let sexp1 = gen.generate_path(&path);

    let printed = print_sexp(&sexp1);
    let sexp2 = Parser::parse_str(&printed).unwrap();

    let mut builder = AstBuilder::new();
    let path2 = builder.build_path(&sexp2).unwrap();

    let sexp3 = gen.generate_path(&path2);

    assert_eq!(sexp1, sexp3);
}

#[test]
fn test_round_trip_simple_crate() {
    // Use an empty crate for basic round-trip testing
    let crate_node = Crate::new(vec![], ModSpans::new(Span::new(0, 10)), NodeId(0));

    let gen = Generator::new();
    let sexp1 = gen.generate_crate(&crate_node).unwrap();

    // Parse the generated S-expression
    let printed = print_sexp(&sexp1);
    let sexp2 = Parser::parse_str(&printed).unwrap();

    // Build back to AST
    let mut builder = AstBuilder::new();
    let crate_node2 = builder.build_crate(&sexp2).unwrap();

    // Generate again
    let sexp3 = gen.generate_crate(&crate_node2).unwrap();

    // Should be equivalent
    assert_eq!(sexp1, sexp3);
}
