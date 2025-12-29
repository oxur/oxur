use oxur_ast::integration::parse_rust_file;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <rust-file>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let source = fs::read_to_string(filename)
        .expect("Failed to read file");

    println!("Parsing: {}\n", filename);

    match parse_rust_file(&source) {
        Ok(crate_node) => {
            println!("✓ Parsed successfully!");
            println!("  Items: {}", crate_node.items.len());

            for (i, item) in crate_node.items.iter().enumerate() {
                println!("  Item {}: {}", i, item.ident.name);
            }
        }
        Err(e) => {
            eprintln!("✗ Parse error: {}", e);
            std::process::exit(1);
        }
    }
}
