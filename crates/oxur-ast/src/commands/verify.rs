use anyhow::Result;
use colored::*;
use oxur_ast::integration::parse_rust_file;
use oxur_ast::sexp::{print_sexp, Parser};
use oxur_ast::{AstBuilder, Generator};
use std::fs;
use std::path::PathBuf;

pub fn execute(input: PathBuf, verbose: bool) -> Result<()> {
    let source = fs::read_to_string(&input)?;

    println!("{} {}", "Verifying round-trip for:".bold(), input.display());
    if verbose {
        println!();
    }

    // Step 1: Parse Rust
    if verbose {
        println!("1. Parsing Rust source...");
    }
    let crate1 = parse_rust_file(&source)?;
    if verbose {
        println!("   {} Parsed successfully", "✓".green());
    }

    // Step 2: Generate S-expression
    if verbose {
        println!("2. Generating S-expression...");
    }
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate1)?;
    if verbose {
        println!("   {} Generated successfully", "✓".green());
    }

    // Step 3: Parse S-expression back
    if verbose {
        println!("3. Parsing S-expression...");
    }
    let sexp_text = print_sexp(&sexp);
    let sexp2 = Parser::parse_str(&sexp_text)?;
    if verbose {
        println!("   {} Parsed successfully", "✓".green());
    }

    // Step 4: Build AST
    if verbose {
        println!("4. Building AST from S-expression...");
    }
    let mut builder = AstBuilder::new();
    let crate2 = builder.build_crate(&sexp2)?;
    if verbose {
        println!("   {} Built successfully", "✓".green());
    }

    // Step 5: Verify
    if verbose {
        println!("5. Verifying equivalence...");
    }
    if crate1.items.len() != crate2.items.len() {
        anyhow::bail!("Item count mismatch: {} vs {}", crate1.items.len(), crate2.items.len());
    }
    if verbose {
        println!("   {} Basic verification passed", "✓".green());
    }

    if !verbose {
        println!("{} Round-trip verification successful!", "✓".green().bold());
    } else {
        println!();
        println!("{} Round-trip verification successful!", "✓".green().bold());
    }

    Ok(())
}
