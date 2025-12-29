//! oxur - Oxur CLI Tool
//!
//! Main command-line interface for Oxur projects.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "oxur")]
#[command(about = "Oxur - A Lisp that compiles to Rust", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

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

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile { input, output } => {
            println!("Compiling: {}", input.display());

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

            println!("Compiled successfully: {}", output.display());
        }

        Commands::Run { input, args } => {
            println!("Running: {}", input.display());

            // Would compile and execute
            if !args.is_empty() {
                println!("With args: {:?}", args);
            }

            println!("(Not yet implemented)");
        }

        Commands::Repl => {
            println!("Starting REPL...");
            let mut client = oxur_repl::ReplClient::new();
            client.run()?;
        }

        Commands::New { name } => {
            println!("Creating new project: {}", name);

            // Would create project directory structure
            let project_dir = PathBuf::from(&name);
            std::fs::create_dir_all(&project_dir)?;

            println!("Created project directory: {}", project_dir.display());
            println!("(Not yet fully implemented)");
        }

        Commands::Build { release } => {
            println!("Building project...");
            if release {
                println!("Release mode enabled");
            }
            println!("(Not yet implemented)");
        }

        Commands::Test => {
            println!("Running tests...");
            println!("(Not yet implemented)");
        }
    }

    Ok(())
}
