//! oxur - Oxur CLI Tool
//!
//! Main command-line interface for Oxur projects.

#[cfg(feature = "binary")]
use anyhow::Result;
#[cfg(feature = "binary")]
use clap::{Parser, Subcommand};
#[cfg(feature = "binary")]
use oxur_cli::common::output;
#[cfg(feature = "binary")]
use std::path::PathBuf;

#[cfg(feature = "binary")]
#[derive(Parser)]
#[command(name = "oxur")]
#[command(about = "Oxur - A Lisp that compiles to Rust", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[cfg(feature = "binary")]
#[derive(Subcommand)]
enum Commands {
    /// Compile an Oxur file to binary
    Compile {
        /// Input Oxur source file
        input: PathBuf,

        /// Output binary path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Run an Oxur file (compile and execute)
    Run {
        /// Input Oxur source file
        input: PathBuf,

        /// Arguments to pass to the program
        args: Vec<String>,
    },

    /// Start the interactive REPL
    Repl,

    /// Create a new Oxur project
    New {
        /// Project name
        name: String,
    },

    /// Build the current project
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },

    /// Run tests
    Test,
}

#[cfg(feature = "binary")]
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile { input, output } => {
            output::info(&format!("Compiling: {}", input.display()));

            // Read source
            let source = std::fs::read_to_string(&input)?;

            // Parse and expand
            let mut parser = oxur_lang::Parser::new(source);
            let surface_forms = parser.parse()?;

            let mut expander = oxur_lang::Expander::new();
            let core_forms = expander.expand(surface_forms)?;

            // Compile
            let output = output.unwrap_or_else(|| input.with_extension(""));
            let build_dir = PathBuf::from(".oxur-build");

            let mut compiler = oxur_comp::Compiler::new(build_dir);
            compiler.compile(core_forms, &output)?;

            output::success(&format!("Compiled successfully: {}", output.display()));
        }

        Commands::Run { input, args } => {
            output::info(&format!("Running: {}", input.display()));

            // Would compile and execute
            if !args.is_empty() {
                output::info(&format!("With args: {:?}", args));
            }

            output::warning("Not yet implemented");
        }

        Commands::Repl => {
            output::info("Starting REPL...");
            let mut client = oxur_repl::ReplClient::new();
            client.run()?;
        }

        Commands::New { name } => {
            output::info(&format!("Creating new project: {}", name));

            // Would create project directory structure
            let project_dir = PathBuf::from(&name);
            std::fs::create_dir_all(&project_dir)?;

            output::success(&format!("Created project directory: {}", project_dir.display()));
            output::warning("Not yet fully implemented");
        }

        Commands::Build { release } => {
            output::info("Building project...");
            if release {
                output::info("Release mode enabled");
            }
            output::warning("Not yet implemented");
        }

        Commands::Test => {
            output::info("Running tests...");
            output::warning("Not yet implemented");
        }
    }

    Ok(())
}

#[cfg(not(feature = "binary"))]
fn main() {
    eprintln!("Error: The oxur binary must be built with the 'binary' feature enabled");
    eprintln!("Use: cargo build --bin oxur --features binary");
    std::process::exit(1);
}
