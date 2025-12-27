use oxur_ast::error::Position;
use oxur_ast::sexp::{HasPosition, Keyword, List, Nil, Number, SExp, StringLit, Symbol};

#[test]
fn test_symbol_new() {
    let pos = Position::new(0, 1, 1);
    let sym = Symbol::new("foo", pos);
    assert_eq!(sym.value, "foo");
    assert_eq!(sym.pos, pos);
}

#[test]
fn test_symbol_new_from_string() {
    let sym = Symbol::new(String::from("bar"), Position::new(0, 1, 1));
    assert_eq!(sym.value, "bar");
}

#[test]
fn test_keyword_new() {
    let pos = Position::new(0, 1, 1);
    let kw = Keyword::new("name", pos);
    assert_eq!(kw.name, "name");
    assert_eq!(kw.pos, pos);
}

#[test]
fn test_keyword_new_from_string() {
    let kw = Keyword::new(String::from("kind"), Position::new(0, 1, 1));
    assert_eq!(kw.name, "kind");
}

#[test]
fn test_string_lit_new() {
    let pos = Position::new(0, 1, 1);
    let s = StringLit::new("hello", pos);
    assert_eq!(s.value, "hello");
    assert_eq!(s.pos, pos);
}

#[test]
fn test_string_lit_new_from_string() {
    let s = StringLit::new(String::from("world"), Position::new(0, 1, 1));
    assert_eq!(s.value, "world");
}

#[test]
fn test_number_new() {
    let pos = Position::new(0, 1, 1);
    let n = Number::new("42", pos);
    assert_eq!(n.value, "42");
    assert_eq!(n.pos, pos);
}

#[test]
fn test_number_new_from_string() {
    let n = Number::new(String::from("123"), Position::new(0, 1, 1));
    assert_eq!(n.value, "123");
}

#[test]
fn test_nil_new() {
    let pos = Position::new(0, 1, 1);
    let nil = Nil::new(pos);
    assert_eq!(nil.pos, pos);
}

#[test]
fn test_list_new() {
    let pos = Position::new(0, 1, 1);
    let elements =
        vec![SExp::Symbol(Symbol::new("foo", pos)), SExp::Number(Number::new("42", pos))];
    let list = List::new(elements.clone(), pos);
    assert_eq!(list.elements.len(), 2);
    assert_eq!(list.pos, pos);
}

#[test]
fn test_list_empty() {
    let pos = Position::new(0, 1, 1);
    let list = List::new(vec![], pos);
    assert_eq!(list.elements.len(), 0);
}

#[test]
fn test_has_position_symbol() {
    let pos = Position::new(5, 2, 3);
    let sym = Symbol::new("test", pos);
    let sexp = SExp::Symbol(sym);
    assert_eq!(sexp.position(), pos);
}

#[test]
fn test_has_position_keyword() {
    let pos = Position::new(10, 3, 4);
    let kw = Keyword::new("key", pos);
    let sexp = SExp::Keyword(kw);
    assert_eq!(sexp.position(), pos);
}

#[test]
fn test_has_position_string() {
    let pos = Position::new(15, 4, 5);
    let s = StringLit::new("test", pos);
    let sexp = SExp::String(s);
    assert_eq!(sexp.position(), pos);
}

#[test]
fn test_has_position_number() {
    let pos = Position::new(20, 5, 6);
    let n = Number::new("99", pos);
    let sexp = SExp::Number(n);
    assert_eq!(sexp.position(), pos);
}

#[test]
fn test_has_position_nil() {
    let pos = Position::new(25, 6, 7);
    let nil = Nil::new(pos);
    let sexp = SExp::Nil(nil);
    assert_eq!(sexp.position(), pos);
}

#[test]
fn test_has_position_list() {
    let pos = Position::new(30, 7, 8);
    let list = List::new(vec![], pos);
    let sexp = SExp::List(list);
    assert_eq!(sexp.position(), pos);
}

#[test]
fn test_sexp_clone() {
    let pos = Position::new(0, 1, 1);
    let sym = Symbol::new("test", pos);
    let sexp = SExp::Symbol(sym);
    let cloned = sexp.clone();
    assert_eq!(sexp, cloned);
}

#[test]
fn test_sexp_partial_eq() {
    let pos1 = Position::new(0, 1, 1);
    let pos2 = Position::new(0, 1, 1);
    let sym1 = Symbol::new("test", pos1);
    let sym2 = Symbol::new("test", pos2);
    let sexp1 = SExp::Symbol(sym1);
    let sexp2 = SExp::Symbol(sym2);
    assert_eq!(sexp1, sexp2);
}

#[test]
fn test_sexp_not_equal_different_values() {
    let pos = Position::new(0, 1, 1);
    let sym1 = Symbol::new("foo", pos);
    let sym2 = Symbol::new("bar", pos);
    let sexp1 = SExp::Symbol(sym1);
    let sexp2 = SExp::Symbol(sym2);
    assert_ne!(sexp1, sexp2);
}

#[test]
fn test_sexp_not_equal_different_types() {
    let pos = Position::new(0, 1, 1);
    let sym = Symbol::new("test", pos);
    let kw = Keyword::new("test", pos);
    let sexp1 = SExp::Symbol(sym);
    let sexp2 = SExp::Keyword(kw);
    assert_ne!(sexp1, sexp2);
}
