use crate::integration::parse_rust_file;
use crate::sexp::print_sexp;
use crate::Generator;
use anyhow::Result;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

pub fn execute(input: PathBuf, output: Option<PathBuf>, compact: bool) -> Result<()> {
    // Read input
    let source = if input.to_str() == Some("-") {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        fs::read_to_string(&input)?
    };

    // Parse Rust
    let crate_node = parse_rust_file(&source)?;

    // Generate S-expression
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node)?;

    // Format
    // Note: compact mode is not yet implemented, using pretty print for now
    let output_text = if compact {
        // TODO: Implement compact printing in Phase 4
        print_sexp(&sexp)
    } else {
        print_sexp(&sexp)
    };

    // Write output
    if let Some(output_path) = output {
        if output_path.to_str() == Some("-") {
            println!("{}", output_text);
        } else {
            fs::write(output_path, output_text)?;
        }
    } else {
        println!("{}", output_text);
    }

    Ok(())
}
