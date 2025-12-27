use oxur_ast::error::Position;
use oxur_ast::sexp::{
    print_sexp, Keyword, List, Nil, Number, Parser, Printer, SExp, StringLit, Symbol,
};

#[test]
fn test_print_symbol() {
    let sym = Symbol::new("foo", Position::new(0, 1, 1));
    let sexp = SExp::Symbol(sym);
    assert_eq!(print_sexp(&sexp), "foo");
}

#[test]
fn test_print_keyword() {
    let kw = Keyword::new("name", Position::new(0, 1, 1));
    let sexp = SExp::Keyword(kw);
    assert_eq!(print_sexp(&sexp), ":name");
}

#[test]
fn test_print_string() {
    let s = StringLit::new("hello", Position::new(0, 1, 1));
    let sexp = SExp::String(s);
    assert_eq!(print_sexp(&sexp), r#""hello""#);
}

#[test]
fn test_print_string_with_escapes() {
    let s = StringLit::new("hello\nworld", Position::new(0, 1, 1));
    let sexp = SExp::String(s);
    assert_eq!(print_sexp(&sexp), r#""hello\nworld""#);

    let s = StringLit::new("tab\there", Position::new(0, 1, 1));
    let sexp = SExp::String(s);
    assert_eq!(print_sexp(&sexp), r#""tab\there""#);

    let s = StringLit::new("return\rhere", Position::new(0, 1, 1));
    let sexp = SExp::String(s);
    assert_eq!(print_sexp(&sexp), r#""return\rhere""#);

    let s = StringLit::new("back\\slash", Position::new(0, 1, 1));
    let sexp = SExp::String(s);
    assert_eq!(print_sexp(&sexp), r#""back\\slash""#);

    let s = StringLit::new(r#"quote"here"#, Position::new(0, 1, 1));
    let sexp = SExp::String(s);
    assert_eq!(print_sexp(&sexp), r#""quote\"here""#);
}

#[test]
fn test_print_number() {
    let n = Number::new("42", Position::new(0, 1, 1));
    let sexp = SExp::Number(n);
    assert_eq!(print_sexp(&sexp), "42");
}

#[test]
fn test_print_negative_number() {
    let n = Number::new("-42", Position::new(0, 1, 1));
    let sexp = SExp::Number(n);
    assert_eq!(print_sexp(&sexp), "-42");
}

#[test]
fn test_print_nil() {
    let nil = Nil::new(Position::new(0, 1, 1));
    let sexp = SExp::Nil(nil);
    assert_eq!(print_sexp(&sexp), "nil");
}

#[test]
fn test_print_empty_list() {
    let list = List::new(vec![], Position::new(0, 1, 1));
    let sexp = SExp::List(list);
    assert_eq!(print_sexp(&sexp), "()");
}

#[test]
fn test_print_simple_list() {
    let elements = vec![
        SExp::Symbol(Symbol::new("foo", Position::new(0, 1, 1))),
        SExp::Symbol(Symbol::new("bar", Position::new(0, 1, 1))),
    ];
    let list = List::new(elements, Position::new(0, 1, 1));
    let sexp = SExp::List(list);
    assert_eq!(print_sexp(&sexp), "(foo bar)");
}

#[test]
fn test_print_list_with_three_elements() {
    let elements = vec![
        SExp::Symbol(Symbol::new("a", Position::new(0, 1, 1))),
        SExp::Symbol(Symbol::new("b", Position::new(0, 1, 1))),
        SExp::Symbol(Symbol::new("c", Position::new(0, 1, 1))),
    ];
    let list = List::new(elements, Position::new(0, 1, 1));
    let sexp = SExp::List(list);
    let output = print_sexp(&sexp);
    assert!(output.contains('a'));
    assert!(output.contains('b'));
    assert!(output.contains('c'));
}

#[test]
fn test_print_nested_list() {
    let inner = vec![
        SExp::Symbol(Symbol::new("bar", Position::new(0, 1, 1))),
        SExp::Symbol(Symbol::new("baz", Position::new(0, 1, 1))),
    ];
    let elements = vec![
        SExp::Symbol(Symbol::new("foo", Position::new(0, 1, 1))),
        SExp::List(List::new(inner, Position::new(0, 1, 1))),
    ];
    let list = List::new(elements, Position::new(0, 1, 1));
    let sexp = SExp::List(list);
    let output = print_sexp(&sexp);
    assert!(output.contains("foo"));
    assert!(output.contains("bar"));
    assert!(output.contains("baz"));
}

#[test]
fn test_print_with_custom_indent() {
    let printer = Printer::with_indent(4);
    let elements = vec![
        SExp::Symbol(Symbol::new("foo", Position::new(0, 1, 1))),
        SExp::Symbol(Symbol::new("bar", Position::new(0, 1, 1))),
    ];
    let list = List::new(elements, Position::new(0, 1, 1));
    let sexp = SExp::List(list);
    let output = printer.print(&sexp);
    assert!(output.contains("foo"));
}

#[test]
fn test_round_trip_symbol() {
    round_trip("foo");
}

#[test]
fn test_round_trip_keyword() {
    round_trip(":name");
}

#[test]
fn test_round_trip_string() {
    round_trip(r#""hello""#);
}

#[test]
fn test_round_trip_string_with_escapes() {
    let parsed = Parser::parse_str(r#""hello\nworld""#).unwrap();
    let printed = print_sexp(&parsed);
    let reparsed = Parser::parse_str(&printed).unwrap();
    assert_eq!(parsed, reparsed);
}

#[test]
fn test_round_trip_number() {
    round_trip("42");
    round_trip("-42");
    round_trip("0");
}

#[test]
fn test_round_trip_nil() {
    round_trip("nil");
}

#[test]
fn test_round_trip_empty_list() {
    round_trip("()");
}

#[test]
fn test_round_trip_simple_list() {
    round_trip("(foo bar)");
}

#[test]
fn test_round_trip_nested_list() {
    let input = "(foo (bar baz))";
    let parsed = Parser::parse_str(input).unwrap();
    let printed = print_sexp(&parsed);
    let reparsed = Parser::parse_str(&printed).unwrap();
    // Positions will differ due to formatting, but structure should be same
    assert_sexp_structure_eq(&parsed, &reparsed);
}

#[test]
fn test_round_trip_complex_structure() {
    let input = r#"(Crate :attrs () :items ())"#;
    let parsed = Parser::parse_str(input).unwrap();
    let printed = print_sexp(&parsed);
    let reparsed = Parser::parse_str(&printed).unwrap();
    assert_sexp_structure_eq(&parsed, &reparsed);
}

#[test]
fn test_round_trip_with_all_types() {
    let input = r#"(foo 42 "hello" :key nil ())"#;
    let parsed = Parser::parse_str(input).unwrap();
    let printed = print_sexp(&parsed);
    let reparsed = Parser::parse_str(&printed).unwrap();
    assert_sexp_structure_eq(&parsed, &reparsed);
}

// Helper to compare S-expression structure ignoring positions
fn assert_sexp_structure_eq(left: &SExp, right: &SExp) {
    match (left, right) {
        (SExp::Symbol(l), SExp::Symbol(r)) => assert_eq!(l.value, r.value),
        (SExp::Keyword(l), SExp::Keyword(r)) => assert_eq!(l.name, r.name),
        (SExp::String(l), SExp::String(r)) => assert_eq!(l.value, r.value),
        (SExp::Number(l), SExp::Number(r)) => assert_eq!(l.value, r.value),
        (SExp::Nil(_), SExp::Nil(_)) => (),
        (SExp::List(l), SExp::List(r)) => {
            assert_eq!(l.elements.len(), r.elements.len());
            for (le, re) in l.elements.iter().zip(r.elements.iter()) {
                assert_sexp_structure_eq(le, re);
            }
        }
        _ => panic!("Mismatched S-expression types"),
    }
}

fn round_trip(input: &str) {
    let parsed = Parser::parse_str(input).unwrap();
    let printed = print_sexp(&parsed);
    let reparsed = Parser::parse_str(&printed).unwrap();
    assert_eq!(parsed, reparsed);
}
