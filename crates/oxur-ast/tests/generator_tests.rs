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

#[test]
fn test_generate_crate_with_placeholder() {
    let mut crate_node = Crate::new(vec![], ModSpans::new(Span::new(0, 10)), NodeId(0));
    crate_node.is_placeholder = true;

    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node).unwrap();

    let output = print_sexp(&sexp);
    assert!(output.contains("Crate"));
    assert!(output.contains(":is-placeholder"));
    assert!(output.contains("true"));
}

#[test]
fn test_generate_span_with_ctxt() {
    let span = Span { lo: 0, hi: 10, ctxt: 42 };
    let gen = Generator::new();
    let sexp = gen.generate_span(span);

    let output = print_sexp(&sexp);
    assert!(output.contains("Span"));
    assert!(output.contains(":ctxt"));
    assert!(output.contains("42"));
}

#[test]
fn test_generator_default() {
    let gen1 = Generator::new();
    let gen2 = Generator::default();

    // Both should be able to generate the same output
    let span = Span::new(0, 5);
    let sexp1 = gen1.generate_span(span);
    let sexp2 = gen2.generate_span(span);

    assert_eq!(print_sexp(&sexp1), print_sexp(&sexp2));
}

#[test]
fn test_generate_crate_with_items() {
    let item = Item {
        ident: Ident::new("foo", Span::new(0, 3)),
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

    let crate_node = Crate::new(vec![item], ModSpans::new(Span::new(0, 10)), NodeId(0));

    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node).unwrap();

    let output = print_sexp(&sexp);
    assert!(output.contains("Crate"));
    assert!(output.contains(":items"));
    assert!(output.contains("Item"));
    assert!(output.contains("foo"));
}
