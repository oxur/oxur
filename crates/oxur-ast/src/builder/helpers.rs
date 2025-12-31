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
///
/// # Design Note
/// This function enforces strict keyword-value pair syntax.
/// All elements after the node type (index 0) must be keyword-value pairs.
///
/// **Mixed positional/keyword syntax is intentionally NOT supported.**
///
/// # Format
/// ```lisp
/// (NodeType :key1 value1 :key2 value2 ...)
/// ```
///
/// # Errors
/// - Returns error if odd number of elements after node type
/// - Returns error if non-keyword found where keyword expected
/// - Returns error if value is missing after a keyword
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

/// Extract a required field from kwargs map
///
/// This helper reduces boilerplate when extracting required fields from S-expression kwargs.
/// Returns a clear error if the field is missing.
///
/// # Example
/// ```ignore
/// let cond = Box::new(self.build_expr(require_field(&kwargs, "cond", list.pos)?)?);
/// ```
pub fn require_field<'a>(
    kwargs: &'a std::collections::HashMap<String, &'a SExp>,
    field_name: &str,
    pos: crate::Position,
) -> Result<&'a SExp> {
    kwargs.get(field_name).copied().ok_or_else(|| ParseError::Expected {
        expected: format!(":{} field", field_name),
        found: "missing".to_string(),
        pos,
    })
}

/// Extract an optional field from kwargs map, handling nil values
///
/// This helper reduces boilerplate when extracting optional fields.
/// Returns None if the field is missing or contains nil.
///
/// # Example
/// ```ignore
/// let label = optional_field(&kwargs, "label");
/// ```
pub fn optional_field<'a>(
    kwargs: &'a std::collections::HashMap<String, &'a SExp>,
    field_name: &str,
) -> Option<&'a SExp> {
    kwargs.get(field_name).copied().filter(|sexp| !is_nil(sexp))
}

/// Macro to generate simple enum parsers from S-expressions
///
/// This macro eliminates the boilerplate of parsing enums where each variant
/// is represented by a symbol with the same name as the variant.
///
/// # Example
/// ```ignore
/// build_enum_parser!(build_binop, BinOp, [Add, Sub, Mul, Div]);
/// ```
///
/// Generates a function that parses "Add" → BinOp::Add, etc.
#[macro_export]
macro_rules! build_enum_parser {
    ($fn_name:ident, $enum_type:ty, [$($variant:ident),+ $(,)?]) => {
        pub fn $fn_name(sexp: &$crate::sexp::SExp) -> $crate::error::Result<$enum_type> {
            let sym = $crate::builder::helpers::expect_symbol(sexp)?;
            match sym.value.as_str() {
                $(stringify!($variant) => Ok(<$enum_type>::$variant),)+
                _ => Err($crate::error::ParseError::Expected {
                    expected: format!("{} variant", stringify!($enum_type)),
                    found: sym.value.clone(),
                    pos: sym.pos,
                }),
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sexp::{Number, StringLit};
    use crate::Position;

    fn dummy_pos() -> Position {
        Position { line: 1, column: 1, offset: 0 }
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
        let s = SExp::String(StringLit { value: "hello".to_string(), pos: dummy_pos() });
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
            elements: vec![SExp::Symbol(Symbol { value: "Node".to_string(), pos: dummy_pos() })],
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

    #[test]
    fn test_require_field_success() {
        let mut map = std::collections::HashMap::new();
        let value = SExp::Number(Number { value: "42".to_string(), pos: dummy_pos() });
        map.insert("id".to_string(), &value);

        let result = require_field(&map, "id", dummy_pos());
        assert!(result.is_ok());
    }

    #[test]
    fn test_require_field_missing() {
        let map = std::collections::HashMap::new();
        let result = require_field(&map, "id", dummy_pos());
        assert!(result.is_err());

        if let Err(ParseError::Expected { expected, found, .. }) = result {
            assert_eq!(expected, ":id field");
            assert_eq!(found, "missing");
        } else {
            panic!("Expected ParseError::Expected");
        }
    }

    #[test]
    fn test_optional_field_present() {
        let mut map = std::collections::HashMap::new();
        let value = SExp::Number(Number { value: "42".to_string(), pos: dummy_pos() });
        map.insert("id".to_string(), &value);

        let result = optional_field(&map, "id");
        assert!(result.is_some());
    }

    #[test]
    fn test_optional_field_missing() {
        let map = std::collections::HashMap::new();
        let result = optional_field(&map, "id");
        assert!(result.is_none());
    }

    #[test]
    fn test_optional_field_nil() {
        use crate::sexp::Nil;
        let mut map = std::collections::HashMap::new();
        let nil_value = SExp::Nil(Nil { pos: dummy_pos() });
        map.insert("id".to_string(), &nil_value);

        let result = optional_field(&map, "id");
        assert!(result.is_none(), "Should return None for nil values");
    }
}
