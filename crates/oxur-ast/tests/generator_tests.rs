use oxur_ast::ast::*;
use oxur_ast::sexp::print_sexp;
use oxur_ast::*;

#[test]
fn test_generate_span() {
    let span = Span::new(0, 10);
    let gen = Generator::new();
    let sexp = gen.generate_span(span);

    let output = print_sexp(&sexp);
    assert!(output.contains("Span"));
    assert!(output.contains(":lo"));
    assert!(output.contains("0"));
    assert!(output.contains(":hi"));
    assert!(output.contains("10"));
}

#[test]
fn test_generate_empty_crate() {
    let crate_node = Crate::new(vec![], ModSpans::new(Span::new(0, 10)), NodeId(0));

    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node).unwrap();

    let output = print_sexp(&sexp);
    assert!(output.contains("Crate"));
    assert!(output.contains(":items"));
    assert!(output.contains(":spans"));
}

#[test]
fn test_generate_ident() {
    let ident = Ident::new("main", Span::new(3, 7));
    let gen = Generator::new();
    let sexp = gen.generate_ident(&ident);

    let output = print_sexp(&sexp);
    assert!(output.contains("Ident"));
    assert!(output.contains(":name"));
    assert!(output.contains("main"));
}

#[test]
fn test_generate_path() {
    let ident = Ident::new("println", Span::new(17, 24));
    let segment = PathSegment::from_ident(ident);
    let path = Path { span: Span::new(17, 24), segments: vec![segment], tokens: None };

    let gen = Generator::new();
    let sexp = gen.generate_path(&path);

    let output = print_sexp(&sexp);
    assert!(output.contains("Path"));
    assert!(output.contains("PathSegment"));
    assert!(output.contains("println"));
}

#[test]
fn test_generate_visibility() {
    let gen = Generator::new();

    let vis = Visibility::Public;
    let sexp = gen.generate_visibility(&vis);
    let output = print_sexp(&sexp);
    assert!(output.contains("Public"));

    let vis = Visibility::Inherited;
    let sexp = gen.generate_visibility(&vis);
    let output = print_sexp(&sexp);
    assert!(output.contains("Inherited"));
}

#[test]
fn test_generate_mac_call() {
    let ident = Ident::new("println", Span::new(17, 24));
    let segment = PathSegment::from_ident(ident);
    let path = Path { span: Span::new(17, 24), segments: vec![segment], tokens: None };

    let args = MacArgs::Delimited {
        dspan: DelSpan::new(Span::new(24, 25), Span::new(42, 43)),
        delim: Delimiter::Paren,
        tokens: TokenStream::Source("\"Hello, world!\"".to_string()),
    };

    let mac_call = MacCall::new(path, args);

    let gen = Generator::new();
    let sexp = gen.generate_mac_call(&mac_call).unwrap();

    let output = print_sexp(&sexp);
    assert!(output.contains("MacCall"));
    assert!(output.contains("println"));
    assert!(output.contains("Delimited"));
    assert!(output.contains("Hello, world!"));
}
