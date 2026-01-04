use oxur_ast::ast::*;
use oxur_ast::integration::parse_rust_file;

/// Phase 10: Test basic for loop
#[test]
fn test_basic_for_loop() {
    let code = r#"
        fn main() {
            for i in 0..10 {
                println!("{}", i);
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse basic for loop: {:?}", result.err());
}

/// Phase 10: Test for loop with iterator
#[test]
fn test_for_loop_with_iterator() {
    let code = r#"
        fn main() {
            let items = vec![1, 2, 3];
            for item in items.iter() {
                println!("{}", item);
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse for loop with iterator: {:?}", result.err());
}

/// Phase 10: Test for loop with pattern destructuring
#[test]
fn test_for_loop_with_pattern() {
    let code = r#"
        fn main() {
            let pairs = vec![(1, 2), (3, 4)];
            for (x, y) in pairs {
                println!("x: {}, y: {}", x, y);
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse for loop with pattern: {:?}", result.err());
}

/// Phase 10: Test labeled for loop
#[test]
fn test_labeled_for_loop() {
    let code = r#"
        fn main() {
            'outer: for x in 0..10 {
                for y in 0..10 {
                    if x == y {
                        break 'outer;
                    }
                }
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse labeled for loop: {:?}", result.err());
}

/// Phase 10: Test infinite loop
#[test]
fn test_infinite_loop() {
    let code = r#"
        fn main() {
            let mut count = 0;
            loop {
                count += 1;
                if count > 10 {
                    break;
                }
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse infinite loop: {:?}", result.err());
}

/// Phase 10: Test loop with break value
#[test]
fn test_loop_with_break_value() {
    let code = r#"
        fn main() {
            let mut counter = 0;
            let result = loop {
                counter += 1;
                if counter == 10 {
                    break counter * 2;
                }
            };
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse loop with break value: {:?}", result.err());
}

/// Phase 10: Test nested loops
#[test]
fn test_nested_loops() {
    let code = r#"
        fn main() {
            'outer: loop {
                'inner: loop {
                    break 'outer;
                }
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse nested loops: {:?}", result.err());
}

/// Phase 10: Test while loop
#[test]
fn test_while_loop() {
    let code = r#"
        fn main() {
            let mut count = 0;
            while count < 10 {
                count += 1;
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse while loop: {:?}", result.err());
}

/// Phase 10: Test while loop with complex condition
#[test]
fn test_while_complex_condition() {
    let code = r#"
        fn main() {
            let mut attempts = 0;
            while attempts < 10 {
                attempts += 1;
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse while with complex condition: {:?}", result.err());
}

/// Phase 10: Test labeled while loop
#[test]
fn test_labeled_while_loop() {
    let code = r#"
        fn main() {
            let mut count = 0;
            'waiting: while count < 10 {
                count += 1;
                break 'waiting;
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse labeled while loop: {:?}", result.err());
}

/// Phase 10: Test loop with continue
#[test]
fn test_loop_with_continue() {
    let code = r#"
        fn main() {
            for i in 0..10 {
                if i % 2 == 0 {
                    continue;
                }
                println!("{}", i);
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse loop with continue: {:?}", result.err());
}

/// Phase 10: Integration test - real loop patterns
#[test]
fn test_real_loop_patterns() {
    let code = r#"
        fn process_items(items: Vec<i32>) {
            for item in items {
                match item {
                    x if x < 0 => continue,
                    x if x > 100 => break,
                    x => println!("{}", x),
                }
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse real loop patterns: {:?}", result.err());
}

/// Phase 10: Test all loop types together
#[test]
fn test_all_loop_types() {
    let code = r#"
        fn main() {
            // For loop
            for i in 0..5 {
                println!("{}", i);
            }

            // While loop
            let mut x = 0;
            while x < 5 {
                x += 1;
            }

            // Infinite loop
            let mut y = 0;
            loop {
                y += 1;
                if y >= 5 {
                    break;
                }
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse all loop types: {:?}", result.err());
}
