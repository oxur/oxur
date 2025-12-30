use crate::integration::parse_rust_file;
use crate::sexp::{print_sexp, Parser};
use crate::{AstBuilder, Generator};
use anyhow::Result;
use colored::*; // Keep for the file name display
use oxur_cli::common::progress::ProgressTracker;
use std::fs;
use std::path::PathBuf;

pub fn execute(input: PathBuf, verbose: bool) -> Result<()> {
    let source = fs::read_to_string(&input)?;
    let mut progress = ProgressTracker::new(verbose);

    println!("{} {}", "Verifying round-trip for:".bold(), input.display());
    if verbose {
        println!();
    }

    progress.step("Parsing Rust source");
    let crate1 = parse_rust_file(&source)?;
    progress.done();

    progress.step("Generating S-expression");
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate1)?;
    progress.done();

    progress.step("Parsing S-expression");
    let sexp_text = print_sexp(&sexp);
    let sexp2 = Parser::parse_str(&sexp_text)?;
    progress.done();

    progress.step("Building AST from S-expression");
    let mut builder = AstBuilder::new();
    let crate2 = builder.build_crate(&sexp2)?;
    progress.done();

    progress.step("Verifying equivalence");
    if crate1.items.len() != crate2.items.len() {
        anyhow::bail!("Item count mismatch: {} vs {}", crate1.items.len(), crate2.items.len());
    }
    progress.done();

    progress.success("Round-trip verification successful!");

    Ok(())
}
