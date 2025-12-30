use oxur_ast::ast::*;
use oxur_ast::sexp::print_sexp;
use oxur_ast::Generator;

fn main() {
    println!("Building Hello World AST manually...\n");

    // Build the AST by hand
    let println_ident = Ident::new("println", Span::new(17, 24));
    let println_segment = PathSegment::from_ident(println_ident);
    let println_path =
        Path { span: Span::new(17, 24), segments: vec![println_segment], tokens: None };

    let mac_args = MacArgs::Delimited {
        dspan: DelSpan::new(Span::new(24, 25), Span::new(42, 43)),
        delim: Delimiter::Paren,
        tokens: TokenStream::Source("\"Hello, world!\"".to_string()),
    };

    let mac_call = MacCall::new(println_path, mac_args);

    let expr = Expr {
        id: NodeId(2),
        kind: ExprKind::MacCall(mac_call),
        span: Span::new(17, 44),
        attrs: vec![],
        tokens: None,
    };

    let stmt = Stmt { id: NodeId(1), kind: StmtKind::Semi(expr), span: Span::new(17, 44) };

    let block = Block::new(vec![stmt], NodeId(3), Span::new(13, 48));

    let fn_sig = FnSig {
        header: FnHeader {
            safety: Safety::Default,
            coroutine_kind: None,
            constness: Constness::NotConst,
            ext: Extern::None,
        },
        decl: FnDecl { inputs: vec![], output: FnRetTy::Default(Span::new(10, 10)) },
        span: Span::new(0, 10),
    };

    let fn_item = Fn {
        defaultness: Defaultness::Final,
        sig: fn_sig,
        generics: Generics {
            params: Vec::new(),
            where_clause: WhereClause {
                has_where_token: false,
                predicates: Vec::new(),
                span: Span::new(10, 10),
            },
            span: Span::new(7, 10),
        },
        body: Some(block),
    };

    let item = Item {
        attrs: vec![],
        id: NodeId(0),
        span: Span::new(0, 50),
        vis: Visibility::Inherited,
        ident: Ident::new("main", Span::new(3, 7)),
        kind: ItemKind::Fn(Box::new(fn_item)),
        tokens: None,
    };

    let crate_node = Crate::new(vec![item], ModSpans::new(Span::new(0, 50)), NodeId(0));

    println!("✓ AST built successfully!\n");

    // Generate S-expression
    println!("Generating S-expression...\n");
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node).expect("Failed to generate S-expression");

    println!("✓ S-expression generated!\n");

    // Print it
    let output = print_sexp(&sexp);
    println!("Generated S-expression:\n");
    println!("{}", output);
}
