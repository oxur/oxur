use crate::integration::{generate_error_comment, parse_rust_file, parse_rust_file_partial};
use crate::sexp::print_sexp;
use crate::Generator;
use anyhow::Result;
use std::path::PathBuf;

pub fn execute(
    input: PathBuf,
    output: Option<PathBuf>,
    compact: bool,
    continue_after_error: bool,
) -> Result<()> {
    // Read input using common utility
    let source = oxur_cli::common::io::read_input(&input)?;

    let output_text = if continue_after_error {
        // Use partial conversion - collect errors as comments
        let (crate_node, error_comments) = parse_rust_file_partial(&source)?;

        // Generate S-expression for successful items
        let gen = Generator::new();
        let sexp = gen.generate_crate(&crate_node)?;
        let sexp_text = print_sexp(&sexp);

        // Combine error comments and S-expression output
        let mut output = String::new();

        // Add error comments at the top
        for error in &error_comments {
            output.push_str(&generate_error_comment(error));
            output.push_str("\n\n");
        }

        // Add the generated S-expression
        output.push_str(&sexp_text);

        output
    } else {
        // Existing behavior - fail on first error
        let crate_node = parse_rust_file(&source)?;

        // Generate S-expression
        let gen = Generator::new();
        let sexp = gen.generate_crate(&crate_node)?;

        // Format
        // Note: compact mode is not yet implemented, using pretty print for now
        if compact {
            // TODO: Implement compact printing in Phase 4
            print_sexp(&sexp)
        } else {
            print_sexp(&sexp)
        }
    };

    // Write output using common utility
    oxur_cli::common::io::write_output(&output_text, output.as_deref())?;

    Ok(())
}
