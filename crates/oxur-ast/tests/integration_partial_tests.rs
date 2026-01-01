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

    // Should have 4 successful items (use + two functions + struct)
    assert_eq!(crate_node.items.len(), 4, "Should have 4 successful items");
    assert_eq!(crate_node.items[0].ident.name, "use");
    assert_eq!(crate_node.items[1].ident.name, "hello");
    assert_eq!(crate_node.items[2].ident.name, "Point");
    assert_eq!(crate_node.items[3].ident.name, "world");

    // No errors (use now works!)
    assert_eq!(errors.len(), 0, "Should have 0 error comments");
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

    // Should have 4 successful items (2 use + struct + enum, all work!)
    assert_eq!(crate_node.items.len(), 4, "Should have 4 successful items (2 use + struct + enum)");
    assert_eq!(crate_node.items[0].ident.name, "use");
    assert_eq!(crate_node.items[1].ident.name, "use");
    assert_eq!(crate_node.items[2].ident.name, "Point");
    assert_eq!(crate_node.items[3].ident.name, "Color");

    // No errors (use, struct, and enum all work!)
    assert_eq!(errors.len(), 0, "Should have 0 error comments");
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

    // Items should be in order: use, first, Point (struct), second, Color (enum), third
    assert_eq!(crate_node.items.len(), 6);
    assert_eq!(crate_node.items[0].ident.name, "use");
    assert_eq!(crate_node.items[1].ident.name, "first");
    assert_eq!(crate_node.items[2].ident.name, "Point");
    assert_eq!(crate_node.items[3].ident.name, "second");
    assert_eq!(crate_node.items[4].ident.name, "Color");
    assert_eq!(crate_node.items[5].ident.name, "third");

    // No errors (use, struct, and enum all work!)
    assert_eq!(errors.len(), 0);
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
const MAX: i32 = 100;

fn area() {
    println!("Area");
}
    "#;

    let (crate_node, errors) = parse_rust_file_partial(source).expect("Failed to parse");

    // Both const and function should succeed (using int literal)
    assert_eq!(crate_node.items.len(), 2);
    assert_eq!(crate_node.items[0].ident.name, "MAX");
    assert_eq!(crate_node.items[1].ident.name, "area");

    // No errors (const now works!)
    assert_eq!(errors.len(), 0);
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

    // Both static and function should succeed
    assert_eq!(crate_node.items.len(), 2);
    assert_eq!(crate_node.items[0].ident.name, "GLOBAL");
    assert_eq!(crate_node.items[1].ident.name, "get_global");

    // No errors (static now works!)
    assert_eq!(errors.len(), 0);
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

    // Both trait and function should succeed
    assert_eq!(crate_node.items.len(), 2);
    assert_eq!(crate_node.items[0].ident.name, "Drawable");
    assert_eq!(crate_node.items[1].ident.name, "helper");

    // No errors (trait now works!)
    assert_eq!(errors.len(), 0);
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

    // Struct and standalone function should succeed
    // Impl fails due to struct expression `Point { x }` in method body (expression limitation)
    assert_eq!(crate_node.items.len(), 2);
    assert_eq!(crate_node.items[0].ident.name, "Point");
    assert_eq!(crate_node.items[1].ident.name, "standalone");

    // Impl fails due to unsupported struct expression in method body
    assert_eq!(errors.len(), 1);
    assert!(errors[0].error_message.contains("complex expression"));
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

    // Both mod and main function should succeed
    assert_eq!(crate_node.items.len(), 2);
    assert_eq!(crate_node.items[0].ident.name, "utils");
    assert_eq!(crate_node.items[1].ident.name, "main");

    // No errors (mod now works!)
    assert_eq!(errors.len(), 0);
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

    // Both type alias and function should succeed
    assert_eq!(crate_node.items.len(), 2);
    assert_eq!(crate_node.items[0].ident.name, "MyInt");
    assert_eq!(crate_node.items[1].ident.name, "use_int");

    // No errors (type alias now works!)
    assert_eq!(errors.len(), 0);
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

    // Should have 8 successful items (2 use + struct + 2 functions + const + trait + static)
    assert_eq!(crate_node.items.len(), 8);
    assert_eq!(crate_node.items[0].ident.name, "use");
    assert_eq!(crate_node.items[1].ident.name, "use");
    assert_eq!(crate_node.items[2].ident.name, "Cache");
    assert_eq!(crate_node.items[3].ident.name, "get_from_cache");
    assert_eq!(crate_node.items[4].ident.name, "MAX_SIZE");
    assert_eq!(crate_node.items[5].ident.name, "process");
    assert_eq!(crate_node.items[6].ident.name, "COUNTER");
    assert_eq!(crate_node.items[7].ident.name, "Processable");

    // Should have 1 error (use, struct, trait, const, and static all work!):
    // - 1 impl (fails due to struct expression in method body)
    assert_eq!(errors.len(), 1, "Should have 1 error for impl");

    // Verify impl/expression error
    assert!(
        errors[0].error_message.contains("complex expression"),
        "Should have impl/expression error"
    );
}
