use oxur_ast::builder::AstBuilder;
use oxur_ast::sexp::Parser;
use std::path::PathBuf;

fn main() {
    println!("=== Building Crate AST from S-expression ===\n");

    // Get path to test data
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let test_data = PathBuf::from(manifest_dir).join("test-data");

    // Example 1: Build from empty crate fixture
    println!("1. Building from empty crate fixture");
    let empty_crate = test_data.join("fixtures/crate/empty.sexp");

    match Parser::parse_file(&empty_crate) {
        Ok(sexp) => {
            println!("   ✓ Parsed S-expression from file");

            let mut builder = AstBuilder::new();
            match builder.build_crate(&sexp) {
                Ok(crate_ast) => {
                    println!("   ✓ Successfully built Crate AST!");
                    println!("      Items: {}", crate_ast.items.len());
                    println!("      ID: {:?}", crate_ast.id);
                }
                Err(e) => {
                    eprintln!("   Builder error: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("   Parse error: {}", e);
        }
    }

    // Example 2: Build from crate with items
    println!("\n2. Building from crate with one item");
    let crate_with_items = test_data.join("fixtures/crate/with-one-item.sexp");

    match Parser::parse_file(&crate_with_items) {
        Ok(sexp) => {
            println!("   ✓ Parsed S-expression from file");

            let mut builder = AstBuilder::new();
            match builder.build_crate(&sexp) {
                Ok(crate_ast) => {
                    println!("   ✓ Successfully built Crate AST!");
                    println!("      Items: {}", crate_ast.items.len());
                    println!("      ID: {:?}", crate_ast.id);
                    if !crate_ast.items.is_empty() {
                        println!("      First item: {:?}", crate_ast.items[0].ident.name);
                    }
                }
                Err(e) => {
                    eprintln!("   Builder error: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("   Parse error: {}", e);
        }
    }

    println!("\n=== Examples completed! ===");
}
