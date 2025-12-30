use crate::sexp::Parser;
use crate::AstBuilder;
use anyhow::Result;
use std::path::PathBuf;

pub fn execute(input: PathBuf, output: Option<PathBuf>) -> Result<()> {
    // Read input using common utility
    let sexp_text = oxur_cli::common::io::read_input(&input)?;

    // Parse S-expression
    let sexp = Parser::parse_str(&sexp_text)?;

    // Build AST
    let mut builder = AstBuilder::new();
    let crate_node = builder.build_crate(&sexp)?;

    // Generate Rust code (Phase 4: actual code generation)
    let rust_output = crate::codegen::generate_rust(&crate_node)?;

    // Write output using common utility
    oxur_cli::common::io::write_output(&rust_output, output.as_deref())?;

    Ok(())
}
