use crate::error::{ParseError, Result};
use crate::sexp::{Keyword, List, SExp, Symbol};

/// Extract a symbol from an S-expression
pub fn expect_symbol(sexp: &SExp) -> Result<&Symbol> {
    match sexp {
        SExp::Symbol(s) => Ok(s),
        _ => Err(ParseError::Expected {
            expected: "symbol".to_string(),
            found: format!("{:?}", sexp),
            pos: sexp.position(),
        }),
    }
}

/// Extract a keyword from an S-expression
pub fn expect_keyword(sexp: &SExp) -> Result<&Keyword> {
    match sexp {
        SExp::Keyword(k) => Ok(k),
        _ => Err(ParseError::Expected {
            expected: "keyword".to_string(),
            found: format!("{:?}", sexp),
            pos: sexp.position(),
        }),
    }
}

/// Extract a string from an S-expression
pub fn expect_string(sexp: &SExp) -> Result<String> {
    match sexp {
        SExp::String(s) => Ok(s.value.clone()),
        _ => Err(ParseError::Expected {
            expected: "string".to_string(),
            found: format!("{:?}", sexp),
            pos: sexp.position(),
        }),
    }
}

/// Extract a number from an S-expression
pub fn expect_number(sexp: &SExp) -> Result<i128> {
    match sexp {
        SExp::Number(n) => n.value.parse().map_err(|_| ParseError::Expected {
            expected: "valid number".to_string(),
            found: n.value.clone(),
            pos: n.pos,
        }),
        _ => Err(ParseError::Expected {
            expected: "number".to_string(),
            found: format!("{:?}", sexp),
            pos: sexp.position(),
        }),
    }
}

/// Extract a list from an S-expression
pub fn expect_list(sexp: &SExp) -> Result<&List> {
    match sexp {
        SExp::List(l) => Ok(l),
        _ => Err(ParseError::Expected {
            expected: "list".to_string(),
            found: format!("{:?}", sexp),
            pos: sexp.position(),
        }),
    }
}

/// Check if an S-expression is nil
pub fn is_nil(sexp: &SExp) -> bool {
    matches!(sexp, SExp::Nil(_))
}

/// Parse keyword-value pairs from a list
/// Returns a map of keyword name -> value
pub fn parse_kwargs(list: &List) -> Result<std::collections::HashMap<String, &SExp>> {
    let mut map = std::collections::HashMap::new();
    let mut i = 1; // Skip first element (node type)

    while i < list.elements.len() {
        if i + 1 >= list.elements.len() {
            return Err(ParseError::Expected {
                expected: "value after keyword".to_string(),
                found: "end of list".to_string(),
                pos: list.pos,
            });
        }

        let key = expect_keyword(&list.elements[i])?;
        let value = &list.elements[i + 1];

        map.insert(key.name.clone(), value);
        i += 2;
    }

    Ok(map)
}

use crate::sexp::HasPosition;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sexp::{Nil, Number, StringLit};
    use crate::Position;

    fn dummy_pos() -> Position {
        Position { line: 1, column: 1 }
    }

    #[test]
    fn test_expect_symbol_success() {
        let sym = SExp::Symbol(Symbol { value: "test".to_string(), pos: dummy_pos() });
        let result = expect_symbol(&sym);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, "test");
    }

    #[test]
    fn test_expect_symbol_failure() {
        let num = SExp::Number(Number { value: "42".to_string(), pos: dummy_pos() });
        let result = expect_symbol(&num);
        assert!(result.is_err());
    }

    #[test]
    fn test_expect_keyword_success() {
        let kw = SExp::Keyword(Keyword { name: "test".to_string(), pos: dummy_pos() });
        let result = expect_keyword(&kw);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "test");
    }

    #[test]
    fn test_expect_keyword_failure() {
        let sym = SExp::Symbol(Symbol { value: "test".to_string(), pos: dummy_pos() });
        let result = expect_keyword(&sym);
        assert!(result.is_err());
    }

    #[test]
    fn test_expect_string_success() {
        let s = SExp::String(StringLit {
            value: "hello".to_string(),
            pos: dummy_pos(),
        });
        let result = expect_string(&s);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_expect_string_failure() {
        let num = SExp::Number(Number { value: "42".to_string(), pos: dummy_pos() });
        let result = expect_string(&num);
        assert!(result.is_err());
    }

    #[test]
    fn test_expect_number_success() {
        let num = SExp::Number(Number { value: "42".to_string(), pos: dummy_pos() });
        let result = expect_number(&num);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_expect_number_negative() {
        let num = SExp::Number(Number { value: "-100".to_string(), pos: dummy_pos() });
        let result = expect_number(&num);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), -100);
    }

    #[test]
    fn test_expect_number_invalid() {
        let num = SExp::Number(Number { value: "not_a_number".to_string(), pos: dummy_pos() });
        let result = expect_number(&num);
        assert!(result.is_err());
    }

    #[test]
    fn test_expect_number_failure() {
        let sym = SExp::Symbol(Symbol { value: "test".to_string(), pos: dummy_pos() });
        let result = expect_number(&sym);
        assert!(result.is_err());
    }

    #[test]
    fn test_expect_list_success() {
        let list = SExp::List(List { elements: vec![], pos: dummy_pos() });
        let result = expect_list(&list);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().elements.len(), 0);
    }

    #[test]
    fn test_expect_list_failure() {
        let sym = SExp::Symbol(Symbol { value: "test".to_string(), pos: dummy_pos() });
        let result = expect_list(&sym);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_nil_true() {
        use crate::sexp::Nil;
        let nil = SExp::Nil(Nil { pos: dummy_pos() });
        assert!(is_nil(&nil));
    }

    #[test]
    fn test_is_nil_false() {
        let sym = SExp::Symbol(Symbol { value: "test".to_string(), pos: dummy_pos() });
        assert!(!is_nil(&sym));
    }

    #[test]
    fn test_parse_kwargs_success() {
        let list = List {
            elements: vec![
                SExp::Symbol(Symbol { value: "Node".to_string(), pos: dummy_pos() }),
                SExp::Keyword(Keyword { name: "id".to_string(), pos: dummy_pos() }),
                SExp::Number(Number { value: "42".to_string(), pos: dummy_pos() }),
                SExp::Keyword(Keyword { name: "name".to_string(), pos: dummy_pos() }),
                SExp::String(StringLit { value: "test".to_string(), pos: dummy_pos() }),
            ],
            pos: dummy_pos(),
        };

        let result = parse_kwargs(&list);
        assert!(result.is_ok());

        let map = result.unwrap();
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("id"));
        assert!(map.contains_key("name"));
    }

    #[test]
    fn test_parse_kwargs_empty() {
        let list = List {
            elements: vec![SExp::Symbol(Symbol {
                value: "Node".to_string(),
                pos: dummy_pos(),
            })],
            pos: dummy_pos(),
        };

        let result = parse_kwargs(&list);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_parse_kwargs_missing_value() {
        let list = List {
            elements: vec![
                SExp::Symbol(Symbol { value: "Node".to_string(), pos: dummy_pos() }),
                SExp::Keyword(Keyword { name: "id".to_string(), pos: dummy_pos() }),
                // Missing value after keyword
            ],
            pos: dummy_pos(),
        };

        let result = parse_kwargs(&list);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_kwargs_odd_number() {
        let list = List {
            elements: vec![
                SExp::Symbol(Symbol { value: "Node".to_string(), pos: dummy_pos() }),
                SExp::Keyword(Keyword { name: "id".to_string(), pos: dummy_pos() }),
                SExp::Number(Number { value: "42".to_string(), pos: dummy_pos() }),
                SExp::Keyword(Keyword { name: "orphan".to_string(), pos: dummy_pos() }),
                // This keyword has no value - should error
            ],
            pos: dummy_pos(),
        };

        let result = parse_kwargs(&list);
        assert!(result.is_err());
    }
}
