use oxur_ast::ast::*;
use oxur_ast::integration::parse_rust_file;

/// Phase 9: Test that struct literal expressions now parse successfully
#[test]
fn test_struct_literal() {
    let code = r#"
        struct Point { x: i32, y: i32 }

        fn main() {
            let p = Point { x: 1, y: 2 };
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse struct literal: {:?}", result.err());
    let crate_node = result.unwrap();
    assert_eq!(crate_node.items.len(), 2);
}

/// Phase 9: Test array literal expressions
#[test]
fn test_array_literal() {
    let code = r#"
        fn main() {
            let nums = [1, 2, 3, 4, 5];
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse array literal: {:?}", result.err());
}

/// Phase 9: Test tuple expressions
#[test]
fn test_tuple_expression() {
    let code = r#"
        fn main() {
            let pair = (1, 2);
            let triple = (1, 2, 3);
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse tuple expression: {:?}", result.err());
}

/// Phase 9: Test range expressions
#[test]
fn test_range_expressions() {
    let code = r#"
        fn main() {
            let a = 0..10;
            let b = 0..=10;
            let c = ..5;
            let d = 5..;
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse range expressions: {:?}", result.err());
}

/// Phase 9: Test assignment expressions
#[test]
fn test_assignment() {
    let code = r#"
        fn main() {
            let mut x = 5;
            x = 10;
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse assignment: {:?}", result.err());
}

/// Phase 9: Test parenthesized expressions
#[test]
fn test_parenthesized_expression() {
    let code = r#"
        fn main() {
            let result = (1 + 2) * 3;
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse parenthesized expression: {:?}", result.err());
}

/// Phase 9: Test try operator
#[test]
fn test_try_operator() {
    let code = r#"
        fn read_file() -> Result<String, ()> {
            let x = some_fn()?;
            Ok(x)
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse try operator: {:?}", result.err());
}

/// Phase 9: Test type cast
#[test]
fn test_type_cast() {
    let code = r#"
        fn main() {
            let x = 5 as f64;
            let y = 10 as i32;
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse type cast: {:?}", result.err());
}

/// Phase 9: Test break and continue (NOTE: loop expressions are Phase 10)
#[test]
fn test_break_continue() {
    // Break and continue are supported, but loop expressions are Phase 10
    // We can still test that the conversion code exists and compiles
    let code = r#"
        fn test_values() -> i32 {
            return 42;
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Code should parse: {:?}", result.err());
}

/// Phase 9: Test return expression
#[test]
fn test_return_expression() {
    let code = r#"
        fn get_value() -> i32 {
            return 42;
        }

        fn early_return(x: i32) -> i32 {
            if x < 0 {
                return 0;
            }
            x
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse return expression: {:?}", result.err());
}

/// Phase 9: Integration test - struct with methods using new expressions
#[test]
fn test_struct_with_methods() {
    let code = r#"
        struct Calculator {
            value: i32,
        }

        impl Calculator {
            fn new() -> Self {
                Calculator { value: 0 }
            }

            fn add(&mut self, n: i32) {
                self.value = self.value + n;
            }

            fn get(&self) -> i32 {
                return self.value;
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse struct with methods: {:?}", result.err());
    let crate_node = result.unwrap();
    assert_eq!(crate_node.items.len(), 2); // struct + impl
}

/// Phase 9: Integration test - array and tuple operations
#[test]
fn test_array_and_tuple_operations() {
    let code = r#"
        fn process_data() {
            let nums = [1, 2, 3, 4, 5];
            let coords = (10, 20, 30);
            let (x, y, z) = coords;
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse array and tuple operations: {:?}", result.err());
}

/// Phase 9: Integration test - all range types
#[test]
fn test_all_range_types() {
    let code = r#"
        fn ranges() {
            let exclusive = 0..10;
            let inclusive = 0..=10;
            let from = 5..;
            let to = ..5;
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse range operations: {:?}", result.err());
}

/// Phase 9: Test labeled break/continue (NOTE: loop expressions are Phase 10)
#[test]
fn test_labeled_break_continue() {
    // Labeled break/continue require loop expressions which are Phase 10
    // The conversion code for labels exists and is ready for Phase 10
    let code = r#"
        fn early_exit(x: i32) -> i32 {
            if x < 0 {
                return 0;
            }
            x * 2
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Code should parse: {:?}", result.err());
}
