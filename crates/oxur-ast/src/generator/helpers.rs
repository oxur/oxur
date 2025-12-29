use crate::error::Position;
use crate::sexp::{Keyword, List, Number, SExp, StringLit, Symbol};

/// Create a symbol S-expression
pub fn sym(name: impl Into<String>) -> SExp {
    SExp::Symbol(Symbol::new(name, Position::new(0, 1, 1)))
}

/// Create a keyword S-expression
pub fn kw(name: impl Into<String>) -> SExp {
    SExp::Keyword(Keyword::new(name, Position::new(0, 1, 1)))
}

/// Create a string S-expression
pub fn string(value: impl Into<String>) -> SExp {
    SExp::String(StringLit::new(value, Position::new(0, 1, 1)))
}

/// Create a number S-expression
pub fn num(value: impl ToString) -> SExp {
    SExp::Number(Number::new(value.to_string(), Position::new(0, 1, 1)))
}

/// Create a list S-expression
pub fn list(elements: Vec<SExp>) -> SExp {
    SExp::List(List::new(elements, Position::new(0, 1, 1)))
}

/// Create an empty list
pub fn empty_list() -> SExp {
    list(vec![])
}

/// Create a keyword-value pair
pub fn kwarg(key: &str, value: SExp) -> Vec<SExp> {
    vec![kw(key), value]
}

/// Create a typed node: (Type :field1 val1 :field2 val2 ...)
pub fn typed_node(type_name: &str, fields: Vec<SExp>) -> SExp {
    let mut elements = vec![sym(type_name)];
    elements.extend(fields);
    list(elements)
}

/// Flatten multiple kwarg pairs into a single vec
pub fn kwargs(pairs: Vec<Vec<SExp>>) -> Vec<SExp> {
    pairs.into_iter().flatten().collect()
}
