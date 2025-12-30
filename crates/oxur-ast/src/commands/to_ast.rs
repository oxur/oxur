use crate::integration::parse_rust_file;
use crate::sexp::print_sexp;
use crate::Generator;
use anyhow::Result;
use std::path::PathBuf;

pub fn execute(input: PathBuf, output: Option<PathBuf>, compact: bool) -> Result<()> {
    // Read input using common utility
    let source = oxur_cli::common::io::read_input(&input)?;

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

    // Write output using common utility
    oxur_cli::common::io::write_output(&output_text, output.as_deref())?;

    Ok(())
}
