use oxur_ast::ast::*;

// ===== Span Tests =====

#[test]
fn test_span_dummy() {
    let span = Span::DUMMY;
    assert_eq!(span.lo, 0);
    assert_eq!(span.hi, 0);
    assert_eq!(span.ctxt, 0);
}

#[test]
fn test_span_new() {
    let span = Span::new(10, 20);
    assert_eq!(span.lo, 10);
    assert_eq!(span.hi, 20);
    assert_eq!(span.ctxt, 0);
}

#[test]
fn test_span_with_ctxt() {
    let span = Span::with_ctxt(5, 15, 42);
    assert_eq!(span.lo, 5);
    assert_eq!(span.hi, 15);
    assert_eq!(span.ctxt, 42);
}

#[test]
fn test_span_equality() {
    let span1 = Span::new(10, 20);
    let span2 = Span::new(10, 20);
    assert_eq!(span1, span2);
}

#[test]
fn test_span_inequality() {
    let span1 = Span::new(10, 20);
    let span2 = Span::new(10, 21);
    assert_ne!(span1, span2);
}

#[test]
fn test_mod_spans_new() {
    let inner = Span::new(0, 100);
    let mod_spans = ModSpans::new(inner);
    assert_eq!(mod_spans.inner_span, inner);
    assert_eq!(mod_spans.inject_use_span, Span::DUMMY);
}

#[test]
fn test_mod_spans_custom() {
    let inner = Span::new(0, 100);
    let inject = Span::new(10, 20);
    let mod_spans = ModSpans {
        inner_span: inner,
        inject_use_span: inject,
    };
    assert_eq!(mod_spans.inner_span, inner);
    assert_eq!(mod_spans.inject_use_span, inject);
}

#[test]
fn test_del_span_new() {
    let open = Span::new(0, 1);
    let close = Span::new(10, 11);
    let del_span = DelSpan::new(open, close);
    assert_eq!(del_span.open, open);
    assert_eq!(del_span.close, close);
}

// ===== Expr Tests =====

#[test]
fn test_mac_call_new() {
    let path = Path::from_ident(Ident::new("println", Span::DUMMY));
    let args = MacArgs::Empty;
    let mac_call = MacCall::new(path.clone(), args.clone());

    assert_eq!(mac_call.path, path);
    assert_eq!(mac_call.args, args);
    assert!(mac_call.prior_type_ascription.is_none());
}

#[test]
fn test_mac_call_with_delimited_args() {
    let path = Path::from_ident(Ident::new("vec", Span::DUMMY));
    let del_span = DelSpan::new(Span::new(0, 1), Span::new(10, 11));
    let args = MacArgs::Delimited {
        dspan: del_span,
        delim: Delimiter::Bracket,
        tokens: TokenStream::Empty,
    };
    let mac_call = MacCall::new(path, args);

    assert!(matches!(mac_call.args, MacArgs::Delimited { .. }));
}

#[test]
fn test_delimiter_variants() {
    assert_eq!(Delimiter::Paren, Delimiter::Paren);
    assert_eq!(Delimiter::Brace, Delimiter::Brace);
    assert_eq!(Delimiter::Bracket, Delimiter::Bracket);
    assert_eq!(Delimiter::Invisible, Delimiter::Invisible);
    assert_ne!(Delimiter::Paren, Delimiter::Brace);
}

#[test]
fn test_lit_string() {
    let lit = Lit {
        kind: LitKind::Str("hello".to_string()),
        span: Span::new(0, 5),
    };

    match lit.kind {
        LitKind::Str(ref s) => assert_eq!(s, "hello"),
        _ => panic!("Expected string literal"),
    }
}

#[test]
fn test_lit_int() {
    let lit = Lit {
        kind: LitKind::Int(42),
        span: Span::new(0, 2),
    };

    match lit.kind {
        LitKind::Int(n) => assert_eq!(n, 42),
        _ => panic!("Expected int literal"),
    }
}

#[test]
fn test_lit_negative_int() {
    let lit = Lit {
        kind: LitKind::Int(-100),
        span: Span::new(0, 4),
    };

    match lit.kind {
        LitKind::Int(n) => assert_eq!(n, -100),
        _ => panic!("Expected int literal"),
    }
}

#[test]
fn test_expr_kind_macro_call() {
    let path = Path::from_ident(Ident::new("test", Span::DUMMY));
    let mac_call = MacCall::new(path, MacArgs::Empty);
    let kind = ExprKind::MacCall(mac_call);

    assert!(matches!(kind, ExprKind::MacCall(_)));
}

#[test]
fn test_expr_kind_lit() {
    let lit = Lit {
        kind: LitKind::Int(123),
        span: Span::DUMMY,
    };
    let kind = ExprKind::Lit(lit);

    assert!(matches!(kind, ExprKind::Lit(_)));
}

#[test]
fn test_expr_kind_path() {
    let path = Path::from_ident(Ident::new("x", Span::DUMMY));
    let kind = ExprKind::Path(None, path);

    assert!(matches!(kind, ExprKind::Path(None, _)));
}

#[test]
fn test_mac_args_empty() {
    let args = MacArgs::Empty;
    assert!(matches!(args, MacArgs::Empty));
}

#[test]
fn test_mac_args_eq() {
    let args = MacArgs::Eq {
        eq_span: Span::new(0, 1),
        tokens: TokenStream::Empty,
    };
    assert!(matches!(args, MacArgs::Eq { .. }));
}

// ===== Type Tests =====

#[test]
fn test_node_id_equality() {
    let id1 = NodeId(42);
    let id2 = NodeId(42);
    let id3 = NodeId(43);

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn test_safety_variants() {
    assert!(matches!(Safety::Default, Safety::Default));
    assert!(matches!(Safety::Unsafe, Safety::Unsafe));
    assert_ne!(Safety::Default, Safety::Unsafe);
}

#[test]
fn test_constness_variants() {
    assert!(matches!(Constness::NotConst, Constness::NotConst));
    assert!(matches!(Constness::Const, Constness::Const));
    assert_ne!(Constness::NotConst, Constness::Const);
}

#[test]
fn test_defaultness_variants() {
    assert!(matches!(Defaultness::Final, Defaultness::Final));
    assert!(matches!(Defaultness::Default, Defaultness::Default));
    assert_ne!(Defaultness::Final, Defaultness::Default);
}

#[test]
fn test_fn_ret_ty_default() {
    let ret = FnRetTy::Default(Span::DUMMY);
    assert!(matches!(ret, FnRetTy::Default(_)));
}

#[test]
fn test_generics_empty() {
    let gen = Generics::empty();
    assert_eq!(gen.params.len(), 0);
    assert!(!gen.where_clause.has_where_token);
}

#[test]
fn test_where_clause_empty() {
    let wc = WhereClause::empty();
    assert!(!wc.has_where_token);
    assert_eq!(wc.predicates.len(), 0);
}

// ===== Item Tests =====

#[test]
fn test_visibility_public() {
    let vis = Visibility::Public;
    assert!(matches!(vis, Visibility::Public));
}

#[test]
fn test_visibility_inherited() {
    let vis = Visibility::Inherited;
    assert!(matches!(vis, Visibility::Inherited));
}

#[test]
fn test_fn_header_new() {
    let header = FnHeader {
        safety: Safety::Default,
        coroutine_kind: None,
        constness: Constness::NotConst,
        ext: Extern::None,
    };

    assert!(matches!(header.safety, Safety::Default));
    assert!(matches!(header.constness, Constness::NotConst));
    assert!(matches!(header.ext, Extern::None));
}

#[test]
fn test_fn_decl_new() {
    let decl = FnDecl {
        inputs: vec![],
        output: FnRetTy::Default(Span::DUMMY),
    };

    assert_eq!(decl.inputs.len(), 0);
    assert!(matches!(decl.output, FnRetTy::Default(_)));
}

#[test]
fn test_fn_sig_new() {
    let header = FnHeader {
        safety: Safety::Unsafe,
        coroutine_kind: None,
        constness: Constness::Const,
        ext: Extern::None,
    };
    let decl = FnDecl {
        inputs: vec![],
        output: FnRetTy::Default(Span::DUMMY),
    };
    let sig = FnSig {
        header,
        decl,
        span: Span::DUMMY,
    };

    assert!(matches!(sig.header.safety, Safety::Unsafe));
    assert!(matches!(sig.header.constness, Constness::Const));
}

#[test]
fn test_crate_new() {
    let span = Span::new(0, 100);
    let mod_spans = ModSpans::new(span);
    let krate = Crate::new(vec![], mod_spans, NodeId(0));

    assert_eq!(krate.items.len(), 0);
    assert_eq!(krate.attrs.len(), 0);
    assert!(!krate.is_placeholder);
    assert_eq!(krate.id, NodeId(0));
}

#[test]
fn test_block_new() {
    let stmts = vec![];
    let id = NodeId(1);
    let span = Span::new(0, 10);
    let block = Block::new(stmts, id, span);

    assert_eq!(block.stmts.len(), 0);
    assert_eq!(block.id, id);
    assert_eq!(block.span, span);
}

#[test]
fn test_token_stream_empty() {
    let ts = TokenStream::Empty;
    assert!(matches!(ts, TokenStream::Empty));
}

#[test]
fn test_token_stream_source() {
    let ts = TokenStream::Source("println!(\"hello\")".to_string());
    match ts {
        TokenStream::Source(ref s) => assert_eq!(s, "println!(\"hello\")"),
        _ => panic!("Expected Source variant"),
    }
}

#[test]
fn test_extern_none() {
    let ext = Extern::None;
    assert!(matches!(ext, Extern::None));
}

#[test]
fn test_extern_explicit() {
    let ext = Extern::Explicit("C".to_string());
    match ext {
        Extern::Explicit(ref s) => assert_eq!(s, "C"),
        _ => panic!("Expected Explicit variant"),
    }
}

#[test]
fn test_path_segment_new() {
    let ident = Ident::new("std", Span::DUMMY);
    let seg = PathSegment {
        ident,
        id: NodeId(0),
        args: None,
    };

    assert_eq!(seg.ident.name, "std");
    assert!(seg.args.is_none());
}

#[test]
fn test_ident_equality() {
    let ident1 = Ident::new("foo", Span::new(0, 3));
    let ident2 = Ident::new("foo", Span::new(0, 3));
    let ident3 = Ident::new("bar", Span::new(0, 3));

    assert_eq!(ident1.name, ident2.name);
    assert_ne!(ident1.name, ident3.name);
}
