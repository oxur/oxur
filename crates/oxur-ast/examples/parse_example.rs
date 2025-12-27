use oxur_ast::sexp::print_sexp;
use oxur_ast::sexp::Parser;
use std::path::PathBuf;

fn main() {
    println!("=== Parsing S-expressions ===\n");

    // Get path to test data
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let test_data = PathBuf::from(manifest_dir).join("test-data");

    // Example 1: Parse a simple crate from file
    println!("1. Parsing simple Crate from file");
    let simple_crate = test_data.join("examples/simple/empty-crate.sexp");

    match Parser::parse_file(&simple_crate) {
        Ok(sexp) => {
            println!("   ✓ Parsed successfully!");
            println!("   S-expression: {}", print_sexp(&sexp));
        }
        Err(e) => {
            eprintln!("   Parse error: {}", e);
        }
    }

    // Example 2: Parse a function from file
    println!("\n2. Parsing function definition from file");
    let simple_fn = test_data.join("examples/intermediate/simple-fn.sexp");

    match Parser::parse_file(&simple_fn) {
        Ok(sexp) => {
            println!("   ✓ Parsed complex S-expression successfully!");
            println!("   Result (first 200 chars):");
            let output = print_sexp(&sexp);
            println!("   {}", output.chars().take(200).collect::<String>());
            if output.len() > 200 {
                println!("   ... ({} more characters)", output.len() - 200);
            }
        }
        Err(e) => {
            eprintln!("\n   Parse error: {}", e);
        }
    }

    // Example 3: Also demonstrate parsing from strings (still supported)
    println!("\n3. Parsing from string (also supported)");
    let input = r#"(Symbol :name "example")"#;

    match Parser::parse_str(input) {
        Ok(sexp) => {
            println!("   ✓ Parsed from string: {}", print_sexp(&sexp));
        }
        Err(e) => {
            eprintln!("   Parse error: {}", e);
        }
    }
}
