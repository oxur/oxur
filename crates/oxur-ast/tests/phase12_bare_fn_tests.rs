use oxur_ast::integration::parse_rust_file;

/// Phase 12: Test simple function pointer
#[test]
fn test_function_pointer_simple() {
    let code = r#"
        type Callback = fn(i32) -> bool;

        fn main() {
            let cb: Callback = |x| x > 0;
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse simple function pointer: {:?}", result.err());
}

/// Phase 12: Test function pointer with named parameters
#[test]
fn test_function_pointer_named_params() {
    let code = r#"
        type Operation = fn(x: i32, y: i32) -> i32;
    "#;

    let result = parse_rust_file(code);
    assert!(
        result.is_ok(),
        "Failed to parse function pointer with named params: {:?}",
        result.err()
    );
}

/// Phase 12: Test unsafe function pointer
#[test]
fn test_unsafe_function_pointer() {
    let code = r#"
        type UnsafeCallback = unsafe fn(*const u8) -> i32;
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse unsafe function pointer: {:?}", result.err());
}

/// Phase 12: Test FFI function pointer
#[test]
fn test_ffi_function_pointer() {
    let code = r#"
        type CCallback = extern "C" fn(i32) -> i32;
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse FFI function pointer: {:?}", result.err());
}

/// Phase 12: Test function pointer with no return
#[test]
fn test_function_pointer_no_return() {
    let code = r#"
        type VoidCallback = fn(i32);
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse function pointer with no return: {:?}", result.err());
}

/// Phase 12: Test function pointer with no parameters
#[test]
fn test_function_pointer_no_params() {
    let code = r#"
        type Generator = fn() -> i32;
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse function pointer with no params: {:?}", result.err());
}

/// Phase 12: Test function pointer in struct field
#[test]
fn test_function_pointer_in_struct() {
    let code = r#"
        struct Handler {
            callback: fn(i32) -> bool,
            on_error: fn(&str),
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse function pointer in struct: {:?}", result.err());
}

/// Phase 12: Test function pointer as parameter
#[test]
fn test_function_pointer_as_param() {
    let code = r#"
        fn apply(f: fn(i32) -> i32, x: i32) -> i32 {
            f(x)
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse function pointer as param: {:?}", result.err());
}

/// Phase 12: Test function pointer as return type
#[test]
fn test_function_pointer_as_return() {
    let code = r#"
        fn get_handler() -> fn(i32) -> bool {
            |x| x > 0
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse function pointer as return: {:?}", result.err());
}

/// Phase 12: Test function pointer with multiple parameters
#[test]
fn test_function_pointer_multiple_params() {
    let code = r#"
        type Complex = fn(i32, &str, bool) -> Result<i32, String>;
    "#;

    let result = parse_rust_file(code);
    assert!(
        result.is_ok(),
        "Failed to parse function pointer with multiple params: {:?}",
        result.err()
    );
}

/// Phase 12: Test extern "system" ABI
#[test]
fn test_function_pointer_system_abi() {
    let code = r#"
        type SystemCallback = extern "system" fn() -> i32;
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse function pointer with system ABI: {:?}", result.err());
}

/// Phase 12: Test static function pointer
#[test]
fn test_static_function_pointer() {
    let code = r#"
        static OPERATION: fn(i32, i32) -> i32 = |a, b| a + b;
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse static function pointer: {:?}", result.err());
}

/// Phase 12: Integration test - callback pattern
#[test]
fn test_callback_pattern() {
    let code = r#"
        struct EventHandler {
            on_click: fn(),
            on_hover: fn(i32, i32),
            on_key: fn(char) -> bool,
        }

        impl EventHandler {
            fn new() -> Self {
                EventHandler {
                    on_click: default_click,
                    on_hover: default_hover,
                    on_key: default_key,
                }
            }
        }

        fn default_click() {}
        fn default_hover(x: i32, y: i32) {}
        fn default_key(c: char) -> bool { true }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse callback pattern: {:?}", result.err());
}

/// Phase 12: Integration test - function pointer array
#[test]
fn test_function_pointer_array() {
    let code = r#"
        type OpCode = fn(i32, i32) -> i32;

        const OPS: [OpCode; 4] = [
            |a, b| a + b,
            |a, b| a - b,
            |a, b| a * b,
            |a, b| a / b,
        ];
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse function pointer array: {:?}", result.err());
}

/// Phase 12: Test parenthesized type
#[test]
fn test_parenthesized_type() {
    let code = r#"
        type Wrapped = (Box<dyn Display>);
        type Nested = ((i32));
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse parenthesized type: {:?}", result.err());
}
