use oxur_ast::gen_rs::RustCodegen;
use oxur_ast::integration::parse_rust_file;
use oxur_ast::Generator;

#[test]
fn test_simple_macro_rules() {
    let code = r#"
        macro_rules! say_hello {
            () => {
                println!("Hello!");
            }
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    // Verify S-expression contains MacroDef
    let sexp_gen = Generator::new();
    let sexp = sexp_gen.generate_crate(&ast).unwrap();
    let sexp_str = format!("{:?}", sexp);

    assert!(sexp_str.contains("MacroDef"), "S-expression should contain MacroDef");
    assert!(sexp_str.contains("say_hello"), "S-expression should contain macro name");

    // Verify Rust generation
    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    assert!(rust.contains("macro_rules!"), "Generated code should contain macro_rules!");
    assert!(rust.contains("say_hello"), "Generated code should contain macro name");
    assert!(rust.contains("println"), "Generated code should contain println in macro body");
}

#[test]
fn test_vec_macro_definition() {
    let code = r#"
        macro_rules! vec {
            ( $( $x:expr ),* ) => {
                {
                    let mut temp_vec = Vec::new();
                    $(
                        temp_vec.push($x);
                    )*
                    temp_vec
                }
            };
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    assert!(rust.contains("macro_rules! vec"), "Should contain macro_rules! vec");
    assert!(
        rust.contains("$") && rust.contains("x") && rust.contains("expr"),
        "Should contain macro pattern variables"
    );
}

#[test]
fn test_macro_with_multiple_patterns() {
    let code = r#"
        macro_rules! max {
            ($x:expr) => ($x);
            ($x:expr, $($y:expr),+) => {
                std::cmp::max($x, max!($($y),+))
            }
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    assert!(rust.contains("macro_rules! max"), "Should contain macro_rules! max");
    assert!(
        rust.contains("$") && rust.contains("x") && rust.contains("y") && rust.contains("expr"),
        "Should contain macro pattern variables"
    );
}

#[test]
fn test_macro_with_repetition() {
    let code = r#"
        macro_rules! create_function {
            ($func_name:ident) => {
                fn $func_name() {
                    println!("function called");
                }
            }
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    assert!(rust.contains("macro_rules! create_function"), "Should contain macro definition");
    assert!(
        rust.contains("$") && rust.contains("func_name") && rust.contains("ident"),
        "Should contain macro pattern variable"
    );
}

#[test]
fn test_macro_round_trip() {
    let code = r#"
        macro_rules! double {
            ($x:expr) => { $x * 2 }
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    // Generate Rust code
    let mut rust_gen = RustCodegen::new();
    let rust1 = rust_gen.generate_crate(&ast).unwrap();

    // Parse generated code
    let ast2 = parse_rust_file(&rust1).unwrap();

    // Generate again
    let mut rust_gen2 = RustCodegen::new();
    let rust2 = rust_gen2.generate_crate(&ast2).unwrap();

    // Both should be equivalent
    assert_eq!(rust1.trim(), rust2.trim(), "Round-trip should produce identical output");
}

#[test]
fn test_sexp_macro_def_generation() {
    let code = r#"
        macro_rules! my_macro {
            () => { 42 }
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let sexp_gen = Generator::new();
    let sexp = sexp_gen.generate_crate(&ast).unwrap();
    let sexp_str = format!("{:?}", sexp);

    // Verify S-expression structure
    assert!(sexp_str.contains("MacroDef"), "Should contain MacroDef node");
    assert!(sexp_str.contains("macro-rules"), "Should contain macro-rules field");
    assert!(sexp_str.contains("true"), "Should indicate this is macro_rules! (not macro)");
    assert!(sexp_str.contains("my_macro"), "Should contain macro name");
}

#[test]
fn test_macro_with_attributes() {
    let code = r#"
        #[allow(unused_macros)]
        macro_rules! test_macro {
            () => { }
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    assert!(rust.contains("#[allow"), "Should contain attribute");
    assert!(rust.contains("macro_rules! test_macro"), "Should contain macro definition");
}

#[test]
fn test_macro_with_doc_comment() {
    let code = r#"
        /// This is a test macro
        macro_rules! documented_macro {
            () => { }
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    // Verify doc comment is present in S-expression
    let sexp_gen = Generator::new();
    let sexp = sexp_gen.generate_crate(&ast).unwrap();
    let sexp_str = format!("{:?}", sexp);

    assert!(
        sexp_str.contains("DocComment") || sexp_str.contains("doc"),
        "Should contain doc comment"
    );

    // Verify in generated Rust
    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    assert!(rust.contains("#[doc"), "Generated code should contain doc attribute");
    assert!(rust.contains("macro_rules! documented_macro"), "Should contain macro definition");
}

#[test]
fn test_complex_macro_pattern() {
    let code = r#"
        macro_rules! hash_map {
            ($($key:expr => $val:expr),*) => {
                {
                    let mut map = std::collections::HashMap::new();
                    $(
                        map.insert($key, $val);
                    )*
                    map
                }
            };
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    assert!(rust.contains("macro_rules! hash_map"), "Should contain macro definition");
    assert!(
        rust.contains("$") && rust.contains("key") && rust.contains("expr"),
        "Should contain key pattern"
    );
    assert!(rust.contains("val") && rust.contains("expr"), "Should contain value pattern");
}

#[test]
fn test_macro_with_empty_body() {
    let code = r#"
        macro_rules! empty {
            () => {}
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    assert!(rust.contains("macro_rules! empty"), "Should contain macro definition");
}
