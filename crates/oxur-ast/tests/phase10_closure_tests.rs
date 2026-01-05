use oxur_ast::integration::parse_rust_file;

/// Phase 10: Test simple closure
#[test]
fn test_simple_closure() {
    let code = r#"
        fn main() {
            let double = |x| x * 2;
            let result = double(5);
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse simple closure: {:?}", result.err());
}

/// Phase 10: Test closure with multiple parameters
#[test]
fn test_closure_multiple_params() {
    let code = r#"
        fn main() {
            let add = |a, b| a + b;
            let sum = add(3, 4);
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse closure with multiple params: {:?}", result.err());
}

/// Phase 10: Test closure with type annotations
#[test]
fn test_closure_with_types() {
    let code = r#"
        fn main() {
            let parse = |s: &str| s.len();
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse closure with types: {:?}", result.err());
}

/// Phase 10: Test closure with block body
#[test]
fn test_closure_block_body() {
    let code = r#"
        fn main() {
            let process = |x| {
                let doubled = x * 2;
                doubled + 1
            };
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse closure with block body: {:?}", result.err());
}

/// Phase 10: Test closure in iterator chain (CRITICAL)
#[test]
fn test_closure_in_iterator() {
    let code = r#"
        fn main() {
            let numbers = vec![1, 2, 3, 4, 5];
            let doubled: Vec<_> = numbers
                .iter()
                .map(|x| x * 2)
                .collect();
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse closure in iterator: {:?}", result.err());
}

/// Phase 10: Test nested closures in iterator chain
#[test]
fn test_nested_closures() {
    let code = r#"
        fn process_data(items: Vec<i32>) -> Vec<i32> {
            items
                .iter()
                .filter(|x| **x > 0)
                .map(|x| x * 2)
                .collect()
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse nested closures: {:?}", result.err());
}

/// Phase 10: Test closure as function parameter
#[test]
fn test_closure_as_parameter() {
    let code = r#"
        fn apply<F>(f: F, x: i32) -> i32
        where
            F: Fn(i32) -> i32,
        {
            f(x)
        }

        fn main() {
            let result = apply(|x| x * 2, 5);
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse closure as parameter: {:?}", result.err());
}

/// Phase 10: Test closure capturing variables
#[test]
fn test_closure_capture() {
    let code = r#"
        fn main() {
            let multiplier = 2;
            let multiply = |x| x * multiplier;
            let result = multiply(5);
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse closure with capture: {:?}", result.err());
}

/// Phase 10: Test multiple closures
#[test]
fn test_multiple_closures() {
    let code = r#"
        fn main() {
            let add = |a, b| a + b;
            let sub = |a, b| a - b;
            let mul = |a, b| a * b;
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse multiple closures: {:?}", result.err());
}

/// Phase 10: Integration test - real iterator patterns
#[test]
fn test_real_iterator_patterns() {
    let code = r#"
        fn process_numbers(nums: Vec<i32>) -> Vec<i32> {
            nums.into_iter()
                .filter(|n| *n > 0)
                .map(|n| n * 2)
                .filter(|n| *n < 100)
                .collect()
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse real iterator patterns: {:?}", result.err());
}
