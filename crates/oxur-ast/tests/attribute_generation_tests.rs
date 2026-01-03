use oxur_ast::gen_rs::RustCodegen;
use oxur_ast::integration::parse_rust_file;
use oxur_ast::Generator;

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

#[test]
fn test_doc_comment_attribute() {
    let code = r#"
        /// This is a doc comment
        /// on multiple lines
        struct Documented {
            value: i32,
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    // Verify doc comment in S-expression
    let sexp_gen = Generator::new();
    let sexp = sexp_gen.generate_crate(&ast).unwrap();
    let sexp_str = format!("{:?}", sexp);

    assert!(sexp_str.contains("DocComment"), "S-expression should contain DocComment");

    // Verify doc comment in generated Rust
    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    // Doc comments are converted to #[doc = "..."] attributes
    assert!(rust.contains("#[doc"), "Generated code should contain doc attribute");
}

#[test]
fn test_inner_attribute() {
    let code = r#"
        fn main() {
            #![allow(unused_variables)]
            let x = 42;
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    // Verify inner attribute in S-expression
    let sexp_gen = Generator::new();
    let sexp = sexp_gen.generate_crate(&ast).unwrap();
    let sexp_str = format!("{:?}", sexp);

    assert!(sexp_str.contains("Inner"), "S-expression should contain Inner style");

    // Verify inner attribute in generated Rust
    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    assert!(rust.contains("#!"), "Generated code should contain inner attribute marker");
}

#[test]
fn test_attributes_on_const() {
    let code = r#"
        #[allow(dead_code)]
        const MAX_SIZE: usize = 100;
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    assert!(rust.contains("#[allow"), "Should contain attribute on const");
    assert!(rust.contains("const MAX_SIZE"), "Should contain const declaration");
}

#[test]
fn test_attributes_on_static() {
    let code = r#"
        #[used]
        static GLOBAL: i32 = 42;
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    assert!(rust.contains("#[used"), "Should contain attribute on static");
    assert!(rust.contains("static GLOBAL"), "Should contain static declaration");
}

#[test]
fn test_attributes_on_type_alias() {
    let code = r#"
        #[allow(dead_code)]
        type MyInt = i32;
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    assert!(rust.contains("#[allow"), "Should contain attribute on type alias");
    assert!(rust.contains("type MyInt"), "Should contain type alias");
}

#[test]
fn test_attributes_on_module() {
    let code = r#"
        #[cfg(test)]
        mod tests {
            fn test() {}
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    assert!(rust.contains("#[cfg"), "Should contain attribute on module");
    assert!(rust.contains("mod tests"), "Should contain module declaration");
}

#[test]
fn test_attributes_on_use() {
    let code = r#"
        #[allow(unused_imports)]
        use std::collections::HashMap;
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    assert!(rust.contains("#[allow"), "Should contain attribute on use");
    assert!(rust.contains("use std::collections::HashMap"), "Should contain use declaration");
}

#[test]
fn test_attributes_on_impl() {
    let code = r#"
        struct Point;

        #[allow(dead_code)]
        impl Point {
            fn new() -> Self {
                Point
            }
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    assert!(rust.contains("#[allow"), "Should contain attribute on impl");
    assert!(rust.contains("impl Point"), "Should contain impl block");
}

#[test]
fn test_attributes_on_trait() {
    let code = r#"
        #[allow(dead_code)]
        trait MyTrait {
            fn method(&self);
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    assert!(rust.contains("#[allow"), "Should contain attribute on trait");
    assert!(rust.contains("trait MyTrait"), "Should contain trait declaration");
}

#[test]
fn test_attributes_on_enum() {
    let code = r#"
        #[derive(Debug, Clone, Copy)]
        enum Color {
            Red,
            Green,
            Blue,
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let mut rust_gen = RustCodegen::new();
    let rust = rust_gen.generate_crate(&ast).unwrap();

    assert!(rust.contains("#[derive"), "Should contain attribute on enum");
    assert!(rust.contains("Debug"), "Should contain Debug trait");
    assert!(rust.contains("Clone"), "Should contain Clone trait");
    assert!(rust.contains("Copy"), "Should contain Copy trait");
}

#[test]
fn test_attribute_round_trip() {
    let code = r#"
        #[derive(Debug)]
        #[allow(dead_code)]
        struct Data {
            value: i32,
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
fn test_sexp_attribute_generation() {
    let code = r#"
        #[derive(Debug)]
        struct Point {
            x: i32,
        }
    "#;

    let ast = parse_rust_file(code).unwrap();

    let sexp_gen = Generator::new();
    let sexp = sexp_gen.generate_crate(&ast).unwrap();
    let sexp_str = format!("{:?}", sexp);

    // Verify S-expression contains attribute structures
    assert!(sexp_str.contains("Attribute"), "S-expression should contain Attribute node");
    assert!(sexp_str.contains("derive"), "S-expression should contain derive path");
    assert!(sexp_str.contains("Outer"), "S-expression should contain Outer style");
}
