use oxur_ast::integration::parse_rust_file;

#[test]
fn test_parse_hello_world() {
    let source = r#"
fn main() {
    println!("Hello, world!");
}
    "#;

    let crate_node = parse_rust_file(source).expect("Failed to parse");

    assert_eq!(crate_node.items.len(), 1);
    assert_eq!(crate_node.items[0].ident.name, "main");
}

#[test]
fn test_parse_simple_function() {
    let source = r#"
fn add(a: i32, b: i32) -> i32 {
    42
}
    "#;

    let crate_node = parse_rust_file(source).expect("Failed to parse");
    assert_eq!(crate_node.items.len(), 1);
    assert_eq!(crate_node.items[0].ident.name, "add");
}

#[test]
fn test_parse_function_with_let() {
    let source = r#"
fn test() {
    let x = 42;
    let y = "hello";
}
    "#;

    let crate_node = parse_rust_file(source).expect("Failed to parse");
    assert_eq!(crate_node.items.len(), 1);
}
