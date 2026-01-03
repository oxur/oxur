use oxur_ast::gen_rs::RustCodegen;
use oxur_ast::integration::parse_rust_file;

#[test]
fn test_derive_attribute_on_struct() {
    let code = r#"
        #[derive(Debug, Clone)]
        struct Point {
            x: i32,
            y: i32,
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    // Should contain the derive attribute
    assert!(rust.contains("#[derive"), "Generated code should contain #[derive");
    assert!(rust.contains("Debug"), "Generated code should contain Debug");
    assert!(rust.contains("Clone"), "Generated code should contain Clone");
    assert!(rust.contains("struct Point"), "Generated code should contain struct Point");
}

#[test]
fn test_cfg_test_attribute() {
    let code = r#"
        #[cfg(test)]
        fn test_function() {}
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    assert!(rust.contains("#[cfg"), "Generated code should contain #[cfg");
    assert!(rust.contains("test"), "Generated code should contain test");
}

#[test]
fn test_multiple_attributes() {
    let code = r#"
        #[derive(Debug)]
        #[allow(dead_code)]
        struct Data {
            value: i32,
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    assert!(rust.contains("#[derive(Debug)]"), "Should contain derive attribute");
    assert!(rust.contains("#[allow"), "Should contain allow attribute");
}
