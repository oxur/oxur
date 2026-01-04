/// Comprehensive tests for generics and lifetime functionality
/// Tests Phase 7 Priority 1: Lifetime System implementation
///
/// This file tests:
/// - Lifetime creation and manipulation
/// - LifetimeParam with bounds
/// - Round-trip conversions (AST → S-expr → AST)
/// - Integration with GenericParam and WhereClause
use oxur_ast::ast::*;
use oxur_ast::sexp::{print_sexp, Parser};
use oxur_ast::*;

// ============================================================================
// LIFETIME TESTS
// ============================================================================
// Note: Lifetime methods (generate_lifetime, build_lifetime) are pub(crate)
// and not accessible from test files. We test lifetime functionality indirectly
// through Generics, GenericParam, and WhereClause public APIs below.

// ============================================================================
// LIFETIME PARAMETER TESTS (via GenericParam)
// ============================================================================
// Note: LifetimeParam methods are pub(crate), so we test them indirectly
// through GenericParam which has public interfaces

// ============================================================================
// GENERIC PARAMETER TESTS (LIFETIME VARIANT)
// ============================================================================
// Note: GenericParam methods are private - tested through Generics which is public

// ============================================================================
// GENERICS WITH LIFETIME PARAMETERS
// ============================================================================

#[test]
fn test_generate_generics_with_lifetime() {
    let gen = Generator::new();
    let generics = Generics {
        params: vec![GenericParam {
            attrs: vec![],
            id: NodeId(1),
            span: Span::new(0, 2),
            kind: GenericParamKind::Lifetime(LifetimeParam {
                ident: Ident::new("a", Span::new(0, 2)),
                bounds: vec![],
                colon_span: None,
            }),
        }],
        where_clause: WhereClause::empty(),
        span: Span::new(0, 3),
    };

    let sexp = gen.generate_generics(&generics).unwrap();
    let output = print_sexp(&sexp);

    assert!(output.contains("Generics"));
    assert!(output.contains("GenericParam"));
    assert!(output.contains("Lifetime"));
    assert!(output.contains("\"a\""));
}

#[test]
fn test_round_trip_generics_with_lifetime() {
    let generics = Generics {
        params: vec![GenericParam {
            attrs: vec![],
            id: NodeId(1),
            span: Span::new(0, 2),
            kind: GenericParamKind::Lifetime(LifetimeParam {
                ident: Ident::new("a", Span::new(0, 2)),
                bounds: vec![],
                colon_span: None,
            }),
        }],
        where_clause: WhereClause::empty(),
        span: Span::new(0, 3),
    };

    let gen = Generator::new();
    let sexp1 = gen.generate_generics(&generics).unwrap();

    let printed = print_sexp(&sexp1);
    let sexp2 = Parser::parse_str(&printed).unwrap();

    let mut builder = AstBuilder::new();
    let generics2 = builder.build_generics(&sexp2).unwrap();

    let sexp3 = gen.generate_generics(&generics2).unwrap();

    assert_eq!(sexp1, sexp3);
    assert_eq!(generics.params.len(), generics2.params.len());
}

#[test]
fn test_round_trip_generics_multiple_lifetimes() {
    let generics = Generics {
        params: vec![
            GenericParam {
                attrs: vec![],
                id: NodeId(1),
                span: Span::new(0, 2),
                kind: GenericParamKind::Lifetime(LifetimeParam {
                    ident: Ident::new("a", Span::new(0, 2)),
                    bounds: vec![],
                    colon_span: None,
                }),
            },
            GenericParam {
                attrs: vec![],
                id: NodeId(2),
                span: Span::new(4, 6),
                kind: GenericParamKind::Lifetime(LifetimeParam {
                    ident: Ident::new("b", Span::new(4, 6)),
                    bounds: vec![Lifetime { ident: Ident::new("a", Span::new(9, 11)) }],
                    colon_span: Some(Span::new(6, 7)),
                }),
            },
        ],
        where_clause: WhereClause::empty(),
        span: Span::new(0, 12),
    };

    let gen = Generator::new();
    let sexp1 = gen.generate_generics(&generics).unwrap();

    let printed = print_sexp(&sexp1);
    let sexp2 = Parser::parse_str(&printed).unwrap();

    let mut builder = AstBuilder::new();
    let generics2 = builder.build_generics(&sexp2).unwrap();

    let sexp3 = gen.generate_generics(&generics2).unwrap();

    assert_eq!(sexp1, sexp3);
    assert_eq!(generics.params.len(), 2);
    assert_eq!(generics2.params.len(), 2);

    // Verify first lifetime
    match &generics2.params[0].kind {
        GenericParamKind::Lifetime(lp) => {
            assert_eq!(lp.ident.name, "a");
            assert_eq!(lp.bounds.len(), 0);
        }
        _ => panic!("Expected Lifetime"),
    }

    // Verify second lifetime with bound
    match &generics2.params[1].kind {
        GenericParamKind::Lifetime(lp) => {
            assert_eq!(lp.ident.name, "b");
            assert_eq!(lp.bounds.len(), 1);
            assert_eq!(lp.bounds[0].ident.name, "a");
        }
        _ => panic!("Expected Lifetime"),
    }
}

// ============================================================================
// WHERE CLAUSE LIFETIME TESTS (via WhereClause)
// ============================================================================
// Note: WhereRegionPredicate methods are pub(crate), so we test them
// indirectly through WhereClause which has public interfaces

#[test]
fn test_where_clause_with_region_predicate() {
    let where_clause = WhereClause {
        has_where_token: true,
        predicates: vec![WherePredicate::RegionPredicate(WhereRegionPredicate {
            span: Span::new(0, 10),
            lifetime: Lifetime { ident: Ident::new("a", Span::new(0, 2)) },
            bounds: vec![Lifetime { ident: Ident::new("b", Span::new(5, 7)) }],
        })],
        span: Span::new(0, 15),
    };

    let gen = Generator::new();
    let sexp1 = gen.generate_where_clause(&where_clause).unwrap();

    let printed = print_sexp(&sexp1);
    let sexp2 = Parser::parse_str(&printed).unwrap();

    let mut builder = AstBuilder::new();
    let where_clause2 = builder.build_where_clause(&sexp2).unwrap();

    let sexp3 = gen.generate_where_clause(&where_clause2).unwrap();

    assert_eq!(sexp1, sexp3);
    assert_eq!(where_clause.predicates.len(), where_clause2.predicates.len());
}

// ============================================================================
// GENERIC BOUND OUTLIVES TESTS
// ============================================================================
// Note: GenericBound methods are pub(crate), tested indirectly through
// TypeParam and WhereBoundPredicate which use generic bounds

// ============================================================================
// CODEGEN TESTS - End-to-end code generation
// ============================================================================
// Note: Codegen methods are pub(crate). Full code generation is tested
// through integration tests and the to-rust command.
// Individual lifetime codegen is indirectly tested through function/struct
// generation with lifetime parameters.

// ============================================================================
// PRIORITY 2: TYPE PARAMETERS
// ============================================================================

#[test]
fn test_generate_generics_with_type_param() {
    let gen = Generator::new();
    let generics = Generics {
        params: vec![GenericParam {
            attrs: vec![],
            id: NodeId(1),
            span: Span::new(0, 1),
            kind: GenericParamKind::Type(TypeParam {
                ident: Ident::new("T", Span::new(0, 1)),
                bounds: vec![],
                default: None,
            }),
        }],
        where_clause: WhereClause::empty(),
        span: Span::new(0, 3),
    };

    let sexp = gen.generate_generics(&generics).unwrap();
    let output = print_sexp(&sexp);

    assert!(output.contains("Generics"));
    assert!(output.contains("GenericParam"));
    assert!(output.contains("Type"));
    assert!(output.contains("TypeParam"));
    assert!(output.contains("\"T\""));
}

#[test]
fn test_round_trip_generics_with_type_param() {
    let generics = Generics {
        params: vec![GenericParam {
            attrs: vec![],
            id: NodeId(1),
            span: Span::new(0, 1),
            kind: GenericParamKind::Type(TypeParam {
                ident: Ident::new("T", Span::new(0, 1)),
                bounds: vec![],
                default: None,
            }),
        }],
        where_clause: WhereClause::empty(),
        span: Span::new(0, 3),
    };

    let gen = Generator::new();
    let sexp1 = gen.generate_generics(&generics).unwrap();

    let printed = print_sexp(&sexp1);
    let sexp2 = Parser::parse_str(&printed).unwrap();

    let mut builder = AstBuilder::new();
    let generics2 = builder.build_generics(&sexp2).unwrap();

    let sexp3 = gen.generate_generics(&generics2).unwrap();

    assert_eq!(sexp1, sexp3);
    assert_eq!(generics.params.len(), generics2.params.len());

    // Verify the type param details
    match (&generics.params[0].kind, &generics2.params[0].kind) {
        (GenericParamKind::Type(tp1), GenericParamKind::Type(tp2)) => {
            assert_eq!(tp1.ident.name, tp2.ident.name);
            assert_eq!(tp1.bounds.len(), tp2.bounds.len());
        }
        _ => panic!("Expected Type GenericParamKind"),
    }
}

#[test]
fn test_round_trip_generics_with_type_param_with_trait_bound() {
    let generics = Generics {
        params: vec![GenericParam {
            attrs: vec![],
            id: NodeId(1),
            span: Span::new(0, 1),
            kind: GenericParamKind::Type(TypeParam {
                ident: Ident::new("T", Span::new(0, 1)),
                bounds: vec![GenericBound::Trait(
                    PolyTraitRef {
                        trait_ref: TraitRef {
                            path: Path {
                                span: Span::new(3, 8),
                                segments: vec![PathSegment::from_ident(Ident::new(
                                    "Clone",
                                    Span::new(3, 8),
                                ))],
                                tokens: None,
                            },
                        },
                        bound_lifetimes: vec![],
                    },
                    TraitBoundModifier::None,
                )],
                default: None,
            }),
        }],
        where_clause: WhereClause::empty(),
        span: Span::new(0, 10),
    };

    let gen = Generator::new();
    let sexp1 = gen.generate_generics(&generics).unwrap();

    let printed = print_sexp(&sexp1);
    let sexp2 = Parser::parse_str(&printed).unwrap();

    let mut builder = AstBuilder::new();
    let generics2 = builder.build_generics(&sexp2).unwrap();

    let sexp3 = gen.generate_generics(&generics2).unwrap();

    assert_eq!(sexp1, sexp3);

    // Verify the type param with bound
    match (&generics.params[0].kind, &generics2.params[0].kind) {
        (GenericParamKind::Type(tp1), GenericParamKind::Type(tp2)) => {
            assert_eq!(tp1.ident.name, tp2.ident.name);
            assert_eq!(tp1.bounds.len(), 1);
            assert_eq!(tp2.bounds.len(), 1);
        }
        _ => panic!("Expected Type GenericParamKind"),
    }
}

#[test]
fn test_round_trip_generics_mixed_lifetime_and_type() {
    let generics = Generics {
        params: vec![
            GenericParam {
                attrs: vec![],
                id: NodeId(1),
                span: Span::new(0, 2),
                kind: GenericParamKind::Lifetime(LifetimeParam {
                    ident: Ident::new("a", Span::new(0, 2)),
                    bounds: vec![],
                    colon_span: None,
                }),
            },
            GenericParam {
                attrs: vec![],
                id: NodeId(2),
                span: Span::new(4, 5),
                kind: GenericParamKind::Type(TypeParam {
                    ident: Ident::new("T", Span::new(4, 5)),
                    bounds: vec![],
                    default: None,
                }),
            },
        ],
        where_clause: WhereClause::empty(),
        span: Span::new(0, 6),
    };

    let gen = Generator::new();
    let sexp1 = gen.generate_generics(&generics).unwrap();

    let printed = print_sexp(&sexp1);
    let sexp2 = Parser::parse_str(&printed).unwrap();

    let mut builder = AstBuilder::new();
    let generics2 = builder.build_generics(&sexp2).unwrap();

    let sexp3 = gen.generate_generics(&generics2).unwrap();

    assert_eq!(sexp1, sexp3);
    assert_eq!(generics.params.len(), 2);
    assert_eq!(generics2.params.len(), 2);
}

// ============================================================================
// PRIORITY 2: CONST PARAMETERS
// ============================================================================

#[test]
fn test_generate_generics_with_const_param() {
    let gen = Generator::new();
    let generics = Generics {
        params: vec![GenericParam {
            attrs: vec![],
            id: NodeId(1),
            span: Span::new(0, 1),
            kind: GenericParamKind::Const(ConstParam {
                ident: Ident::new("N", Span::new(6, 7)),
                ty: Ty {
                    id: NodeId(2),
                    kind: TyKind::Path(
                        None,
                        Path {
                            span: Span::new(9, 14),
                            segments: vec![PathSegment::from_ident(Ident::new(
                                "usize",
                                Span::new(9, 14),
                            ))],
                            tokens: None,
                        },
                    ),
                    span: Span::new(9, 14),
                    tokens: None,
                },
                default: None,
            }),
        }],
        where_clause: WhereClause::empty(),
        span: Span::new(0, 15),
    };

    let sexp = gen.generate_generics(&generics).unwrap();
    let output = print_sexp(&sexp);

    assert!(output.contains("Generics"));
    assert!(output.contains("GenericParam"));
    assert!(output.contains("Const"));
    assert!(output.contains("ConstParam"));
    assert!(output.contains("\"N\""));
    assert!(output.contains("\"usize\""));
}

#[test]
fn test_round_trip_generics_with_const_param() {
    let generics = Generics {
        params: vec![GenericParam {
            attrs: vec![],
            id: NodeId(1),
            span: Span::new(0, 1),
            kind: GenericParamKind::Const(ConstParam {
                ident: Ident::new("N", Span::new(6, 7)),
                ty: Ty {
                    id: NodeId(2),
                    kind: TyKind::Path(
                        None,
                        Path {
                            span: Span::new(9, 14),
                            segments: vec![PathSegment::from_ident(Ident::new(
                                "usize",
                                Span::new(9, 14),
                            ))],
                            tokens: None,
                        },
                    ),
                    span: Span::new(9, 14),
                    tokens: None,
                },
                default: None,
            }),
        }],
        where_clause: WhereClause::empty(),
        span: Span::new(0, 15),
    };

    let gen = Generator::new();
    let sexp1 = gen.generate_generics(&generics).unwrap();

    let printed = print_sexp(&sexp1);
    let sexp2 = Parser::parse_str(&printed).unwrap();

    let mut builder = AstBuilder::new();
    let generics2 = builder.build_generics(&sexp2).unwrap();

    let sexp3 = gen.generate_generics(&generics2).unwrap();

    assert_eq!(sexp1, sexp3);
    assert_eq!(generics.params.len(), generics2.params.len());

    // Verify the const param details
    match (&generics.params[0].kind, &generics2.params[0].kind) {
        (GenericParamKind::Const(cp1), GenericParamKind::Const(cp2)) => {
            assert_eq!(cp1.ident.name, cp2.ident.name);
        }
        _ => panic!("Expected Const GenericParamKind"),
    }
}

#[test]
fn test_round_trip_generics_all_param_types() {
    // Test with lifetime, type, and const parameters all together
    let generics = Generics {
        params: vec![
            GenericParam {
                attrs: vec![],
                id: NodeId(1),
                span: Span::new(0, 2),
                kind: GenericParamKind::Lifetime(LifetimeParam {
                    ident: Ident::new("a", Span::new(0, 2)),
                    bounds: vec![],
                    colon_span: None,
                }),
            },
            GenericParam {
                attrs: vec![],
                id: NodeId(2),
                span: Span::new(4, 5),
                kind: GenericParamKind::Type(TypeParam {
                    ident: Ident::new("T", Span::new(4, 5)),
                    bounds: vec![],
                    default: None,
                }),
            },
            GenericParam {
                attrs: vec![],
                id: NodeId(3),
                span: Span::new(7, 8),
                kind: GenericParamKind::Const(ConstParam {
                    ident: Ident::new("N", Span::new(13, 14)),
                    ty: Ty {
                        id: NodeId(4),
                        kind: TyKind::Path(
                            None,
                            Path {
                                span: Span::new(16, 21),
                                segments: vec![PathSegment::from_ident(Ident::new(
                                    "usize",
                                    Span::new(16, 21),
                                ))],
                                tokens: None,
                            },
                        ),
                        span: Span::new(16, 21),
                        tokens: None,
                    },
                    default: None,
                }),
            },
        ],
        where_clause: WhereClause::empty(),
        span: Span::new(0, 22),
    };

    let gen = Generator::new();
    let sexp1 = gen.generate_generics(&generics).unwrap();

    let printed = print_sexp(&sexp1);
    let sexp2 = Parser::parse_str(&printed).unwrap();

    let mut builder = AstBuilder::new();
    let generics2 = builder.build_generics(&sexp2).unwrap();

    let sexp3 = gen.generate_generics(&generics2).unwrap();

    assert_eq!(sexp1, sexp3);
    assert_eq!(generics.params.len(), 3);
    assert_eq!(generics2.params.len(), 3);

    // Verify all three param types
    assert!(matches!(generics2.params[0].kind, GenericParamKind::Lifetime(_)));
    assert!(matches!(generics2.params[1].kind, GenericParamKind::Type(_)));
    assert!(matches!(generics2.params[2].kind, GenericParamKind::Const(_)));
}

// ============================================================================
// PRIORITY 3: WHERE CLAUSES
// ============================================================================

#[test]
fn test_where_clause_with_bound_predicate() {
    // Test: where T: Clone
    let where_clause = WhereClause {
        has_where_token: true,
        predicates: vec![WherePredicate::BoundPredicate(WhereBoundPredicate {
            span: Span::new(0, 15),
            bounded_ty: Ty {
                id: NodeId(1),
                kind: TyKind::Path(
                    None,
                    Path {
                        span: Span::new(6, 7),
                        segments: vec![PathSegment::from_ident(Ident::new("T", Span::new(6, 7)))],
                        tokens: None,
                    },
                ),
                span: Span::new(6, 7),
                tokens: None,
            },
            bounds: vec![GenericBound::Trait(
                PolyTraitRef {
                    trait_ref: TraitRef {
                        path: Path {
                            span: Span::new(9, 14),
                            segments: vec![PathSegment::from_ident(Ident::new(
                                "Clone",
                                Span::new(9, 14),
                            ))],
                            tokens: None,
                        },
                    },
                    bound_lifetimes: vec![],
                },
                TraitBoundModifier::None,
            )],
            bound_lifetimes: vec![],
        })],
        span: Span::new(0, 15),
    };

    let gen = Generator::new();
    let sexp1 = gen.generate_where_clause(&where_clause).unwrap();

    let printed = print_sexp(&sexp1);
    let sexp2 = Parser::parse_str(&printed).unwrap();

    let mut builder = AstBuilder::new();
    let where_clause2 = builder.build_where_clause(&sexp2).unwrap();

    let sexp3 = gen.generate_where_clause(&where_clause2).unwrap();

    assert_eq!(sexp1, sexp3);
    assert_eq!(where_clause.predicates.len(), 1);
    assert_eq!(where_clause2.predicates.len(), 1);
}

#[test]
fn test_where_clause_with_multiple_bounds() {
    // Test: where T: Clone + Send
    let where_clause = WhereClause {
        has_where_token: true,
        predicates: vec![WherePredicate::BoundPredicate(WhereBoundPredicate {
            span: Span::new(0, 20),
            bounded_ty: Ty {
                id: NodeId(1),
                kind: TyKind::Path(
                    None,
                    Path {
                        span: Span::new(6, 7),
                        segments: vec![PathSegment::from_ident(Ident::new("T", Span::new(6, 7)))],
                        tokens: None,
                    },
                ),
                span: Span::new(6, 7),
                tokens: None,
            },
            bounds: vec![
                GenericBound::Trait(
                    PolyTraitRef {
                        trait_ref: TraitRef {
                            path: Path {
                                span: Span::new(9, 14),
                                segments: vec![PathSegment::from_ident(Ident::new(
                                    "Clone",
                                    Span::new(9, 14),
                                ))],
                                tokens: None,
                            },
                        },
                        bound_lifetimes: vec![],
                    },
                    TraitBoundModifier::None,
                ),
                GenericBound::Trait(
                    PolyTraitRef {
                        trait_ref: TraitRef {
                            path: Path {
                                span: Span::new(17, 21),
                                segments: vec![PathSegment::from_ident(Ident::new(
                                    "Send",
                                    Span::new(17, 21),
                                ))],
                                tokens: None,
                            },
                        },
                        bound_lifetimes: vec![],
                    },
                    TraitBoundModifier::None,
                ),
            ],
            bound_lifetimes: vec![],
        })],
        span: Span::new(0, 20),
    };

    let gen = Generator::new();
    let sexp1 = gen.generate_where_clause(&where_clause).unwrap();

    let printed = print_sexp(&sexp1);
    let sexp2 = Parser::parse_str(&printed).unwrap();

    let mut builder = AstBuilder::new();
    let where_clause2 = builder.build_where_clause(&sexp2).unwrap();

    let sexp3 = gen.generate_where_clause(&where_clause2).unwrap();

    assert_eq!(sexp1, sexp3);

    // Verify the bound predicate details
    match &where_clause2.predicates[0] {
        WherePredicate::BoundPredicate(bp) => {
            assert_eq!(bp.bounds.len(), 2);
        }
        _ => panic!("Expected BoundPredicate"),
    }
}

#[test]
fn test_where_clause_with_eq_predicate() {
    // Test: where T = String
    let where_clause = WhereClause {
        has_where_token: true,
        predicates: vec![WherePredicate::EqPredicate(WhereEqPredicate {
            span: Span::new(0, 15),
            lhs_ty: Ty {
                id: NodeId(1),
                kind: TyKind::Path(
                    None,
                    Path {
                        span: Span::new(6, 7),
                        segments: vec![PathSegment::from_ident(Ident::new("T", Span::new(6, 7)))],
                        tokens: None,
                    },
                ),
                span: Span::new(6, 7),
                tokens: None,
            },
            rhs_ty: Ty {
                id: NodeId(2),
                kind: TyKind::Path(
                    None,
                    Path {
                        span: Span::new(10, 16),
                        segments: vec![PathSegment::from_ident(Ident::new(
                            "String",
                            Span::new(10, 16),
                        ))],
                        tokens: None,
                    },
                ),
                span: Span::new(10, 16),
                tokens: None,
            },
        })],
        span: Span::new(0, 15),
    };

    let gen = Generator::new();
    let sexp1 = gen.generate_where_clause(&where_clause).unwrap();

    let printed = print_sexp(&sexp1);
    let sexp2 = Parser::parse_str(&printed).unwrap();

    let mut builder = AstBuilder::new();
    let where_clause2 = builder.build_where_clause(&sexp2).unwrap();

    let sexp3 = gen.generate_where_clause(&where_clause2).unwrap();

    assert_eq!(sexp1, sexp3);
    assert_eq!(where_clause.predicates.len(), 1);
    assert_eq!(where_clause2.predicates.len(), 1);

    // Verify it's an EqPredicate
    assert!(matches!(where_clause2.predicates[0], WherePredicate::EqPredicate(_)));
}

#[test]
fn test_where_clause_mixed_predicates() {
    // Test: where T: Clone, 'a: 'b, U = String
    let where_clause = WhereClause {
        has_where_token: true,
        predicates: vec![
            WherePredicate::BoundPredicate(WhereBoundPredicate {
                span: Span::new(0, 10),
                bounded_ty: Ty {
                    id: NodeId(1),
                    kind: TyKind::Path(
                        None,
                        Path {
                            span: Span::new(6, 7),
                            segments: vec![PathSegment::from_ident(Ident::new(
                                "T",
                                Span::new(6, 7),
                            ))],
                            tokens: None,
                        },
                    ),
                    span: Span::new(6, 7),
                    tokens: None,
                },
                bounds: vec![GenericBound::Trait(
                    PolyTraitRef {
                        trait_ref: TraitRef {
                            path: Path {
                                span: Span::new(9, 14),
                                segments: vec![PathSegment::from_ident(Ident::new(
                                    "Clone",
                                    Span::new(9, 14),
                                ))],
                                tokens: None,
                            },
                        },
                        bound_lifetimes: vec![],
                    },
                    TraitBoundModifier::None,
                )],
                bound_lifetimes: vec![],
            }),
            WherePredicate::RegionPredicate(WhereRegionPredicate {
                span: Span::new(16, 23),
                lifetime: Lifetime { ident: Ident::new("a", Span::new(16, 18)) },
                bounds: vec![Lifetime { ident: Ident::new("b", Span::new(21, 23)) }],
            }),
            WherePredicate::EqPredicate(WhereEqPredicate {
                span: Span::new(25, 35),
                lhs_ty: Ty {
                    id: NodeId(2),
                    kind: TyKind::Path(
                        None,
                        Path {
                            span: Span::new(25, 26),
                            segments: vec![PathSegment::from_ident(Ident::new(
                                "U",
                                Span::new(25, 26),
                            ))],
                            tokens: None,
                        },
                    ),
                    span: Span::new(25, 26),
                    tokens: None,
                },
                rhs_ty: Ty {
                    id: NodeId(3),
                    kind: TyKind::Path(
                        None,
                        Path {
                            span: Span::new(29, 35),
                            segments: vec![PathSegment::from_ident(Ident::new(
                                "String",
                                Span::new(29, 35),
                            ))],
                            tokens: None,
                        },
                    ),
                    span: Span::new(29, 35),
                    tokens: None,
                },
            }),
        ],
        span: Span::new(0, 35),
    };

    let gen = Generator::new();
    let sexp1 = gen.generate_where_clause(&where_clause).unwrap();

    let printed = print_sexp(&sexp1);
    let sexp2 = Parser::parse_str(&printed).unwrap();

    let mut builder = AstBuilder::new();
    let where_clause2 = builder.build_where_clause(&sexp2).unwrap();

    let sexp3 = gen.generate_where_clause(&where_clause2).unwrap();

    assert_eq!(sexp1, sexp3);
    assert_eq!(where_clause.predicates.len(), 3);
    assert_eq!(where_clause2.predicates.len(), 3);

    // Verify all three predicate types
    assert!(matches!(where_clause2.predicates[0], WherePredicate::BoundPredicate(_)));
    assert!(matches!(where_clause2.predicates[1], WherePredicate::RegionPredicate(_)));
    assert!(matches!(where_clause2.predicates[2], WherePredicate::EqPredicate(_)));
}

#[test]
fn test_complete_generics_with_where_clause() {
    // Test: <'a, T> where T: Clone + 'a
    let generics = Generics {
        params: vec![
            GenericParam {
                attrs: vec![],
                id: NodeId(1),
                span: Span::new(0, 2),
                kind: GenericParamKind::Lifetime(LifetimeParam {
                    ident: Ident::new("a", Span::new(0, 2)),
                    bounds: vec![],
                    colon_span: None,
                }),
            },
            GenericParam {
                attrs: vec![],
                id: NodeId(2),
                span: Span::new(4, 5),
                kind: GenericParamKind::Type(TypeParam {
                    ident: Ident::new("T", Span::new(4, 5)),
                    bounds: vec![],
                    default: None,
                }),
            },
        ],
        where_clause: WhereClause {
            has_where_token: true,
            predicates: vec![WherePredicate::BoundPredicate(WhereBoundPredicate {
                span: Span::new(0, 20),
                bounded_ty: Ty {
                    id: NodeId(3),
                    kind: TyKind::Path(
                        None,
                        Path {
                            span: Span::new(6, 7),
                            segments: vec![PathSegment::from_ident(Ident::new(
                                "T",
                                Span::new(6, 7),
                            ))],
                            tokens: None,
                        },
                    ),
                    span: Span::new(6, 7),
                    tokens: None,
                },
                bounds: vec![
                    GenericBound::Trait(
                        PolyTraitRef {
                            trait_ref: TraitRef {
                                path: Path {
                                    span: Span::new(9, 14),
                                    segments: vec![PathSegment::from_ident(Ident::new(
                                        "Clone",
                                        Span::new(9, 14),
                                    ))],
                                    tokens: None,
                                },
                            },
                            bound_lifetimes: vec![],
                        },
                        TraitBoundModifier::None,
                    ),
                    GenericBound::Outlives(Lifetime { ident: Ident::new("a", Span::new(17, 19)) }),
                ],
                bound_lifetimes: vec![],
            })],
            span: Span::new(0, 20),
        },
        span: Span::new(0, 25),
    };

    let gen = Generator::new();
    let sexp1 = gen.generate_generics(&generics).unwrap();

    let printed = print_sexp(&sexp1);
    let sexp2 = Parser::parse_str(&printed).unwrap();

    let mut builder = AstBuilder::new();
    let generics2 = builder.build_generics(&sexp2).unwrap();

    let sexp3 = gen.generate_generics(&generics2).unwrap();

    assert_eq!(sexp1, sexp3);
    assert_eq!(generics.params.len(), 2);
    assert_eq!(generics2.params.len(), 2);
    assert_eq!(generics.where_clause.predicates.len(), 1);
    assert_eq!(generics2.where_clause.predicates.len(), 1);

    // Verify the where clause bound predicate has both trait and lifetime bounds
    match &generics2.where_clause.predicates[0] {
        WherePredicate::BoundPredicate(bp) => {
            assert_eq!(bp.bounds.len(), 2);
            assert!(matches!(bp.bounds[0], GenericBound::Trait(_, _)));
            assert!(matches!(bp.bounds[1], GenericBound::Outlives(_)));
        }
        _ => panic!("Expected BoundPredicate"),
    }
}
