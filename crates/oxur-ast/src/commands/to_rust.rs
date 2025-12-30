use crate::sexp::Parser;
use crate::AstBuilder;
use anyhow::Result;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

pub fn execute(input: PathBuf, output: Option<PathBuf>) -> Result<()> {
    // Read input
    let sexp_text = if input.to_str() == Some("-") {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        fs::read_to_string(&input)?
    };

    // Parse S-expression
    let sexp = Parser::parse_str(&sexp_text)?;

    // Build AST
    let mut builder = AstBuilder::new();
    let crate_node = builder.build_crate(&sexp)?;

    // Generate Rust (Phase 3: simplified - just Debug output)
    // Proper Rust code generation will be implemented in Phase 4+
    let rust_output = format!("// Generated from S-expression\n// AST: {:#?}", crate_node);

    // Write output
    if let Some(output_path) = output {
        if output_path.to_str() == Some("-") {
            println!("{}", rust_output);
        } else {
            fs::write(output_path, rust_output)?;
        }
    } else {
        println!("{}", rust_output);
    }

    Ok(())
}
