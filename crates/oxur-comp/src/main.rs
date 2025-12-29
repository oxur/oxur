//! oxurc - Oxur Compiler Binary
//!
//! Main entry point for the Oxur compiler.

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "oxurc")]
#[command(about = "Oxur compiler - compiles Oxur code to native binaries", long_about = None)]
struct Cli {
    /// Input Oxur source file
    #[arg(value_name = "FILE")]
    input: PathBuf,

    /// Output binary path
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Output directory for intermediate files
    #[arg(long, default_value = ".oxur-build")]
    build_dir: PathBuf,

    /// Emit generated Rust source (don't delete)
    #[arg(long)]
    emit_rust: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        println!("Compiling: {}", cli.input.display());
    }

    // Placeholder: Read input file
    let source = std::fs::read_to_string(&cli.input)?;

    if cli.verbose {
        println!("Source length: {} bytes", source.len());
    }

    // Parse
    if cli.verbose {
        println!("Stage 1: Parsing...");
    }
    let mut parser = oxur_lang::Parser::new(source);
    let surface_forms = parser.parse()?;

    // Expand
    if cli.verbose {
        println!("Stage 2: Expanding macros...");
    }
    let mut expander = oxur_lang::Expander::new();
    let core_forms = expander.expand(surface_forms)?;

    // Compile
    if cli.verbose {
        println!("Stage 3-5: Lowering, generating, and compiling...");
    }
    let output = cli.output.unwrap_or_else(|| cli.input.with_extension(""));

    let mut compiler = oxur_comp::Compiler::new(cli.build_dir.clone());
    compiler.compile(core_forms, &output)?;

    if cli.verbose {
        println!("Successfully compiled to: {}", output.display());
    }

    // Clean up build directory unless --emit-rust
    if !cli.emit_rust && cli.build_dir.exists() {
        std::fs::remove_dir_all(&cli.build_dir)?;
    }

    Ok(())
}
