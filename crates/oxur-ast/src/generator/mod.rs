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
}
