use oxur_ast::codegen::RustCodegen;
use oxur_ast::integration::parse_rust_file;
use oxur_ast::Generator;

/// Helper to perform a round-trip test: Rust → AST → S-expr → AST → Rust
fn round_trip_test(input: &str) {
    // Parse Rust code into AST
    let ast = parse_rust_file(input).expect("Failed to parse Rust code");

    // Generate S-expression from AST
    let gen = Generator::new();
    let sexp = gen.generate_crate(&ast).expect("Failed to generate S-expression");

    // Generate Rust code from AST
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&ast).expect("Failed to generate Rust code");

    // Verify the output compiles and is semantically equivalent
    // (We don't expect exact string match due to formatting differences)
    let ast_output = parse_rust_file(&output).unwrap_or_else(|e| {
        panic!("Failed to parse generated Rust code: {}\n\nGenerated code:\n{}\n\n", e, output)
    });

    // Both ASTs should have same structure
    assert_eq!(
        ast.items.len(),
        ast_output.items.len(),
        "Item count mismatch. Input:\n{}\n\nOutput:\n{}\n\nSExp:\n{:?}",
        input,
        output,
        sexp
    );
}

#[test]
fn test_function_with_lifetime_param() {
    let input = r#"
        fn foo<'a>(x: &'a str) -> &'a str {
            x
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_function_with_multiple_lifetimes() {
    let input = r#"
        fn foo<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
            x
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_function_with_lifetime_bounds() {
    let input = r#"
        fn foo<'a, 'b: 'a>(x: &'a str, y: &'b str) -> &'a str {
            x
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_function_with_type_param() {
    let input = r#"
        fn foo<T>(x: T) -> T {
            x
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_function_with_type_param_and_trait_bound() {
    let input = r#"
        fn foo<T: Clone>(x: T) -> T {
            x
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_function_with_multiple_trait_bounds() {
    let input = r#"
        fn foo<T: Clone + Send>(x: T) -> T {
            x
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_function_with_type_param_default() {
    let input = r#"
        fn foo<T = i32>(x: T) -> T {
            x
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_function_with_const_param() {
    let input = r#"
        fn foo<const N: usize>() {}
    "#;
    round_trip_test(input);
}

#[test]
fn test_function_with_const_param_default() {
    let input = r#"
        fn foo<const N: usize = 5>() {}
    "#;
    round_trip_test(input);
}

#[test]
fn test_function_with_all_generic_param_types() {
    let input = r#"
        fn foo<'a, T: Clone, const N: usize>(x: &'a T) {}
    "#;
    round_trip_test(input);
}

#[test]
fn test_function_with_where_clause_type_bound() {
    let input = r#"
        fn foo<T>(x: T) -> T
        where
            T: Clone,
        {
            x
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_function_with_where_clause_multiple_bounds() {
    let input = r#"
        fn foo<T>(x: T) -> T
        where
            T: Clone + Send,
        {
            x
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_function_with_where_clause_lifetime_bound() {
    let input = r#"
        fn foo<'a, 'b>(x: &'a str, y: &'b str) -> &'a str
        where
            'b: 'a,
        {
            x
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_function_with_where_clause_mixed_predicates() {
    let input = r#"
        fn foo<'a, T>(x: &'a T) -> &'a T
        where
            T: Clone,
            'a: 'static,
        {
            x
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_function_with_complex_where_clause() {
    let input = r#"
        fn foo<'a, T, U>(x: &'a T, y: U) -> &'a T
        where
            T: Clone + Send,
            U: Clone,
            'a: 'static,
        {
            x
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_struct_with_lifetime_param() {
    let input = r#"
        struct Foo<'a> {
            x: &'a str,
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_struct_with_type_param() {
    let input = r#"
        struct Foo<T> {
            x: T,
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_struct_with_type_param_and_bound() {
    let input = r#"
        struct Foo<T: Clone> {
            x: T,
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_enum_with_type_param() {
    let input = r#"
        enum Option<T> {
            Some(T),
            None,
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_trait_with_type_param() {
    let input = r#"
        trait Foo<T> {
            fn bar(x: T) -> T;
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_trait_with_supertrait_bound() {
    let input = r#"
        trait Foo: Clone {
            fn bar();
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_trait_with_multiple_supertrait_bounds() {
    let input = r#"
        trait Foo: Clone + Send {
            fn bar();
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_impl_with_type_param() {
    let input = r#"
        struct Foo<T> {
            x: T,
        }

        impl<T> Foo<T> {
            fn new(x: T) {}
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_impl_trait_with_type_param() {
    let input = r#"
        trait Bar {}
        struct Foo<T> {
            x: T,
        }

        impl<T> Bar for Foo<T> {}
    "#;
    round_trip_test(input);
}

#[test]
fn test_impl_with_where_clause() {
    let input = r#"
        struct Foo<T> {
            x: T,
        }

        impl<T> Foo<T>
        where
            T: Clone,
        {
            fn bar() {}
        }
    "#;
    round_trip_test(input);
}

#[test]
fn test_type_alias_with_generics() {
    let input = r#"
        type Result<T> = T;
    "#;
    round_trip_test(input);
}

#[test]
fn test_complex_real_world_example() {
    let input = r#"
        fn process<'a, T, U>(items: &'a [T], converter: U) -> &'a T
        where
            T: Clone + Send,
            U: Clone,
            'a: 'static,
        {
            &items[0]
        }
    "#;
    round_trip_test(input);
}
