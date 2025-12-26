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
