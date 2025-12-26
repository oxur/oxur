use oxur_ast::builder::AstBuilder;
use oxur_ast::sexp::{Parser, print_sexp};

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
    let input = r#"(Crate :attrs () :items () :spans (ModSpans :inner-span (Span :lo 0 :hi 10)) :id 0)"#;

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
