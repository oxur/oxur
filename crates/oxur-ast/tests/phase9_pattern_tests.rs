use oxur_ast::ast::*;
use oxur_ast::integration::parse_rust_file;

/// Phase 9: Test wildcard pattern (CRITICAL)
#[test]
fn test_wildcard_pattern() {
    let code = r#"
        fn ignore_value(_: i32) {}
        fn multi_ignore(_: i32, _: String) {}
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse wildcard pattern: {:?}", result.err());
}

/// Phase 9: Test tuple patterns
#[test]
fn test_tuple_pattern() {
    let code = r#"
        fn main() {
            let (x, y, z) = (1, 2, 3);
            let (a, b) = (10, 20);
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse tuple pattern: {:?}", result.err());
}

/// Phase 9: Test struct patterns
#[test]
fn test_struct_pattern() {
    let code = r#"
        struct Point { x: i32, y: i32 }

        fn main() {
            let Point { x, y } = Point { x: 1, y: 2 };
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse struct pattern: {:?}", result.err());
}

/// Phase 9: Test tuple struct patterns (Option/Result)
#[test]
fn test_tuple_struct_pattern() {
    let code = r#"
        fn get_option() -> Option<i32> {
            Some(42)
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse code with Option: {:?}", result.err());
}

/// Phase 9: Test reference patterns
#[test]
fn test_reference_pattern() {
    let code = r#"
        fn borrow_value(x: &i32) {}
        fn mut_borrow_value(x: &mut i32) {}
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse reference patterns: {:?}", result.err());
}

/// Phase 9: Test slice patterns
#[test]
fn test_slice_pattern() {
    let code = r#"
        fn takes_slice(arr: &[i32]) {}
        fn takes_mut_slice(arr: &mut [String]) {}
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse slice patterns: {:?}", result.err());
}

/// Phase 9: Integration test - struct with method patterns
#[test]
fn test_struct_with_method_patterns() {
    let code = r#"
        struct Calculator {
            value: i32,
        }

        impl Calculator {
            fn new() -> Self {
                Calculator { value: 0 }
            }

            fn process(&self, _: i32) -> i32 {
                self.value
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse struct with method patterns: {:?}", result.err());
}

/// Phase 9: Integration test - tuple destructuring
#[test]
fn test_tuple_destructuring() {
    let code = r#"
        fn process_data() {
            let coords = (10, 20, 30);
            let (x, y, z) = coords;
            
            let nested = ((1, 2), (3, 4));
            let ((a, b), (c, d)) = nested;
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse tuple destructuring: {:?}", result.err());
}

/// Phase 9: Integration test - struct pattern with multiple fields
#[test]
fn test_complex_struct_pattern() {
    let code = r#"
        struct Person {
            name: String,
            age: i32,
            city: String,
        }

        fn main() {
            let p = Person {
                name: "Alice".to_string(),
                age: 30,
                city: "NYC".to_string(),
            };
            
            let Person { name, age, city } = p;
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse complex struct pattern: {:?}", result.err());
}

/// Phase 9: Test mixed patterns
#[test]
fn test_mixed_patterns() {
    let code = r#"
        struct Data {
            values: (i32, i32),
        }

        fn process(data: Data) {
            let Data { values: (x, y) } = data;
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse mixed patterns: {:?}", result.err());
}
