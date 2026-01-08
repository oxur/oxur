//! oxur - Oxur CLI Tool
//!
//! Main command-line interface for Oxur projects.

#[cfg(feature = "binary")]
mod repl;

#[cfg(feature = "binary")]
use anyhow::Result;
#[cfg(feature = "binary")]
use clap::{Args, Parser, Subcommand};
#[cfg(feature = "binary")]
use oxur_cli::common::output;
#[cfg(feature = "binary")]
use oxur_cli::config::ReplConfig;
#[cfg(feature = "binary")]
use std::path::PathBuf;

#[cfg(feature = "binary")]
#[derive(Parser)]
#[command(name = "oxur")]
#[command(about = "Oxur - A Lisp that compiles to Rust", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
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

    /// Start the interactive REPL (default command)
    Repl(ReplArgs),

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

/// REPL command arguments
#[cfg(feature = "binary")]
#[derive(Args, Debug, Default, Clone)]
pub struct ReplArgs {
    /// Start the default built-in REPL server and connect to it
    /// with the built-in client. This is the default behavior.
    #[arg(short = 'i', long = "interactive")]
    pub interactive: bool,

    /// Connect to a running REPL server with the built-in client.
    /// If no address given, connects to 127.0.0.1:5099
    #[arg(short = 'c', long = "connect", value_name = "HOST:PORT")]
    pub connect: Option<Option<String>>,

    /// Disable ANSI colors in interactive or connect modes.
    #[arg(long = "no-color")]
    pub no_color: bool,

    /// Start a REPL server only (no client).
    /// Use a path for Unix socket, or HOST:PORT for TCP.
    #[arg(short = 's', long = "serve", value_name = "PATH|HOST:PORT")]
    pub serve: Option<String>,

    /// Acknowledge the port of this server to another nREPL server
    /// running on ACK-PORT. Used for tooling integration.
    #[arg(long = "ack", value_name = "ACK-PORT")]
    pub ack: Option<u16>,

    /// The transport module to use.
    /// Default: oxur_repl::transport::tcp
    #[arg(short = 't', long = "transport", value_name = "TRANSPORT")]
    pub transport: Option<String>,
}

#[cfg(feature = "binary")]
fn main() -> Result<()> {
    let cli = Cli::parse();

    // Default to REPL if no command given
    let command = cli.command.unwrap_or(Commands::Repl(ReplArgs::default()));

    match command {
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

        Commands::Repl(args) => {
            handle_repl(args)?;
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

/// Handle the REPL command with its various modes
#[cfg(feature = "binary")]
fn handle_repl(args: ReplArgs) -> Result<()> {
    // Load configuration with layered resolution:
    // defaults -> file -> env -> CLI args
    let config = ReplConfig::load(args.no_color)?;

    if let Some(addr) = args.serve {
        // Server mode: start ReplServer and listen
        run_server_mode(&addr, args.ack)
    } else if let Some(addr) = args.connect {
        // Connect mode: connect to existing server
        let addr = addr.unwrap_or_else(|| "127.0.0.1:5099".to_string());
        run_connect_mode(&addr, config)
    } else {
        // Interactive mode (default): in-memory server + client
        run_interactive_mode(config)
    }
}

/// Run the default interactive REPL mode
///
/// Creates an in-process server and client connected via channels,
/// providing the fastest possible REPL experience.
#[cfg(feature = "binary")]
fn run_interactive_mode(config: ReplConfig) -> Result<()> {
    // Create tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;

    // Run the interactive REPL
    rt.block_on(repl::interactive::run(config))
}

/// Run the REPL in server-only mode
///
/// Starts a REPL server listening on the specified address.
/// Use a path for Unix socket, or HOST:PORT for TCP.
#[cfg(feature = "binary")]
fn run_server_mode(addr: &str, ack_port: Option<u16>) -> Result<()> {
    output::info(&format!("Starting REPL server on {}...", addr));

    // Create tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;

    // Run the server
    rt.block_on(repl::server::run(addr, ack_port))
}

/// Run the REPL in connect mode
///
/// Connects to an existing REPL server and provides terminal interface.
#[cfg(feature = "binary")]
fn run_connect_mode(addr: &str, config: ReplConfig) -> Result<()> {
    output::info(&format!("Connecting to REPL server at {}...", addr));

    // Create tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;

    // Run the connect mode client
    rt.block_on(repl::connect::run(addr, config))
}

#[cfg(not(feature = "binary"))]
fn main() {
    eprintln!("Error: The oxur binary must be built with the 'binary' feature enabled");
    eprintln!("Use: cargo build --bin oxur --features binary");
    std::process::exit(1);
}
