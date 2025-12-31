//! CLI argument parsing

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "aster")]
#[command(about = "AST manipulation and conversion tool", long_about = None)]
#[command(after_help = "Use 'aster <command> --help' for more information.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Convert Rust source to S-expression AST
    #[command(visible_alias = "ast")]
    ToAst {
        /// Input Rust file (or - for stdin)
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file (or - for stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Use compact formatting
        #[arg(short, long)]
        compact: bool,

        /// Continue processing after errors, generating comments for unsupported items
        #[arg(long)]
        continue_after_error: bool,
    },

    /// Convert S-expression to Rust source
    ToRust {
        /// Input S-expression file (or - for stdin)
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file (or - for stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Verify round-trip conversion
    Verify {
        /// Input Rust file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}
