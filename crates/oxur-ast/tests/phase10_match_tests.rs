use oxur_ast::ast::*;
use oxur_ast::integration::parse_rust_file;

/// Phase 10: Test basic match expression
#[test]
fn test_basic_match() {
    let code = r#"
        fn main() {
            let x = 5;
            let desc = match x {
                0 => "zero",
                1 => "one",
                _ => "other",
            };
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse basic match: {:?}", result.err());
}

/// Phase 10: Test match with Option
#[test]
fn test_match_with_option() {
    let code = r#"
        fn main() {
            let opt = Some(42);
            let value = match opt {
                Some(x) => x,
                None => 0,
            };
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse match with Option: {:?}", result.err());
}

/// Phase 10: Test match with Result
#[test]
fn test_match_with_result() {
    let code = r#"
        fn handle_result(res: Result<i32, String>) -> i32 {
            match res {
                Ok(value) => value,
                Err(msg) => 0,
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse match with Result: {:?}", result.err());
}

/// Phase 10: Test match with guards
#[test]
fn test_match_with_guards() {
    let code = r#"
        fn classify(number: i32) -> &'static str {
            match number {
                x if x < 0 => "negative",
                x if x > 0 => "positive",
                _ => "zero",
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse match with guards: {:?}", result.err());
}

/// Phase 10: Test match with struct patterns
#[test]
fn test_match_with_struct_patterns() {
    let code = r#"
        struct Point { x: i32, y: i32 }

        fn describe_point(point: Point) -> String {
            match point {
                Point { x: 0, y: 0 } => "origin".to_string(),
                Point { x: 0, y } => format!("on y-axis at {}", y),
                Point { x, y: 0 } => format!("on x-axis at {}", x),
                Point { x, y } => format!("at ({}, {})", x, y),
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse match with struct patterns: {:?}", result.err());
}

/// Phase 10: Test match with tuple patterns
#[test]
fn test_match_with_tuple_patterns() {
    let code = r#"
        fn main() {
            let pair = (2, 3);
            match pair {
                (0, y) => println!("x is zero, y is {}", y),
                (x, 0) => println!("x is {}, y is zero", x),
                (x, y) => println!("x is {}, y is {}", x, y),
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse match with tuple patterns: {:?}", result.err());
}

/// Phase 10: Test nested match
#[test]
fn test_nested_match() {
    let code = r#"
        fn process(outer: Option<Result<i32, String>>) -> i32 {
            match outer {
                Some(inner) => match inner {
                    Ok(value) => value,
                    Err(_) => 0,
                },
                None => 0,
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse nested match: {:?}", result.err());
}

/// Phase 10: Test match with literal patterns
#[test]
fn test_match_with_literals() {
    let code = r#"
        fn main() {
            let number = 42;
            match number {
                0 => println!("zero"),
                1 => println!("one"),
                42 => println!("answer"),
                _ => println!("other"),
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse match with literals: {:?}", result.err());
}

/// Phase 10: Test match with range patterns
#[test]
fn test_match_with_range_patterns() {
    let code = r#"
        fn classify_age(age: i32) -> &'static str {
            match age {
                0..=17 => "minor",
                18..=64 => "adult",
                65.. => "senior",
                _ => "invalid",
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse match with range patterns: {:?}", result.err());
}

/// Phase 10: Test match as expression
#[test]
fn test_match_as_expression() {
    let code = r#"
        fn get_description(x: i32) -> String {
            let result = match x {
                n if n < 0 => format!("{} is negative", n),
                0 => "zero".to_string(),
                n => format!("{} is positive", n),
            };
            result
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse match as expression: {:?}", result.err());
}

/// Phase 10: Test match with wildcard
#[test]
fn test_match_with_wildcard() {
    let code = r#"
        fn process(opt: Option<i32>) -> i32 {
            match opt {
                Some(_) => 1,
                None => 0,
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse match with wildcard: {:?}", result.err());
}

/// Phase 10: Integration test - complex match
#[test]
fn test_complex_match() {
    let code = r#"
        enum Message {
            Quit,
            Move { x: i32, y: i32 },
            Write(String),
        }

        fn process_message(msg: Message) {
            match msg {
                Message::Quit => println!("Quit"),
                Message::Move { x, y } => println!("Move to ({}, {})", x, y),
                Message::Write(text) => println!("Text: {}", text),
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse complex match: {:?}", result.err());
}
