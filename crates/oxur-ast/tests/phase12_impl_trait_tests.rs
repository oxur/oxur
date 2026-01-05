use oxur_ast::integration::parse_rust_file;

/// Phase 12: Test impl trait in return position
#[test]
fn test_impl_trait_return() {
    let code = r#"
        fn numbers() -> impl Iterator<Item = i32> {
            vec![1, 2, 3].into_iter()
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse impl trait return: {:?}", result.err());
}

/// Phase 12: Test impl trait in parameter position
#[test]
fn test_impl_trait_parameter() {
    let code = r#"
        fn process(iter: impl Iterator<Item = i32>) {
            for item in iter {
                println!("{}", item);
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse impl trait parameter: {:?}", result.err());
}

/// Phase 12: Test impl trait with multiple bounds
#[test]
fn test_impl_trait_multiple_bounds() {
    let code = r#"
        fn complex() -> impl Display + Debug + Send {
            42
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse impl trait with multiple bounds: {:?}", result.err());
}

/// Phase 12: Test impl trait with associated types
#[test]
fn test_impl_trait_associated_type() {
    let code = r#"
        fn make_iter() -> impl Iterator<Item = String> {
            vec!["hello".to_string()].into_iter()
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse impl trait with associated type: {:?}", result.err());
}

/// Phase 12: Test impl trait in both positions
#[test]
fn test_impl_trait_both_positions() {
    let code = r#"
        fn transform(input: impl Iterator<Item = i32>) -> impl Iterator<Item = String> {
            input.map(|x| x.to_string())
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse impl trait in both positions: {:?}", result.err());
}

/// Phase 12: Test impl trait with lifetime bounds
#[test]
fn test_impl_trait_with_lifetime() {
    let code = r#"
        fn make_ref(s: &str) -> impl Fn() -> &str {
            move || s
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse impl trait with lifetime: {:?}", result.err());
}

/// Phase 12: Test impl trait with Clone bound
#[test]
fn test_impl_trait_clone() {
    let code = r#"
        fn make_cloneable() -> impl Clone {
            42
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse impl trait with Clone: {:?}", result.err());
}

/// Phase 12: Test impl trait with nested generics
#[test]
fn test_impl_trait_nested_generics() {
    let code = r#"
        fn nested() -> impl Iterator<Item = Vec<i32>> {
            vec![vec![1, 2], vec![3, 4]].into_iter()
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse impl trait with nested generics: {:?}", result.err());
}

/// Phase 12: Test multiple impl trait parameters
#[test]
fn test_multiple_impl_trait_params() {
    let code = r#"
        fn compare(a: impl PartialEq, b: impl PartialEq) -> bool {
            a == b
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse multiple impl trait params: {:?}", result.err());
}

/// Phase 12: Integration test - real iterator patterns
#[test]
fn test_real_iterator_impl_trait() {
    let code = r#"
        fn filter_positive(nums: Vec<i32>) -> impl Iterator<Item = i32> {
            nums.into_iter().filter(|&x| x > 0)
        }

        fn map_to_string(nums: impl Iterator<Item = i32>) -> impl Iterator<Item = String> {
            nums.map(|x| x.to_string())
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse real iterator impl trait: {:?}", result.err());
}
