use oxur_ast::integration::parse_rust_file;

/// Phase 11: Test boolean literal - true
#[test]
fn test_bool_literal_true() {
    let code = r#"
        fn main() {
            let flag = true;
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse bool literal (true): {:?}", result.err());
}

/// Phase 11: Test boolean literal - false
#[test]
fn test_bool_literal_false() {
    let code = r#"
        fn main() {
            let flag = false;
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse bool literal (false): {:?}", result.err());
}

/// Phase 11: Test boolean in conditions
#[test]
fn test_bool_in_conditions() {
    let code = r#"
        fn main() {
            let enabled = true;
            if enabled {
                println!("enabled");
            }

            while false {
                break;
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse bool in conditions: {:?}", result.err());
}

/// Phase 11: Test float literal
#[test]
fn test_float_literal() {
    let code = r#"
        fn main() {
            let pi = 3.14159;
            let e = 2.71828;
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse float literal: {:?}", result.err());
}

/// Phase 11: Test float with scientific notation
#[test]
fn test_float_scientific_notation() {
    let code = r#"
        fn main() {
            let small = 1.5e-10;
            let large = 3.0e8;
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse float scientific notation: {:?}", result.err());
}

/// Phase 11: Test character literal
#[test]
fn test_char_literal() {
    let code = r#"
        fn main() {
            let letter = 'a';
            let digit = '5';
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse char literal: {:?}", result.err());
}

/// Phase 11: Test character escape sequences
#[test]
fn test_char_escapes() {
    let code = r#"
        fn main() {
            let newline = '\n';
            let tab = '\t';
            let quote = '\'';
            let backslash = '\\';
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse char escapes: {:?}", result.err());
}

/// Phase 11: Test unicode character
#[test]
fn test_char_unicode() {
    let code = r#"
        fn main() {
            let emoji = '😀';
            let heart = '❤';
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse unicode char: {:?}", result.err());
}

/// Phase 11: Test byte literal
#[test]
fn test_byte_literal() {
    let code = r#"
        fn main() {
            let byte_a = b'A';
            let byte_newline = b'\n';
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse byte literal: {:?}", result.err());
}

/// Phase 11: Test byte string literal
#[test]
fn test_byte_string_literal() {
    let code = r#"
        fn main() {
            let bytes = b"Hello, World!";
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse byte string literal: {:?}", result.err());
}

/// Phase 11: Test byte string with escapes
#[test]
fn test_byte_string_escapes() {
    let code = r#"
        fn main() {
            let bytes = b"Line 1\nLine 2\tTabbed";
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse byte string with escapes: {:?}", result.err());
}

/// Phase 11: Test C string literal
#[test]
fn test_c_string_literal() {
    let code = r#"
        fn main() {
            let cstr = c"Hello from C";
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse C string literal: {:?}", result.err());
}

/// Phase 11: Integration test - all number types
#[test]
fn test_all_number_types() {
    let code = r#"
        fn numeric_types() {
            let int = 42;
            let negative = -100;
            let hex = 0xFF;
            let octal = 0o755;
            let binary = 0b1010;
            let float = 3.14;
            let scientific = 1.5e-3;
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse all number types: {:?}", result.err());
}

/// Phase 11: Integration test - all string types
#[test]
fn test_all_string_types() {
    let code = r#"
        fn string_types() {
            let string = "regular string";
            let byte_str = b"byte string";
            let c_str = c"C string";
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse all string types: {:?}", result.err());
}

/// Phase 11: Integration test - all character types
#[test]
fn test_all_char_types() {
    let code = r#"
        fn char_types() {
            let ch = 'a';
            let unicode = '🦀';
            let byte_ch = b'X';
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse all char types: {:?}", result.err());
}

/// Phase 11: Integration test - literals in expressions
#[test]
fn test_literals_in_expressions() {
    let code = r#"
        fn calculate() -> f64 {
            let x = 3.14 * 2.0;
            let valid = true && false;
            let ch = if true { 'a' } else { 'b' };
            x + 1.0
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse literals in expressions: {:?}", result.err());
}

/// Phase 11: Integration test - literals in match patterns
#[test]
fn test_literals_in_match() {
    let code = r#"
        fn classify(ch: char) -> &'static str {
            match ch {
                'a' => "letter a",
                'b' => "letter b",
                '0' => "zero",
                _ => "other",
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse literals in match: {:?}", result.err());
}

/// Phase 11: Integration test - literals in arrays
#[test]
fn test_literals_in_arrays() {
    let code = r#"
        fn arrays() {
            let bools = [true, false, true];
            let floats = [1.0, 2.5, 3.14];
            let chars = ['a', 'b', 'c'];
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse literals in arrays: {:?}", result.err());
}

/// Phase 11: Integration test - real-world usage
#[test]
fn test_real_world_literals() {
    let code = r#"
        struct Config {
            enabled: bool,
            threshold: f64,
            separator: char,
        }

        fn create_config() -> Config {
            Config {
                enabled: true,
                threshold: 0.75,
                separator: ',',
            }
        }

        fn validate(config: &Config) -> bool {
            config.threshold > 0.0 && config.threshold < 1.0
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse real-world literals: {:?}", result.err());
}

/// Phase 11: Test all literal types together
#[test]
fn test_all_literal_types() {
    let code = r#"
        fn all_literals() {
            // Strings
            let s = "string";
            let bs = b"bytes";
            let cs = c"cstring";

            // Numbers
            let i = 42;
            let f = 3.14;

            // Booleans
            let t = true;
            let f = false;

            // Characters
            let c = 'x';
            let b = b'y';

            // Unicode
            let emoji = '🦀';
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse all literal types: {:?}", result.err());
}
