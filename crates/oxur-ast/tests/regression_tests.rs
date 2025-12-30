use oxur_ast::integration::parse_rust_file;
use std::fs;
use std::path::Path;

#[test]
fn test_all_fixtures() {
    let fixtures = [
        "tests/fixtures/hello_world.rs",
        "tests/fixtures/simple_fn.rs",
        "tests/fixtures/let_bindings.rs",
    ];

    for fixture in &fixtures {
        let path = Path::new(fixture);
        if !path.exists() {
            eprintln!("Skipping missing fixture: {}", fixture);
            continue;
        }

        let source =
            fs::read_to_string(fixture).unwrap_or_else(|_| panic!("Failed to read {}", fixture));

        let result = parse_rust_file(&source);

        match result {
            Ok(crate_node) => {
                println!("✓ Parsed {}: {} items", fixture, crate_node.items.len());
            }
            Err(e) => {
                panic!("Failed to parse {}: {:?}", fixture, e);
            }
        }
    }
}
