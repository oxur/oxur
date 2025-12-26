use oxur_ast::sexp::{Parser, SExp};
use oxur_ast::ParseError;

#[test]
fn test_parse_symbol() {
    let sexp = Parser::parse_str("foo").unwrap();
    match sexp {
        SExp::Symbol(s) => assert_eq!(s.value, "foo"),
        _ => panic!("Expected Symbol, got {:?}", sexp),
    }
}

#[test]
fn test_parse_keyword() {
    let sexp = Parser::parse_str(":name").unwrap();
    match sexp {
        SExp::Keyword(k) => assert_eq!(k.name, "name"),
        _ => panic!("Expected Keyword"),
    }
}

#[test]
fn test_parse_string() {
    let sexp = Parser::parse_str(r#""hello""#).unwrap();
    match sexp {
        SExp::String(s) => assert_eq!(s.value, "hello"),
        _ => panic!("Expected String"),
    }
}

#[test]
fn test_parse_string_with_escapes() {
    let sexp = Parser::parse_str(r#""hello\nworld""#).unwrap();
    match sexp {
        SExp::String(s) => assert_eq!(s.value, "hello\nworld"),
        _ => panic!("Expected String"),
    }
}

#[test]
fn test_parse_number() {
    let sexp = Parser::parse_str("42").unwrap();
    match sexp {
        SExp::Number(n) => assert_eq!(n.value, "42"),
        _ => panic!("Expected Number"),
    }
}

#[test]
fn test_parse_negative_number() {
    let sexp = Parser::parse_str("-42").unwrap();
    match sexp {
        SExp::Number(n) => assert_eq!(n.value, "-42"),
        _ => panic!("Expected Number"),
    }
}

#[test]
fn test_parse_nil() {
    let sexp = Parser::parse_str("nil").unwrap();
    match sexp {
        SExp::Nil(_) => (),
        _ => panic!("Expected Nil"),
    }
}

#[test]
fn test_parse_empty_list() {
    let sexp = Parser::parse_str("()").unwrap();
    match sexp {
        SExp::List(l) => assert_eq!(l.elements.len(), 0),
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_list_with_one_element() {
    let sexp = Parser::parse_str("(foo)").unwrap();
    match sexp {
        SExp::List(l) => {
            assert_eq!(l.elements.len(), 1);
            match &l.elements[0] {
                SExp::Symbol(s) => assert_eq!(s.value, "foo"),
                _ => panic!("Expected Symbol in list"),
            }
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_list_with_multiple_elements() {
    let sexp = Parser::parse_str("(foo bar baz)").unwrap();
    match sexp {
        SExp::List(l) => {
            assert_eq!(l.elements.len(), 3);
            match &l.elements[0] {
                SExp::Symbol(s) => assert_eq!(s.value, "foo"),
                _ => panic!("Expected Symbol"),
            }
            match &l.elements[1] {
                SExp::Symbol(s) => assert_eq!(s.value, "bar"),
                _ => panic!("Expected Symbol"),
            }
            match &l.elements[2] {
                SExp::Symbol(s) => assert_eq!(s.value, "baz"),
                _ => panic!("Expected Symbol"),
            }
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_nested_list() {
    let sexp = Parser::parse_str("(foo (bar baz))").unwrap();
    match sexp {
        SExp::List(l) => {
            assert_eq!(l.elements.len(), 2);
            match &l.elements[1] {
                SExp::List(inner) => {
                    assert_eq!(inner.elements.len(), 2);
                }
                _ => panic!("Expected nested List"),
            }
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_deeply_nested_list() {
    let sexp = Parser::parse_str("(a (b (c (d))))").unwrap();
    match sexp {
        SExp::List(l) => assert_eq!(l.elements.len(), 2),
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_list_with_mixed_types() {
    let sexp = Parser::parse_str(r#"(foo 42 "hello" :key nil)"#).unwrap();
    match sexp {
        SExp::List(l) => {
            assert_eq!(l.elements.len(), 5);
            assert!(matches!(l.elements[0], SExp::Symbol(_)));
            assert!(matches!(l.elements[1], SExp::Number(_)));
            assert!(matches!(l.elements[2], SExp::String(_)));
            assert!(matches!(l.elements[3], SExp::Keyword(_)));
            assert!(matches!(l.elements[4], SExp::Nil(_)));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_keyword_value_pairs() {
    let sexp = Parser::parse_str("(:name \"foo\" :value 42)").unwrap();
    match sexp {
        SExp::List(l) => {
            assert_eq!(l.elements.len(), 4);
            match &l.elements[0] {
                SExp::Keyword(k) => assert_eq!(k.name, "name"),
                _ => panic!("Expected Keyword"),
            }
            match &l.elements[1] {
                SExp::String(s) => assert_eq!(s.value, "foo"),
                _ => panic!("Expected String"),
            }
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_empty_input() {
    let result = Parser::parse_str("");
    assert!(result.is_err());
    match result {
        Err(ParseError::EmptyInput) => (),
        _ => panic!("Expected EmptyInput error"),
    }
}

#[test]
fn test_parse_whitespace_only() {
    let result = Parser::parse_str("   \n\t   ");
    assert!(result.is_err());
}

#[test]
fn test_parse_comment_only() {
    let result = Parser::parse_str("; just a comment");
    assert!(result.is_err());
}

#[test]
fn test_parse_with_leading_whitespace() {
    let sexp = Parser::parse_str("  foo").unwrap();
    match sexp {
        SExp::Symbol(s) => assert_eq!(s.value, "foo"),
        _ => panic!("Expected Symbol"),
    }
}

#[test]
fn test_parse_with_trailing_whitespace() {
    let sexp = Parser::parse_str("foo  ").unwrap();
    match sexp {
        SExp::Symbol(s) => assert_eq!(s.value, "foo"),
        _ => panic!("Expected Symbol"),
    }
}

#[test]
fn test_parse_with_comment() {
    let sexp = Parser::parse_str("foo ; comment").unwrap();
    match sexp {
        SExp::Symbol(s) => assert_eq!(s.value, "foo"),
        _ => panic!("Expected Symbol"),
    }
}

#[test]
fn test_parse_unterminated_list() {
    let result = Parser::parse_str("(foo bar");
    assert!(result.is_err());
    match result {
        Err(ParseError::UnterminatedList { .. }) => (),
        _ => panic!("Expected UnterminatedList error"),
    }
}

#[test]
fn test_parse_unexpected_close_paren() {
    let result = Parser::parse_str(")");
    assert!(result.is_err());
    match result {
        Err(ParseError::UnexpectedCloseParen { .. }) => (),
        _ => panic!("Expected UnexpectedCloseParen error"),
    }
}

#[test]
fn test_parse_mismatched_parens() {
    let result = Parser::parse_str("(foo))");
    // Should parse (foo) successfully, extra ) should cause issue if we try to parse it
    // But our parser only parses one S-expression
    let sexp = result.unwrap();
    match sexp {
        SExp::List(_) => (),
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_complex_nested_structure() {
    let input = r#"(Crate
      :attrs ()
      :items ((Item :name "foo")))"#;
    let sexp = Parser::parse_str(input).unwrap();
    match sexp {
        SExp::List(l) => {
            assert!(l.elements.len() >= 3);
            match &l.elements[0] {
                SExp::Symbol(s) => assert_eq!(s.value, "Crate"),
                _ => panic!("Expected Symbol"),
            }
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_preserves_position() {
    let sexp = Parser::parse_str("foo").unwrap();
    match sexp {
        SExp::Symbol(s) => {
            assert_eq!(s.pos.line, 1);
            assert_eq!(s.pos.column, 1);
        }
        _ => panic!("Expected Symbol"),
    }
}

#[test]
fn test_parse_multiline() {
    let input = "foo\nbar";
    let sexp = Parser::parse_str(input).unwrap();
    match sexp {
        SExp::Symbol(s) => assert_eq!(s.value, "foo"),
        _ => panic!("Expected Symbol (should parse first element)"),
    }
}

#[test]
fn test_parse_real_world_example() {
    let input = r#"(Fn
      :defaultness Final
      :sig (FnSig
             :header (FnHeader :safety Default :constness NotConst)
             :decl (FnDecl :inputs () :output (Default)))
      :body nil)"#;

    let sexp = Parser::parse_str(input).unwrap();
    match sexp {
        SExp::List(l) => {
            assert!(l.elements.len() > 0);
            match &l.elements[0] {
                SExp::Symbol(s) => assert_eq!(s.value, "Fn"),
                _ => panic!("Expected Symbol"),
            }
        }
        _ => panic!("Expected List"),
    }
}
