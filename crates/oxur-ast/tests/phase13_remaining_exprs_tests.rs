use oxur_ast::integration::parse_rust_file;

/// Phase 13: Test if-let expression
#[test]
fn test_if_let() {
    let code = r#"
        fn main() {
            if let Some(x) = option {
                println!("{}", x);
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse if-let: {:?}", result.err());
}

/// Phase 13: Test while-let expression
#[test]
fn test_while_let() {
    let code = r#"
        fn main() {
            while let Some(item) = iterator.next() {
                process(item);
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse while-let: {:?}", result.err());
}

/// Phase 13: Test if-let with else
#[test]
fn test_if_let_else() {
    let code = r#"
        fn main() {
            let result = if let Some(x) = option {
                x
            } else {
                0
            };
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse if-let with else: {:?}", result.err());
}

/// Phase 13: Test array repeat basic
#[test]
fn test_array_repeat() {
    let code = r#"
        fn main() {
            let zeros = [0; 100];
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse array repeat: {:?}", result.err());
}

/// Phase 13: Test array repeat with type
#[test]
fn test_array_repeat_typed() {
    let code = r#"
        fn main() {
            let buffer = [0u8; 1024];
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse array repeat typed: {:?}", result.err());
}

/// Phase 13: Test 2D array repeat
#[test]
fn test_array_repeat_2d() {
    let code = r#"
        fn main() {
            let grid = [[0; 10]; 10];
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse 2D array repeat: {:?}", result.err());
}

/// Phase 13: Test unsafe block basic
#[test]
fn test_unsafe_block() {
    let code = r#"
        fn main() {
            unsafe {
                *ptr = 42;
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse unsafe block: {:?}", result.err());
}

/// Phase 13: Test unsafe block with return value
#[test]
fn test_unsafe_block_return() {
    let code = r#"
        fn main() {
            let value = unsafe { *raw_pointer };
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse unsafe block return: {:?}", result.err());
}

/// Phase 13: Test unsafe block with multiple statements
#[test]
fn test_unsafe_block_multi_stmt() {
    let code = r#"
        fn main() {
            unsafe {
                let ptr = raw_pointer as *mut i32;
                *ptr = 42;
                let value = *ptr;
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse unsafe block multi stmt: {:?}", result.err());
}

/// Phase 13: Test const block
#[test]
fn test_const_block() {
    let code = r#"
        const VALUE: i32 = const { 2 + 2 };
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse const block: {:?}", result.err());
}

/// Phase 13: Test const block in expression
#[test]
fn test_const_block_expr() {
    let code = r#"
        fn main() {
            let x = const { 10 * 10 };
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse const block expr: {:?}", result.err());
}

/// Phase 13: Integration test - let in nested contexts
#[test]
fn test_let_nested() {
    let code = r#"
        fn main() {
            if let Some(x) = option1 {
                if let Some(y) = option2 {
                    println!("{} {}", x, y);
                }
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse nested let: {:?}", result.err());
}

/// Phase 13: Integration test - array repeat in structs
#[test]
fn test_array_repeat_in_struct() {
    let code = r#"
        struct Buffer {
            data: [u8; 1024],
        }

        fn create_buffer() -> Buffer {
            Buffer {
                data: [0; 1024],
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse array repeat in struct: {:?}", result.err());
}

/// Phase 13: Integration test - unsafe with raw pointers
#[test]
fn test_unsafe_raw_pointers() {
    let code = r#"
        fn manipulate_raw(ptr: *mut i32) -> i32 {
            unsafe {
                *ptr = 42;
                *ptr
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse unsafe with raw pointers: {:?}", result.err());
}

/// Phase 13: Integration test - all modern features together
#[test]
fn test_all_modern_features() {
    let code = r#"
        async fn process_buffer() -> Result<Vec<u8>, ()> {
            let buffer = [0u8; 1024];

            if let Some(data) = fetch_data().await {
                let result = unsafe {
                    process_raw(buffer.as_ptr())
                };

                Ok(result.to_vec())
            } else {
                Ok(Vec::new())
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse all modern features: {:?}", result.err());
}
