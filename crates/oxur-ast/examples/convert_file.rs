use oxur_ast::*;
use oxur_ast::integration::parse_rust_file;
use oxur_ast::sexp::print_sexp;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <input.rs> <output.sexp>", args[0]);
        std::process::exit(1);
    }

    let input = &args[1];
    let output = &args[2];

    println!("Converting: {} → {}\n", input, output);

    // Read input
    let source = fs::read_to_string(input)
        .expect("Failed to read input file");

    // Parse Rust
    println!("1. Parsing Rust...");
    let crate_node = parse_rust_file(&source)
        .expect("Failed to parse Rust");
    println!("   ✓ Parsed {} items", crate_node.items.len());

    // Generate S-expression
    println!("2. Generating S-expression...");
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node)
        .expect("Failed to generate S-expression");
    println!("   ✓ Generated");

    // Format and write
    println!("3. Writing output...");
    let sexp_text = print_sexp(&sexp);
    fs::write(output, sexp_text)
        .expect("Failed to write output file");
    println!("   ✓ Written");

    println!("\n✓ Conversion complete!");
}
