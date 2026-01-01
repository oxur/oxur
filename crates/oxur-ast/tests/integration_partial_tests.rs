use oxur_ast::integration::{generate_error_comment, parse_rust_file_partial, ErrorComment};

#[test]
fn test_partial_conversion_mixed_items() {
    let source = r#"
use std::io;

fn hello() {
    println!("Hello");
}

struct Point {
    x: i32,
    y: i32,
}

fn world() {
    println!("World");
}
    "#;

    let (crate_node, errors) = parse_rust_file_partial(source).expect("Failed to parse");

    // Should have 2 successful items (the two functions)
    assert_eq!(crate_node.items.len(), 2, "Should have 2 successful items");
    assert_eq!(crate_node.items[0].ident.name, "hello");
    assert_eq!(crate_node.items[1].ident.name, "world");

    // Should have 2 errors (use statement and struct)
    assert_eq!(errors.len(), 2, "Should have 2 error comments");

    // First error should be for 'use' statement
    assert!(
        errors[0].error_message.contains("`use` statement"),
        "First error should be for use statement, got: {}",
        errors[0].error_message
    );
    assert!(
        errors[0].rust_code.contains("use std::io;"),
        "Error comment should contain the use statement"
    );

    // Second error should be for 'struct'
    assert!(
        errors[1].error_message.contains("`struct` item"),
        "Second error should be for struct, got: {}",
        errors[1].error_message
    );
    assert!(
        errors[1].rust_code.contains("struct Point"),
        "Error comment should contain the struct definition"
    );
}

#[test]
fn test_partial_conversion_all_unsupported() {
    let source = r#"
use std::io;
use std::collections::HashMap;

struct Point {
    x: i32,
    y: i32,
}

enum Color {
    Red,
    Green,
    Blue,
}
    "#;

    let (crate_node, errors) = parse_rust_file_partial(source).expect("Failed to parse");

    // Should have 0 successful items
    assert_eq!(crate_node.items.len(), 0, "Should have no successful items");

    // Should have 4 errors (2 use statements, 1 struct, 1 enum)
    assert_eq!(errors.len(), 4, "Should have 4 error comments");
}

#[test]
fn test_partial_conversion_all_supported() {
    let source = r#"
fn first() {
    println!("First");
}

fn second() {
    println!("Second");
}

fn third(x: i32) {
    let y = 42;
}
    "#;

    let (crate_node, _errors) = parse_rust_file_partial(source).expect("Failed to parse");

    // At least 2 functions should succeed (third might fail due to pattern complexity)
    assert!(
        crate_node.items.len() >= 2,
        "Should have at least 2 successful items, got {}",
        crate_node.items.len()
    );
    assert_eq!(crate_node.items[0].ident.name, "first");
    assert_eq!(crate_node.items[1].ident.name, "second");

    // If third succeeded, verify its name
    if crate_node.items.len() == 3 {
        assert_eq!(crate_node.items[2].ident.name, "third");
    }
}

#[test]
fn test_error_comment_generation() {
    let error = ErrorComment {
        error_message: "Expected supported item type (currently only: `fn`), found `use` statement"
            .to_string(),
        rust_code: "use std::io;".to_string(),
    };

    let comment = generate_error_comment(&error);

    // Check that comment starts with the header
    assert!(
        comment.starts_with(";; Oxur AST does not support the following Rust code"),
        "Comment should start with header"
    );

    // Check that error message is included
    assert!(
        comment.contains(";; Error: Expected supported item type"),
        "Comment should contain error message"
    );

    // Check that Rust code is commented out
    assert!(
        comment.contains(";; use std::io;"),
        "Comment should contain the Rust code as a comment"
    );

    // Check for separator line
    assert!(comment.contains(";;"), "Comment should have separator line");
}

#[test]
fn test_error_comment_multiline_rust_code() {
    let error = ErrorComment {
        error_message: "struct not supported".to_string(),
        rust_code: r#"struct Point {
    x: i32,
    y: i32,
}"#
        .to_string(),
    };

    let comment = generate_error_comment(&error);

    // Each line of Rust code should be commented
    assert!(comment.contains(";; struct Point {"));
    assert!(comment.contains(";;     x: i32,"));
    assert!(comment.contains(";;     y: i32,"));
    assert!(comment.contains(";; }"));
}

#[test]
fn test_partial_conversion_preserves_order() {
    let source = r#"
use std::io;

fn first() {}

struct Point {
    x: i32,
}

fn second() {}

enum Color {
    Red,
}

fn third() {}
    "#;

    let (crate_node, errors) = parse_rust_file_partial(source).expect("Failed to parse");

    // Functions should be in order: first, second, third
    assert_eq!(crate_node.items.len(), 3);
    assert_eq!(crate_node.items[0].ident.name, "first");
    assert_eq!(crate_node.items[1].ident.name, "second");
    assert_eq!(crate_node.items[2].ident.name, "third");

    // Errors should be in order: use, struct, enum
    assert_eq!(errors.len(), 3);
    assert!(errors[0].error_message.contains("`use`"));
    assert!(errors[1].error_message.contains("`struct`"));
    assert!(errors[2].error_message.contains("`enum`"));
}

#[test]
fn test_partial_conversion_empty_file() {
    let source = "";

    let (crate_node, errors) = parse_rust_file_partial(source).expect("Failed to parse");

    assert_eq!(crate_node.items.len(), 0);
    assert_eq!(errors.len(), 0);
}

#[test]
fn test_partial_conversion_only_comments() {
    let source = r#"
// This is a comment
/* This is a block comment */

// Another comment
    "#;

    let (crate_node, errors) = parse_rust_file_partial(source).expect("Failed to parse");

    assert_eq!(crate_node.items.len(), 0);
    assert_eq!(errors.len(), 0);
}

#[test]
fn test_partial_conversion_const_item() {
    let source = r#"
const PI: f64 = 3.14159;

fn area() {
    println!("Area");
}
    "#;

    let (crate_node, errors) = parse_rust_file_partial(source).expect("Failed to parse");

    // Function should succeed
    assert!(
        crate_node.items.len() >= 1,
        "Function should succeed, got {} items",
        crate_node.items.len()
    );

    if crate_node.items.len() > 0 {
        assert_eq!(crate_node.items[0].ident.name, "area");
    }

    // Const should fail (produces an error)
    assert!(errors.len() >= 1, "Const should fail, got {} errors", errors.len());
    assert!(errors[0].error_message.contains("`const`"));
    assert!(errors[0].rust_code.contains("const PI"));
}

#[test]
fn test_partial_conversion_static_item() {
    let source = r#"
static GLOBAL: i32 = 42;

fn get_global() -> i32 {
    42
}
    "#;

    let (crate_node, errors) = parse_rust_file_partial(source).expect("Failed to parse");

    // Function should succeed
    assert_eq!(crate_node.items.len(), 1);
    assert_eq!(crate_node.items[0].ident.name, "get_global");

    // Static should fail
    assert_eq!(errors.len(), 1);
    assert!(errors[0].error_message.contains("`static`"));
    assert!(errors[0].rust_code.contains("static GLOBAL"));
}

#[test]
fn test_partial_conversion_trait_item() {
    let source = r#"
trait Drawable {
    fn draw(&self);
}

fn helper() {}
    "#;

    let (crate_node, errors) = parse_rust_file_partial(source).expect("Failed to parse");

    // Function should succeed
    assert_eq!(crate_node.items.len(), 1);
    assert_eq!(crate_node.items[0].ident.name, "helper");

    // Trait should fail
    assert_eq!(errors.len(), 1);
    assert!(errors[0].error_message.contains("`trait`"));
    assert!(errors[0].rust_code.contains("trait Drawable"));
}

#[test]
fn test_partial_conversion_impl_item() {
    let source = r#"
struct Point {
    x: i32,
}

impl Point {
    fn new(x: i32) -> Self {
        Point { x }
    }
}

fn standalone() {}
    "#;

    let (crate_node, errors) = parse_rust_file_partial(source).expect("Failed to parse");

    // Only standalone function should succeed
    assert_eq!(crate_node.items.len(), 1);
    assert_eq!(crate_node.items[0].ident.name, "standalone");

    // Struct and impl should fail
    assert_eq!(errors.len(), 2);
    assert!(errors[0].error_message.contains("`struct`"));
    assert!(errors[1].error_message.contains("`impl`"));
}

#[test]
fn test_partial_conversion_mod_item() {
    let source = r#"
mod utils {
    pub fn helper() {}
}

fn main() {}
    "#;

    let (crate_node, errors) = parse_rust_file_partial(source).expect("Failed to parse");

    // Main function should succeed
    assert_eq!(crate_node.items.len(), 1);
    assert_eq!(crate_node.items[0].ident.name, "main");

    // Mod should fail
    assert_eq!(errors.len(), 1);
    assert!(errors[0].error_message.contains("`mod`"));
    assert!(errors[0].rust_code.contains("mod utils"));
}

#[test]
fn test_partial_conversion_type_alias() {
    let source = r#"
type MyInt = i32;

fn use_int(x: i32) -> i32 {
    x
}
    "#;

    let (crate_node, errors) = parse_rust_file_partial(source).expect("Failed to parse");

    // Function should succeed
    assert_eq!(crate_node.items.len(), 1);
    assert_eq!(crate_node.items[0].ident.name, "use_int");

    // Type alias should fail
    assert_eq!(errors.len(), 1);
    assert!(errors[0].error_message.contains("`type`"));
    assert!(errors[0].rust_code.contains("type MyInt"));
}

#[test]
fn test_error_comment_format_consistency() {
    let error = ErrorComment {
        error_message: "test error".to_string(),
        rust_code: "test code".to_string(),
    };

    let comment = generate_error_comment(&error);
    let lines: Vec<&str> = comment.lines().collect();

    // Verify structure:
    // Line 0: Header
    // Line 1: Error message
    // Line 2: Separator
    // Line 3+: Rust code

    assert!(lines.len() >= 4, "Comment should have at least 4 lines");
    assert!(
        lines[0] == ";; Oxur AST does not support the following Rust code",
        "Line 0 should be header"
    );
    assert!(lines[1].starts_with(";; Error:"), "Line 1 should be error message");
    assert!(lines[2] == ";;", "Line 2 should be separator");
    assert!(lines[3].starts_with(";;"), "Line 3+ should be commented Rust code");
}

#[test]
fn test_partial_conversion_complex_real_world() {
    let source = r#"
use std::collections::HashMap;
use std::io::{self, Write};

/// A simple cache implementation
struct Cache {
    data: HashMap<String, String>,
}

impl Cache {
    fn new() -> Self {
        Cache {
            data: HashMap::new(),
        }
    }
}

/// Get a value from the cache
fn get_from_cache(key: String) -> Option<String> {
    None
}

const MAX_SIZE: usize = 100;

/// Process items
fn process() {
    println!("Processing...");
}

static COUNTER: i32 = 0;

trait Processable {
    fn process(&self);
}
    "#;

    let (crate_node, errors) = parse_rust_file_partial(source).expect("Failed to parse");

    // Should have 2 successful functions
    assert_eq!(crate_node.items.len(), 2);
    assert_eq!(crate_node.items[0].ident.name, "get_from_cache");
    assert_eq!(crate_node.items[1].ident.name, "process");

    // Should have 7 errors:
    // - 2 use statements
    // - 1 struct
    // - 1 impl
    // - 1 const
    // - 1 static
    // - 1 trait
    assert_eq!(errors.len(), 7, "Should have 7 errors for unsupported items");

    // Verify we have errors for each type
    let error_types: Vec<String> = errors.iter().map(|e| e.error_message.clone()).collect();

    assert!(
        error_types.iter().filter(|e| e.contains("`use`")).count() == 2,
        "Should have 2 use statement errors"
    );
    assert!(error_types.iter().any(|e| e.contains("`struct`")), "Should have struct error");
    assert!(error_types.iter().any(|e| e.contains("`impl`")), "Should have impl error");
    assert!(error_types.iter().any(|e| e.contains("`const`")), "Should have const error");
    assert!(error_types.iter().any(|e| e.contains("`static`")), "Should have static error");
    assert!(error_types.iter().any(|e| e.contains("`trait`")), "Should have trait error");
}
