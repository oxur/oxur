/// Integration tests to improve generator coverage by parsing real Rust code
use oxur_ast::integration::parse_rust_file;
use oxur_ast::sexp::print_sexp;
use oxur_ast::Generator;

#[test]
fn test_generate_public_function() {
    let rust_code = "pub fn public_function() {}";

    let crate_node = parse_rust_file(rust_code).unwrap();
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node).unwrap();

    let output = print_sexp(&sexp);
    assert!(output.contains("Public"));
}

#[test]
fn test_generate_multiple_functions() {
    let rust_code = r#"
fn first() {}
fn second() {}
fn third() {}
"#;

    let crate_node = parse_rust_file(rust_code).unwrap();
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node).unwrap();

    let output = print_sexp(&sexp);
    assert!(output.contains("first"));
    assert!(output.contains("second"));
    assert!(output.contains("third"));
}

#[test]
fn test_generate_empty_function_body() {
    let rust_code = "fn empty_body() {}";

    let crate_node = parse_rust_file(rust_code).unwrap();
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node).unwrap();

    let output = print_sexp(&sexp);
    assert!(output.contains("empty_body"));
    assert!(output.contains("Block"));
}

#[test]
fn test_generate_unsafe_function() {
    let rust_code = "unsafe fn unsafe_op() {}";

    let crate_node = parse_rust_file(rust_code).unwrap();
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node).unwrap();

    let output = print_sexp(&sexp);
    assert!(output.contains("Unsafe"));
}

#[test]
fn test_generate_const_function() {
    let rust_code = "const fn const_eval() {}";

    let crate_node = parse_rust_file(rust_code).unwrap();
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node).unwrap();

    let output = print_sexp(&sexp);
    assert!(output.contains("Const"));
}

#[test]
fn test_generate_pub_unsafe_function() {
    let rust_code = "pub unsafe fn pub_unsafe() {}";

    let crate_node = parse_rust_file(rust_code).unwrap();
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node).unwrap();

    let output = print_sexp(&sexp);
    assert!(output.contains("Public"));
    assert!(output.contains("Unsafe"));
}

#[test]
fn test_generate_pub_const_function() {
    let rust_code = "pub const fn pub_const() {}";

    let crate_node = parse_rust_file(rust_code).unwrap();
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node).unwrap();

    let output = print_sexp(&sexp);
    assert!(output.contains("Public"));
    assert!(output.contains("Const"));
}

#[test]
fn test_generate_function_defaultness_final() {
    let rust_code = "fn final_impl() {}";

    let crate_node = parse_rust_file(rust_code).unwrap();
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node).unwrap();

    let output = print_sexp(&sexp);
    assert!(output.contains("Final"));
}

#[test]
fn test_generate_multiple_visibility_variants() {
    let rust_code = r#"
fn private() {}
pub fn public() {}
"#;

    let crate_node = parse_rust_file(rust_code).unwrap();
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node).unwrap();

    let output = print_sexp(&sexp);
    assert!(output.contains("Inherited"));
    assert!(output.contains("Public"));
}

#[test]
fn test_generate_extern_variants() {
    let rust_code = r#"
fn no_extern() {}
"#;

    let crate_node = parse_rust_file(rust_code).unwrap();
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node).unwrap();

    let output = print_sexp(&sexp);
    assert!(output.contains("None") || !output.contains("Extern"));
}
