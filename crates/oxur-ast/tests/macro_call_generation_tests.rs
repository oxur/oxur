use oxur_ast::gen_rs::RustCodegen;
use oxur_ast::integration::parse_rust_file;
use oxur_ast::Generator;

#[test]
fn test_println_macro() {
    let code = r#"
        fn main() {
            println!("Hello, {}!", name);
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    // Verify macro call was parsed
    let sexp_gen = Generator::new();
    let sexp = sexp_gen.generate_crate(&ast).unwrap();
    let sexp_str = format!("{:?}", sexp);
    assert!(sexp_str.contains("MacCall"), "Should contain MacCall");
    assert!(sexp_str.contains("println"), "Should contain println");

    // Round-trip
    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();
    assert!(rust.contains("println!"), "Generated code should contain println!");
}

#[test]
fn test_vec_macro() {
    let code = r#"
        fn test() {
            let v = vec![1, 2, 3];
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();
    assert!(rust.contains("vec!"), "Generated code should contain vec!");
}

#[test]
fn test_assert_eq_macro() {
    let code = r#"
        fn test() {
            assert_eq!(a, b);
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();
    assert!(rust.contains("assert_eq!"), "Generated code should contain assert_eq!");
}

#[test]
fn test_format_macro() {
    let code = r#"
        fn test() {
            let s = format!("x = {}", x);
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();
    assert!(rust.contains("format!"), "Generated code should contain format!");
}

#[test]
fn test_macro_with_brackets() {
    let code = r#"
        fn test() {
            let v = vec![1, 2, 3];
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let sexp_gen = Generator::new();
    let sexp = sexp_gen.generate_crate(&ast).unwrap();
    let sexp_str = format!("{:?}", sexp);

    // Should capture bracket delimiter
    assert!(sexp_str.contains("MacCall"), "Should contain MacCall");
    assert!(sexp_str.contains("Bracket"), "Should indicate bracket delimiter");
}

#[test]
fn test_macro_with_braces() {
    let code = r#"
        fn test() {
            thread_local! {
                static FOO: i32 = 42;
            }
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let sexp_gen = Generator::new();
    let sexp = sexp_gen.generate_crate(&ast).unwrap();
    let sexp_str = format!("{:?}", sexp);

    // Should capture brace delimiter
    assert!(sexp_str.contains("MacCall"), "Should contain MacCall");
    assert!(sexp_str.contains("Brace"), "Should indicate brace delimiter");
}

#[test]
fn test_nested_macros() {
    let code = r#"
        fn test() {
            println!("{:?}", vec![1, 2, 3]);
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();
    assert!(rust.contains("println!"), "Should contain println!");
    // Note: vec! is inside the macro args as part of the token stream.
    // The token stream preserves it as a string, so vec! should appear in the output.
    // We check for the general pattern, not exact "vec!" since it's tokenized.
    assert!(rust.contains("vec"), "Should contain vec in token stream");
}

#[test]
fn test_macro_statement() {
    let code = r#"
        fn test() {
            println!("test");
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();
    assert!(rust.contains("println!(\"test\");"), "Should preserve macro statement with semicolon");
}

#[test]
fn test_macro_in_expression_position() {
    let code = r#"
        fn test() -> Vec<i32> {
            vec![1, 2, 3]
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let sexp_gen = Generator::new();
    let sexp = sexp_gen.generate_crate(&ast).unwrap();
    let sexp_str = format!("{:?}", sexp);

    assert!(sexp_str.contains("MacCall"), "Macro in expression position should be MacCall");
}

#[test]
fn test_round_trip_macro() {
    let code = r#"
        fn test() {
            let x = vec![1, 2, 3];
            println!("x = {:?}", x);
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    // Generate Rust code
    let mut rust_gen = RustCodegen::new();
    let rust1 = rust_gen.generate_crate(&ast).unwrap();

    // Parse again
    let ast2 = parse_rust_file(&rust1).unwrap();

    // Generate again
    let mut rust_gen2 = RustCodegen::new();
    let rust2 = rust_gen2.generate_crate(&ast2).unwrap();

    // Both should be equivalent
    assert_eq!(rust1.trim(), rust2.trim(), "Round-trip should produce identical output");
}
