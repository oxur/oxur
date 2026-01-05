use oxur_ast::integration::parse_rust_file;

/// Phase 12: Test basic trait object
#[test]
fn test_trait_object_basic() {
    let code = r#"
        fn main() {
            let display: Box<dyn Display> = Box::new(42);
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse basic trait object: {:?}", result.err());
}

/// Phase 12: Test trait object with multiple bounds
#[test]
fn test_trait_object_multiple_bounds() {
    let code = r#"
        fn main() {
            let complex: Box<dyn Display + Debug + Send> = Box::new("hello");
        }
    "#;

    let result = parse_rust_file(code);
    assert!(
        result.is_ok(),
        "Failed to parse trait object with multiple bounds: {:?}",
        result.err()
    );
}

/// Phase 12: Test trait object reference
#[test]
fn test_trait_object_reference() {
    let code = r#"
        fn print_it(obj: &dyn Display) {
            println!("{}", obj);
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse trait object reference: {:?}", result.err());
}

/// Phase 12: Test trait object return type
#[test]
fn test_trait_object_return() {
    let code = r#"
        fn make_display() -> Box<dyn Display> {
            Box::new(42)
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse trait object return: {:?}", result.err());
}

/// Phase 12: Test trait object with lifetime
#[test]
fn test_trait_object_lifetime() {
    let code = r#"
        fn with_lifetime(obj: &(dyn Display + 'static)) {
            println!("{}", obj);
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse trait object with lifetime: {:?}", result.err());
}

/// Phase 12: Test trait object without dyn keyword (deprecated syntax)
#[test]
fn test_trait_object_no_dyn() {
    let code = r#"
        fn old_style(obj: Box<Display>) {
            println!("{}", obj);
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse trait object without dyn: {:?}", result.err());
}

/// Phase 12: Test dyn trait in vector
#[test]
fn test_trait_object_in_vec() {
    let code = r#"
        fn main() {
            let items: Vec<Box<dyn Display>> = vec![
                Box::new(42),
                Box::new("hello"),
            ];
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse trait object in vec: {:?}", result.err());
}

/// Phase 12: Test trait object with Send + Sync
#[test]
fn test_trait_object_send_sync() {
    let code = r#"
        fn thread_safe(obj: Box<dyn Send + Sync>) {
            // Can be sent across threads
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse trait object with Send + Sync: {:?}", result.err());
}

/// Phase 12: Test mutable trait object reference
#[test]
fn test_trait_object_mut_ref() {
    let code = r#"
        fn modify(obj: &mut dyn Write) {
            obj.write_all(b"data");
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse mutable trait object reference: {:?}", result.err());
}

/// Phase 12: Integration test - trait objects in structs
#[test]
fn test_trait_object_in_struct() {
    let code = r#"
        struct Container {
            handler: Box<dyn Fn() -> i32>,
            logger: Box<dyn Write + Send>,
        }

        fn create_container() -> Container {
            Container {
                handler: Box::new(|| 42),
                logger: Box::new(std::io::stdout()),
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse trait object in struct: {:?}", result.err());
}

/// Phase 12: Integration test - trait objects for polymorphism
#[test]
fn test_trait_object_polymorphism() {
    let code = r#"
        trait Shape {
            fn area(&self) -> f64;
        }

        fn total_area(shapes: Vec<Box<dyn Shape>>) -> f64 {
            shapes.iter().map(|s| s.area()).sum()
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse trait object polymorphism: {:?}", result.err());
}
