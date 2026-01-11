mod expr;
mod gen;
mod generics;
mod helpers;
mod item;
mod stmt;

pub use gen::Generator;
pub use helpers::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sexp::print_sexp;

    #[test]
    fn test_helpers() {
        let node =
            typed_node("Test", kwargs(vec![kwarg("name", string("foo")), kwarg("id", num(42))]));

        let output = print_sexp(&node);
        assert!(output.contains("Test"));
        assert!(output.contains(":name"));
        assert!(output.contains("foo"));
    }

    #[test]
    fn test_helpers_empty_list() {
        let node = empty_list();
        assert!(matches!(node, crate::sexp::SExp::List(_)));
    }

    #[test]
    fn test_helpers_list() {
        let node = list(vec![sym("a"), sym("b")]);
        let output = print_sexp(&node);
        assert!(output.contains("a"));
        assert!(output.contains("b"));
    }

    #[test]
    fn test_helpers_sym() {
        let node = sym("test_symbol");
        let output = print_sexp(&node);
        assert!(output.contains("test_symbol"));
    }

    #[test]
    fn test_helpers_kw() {
        let node = kw("test_keyword");
        let output = print_sexp(&node);
        assert!(output.contains(":test_keyword"));
    }

    #[test]
    fn test_helpers_string() {
        let node = string("test string");
        let output = print_sexp(&node);
        assert!(output.contains("test string"));
    }

    #[test]
    fn test_helpers_num() {
        let node = num(42);
        let output = print_sexp(&node);
        assert!(output.contains("42"));
    }

    #[test]
    fn test_helpers_num_negative() {
        let node = num(-123);
        let output = print_sexp(&node);
        assert!(output.contains("-123"));
    }

    #[test]
    fn test_helpers_kwarg() {
        let args = kwarg("key", sym("value"));
        assert!(args.len() == 2);
    }

    #[test]
    fn test_helpers_kwargs() {
        let args = kwargs(vec![kwarg("a", num(1)), kwarg("b", num(2))]);
        assert!(args.len() == 4);
    }

    #[test]
    fn test_helpers_typed_node() {
        let node = typed_node("MyNode", kwargs(vec![kwarg("field", string("value"))]));
        let output = print_sexp(&node);
        assert!(output.contains("MyNode"));
        assert!(output.contains(":field"));
        assert!(output.contains("value"));
    }
}
